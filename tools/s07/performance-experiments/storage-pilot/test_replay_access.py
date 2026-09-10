import copy
import json
from pathlib import Path
import tempfile
import unittest

import replay_access as replay
import replay_chunks as chunks
import replay_lists as original
import run_chunks
import run_lists


class AccessArchiveTests(unittest.TestCase):
    def test_archive_adapter_preserves_prior_experiments_and_member_checks(self):
        self.assertIs(replay.validate_members, original.validate_members)
        self.assertIs(replay.read_members, original.read_members)
        self.assertEqual(replay.probe.POLICIES, ("legacy", "chunk256", "chunk256-owned", "chunk256-borrowed"))
        self.assertEqual(run_lists.POLICIES, ("legacy", "page64", "page256", "page1024"))
        self.assertEqual(run_chunks.backend.POLICIES, ("legacy", "page256", "chunk256", "chunk1024"))
        self.assertEqual(len({replay.BUILD_SHA, chunks.BUILD_SHA, original.BUILD_SHA}), 3)
        self.assertEqual(len({replay.CAPTURE_SHA, chunks.CAPTURE_SHA, original.CAPTURE_SHA}), 3)
        self.assertEqual(set(replay.replay_inputs()), {"replay_access.py", "test_replay_access.py", "replay_lists.py"})

    def test_neither_prior_experiment_can_be_relabeled_as_access_trial(self):
        for relative in ("results/2026-09-08", "results/2026-09-08-chunks"):
            prior = json.loads((original.HERE / relative / "manifest.json").read_bytes())
            with tempfile.TemporaryDirectory() as temporary:
                directory = Path(temporary)
                (directory / "manifest.json").write_text(json.dumps(prior))
                with self.subTest(relative=relative), self.assertRaisesRegex(ValueError, "invalid access archive identity"):
                    replay.replay(directory)
                prior["kind"] = "s07_bis_list_access_archive"
                (directory / "manifest.json").write_text(json.dumps(prior))
                with self.subTest(relative=relative), self.assertRaisesRegex(ValueError, "changed the recorded experiment"):
                    replay.replay(directory)

    def test_replayer_requires_its_exact_active_source_closure(self):
        manifest = json.loads((original.HERE / "results/2026-09-08/manifest.json").read_bytes())
        manifest.update(kind="s07_bis_list_access_archive", build_manifest_sha256=replay.BUILD_SHA,
                        capture_manifest_sha256=replay.CAPTURE_SHA, replay_inputs=replay.replay_inputs())
        for source in replay.replay_inputs():
            changed = copy.deepcopy(manifest)
            del changed["replay_inputs"][source]
            with tempfile.TemporaryDirectory() as temporary:
                directory = Path(temporary)
                (directory / "manifest.json").write_text(json.dumps(changed))
                with self.subTest(source=source), self.assertRaisesRegex(ValueError, "replay implementation changed"):
                    replay.replay(directory)


if __name__ == "__main__":
    unittest.main()
