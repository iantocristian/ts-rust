import copy
import json
from pathlib import Path
import tempfile
import unittest

import run_access as access
import run_chunks as chunks
import run_lists as baseline
from test_run_chunks import raw_input


def child(expected, identity):
    mode = identity["mode"]
    cache = expected["borrowed_cache"] if mode == "chunk256-borrowed" else {"borrowed_view_count": 0, "borrowed_view_capacity_bytes": 0}
    storage = {**expected["chunk_storage"]["chunk256"], "retained_scratch_words": expected["scratch_minimum"]}
    if mode in ("chunk256-owned", "chunk256-borrowed"):
        storage.update(cache)
    elif mode == "legacy":
        storage = None
    retained = expected["edges"] * 8 + 10_000 + cache["borrowed_view_capacity_bytes"]
    return {"version": 1, "diagnostic_only": True, "mode": mode, "scope": baseline.SCOPE,
        **{key: expected[key] for key in ("input_sha256", "files", "backings", "edges", "checksum")},
        "storage": storage,
        "allocation": {"requested_bytes": retained + 1000, "retained_requested_bytes": retained,
            "freed_or_superseded_requests": 1000, "drop_returns_to_start": True} if identity["allocation"] else None,
        "timing": None if identity["allocation"] else {"construction_ns": 100, "traversal_ns": 200, "sweeps": 8}}


def report_in(directory, expected, mutate=None):
    report = {"schedule": access.backend.schedule(), "sweeps": 8, "observations": [],
        "build_directory": "/frozen/access-build", "capture_directory": str(directory),
        "artifacts": {role: {"path": "artifacts/list-pilot-" + role} for role in ("normal", "allocation")}}
    for index, identity in enumerate(report["schedule"]):
        value = child(expected, identity)
        if mutate:
            mutate(value, identity)
        stdout = (json.dumps(value) + "\n").encode()
        for stream, raw in (("stdout", stdout), ("stderr", b"")):
            baseline.compressed(directory / baseline.raw_name(index, stream), raw)
        role = "allocation" if identity["allocation"] else "normal"
        report["observations"].append({"identity": identity, "returncode": 0,
            "command": ["/frozen/access-build/artifacts/list-pilot-" + role, str(directory / "input.ndjson"), identity["mode"], "8"],
            "stdout_sha256": baseline.runner.sha(stdout), "stderr_sha256": baseline.runner.sha(b""), "child": value})
    report["summary"] = access.backend.summarize(report["observations"])
    return report


