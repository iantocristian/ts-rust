import copy
import gzip
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import run_lists as probe


def census(lengths=((0, 1, 17, 65), (), (3, 2))):
    return b"".join((json.dumps({"index": index, "node_backings": {"physical_lengths_in_aux_order": list(values)}}) + "\n").encode() for index, values in enumerate(lengths))


def child(expected, identity, sweeps=8):
    value = {"version": 1, "diagnostic_only": True, "mode": identity["mode"], "scope": probe.SCOPE,
        **{key: expected[key] for key in ("input_sha256", "files", "backings", "edges", "checksum")},
        "allocation": None, "timing": None, "storage": None}
    if identity["allocation"]:
        retained = expected["edges"] * (8 if identity["mode"] == "legacy" else 4) + 1024
        value["allocation"] = {"requested_bytes": retained + 1024, "retained_requested_bytes": retained, "freed_or_superseded_requests": 1024, "drop_returns_to_start": True}
    else:
        value["timing"] = {"construction_ns": 1000 + identity["index"], "traversal_ns": 800 + identity["index"], "sweeps": sweeps}
    if identity["mode"] != "legacy":
        pages = expected["page_counts"][identity["mode"]]
        width = int(identity["mode"].removeprefix("page"))
        value["storage"] = {"edge_pages": pages, "spare_edge_words": pages * width - expected["edges"], "retained_scratch_words": expected["scratch_minimum"], "wide_backings": expected["wide_backings"]}
    return value


def write_observations(directory, expected):
    (directory / "raw").mkdir()
    report = {"schedule": probe.schedule(), "sweeps": 8, "observations": [], "build_directory": "/frozen/list-build", "capture_directory": str(directory),
        "artifacts": {role: {"path": "artifacts/list-pilot-" + role} for role in ("normal", "allocation")}}
    for index, identity in enumerate(probe.schedule()):
        value = child(expected, identity)
        stdout = (json.dumps(value) + "\n").encode()
        stderr = b"diagnostic stderr retained\n" if index == 0 else b""
        for stream, data in (("stdout", stdout), ("stderr", stderr)):
            probe.compressed(directory / probe.raw_name(index, stream), data)
        role = "allocation" if identity["allocation"] else "normal"
        report["observations"].append({"identity": identity, "returncode": 0,
            "command": ["/frozen/list-build/artifacts/list-pilot-" + role, str(directory / "input.ndjson"), identity["mode"], "8"],
            "stdout_sha256": probe.runner.sha(stdout), "stderr_sha256": probe.runner.sha(stderr), "child": value})
    report["summary"] = probe.summarize(report["observations"])
    return report


class InputTests(unittest.TestCase):
    def test_independent_affine_checksum_matches_literal_reference(self):
        lengths = [0, 1, 16, 17, 18, 34, 257, 1001]
        checksum = 0
        for length in lengths:
            for index in range(length):
                checksum = (checksum * 31 + index % 17) & probe.MASK
        expected = probe.expected_from_raw(census((lengths,)))
        self.assertEqual(expected["checksum"], checksum)
        self.assertEqual(expected["backings"], len(lengths))
        self.assertEqual(expected["edges"], sum(lengths))

    def test_input_requires_exact_types_order_domain_and_complete_lines(self):
        for raw in (b"", census().rstrip(), b"\n", b'{"index":true,"node_backings":{"physical_lengths_in_aux_order":[]}}\n',
                    census(((True,),)), census(((-1,),)), census(((2**32,),)), census().replace(b'"index": 0', b'"index": 1'),
                    b'{"index":0,"index":0,"node_backings":{"physical_lengths_in_aux_order":[]}}\n'):
            with self.subTest(raw=raw[:80]), self.assertRaises((ValueError, TypeError)):
                probe.expected_from_raw(raw)

    def test_same_counts_different_backing_order_changes_checksum(self):
        first = probe.expected_from_raw(census(((1, 17),)))
        second = probe.expected_from_raw(census(((17, 1),)))
        self.assertEqual((first["files"], first["backings"], first["edges"]), (second["files"], second["backings"], second["edges"]))
        self.assertNotEqual(first["checksum"], second["checksum"])
        self.assertNotEqual(first["input_sha256"], second["input_sha256"])

    def test_histogram_cannot_disagree_with_order(self):
        raw = b'{"index":0,"node_backings":{"physical_lengths_in_aux_order":[1],"physical_length_histogram":{"2":1}}}\n'
        with self.assertRaisesRegex(ValueError, "histogram"):
            probe.expected_from_raw(raw)

    def test_exact_preflight_rejects_positive_but_wrong_accounting(self):
        good = {"version": 1, "preflight": True, "expected_bytes": 1_200_050, "requested_bytes": 1_200_050, "live_before": 42, "live_after": 42}
        self.assertEqual(probe.preflight(good), good)
        for key, value in (("requested_bytes", 1_200_049), ("expected_bytes", 1), ("live_after", 43), ("preflight", 1), ("version", True)):
            with self.subTest(key=key), self.assertRaises(ValueError):
                probe.preflight({**good, key: value})


