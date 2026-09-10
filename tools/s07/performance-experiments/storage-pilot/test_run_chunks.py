import copy
import json
from pathlib import Path
import tempfile
import unittest

import run_chunks as chunk
import run_lists as baseline
from test_run_lists import child as control_child


def raw_input(files):
    return b"".join((json.dumps({"index": index, "node_backings": {"physical_lengths_in_aux_order": lengths}}) + "\n").encode() for index, lengths in enumerate(files))


class ChunkRunnerTests(unittest.TestCase):
    def test_exact_fit_abandoned_tail_oversized_and_empty_layout(self):
        self.assertEqual(chunk.chunk_layout([], 4), {"edge_chunks": 0, "spare_edge_words": 0})
        self.assertEqual(chunk.chunk_layout([0, 0], 4), {"edge_chunks": 0, "spare_edge_words": 0})
        self.assertEqual(chunk.chunk_layout([1, 3, 4], 4), {"edge_chunks": 2, "spare_edge_words": 0})
        # The last 1 cannot reuse the abandoned first chunk's spare word.
        self.assertEqual(chunk.chunk_layout([3, 4, 1], 4), {"edge_chunks": 3, "spare_edge_words": 4})
        self.assertEqual(chunk.chunk_layout([1, 7, 0, 2], 4), {"edge_chunks": 3, "spare_edge_words": 5})

    def test_full_domain_and_invalid_types_do_not_wrap(self):
        self.assertEqual(chunk.chunk_layout([2**32 - 1, 1], 256), {"edge_chunks": 2, "spare_edge_words": 255})
        for lengths, width in (([True], 4), ([-1], 4), ([2**32], 4), ([1], 0), ([1], True)):
            with self.subTest(lengths=lengths, width=width), self.assertRaises(ValueError):
                chunk.chunk_layout(lengths, width)

    def test_each_file_has_a_separate_tail_and_checksum_is_unchanged(self):
        raw = raw_input([[1], [1]])
        expected = chunk.expected_from_raw(raw)
        self.assertEqual(expected["chunk_storage"]["chunk256"], {"edge_chunks": 2, "spare_edge_words": 510})
        ordinary = baseline.expected_from_raw(raw)
        for key in ("input_sha256", "checksum", "files", "backings", "edges", "scratch_minimum"):
            self.assertEqual(expected[key], ordinary[key])

    def test_adapter_does_not_modify_the_first_runner(self):
        self.assertEqual(baseline.POLICIES, ("legacy", "page64", "page256", "page1024"))
        self.assertEqual(baseline.BUILD_KIND, "s07_bis_list_distribution_build")
        self.assertEqual(chunk.backend.POLICIES, ("legacy", "page256", "chunk256", "chunk1024"))
        self.assertNotEqual(chunk.backend.BUILD_KIND, baseline.BUILD_KIND)
        self.assertNotEqual(chunk.backend.CAPTURE_KIND, baseline.CAPTURE_KIND)
        schedule = chunk.backend.schedule()
        self.assertEqual(len(schedule), 64)
        self.assertEqual(sum(row["warmup"] for row in schedule), 8)
        for mode in chunk.POLICIES:
            for allocation in (False, True):
                selected = [row for row in schedule if row["mode"] == mode and row["allocation"] == allocation and not row["warmup"]]
                self.assertEqual([row["index"] for row in selected], list(range(7)))

    def test_chunk_protocol_does_not_accept_page_shaped_or_approximate_counts(self):
        expected = chunk.expected_from_raw(raw_input([[3, 258, 0, 2], [1]]))
        identity = {"mode": "chunk256", "allocation": False, "warmup": False, "index": 0}
        value = {"version": 1, "diagnostic_only": True, "mode": "chunk256", "scope": baseline.SCOPE,
            **{key: expected[key] for key in ("input_sha256", "files", "backings", "edges", "checksum")},
            "allocation": None, "timing": {"construction_ns": 1, "traversal_ns": 1, "sweeps": 8},
            "storage": {**expected["chunk_storage"]["chunk256"], "retained_scratch_words": expected["scratch_minimum"]}}
        self.assertEqual(chunk.validate_child(value, expected, identity, 8), value)
        for section, key, replacement in (("storage", "edge_chunks", value["storage"]["edge_chunks"] - 1),
                                           ("storage", "spare_edge_words", True), ("storage", "retained_scratch_words", 0), ("timing", "sweeps", 7)):
            changed = copy.deepcopy(value)
            changed[section][key] = replacement
            with self.subTest(key=key), self.assertRaises(ValueError):
                chunk.validate_child(changed, expected, identity, 8)
        changed = copy.deepcopy(value)
        changed["storage"]["edge_pages"] = changed["storage"].pop("edge_chunks")
        with self.assertRaises(ValueError):
            chunk.validate_child(changed, expected, identity, 8)
        allocation = copy.deepcopy(value)
        allocation["timing"] = None
        allocation["allocation"] = {"requested_bytes": 5000, "retained_requested_bytes": 4000, "freed_or_superseded_requests": 1000, "drop_returns_to_start": True}
        self.assertEqual(chunk.validate_child(allocation, expected, {**identity, "allocation": True}, 8), allocation)
        with self.assertRaises(ValueError):
            chunk.validate_child(allocation, expected, identity, 8)
        allocation["allocation"]["drop_returns_to_start"] = False
        with self.assertRaises(ValueError):
            chunk.validate_child(allocation, expected, {**identity, "allocation": True}, 8)

    def test_isolated_backend_replays_all_new_policies_and_rejects_old_schedule(self):
        expected = chunk.expected_from_raw(raw_input([[3, 258, 0, 2], [1]]))
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            (directory / "raw").mkdir()
            report = {"schedule": chunk.backend.schedule(), "sweeps": 8, "observations": [],
                "build_directory": "/frozen/chunk-build", "capture_directory": str(directory),
                "artifacts": {role: {"path": "artifacts/list-pilot-" + role} for role in ("normal", "allocation")}}
            for index, identity in enumerate(chunk.backend.schedule()):
                mode = identity["mode"]
                if mode.startswith("chunk"):
                    value = control_child(expected, {**identity, "mode": "page256"})
                    value["mode"] = mode
                    value["storage"] = {**expected["chunk_storage"][mode], "retained_scratch_words": expected["scratch_minimum"]}
                else:
                    value = control_child(expected, identity)
                stdout = (json.dumps(value) + "\n").encode()
                for stream, raw in (("stdout", stdout), ("stderr", b"")):
                    baseline.compressed(directory / baseline.raw_name(index, stream), raw)
                role = "allocation" if identity["allocation"] else "normal"
                report["observations"].append({"identity": identity, "returncode": 0,
                    "command": ["/frozen/chunk-build/artifacts/list-pilot-" + role, str(directory / "input.ndjson"), mode, "8"],
                    "stdout_sha256": baseline.runner.sha(stdout), "stderr_sha256": baseline.runner.sha(b""), "child": value})
            report["summary"] = chunk.backend.summarize(report["observations"])
            chunk.backend.validate_observations(report, directory, expected)
            self.assertEqual(set(report["summary"]), set(chunk.POLICIES))
            report["schedule"] = baseline.schedule()
            with self.assertRaisesRegex(ValueError, "schedule"):
                chunk.backend.validate_observations(report, directory, expected)


if __name__ == "__main__":
    unittest.main()
