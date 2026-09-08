#!/usr/bin/env python3
"""Pause both memory adapters at named checkpoints, then inspect native VM state."""
import argparse
import fcntl
import json
import math
import os
from pathlib import Path
import selectors
import re
import signal
import subprocess
import sys
import time

import build
ROOT,TOOLS,OUTPUT=build.ROOT,build.TOOLS,build.OUTPUT
sys.path.insert(0,str(ROOT/'scripts'))
from s07_benchmark import CACHE, native_environment, go_native_environment, source_fingerprint
from s07_benchmark_report import read_capture
from s07_benchmark_measure import reject_concurrent_builds, host_info

CHECKPOINTS={'rust':['pre_pipeline','retained_endpoint','post_roots_retirement','post_retirement'],
             'go':['pre_pipeline','retained_endpoint','retained_after_gc','post_retirement','post_retirement_after_gc']}

def uint(value,name):
    if type(value) is not int or value<0: raise ValueError('invalid nonnegative integer: '+name)

def digest(value,name):
    if type(value) is not str or re.fullmatch('[0-9a-f]{64}',value) is None:
        raise ValueError('invalid SHA256: '+name)

def validate_build(manifest,runtimes):
    if not isinstance(manifest,dict) or type(manifest.get('schema')) is not int or manifest['schema']!=1 or manifest.get('diagnostic_only') is not True:
        raise ValueError('invalid diagnostic build schema')
    if not runtimes or any(runtime not in CHECKPOINTS for runtime in runtimes):
        raise ValueError('unknown runtime selection')
    if manifest.get('source_fingerprint')!=source_fingerprint():
        raise ValueError('compiler source changed after build')
    if manifest.get('tool_inputs')!=build.tool_inputs():
        raise ValueError('diagnostic tool inputs changed after build')
    config={str(path):build.sha(path) if path.is_file() else None
            for path in build.cargo_configuration_paths(native_environment(),build.STAGE)}
    if manifest.get('cargo_configuration')!=config:
        raise ValueError('Cargo configuration changed after build')
    for runtime in runtimes:
        info=manifest.get(runtime)
        if not isinstance(info,dict) or type(info.get('binary')) is not str:
            raise ValueError('missing built adapter: '+runtime)
        digest(info.get('sha256'),runtime+' binary')
        if not Path(info['binary']).is_file() or build.sha(info['binary'])!=info['sha256']:
            raise ValueError('adapter binary changed: '+runtime)

def validate_memory(value):
    fields={'live_requested_bytes','total_requested_bytes','peak_requested_bytes'}
    if not isinstance(value,dict) or set(value)!=fields: raise ValueError('invalid Rust memory schema')
    for key,item in value.items(): uint(item,key)
    if not value['live_requested_bytes']<=value['peak_requested_bytes']<=value['total_requested_bytes']:
        raise ValueError('inconsistent Rust allocator counters')

GO_MEM_INTS={'Alloc','TotalAlloc','Sys','Lookups','Mallocs','Frees','HeapAlloc','HeapSys','HeapIdle','HeapInuse',
    'HeapReleased','HeapObjects','StackInuse','StackSys','MSpanInuse','MSpanSys','MCacheInuse','MCacheSys',
    'BuckHashSys','GCSys','OtherSys','NextGC','LastGC','PauseTotalNs','NumGC','NumForcedGC'}

