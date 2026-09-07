#!/usr/bin/env python3
"""Freeze or verify direct pinned Go AST utility observations."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT=Path(__file__).resolve().parents[3]
sys.path.insert(0,str(ROOT/'scripts'))
from s04 import go_environment, verified_upstream

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check',action='store_true')
    args=parser.parse_args()
    upstream=verified_upstream()
    adapter=Path(__file__).with_name('export_test.go')
    with tempfile.TemporaryDirectory(prefix='s06-middle-') as temporary:
        directory=Path(temporary)
        overlay=directory/'overlay.json'
        overlay.write_text(json.dumps({'Replace':{str(upstream/'tsc/internal/ast/s06_middle_test.go'):str(adapter)}}))
        subprocess.run(['go','test','-overlay',str(overlay),'./internal/ast','-run','^TestS06MiddleUtilities$','-count=1'],cwd=upstream/'tsc',env={**go_environment(),'S06_MIDDLE_OUTPUT':str(directory)},check=True)
        outputs={name:(directory/name).read_bytes() for name in ('ast-utilities-middle-kinds.bin','ast-utilities-middle-behaviors.tsv')}
    verified_upstream()
    manifest={'upstream_pin':json.loads((ROOT/'data/upstream.json').read_bytes())['pin'],
              'source_sha256':hashlib.sha256((upstream/'tsc/internal/ast/utilities.go').read_bytes()).hexdigest(),
              'kind_inputs':65536,'kind_predicates':27,
              'behavior_rows':len(outputs['ast-utilities-middle-behaviors.tsv'].splitlines()),
              'outputs':{name:hashlib.sha256(raw).hexdigest() for name,raw in outputs.items()}}
    outputs['ast-utilities-middle.json']=(json.dumps(manifest,indent=2,sort_keys=True)+'\n').encode()
    for name,raw in outputs.items():
        path=ROOT/'data/s06'/name
        if args.check:
            if not path.exists() or path.read_bytes()!=raw:raise ValueError('Go AST utility observation drift: '+name)
        else:path.write_bytes(raw)
    print(f"65536 signed kinds x27 predicates; {manifest['behavior_rows']} direct Go behavior rows")

if __name__=='__main__':
    try:main()
    except (OSError,ValueError,subprocess.SubprocessError) as error:raise SystemExit(str(error)) from None
