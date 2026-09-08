#!/usr/bin/env python3
"""Observe unchanged packagejson.Parse and field readers at the Go pin."""
import argparse
import hashlib
import json
import shutil
from s04_common import command, strict_json_loads
from s06_build import ROOT, oracle_export


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write', action='store_true')
    args = parser.parse_args()
    adapter = ROOT / 'tools/s07/packagejson/export_test.go'
    requests = ROOT / 'tools/s07/packagejson/requests.json'
    rows = strict_json_loads(requests.read_bytes())
    ids = [r['id'] for r in rows]
    if len(ids) != len(set(ids)) or any(not isinstance(v, str) or not v for v in ids):
        raise ValueError('invalid or duplicate request identities')
    for row in rows:
        value = row['hex']
        if not isinstance(value, str) or len(value) % 2 or any(c not in '0123456789abcdef' for c in value):
            raise ValueError('malformed request hex')
    with oracle_export() as (checkout, env, pin):
        package = checkout / 'tsc/internal/packagejson'
        source_hashes = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(package.glob('*.go'))}
        shutil.copyfile(adapter, package / 's07_export_test.go')
        output = checkout / 'packagejson.json'
        env.update(S07_PACKAGEJSON_REQUESTS=str(requests), S07_PACKAGEJSON_OUTPUT=str(output))
        repo_path = f'-gcflags=github.com/microsoft/TypeScript/tsc/internal/repo=-trimpath={checkout}/s07-unmatched-prefix'
        command(['go', 'test', '-trimpath', '-mod=readonly', repo_path, './internal/packagejson', '-count=1'], cwd=checkout / 'tsc', env=env)
        data = output.read_bytes()
        if [r['id'] for r in strict_json_loads(data)] != ids:
            raise ValueError('missing, extra, or reordered observation rows')
    manifest = {'version':1,'pin':pin,'source_sha256':source_hashes,'inputs_sha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [adapter, requests, ROOT/'scripts/s07_packagejson.py', ROOT/'scripts/s04.py', ROOT/'scripts/s04_common.py', ROOT/'scripts/s06_build.py', ROOT/'data/upstream.json', ROOT/'data/s04/toolchains.toml']},'observations_sha256':hashlib.sha256(data).hexdigest(),'requests':len(rows)}
    for name, raw in [('packagejson-observations.json',data),('packagejson-manifest.json',(json.dumps(manifest,sort_keys=True,separators=(',',':'))+'\n').encode())]:
        target=ROOT/'data/s07'/name
        if args.write: target.write_bytes(raw)
        elif not target.exists() or target.read_bytes()!=raw: raise ValueError('pinned package JSON evidence changed: '+name)
    print(f'{len(rows)} direct Go package JSON observations')


if __name__ == '__main__':
    main()