def validate_go_snapshot(value,name=None):
    if not isinstance(value,dict) or set(value)!={'name','unix_time_ns','mem_stats'}:
        raise ValueError('invalid Go snapshot schema')
    if type(value['name']) is not str or (name is not None and value['name']!=name):
        raise ValueError('wrong Go snapshot name')
    uint(value['unix_time_ns'],'snapshot time')
    stats=value['mem_stats']
    if not isinstance(stats,dict) or set(stats)!=GO_MEM_INTS|{'PauseNs','PauseEnd','GCCPUFraction','EnableGC','DebugGC','BySize'}:
        raise ValueError('invalid pinned Go MemStats schema')
    for key in GO_MEM_INTS: uint(stats[key],key)
    for key in ('PauseNs','PauseEnd'):
        if type(stats[key]) is not list or len(stats[key])!=256: raise ValueError('invalid Go pause ring')
        for item in stats[key]: uint(item,key)
    for key in ('EnableGC','DebugGC'):
        if type(stats[key]) is not bool: raise ValueError('invalid Go runtime flag')
    fraction=stats['GCCPUFraction']
    if type(fraction) not in (int,float) or not math.isfinite(fraction) or not 0<=fraction<=1:
        raise ValueError('invalid Go GC fraction')
    if type(stats['BySize']) is not list: raise ValueError('invalid Go size classes')
    for entry in stats['BySize']:
        if type(entry) is not dict or set(entry)!={'Size','Mallocs','Frees'}: raise ValueError('invalid Go size class')
        for key,item in entry.items(): uint(item,key)
        if entry['Frees']>entry['Mallocs']: raise ValueError('negative Go class live objects')
    if (stats['Alloc']!=stats['HeapAlloc'] or stats['Alloc']>stats['TotalAlloc']
        or stats['HeapObjects']!=stats['Mallocs']-stats['Frees']
        or stats['HeapIdle']+stats['HeapInuse']!=stats['HeapSys']
        or stats['HeapReleased']>stats['HeapIdle'] or stats['HeapAlloc']>stats['HeapInuse']
        or stats['NumForcedGC']>stats['NumGC']):
        raise ValueError('inconsistent Go runtime counters')

def validate_protocol_record(value,runtime,checkpoint_count,complete):
    if runtime not in CHECKPOINTS or type(value) is not dict or complete:
        raise ValueError('unknown runtime or record after completion')
    points=CHECKPOINTS[runtime]
    if 'checkpoint' in value:
        expected_fields={'checkpoint','memory' if runtime=='rust' else 'snapshot'}
        if set(value)!=expected_fields or checkpoint_count>=len(points) or value['checkpoint']!=points[checkpoint_count]:
            raise ValueError('unexpected/duplicate checkpoint')
        if runtime=='rust': validate_memory(value['memory'])
        else: validate_go_snapshot(value['snapshot'],value['checkpoint'])
        return 'checkpoint'
    if checkpoint_count!=len(points) or value.get('complete') is not True:
        raise ValueError('premature/invalid completion record')
    if runtime=='rust':
        if set(value)!={'complete'}: raise ValueError('invalid Rust completion schema')
    else:
        if set(value)!={'complete','diagnostic_only','files','loaded_input_sha256','report','report_sha256'} or value['diagnostic_only'] is not True:
            raise ValueError('invalid Go completion schema')
        uint(value['files'],'completed files')
        if type(value['report']) is not str or not value['report']: raise ValueError('invalid report path')
        digest(value['loaded_input_sha256'],'completed input'); digest(value['report_sha256'],'completed report')
    return 'complete'

def validate_work(report,expected,workers):
    for key,value in {**expected,'workers':workers}.items():
        if type(report.get(key)) is not type(value) or report[key] != value:
            raise ValueError('frozen work mismatch: '+key)
    if report.get('diagnostic_only') is not True: raise ValueError('missing diagnostic marker')

