"""Build provenance regressions, including an actual conflicting Cargo build."""
import copy
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import s07_benchmark as benchmark


class CargoArtifacts(unittest.TestCase):
    def artifact(self):
        manifest = Path("/benchmark/crates/ts_bench/Cargo.toml")
        executable = "/configured-output/native/release/ts-bench"
        return manifest, {
            "reason": "compiler-artifact", "manifest_path": str(manifest),
            "target": {"name": "ts-bench", "kind": ["bin"], "crate_types": ["bin"]},
            "features": [], "profile": {"opt_level": "3", "debuginfo": 0,
                "debug_assertions": False, "overflow_checks": False, "test": False},
            "filenames": [executable], "executable": executable,
        }

    def decode(self, manifest, rows, allocation=False):
        return benchmark.rust_executable(
            b"\n".join(json.dumps(row).encode() for row in rows), manifest, allocation)

    def test_exact_artifact_selection_and_feature_mode(self):
        manifest, artifact = self.artifact()
        done = {"reason": "build-finished", "success": True}
        unrelated = copy.deepcopy(artifact)
        unrelated["manifest_path"] = "/another/Cargo.toml"
        self.assertEqual(self.decode(manifest, [unrelated, artifact, done]), Path(artifact["executable"]))
        artifact["features"] = ["allocation"]
        self.assertEqual(self.decode(manifest, [artifact, done], True), Path(artifact["executable"]))

    def test_rejects_missing_duplicate_failed_wrong_profile_or_feature_artifact(self):
        manifest, artifact = self.artifact()
        done = {"reason": "build-finished", "success": True}
        invalid = [[], [artifact], [done], [artifact, artifact, done],
                   [artifact, done, done], [done, artifact],
                   [artifact, {**done, "success": False}], [[], artifact, done]]
        for field, value in (("features", ["allocation"]), ("features", ["profile"]),
                             ("executable", "relative/ts-bench"), ("filenames", []),
                             ("filenames", "/configured-output/native/release/ts-bench"),
                             ("target", None), ("profile", None), ("manifest_path", None),
                             ("manifest_path", "/different/Cargo.toml")):
            invalid.append([{**artifact, field: value}, done])
        for field, value in (("opt_level", "0"), ("debuginfo", False),
                             ("debug_assertions", 0), ("overflow_checks", True), ("test", True)):
            invalid.append([{**artifact, "profile": {**artifact["profile"], field: value}}, done])
        for rows in invalid:
            with self.subTest(rows=rows), self.assertRaises(ValueError):
                self.decode(manifest, rows)


