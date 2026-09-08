#!/usr/bin/env python3
"""Source content-mapper declaration/manifest and compact JSON observations."""
import argparse,hashlib,json,shutil
from s04_common import command,strict_json_loads
from s06_build import ROOT,oracle_export

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--check',action='store_true');args=parser.parse_args()
    adapter=ROOT/'tools/s07/config-mappers/export_test.go';requests=ROOT/'data/s07/config-mapper-requests.json'
    expected=strict_json_loads(requests.read_bytes())
    with oracle_export() as (checkout,env,pin):
        shutil.copyfile(adapter,checkout/'tsc/internal/tsoptions/s07_config_mappers_test.go')
        output=checkout/'config-mappers.json';env.update(S07_MAPPERS_REQUESTS=str(requests),S07_MAPPERS_OUTPUT=str(output))
        repo_path=f'-gcflags=github.com/microsoft/TypeScript/tsc/internal/repo=-trimpath={checkout}/s07-unmatched-prefix'
        command(['go','test','-trimpath','-mod=readonly',repo_path,'./internal/tsoptions','-run','^TestS07ConfigMappers$','-count=1'],cwd=checkout/'tsc',env=env)
        raw=output.read_bytes();rows=strict_json_loads(raw)
        for family in ('mappers','json'):
            if [row['id'] for row in rows[family]]!=[row['id'] for row in expected[family]]:raise ValueError('missing/duplicate/reordered request: '+family)
        sha=lambda path:hashlib.sha256(path.read_bytes()).hexdigest()
        manifest={'upstream_pin':pin,'adapter_sha256':sha(adapter),'requests_sha256':sha(requests),'observations_sha256':hashlib.sha256(raw).hexdigest(),'requests':{name:len(rows[name]) for name in rows},'sources':{name:sha(checkout/name) for name in ('tsc/internal/tsoptions/contentmappers.go','tsc/internal/tsoptions/tsconfigparsing.go','tsc/internal/tsoptions/parsinghelpers.go','tsc/internal/json/json.go')}}
    for name,content in {'config-mapper-observations.json':raw,'config-mapper-manifest.json':(json.dumps(manifest,indent=2,sort_keys=True)+'\n').encode()}.items():
        path=ROOT/'data/s07'/name
        if args.check:
            if path.read_bytes()!=content:raise ValueError('source config mapper observation changed: '+name)
        else:path.write_bytes(content)
    print(json.dumps(manifest))
if __name__=='__main__':main()