def validate_report(report,runtime,workers,checkpoints):
    if type(report.get('version')) is not int or report['version']!=1: raise ValueError('invalid report version')
    if runtime=='rust':
        if report.get('runtime')!='rust': raise ValueError('wrong report runtime')
        for name in ('pre_pipeline','retained_endpoint'): validate_memory(report.get(name))
        if report['retained_endpoint']!=checkpoints[1]['adapter']['memory']:
            raise ValueError('retained Rust checkpoint/report differ')
        before,after=report['pre_pipeline'],report['retained_endpoint']
        if after['total_requested_bytes']<before['total_requested_bytes']: raise ValueError('Rust total counter reversed')
        allocated=after['total_requested_bytes']-before['total_requested_bytes']
        live=after['live_requested_bytes']-before['live_requested_bytes']
        for name,want in [('pipeline_allocated_bytes',allocated),('pipeline_live_growth_bytes',live),
                          ('pipeline_superseded_or_freed_requested_bytes',allocated-live)]:
            if type(report.get(name)) is not int or report[name]!=want: raise ValueError('inconsistent '+name)
        uint(report.get('pipeline_wall_ns'),'pipeline wall time')
        signed={'parse_live_growth_bytes','publish_live_growth_bytes','bind_live_growth_bytes'}
        fields={'parse_ns','publish_ns','bind_ns','parse_allocated_bytes','publish_allocated_bytes','bind_allocated_bytes'}|signed
        times=report.get('worker_times');total=report.get('elapsed_worker_totals')
        if type(times) is not list or len(times)!=workers or type(total) is not dict or set(total)!=fields:
            raise ValueError('invalid Rust worker timers')
        for row in [*times,total]:
            if type(row) is not dict or set(row)!=fields: raise ValueError('invalid worker timer schema')
            for key,value in row.items():
                if key in signed:
                    if type(value) is not int: raise ValueError('invalid signed live-growth integer')
                else: uint(value,key)
        for key in fields:
            if total[key]!=sum(row[key] for row in times): raise ValueError('worker timer sum differs')
            if workers>1 and (key.endswith('allocated_bytes') or key in signed) and total[key]!=0: raise ValueError('unqualified concurrent phase allocation')
        for key in ('timer_domain','allocation_domain'):
            if type(report.get(key)) is not str or not report[key]: raise ValueError('missing measurement domain')
        cost=report.get('census_cost')
        if type(cost) is not dict or set(cost)!={'before','after','wall_ns'}: raise ValueError('invalid census cost')
        validate_memory(cost['before']);validate_memory(cost['after']);uint(cost['wall_ns'],'census wall time')
    else:
        if report.get('operation')!='parse_bind_memory_profile': raise ValueError('wrong Go operation')
        for key,want in [('gomaxprocs',workers),('gogc',100),('mem_profile_rate',65536)]:
            if type(report.get(key)) is not int or report[key]!=want: raise ValueError('wrong Go setting '+key)
        for key in ('wall_time_ns','allocated_bytes','cpu_capacity','goroutines_ready'): uint(report.get(key),key)
        points=report.get('snapshots')
        if type(points) is not list or len(points)!=len(CHECKPOINTS['go']): raise ValueError('invalid Go snapshots')
        for point,name,observed in zip(points,CHECKPOINTS['go'],checkpoints):
            validate_go_snapshot(point,name)
            if point!=observed['adapter']['snapshot']: raise ValueError('Go checkpoint/report differ')
        before=report.get('pipeline_before');validate_go_snapshot(before,'pipeline_before_barrier')
        if report['allocated_bytes']!=points[1]['mem_stats']['TotalAlloc']-before['mem_stats']['TotalAlloc']:
            raise ValueError('Go allocated-byte delta differs')
        times=report.get('worker_times')
        if type(times) is not list or len(times)!=workers: raise ValueError('invalid Go workers')
        for index,row in enumerate(times):
            fields={'worker','files','parse_calls','bind_calls','parse_ns','bind_ns'}
            if type(row) is not dict or set(row)!=fields: raise ValueError('invalid Go worker schema')
            for key,value in row.items(): uint(value,key)
            count=report['files']//workers+int(index<report['files']%workers)
            if row['worker']!=index or any(row[key]!=count for key in ('files','parse_calls','bind_calls')):
                raise ValueError('incomplete Go worker work')

def validate_census(census,report):
    if type(census) is not dict or type(census.get('schema')) is not int or census['schema']!=1 or census.get('domain')!='retained-owned-storage-census':
        raise ValueError('invalid Rust census schema')
    rows=census.get('rows')
    if type(rows) is not dict or not rows: raise ValueError('missing census rows')
    fields={'containers','allocations','elements','capacity_elements','used_payload_bytes','capacity_payload_bytes',
            'shared_references','shared_header_estimate_bytes','unreported_layout_containers'}
    for key,row in rows.items():
        if type(key) is not str or type(row) is not dict or set(row)!=fields: raise ValueError('invalid census row schema')
        for field,value in row.items(): uint(value,key+'.'+field)
        if row['used_payload_bytes']>row['capacity_payload_bytes'] or row['elements']>row['capacity_elements']:
            raise ValueError('census used exceeds capacity')
        if key.endswith('.records') and key not in ('preload.records',) and (row['used_payload_bytes'] or row['capacity_payload_bytes'] or row['allocations']):
            raise ValueError('census record observation double charges bytes')
    def count(name):
        if name not in rows: raise ValueError('missing census count '+name)
        return rows[name]['elements']
    def optional(name): return rows.get(name,{}).get('elements',0)
    for name,key in [('core.nodes.page_payload','nodes'),('BindResult.symbols.page_payload','symbols'),
                     ('SourceFileState.diagnostics','parse_diagnostics'),('BindResult.diagnostics','bind_diagnostics')]:
        if count(name)!=report[key]: raise ValueError('census/frozen work differs: '+key)
    nodes=sum(row['elements'] for key,row in rows.items() if key.startswith('NodeData.') and key.endswith('.records'))
    if nodes!=count('core.nodes.page_payload')+count('BindResult.nodes')+optional('lazy.nodes.page_payload.initialized_slots'):
        raise ValueError('node variants do not reconcile with allocated core/overlay/lazy records')
    aux=sum(row['elements'] for key,row in rows.items() if key.startswith('AstStorageData.') and key.endswith('.records'))
    if aux!=count('core.auxiliary.page_payload')+optional('lazy.auxiliary.page_payload.initialized_slots'):
        raise ValueError('auxiliary variants do not reconcile with allocated records')
    if type(census.get('limitations')) is not list or not census['limitations'] or any(type(v) is not str for v in census['limitations']):
        raise ValueError('missing census measurement limitations')

