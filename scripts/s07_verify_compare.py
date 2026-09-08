"""Compare original option-verifier writes on the complete selected denominator."""
import shutil
from pathlib import Path

from s04 import verified_upstream
from s04_common import command, strict_json_loads
from s06_protocol import canonical
from s07_program_compare import ROOT, digest, differences, identities, input_fingerprints, rust_binary

FIELDS = {'id', 'diagnostics', 'includes', 'blocked'}


def compare(requests, expected, actual):
    ids = identities(requests, 'id', 'option-verifier requests')
    for rows, label in ((expected, 'Go verification'), (actual, 'Rust verification')):
        if identities(rows, 'id', label) != ids:
            raise ValueError(label + ' denominator mismatch')
    results = []
    for identity, left, right in zip(ids, expected, actual):
        if set(left) != FIELDS:
            raise ValueError('Go verifier observation fields changed')
        for key in FIELDS-{'id'}:
            if type(left[key]) is not list:
                raise ValueError('Go verifier field is not an array: ' + key)
        paths = differences(left, right)
        results.append({'id': identity, 'passed': not paths,
                        'differing_paths': paths,
                        'failure': {key: right[key] for key in ('Error', 'Panic') if key in right}})
    return results


def capture(requests, oracle, output):
    requests, oracle, output = map(Path, (requests, oracle, output))
    output.mkdir(parents=True, exist_ok=True)
    manifest_path = oracle.with_suffix('.manifest.json')
    manifest = strict_json_loads(manifest_path.read_bytes())
    pin = strict_json_loads((ROOT/'data/upstream.json').read_bytes())['pin']
    adapters = {name: digest(ROOT/name) for name in ('tools/s07/program/export_test.go', 'tools/s07/verify-options/export_test.go')}
    wanted = dict(pin=pin, requests_sha256=digest(requests), observations_sha256=digest(oracle), adapters=adapters,
                  source_sha256=digest(verified_upstream()/'tsc/internal/compiler/program.go'))
    if manifest != wanted:
        raise ValueError('stale or incompatible original option-verifier manifest')
    rows = strict_json_loads(requests.read_bytes())
    expected = strict_json_loads(oracle.read_bytes())
    ids = identities(rows, 'id', 'requests')
    if identities(expected, 'id', 'Go option verifier') != ids:
        raise ValueError('Go option-verifier denominator mismatch')
    before = input_fingerprints()
    executable, env = rust_binary()
    binary = output/'program-probe'
    shutil.copyfile(executable, binary)
    binary.chmod(0o755)
    actual_path = output/'observations.jsonl'
    command([str(binary.resolve()), str(requests.resolve()), str(actual_path.resolve()), '--verify-options'], cwd=ROOT, env=env)
    actual = [strict_json_loads(line) for line in actual_path.read_bytes().splitlines()]
    comparisons = compare(rows, expected, actual)
    stable = before == input_fingerprints()
    report = dict(schema=1, operation='verify_compiler_options', pin=pin,
                  requests_sha256=digest(requests), oracle_sha256=digest(oracle),
                  oracle_manifest_sha256=digest(manifest_path), rust_sha256=digest(actual_path),
                  binary_sha256=digest(binary), inputs=before, source_stable=stable,
                  required_variants=len(ids), passed_variants=sum(row['passed'] for row in comparisons),
                  rows=comparisons, metrics={'option_verification': stable and all(row['passed'] for row in comparisons)})
    (output/'report.json').write_bytes(canonical(report)+b'\n')
    return report
