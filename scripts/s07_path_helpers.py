#!/usr/bin/env python3
"""Verify source path observations including byte, folding and panic boundaries."""
import argparse,hashlib,json,shutil
from s04_common import command,strict_json_loads
from s06_build import ROOT,oracle_export

def recipes():
    paths=[b'',b'.',b'..',b'./',b'../',b'a',b'a/',b'a//b',b'a/../b',b'a./b',b'a/b',b'a../b',b'/a',b'/A',b'/a/b',b'/a/',b'/a/../b',b'c:',b'C:/a',b'c:\\A',b'//server',b'//SERVER/a',b'http://host/a',b'file:///c:',b'^/untitled/a','/K/ς'.encode(),'/k/Σ'.encode(),'/İ/ı'.encode(),b'/I/i',b'/a/\xff',b'/a/\xfe',b'\xe2\x82']
    return [dict(id=index,a=a.hex(),b=b.hex(),cwd=cwd.hex(),sensitive=sensitive) for index,(a,b,cwd,sensitive) in enumerate((a,b,cwd,sensitive) for a in paths for b in paths for cwd in [b'',b'/here'] for sensitive in [True,False])]

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--check',action='store_true');args=parser.parse_args()
    requests=(json.dumps(recipes(),sort_keys=True,indent=2)+'\n').encode();adapter=ROOT/'tools/s07/paths/export_test.go'
    with oracle_export() as (checkout,env,pin):
        request_path=checkout/'requests.json';request_path.write_bytes(requests);output=checkout/'paths.json'
        shutil.copyfile(adapter,checkout/'tsc/internal/tspath/s07_paths_test.go');env.update(S07_PATH_REQUESTS=str(request_path),S07_PATH_OUTPUT=str(output))
        command(['go','test','-trimpath','-mod=readonly','./internal/tspath','-run','^TestS07Paths$','-count=1'],cwd=checkout/'tsc',env=env)
        data=output.read_bytes();rows=strict_json_loads(data)
        if [r['id'] for r in rows]!=list(range(len(recipes()))):raise ValueError('path observations lost exact request order')
        manifest=dict(pin=pin,requests=len(rows),observations_sha256=hashlib.sha256(data).hexdigest(),requests_sha256=hashlib.sha256(requests).hexdigest(),adapter_sha256=hashlib.sha256(adapter.read_bytes()).hexdigest(),source_sha256=hashlib.sha256((checkout/'tsc/internal/tspath/path.go').read_bytes()).hexdigest())
    for name,raw in {'path-requests.json':requests,'path-observations.json':data,'path-manifest.json':(json.dumps(manifest,sort_keys=True,indent=2)+'\n').encode()}.items():
        path=ROOT/'data/s07'/name
        if args.check:
            if path.read_bytes()!=raw:raise ValueError('pinned path observations drift: '+name)
        else:path.write_bytes(raw)
    print(f'{len(rows)} direct Go path requests')
if __name__=='__main__':main()