def validate_sites(rows,inventory,report):
    phases=('unscoped','preload','parse','publish','bind','retire')
    if type(inventory) is not dict or type(inventory.get('schema')) is not int or inventory['schema']!=1 or inventory.get('operation')!='staged-safe-allocation-scopes':
        raise ValueError('invalid sites manifest')
    sites=inventory.get('sites')
    if type(sites) is not list or not sites: raise ValueError('missing site inventory')
    expected={}
    for site in sites:
        for field in ('name','source_file'):
            if type(site.get(field)) is not str or not site[field]: raise ValueError('invalid site identity')
        uint(site.get('source_line'),'site source line')
        if site['source_line']==0: raise ValueError('site source line missing')
        for phase in phases:
            key=(phase,'inclusive_site',site['name'])
            if key in expected: raise ValueError('duplicate site inventory')
            expected[key]=(site['source_file'],site['source_line'])
    for phase in phases:
        expected[phase,'phase','phase']=('adapter phase guard',0)
        expected[phase,'selected_union','selected source regions']=('dynamic outermost source-site union',0)
    if type(rows) is not list or len(rows)!=len(expected): raise ValueError('missing/extra allocation site rows')
    observed={}
    fields={'name','phase','source_file','source_line','scope_kind','requested_bytes','allocation_calls','observations'}
    counters=('requested_bytes','allocation_calls','observations')
    for row in rows:
        if type(row) is not dict or set(row)!=fields: raise ValueError('invalid site row schema')
        if any(type(row[key]) is not str for key in ('name','phase','source_file','scope_kind')):
            raise ValueError('invalid site row identity type')
        key=(row['phase'],row['scope_kind'],row['name'])
        if key in observed or key not in expected: raise ValueError('duplicate/unknown allocation site row')
        uint(row['source_line'],'site source line')
        if (row['source_file'],row['source_line'])!=expected[key]: raise ValueError('allocation site source changed')
        for field in counters: uint(row[field],'site '+field)
        if row['observations']==0 and (row['requested_bytes'] or row['allocation_calls']):
            raise ValueError('allocation site traffic without observations')
        if row['requested_bytes'] and row['allocation_calls']==0: raise ValueError('allocation site bytes without calls')
        observed[key]=row
    counts={'unscoped':0,'preload':1,'parse':report['files'],'publish':report['files'],'bind':report['files'],'retire':1}
    for phase in phases:
        whole=observed[phase,'phase','phase'];union=observed[phase,'selected_union','selected source regions']
        if whole['observations']!=counts[phase]: raise ValueError('allocation phase observation/work mismatch')
        members=[row for key,row in observed.items() if key[0]==phase and key[1]=='inclusive_site']
        if phase!='unscoped' and any(union[key]>whole[key] for key in ('requested_bytes','allocation_calls')):
            raise ValueError('allocation site union exceeds its enclosing phase')
        if any(union[key]>sum(row[key] for row in members) for key in counters):
            raise ValueError('allocation union exceeds inclusive source inventory')
        # Inclusive recursive source rows may exceed the enclosing phase. They
        # are not disjoint; only the dynamic outermost-site union is bounded.
    return {'row_count':len(rows),'site_count':len(sites),'phases':list(phases),
            'unscoped_domain':'no enclosing phase guard; source rows/union preserved without a fabricated total'}

