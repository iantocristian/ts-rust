#!/usr/bin/env python3
"""The e4 producer: compare ts_jsstring with the pinned Corsa packages.

Both sides read `data/e4-fixtures.json` and write one JSON object per criterion
of probe -> value pairs. The Go side (`oracle/e4`) calls only exported entry
points of the pinned packages, so its answers come from production code; the Rust
side calls `ts_jsstring`. Neither sees the other's answers. A criterion passes
only when every probe is present on both sides and every value is identical.

Only the criteria S04 settles are emitted. The scanner's token values (S05), the
encoder (S06) and the printer's source-reuse decision (S08) have no production
path yet, so their criteria stay pending rather than being reported as passing.

Scope limit worth stating: `LowerFirstChar` reaches Go's `unicode.ToLower`, whose
tables follow the Go toolchain's Unicode version rather than the upstream pin.
The port stays on Unicode 15.1.0 like the rest of the crate, so the sweep covers
every code point that version gives a case mapping. Code points added to Unicode
after 15.1.0 are outside the compared domain; upstream calls the helper only from
the fourslash test harness, on ASCII command names.
"""

import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

CRITERIA = (
    "source_decoding",
    "helper_semantics",
    "slice_validity",
    "utf8_positions",
    "utf16_positions",
)


def load_bootstrap(root):
    """Reuse the S01 upstream verification instead of writing a second one."""
    spec = importlib.util.spec_from_file_location(
        "tracking_bootstrap", root / "scripts/tracking-bootstrap.py"
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def require(args, cwd, env=None):
    try:
        completed = subprocess.run(args, cwd=cwd, env=env, stdout=sys.stderr, stderr=sys.stderr, check=False)
    except OSError as error:
        raise OSError(f"{args[0]} is not available: {error}") from error
    if completed.returncode != 0:
        raise OSError(f"{' '.join(args)} failed; the tool is not usable")


def run(args, cwd, env=None):
    print("+ " + " ".join(args), file=sys.stderr)
    return subprocess.run(args, cwd=cwd, env=env, stdout=sys.stderr, stderr=sys.stderr, check=False)


def go_env():
    env = os.environ.copy()
    # The same build policy the S01 oracle producer uses: a host executable from
    # the pinned module, with no workspace file and no cgo.
    for variable in ("GOOS", "GOARCH", "GOARM", "GOARM64", "GOAMD64", "GO386", "GOMIPS", "GOMIPS64", "GOPPC64", "GORISCV64"):
        env.pop(variable, None)
    env.update(CGO_ENABLED="0", GOWORK="off", GOFLAGS="")
    return env


def oracle(root, fixtures, scratch, output):
    env = go_env()
    require(["go", "version"], root, env)
    executable = scratch / ("e4.exe" if os.name == "nt" else "e4")
    if run(["go", "build", "-mod=readonly", "-o", str(executable), "./e4"], root / "oracle", env).returncode != 0:
        raise OSError("the oracle did not build against the pinned submodule")
    files = scratch / "files"
    files.mkdir()
    if run([str(executable), str(fixtures), str(files), str(output)], root, env).returncode != 0:
        raise OSError("the oracle did not answer the fixture probes")
    return json.loads(output.read_text())


def harness(root, fixtures, output):
    command = [
        "cargo", "run", "--release", "--locked", "--quiet", "--package", "ts_e4_harness",
        "--", str(fixtures), str(output),
    ]
    if run(command, root).returncode != 0:
        raise OSError("the Rust harness did not answer the fixture probes")
    return json.loads(output.read_text())


def compare(expected, actual):
    """One criterion's verdict, with the first few differences on stderr."""
    metrics = {}
    for criterion in CRITERIA:
        left, right = expected.get(criterion, {}), actual.get(criterion, {})
        missing = sorted(set(left) - set(right))
        extra = sorted(set(right) - set(left))
        differing = sorted(key for key in set(left) & set(right) if left[key] != right[key])
        ok = not missing and not extra and not differing and bool(left)
        metrics[criterion] = ok
        print(
            f"{criterion}: {len(left)} oracle probes, {len(right)} port probes, "
            f"{len(missing)} missing, {len(extra)} unexpected, {len(differing)} differing"
            f" -> {'pass' if ok else 'FAIL'}",
            file=sys.stderr,
        )
        for key in missing[:5]:
            print(f"    only the oracle answered {key}", file=sys.stderr)
        for key in extra[:5]:
            print(f"    only the port answered {key}", file=sys.stderr)
        for key in differing[:20]:
            print(f"    {key}: oracle {left[key]!r}, port {right[key]!r}", file=sys.stderr)
    return metrics


def main():
    root = Path(__file__).resolve().parent.parent
    try:
        bootstrap = load_bootstrap(root)
        # The oracle's answers are only evidence if they came from the pin.
        bootstrap.verify_upstream(root)
        require(["cargo", "--version"], root)
        # A generated table or fixture manifest that has drifted from the pin
        # would make the comparison meaningless, so it fails the run rather than
        # producing a measurement.
        for generator in ("gen-unicode-case.py", "gen-e4-fixtures.py"):
            require([sys.executable, f"scripts/{generator}", "--check"], root)
        fixtures = root / "data/e4-fixtures.json"
        with tempfile.TemporaryDirectory(prefix="ts-rust-e4-") as temp:
            scratch = Path(temp)
            expected = oracle(root, fixtures, scratch, scratch / "oracle.json")
            actual = harness(root, fixtures, scratch / "port.json")
        metrics = compare(expected, actual)
        bootstrap.verify_upstream(root)
    except (OSError, ValueError, KeyError, subprocess.SubprocessError, json.JSONDecodeError) as error:
        print(f"e4 harness failed: {error}", file=sys.stderr)
        return 1
    print(json.dumps({"metrics": metrics}, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
