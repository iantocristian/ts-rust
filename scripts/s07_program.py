#!/usr/bin/env python3
"""Capture the actual pinned compiler loader's ordered program closure."""
import argparse
import hashlib
import json
import shutil
from s04_common import command, strict_json_loads
from s06_build import ROOT, oracle_export

def validate_requests(request_rows):
    if not isinstance(request_rows,list) or not request_rows: raise ValueError('requests must be an array')
    ids=[]
    for row in request_rows:
        if not isinstance(row,dict) or not isinstance(row.get('id'),str) or not row['id']: raise ValueError('invalid request identity')
        ids.append(row['id'])
        if not isinstance(row.get('cwd'),str) or type(row.get('case_sensitive')) is not bool: raise ValueError('invalid host request')
        if not isinstance(row.get('roots'),list) or any(not isinstance(v,str) for v in row['roots']): raise ValueError('invalid ordered roots')
        if not isinstance(row.get('files'),dict) or not isinstance(row.get('options'),dict): raise ValueError('invalid files/options')
        for name,value in row['files'].items():
            if not isinstance(value,str) or any(c not in '0123456789abcdef' for c in value) or len(value)%2: raise ValueError('invalid file hex '+name)
        if not isinstance(row.get('symlinks',{}),dict) or any(not isinstance(v,str) for v in row.get('symlinks',{}).values()): raise ValueError('invalid symlinks')
        if set(row['files']) & set(row.get('symlinks',{})): raise ValueError('duplicate file/symlink')
    if len(ids)!=len(set(ids)): raise ValueError('duplicate request IDs')
    return ids

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check',action='store_true')
    parser.add_argument('--check-subset',action='store_true')
    parser.add_argument('--oracle',type=__import__('pathlib').Path)
    parser.add_argument('--config-evidence',type=__import__('pathlib').Path)
    parser.add_argument('--requests',type=__import__('pathlib').Path)
    parser.add_argument('--output',type=__import__('pathlib').Path)
    args=parser.parse_args()
    if args.check_subset:
        if args.check or not args.requests or not args.output or not args.oracle:
            raise ValueError('--check-subset requires --requests, --oracle, --output and forbids --check')
        from s07_program_compare import check_subset
        report=check_subset(args.requests,args.oracle,args.output,args.config_evidence)
        print(json.dumps({key:report[key] for key in ('required_variants','passed_variants','loader_graph_parity','config_parity','metrics')}))
        if not report['metrics']['subset_loads']: raise SystemExit(1)
        return
    if args.oracle or args.config_evidence: raise ValueError('--oracle/--config-evidence require --check-subset')
    adapter=ROOT/'tools/s07/program/export_test.go'
    requests=args.requests.resolve() if args.requests else ROOT/'data/s07/program-requests.json'
    if bool(args.requests)!=bool(args.output): raise ValueError('--requests and --output must be provided together')
    request_rows=strict_json_loads(requests.read_bytes())
    validate_requests(request_rows)
    with oracle_export() as (checkout,env,pin):
        shutil.copyfile(adapter,checkout/'tsc/internal/compiler/s07_program_test.go')
        observed=checkout/'program.json'
        env['S07_PROGRAM_REQUESTS']=str(requests)
        env['S07_PROGRAM_OUTPUT']=str(observed)
        repo_path=f'-gcflags=github.com/microsoft/TypeScript/tsc/internal/repo=-trimpath={checkout}/s07-unmatched-prefix'
        command(['go','test','-trimpath','-mod=readonly',repo_path,'./internal/compiler','-run','^TestS07Programs$','-count=1'],cwd=checkout/'tsc',env=env)
        data=observed.read_bytes()
        rows=strict_json_loads(data)
        if [r['ID'] for r in rows]!=[r['id'] for r in request_rows]:
            raise ValueError('missing, extra, or reordered program observations')
        manifest={'upstream_pin':pin,'adapter_sha256':hashlib.sha256(adapter.read_bytes()).hexdigest(),'requests_sha256':hashlib.sha256(requests.read_bytes()).hexdigest(),'observations_sha256':hashlib.sha256(data).hexdigest(),'rows':len(rows),'source_sha256':{str(p.relative_to(checkout)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted((checkout/'tsc/internal/compiler').glob('*.go')) if not p.name.endswith('_test.go')}}
    if args.output:
        if args.check: raise ValueError('--check is only valid for the frozen fixture')
        args.output.write_bytes(data)
        args.output.with_suffix('.manifest.json').write_text(json.dumps(manifest,sort_keys=True,indent=2)+'\n')
        print(f'{len(rows)} direct Go program loader rows')
        return
    for name,raw in {'program-observations.json':data,'program-manifest.json':(json.dumps(manifest,sort_keys=True,indent=2)+'\n').encode()}.items():
        target=ROOT/'data/s07'/name
        if args.check:
            if target.read_bytes()!=raw: raise ValueError('pinned program observations changed: '+name)
        else: target.write_bytes(raw)
    print(f'{len(rows)} direct Go program loader rows')
if __name__=='__main__': main()
