"""Package the one diagnostic and retained failures; never rerun or overwrite it."""
import hashlib
import json
from pathlib import Path
import tarfile

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / 'tools/s07/performance-experiments/results/2026-09-09-allocation-traffic'
BUILD_SHA = 'fd9bd1e99907cba381bb70ad8ad3f8f3878530477260482c99a48b0c4d09f26a'
CAPTURE_SHA = '236acf700e24569b562bdc692055d8ffbf8c0ff2212600bea9e135397c073887'
BASELINE_SHA = 'f05a8bff9d53ffc76eac7c0066ff9fc93969516b40b56c2e905cc16264d2fe5a'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify_sealed(path, expected):
    assert digest(path / 'manifest.json') == expected
    manifest = json.loads((path / 'manifest.json').read_text())
    actual = {p.relative_to(path).as_posix() for p in path.rglob('*') if p.is_file()}
    assert actual == set(manifest['inventory']) | {'manifest.json'}
    for name, entry in manifest['inventory'].items():
        data = (path / name).read_bytes()
        assert len(data) == entry['bytes'] and hashlib.sha256(data).hexdigest() == entry['sha256']
    assert all(not p.is_symlink() and not p.stat().st_mode & 0o222 for p in (path, *path.rglob('*')))
    return manifest


def main():
    assert not OUT.exists(), 'Do not overwrite retained evidence'
    base = ROOT / 'target/s07-bis'
    build = verify_sealed(base / 'allocation-traffic-build', BUILD_SHA)
    capture = verify_sealed(base / 'allocation-traffic-capture', CAPTURE_SHA)
    assert capture['invocations'] == 1 and capture['build_manifest_sha256'] == BUILD_SHA
    baseline = OUT.parent / '2026-09-09-compact-text-processing/review.tar.xz'
    assert digest(baseline) == BASELINE_SHA
    files = {ROOT / 'docs/S07-bis-allocation-traffic.md'}
    files.update(p for p in Path(__file__).resolve().parent.iterdir() if p.is_file())
    for path in base.glob('allocation-traffic*'):
        if path.is_dir():
            files.update(p for p in path.rglob('*') if p.is_file() and '__pycache__' not in p.parts)
        elif path.is_file() and path.name != 'allocation-traffic-package.log':
            files.add(path)
    files = sorted(files)
    assert all(not path.is_symlink() and path.resolve().is_relative_to(ROOT) for path in files)
    members = {path.relative_to(ROOT).as_posix(): {'bytes': path.stat().st_size, 'sha256': digest(path)} for path in files}
    OUT.mkdir()
    archive = OUT / 'review.tar.xz'
    with tarfile.open(archive, 'w:xz', preset=3) as contents:
        for path in files:
            contents.add(path, arcname=path.relative_to(ROOT).as_posix(), recursive=False)
    with tarfile.open(archive, 'r:xz') as contents:
        assert len(contents.getnames()) == len(members) and set(contents.getnames()) == set(members)
        for member in contents:
            assert member.isfile() and not member.issym() and not member.islnk()
            data = contents.extractfile(member).read()
            expected = members[member.name]
            assert len(data) == expected['bytes'] and hashlib.sha256(data).hexdigest() == expected['sha256']
    assert members == {path.relative_to(ROOT).as_posix(): {'bytes': path.stat().st_size, 'sha256': digest(path)} for path in files}
    result = {
        'archive': {'bytes': archive.stat().st_size, 'sha256': digest(archive)},
        'diagnostic_only': True, 'implementation_source_commit': '8f7236eaad350be329ef7ad3089bb5a13509c44e',
        'build_manifest_sha256': BUILD_SHA, 'capture_manifest_sha256': CAPTURE_SHA,
        'allocation_executable_sha256': build['artifacts']['allocation']['sha256'],
        'completed_workload_invocations': 1,
        'baseline_archive': '../2026-09-09-compact-text-processing/review.tar.xz',
        'baseline_archive_sha256': BASELINE_SHA,
        'observer_archive': '../2026-09-09-name-table-attribution/review.tar.xz',
        'observer_archive_sha256': 'dd0783fd71b42f7ae38abf176d7ab62ac3eda22e5a2c45f86d43ac4f136048d6',
        'restore_read_only_directories': ['target/s07-bis/allocation-traffic-build', 'target/s07-bis/allocation-traffic-capture'],
        'scope': 'Exactly one calibrated current-layout allocation diagnostic, including requested/replaced/released/live backing and process phase windows. No CPU/RSS or 209 MB saving claim. Both failed builds and pre-launch sandbox process-inspection failure retained; first failed build stopped before sealing/copying its original driver. Exact successful staged source/binaries and all checks/raw output included. Workload/toolchains external; original freeze supplied by prior archive.',
        'member_count': len(members), 'members': members,
    }
    (OUT / 'archive.json').write_text(json.dumps(result, indent=2, sort_keys=True) + '\n')
    print(json.dumps({key: value for key, value in result.items() if key != 'members'}, indent=2))


if __name__ == '__main__':
    main()