def validate_artifacts(runtime,prefix,report,manifest=None):
    artifacts={}
    if runtime=='rust':
        path=Path(str(prefix)+'-census.json');census=json.loads(path.read_text())
        validate_census(census,report)
        artifacts['census']={'path':str(path),'sha256':build.sha(path)}
        sites=Path(str(prefix)+'-sites.json')
        if manifest is not None and 'sites_manifest' in manifest:
            summary=validate_sites(json.loads(sites.read_text()),manifest['sites_manifest'],report)
            artifacts['sites']={'path':str(sites),'sha256':build.sha(sites),'validation':summary}
        elif sites.exists():
            raise ValueError('unexpected allocation-site artifact for native build')
    else:
        census=report.get('census')
        if type(census) is not dict or census.get('files')!=report['files'] or type(census.get('files')) is not int:
            raise ValueError('incomplete Go census work')
        uint(census.get('maximum_file_pointer_identities'),'census pointer count')
        uint(census.get('wall_time_ns'),'Go census wall time')
        validate_go_snapshot(census.get('before'));validate_go_snapshot(census.get('after'))
        profiles=report.get('profiles');names=['pre_pipeline','retained_endpoint','retained_after_gc','post_retirement_after_gc']
        if type(profiles) is not list or len(profiles)!=len(names): raise ValueError('incomplete Go profiles')
        for profile,name in zip(profiles,names):
            if type(profile) is not dict or profile.get('name')!=name: raise ValueError('wrong Go profile order')
            uint(profile.get('wall_time_ns'),'profile wall time')
            validate_go_snapshot(profile.get('before'));validate_go_snapshot(profile.get('after'))
            for kind in ('heap','allocs'):
                path=Path(str(prefix)+'-'+name+'.'+kind+'.pprof')
                digest(profile.get(kind+'_sha256'),'profile digest')
                if type(profile.get(kind+'_path')) is not str or Path(profile[kind+'_path']).resolve()!=path.resolve() or build.sha(path)!=profile[kind+'_sha256']:
                    raise ValueError('Go profile artifact identity differs')
                artifacts[name+'.'+kind]={'path':str(path),'sha256':build.sha(path)}
    return artifacts

def native_snapshot(pid,prefix):
    start=time.monotonic_ns()
    before=subprocess.check_output(['/bin/ps','-o','rss=','-p',str(pid)],text=True).strip()
    vm=subprocess.run(['/usr/bin/vmmap','-summary',str(pid)],text=True,capture_output=True,timeout=45)
    prefix.with_suffix('.vmmap.txt').write_text(vm.stdout)
    prefix.with_suffix('.vmmap.stderr').write_text(vm.stderr)
    if vm.returncode: raise ValueError('vmmap failed: '+str(prefix))
    after=subprocess.check_output(['/bin/ps','-o','rss=','-p',str(pid)],text=True).strip()
    if not before.isdigit() or not after.isdigit(): raise ValueError('invalid ps RSS')
    return {'rss_before_bytes':int(before)*1024,'rss_after_bytes':int(after)*1024,
        'snapshot_wall_ns':time.monotonic_ns()-start,'vmmap_sha256':build.sha(prefix.with_suffix('.vmmap.txt')),
        'domain':'ps RSS (KiB converted to bytes); vmmap footprint/region summary retained separately; child cooperatively paused, runtime background threads may run'}

