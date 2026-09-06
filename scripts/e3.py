#!/usr/bin/env python3
"""Run the S04 ownership scenarios normally, under Miri, and under ASan."""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent


def run(command: list[str], env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        command,
        cwd=ROOT,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    if result.stdout:
        print(result.stdout, file=sys.stderr, end="")
    if result.returncode:
        raise SystemExit(result.returncode)
    return result


def main() -> None:
    run(["cargo", "test", "-p", "e3_harness"])
    run(["cargo", "test", "--release", "-p", "e3_harness"])
    run(["cargo", "+nightly", "miri", "test", "-p", "e3_harness"])

    version = run(["rustc", "+nightly", "-vV"]).stdout
    host = next(line.split(": ", 1)[1] for line in version.splitlines() if line.startswith("host: "))
    sanitizer_env = os.environ.copy()
    sanitizer_env["RUSTFLAGS"] = "-Zsanitizer=address"
    sanitizer_env["RUSTDOCFLAGS"] = "-Zsanitizer=address"
    run(
        [
            "cargo",
            "+nightly",
            "test",
            "-p",
            "e3_harness",
            "--target",
            host,
        ],
        sanitizer_env,
    )

    result = run(["cargo", "run", "--quiet", "-p", "e3_harness", "--bin", "e3-harness"])
    report = json.loads(result.stdout.splitlines()[-1])
    report["metrics"]["miri"] = True
    report["metrics"]["address_sanitizer"] = True
    print(json.dumps(report, separators=(",", ":")))


if __name__ == "__main__":
    main()
