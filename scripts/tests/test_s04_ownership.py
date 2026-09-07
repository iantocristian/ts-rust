import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch


SOURCE = Path(__file__).resolve().parents[1] / "s04_ownership.py"
SPEC = importlib.util.spec_from_file_location("s04_ownership_tests_module", SOURCE)
ownership = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ownership)


def measured_report():
    return {"metrics": {**{name: True for name in ownership.SCENARIOS[:5]},
                        "live_owner_delta": 0, "live_allocation_delta": 0},
            "tests": [{"id": name, "result": "pass"} for name in ownership.SCENARIOS]}


def suite_output():
    tests = "\n".join(f"test tests::e3_{name} ... ok" for name in ownership.SCENARIOS)
    return (tests + "\ntest result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;\n").encode()


class OwnershipProducerTests(unittest.TestCase):
    def test_duplicate_and_nonfinite_json_cannot_hide_failed_measurements(self):
        for text in ('{"live_owner_delta":2,"live_owner_delta":0}',
                     '{"live_owner_delta":NaN}', '{"live_owner_delta":Infinity}'):
            with self.assertRaises(ValueError):
                ownership.strict_json_loads(text)

    def test_native_example_cannot_claim_unimplemented_criteria(self):
        report = measured_report()
        report["metrics"]["shared_pool_panic_retirement"] = True
        with self.assertRaises(ValueError):
            ownership.validate_measurements(report)

    def test_requires_actual_nonnegative_numeric_counters(self):
        for value in (None, False, "0", -1, 0.0):
            with self.subTest(value=value):
                report = measured_report()
                report["metrics"]["live_owner_delta"] = value
                with self.assertRaises(ValueError):
                    ownership.validate_measurements(report)

    def test_missing_duplicate_skipped_or_failed_scenario_cannot_pass(self):
        for mutate in (
            lambda rows: rows.pop(),
            lambda rows: rows.append(rows[0].copy()),
            lambda rows: rows[0].update(result="skip"),
            lambda rows: rows[0].update(result="fail"),
        ):
            report = measured_report()
            mutate(report["tests"])
            with self.assertRaises(ValueError):
                ownership.validate_measurements(report)

    def test_native_example_cannot_claim_instrumentation(self):
        for metric in ("miri", "address_sanitizer"):
            report = measured_report()
            report["metrics"][metric] = True
            with self.assertRaises(ValueError):
                ownership.validate_measurements(report)

    def test_zero_filtered_ignored_and_missing_named_tests_are_rejected(self):
        output = suite_output()
        for changed in (
            b"test result: ok. 0 passed; 0 failed; 0 ignored; 7 filtered out;\n",
            output.replace(b"0 ignored", b"1 ignored"),
            output.replace(b"e3_id_exhaustion", b"unrelated_test"),
            output.replace(b"e3_id_exhaustion ... ok", b"e3_id_exhaustion ... FAILED"),
        ):
            with self.assertRaises(ValueError):
                ownership.validate_test_output(changed)

    def test_measured_nonzero_delta_remains_a_failing_metric(self):
        report = measured_report()
        report["metrics"]["live_owner_delta"] = 2
        self.assertEqual(ownership.validate_measurements(report)["metrics"]["live_owner_delta"], 2)

    def test_sanitizer_flags_cannot_be_overridden_by_encoded_rustflags(self):
        calls = []

        def invoke(root, args, env=None):
            calls.append((args, copy.deepcopy(env)))
            if "-Vv" in args:
                return b"rustc test\nhost: aarch64-apple-darwin\n"
            if "run" in args and "e3" in args:
                return json.dumps(measured_report()).encode()
            if "setup" in args:
                return b""
            return suite_output()

        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "data/s04").mkdir(parents=True)
            (root / "data/s04/e3-cases.json").write_text(json.dumps(sorted(ownership.SCENARIOS)))
            (root / "data/s04/toolchains.toml").write_text('nightly = "nightly-2026-09-05"\n')
            with patch.object(ownership, "invoke", invoke), patch.dict(ownership.os.environ, {"CARGO_ENCODED_RUSTFLAGS": ""}):
                report = ownership.run(root)
        asan = [(args, env) for args, env in calls if "-Zbuild-std" in args]
        self.assertEqual(len(asan), 1)
        args, env = asan[0]
        self.assertNotIn("CARGO_ENCODED_RUSTFLAGS", env)
        self.assertEqual(env["RUSTFLAGS"], "-Zsanitizer=address")
        self.assertIn("--target", args)
        self.assertEqual(report["metrics"]["miri"], True)
        self.assertEqual(report["metrics"]["address_sanitizer"], True)
        self.assertEqual(report["tests"], {name: "pass" for name in ownership.SCENARIOS})

    def test_failed_instrumentation_never_returns_passing_report(self):
        # invoke is the shared failure boundary used for every child process.
        failed = ownership.subprocess.CompletedProcess([], 1, stdout=b"partial", stderr=b"instrumentation error")
        with patch.object(ownership.subprocess, "run", return_value=failed):
            with self.assertRaises(RuntimeError):
                ownership.invoke(Path("."), ["cargo", "miri", "test"])

    def test_instrumentation_scrubs_flags_wrappers_sysroots_and_runners(self):
        injected = {
            "PATH": "/kept", "RUSTC": "/wrong/compiler", "RUSTC_WRAPPER": "/wrong/wrapper",
            "RUSTC_WORKSPACE_WRAPPER": "/wrong/workspace-wrapper", "RUSTUP_TOOLCHAIN": "nightly",
            "RUSTFLAGS": "--cfg bypass", "CARGO_ENCODED_RUSTFLAGS": "",
            "CARGO_BUILD_RUSTFLAGS": "--cfg bypass", "CARGO_BUILD_RUSTC": "/wrong/compiler",
            "CARGO_TARGET_AARCH64_APPLE_DARWIN_RUSTFLAGS": "--cfg bypass",
            "CARGO_TARGET_AARCH64_APPLE_DARWIN_RUNNER": "/wrong/runner",
            "MIRIFLAGS": "-Zmiri-disable-validation", "MIRI_SYSROOT": "/wrong/sysroot",
            "MIRI_BE_RUSTC": "1", "ASAN_OPTIONS": "start_deactivated=1",
            "LSAN_OPTIONS": "suppressions=/wrong/suppressions",
        }
        env = ownership.instrumentation_environment(injected, Path("/workspace"))
        self.assertEqual(env, {"PATH": "/kept", "RUSTUP_HOME": "/workspace/target/s04-rustup",
                               "CARGO_HOME": "/workspace/target/s04-cargo-home"})

    def test_required_scenarios_run_under_strict_provenance(self):
        calls = []

        def invoke(root, args, env=None):
            calls.append((args, copy.deepcopy(env)))
            if "-Vv" in args:
                return b"rustc test\nhost: aarch64-apple-darwin\n"
            if "run" in args and "e3" in args:
                return json.dumps(measured_report()).encode()
            if "setup" in args:
                return b""
            return suite_output()

        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "data/s04").mkdir(parents=True)
            (root / "data/s04/e3-cases.json").write_text(json.dumps(sorted(ownership.SCENARIOS)))
            (root / "data/s04/toolchains.toml").write_text('nightly = "nightly-2026-09-05"\n')
            with patch.object(ownership, "invoke", invoke):
                ownership.run(root)
        runs = [(args, env) for args, env in calls if "miri" in args and "test" in args]
        docs = [args for args, _ in calls if "--doc" in args]
        self.assertEqual(docs, [["cargo", "test", "--package", "ts_arena", "--doc", "--locked"]])
        self.assertEqual(len(runs), 1)
        args, env = runs[0]
        self.assertEqual(env["MIRIFLAGS"], "-Zmiri-strict-provenance")
        self.assertIn("--lib", args)
        self.assertIn("+nightly-2026-09-05", args)


if __name__ == "__main__":
    unittest.main()
