import copy
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import sys
import unittest
from unittest.mock import patch


sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

SPEC = importlib.util.spec_from_file_location("s04", Path(__file__).resolve().parents[1] / "s04.py")
s04 = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(s04)


class ComparisonTests(unittest.TestCase):
    def setUp(self):
        self.probes = [
            {"id": f"{criterion}/1", "group": criterion, "criterion": criterion}
            for criterion in s04.TEXT_CRITERIA
        ]
        self.results = [
            {"id": probe["id"], "panic": False, "value": ["6162", True, 1]}
            for probe in self.probes
        ]

    def test_complete_equal_results_pass_each_implemented_criterion(self):
        report, failures = s04.compare(self.probes, self.results, copy.deepcopy(self.results))
        self.assertFalse(failures)
        self.assertTrue(all(report["metrics"][criterion] for criterion in s04.TEXT_CRITERIA))
        self.assertEqual(report["tests"], {probe["group"]: "pass" for probe in self.probes})

    def test_boolean_and_numeric_result_types_cannot_be_interchanged(self):
        for original, changed in [(True, 1), (False, 0), (1, 1.0), (0, False)]:
            with self.subTest(original=original, changed=changed):
                expected = copy.deepcopy(self.results)
                actual = copy.deepcopy(self.results)
                for result in expected:
                    result["value"] = {"nested": [original]}
                for result in actual:
                    result["value"] = {"nested": [changed]}
                report, failures = s04.compare(self.probes, expected, actual)
                self.assertEqual(len(failures), len(self.probes))
                self.assertTrue(all(not report["metrics"][criterion] for criterion in s04.TEXT_CRITERIA))

    def test_missing_extra_reordered_and_duplicate_results_are_rejected(self):
        bad_inventories = [
            self.results[:-1],
            self.results + [self.results[0]],
            list(reversed(self.results)),
            [self.results[0]] * len(self.results),
        ]
        for bad in bad_inventories:
            for side in ("oracle", "rust"):
                with self.subTest(side=side, bad=bad):
                    with self.assertRaises(ValueError):
                        s04.compare(self.probes, bad if side == "oracle" else self.results,
                                    bad if side == "rust" else self.results)

    def test_malformed_result_shapes_are_rejected(self):
        for invalid in [None, [], "result", {"id": self.results[0]["id"]},
                        {"id": self.results[0]["id"], "panic": 0, "value": None},
                        {"id": self.results[0]["id"], "panic": True, "value": 1},
                        {**self.results[0], "extra": False}]:
            with self.subTest(invalid=invalid):
                actual = copy.deepcopy(self.results)
                actual[0] = invalid
                with self.assertRaises(ValueError):
                    s04.compare(self.probes, self.results, actual)
        with self.assertRaises(ValueError):
            s04.compare(self.probes, self.results, {})

    def test_nonfinite_and_nonjson_result_values_are_rejected(self):
        for value in [float("nan"), float("inf"), float("-inf"), (1, 2), {1: "key"}, b"bytes"]:
            for side in ("oracle", "rust"):
                with self.subTest(value=value, side=side):
                    invalid = copy.deepcopy(self.results)
                    invalid[0]["value"] = {"nested": [value]}
                    with self.assertRaises(ValueError):
                        s04.compare(self.probes, invalid if side == "oracle" else self.results,
                                    invalid if side == "rust" else self.results)

    def test_matching_panics_pass_but_success_is_not_a_panic(self):
        expected = copy.deepcopy(self.results)
        expected[0].update(panic=True, value="runtime error: index out of range [-1]")
        actual = copy.deepcopy(expected)
        actual[0]["value"] = "index out of bounds: the len is 2 but the index is 18446744073709551615"
        report, failures = s04.compare(self.probes, expected, actual)
        self.assertTrue(report["metrics"][s04.TEXT_CRITERIA[0]])
        self.assertFalse(failures)
        actual[0]["panic"] = False
        report, failures = s04.compare(self.probes, expected, actual)
        self.assertFalse(report["metrics"][s04.TEXT_CRITERIA[0]])
        self.assertEqual(len(failures), 1)

    def test_bounds_cannot_match_assertion_overflow_or_unknown_panic(self):
        expected = copy.deepcopy(self.results)
        expected[0].update(panic=True, value="runtime error: slice bounds out of range [:9] with length 2")
        actual = copy.deepcopy(expected)
        for reason, kind in [
            ("assertion failed: offset <= len", "assertion"),
            ("attempt to add with overflow", "overflow"),
            ("unexpected slice bounds failure", "other"),
            ("non-string Rust panic payload", "other"),
        ]:
            with self.subTest(reason=reason):
                actual[0]["value"] = reason
                report, failures = s04.compare(self.probes, expected, actual)
                self.assertFalse(report["metrics"][self.probes[0]["criterion"]])
                self.assertEqual(len(failures), 1)
                self.assertEqual(failures[0]["panic_classes"], {"oracle": "bounds", "rust": kind})
                self.assertEqual(failures[0]["actual"]["value"], reason)

    def test_unknown_panics_never_pass_even_with_identical_payloads(self):
        for reason in ("unexpected panic", "assertion failed: invariant", "attempt to add with overflow"):
            with self.subTest(reason=reason):
                expected = copy.deepcopy(self.results)
                expected[0].update(panic=True, value=reason)
                report, failures = s04.compare(self.probes, expected, copy.deepcopy(expected))
                self.assertEqual(len(failures), 1)
                self.assertFalse(report["metrics"][self.probes[0]["criterion"]])

    def test_contract_class_and_payload_are_checked_for_every_panic(self):
        expected = copy.deepcopy(self.results)
        expected[0].update(panic=True, value="Bad line number. Line: -1, lineStarts.length: 2.")
        actual = copy.deepcopy(expected)
        for message in ("Bad line number. Line: 2, lineStarts.length: 2.",
                        "Bad UTF-16 character offset. Line: 0, character: 9."):
            actual[0]["value"] = message
            _, failures = s04.compare(self.probes, expected, actual)
            self.assertEqual(len(failures), 1)

    def test_runtime_bounds_normalization_is_narrow(self):
        for message, runtime in [
            ("runtime error: index out of range [-1]", "oracle"),
            ("runtime error: index out of range [8] with length 2", "oracle"),
            ("runtime error: slice bounds out of range [:8] with length 2", "oracle"),
            ("runtime error: slice bounds out of range [-1:]", "oracle"),
            ("runtime error: slice bounds out of range [9:2]", "oracle"),
            ("index out of bounds: the len is 2 but the index is 8", "rust"),
            ("range end index 8 out of range for slice of length 2", "rust"),
            ("range start index 8 out of range for slice of length 2", "rust"),
            ("slice index starts at 9 but ends at 2", "rust"),
        ]:
            with self.subTest(message=message):
                self.assertEqual(s04.panic_class(message, runtime), "bounds")
                self.assertEqual(s04.panic_class("assertion failed: " + message, runtime), "assertion")
                self.assertEqual(s04.panic_class(message + ": unexpected suffix", runtime), "other")

    def test_empty_or_duplicate_probe_inventory_cannot_pass(self):
        for probes in [[], [self.probes[0]] * 2, [None], [{"id": 1, "group": "g", "criterion": "c"}]]:
            with self.subTest(probes=probes):
                with self.assertRaises(ValueError):
                    s04.compare(probes, [], [])

    def test_missing_criterion_stays_false(self):
        report, failures = s04.compare(self.probes[:1], self.results[:1], self.results[:1])
        self.assertFalse(failures)
        for criterion in s04.TEXT_CRITERIA[1:]:
            self.assertFalse(report["metrics"][criterion])
            self.assertEqual(report["metrics"][f"{criterion}_probes"], 0)

    def test_criterion_mixing_and_unsupported_criterion_are_rejected(self):
        mixed = copy.deepcopy(self.probes)
        mixed[1]["group"] = mixed[0]["group"]
        with self.assertRaises(ValueError):
            s04.compare(mixed, self.results, self.results)
        unknown = copy.deepcopy(self.probes)
        unknown[0]["criterion"] = "not_implemented"
        with self.assertRaises(ValueError):
            s04.compare(unknown, self.results, self.results)

    def test_one_failed_probe_fails_its_whole_scenario_and_criterion(self):
        extra = {**self.probes[0], "id": self.probes[0]["id"] + "-extra"}
        expected = self.results + [{"id": extra["id"], "panic": False, "value": 2}]
        actual = copy.deepcopy(expected)
        actual[-1]["value"] = 3
        report, failures = s04.compare(self.probes + [extra], expected, actual)
        self.assertEqual(len(failures), 1)
        self.assertFalse(report["metrics"][extra["criterion"]])
        self.assertEqual(report["metrics"][extra["criterion"] + "_probes"], 2)
        self.assertEqual(report["tests"][extra["group"]], "fail")

    def test_json_decoder_rejects_duplicate_keys_and_nonfinite_literals(self):
        for text in ['{"id":1,"id":2}', '[NaN]', '[Infinity]', '[-Infinity]', '[1e999]']:
            with self.subTest(text=text):
                with self.assertRaises(ValueError):
                    s04.strict_json_loads(text)
        self.assertEqual(s04.strict_json_loads('[null,true,1,"x"]'), [None, True, 1, "x"])

    def test_e4_executes_cargos_selected_artifact_with_custom_target_directory(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = root / "data/s04/e4-cases.json"
            manifest.parent.mkdir(parents=True)
            manifest.write_text(json.dumps(sorted(probe["group"] for probe in self.probes)))
            manifest.with_name("e4-probes.json").write_text(json.dumps(s04.probe_inventory(self.probes)))
            commands = []

            def fake_command(args, **kwargs):
                commands.append(args)
                if args[-1] == "--check" or args[:2] == ["cargo", "test"]:
                    return b""
                self.assertIsNotNone(kwargs.get("data"))
                return json.dumps(self.results).encode()

            with patch.object(s04, "ROOT", root), \
                 patch.object(s04, "verified_upstream", return_value=root / "upstream"), \
                 patch.object(s04, "text_probes", return_value=self.probes), \
                 patch.object(s04, "go_environment", return_value={}), \
                 patch.object(s04, "go_oracle", return_value=root / "oracle"), \
                 patch.object(s04, "command", side_effect=fake_command), \
                 patch.dict(os.environ, {"CARGO_TARGET_DIR": str(root / "other-target")}):
                report = s04.e4()
            self.assertTrue(report["metrics"][s04.TEXT_CRITERIA[0]])
            self.assertEqual(commands[0][-1], "--check")
            self.assertEqual(commands[2][:2], ["cargo", "test"])
            self.assertIn("--all-targets", commands[2])
            self.assertEqual(commands[3][:2], ["cargo", "run"])
            self.assertNotIn(str(root / "target/release/examples/e4"), commands[3])

    def test_sigma_contexts_cover_range_edges_stride_members_and_holes(self):
        table = """var unicodeCasedRanges = &unicode.RangeTable{
    R16: []unicode.Range16{{0x41, 0x45, 2}},
}
var unicodeCaseIgnorableRanges = &unicode.RangeTable{
    R16: []unicode.Range16{{0x27, 0x2E, 7}},
}
"""
        contexts = set(s04.sigma_contexts(table))
        for char in "@ABCDEF&'(-./":
            for context in ("AΣ" + char, char + "Σ", "A" + char + "Σ", "AΣ" + char + "B"):
                self.assertIn(context.encode(), contexts)
        with self.assertRaises(ValueError):
            list(s04.sigma_contexts(""))

    def test_probe_inventory_rejects_shrink_growth_reorder_and_changed_requests(self):
        probes = self.probes + [{**self.probes[0], "id": "additional", "a": 1}]
        frozen = s04.probe_inventory(probes)
        s04.validate_probe_inventory(copy.deepcopy(probes), frozen)
        changed_request = copy.deepcopy(probes)
        changed_request[-1]["a"] = 2
        changed_type = copy.deepcopy(probes)
        changed_type[-1]["a"] = True
        for changed in (probes[:-1], probes + [{**probes[0], "id": "extra"}],
                        list(reversed(probes)), changed_request, changed_type):
            with self.subTest(changed=changed):
                self.assertEqual(sorted({p["group"] for p in changed}),
                                 sorted({p["group"] for p in probes}))
                with self.assertRaisesRegex(ValueError, "inventory drift"):
                    s04.validate_probe_inventory(changed, frozen)

    def test_prescribed_panic_messages_are_compared_exactly(self):
        probes = copy.deepcopy(self.probes)
        probes[0]["panic_message"] = True
        expected = copy.deepcopy(self.results)
        expected[0].update(panic=True, value="Bad line number. Line: -1, lineStarts.length: 2.")
        report, failures = s04.compare(probes, expected, copy.deepcopy(expected))
        self.assertFalse(failures)
        self.assertTrue(report["metrics"][probes[0]["criterion"]])
        actual = copy.deepcopy(expected)
        actual[0]["value"] = "different panic"
        report, failures = s04.compare(probes, expected, actual)
        self.assertEqual(len(failures), 1)
        self.assertFalse(report["metrics"][probes[0]["criterion"]])
        for value in (None, "", 1):
            actual[0]["value"] = value
            with self.assertRaisesRegex(ValueError, "panic message"):
                s04.compare(probes, expected, actual)


class OracleEnvironmentTests(unittest.TestCase):
    def test_verified_go_cannot_auto_select_a_different_toolchain(self):
        with patch.dict(os.environ, {"GOTOOLCHAIN": "auto", "GOFLAGS": "-tags=changed",
                                     "GOOS": "other", "GOCACHE": "/custom/go-cache"}), \
             patch.object(s04, "command", return_value=b"go version go1.27.1 darwin/arm64\n") as command:
            env = s04.go_environment()
        self.assertEqual(env["GOTOOLCHAIN"], "local")
        self.assertEqual(command.call_args.kwargs["env"]["GOTOOLCHAIN"], "local")
        self.assertEqual(env["GOCACHE"], "/custom/go-cache")
        self.assertEqual(env["GOFLAGS"], "")
        self.assertNotIn("GOOS", env)

    def test_wrong_go_version_is_rejected_before_oracle_build(self):
        with patch.object(s04, "command", return_value=b"go version go1.28.0 darwin/arm64\n"):
            with self.assertRaisesRegex(ValueError, "oracle requires go1.27.1"):
                s04.go_environment()

    def test_git_export_failure_is_reported_before_tar_decoding(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "data").mkdir()
            (root / "data/upstream.json").write_text('{"pin":"missing"}')
            with patch.object(s04, "ROOT", root), \
                 patch.object(s04, "command", side_effect=RuntimeError("command exited 128: git archive")), \
                 patch.object(s04.tarfile, "open") as open_tar:
                with self.assertRaisesRegex(RuntimeError, "command exited 128"):
                    s04.go_oracle(root / "upstream", {})
                open_tar.assert_not_called()

    def test_invalid_git_archive_has_a_contextual_failure(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "data").mkdir()
            (root / "data/upstream.json").write_text('{"pin":"bad-archive"}')
            with patch.object(s04, "ROOT", root), patch.object(s04, "command", return_value=b"bad tar"):
                with self.assertRaisesRegex(RuntimeError, "cannot unpack pinned Go source"):
                    s04.go_oracle(root / "upstream", {})


if __name__ == "__main__":
    unittest.main()
