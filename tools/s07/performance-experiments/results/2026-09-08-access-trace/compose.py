#!/usr/bin/env python3
"""Reconcile the frozen capture with its separate verification; no trace decoding.

Run from the repository with the original immutable target bundles present.
This checks receipt provenance and per-file graph/work agreement. Re-executing
the native verifier is a separate operation requiring the original raw trace.
"""
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[5]
sys.path.insert(0, str(ROOT / 'tools/s07/performance-experiments/access-trace'))
import probe

PINS = {
    'access-trace-build-4': '0b585e058ed95e73e77d1d230d92e17c48e80291d7054192d84527beb2baa2fe',
    'access-trace-full-1': 'd37189f0581fa11d52978b7f05d581f4886919c3bb3d0889ef69c7d8923c7c9c',
    'native-verifier-build-1': '1e8bd155938150d3f478cb43e6eed8a7e6f9a9fdf7adb196eecfa95728249732',
    'native-verifier-full-1': 'ddc6a4c990ca6b2f5e39f2e1652fbbc0c1d8da49b677b29c92e3368c56b5c658',
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def read(path):
    return probe.strict_json_loads(path.read_bytes())


def main():
    base = ROOT / 'target/s07-bis'
    records = {}
    for name, sha in PINS.items():
        directory = base / name
        require(probe.digest(directory / 'manifest.json') == sha, name + ': manifest changed')
        record = read(directory / 'manifest.json')
        require(probe.inventory(directory) == record['inventory'], name + ': inventory changed')
        require(all(not p.is_symlink() and not p.stat().st_mode & 0o222
                    for p in (directory, *directory.rglob('*'))), name + ': bundle is mutable')
        records[name] = record
    build, capture, native, verification = (records[name] for name in PINS)
    require(all(probe.digest(ROOT / name) == sha for name, sha in capture['tool_inputs'].items()),
            'executed recorder/helper closure differs from the frozen capture')
    require(build['kind'] == 's07_bis_scoped_access_trace_build', 'wrong recorder build')
    require(native['kind'] == 's07_access_trace_native_verifier_build', 'wrong verifier build')
    require(capture['kind'] == 's07_bis_access_trace_pending_verification'
            and capture['trace_verified'] is False, 'original pending record changed')
    require(verification['kind'] == 's07_access_trace_native_verification', 'wrong verification')
    require(capture['build_manifest_sha256'] == PINS['access-trace-build-4'], 'recorder mismatch')
    require(verification['build_manifest_sha256'] == PINS['native-verifier-build-1'], 'verifier mismatch')
    captured = base / 'access-trace-full-1'
    verified = base / 'native-verifier-full-1'
    summary, trace = read(captured / 'summary.json'), read(verified / 'verification.json')
    receipt, invocation = read(verified / 'receipt.json'), read(verified / 'declaration.json')
    child = read(captured / 'capture-receipt.json')
    require(child == capture['receipt'] and child['child_returncode'] == 0
            and child['compression_error'] is None, 'recording failed')
    control = read(captured / 'control-receipt.json')
    require(control['returncode'] == 0 and control['command'] == [
        str(base / 'access-trace-build-4/artifacts/control'), str(captured / 'inputs.json'),
        '1', '--graphs'], 'control failed or used a different invocation')
    require(build['baseline_manifest_sha256']
            == '3a57976f667b9857891edbe9f76de5261c43480ac8c62d946b4545d9ee19d931'
            and probe.digest(base / 'access-trace-build-4/artifacts/control')
            == '36c8eaf369846e811287be7da517555ca742e40e88dd2599adee31b3450278c5',
            'control differs from the accepted CP1 executable')
    require(receipt['accepted'] is True and receipt['returncode'] == 0
            and receipt['gzip_complete'] is True and receipt['wrapper_error'] is None, 'verification failed')
    require(receipt['binary_sha256'] == native['artifact']['sha256']
            and receipt['build_manifest_sha256'] == verification['build_manifest_sha256'], 'wrong child')
    for key in ('command', 'binary_sha256', 'build_manifest_sha256'):
        require(receipt[key] == invocation[key], 'native declaration/receipt mismatch: ' + key)
    require(receipt['command'] == [str(base / 'native-verifier-build-1/artifacts/native-verifier'),
            str(verified / 'config.json'), *(str(verified / f'registry-{i}.json') for i in range(3))],
            'native child invocation differs')
    require(probe.digest(base / 'native-verifier-build-1' / native['artifact']['path'])
            == receipt['binary_sha256'], 'native executable identity differs')
    require(receipt['verification_sha256'] == verification['verification_sha256']
            == probe.digest(verified / 'verification.json'), 'report changed')
    require(receipt['stdout_sha256'] == probe.digest(verified / 'child.stdout')
            and receipt['stderr_sha256'] == probe.digest(verified / 'child.stderr'), 'child output changed')
    require(receipt['compressed_sha256'] == verification['trace_sha256'] == child['trace_sha256']
            == trace['compressed_sha256'] == invocation['expected_trace_sha256']
            == probe.digest(captured / 'trace.bin.gz')
            and invocation['input_trace'] == str(captured / 'trace.bin.gz'), 'trace mismatch')
    decoded_report = read(verified / 'child.stdout')
    require('compressed_bytes' not in decoded_report and 'compressed_sha256' not in decoded_report,
            'native child claims gzip facts')
    decoded_report.update(compressed_bytes=receipt['compressed_bytes'],
                          compressed_sha256=receipt['compressed_sha256'])
    require(decoded_report == trace, 'wrapper report differs from child plus gzip facts')
    require(receipt['stream_sha256'] == trace['stream_sha256']
            and receipt['stream_bytes'] == child['stream_bytes'], 'decoded stream mismatch')
    require(trace['complete'] is True and trace['full_semantic_replay'] is False, 'wrong coverage')
    require(summary == capture['summary'], 'summary changed')
    require(all(summary[k] == v for k, v in build['expected_work'].items()), 'workload changed')
    require(capture['declaration']['sizing_subset'] is False, 'subset cannot stand for full workload')
    expected = dict(files=summary['files'], nodes=summary['nodes'], symbols=summary['symbols'],
                    source_bytes=summary['loaded_bytes'])
    config = read(verified / 'config.json')
    require(config == invocation['config'] and config['expected'] == trace['totals'] == expected,
            'verification totals/config differ from recording')
    require(config['max_payload'] == capture['declaration']['payload_limit']
            and invocation['compressed_limit'] == capture['declaration']['compressed_limit'], 'limits differ')
    registry_root = base / 'access-trace-build-4/tool-snapshot/tools/s07/performance-experiments/access-trace'
    for index, name in enumerate(('protocol', 'state', 'hooks')):
        require(probe.digest(verified / f'registry-{index}.json')
                == probe.digest(registry_root / f'{name}-registry.json')
                == invocation['registries'][index]['sha256'], 'recorder registry mismatch')
    graphs = probe.graph_match(captured / 'graphs.ndjson', captured / 'control-graphs.ndjson')
    require(graphs == summary['files'] == capture['graph_matches'], 'graph count differs')
    probe.match_trace_work(trace, summary, captured / 'graphs.ndjson')
    result = dict(version=1, diagnostic_only=True, receipt_composition_passed=True,
                  raw_trace_redecoded_by_this_command=False, full_semantic_replay=False,
                  manifests=PINS, summary=summary, graph_matches=graphs,
                  records=trace['records'], payload_bytes=trace['payload_bytes'],
                  stream_bytes=receipt['stream_bytes'], compressed_bytes=trace['compressed_bytes'],
                  trace_sha256=trace['compressed_sha256'], stream_sha256=trace['stream_sha256'],
                  binder_operations=trace['binder_operations'], by_op=trace['by_op'],
                  by_op_shape=trace['by_op_shape'], by_op_site=trace['by_op_site'],
                  unobserved_operation_count=trace['unobserved_operation_count'],
                  composition_script_sha256=probe.digest(Path(__file__)),
                  graph_check_helper_sha256=probe.digest(Path(probe.__file__)))
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == '__main__':
    main()
