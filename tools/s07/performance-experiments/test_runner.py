"""Counterexamples for immutable artifacts, fixed observations and honest screens."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location("bis_runner", Path(__file__).with_name("runner.py"))
runner = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(runner)

EXPECTED = {"files": 13094, "loaded_bytes": 161740237, "nodes": 19593488,
            "symbols": 2459867, "parse_diagnostics": 423, "bind_diagnostics": 5250,
            "loaded_input_sha256": "d4ff2ad3b0d590babb596522a7105d961dbb3c4f2ebc987488d88b7ebd8e0c88"}
PREFLIGHT = [{"mode": mode, "toolchain": "1.96.0", "version": 1, "workers": 8,
              "requested_bytes": 9600400, "expected_bytes": 9600400, "live_before": 1548,
              "live_after": 1548} for mode in ("debug", "release")]


def observation(identity, ratio=0.9):
    factor = ratio if identity["variant"] == "candidate" else 1.0
    report = {**EXPECTED, "version": 1, "workers": identity["workers"],
              "wall_time_ns": int(1_000_000 * factor), "allocated_bytes": 1_000_000 if identity["allocation"] else None,
              "startup_ns": 10, "preload_ns": 10, "worker_setup_ns": 10,
              "cpu_capacity": 18, "goroutines_ready": None}
    return {"report": report, "peak_rss_bytes": 1_000_000, "process_time_ns": 2_000_000,
            "user_time_ns": 1_000_000, "system_time_ns": 100_000, "stderr": ""}


def rows(ratio=0.9, warmup=False):
    return [{**identity, "sample": observation(identity, ratio)} for identity in runner.row_order(warmup)]


class RunnerTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)

    def tearDown(self):
        for path in (self.root, *self.root.rglob("*")):
            path.chmod(0o755 if path.is_dir() else 0o644)
        self.temporary.cleanup()

    def bundle(self, name="control", kind="control", control_sha=None):
        directory = self.root / name
        (directory / "artifacts").mkdir(parents=True)
        (directory / "source").mkdir()
        (directory / "source/input.rs").write_text("original source\n")
        files = {"input.rs": runner.digest(directory / "source/input.rs")}
        fingerprint = {"files": files, "sha256": runner.sha(json.dumps(files, sort_keys=True, separators=(",", ":")).encode())}
        artifacts = {}
        for role in ("normal", "allocation", "go"):
            path = directory / "artifacts" / role
            path.write_text(role + " executable")
            artifacts[role] = {"path": "artifacts/" + role, "sha256": runner.digest(path)}
        (directory / "inputs.json").write_text("[]\n")
        manifest = {"version": 1, "kind": kind, "label": name, "diagnostic_only": True,
                    "artifacts": artifacts, "source_fingerprint": fingerprint,
                    "cargo_configuration": {}, "expected_work": EXPECTED,
                    "allocation_preflight": PREFLIGHT, "go_version": "go1.27.1",
                    "rust_profile": "release", "rustc": "rustc 1.97.1", "measurement_domains": {}}
        if control_sha:
            manifest.update(control_manifest_sha256=control_sha, target_metrics=["wall_time_ns"], hypothesis="test")
        digest = runner.seal(directory, manifest)
        return directory, digest, manifest

    def mutate_file(self, path, content):
        path.chmod(0o644)
        path.write_text(content)
        path.chmod(0o555 if path.parent.name == "artifacts" else 0o444)

    def test_frozen_bundle_verifies_and_rejects_binary_swap(self):
        directory, digest, _ = self.bundle()
        runner.validate_bundle(directory, digest)
        normal, allocation = directory / "artifacts/normal", directory / "artifacts/allocation"
        normal_bytes, allocation_bytes = normal.read_text(), allocation.read_text()
        self.mutate_file(normal, allocation_bytes)
        self.mutate_file(allocation, normal_bytes)
        with self.assertRaisesRegex(ValueError, "inventory changed"):
            runner.validate_bundle(directory, digest)

    def test_modified_source_rejected_even_if_artifacts_unchanged(self):
        directory, digest, _ = self.bundle()
        self.mutate_file(directory / "source/input.rs", "changed source\n")
        with self.assertRaisesRegex(ValueError, "inventory changed"):
            runner.validate_bundle(directory, digest)

    def test_rewritten_manifest_cannot_relabel_changed_binary(self):
        directory, digest, manifest = self.bundle()
        manifest["label"] = "a different build"
        self.mutate_file(directory / "manifest.json", json.dumps(manifest))
        with self.assertRaisesRegex(ValueError, "manifest changed"):
            runner.validate_bundle(directory, digest)

    def test_writable_bundle_rejected(self):
        directory, digest, _ = self.bundle()
        (directory / "artifacts/normal").chmod(0o755)
        with self.assertRaisesRegex(ValueError, "not immutable"):
            runner.validate_bundle(directory, digest)

    def test_sample_inventory_is_exactly_56_plus_8_warmups(self):
        self.assertEqual(len(rows()), 56)
        self.assertEqual(len(rows(warmup=True)), 8)
        runner.validate_rows(rows(), EXPECTED)
        runner.validate_rows(rows(warmup=True), EXPECTED, warmup=True)
        for changed in (rows()[:-1], rows() + rows()[:1]):
            with self.assertRaisesRegex(ValueError, "incomplete or extended"):
                runner.validate_rows(changed, EXPECTED)

    def test_duplicate_or_reordered_observations_rejected(self):
        changed = rows()
        changed[0], changed[1] = changed[1], changed[0]
        with self.assertRaisesRegex(ValueError, "sample identity"):
            runner.validate_rows(changed, EXPECTED)
        changed = rows()
        changed[1] = copy.deepcopy(changed[0])
        with self.assertRaisesRegex(ValueError, "sample identity"):
            runner.validate_rows(changed, EXPECTED)

    def test_missing_allocation_observation_cannot_pass(self):
        changed = rows()
        next(row for row in changed if row["allocation"])["sample"]["report"]["allocated_bytes"] = None
        with self.assertRaisesRegex(ValueError, "allocated_bytes"):
            runner.validate_rows(changed, EXPECTED)

    def test_instrumented_artifact_cannot_supply_normal_timing(self):
        changed = rows()
        changed[0]["sample"]["report"]["allocated_bytes"] = 100
        with self.assertRaisesRegex(ValueError, "allocation-instrumented"):
            runner.validate_rows(changed, EXPECTED)

    def test_same_counts_with_different_loaded_bytes_rejected(self):
        changed = rows()
        changed[0]["sample"]["report"]["loaded_input_sha256"] = "f" * 64
        with self.assertRaisesRegex(ValueError, "preload bytes/options differ"):
            runner.validate_rows(changed, EXPECTED)

    def test_timing_win_is_labeled_candidate_control_without_tracker_metrics(self):
        report = runner.summarize(rows(), EXPECTED, ["wall_time_ns"])
        self.assertEqual(report["screening_status"], "eligible_for_review")
        wall = report["modes"]["1"]["wall_time_ns"]
        self.assertEqual(wall["ratio"], 0.9)
        self.assertEqual(wall["samples_per_variant"], 7)
        self.assertNotIn("go_median", wall)
        self.assertNotIn("metrics", report)
        self.assertNotIn("bootstrap", report["modes"]["1"]["allocated_bytes"])

    def test_rss_regression_blocks_timing_win(self):
        changed = rows()
        for row in changed:
            if row["variant"] == "candidate" and row["workers"] == 8 and not row["allocation"]:
                row["sample"]["peak_rss_bytes"] = 1_030_000
        result = runner.summarize(changed, EXPECTED, ["wall_time_ns"])
        self.assertEqual(result["screening_status"], "regressing_or_uncertain")
        self.assertFalse(result["nonregression_conditions_met"])

    def test_noise_does_not_extend_batch_or_manufacture_promotion(self):
        changed = rows()
        for row in changed:
            if row["workers"] == 1 and not row["allocation"]:
                row["sample"]["report"]["wall_time_ns"] = 400_000 + row["index"] * 100_000
        result = runner.summarize(changed, EXPECTED, ["wall_time_ns"])
        self.assertEqual(result["screening_status"], "inconclusive")
        self.assertEqual(result["modes"]["1"]["wall_time_ns"]["samples_per_variant"], 7)

    def test_build_copies_each_shared_artifact_before_next_build(self):
        control, control_sha, baseline = self.bundle()
        output = self.root / "candidate"
        shared = self.root / "shared-binary"
        (self.root / "input.rs").write_text("original source\n")

        def build(allocation):
            shared.write_text("allocation build" if allocation else "normal build")
            return shared, {}

        with patch.object(runner, "ROOT", self.root), \
                patch.object(runner, "validate_inputs"), \
                patch.object(runner, "reject_concurrent_builds"), \
                patch.object(runner, "source_fingerprint", return_value=baseline["source_fingerprint"]), \
                patch.object(runner, "cargo_configuration", return_value={}), \
                patch.object(runner, "allocation_preflight", return_value=PREFLIGHT), \
                patch.object(runner, "build_rust", side_effect=build), \
                patch.object(runner, "runtime_libraries", side_effect=lambda binary, dest: dest.write_text("system libc")), \
                patch.object(runner, "rust_native_toolchain", return_value=("1.97.1", "aarch64-apple-darwin")), \
                patch.object(runner, "command", return_value=b"test revision\n"):
            runner.build_candidate(output, control, control_sha, "candidate", "test", ["wall_time_ns"])
        self.assertEqual((output / "artifacts/rust-benchmark").read_text(), "normal build")
        self.assertEqual((output / "artifacts/rust-benchmark-allocation").read_text(), "allocation build")

    def test_verify_recomputes_summary_and_rejects_partial_capture(self):
        control, control_sha, _ = self.bundle()
        candidate, candidate_sha, _ = self.bundle("candidate", "candidate", control_sha)
        output = self.root / "screen"
        output.mkdir()
        for name, values in (("samples", rows()), ("warmups", rows(warmup=True))):
            (output / (name + ".ndjson")).write_text("".join(json.dumps(row) + "\n" for row in values))
            folder = output / ("sample-raw" if name == "samples" else "warmup-raw")
            folder.mkdir()
            for row in values:
                stem = "{workers}-{allocation}-{index}-{variant}".format(**row)
                runner.write_json(folder / (stem + ".stdout"), row["sample"])
                (folder / (stem + ".stderr")).write_text("")
        summary = runner.summarize(rows(), EXPECTED, ["wall_time_ns"])
        report = {"status": "complete", "diagnostic_only": True, "samples": 56, **summary,
                  "bundles": {"control": str(control), "candidate": str(candidate)},
                  "manifest_sha256": {"control": control_sha, "candidate": candidate_sha},
                  "samples_sha256": runner.digest(output / "samples.ndjson"),
                  "warmups_sha256": runner.digest(output / "warmups.ndjson"),
                  "raw_capture_inventory": runner.raw_inventory(output)}
        runner.write_json(output / "report.json", report)
        runner.verify_report(output)
        graph = self.root / "optional-graph.json"
        graph.write_text("original graph evidence")
        report["graph_report"] = {"path": str(graph), "sha256": runner.digest(graph)}
        runner.write_json(output / "report.json", report)
        with patch.object(runner, "validate_graph_report") as validate:
            runner.verify_report(output)
            validate.assert_called_once()
        graph.write_text("changed graph evidence")
        with self.assertRaisesRegex(ValueError, "graph prerequisite changed"):
            runner.verify_report(output)
        graph.unlink()
        with self.assertRaises(FileNotFoundError):
            runner.verify_report(output)
        del report["graph_report"]
        report["modes"]["1"]["wall_time_ns"]["ratio"] = 0.1
        runner.write_json(output / "report.json", report)
        with self.assertRaisesRegex(ValueError, "summaries differ"):
            runner.verify_report(output)
        report["status"] = "failed"
        runner.write_json(output / "report.json", report)
        with self.assertRaisesRegex(ValueError, "incomplete"):
            runner.verify_report(output)

    def test_binding_paths_count_every_file_and_reject_wrong_loaded_identity(self):
        observed = {"version": 1, "workers": 1, "files": 13094, "bound_in_place_files": 13094,
                    "fallback_files": 0, "loaded_input_sha256": EXPECTED["loaded_input_sha256"]}
        runner.validate_paths(observed, EXPECTED, 1)
        for change in ({"fallback_files": 1}, {"loaded_input_sha256": "0" * 64}, {"workers": 8}):
            with self.assertRaisesRegex(ValueError, "changed work or lost a file"):
                runner.validate_paths({**observed, **change}, EXPECTED, 1)

    def test_failed_screen_keeps_failed_status_and_cannot_overwrite_directory(self):
        control, control_sha, baseline = self.bundle()
        candidate, candidate_sha, _ = self.bundle("candidate", "candidate", control_sha)
        output = self.root / "failed-screen"
        cache = self.root / "cache"
        (cache / "s07-benchmark").mkdir(parents=True)
        with patch.object(runner, "CACHE", cache), \
                patch.object(runner, "validate_inputs"), \
                patch.object(runner, "reject_concurrent_builds"), \
                patch.object(runner, "source_fingerprint", return_value=baseline["source_fingerprint"]), \
                patch.object(runner, "cargo_configuration", return_value={}), \
                patch.object(runner, "tool_fingerprint", return_value={}), \
                patch.object(runner, "host_info", return_value={}), \
                patch.object(runner, "capture_sample", side_effect=ValueError("child failed; retained stderr")):
            with self.assertRaisesRegex(ValueError, "child failed"):
                runner.screen(control, candidate, control_sha, candidate_sha, output)
            failed = json.loads((output / "report.json").read_text())
            self.assertEqual(failed["status"], "failed")
            self.assertIn("retained stderr", failed["error"])
            self.assertNotIn("modes", failed)
            with self.assertRaises(FileExistsError):
                runner.screen(control, candidate, control_sha, candidate_sha, output)


    def test_graph_prerequisite_requires_exact_artifacts_and_both_complete_modes(self):
        _, control_sha, control = self.bundle()
        _, candidate_sha, candidate = self.bundle("candidate", "candidate", control_sha)
        frozen = json.loads((runner.ROOT / "data/s07/bindworkload-probes.json").read_text())
        binaries = {"oracle": control["artifacts"]["go"]["sha256"], "rust": candidate["artifacts"]["normal"]["sha256"]}
        report = {"version": 1, "diagnostic_only": True, "kind": "full_graph_diagnostic", "files": 13094,
                  "manifest_sha256": {"control": control_sha, "candidate": candidate_sha},
                  "diagnostic_subset": False, "source_stable": True, "parity": 1,
                  "source_fingerprint": candidate["source_fingerprint"],
                  "source_fingerprint_after": candidate["source_fingerprint"],
                  "binary_sha256": binaries, "binary_sha256_after": binaries,
                  "expected_scalars": frozen["expected_scalars"],
                  **{key: frozen[key] for key in ("input_sha256", "options_sha256", "workload_sha256")},
                  "runs": [{"workers": workers, "files": 13094, "passed_files": 13094, "parity": 1,
                            "expected_scalars": frozen["expected_scalars"], "rust_scalars": frozen["expected_scalars"],
                            "results": [{"index": index, "equal": True, "raw_exact": True, "first_difference": None}
                                        for index in range(13094)]} for workers in (1, 8)],
                  "binding_paths": [{"version": 1, "workers": workers, "files": 13094,
                                     "bound_in_place_files": 13094, "fallback_files": 0,
                                     "loaded_input_sha256": EXPECTED["loaded_input_sha256"]} for workers in (1, 8)]}
        path = self.root / "graph.json"
        runner.write_json(path, report)
        runner.validate_graph_report(path, control, candidate, control_sha, candidate_sha)
        report["runs"][1]["results"].pop()
        runner.write_json(path, report)
        with self.assertRaisesRegex(ValueError, "missing per-file"):
            runner.validate_graph_report(path, control, candidate, control_sha, candidate_sha)
        report["binary_sha256"]["rust"] = "0" * 64
        runner.write_json(path, report)
        with self.assertRaisesRegex(ValueError, "graph binaries"):
            runner.validate_graph_report(path, control, candidate, control_sha, candidate_sha)


if __name__ == "__main__":
    unittest.main()
