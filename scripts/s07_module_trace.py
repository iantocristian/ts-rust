#!/usr/bin/env python3
"""Freeze direct pinned module-resolution trace callbacks and typed arguments."""
import argparse
import hashlib
import json
import shutil
from s04_common import command,strict_json_loads
from s06_build import ROOT,oracle_export

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--check',action='store_true');args=parser.parse_args()
    adapter=ROOT/'tools/s07/module-trace/export_test.go';requests=ROOT/'data/s07/module-trace-requests.json'
    recipes=strict_json_loads(requests.read_bytes());expected=[(row['id'],index) for row in recipes for index,_ in enumerate(row['operations'])]
    with oracle_export() as (checkout,env,pin):
        shutil.copyfile(adapter,checkout/'tsc/internal/module/s07_module_trace_test.go')
        output=checkout/'module-trace.json';env.update(S07_MODULE_TRACE_REQUESTS=str(requests),S07_MODULE_TRACE_OUTPUT=str(output))
        repo_path=f'-gcflags=github.com/microsoft/TypeScript/tsc/internal/repo=-trimpath={checkout}/s07-unmatched-prefix'
        command(['go','test','-trimpath','-mod=readonly',repo_path,'./internal/module','-run','^TestS07ModuleTraces$','-count=1'],cwd=checkout/'tsc',env=env)
        raw=output.read_bytes();rows=strict_json_loads(raw)
        if [(row['id'],row['operation']) for row in rows]!=expected:raise ValueError('missing/duplicate/reordered trace request')
        hashes=lambda path:hashlib.sha256(path.read_bytes()).hexdigest()
        manifest={'upstream_pin':pin,'adapter_sha256':hashes(adapter),'requests_sha256':hashes(requests),'observations_sha256':hashlib.sha256(raw).hexdigest(),'operations':len(rows),'callbacks':sum(len(row['traces']) for row in rows),'sources':{name:hashes(checkout/name) for name in ('tsc/internal/module/resolver.go','tsc/internal/packagejson/cache.go')}}
    for name,content in {'module-trace-observations.json':raw,'module-trace-manifest.json':(json.dumps(manifest,indent=2,sort_keys=True)+'\n').encode()}.items():
        path=ROOT/'data/s07'/name
        if args.check:
            if path.read_bytes()!=content:raise ValueError('direct module trace changed: '+name)
        else:path.write_bytes(content)
    print(json.dumps(manifest))
if __name__=='__main__':main()