class ProtocolTests(unittest.TestCase):
    def setUp(self):
        self.expected = probe.expected_from_raw(census())

    def test_schedule_has_one_warmup_seven_samples_and_reversed_orders(self):
        schedule = probe.schedule()
        self.assertEqual(len(schedule), 64)
        for allocation in (False, True):
            for mode in probe.POLICIES:
                selected = [row for row in schedule if row["allocation"] == allocation and row["mode"] == mode]
                self.assertEqual(sum(row["warmup"] for row in selected), 1)
                self.assertEqual([row["index"] for row in selected if not row["warmup"]], list(range(7)))
        measured = [row for row in schedule if not row["warmup"] and not row["allocation"]]
        self.assertEqual([row["mode"] for row in measured[:4]], list(probe.POLICIES))
        self.assertEqual([row["mode"] for row in measured[4:8]], list(reversed(probe.POLICIES)))

    def test_normal_and_allocation_metrics_cannot_cross(self):
        normal = {"warmup": False, "allocation": False, "index": 0, "mode": "page64"}
        allocation = {**normal, "allocation": True}
        for identity in (normal, allocation):
            self.assertEqual(probe.validate_child(child(self.expected, identity), self.expected, identity, 8)["mode"], "page64")
        with self.assertRaises(ValueError):
            probe.validate_child(child(self.expected, normal), self.expected, allocation, 8)
        with self.assertRaises(ValueError):
            probe.validate_child(child(self.expected, allocation), self.expected, normal, 8)

    def test_immutable_bundle_rejects_changed_binary_or_manifest_identity(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary).resolve() / "bundle"
            (directory / "artifacts").mkdir(parents=True)
            records = {}
            for role in ("normal", "allocation"):
                path = directory / "artifacts" / role
                path.write_bytes(role.encode())
                records[role] = {"path": str(path.relative_to(directory)), "sha256": probe.runner.digest(path), "features": ["allocation"] if role == "allocation" else []}
            allocation = {"version": 1, "preflight": True, "expected_bytes": 1_200_050, "requested_bytes": 1_200_050, "live_before": 0, "live_after": 0}
            probe.compressed(directory / "allocation-preflight.stdout.gz", json.dumps(allocation).encode())
            manifest = {"version": 1, "kind": probe.BUILD_KIND, "diagnostic_only": True, "artifacts": records, "allocation_preflight": allocation}
            digest = probe.runner.seal(directory, manifest)
            try:
                probe.validate_build(directory, digest)
                with self.assertRaisesRegex(ValueError, "manifest"):
                    probe.validate_build(directory, "0" * 64)
                path = directory / records["normal"]["path"]
                path.chmod(0o755)
                path.write_bytes(b"allocation")
                path.chmod(0o555)
                with self.assertRaisesRegex(ValueError, "inventory"):
                    probe.validate_build(directory, digest)
            finally:
                directory.chmod(0o755)
                for path in directory.rglob("*"):
                    path.chmod(0o755 if path.is_dir() else 0o644)

    def test_work_hash_checksum_types_page_counts_and_sweeps_are_checked(self):
        identity = {"warmup": False, "allocation": False, "index": 0, "mode": "page64"}
        good = child(self.expected, identity)
        changes = [("files", True), ("edges", good["edges"] + 1), ("checksum", good["checksum"] ^ 1),
                   ("input_sha256", "0" * 64), ("scope", "whole parser benchmark"), ("extra", 1)]
        for key, value in changes:
            with self.subTest(key=key), self.assertRaises(ValueError):
                probe.validate_child({**good, key: value}, self.expected, identity, 8)
        for group, key, value in (("storage", "edge_pages", 0), ("storage", "spare_edge_words", True),
                                  ("storage", "wide_backings", 1), ("storage", "retained_scratch_words", 0),
                                  ("timing", "sweeps", 7), ("timing", "construction_ns", float("nan"))):
            changed = copy.deepcopy(good)
            changed[group][key] = value
            with self.subTest(group=group, key=key), self.assertRaises(ValueError):
                probe.validate_child(changed, self.expected, identity, 8)

    def test_allocation_requires_live_bound_exact_subtraction_and_drop(self):
        identity = {"warmup": False, "allocation": True, "index": 0, "mode": "legacy"}
        good = child(self.expected, identity)
        for key, value in (("drop_returns_to_start", 1), ("drop_returns_to_start", False),
                           ("requested_bytes", 0), ("retained_requested_bytes", 1), ("freed_or_superseded_requests", 0)):
            changed = copy.deepcopy(good)
            changed["allocation"][key] = value
            with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                probe.validate_child(changed, self.expected, identity, 8)

    def test_replay_uses_all_raw_samples_and_rejects_each_schedule_mutation(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            report = write_observations(directory, self.expected)
            probe.validate_observations(report, directory, self.expected)
            mutations = []
            missing = copy.deepcopy(report); missing["observations"].pop(); mutations.append(missing)
            extra = copy.deepcopy(report); extra["observations"].append(extra["observations"][-1]); mutations.append(extra)
            swapped = copy.deepcopy(report); swapped["observations"][0:2] = reversed(swapped["observations"][0:2]); mutations.append(swapped)
            failed = copy.deepcopy(report); failed["observations"][0]["returncode"] = 1; mutations.append(failed)
            boolean = copy.deepcopy(report); boolean["schedule"][0]["index"] = False; mutations.append(boolean)
            command = copy.deepcopy(report); command["observations"][0]["command"][-1] = "7"; mutations.append(command)
            summary = copy.deepcopy(report); summary["summary"]["legacy"]["construction_ns"]["median"] += 1; mutations.append(summary)
            for changed in mutations:
                with self.assertRaises(ValueError):
                    probe.validate_observations(changed, directory, self.expected)

    def test_raw_stdout_stderr_and_inventory_are_not_optional(self):
        for stream in ("stdout", "stderr", "extra"):
            with tempfile.TemporaryDirectory() as temporary:
                directory = Path(temporary)
                report = write_observations(directory, self.expected)
                if stream == "extra":
                    probe.compressed(directory / "raw/extra.stdout.gz", b"extra")
                else:
                    probe.compressed(directory / probe.raw_name(0, stream), b"changed")
                with self.subTest(stream=stream), self.assertRaises(ValueError):
                    probe.validate_observations(report, directory, self.expected)

    def test_failed_capture_keeps_partial_raw_outputs(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            build, census_dir, output = root / "build", root / "census", root / "output"
            (build / "artifacts").mkdir(parents=True)
            census_dir.mkdir()
            (census_dir / "manifest.json").write_text("{}")
            binary = build / "artifacts/normal"
            binary.write_bytes(b"frozen")
            allocation = {"version": 1, "preflight": True, "expected_bytes": 1_200_050, "requested_bytes": 1_200_050, "live_before": 0, "live_after": 0}
            bundle = {"tool_sha256": {}, "artifacts": {role: {"path": "artifacts/normal", "sha256": probe.runner.digest(binary)} for role in ("normal", "allocation")}, "allocation_preflight": allocation}
            completed = type("Completed", (), {"stdout": b"partial stdout", "stderr": b"failure reason", "returncode": 1})()
            with patch.object(probe, "validate_build", return_value=bundle), patch.object(probe, "validate_census", return_value=({"build_manifest_sha256": "c" * 64}, census(), self.expected)), \
                 patch.object(probe, "tool_files", return_value={}), patch.object(probe, "native_environment", return_value={}), patch.object(probe, "configuration", return_value={}), \
                 patch.object(probe, "host_info", return_value={}), patch.object(probe, "quiet_host"), patch.object(probe.subprocess, "run", return_value=completed):
                with self.assertRaisesRegex(ValueError, "child 0"):
                    probe.capture(build, "b" * 64, census_dir, "c" * 64, output, 8)
            self.assertEqual(gzip.decompress((output / probe.raw_name(0, "stdout")).read_bytes()), b"partial stdout")
            self.assertEqual(gzip.decompress((output / probe.raw_name(0, "stderr")).read_bytes()), b"failure reason")
            self.assertEqual(json.loads((output / "report.json").read_text())["status"], "failed")


if __name__ == "__main__":
    unittest.main()
