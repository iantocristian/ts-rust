#!/usr/bin/env python3
"""Replay the durable census without binaries, workload files or native execution."""
import gzip
import hashlib
import io
import json
from pathlib import Path
import sys
import tarfile

HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[3]/'scripts'))
from s04_common import strict_json_loads
from validate import aggregate,validate_capture
from receipt import validate_receipt

ROOT=HERE.parents[3]
REPLAY_INPUTS={str((HERE/name).relative_to(ROOT)) for name in ('replay.py','validate.py','receipt.py')} | {'scripts/s04_common.py'}


def sha(raw): return hashlib.sha256(raw).hexdigest()


def verify_replay_inputs(inputs):
    if set(inputs)!=REPLAY_INPUTS: raise ValueError('replay source closure differs')
    for name,expected in inputs.items():
        if sha((ROOT/name).read_bytes())!=expected: raise ValueError('replay implementation changed: '+name)


def verify_build_logs(provenance,build):
    for name,field in [('test.stdout','tests_stdout'),('test.stderr','tests_stderr'),('cargo.stderr','build_stderr')]:
        if sha(provenance[field].encode())!=build['inventory'][name]['sha256']:
            raise ValueError('retained build/test log identity differs: '+name)


def replay(directory):
    envelope=strict_json_loads((directory/'manifest.json').read_bytes())
    if envelope.get('version')!=2 or envelope.get('diagnostic_only') is not True:
        raise ValueError('expected explicit version-2 replay envelope')
    for name,identity in envelope['files'].items():
        raw=(directory/name).read_bytes()
        if len(raw)!=identity['bytes'] or sha(raw)!=identity['sha256']:
            raise ValueError('packaged artifact changed: '+name)
    verify_replay_inputs(envelope['replay_inputs'])
    provenance=strict_json_loads(gzip.decompress((directory/'provenance.json.gz').read_bytes()))
    raw=gzip.decompress((directory/'owners.ndjson.gz').read_bytes())
    build=provenance['build_manifest'];capture=provenance['capture_manifest']
    if sha(provenance['build_manifest_raw'].encode())!=capture['build_manifest_sha256'] or strict_json_loads(provenance['build_manifest_raw'])!=build:
        raise ValueError('build manifest identity differs')
    if sha(provenance['capture_manifest_raw'].encode())!=envelope['capture_manifest_sha256'] or strict_json_loads(provenance['capture_manifest_raw'])!=capture:
        raise ValueError('capture manifest identity differs')
    if sha(provenance['baseline_manifest_raw'].encode())!=build['baseline_manifest_sha256']:
        raise ValueError('A0-b baseline manifest identity differs')
    baseline=strict_json_loads(provenance['baseline_manifest_raw'])
    if baseline['expected_work']!=build['expected_work'] or baseline['source_fingerprint']!=build['baseline_source_fingerprint'] or baseline['revision']!=build['baseline_revision']:
        raise ValueError('build does not describe the retained A0-b baseline')
    if sha(raw)!=capture['raw_sha256'] or len(raw)!=capture['raw_bytes']:
        raise ValueError('raw owner census changed')
    child=strict_json_loads(provenance['child_stdout'])
    if child!=capture['child']: raise ValueError('raw child protocol differs')
    for name,field in [('child.stdout','child_stdout'),('child.stderr','child_stderr')]:
        if sha(provenance[field].encode())!=capture['inventory'][name]['sha256']:
            raise ValueError('raw child output identity differs')
    verify_build_logs(provenance,build)
    receipt_status=validate_receipt(capture['receipt'],capture['build_manifest_sha256'],raw,
        provenance['child_stdout'].encode(),provenance['child_stderr'].encode(),capture['tool_inputs'])
    records=validate_capture(raw,child,build['expected_work'])
    if aggregate(records)!=capture['totals']: raise ValueError('derived totals differ')
    with tarfile.open(directory/'tools.tar.xz') as archive:
        members=archive.getmembers()
        if any(not member.isfile() for member in members) or len(members)!=len(envelope['tool_members']) or len({m.name for m in members})!=len(members):
            raise ValueError('tool snapshot member inventory differs')
        for member in members:
            identity=envelope['tool_members'].get(member.name)
            data=archive.extractfile(member).read()
            if identity is None or len(data)!=identity['bytes'] or sha(data)!=identity['sha256']:
                raise ValueError('tool snapshot changed: '+member.name)
        for prefix,inputs in [('build-tools',build['tool_inputs']),('capture-tools',capture['tool_inputs']),('finalizer-tools',capture['finalizer_tool_inputs'])]:
            for name,expected in inputs.items():
                if envelope['tool_members'][prefix+'/'+name]['sha256']!=expected:
                    raise ValueError('recorded helper bytes differ: '+name)
        patch=archive.extractfile('observer.patch').read()
        if sha(patch)!=build['patch_sha256']: raise ValueError('additive observer patch changed')
    with tarfile.open(directory/'validation-tools.tar.xz') as archive:
        members=archive.getmembers();expected=envelope['validation_tool_members']
        if len(members)!=len(expected) or len({m.name for m in members})!=len(members) or any(not m.isfile() for m in members):
            raise ValueError('stricter validation tool inventory differs')
        for member in members:
            data=archive.extractfile(member).read()
            if expected.get(member.name)!={'bytes':len(data),'sha256':sha(data)}:
                raise ValueError('stricter validation tool snapshot changed: '+member.name)
        for name,digest in envelope['replay_inputs'].items():
            if expected['current/'+name]['sha256']!=digest:
                raise ValueError('replay source differs from stricter tool snapshot')
    revision=envelope.get('revision')
    if revision is not None:
        previous_raw=(directory/'prior-envelope.json').read_bytes()
        previous=strict_json_loads(previous_raw)
        if sha(previous_raw)!=revision['previous_manifest_sha256'] or previous['capture_manifest_sha256']!=envelope['capture_manifest_sha256']:
            raise ValueError('prior capture envelope identity changed')
        if any(envelope['files'].get(name)!=identity for name,identity in previous['files'].items()):
            raise ValueError('revision changed original census package bytes')
        if sha((directory/'prior-replay.json').read_bytes())!=revision['previous_replay_sha256']:
            raise ValueError('prior replay record changed')
    return {'version':2,'diagnostic_only':True,'files':len(records),'raw_sha256':sha(raw),
        'build_manifest_sha256':capture['build_manifest_sha256'],'capture_manifest_sha256':envelope['capture_manifest_sha256'],
        'physical_nodes':sum(r['arenas']['core_nodes']['len'] for r in records),
        'physical_node_backings':sum(len(r['node_backings']['physical_lengths_in_aux_order']) for r in records),
        'physical_node_backing_slots':sum(sum(r['node_backings']['physical_lengths_in_aux_order']) for r in records),
        'tool_members_verified':len(envelope['tool_members']),
        'validation_tool_members_verified':len(envelope['validation_tool_members']),
        'receipt_status':receipt_status,
        'native_child_reexecuted':False}

if __name__=='__main__':
    import argparse
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('directory',type=Path,nargs='?',default=HERE/'results/2026-09-08')
    print(json.dumps(replay(p.parse_args().directory),indent=2,sort_keys=True))