class AccessRunnerTests(unittest.TestCase):
    def setUp(self):
        self.raw = raw_input([[3, 258, 0, 2], [1], []])
        self.expected = access.expected_from_raw(self.raw)

    def test_cache_charges_all_backings_and_per_file_vectors_including_empty(self):
        self.assertEqual(self.expected["borrowed_cache"], {"borrowed_view_count": 5, "borrowed_view_capacity_bytes": 5 * 16 + 3 * 24})
        reference = chunks.expected_from_raw(self.raw)
        self.assertEqual(self.expected["chunk_storage"]["chunk256"], reference["chunk_storage"]["chunk256"])
        for key in ("files", "backings", "edges", "checksum", "input_sha256", "scratch_minimum"):
            self.assertEqual(self.expected[key], reference[key])

    def test_isolated_policy_schedule_does_not_modify_either_prior_runner(self):
        self.assertEqual(baseline.POLICIES, ("legacy", "page64", "page256", "page1024"))
        self.assertEqual(chunks.backend.POLICIES, ("legacy", "page256", "chunk256", "chunk1024"))
        self.assertEqual(access.backend.POLICIES, ("legacy", "chunk256", "chunk256-owned", "chunk256-borrowed"))
        self.assertEqual(len(access.backend.schedule()), 64)
        self.assertEqual(sum(row["warmup"] for row in access.backend.schedule()), 8)
        for mode in access.POLICIES:
            for allocation in (False, True):
                self.assertEqual([row["index"] for row in access.backend.schedule() if row["mode"] == mode and row["allocation"] == allocation and not row["warmup"]], list(range(7)))
        self.assertEqual(len({baseline.BUILD_KIND, chunks.backend.BUILD_KIND, access.backend.BUILD_KIND}), 3)

    def test_each_route_requires_its_exact_protocol_and_cache_shape(self):
        for mode in access.POLICIES:
            for allocation in (False, True):
                identity = {"mode": mode, "allocation": allocation, "warmup": False, "index": 0}
                value = child(self.expected, identity)
                self.assertEqual(access.validate_child(value, self.expected, identity, 8), value)
                wrong = copy.deepcopy(value)
                wrong["mode"] = "chunk256-owned" if mode != "chunk256-owned" else "chunk256-borrowed"
                with self.assertRaises(ValueError):
                    access.validate_child(wrong, self.expected, identity, 8)
        identity = {"mode": "chunk256-borrowed", "allocation": False, "warmup": False, "index": 0}
        valid = child(self.expected, identity)
        for key, replacement in (("borrowed_view_count", 4), ("borrowed_view_capacity_bytes", 5 * 16),
                ("borrowed_view_count", True), ("edge_chunks", 0), ("retained_scratch_words", 0)):
            wrong = copy.deepcopy(valid)
            wrong["storage"][key] = replacement
            with self.subTest(key=key), self.assertRaises(ValueError):
                access.validate_child(wrong, self.expected, identity, 8)
        wrong = copy.deepcopy(valid)
        del wrong["storage"]["borrowed_view_capacity_bytes"]
        with self.assertRaises(ValueError):
            access.validate_child(wrong, self.expected, identity, 8)

    def test_complete_64_child_replay_rejects_wrong_schedule_and_metric_mode(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            (directory / "raw").mkdir()
            report = report_in(directory, self.expected)
            access.validate_observations(report, directory, self.expected)
            report["schedule"] = chunks.backend.schedule()
            with self.assertRaisesRegex(ValueError, "schedule"):
                access.validate_observations(report, directory, self.expected)
        identity = {"mode": "chunk256-owned", "allocation": False, "warmup": False, "index": 0}
        with self.assertRaises(ValueError):
            access.validate_child(child(self.expected, {**identity, "allocation": True}), self.expected, identity, 8)

    def test_sweep_count_is_fixed_before_capture_or_replay(self):
        identity = {"mode": "legacy", "allocation": False, "warmup": False, "index": 0}
        for count in (1, 7, 9, True, 8.0):
            with self.subTest(count=count), self.assertRaisesRegex(ValueError, "exactly eight sweeps"):
                access.capture(None, None, None, None, None, count)
            with self.subTest(count=count), self.assertRaisesRegex(ValueError, "exactly eight sweeps"):
                access.validate_child(child(self.expected, identity), self.expected, identity, count)

    def test_replay_rejects_uncharged_cache_even_when_edge_lower_bound_passes(self):
        def uncharge(value, identity):
            if identity["allocation"] and identity["mode"] == "chunk256-borrowed":
                for key in ("requested_bytes", "retained_requested_bytes"):
                    value["allocation"][key] -= self.expected["borrowed_cache"]["borrowed_view_capacity_bytes"]
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            (directory / "raw").mkdir()
            report = report_in(directory, self.expected, uncharge)
            with self.assertRaisesRegex(ValueError, "cache allocation is not charged"):
                access.validate_observations(report, directory, self.expected)

    def test_replay_rejects_owner_route_allocations_and_changed_core_scratch(self):
        for allocation_delta in (True, False):
            def change(value, identity):
                if identity["mode"] != "chunk256-owned":
                    return
                if allocation_delta and identity["allocation"]:
                    for key in ("requested_bytes", "retained_requested_bytes"):
                        value["allocation"][key] += 16
                if not allocation_delta:
                    value["storage"]["retained_scratch_words"] += 1
            with tempfile.TemporaryDirectory() as temporary:
                directory = Path(temporary)
                (directory / "raw").mkdir()
                report = report_in(directory, self.expected, change)
                expected_error = "construction allocations" if allocation_delta else "shared physical storage"
                with self.subTest(allocation_delta=allocation_delta), self.assertRaisesRegex(ValueError, expected_error):
                    access.validate_observations(report, directory, self.expected)


if __name__ == "__main__":
    unittest.main()