def run(runtime,binary,workers,prefix,inputs,expected,manifest=None):
    env=go_native_environment() if runtime=='go' else native_environment()
    env['GOCACHE']='/private/tmp/ts-rust-s07-go-cache'
    reject_concurrent_builds()
    command=[str(binary),str(inputs),str(workers),str(prefix)]
    started=time.monotonic_ns()
    checkpoints=[]
    with prefix.with_suffix('.stdout').open('w') as stdout, prefix.with_suffix('.stderr').open('w') as stderr:
        child=subprocess.Popen(command,cwd=ROOT,env=env,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=stderr,start_new_session=True,text=True,bufsize=1)
        try:
            selector=selectors.DefaultSelector(); selector.register(child.stdout,selectors.EVENT_READ)
            complete=False
            while True:
                if not selector.select(timeout=180): raise TimeoutError('memory adapter stalled')
                line=child.stdout.readline()
                if not line: break
                stdout.write(line); stdout.flush()
                value=json.loads(line)
                kind=validate_protocol_record(value,runtime,len(checkpoints),complete)
                if kind=='checkpoint':
                    point=native_snapshot(child.pid,Path(str(prefix)+'-'+value['checkpoint']))
                    checkpoints.append({'adapter':value,'native':point})
                    child.stdin.write('\n'); child.stdin.flush()
                else:
                    complete=True
                    completion=value
            selector.close()
            if child.wait(timeout=30)!=0: raise ValueError('adapter failed; see '+str(prefix)+'.stderr')
            if len(checkpoints)!=len(CHECKPOINTS[runtime]) or not complete: raise ValueError('incomplete checkpoint protocol')
        except BaseException:
            try: os.killpg(child.pid,signal.SIGKILL)
            except ProcessLookupError: pass
            child.wait(); raise
    report_path=Path(str(prefix)+'-report.json')
    report=json.loads(report_path.read_text())
    validate_work(report,expected,workers)
    validate_report(report,runtime,workers,checkpoints)
    artifacts=validate_artifacts(runtime,prefix,report,manifest)
    if runtime=='go':
        if (Path(completion['report']).resolve()!=report_path.resolve()
            or completion['report_sha256']!=build.sha(report_path)
            or completion['files']!=report['files']
            or completion['loaded_input_sha256']!=report['loaded_input_sha256']):
            raise ValueError('Go completion/report identity differs')
    return {'runtime':runtime,'workers':workers,'command':command,'checkpoints':checkpoints,'report':report,
            'report_sha256':build.sha(report_path),'artifacts':artifacts,'elapsed_ns':time.monotonic_ns()-started}

def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--output',type=Path,required=True)
    ap.add_argument('--build-manifest',type=Path,default=OUTPUT/'build.json')
    ap.add_argument('--runtime',choices=('rust','go','both'),default='both')
    ap.add_argument('--workers',type=int,nargs='+',default=[1,8])
    ap.add_argument('--repetitions',type=int,default=3)
    args=ap.parse_args()
    if any(w not in (1,8) for w in args.workers) or len(set(args.workers))!=len(args.workers) or not 1<=args.repetitions<=5:
        raise ValueError('invalid capture matrix')
    args.output=args.output.resolve(); args.output.mkdir(parents=True,exist_ok=False)
    lock=(OUTPUT/'capture.lock').open('a'); fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
    baseline,_=read_capture()
    manifest_path=args.build_manifest.resolve()
    manifest=json.loads(manifest_path.read_text())
    runtimes=['rust','go'] if args.runtime=='both' else [args.runtime]
    validate_build(manifest,runtimes)
    inputs=CACHE/'s07-benchmark/inputs.json'
    metadata={'schema':1,'diagnostic_only':True,'host':host_info(),'build':manifest,
        'build_manifest':str(manifest_path),'build_sha256':build.sha(manifest_path),'expected_work':baseline['expected_work'],'transport_sha256':build.sha(inputs),
        'baseline_samples_sha256':baseline['samples_sha256'],'repetitions':args.repetitions}
    build.write_json(args.output/'metadata.json',metadata)
    rows=[]
    with (args.output/'runs.ndjson').open('w') as log:
        for workers in args.workers:
            for rep in range(args.repetitions):
                for runtime in (runtimes if rep%2==0 else runtimes[::-1]):
                    validate_build(manifest,runtimes)
                    prefix=args.output/f'{runtime}-{workers}-{rep}'
                    row=run(runtime,manifest[runtime]['binary'],workers,prefix,inputs,baseline['expected_work'],manifest)
                    row['repetition']=rep; rows.append(row)
                    log.write(json.dumps(row,sort_keys=True)+'\n'); log.flush()
                    print(f'Captured {runtime} memory workers={workers} repetition={rep}',flush=True)
    validate_build(manifest,runtimes)
    if build.sha(inputs)!=metadata['transport_sha256'] or build.sha(manifest_path)!=metadata['build_sha256']:
        raise ValueError('build/input changed during capture')
    build.write_json(args.output/'report.json',{**metadata,'capture_complete':True,'runs':rows})

if __name__=='__main__': main()
