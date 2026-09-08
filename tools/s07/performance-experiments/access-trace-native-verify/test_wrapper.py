"""Post-capture wrapper failure regressions; no native workload rerun."""
import gzip
import importlib.util
import json
from pathlib import Path
import sys
import time
import unittest
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(HERE))
import run

SPEC = importlib.util.spec_from_file_location('native_wrapper_original_fixtures', HERE.with_name('access-trace') / 'test_verify.py')
fixtures = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = fixtures
SPEC.loader.exec_module(fixtures)
BINARY = ROOT / 'target/s07-bis/native-verifier-build-1/artifacts/native-verifier'
OUTPUT = ROOT / f'target/s07-bis-native-verify/wrapper-regressions-{time.time_ns()}'
OUTPUT.mkdir()


class WrapperFailures(unittest.TestCase):
    def inputs(self, name):
        directory = OUTPUT / name
        directory.mkdir()
        trace = directory / 'input.gz'
        trace.write_bytes(gzip.compress(fixtures.encoded(fixtures.fixture_records())))
        return directory, trace

    def test_spawn_failure_keeps_explicit_unstarted_failed_receipt(self):
        directory, trace = self.inputs('spawn-failure')
        with patch.object(run.subprocess, 'Popen', side_effect=OSError('injected exec failure')):
            with self.assertRaisesRegex(ValueError, 'native/gzip verification failed'):
                run.verify_gzip(BINARY, run.digest(BINARY), trace, fixtures.REGISTRIES, directory / 'verification')
        receipt = json.loads((directory / 'verification/receipt.json').read_text())
        self.assertIsNone(receipt['returncode'])
        self.assertFalse(receipt['child_started'])
        self.assertFalse(receipt['accepted'])
        self.assertFalse(receipt['gzip_complete'])
        self.assertEqual(receipt['wrapper_error'], 'injected exec failure')
        self.assertEqual((directory / 'verification/child.stdout').read_bytes(), b'')
        self.assertFalse((directory / 'verification/verification.json').exists())

    def test_child_configuration_change_is_rejected_after_successful_decode(self):
        directory, trace = self.inputs('config-change')
        shim = directory / 'change-config'
        shim.write_text('#!' + sys.executable + '\n'
            'import os,sys\nfrom pathlib import Path\n'
            'config=Path(sys.argv[1])\nconfig.write_text(config.read_text()+" ")\n'
            f'os.execv({str(BINARY)!r}, [{str(BINARY)!r}, *sys.argv[1:]])\n')
        shim.chmod(0o755)
        with self.assertRaisesRegex(ValueError, 'configuration changed'):
            run.verify_gzip(shim, run.digest(shim), trace, fixtures.REGISTRIES, directory / 'verification')
        receipt = json.loads((directory / 'verification/receipt.json').read_text())
        self.assertEqual(receipt['returncode'], 0)
        self.assertTrue(receipt['gzip_complete'])
        self.assertTrue(receipt['child_started'])
        self.assertFalse(receipt['accepted'])
        self.assertEqual(receipt['validation_error'], 'native verifier configuration changed')
        self.assertFalse((directory / 'verification/verification.json').exists())


if __name__ == '__main__':
    unittest.main()