class CargoConfiguration(unittest.TestCase):
    def test_compiler_configuration_fails_closed_and_legacy_shadow_is_respected(self):
        with tempfile.TemporaryDirectory(prefix="s07-cargo-config-") as temporary:
            root = Path(temporary)
            (root / ".cargo").mkdir()
            (root / "Cargo.toml").write_text('[workspace]\n')
            env = {"CARGO_HOME": str(root / "cargo-home")}
            for contents in ('[build]\nrustflags=["-Cpanic=abort"]\n',
                             '[target.\'cfg(unix)\']\nrustflags=["-Copt-level=0"]\n',
                             '[env]\nCARGO_PROFILE_RELEASE_PANIC="abort"\n'):
                (root / ".cargo/config.toml").write_text(contents)
                with self.subTest(contents=contents), self.assertRaises(ValueError):
                    benchmark.release_configuration(env, root)
            # Cargo chooses the extensionless file when both exist. Keep the
            # ignored file fingerprinted, but do not interpret it as effective.
            (root / ".cargo/config").write_text('[net]\noffline=true\n')
            benchmark.release_configuration(env, root)
            self.assertIn(root / ".cargo/config.toml", benchmark.cargo_configuration_paths(env, root))

    def test_actual_cargo_output_and_profile_override_stale_default_binary(self):
        toolchain = (benchmark.ROOT / "rust-toolchain.toml").read_bytes()
        with tempfile.TemporaryDirectory(prefix="s07-cargo-native-") as temporary:
            root = Path(temporary)
            crate = root / "crates/ts_bench"
            (crate / "src").mkdir(parents=True)
            (root / ".cargo").mkdir()
            (root / "cargo-home").mkdir()
            (root / "rust-toolchain.toml").write_bytes(toolchain)
            (root / "Cargo.toml").write_text('''[workspace]
members=["crates/ts_bench"]
resolver="2"
[profile.release]
panic="unwind"
lto="fat"
codegen-units=1
''')
            (crate / "Cargo.toml").write_text('''[package]
name="ts_bench"
version="0.1.0"
edition="2021"
[features]
allocation=[]
[[bin]]
name="ts-bench"
path="src/main.rs"
''')
            (crate / "src/main.rs").write_text('''fn main() {
    println!("current {} {} {}", cfg!(feature="allocation"), cfg!(debug_assertions), cfg!(panic="unwind"));
}
''')
            (crate / "examples").mkdir()
            (crate / "examples/allocation_probe.rs").write_text('fn main() { println!("probe {} {}", cfg!(debug_assertions), cfg!(panic="unwind")); }')
            config = root / ".cargo/config.toml"
            config.write_text('''[build]
target-dir="configured-output"
target="intentionally-not-a-real-target"
[profile.release]
panic="abort"
lto=false
codegen-units=16
[profile.release.package.ts_bench]
opt-level=0
debug=true
debug-assertions=true
overflow-checks=true
codegen-units=32
[profile.dev]
opt-level=3
panic="abort"
debug=false
debug-assertions=false
overflow-checks=false
[target.'cfg(unix)']
runner="this-runner-must-never-execute"
''')
            home_config = root / "cargo-home/config.toml"
            home_config.write_text('''[net]
offline=true
[registries.preserved]
index="https://example.invalid/preserved-registry"
[profile.release.package."*"]
opt-level=1
codegen-units=64
''')
            originals = {path: path.read_bytes() for path in (config, home_config)}
            stale = root / "target/release/ts-bench"
            stale.parent.mkdir(parents=True)
            stale.write_text("stale executable from before target-dir changed")
            invocations = []

            def execute(args, *, cwd, env, **kwargs):
                # Verbose rustc commands independently establish the effective
                # options Cargo's JSON profile does not report (LTO/CG units).
                actual = args + (["-vv"] if args[0] == "cargo" else [])
                result = subprocess.run(actual, cwd=cwd, env=env, capture_output=True, check=True)
                invocations.append((args, result.stderr.decode()))
                return result.stdout

            with patch.object(benchmark, "ROOT", root), patch.object(benchmark, "CACHE", root / "cache"), \
                 patch.dict(os.environ, {"CARGO_HOME": str(root / "cargo-home")}), \
                 patch.object(benchmark, "command", execute):
                env = benchmark.native_environment()
                stable, host = benchmark.rust_native_toolchain(env)
                subprocess.run(["cargo", "+"+stable, "generate-lockfile", "--offline"],
                               cwd=root, env=env, capture_output=True, check=True)
                for allocation in (False, True):
                    binary, runtime_env = benchmark.build_rust(allocation)
                    output = subprocess.check_output([str(binary)], env=runtime_env).decode().strip()
                    self.assertEqual(output, f"current {str(allocation).lower()} false true")
                    args, compiler = invocations[-1]
                    self.assertEqual(args[args.index("--target")+1], host)
                    self.assertIn("-C lto=fat", compiler)
                    self.assertIn("-C codegen-units=1", compiler)
                    self.assertNotIn("-C panic=abort", compiler)
                    self.assertEqual(binary.read_bytes(), (root / "configured-output" / host / "release/ts-bench").read_bytes())
                for mode in ("debug", "release"):
                    binary, runtime_env = benchmark.build_allocation_probe(mode, stable)
                    self.assertEqual(subprocess.check_output([str(binary)], env=runtime_env).decode().strip(),
                                     f"probe {str(mode == 'debug').lower()} true")
                    args, _ = invocations[-1]
                    self.assertEqual(args[2], "build")
                    self.assertEqual(args[args.index("--target")+1], host)
                self.assertEqual(runtime_env["CARGO_HOME"], str(root / "cargo-home"))
            for path, original in originals.items():
                self.assertEqual(path.read_bytes(), original)
            self.assertEqual(stale.read_text(), "stale executable from before target-dir changed")

    def test_persisted_go_target_and_experiments_cannot_change_native_baseline(self):
        with tempfile.TemporaryDirectory(prefix="s07-go-native-") as temporary:
            settings = Path(temporary) / "go.env"
            settings.write_text("GOOS=linux\nGOARCH=386\nGOEXPERIMENT=nogreenteagc\nGOPROXY=https://example.invalid/preserved\nGONOSUMDB=private.example.invalid\n")
            env = benchmark.go_native_environment({**os.environ, "GOENV": str(settings),
                                                   "GOAMD64": "v3", "MIMALLOC_PURGE_DELAY": "-1",
                                                   "mimalloc_eager_commit": "0"})
            actual = json.loads(subprocess.check_output(["go", "env", "-json", "GOOS", "GOARCH", "GOHOSTOS", "GOHOSTARCH", "GOEXPERIMENT", "GOPROXY", "GONOSUMDB"], env=env))
            self.assertEqual(actual["GOOS"], actual["GOHOSTOS"])
            self.assertEqual(actual["GOARCH"], actual["GOHOSTARCH"])
            self.assertEqual(actual["GOEXPERIMENT"], "")
            self.assertEqual(actual["GOPROXY"], "https://example.invalid/preserved")
            self.assertEqual(actual["GONOSUMDB"], "private.example.invalid")
            self.assertEqual(env["GOENV"], "off")
            self.assertNotIn("GOAMD64", env)
            self.assertFalse(any(key.lower().startswith("mimalloc_") for key in env))


if __name__ == "__main__":
    unittest.main()
