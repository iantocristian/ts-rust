#!/usr/bin/env python3
"""Read the S04 toolchain pins and locate reusable instrumentation caches."""

import argparse
import os
from pathlib import Path
import re
import sys
import tomllib


def load_toolchains(root):
    pins = tomllib.loads((Path(root) / "data/s04/toolchains.toml").read_text())
    patterns = {"nightly": r"nightly-\d{4}-\d{2}-\d{2}",
                "go": r"go\d+\.\d+\.\d+", "msrv": r"\d+\.\d+\.\d+"}
    for name, pattern in patterns.items():
        if not isinstance(pins.get(name), str) or not re.fullmatch(pattern, pins[name]):
            raise ValueError(f"invalid S04 {name} toolchain pin")
    workspace = tomllib.loads((Path(root) / "Cargo.toml").read_text())
    declared = workspace["workspace"]["package"]["rust-version"].split(".")
    if declared + ["0"] * (3 - len(declared)) != pins["msrv"].split("."):
        raise ValueError("S04 msrv pin does not match Cargo.toml workspace rust-version")
    return pins


def cache_home(root=None, base=None):
    """Keep reusable sysroots outside target/, preserving explicit cache choices."""
    env = os.environ if base is None else base
    if env.get("S04_CACHE_HOME"):
        return Path(env["S04_CACHE_HOME"]).expanduser().resolve()
    home = Path(env.get("HOME", str(Path.home())))
    if sys.platform == "darwin":
        cache = home / "Library/Caches"
    else:
        cache = Path(env.get("XDG_CACHE_HOME", str(home / ".cache")))
    return cache / "ts-rust/s04"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=("github-outputs",))
    args = parser.parse_args()
    try:
        pins = load_toolchains(Path(__file__).resolve().parents[1])
        if args.operation == "github-outputs":
            for name, value in pins.items():
                print(f"{name}={value.removeprefix('go') if name == 'go' else value}")
            print(f"cache={cache_home()}")
    except (OSError, KeyError, ValueError) as error:
        print(f"S04 runtime configuration failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
