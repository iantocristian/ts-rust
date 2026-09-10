"""Exactly one full one-worker backing allocation diagnostic; no CPU/RSS screen."""
import fcntl
import shutil
import subprocess
import sys
import time
from pathlib import Path
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
BUILD = ROOT / 'target/s07-bis/allocation-traffic-build'
FROZEN = ROOT / 'target/s07-bis/text-processing-clean-candidate'
FROZEN_SHA = 'bab5c54fc779b86ebdb1f2bd5a2289dcd7a4066612eb004ceb02e16e1c906fdc'
OUT = ROOT / 'target/s07-bis/allocation-traffic-capture'
sys.path.insert(0, str(ROOT / 'tools/s07/performance-experiments'))
import runner
from s04_common import strict_json_loads
from s07_benchmark import CACHE, native_environment
from s07_benchmark_measure import REPORT_FIELDS, COUNTERS, reject_concurrent_builds

helper_names = ('tools/s07/performance-experiments/runner.py', 'scripts/s07_benchmark.py',
                'scripts/s07_benchmark_measure.py', 'scripts/s07_benchmark_inputs.py', 'scripts/s04_common.py')
helper_before = {name: runner.digest(ROOT / name) for name in helper_names}
capture_before = runner.digest(__file__)

if len(sys.argv) != 2:
    raise SystemExit('capture.py <reviewed build manifest sha256>')
