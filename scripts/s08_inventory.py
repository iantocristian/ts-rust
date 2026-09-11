#!/usr/bin/env python3
"""Typed conservative S08 dependency audit. Never a checker completion metric."""
import argparse
import json
import lzma
from pathlib import Path
import subprocess
import sys
import tempfile

from s04 import go_environment, verified_upstream
from s04_common import strict_json_loads
from s08_oracle import ROOT, canonical, digest

TOOL = 'tools/s08/inventory/main.go'
REQUEST = 'tools/s08/inventory/requests.json'
SCRIPT = 'scripts/s08_inventory.py'
FROZEN = ROOT / 'data/s08/dependency-closure.json.xz'


def compact(observed):
    """Dictionary encoding only: no call edge or unresolved site is discarded."""
    functions = observed['functions']
    identifiers = sorted({r['id'] for r in functions} | set(observed['external_boundaries']))
    ids = {value: index for index, value in enumerate(identifiers)}
    tables = {key: sorted({r[key] for r in observed['calls']})
              for key in ('file', 'kind', 'signature', 'value')}
    indexes = {key: {v: i for i, v in enumerate(values)} for key, values in tables.items()}
    target_sets = sorted({tuple(ids[t] for t in r['targets']) for r in observed['calls']})
    targets = {v: i for i, v in enumerate(target_sets)}
    calls = sorted({(ids[r['caller']], indexes['file'][r['file']], r['line'], r['column'],
                     indexes['kind'][r['kind']], indexes['signature'][r['signature']],
                     indexes['value'][r['value']], targets[tuple(ids[t] for t in r['targets'])])
                    for r in observed['calls']})
    return dict(version=1, analysis=observed['analysis'], audit_target=observed['audit_target'],
                identifiers=identifiers,
                functions=[dict(r, id=ids[r['id']]) for r in functions],
                roots={k: sorted({ids[v] for v in vs}) for k, vs in observed['roots'].items()},
                external_boundaries=[ids[v] for v in observed['external_boundaries']],
                tables=tables, target_sets=target_sets,
                call_columns=['caller', 'file', 'line', 'column', 'kind', 'signature', 'value', 'targets'],
                calls=calls, unresolved=[i for i, r in enumerate(calls) if not target_sets[r[-1]]],
                interfaces=observed['interfaces'], sources=observed['sources'])


def sites(document):
    """Decode for review and callback coverage validation."""
    for i, row in enumerate(document['calls']):
        caller, file, line, column, kind, signature, value, targets = row
        yield dict(index=i, caller=document['identifiers'][caller],
                   file=document['tables']['file'][file], line=line, column=column,
                   kind=document['tables']['kind'][kind], signature=document['tables']['signature'][signature],
                   value=document['tables']['value'][value],
                   targets=[document['identifiers'][t] for t in document['target_sets'][targets]])


def prepare(directory):
    directory = Path(directory).resolve()
    directory.mkdir(parents=True, exist_ok=False)
    upstream = verified_upstream()
    sources = {name: digest((ROOT / name).read_bytes()) for name in (TOOL, REQUEST, SCRIPT)}
    command = ['go', 'run', '-mod=readonly', str(ROOT / TOOL), '--dir', str(upstream / 'tsc'),
               '--request', str(ROOT / REQUEST)]
    (directory / 'command.json').write_bytes(canonical(command) + b'\n')
    with (directory / 'observations.json').open('wb') as output, (directory / 'stderr').open('wb') as errors:
        result = subprocess.run(command, cwd=upstream / 'tsc', env=go_environment(),
                                stdout=output, stderr=errors, timeout=300)
    if result.returncode:
        raise ValueError(f'typed inventory failed; see {directory}/stderr')
    observed = strict_json_loads((directory / 'observations.json').read_bytes())
    document = compact(observed)
    raw = canonical(document) + b'\n'
    (directory / 'dependency-closure.json.xz').write_bytes(lzma.compress(raw, preset=6))
    verified_upstream()
    if sources != {name: digest((ROOT / name).read_bytes()) for name in sources}:
        raise ValueError('inventory sources changed')
    native = {name: digest((upstream / 'tsc' / name).read_bytes()) for name in observed['sources']}
    report = dict(version=1, pin=strict_json_loads((ROOT / 'data/upstream.json').read_bytes())['pin'],
                  sources=sources, native_sources=native, decoded_sha256=digest(raw),
                  functions=len(document['functions']), calls=len(document['calls']),
                  unresolved_calls=len(document['unresolved']),
                  interfaces={k: len(v) for k, v in observed['interfaces'].items()},
                  scope=observed['analysis'], audit_target=observed['audit_target'])
    (directory / 'report.json').write_bytes(canonical(report) + b'\n')
    print(json.dumps({k: v for k, v in report.items() if k not in ('sources', 'native_sources')}))
    return document, report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('operation', choices=('prepare', 'check', 'freeze'))
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    if args.operation == 'prepare':
        if args.output is None:
            parser.error('prepare requires --output')
        prepare(args.output)
        return
    with tempfile.TemporaryDirectory(prefix='s08-inventory-', dir=ROOT / 'target') as tmp:
        directory = Path(tmp) / 'capture'
        prepare(directory)
        for source, dest in ((directory / 'dependency-closure.json.xz', FROZEN),
                             (directory / 'report.json', FROZEN.with_name('dependency-closure-report.json'))):
            if args.operation == 'freeze':
                dest.write_bytes(source.read_bytes())
            elif source.read_bytes() != dest.read_bytes():
                raise ValueError(f'typed inventory drift: {dest}')


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, RuntimeError, KeyError, TypeError, subprocess.TimeoutExpired) as error:
        print(f'S08 inventory failed: {error}', file=sys.stderr)
        raise SystemExit(1) from error
