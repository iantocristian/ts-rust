import json
import gzip
from pathlib import Path
import shutil
import tempfile
import unittest
import replay

HERE=Path(__file__).resolve().parent
RESULTS=HERE/'results/2026-09-08'


class ArchiveContracts(unittest.TestCase):
    def test_durable_inventory_and_full_protocol_replay(self):
        result=replay.replay(RESULTS)
        self.assertEqual(result['files'],13094)
        self.assertEqual(result['physical_node_backings'],3114989)
        self.assertFalse(result['native_child_reexecuted'])
    def test_missing_and_corrupt_archive_members_fail(self):
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary)
            for path in RESULTS.iterdir(): shutil.copyfile(path,root/path.name)
            target=root/'owners.ndjson.gz';original=target.read_bytes()
            target.unlink()
            with self.assertRaises(FileNotFoundError):replay.replay(root)
            target.write_bytes(original[:-1]+bytes([original[-1]^1]))
            with self.assertRaises(ValueError):replay.replay(root)
    def test_changed_helper_snapshot_inventory_fails(self):
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary)
            for path in RESULTS.iterdir(): shutil.copyfile(path,root/path.name)
            p=root/'manifest.json';envelope=json.loads(p.read_text())
            first=next(iter(envelope['tool_members']))
            envelope['tool_members'][first]['sha256']='0'*64
            p.write_text(json.dumps(envelope))
            with self.assertRaises(ValueError):replay.replay(root)
    def test_changed_retained_test_log_fails_even_after_outer_checksum_update(self):
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary)
            for path in RESULTS.iterdir(): shutil.copyfile(path,root/path.name)
            artifact=root/'provenance.json.gz'
            provenance=json.loads(gzip.decompress(artifact.read_bytes()))
            provenance['tests_stdout']='fabricated passing tests\n'
            artifact.write_bytes(gzip.compress(json.dumps(provenance).encode(),mtime=0))
            envelope=json.loads((root/'manifest.json').read_text())
            envelope['files']['provenance.json.gz']={'bytes':artifact.stat().st_size,'sha256':replay.sha(artifact.read_bytes())}
            (root/'manifest.json').write_text(json.dumps(envelope))
            with self.assertRaisesRegex(ValueError,'build/test log identity'): replay.replay(root)

if __name__=='__main__': unittest.main()
