#!/usr/bin/env python3
"""Direct original-Go ResolveConfig observations; no interpreted module proxy."""
import argparse,hashlib,json,shutil
from s04_common import command,strict_json_loads
from s06_build import ROOT,oracle_export

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--check',action='store_true');args=parser.parse_args()
    adapter=ROOT/'tools/s07/config-resolver/export_test.go';requests=ROOT/'data/s07/config-resolver-requests.json'
    expected=[row['id'] for row in strict_json_loads(requests.read_bytes())]
    with oracle_export() as (checkout,env,pin):
        shutil.copyfile(adapter,checkout/'tsc/internal/module/s07_config_resolver_test.go')
        output=checkout/'config-resolver.json';env.update(S07_CONFIG_RESOLVER_REQUESTS=str(requests),S07_CONFIG_RESOLVER_OUTPUT=str(output))
        repo_path=f'-gcflags=github.com/microsoft/TypeScript/tsc/internal/repo=-trimpath={checkout}/s07-unmatched-prefix'
        command(['go','test','-trimpath','-mod=readonly',repo_path,'./internal/module','-run','^TestS07ConfigResolver$','-count=1'],cwd=checkout/'tsc',env=env)
        raw=output.read_bytes();rows=strict_json_loads(raw)
        if [row['id'] for row in rows]!=expected:raise ValueError('missing/duplicate/reordered config resolver request')
        sha=lambda path:hashlib.sha256(path.read_bytes()).hexdigest()
        manifest={'upstream_pin':pin,'adapter_sha256':sha(adapter),'requests_sha256':sha(requests),'observations_sha256':hashlib.sha256(raw).hexdigest(),'requests':len(rows),'sources':{name:sha(checkout/name) for name in ('tsc/internal/module/resolver.go','tsc/internal/packagejson/packagejson.go')}}
    for name,content in {'config-resolver-observations.json':raw,'config-resolver-manifest.json':(json.dumps(manifest,indent=2,sort_keys=True)+'\n').encode()}.items():
        path=ROOT/'data/s07'/name
        if args.check:
            if path.read_bytes()!=content:raise ValueError('config resolver source observations changed: '+name)
        else:path.write_bytes(content)
    print(json.dumps(manifest))
if __name__=='__main__':main()
