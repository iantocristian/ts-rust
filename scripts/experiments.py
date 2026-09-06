#!/usr/bin/env python3
"""E3 and E4 producers for status/runs.toml.

e3: runs the S04 ownership scenarios (crates/e3_harness) in release mode, then
    the same scenarios as debug tests, under Miri and under AddressSanitizer on
    the nightly toolchain, and prints the metrics named in status/experiments.toml.
e4: builds the Go oracle (tools/oracle-e4) inside the unmodified pinned module
    through a `go build -overlay`, runs it over tests/e4/fixtures.json, then runs
    the Rust harness (crates/e4_harness) that compares ts_jsstring with the
    oracle and prints one metric per criterion.

Each subcommand prints exactly one JSON object; tool output goes to stderr. A
measured failure is a false metric; a missing tool is a failed run.
"""

import argparse
import importlib.util
import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

OVERLAY = {
    "cmd/oracle-e4/main.go": "tools/oracle-e4/main.go",
    "internal/ls/lsconv/oracle_export.go": "tools/oracle-e4/overlay/lsconv.go",
    "internal/printer/oracle_export.go": "tools/oracle-e4/overlay/printer.go",
    "internal/vfs/internal/oracle_export.go": "tools/oracle-e4/overlay/vfs_internal.go",
    "internal/vfs/oracle/oracle.go": "tools/oracle-e4/overlay/vfs_oracle.go",
    "internal/core/oracle_export.go": "tools/oracle-e4/overlay/core.go",
}


def bootstrap():
    spec = importlib.util.spec_from_file_location("tracking_bootstrap", ROOT / "scripts/tracking-bootstrap.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def run(args, cwd, env=None, capture=False):
    print("+ " + " ".join(str(a) for a in args), file=sys.stderr)
    return subprocess.run(
        args,
        cwd=cwd,
        env=env,
        stdout=subprocess.PIPE if capture else sys.stderr,
        stderr=sys.stderr,
        check=False,
        text=capture,
    )


def require(args):
    try:
        completed = subprocess.run(args, stdout=sys.stderr, stderr=sys.stderr, check=False)
    except OSError as error:
        raise OSError(f"{args[0]} is not available: {error}") from error
    if completed.returncode != 0:
        raise OSError(f"{' '.join(args)} failed; the tool is not usable")


def host_triple():
    out = subprocess.run(["rustc", "-vV"], capture_output=True, text=True, check=True).stdout
    for line in out.splitlines():
        if line.startswith("host: "):
            return line[len("host: "):]
    raise ValueError("rustc -vV did not report a host triple")


def metrics_from(stdout):
    report = json.loads(stdout)
    metrics = report["metrics"]
    if not isinstance(metrics, dict):
        raise ValueError("harness output is not a metrics object")
    return metrics


def e3(root):
    require(["cargo", "--version"])
    require(["cargo", "+nightly", "miri", "--version"])
    release = run(["cargo", "run", "--release", "--locked", "-p", "e3_harness"], root, capture=True)
    if release.returncode != 0:
        raise subprocess.SubprocessError("the release harness did not complete")
    metrics = metrics_from(release.stdout)
    metrics["debug_tests"] = run(["cargo", "test", "--locked", "-p", "e3_harness"], root).returncode == 0
    metrics["miri"] = run(["cargo", "+nightly", "miri", "test", "--locked", "-p", "e3_harness"], root).returncode == 0
    env = dict(os.environ)
    env["RUSTFLAGS"] = "-Zsanitizer=address"
    asan = ["cargo", "+nightly", "test", "--locked", "-p", "e3_harness", "--target", host_triple(), "--target-dir", "target/asan"]
    metrics["address_sanitizer"] = run(asan, root, env=env).returncode == 0
    return metrics


def e4(root):
    require(["go", "version"])
    require(["cargo", "--version"])
    tracking = bootstrap()
    upstream = tracking.verify_upstream(root)
    module = upstream / "tsc"
    out_dir = root / "target/e4"
    out_dir.mkdir(parents=True, exist_ok=True)
    overlay = out_dir / "overlay.json"
    overlay.write_text(json.dumps({"Replace": {str(module / k): str(root / v) for k, v in OVERLAY.items()}}))
    env = os.environ.copy()
    for variable in ("GOOS", "GOARCH", "GOARM", "GOARM64", "GOAMD64", "GO386", "GOMIPS", "GOMIPS64", "GOPPC64", "GORISCV64"):
        env.pop(variable, None)
    env.update(CGO_ENABLED="0", GOWORK="off", GOFLAGS="")
    fixtures = root / "tests/e4/fixtures.json"
    oracle = run(["go", "run", "-mod=readonly", f"-overlay={overlay}", "./cmd/oracle-e4", str(fixtures)], module, env=env, capture=True)
    if oracle.returncode != 0:
        raise subprocess.SubprocessError("the Go oracle did not complete")
    oracle_path = out_dir / "oracle.json"
    oracle_path.write_text(oracle.stdout)
    tracking.verify_upstream(root)
    harness = run(["cargo", "run", "--release", "--locked", "-p", "e4_harness", "--", str(fixtures), str(oracle_path)], root, capture=True)
    if harness.returncode != 0:
        raise subprocess.SubprocessError("the Rust harness did not complete")
    return metrics_from(harness.stdout)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("experiment", choices=("e3", "e4"))
    args = parser.parse_args()
    try:
        metrics = {"e3": e3, "e4": e4}[args.experiment](ROOT)
    except (OSError, ValueError, KeyError, subprocess.SubprocessError, json.JSONDecodeError) as error:
        print(f"{args.experiment} failed: {error}", file=sys.stderr)
        return 1
    print(json.dumps({"metrics": metrics}, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
