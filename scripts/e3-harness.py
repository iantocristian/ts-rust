#!/usr/bin/env python3
"""The e3 producer: ownership scenarios, plus Miri and AddressSanitizer.

Three runs of the same scenarios:

* a release build, which is where the ownership contract's import validation has
  to hold, and which is the run whose report supplies the scenario metrics;
* the same scenarios under Miri, which is why the concurrent scenario has a
  reduced workload there (it still crosses a page boundary and still runs
  several threads through the miss, recheck and publish paths);
* the same scenarios under AddressSanitizer, built with `-Zbuild-std` for the
  host target.

Only the criteria whose scenarios exist are emitted. A scenario that fails is a
false metric with exit code 0; a missing toolchain is a failed run, because an
absent Miri is not a measurement.
"""

import json
import os
from pathlib import Path
import subprocess
import sys

NIGHTLY = "nightly"


def require(args, cwd, env=None):
    """The tool itself must be present and answer; otherwise there is no measurement."""
    try:
        completed = subprocess.run(args, cwd=cwd, env=env, stdout=sys.stderr, stderr=sys.stderr, check=False)
    except OSError as error:
        raise OSError(f"{args[0]} is not available: {error}") from error
    if completed.returncode != 0:
        raise OSError(f"{' '.join(args)} failed; the tool is not usable")


def run(args, cwd, env=None, capture=False):
    print("+ " + " ".join(args), file=sys.stderr)
    return subprocess.run(
        args,
        cwd=cwd,
        env=env,
        stdout=subprocess.PIPE if capture else sys.stderr,
        stderr=sys.stderr,
        text=True,
        check=False,
    )


def host_target():
    output = subprocess.run(["rustc", "-vV"], stdout=subprocess.PIPE, text=True, check=True).stdout
    for line in output.splitlines():
        if line.startswith("host: "):
            return line[len("host: ") :].strip()
    raise OSError("rustc -vV did not report a host target")


def scenarios(root):
    """The release run, whose JSON report carries the scenario metrics."""
    completed = run(
        ["cargo", "run", "--release", "--locked", "--quiet", "--package", "ts_e3_harness"],
        root,
        capture=True,
    )
    if completed.returncode != 0:
        raise OSError("the release scenario run failed to execute")
    report = json.loads(completed.stdout)
    return report["metrics"]


def miri(root):
    env = os.environ.copy()
    # The scenarios allocate and drop across threads; isolation is not needed and
    # the reduced workload keeps the interpreted run to a few minutes.
    env["MIRIFLAGS"] = "-Zmiri-disable-isolation"
    env["TS_E3_SCALE"] = "reduced"
    require(["cargo", f"+{NIGHTLY}", "miri", "--version"], root, env)
    completed = run(
        ["cargo", f"+{NIGHTLY}", "miri", "run", "--quiet", "--package", "ts_e3_harness", "--", "--scenarios"],
        root,
        env,
    )
    return completed.returncode == 0


def address_sanitizer(root):
    target = host_target()
    env = os.environ.copy()
    env["RUSTFLAGS"] = "-Zsanitizer=address"
    # -Zbuild-std rebuilds the standard library with the sanitizer, which is what
    # makes the instrumentation cover allocation and teardown as well.
    require(["cargo", f"+{NIGHTLY}", "--version"], root, env)
    built = run(
        [
            "cargo", f"+{NIGHTLY}", "build", "--quiet", "-Zbuild-std",
            "--target", target, "--package", "ts_e3_harness",
        ],
        root,
        env,
    )
    if built.returncode != 0:
        return False
    binary = root / "target" / target / "debug" / "ts_e3_harness"
    if not binary.is_file():
        raise OSError(f"the sanitizer build produced no binary at {binary}")
    return run([str(binary), "--scenarios"], root, env).returncode == 0


def main():
    root = Path(__file__).resolve().parent.parent
    try:
        require(["cargo", "--version"], root)
        metrics = scenarios(root)
        metrics["miri"] = miri(root)
        metrics["address_sanitizer"] = address_sanitizer(root)
    except (OSError, ValueError, KeyError, subprocess.SubprocessError, json.JSONDecodeError) as error:
        print(f"e3 harness failed: {error}", file=sys.stderr)
        return 1
    print(json.dumps({"metrics": metrics}, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