with (CACHE / 's07-benchmark/measurement.lock').open('a+') as lock:
    fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    reject_concurrent_builds()
    expected = runner.validate_bundle(FROZEN, FROZEN_SHA)['expected_work']
    runner.validate_inputs(FROZEN, expected)
    assert runner.digest(BUILD / 'manifest.json') == sys.argv[1]
    build = strict_json_loads((BUILD / 'manifest.json').read_bytes())
    assert build['kind'] == 'allocation-traffic-build' and build['diagnostic_only'] is True
    assert build['expected_work'] == expected and runner.inventory(BUILD) == build['inventory']
    assert all(not p.is_symlink() and not (p.stat().st_mode & 0o222) for p in (BUILD, *BUILD.rglob('*')))
    required = ('calibration', 'symbol-table-tests', 'list_buffer--tests', 'factory-tests-tests', 'lists-tests-tests', 'clippy')
    for stage in required:
        record = strict_json_loads((BUILD / (stage + '.exit.json')).read_bytes())
        assert record == {'returncode': 0, 'expected_returncode': 0}, stage
    artifact = build['artifacts']['allocation']
    binary = BUILD / artifact['path']
    assert runner.digest(binary) == artifact['sha256']
    OUT.mkdir(exist_ok=False)
    argv = [str(binary), str(BUILD / 'inputs.json'), '1']
    runner.write_json(OUT / 'command.json', argv)
    started = time.monotonic_ns()
    with (OUT / 'full.stdout').open('wb') as stdout, (OUT / 'full.stderr').open('wb') as stderr:
        try:
            result = subprocess.run(argv, cwd=ROOT, env=native_environment(), stdout=stdout, stderr=stderr, timeout=600)
        except subprocess.TimeoutExpired:
            runner.write_json(OUT / 'exit.json', {'timeout': True, 'limit_seconds': 600})
            raise
    exit_record = {'returncode': result.returncode, 'process_elapsed_ns': time.monotonic_ns() - started,
                   'binary_sha256': runner.digest(binary), 'inputs_sha256': runner.digest(BUILD / 'inputs.json'),
                   'stdout_sha256': runner.digest(OUT / 'full.stdout'), 'stderr_sha256': runner.digest(OUT / 'full.stderr')}
    runner.write_json(OUT / 'exit.json', exit_record)
    assert result.returncode == 0
    report = strict_json_loads((OUT / 'full.stdout').read_bytes())
    assert set(report) == REPORT_FIELDS and report['workers'] == 1 and type(report['workers']) is int
    assert type(report['version']) is int and report['version'] == 1 and report['goroutines_ready'] is None
    assert type(report['cpu_capacity']) is int and report['cpu_capacity'] >= 1
    for key in ('wall_time_ns', 'startup_ns', 'preload_ns', 'worker_setup_ns'):
        assert type(report[key]) is int and 0 <= report[key] <= 600_000_000_000
    assert 0 < report['wall_time_ns'] <= exit_record['process_elapsed_ns']
    for key in COUNTERS:
        assert type(report[key]) is int and report[key] == expected[key], key
    assert report['loaded_input_sha256'] == expected['loaded_input_sha256']
    lines = [strict_json_loads(line) for line in (OUT / 'full.stderr').read_bytes().splitlines() if line]
    assert len(lines) == 2
    diagnostic, late_counters = lines
    keys = {'schema', 'diagnostic_only', 'domain', 'workers', *COUNTERS, 'loaded_input_sha256',
            'bound_in_place_files', 'fallback_files', 'element_bytes', 'retained_fields', 'retained',
            'traffic_fields', 'traffic', 'memory', 'backing_families', 'backing_phases', 'backing_fields',
            'backing_traffic', 'phase_process_windows', 'phase_window_fields', 'phase_window_scope'}
    assert set(diagnostic) == keys
    assert type(diagnostic['schema']) is int and diagnostic['schema'] == 1
    assert diagnostic['diagnostic_only'] is True and diagnostic['domain'] == 'current-backing-traffic'
    for key in ('workers', *COUNTERS, 'loaded_input_sha256'):
        assert type(diagnostic[key]) is type(report[key]) and diagnostic[key] == report[key], key
    assert type(diagnostic['bound_in_place_files']) is int and type(diagnostic['fallback_files']) is int
    assert diagnostic['bound_in_place_files'] == expected['files'] and diagnostic['fallback_files'] == 0
    assert len(diagnostic['element_bytes']) == 3 and all(type(v) is int and v > 0 for v in diagnostic['element_bytes'])
    for domain, count in [('retained', 31), ('traffic', 26)]:
        fields, values = diagnostic[domain + '_fields'], diagnostic[domain]
        assert len(fields) == len(values) == count and len(set(fields)) == count
        assert all(type(k) is str and k for k in fields)
        assert all(type(v) is int and v >= 0 for v in values)
    assert diagnostic['backing_families'] == [
        'core_header_pages', 'core_header_directories', 'auxiliary_slot_pages', 'auxiliary_slot_directories',
        'symbol_pages', 'symbol_directories', 'flow_pages', 'flow_directories',
        'flow_list_pages', 'flow_list_directories', 'other_arena_pages', 'other_arena_directories',
        'payload_row_pages', 'payload_row_directories', 'edge_pages', 'edge_directories',
        'text_pool_entries', 'text_pool_free_slots', 'eager_parser_list_buffers']
    assert diagnostic['backing_phases'] == ['unscoped', 'parse', 'bind_publish']
    assert diagnostic['backing_fields'] == ['requested_bytes', 'replaced_bytes', 'allocation_calls', 'release_bytes']
    values = diagnostic['backing_traffic']
    assert len(values) == 3 * 19 * 4 and all(type(v) is int and v >= 0 for v in values)
    assert diagnostic['phase_window_fields'] == ['requested_bytes', 'positive_live_change', 'negative_live_change', 'files']
    values = diagnostic['phase_process_windows']
    assert len(values) == 8 and all(type(v) is int and v >= 0 for v in values)
    assert values[3] == values[7] == expected['files']
    assert diagnostic['phase_window_scope'] == 'process-wide cap deltas while the sole worker parses/binds; concurrent main-thread traffic may be included'
    memory = diagnostic['memory']
    assert set(memory) == {'total_before', 'total_endpoint', 'live_before', 'live_endpoint'}
    assert all(type(v) is int and v >= 0 for v in memory.values())
    assert memory['total_endpoint'] - memory['total_before'] == report['allocated_bytes']
    assert set(late_counters) == {'retained_requested_bytes', 'peak_requested_bytes'}
    assert all(type(v) is int and v >= 0 for v in late_counters.values())
    # Late counters include join/report activity, so never substitute them for
    # the earlier explicit pipeline endpoint or family snapshot.
    assert runner.inventory(BUILD) == build['inventory']
    runner.write_json(OUT / 'observations.json', {'report': report, 'diagnostic': diagnostic, 'late_counters': late_counters})
    assert capture_before == runner.digest(__file__)
    assert helper_before == {name: runner.digest(ROOT / name) for name in helper_names}
    shutil.copyfile(__file__, OUT / 'capture.py')
    helpers = {}
    for name in helper_names:
        runner.copy_file(ROOT / name, OUT / 'tool-snapshot' / name)
        helpers[name] = runner.digest(ROOT / name)
    manifest = {'version': 1, 'kind': 'allocation-traffic-capture', 'diagnostic_only': True,
                'build_manifest_sha256': sys.argv[1], 'frozen_manifest_sha256': FROZEN_SHA,
                'expected_work': expected, 'helper_inventory': helpers, 'helper_inventory_before': helper_before,
                'capture_sha256_before': capture_before, 'invocations': 1,
                'scope': 'one-worker backing requested/replaced/released/live attribution and process phase windows; no CPU/RSS acceptance'}
    print('capture manifest ' + runner.seal(OUT, manifest), flush=True)
