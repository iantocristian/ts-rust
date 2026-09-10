#!/usr/bin/env python3
"""Archive the already sealed census; never replace a package or run a child."""
import argparse
import fcntl
import gzip
import io
import json
from pathlib import Path
import tarfile
import probe
import replay


def validation_tools(output,previous_replay=None):
    names=replay.REPLAY_INPUTS | {str((probe.HERE/name).relative_to(probe.ROOT)) for name in
        ('probe.py','package.py','test_archive.py','test_receipt.py','test_validate.py')}
    payloads={'current/'+name:(probe.ROOT/name).read_bytes() for name in names}
    if previous_replay is not None: payloads['previous/replay.py']=previous_replay
    members={}
    with tarfile.open(output/'validation-tools.tar.xz','w:xz',preset=6) as archive:
        for name,raw in sorted(payloads.items()):
            member=tarfile.TarInfo(name);member.size=len(raw);member.mode=0o444;member.mtime=0
            archive.addfile(member,io.BytesIO(raw));members[name]={'bytes':len(raw),'sha256':replay.sha(raw)}
    return members


def revise_existing(output,previous_replay):
    """Preserve the version-1 package bytes; add a stricter replay envelope."""
    old_raw=(output/'manifest.json').read_bytes();old=probe.strict_json_loads(old_raw)
    if old.get('version')!=1 or (output/'prior-envelope.json').exists():
        raise ValueError('expected an unrevised version-1 census package')
    for name,identity in old['files'].items():
        raw=(output/name).read_bytes()
        if {'bytes':len(raw),'sha256':replay.sha(raw)}!=identity:
            raise ValueError('original package changed: '+name)
    previous_code=previous_replay.read_bytes()
    if replay.sha(previous_code)!=old['replay_inputs']['replay.py']:
        raise ValueError('previous replay source differs from original envelope')
    if replay.sha((probe.HERE/'validate.py').read_bytes())!=old['replay_inputs']['validate.py']:
        raise ValueError('original raw-record validator changed')
    old_replay=(output/'replay.json').read_bytes()
    (output/'prior-envelope.json').write_bytes(old_raw)
    (output/'prior-replay.json').write_bytes(old_replay)
    members=validation_tools(output,previous_code)
    envelope={**old,'version':2,
        'replay_inputs':{name:replay.sha((probe.ROOT/name).read_bytes()) for name in sorted(replay.REPLAY_INPUTS)},
        'validation_tool_members':members,
        'files':{**old['files'],**{name:{'bytes':(output/name).stat().st_size,'sha256':replay.sha((output/name).read_bytes())}
            for name in ('validation-tools.tar.xz','prior-envelope.json','prior-replay.json')}},
        'revision':{'previous_manifest_sha256':replay.sha(old_raw),'previous_replay_sha256':replay.sha(old_replay),
            'note':'Stricter archive-only validation of replay source closure, original build/test logs and recovery receipts. Original raw records, provenance, build/capture manifests and helper archive are unchanged; no native child was rerun.'}}
    probe.runner.write_json(output/'manifest.json',envelope)
    receipt=replay.replay(output)
    probe.runner.write_json(output/'replay.json',receipt)
    return receipt


def package(output,build_dir,build_sha,capture_dir,capture_sha):
    if probe.runner.digest(capture_dir/"manifest.json")!=capture_sha: raise ValueError("capture manifest identity changed")
    probe.replay(capture_dir,build_dir,build_sha)
    build_raw=(build_dir/'manifest.json').read_bytes();build=json.loads(build_raw)
    capture_raw=(capture_dir/'manifest.json').read_bytes();capture=json.loads(capture_raw)
    baseline_raw=(probe.DEFAULT_BASE/'manifest.json').read_bytes()
    if replay.sha(baseline_raw)!=build['baseline_manifest_sha256']: raise ValueError('baseline identity changed')
    output.mkdir(parents=True,exist_ok=False)
    (output/'owners.ndjson.gz').write_bytes(gzip.compress((capture_dir/'owners.ndjson').read_bytes(),mtime=0))
    provenance={'build_manifest':build,'build_manifest_raw':build_raw.decode(),
        'capture_manifest':capture,'capture_manifest_raw':capture_raw.decode(),
        'baseline_manifest_raw':baseline_raw.decode(),
        'child_stdout':(capture_dir/'child.stdout').read_text(),'child_stderr':(capture_dir/'child.stderr').read_text(),
        'tests_stdout':(build_dir/'test.stdout').read_text(),'tests_stderr':(build_dir/'test.stderr').read_text(),
        'build_stderr':(build_dir/'cargo.stderr').read_text()}
    (output/'provenance.json.gz').write_bytes(gzip.compress(json.dumps(provenance,sort_keys=True,separators=(',',':')).encode(),mtime=0))
    members={}
    with tarfile.open(output/'tools.tar.xz','w:xz',preset=6) as archive:
        payloads={'observer.patch':(build_dir/'source/owner-census.patch').read_bytes()}
        for prefix,source in [('build-tools',build_dir/'tool-snapshot'),('capture-tools',capture_dir/'tool-snapshot'),('finalizer-tools',capture_dir/'finalizer-tools')]:
            payloads.update({prefix+'/'+str(path.relative_to(source)):path.read_bytes() for path in source.rglob('*') if path.is_file()})
        for name,raw in sorted(payloads.items()):
            member=tarfile.TarInfo(name);member.size=len(raw);member.mode=0o444;member.mtime=0
            archive.addfile(member,io.BytesIO(raw));members[name]={'bytes':len(raw),'sha256':replay.sha(raw)}
    validation_members=validation_tools(output)
    envelope={'version':2,'diagnostic_only':True,'capture_manifest_sha256':replay.sha(capture_raw),
        'files':{p.name:{'bytes':p.stat().st_size,'sha256':replay.sha(p.read_bytes())} for p in output.iterdir()},
        'tool_members':members,'validation_tool_members':validation_members,
        'replay_inputs':{name:replay.sha((probe.ROOT/name).read_bytes()) for name in sorted(replay.REPLAY_INPUTS)}}
    probe.runner.write_json(output/'manifest.json',envelope)
    receipt=replay.replay(output)
    probe.runner.write_json(output/'replay.json',receipt)
    return receipt

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--build',type=Path);p.add_argument('--build-sha')
    p.add_argument('--capture-sha');p.add_argument('--capture',type=Path);p.add_argument('--output',type=Path,required=True)
    p.add_argument('--revise-with-previous-replay',type=Path)
    a=p.parse_args()
    if a.revise_with_previous_replay:
        print(json.dumps(revise_existing(a.output.resolve(),a.revise_with_previous_replay.resolve()),indent=2))
        raise SystemExit(0)
    if not all((a.build,a.build_sha,a.capture,a.capture_sha)):
        p.error('new packages require --build, --build-sha, --capture and --capture-sha')
    with (probe.runner.CACHE/'s07-benchmark/measurement.lock').open('a+') as lock:
        fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
        print(json.dumps(package(a.output.resolve(),a.build.resolve(),a.build_sha,a.capture.resolve(),a.capture_sha),indent=2))
