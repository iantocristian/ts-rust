import json
from pathlib import Path
import tempfile
import unittest

import replay_chunks as replay
import replay_lists as original
import run_lists


class ChunkArchiveTests(unittest.TestCase):
    def test_archive_adapter_preserves_original_experiment_and_checks(self):
        self.assertIs(replay.validate_members, original.validate_members)
        self.assertIs(replay.read_members, original.read_members)
        self.assertEqual(replay.probe.POLICIES, ("legacy", "page256", "chunk256", "chunk1024"))
        self.assertEqual(run_lists.POLICIES, ("legacy", "page64", "page256", "page1024"))
        self.assertNotEqual(replay.BUILD_SHA, original.BUILD_SHA)
        self.assertNotEqual(replay.CAPTURE_SHA, original.CAPTURE_SHA)
        self.assertEqual(set(replay.replay_inputs()), {"replay_chunks.py", "test_replay_chunks.py", "replay_lists.py"})

    def test_original_experiment_cannot_be_relabeled_as_chunk_trial(self):
        original_manifest = json.loads((original.HERE / "results/2026-09-08/manifest.json").read_bytes())
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            (directory / "manifest.json").write_text(json.dumps(original_manifest))
            with self.assertRaisesRegex(ValueError, "invalid chunk archive identity"):
                replay.replay(directory)
            original_manifest["kind"] = "s07_bis_chunk_distribution_archive"
            (directory / "manifest.json").write_text(json.dumps(original_manifest))
            with self.assertRaisesRegex(ValueError, "changed the recorded experiment"):
                replay.replay(directory)


if __name__ == "__main__":
    unittest.main()
