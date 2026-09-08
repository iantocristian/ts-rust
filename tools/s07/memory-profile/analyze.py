#!/usr/bin/env python3
"""Summarize measured domains without treating unequal censuses as full heaps."""
import argparse
from collections import defaultdict
import json
from pathlib import Path
import statistics
import build


def rust_census(value):
    rows=value['rows']
    sums={key:sum(r[key] for r in rows.values()) for key in
          ('used_payload_bytes','capacity_payload_bytes','shared_header_estimate_bytes')}
    sums['capacity_minus_used_bytes']=sums['capacity_payload_bytes']-sums['used_payload_bytes']
    sums['payload_plus_arc_estimate_bytes']=sums['capacity_payload_bytes']+sums['shared_header_estimate_bytes']
    sums['categories']=dict(sorted(rows.items(),key=lambda item:(-item[1]['capacity_payload_bytes'],item[0])))
    return sums


def go_census(value):
    groups=defaultdict(lambda:{'objects':0,'logical_bytes':0})
    for row in value['objects']:
        groups[row['category']]['objects']+=row['count']
        groups[row['category']]['logical_bytes']+=row['logical_bytes']
    objects=sum(row['logical_bytes'] for row in value['objects'])
    slices=sum(row['visible_capacity_union_bytes'] for row in value['slices'])
    maps=sum(row['entry_logical_bytes'] for row in value['maps'])
    source=value['source_text_bytes']
    return {'object_categories':dict(groups),'objects_bytes':objects,'visible_slice_capacity_bytes':slices,
        'occupied_map_entry_bytes':maps,'source_text_bytes':source,
        'known_logical_payload_bytes':objects+slices+maps+source,
        'node_header_contained_bytes_nonadditive':value['node_header_contained_bytes'],
        'other_visible_string_bytes_not_heap_total':value['strings']['other_visible_union_bytes'],
        'scope':'reachable objects and visible ranges; source bytes counted once; other strings may be static/shared, excluded; arena backing slack and map internals missing'}


def summarize(capture_path):
    data=json.loads((capture_path/'report.json').read_text())
    if data.get('capture_complete') is not True or data.get('diagnostic_only') is not True:
        raise ValueError('not a completed diagnostic capture')
    out=[]
    for run in data['runs']:
        rt,w,i=run['runtime'],run['workers'],run['repetition']
        prefix=capture_path/f'{rt}-{w}-{i}'
        report=run['report']
        points={p['adapter']['checkpoint']:p for p in run['checkpoints']}
        retained=points['retained_endpoint']['native']
        row={'runtime':rt,'workers':w,'repetition':i,'prefix':str(prefix),
            'retained_rss_bytes':retained['rss_before_bytes'],'retained_rss_after_vmmap_bytes':retained['rss_after_bytes']}
        if rt=='rust':
            mem=report['retained_endpoint']; c=rust_census(json.loads(Path(str(prefix)+'-census.json').read_text()))
            row.update(live_requested_bytes=mem['live_requested_bytes'],
                interval_request_bytes=report['pipeline_allocated_bytes'],
                interval_live_growth_bytes=report['pipeline_live_growth_bytes'],
                interval_freed_or_superseded_requests_bytes=report['pipeline_superseded_or_freed_requested_bytes'],
                census=c,census_cost=report['census_cost'],phase_global_counter_totals=report['elapsed_worker_totals'],
                live_minus_census_payload_and_arc_estimate_bytes=mem['live_requested_bytes']-c['payload_plus_arc_estimate_bytes'],
                rss_minus_requested_live_bytes=retained['rss_before_bytes']-mem['live_requested_bytes'])
            site=Path(str(prefix)+'-sites.json')
            if site.exists():
                sites=json.loads(site.read_text()); phases={}
                for phase in ('parse','publish','bind'):
                    total=next(x['requested_bytes'] for x in sites if x['phase']==phase and x['scope_kind']=='phase')
                    union=next(x['requested_bytes'] for x in sites if x['phase']==phase and x['scope_kind']=='selected_union')
                    if union>total: raise ValueError('site union exceeds phase')
                    phases[phase]={'total_requested_bytes':total,'selected_union_bytes':union,'unclassified_region_bytes':total-union}
                row['sites']={'phases':phases,'inclusive_regions_not_additive':sorted((x for x in sites if x['scope_kind']=='inclusive_site' and x['phase'] in phases),key=lambda x:-x['requested_bytes'])}
        else:
            snapshots={s['name']:s['mem_stats'] for s in report['snapshots']}
            end=snapshots['retained_endpoint']; gc=snapshots['retained_after_gc']
            c=go_census(report['census']['categories'])
            row.update(interval_totalalloc_bytes=report['allocated_bytes'],
                native_mem_stats=end,retained_after_two_gc_mem_stats=gc,census=c,
                census_cost={k:report['census'][k] for k in ('before','after','wall_time_ns')},
                post_gc_heapalloc_minus_known_logical_payload_bytes=gc['HeapAlloc']-c['known_logical_payload_bytes'],
                native_heap_inuse_minus_heapalloc_bytes=end['HeapInuse']-end['HeapAlloc'],
                native_rss_minus_heapalloc_bytes=retained['rss_before_bytes']-end['HeapAlloc'])
        out.append(row)
    return out


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('captures',type=Path,nargs='+');ap.add_argument('--output',type=Path,required=True)
    args=ap.parse_args()
    rows=[r for p in args.captures for r in summarize(p)]
    groups=defaultdict(list)
    for r in rows:groups[(r['runtime'],r['workers'],'sites' in r)].append(r)
    medians=[]
    for (rt,w,sites),items in sorted(groups.items()):
        numeric=set.intersection(*(set(k for k,v in item.items() if type(v) is int and k not in {'workers','repetition'}) for item in items))
        medians.append({'runtime':rt,'workers':w,'source_scope_instrumentation':sites,'samples':len(items),
            'metrics':{k:{'median':statistics.median(x[k] for x in items),'minimum':min(x[k] for x in items),'maximum':max(x[k] for x in items)} for k in sorted(numeric)}})
    build.write_json(args.output,{'schema':1,'diagnostic_only':True,'captures':[str(p) for p in args.captures],
        'capture_sha256':{str(p):build.sha(p/'report.json') for p in args.captures},'medians':medians,'runs':rows,
        'limitations':['Rust census walks every allocated owned slot; Go census walks reachable objects and visible backing ranges. Their difference is not a retained-heap delta.',
        'Rust requested bytes, Go HeapAlloc and OS RSS have different inclusion and rounding semantics. Residuals are unattributed differences, not measured allocator overhead.',
        'Go retained two-GC and native endpoint are separate states; census/post-retirement work is deliberately outside both retained snapshots.']})

if __name__=='__main__':main()
