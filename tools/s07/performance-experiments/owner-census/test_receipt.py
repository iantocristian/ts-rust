"""Reject forged success, unexplained recovery and unbound replay dependencies."""
import copy
import json
from pathlib import Path
import tempfile
import unittest
from subprocess import CompletedProcess
from unittest.mock import patch
import probe
import receipt
import replay
from test_validate import fixture


class ReceiptContracts(unittest.TestCase):
    def setUp(self):
        self.raw, self.stdout, self.stderr = b'{}\n', b'{"files":1}\n', b''
        self.tools = {'observer.py': '1' * 64}
        self.build = '2' * 64
        self.normal = {'version': 1, 'build_manifest_sha256': self.build, 'child_returncode': 0,
            'raw_sha256': receipt.sha(self.raw), 'stdout_sha256': receipt.sha(self.stdout),
            'stderr_sha256': receipt.sha(self.stderr), 'capture_tool_inputs': self.tools,
            'command': ['/immutable/child', '/immutable/inputs', '/capture/owners.ndjson']}

    def validate(self, value):
        return receipt.validate_receipt(value, self.build, self.raw, self.stdout, self.stderr, self.tools)

    def test_recorded_success_and_documented_unknown_are_distinct(self):
        self.assertEqual(self.validate(self.normal), 'recorded_success')
        recovered = copy.deepcopy(self.normal)
        del recovered['command']
        recovered.update(child_returncode=None, recovery_note='Metadata failed after raw output; original exit receipt was unavailable.')
        self.assertEqual(self.validate(recovered), 'documented_unknown_exit_recovery')

    def test_failed_or_boolean_exit_status_cannot_be_finalized(self):
        for code in (1, -9, False, True, '0'):
            bad = copy.deepcopy(self.normal); bad['child_returncode'] = code
            with self.assertRaisesRegex(ValueError, 'exit status'): self.validate(bad)

    def test_unknown_exit_requires_nonempty_explicit_recovery(self):
        bad = copy.deepcopy(self.normal); del bad['command']; bad['child_returncode'] = None
        for note in (None, '', '  ', 3):
            if note is not None: bad['recovery_note'] = note
            with self.assertRaisesRegex(ValueError, 'recovery'): self.validate(bad)

    def test_receipt_binds_raw_output_build_and_capture_tools(self):
        for key in ('raw_sha256', 'stdout_sha256', 'stderr_sha256', 'build_manifest_sha256'):
            bad = copy.deepcopy(self.normal); bad[key] = '0' * 64
            with self.assertRaisesRegex(ValueError, 'identity'): self.validate(bad)
        bad = copy.deepcopy(self.normal); bad['capture_tool_inputs'] = {'other.py': '1' * 64}
        with self.assertRaisesRegex(ValueError, 'tool inputs'): self.validate(bad)
        bad['capture_tool_inputs'] = {}
        with self.assertRaisesRegex(ValueError, 'tool receipt'): self.validate(bad)

    def test_retained_build_test_logs_are_bound_to_build_inventory(self):
        fields = {'test.stdout': 'tests_stdout', 'test.stderr': 'tests_stderr', 'cargo.stderr': 'build_stderr'}
        provenance = {field: name for name, field in fields.items()}
        build = {'inventory': {name: {'sha256': receipt.sha(name.encode())} for name in fields}}
        replay.verify_build_logs(provenance, build)
        for field in fields.values():
            bad = {**provenance, field: 'replacement log'}
            with self.assertRaisesRegex(ValueError, 'log identity'): replay.verify_build_logs(bad, build)

    def test_replay_source_closure_includes_actual_json_helper(self):
        expected = {name: receipt.sha((replay.ROOT/name).read_bytes()) for name in replay.REPLAY_INPUTS}
        replay.verify_replay_inputs(expected)
        bad = dict(expected); del bad['scripts/s04_common.py']
        with self.assertRaisesRegex(ValueError, 'source closure'): replay.verify_replay_inputs(bad)
        bad = {**expected, 'scripts/s04_common.py': '0' * 64}
        with self.assertRaisesRegex(ValueError, 'implementation changed'): replay.verify_replay_inputs(bad)

    def test_finalizer_itself_rejects_a_nonzero_persisted_receipt(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root/'owners.ndjson').write_bytes(self.raw)
            (root/'child.stdout').write_bytes(self.stdout)
            (root/'child.stderr').write_bytes(self.stderr)
            bad = {**self.normal, 'child_returncode': 1}
            (root/'capture-receipt.json').write_text(json.dumps(bad))
            with patch.object(probe, 'validate_build', return_value={'expected_work': {}}), \
                 patch.object(probe, 'validate_capture', return_value=[]):
                with self.assertRaisesRegex(ValueError, 'exit status'):
                    probe.finalize(root, root/'unused-build', self.build)
            self.assertFalse((root/'manifest.json').exists())

    def failed_child_then_recovery(self, emit_raw):
        expected = dict(files=1, loaded_bytes=1, nodes=1, symbols=1,
            parse_diagnostics=0, bind_diagnostics=0, loaded_input_sha256='a'*64)
        child = dict(version=1, diagnostic_only=True, runtime='rust', workers=1,
            domain='untimed', **expected, bound_in_place_files=1, fallback_files=0)
        raw = (json.dumps(fixture())+'\n').encode()
        stdout, stderr = (json.dumps(child)+'\n').encode(), b'failed after observation\n'
        self.assertEqual(len(probe.validate_capture(raw, child, expected)), 1)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); output = root/'capture'; build = root/'build'
            manifest = {'expected_work': expected, 'artifact': {'path': 'artifacts/child'}}
            tools = {'scripts/s04_common.py': probe.runner.digest(probe.ROOT/'scripts/s04_common.py')}
            def fail(argv, **kwargs):
                if emit_raw: Path(argv[2]).write_bytes(raw)
                return CompletedProcess(argv, 17, stdout=stdout, stderr=stderr)
            with patch.object(probe, 'validate_build', return_value=manifest), \
                 patch.object(probe.runner, 'validate_inputs'), \
                 patch.object(probe, 'tool_files', return_value=tools), \
                 patch.object(probe, 'native_environment', return_value={}), \
                 patch.object(probe.subprocess, 'run', side_effect=fail):
                with self.assertRaisesRegex(ValueError, 'child failed'):
                    probe.capture(build, self.build, output)
                persisted = (output/'capture-receipt.json').read_bytes()
                observed = json.loads(persisted)
                self.assertEqual(observed['child_returncode'], 17)
                self.assertEqual(observed['raw_sha256'], receipt.sha(raw) if emit_raw else None)
                self.assertEqual(observed['stdout_sha256'], receipt.sha(stdout))
                self.assertEqual(observed['stderr_sha256'], receipt.sha(stderr))
                with self.assertRaisesRegex(ValueError, 'exit status'):
                    probe.finalize(output, build, self.build, recovery_note='Attempted metadata recovery')
                self.assertEqual((output/'capture-receipt.json').read_bytes(), persisted)
                self.assertFalse((output/'manifest.json').exists())

    def test_failed_child_with_complete_valid_raw_cannot_be_recovered_as_unknown(self):
        self.failed_child_then_recovery(True)

    def test_failed_child_without_raw_still_persists_and_rejects_its_exit_status(self):
        self.failed_child_then_recovery(False)


if __name__ == '__main__': unittest.main()
