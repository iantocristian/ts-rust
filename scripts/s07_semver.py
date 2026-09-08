#!/usr/bin/env python3
"""Compare checked-in semver evidence with a fresh, minimal pinned Go export."""
import argparse
import hashlib
import io
import json
from pathlib import Path
import shutil
import tarfile
import tempfile

from s04 import go_environment, verified_upstream
from s04_common import command, strict_json_loads

ROOT = Path(__file__).resolve().parents[1]


def digest(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write', action='store_true', help='replace reviewed requests and observations')
    args = parser.parse_args()
    upstream = verified_upstream()
    pin = strict_json_loads((ROOT / 'data/upstream.json').read_bytes())['pin']
    adapter = ROOT / 'tools/s07/semver/export_test.go'
    supplemental = ROOT / 'tools/s07/semver/supplemental.json'
    inputs = strict_json_loads(supplemental.read_bytes())
    seen = set()
    for row in inputs:
        value = row['hex']
        if not isinstance(value, str) or len(value) % 2 or any(c not in '0123456789abcdef' for c in value):
            raise ValueError('malformed supplemental hex')
        if value in seen or not row['origins'] or any(not isinstance(v, str) for v in row['origins']):
            raise ValueError('duplicate supplemental input or invalid origin')
        seen.add(value)
    destination = ROOT / 'target/s07-semver'
    destination.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='export-', dir=destination) as temporary:
        checkout = Path(temporary)
        archive = command(['git', 'archive', pin, 'tsc/go.mod', 'tsc/go.sum', 'tsc/internal/semver'], cwd=upstream)
        with tarfile.open(fileobj=io.BytesIO(archive)) as stream:
            stream.extractall(checkout, filter='data')
        package = checkout / 'tsc/internal/semver'
        source_hashes = {p.name: digest(p.read_bytes()) for p in sorted(package.glob('*.go'))}
        shutil.copyfile(adapter, package / 's07_export_test.go')
        output = checkout / 'observations.json'
        env = go_environment()
        env.update(S07_SEMVER_OUTPUT=str(output), S07_SEMVER_SUPPLEMENTAL=str(supplemental))
        # Also execute all unchanged upstream semver tests.
        command(['go', 'test', '-trimpath', '-mod=readonly', './internal/semver', '-count=1'], cwd=checkout / 'tsc', env=env)
        rows = strict_json_loads(output.read_bytes())
        requests = rows.pop('inputs')
        keys = [r['hex'] for r in requests]
        if keys != sorted(set(keys)) or not seen.issubset(keys):
            raise ValueError('missing, duplicate, or reordered observed requests')
        count = len(requests)
        if len(rows['versions']) != count or len(rows['ranges']) != count:
            raise ValueError('missing or extra observation rows')
        if len(rows['nil_compare']) != count + 1 or set(rows['nil_compare']) - set('<=>'):
            raise ValueError('invalid nil comparison matrix')
        for version, version_range in zip(rows['versions'], rows['ranges'], strict=True):
            if len(version['compare']) != count + 1 or set(version['compare']) - set('<=>'):
                raise ValueError('invalid version comparison matrix')
            if type(version_range['ok']) is not bool or len(version_range['matches']) != count + 1 or set(version_range['matches']) - set('01'):
                raise ValueError('invalid range test matrix')
    verified_upstream()
    encode = lambda value: (json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=False) + '\n').encode()
    request_data = encode(requests)
    observation_data = encode(rows)
    manifest = {
        'version': 1, 'pin': pin, 'source_sha256': source_hashes,
        'inputs_sha256': {str(p.relative_to(ROOT)): digest(p.read_bytes()) for p in [adapter, supplemental, Path(__file__).resolve(), ROOT / 'scripts/s04.py', ROOT / 'scripts/s04_common.py', ROOT / 'data/s04/toolchains.toml', ROOT / 'data/upstream.json']},
        'requests_sha256': digest(request_data), 'observations_sha256': digest(observation_data),
        'requests': count, 'comparison_pairs': (count + 1) ** 2, 'range_tests': count * (count + 1),
    }
    for name, data in [('semver-requests.json', request_data), ('semver-observations.json', observation_data), ('semver-manifest.json', encode(manifest))]:
        target = ROOT / 'data/s07' / name
        if args.write:
            target.write_bytes(data)
        elif not target.exists() or target.read_bytes() != data:
            raise ValueError('pinned semver evidence changed: ' + name)
    print(f'{count} pinned semver inputs; {(count + 1) ** 2} comparisons; {count * (count + 1)} range tests')


if __name__ == '__main__':
    main()
