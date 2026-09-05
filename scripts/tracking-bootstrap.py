#!/usr/bin/env python3
"""Run the real S01 build checks; stdout is reserved for evidence metrics.

This command never initializes or updates upstream. Register and initialize the
submodule separately before running the oracle check.
"""

import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile


CANONICAL_UPSTREAM = "https://github.com/microsoft/TypeScript"


def git(root, *args):
    return subprocess.check_output(
        ["git", "-C", str(root), *args], text=True, stderr=sys.stderr
    ).strip()


def verify_upstream(root):
    """Return the initialized, clean submodule only after checking its identity."""
    manifest = json.loads((root / "data/upstream.json").read_text())
    if not isinstance(manifest, dict):
        raise ValueError("data/upstream.json must contain an object")
    pin = manifest.get("pin")
    version = manifest.get("schema_version")
    if type(version) is not int or version != 2 or not isinstance(pin, str) or not re.fullmatch(r"[0-9a-f]{40}", pin):
        raise ValueError("data/upstream.json requires schema_version 2 and a full lowercase commit pin")
    if Path(git(root, "rev-parse", "--show-toplevel")).resolve() != root.resolve():
        raise ValueError("workspace must be its own Git worktree")

    # Checking the index prevents an unrelated adjacent clone or an empty
    # directory from satisfying the upstream gate.
    entry = git(root, "ls-files", "--stage", "--", "upstream")
    if entry != f"160000 {pin} 0\tupstream":
        raise ValueError("upstream must be a registered gitlink at the full provenance pin")
    git(root, "ls-files", "--error-unmatch", "--", ".gitmodules")
    module_paths = git(root, "config", "--file", ".gitmodules", "--get-regexp", r"^submodule\..*\.path$")
    matches = [line.split(None, 1)[0][:-5] for line in module_paths.splitlines() if line.split(None, 1)[-1] == "upstream"]
    if len(matches) != 1:
        raise ValueError(".gitmodules must declare exactly one upstream path")
    url = git(root, "config", "--file", ".gitmodules", "--get", matches[0] + ".url")
    if url.rstrip("/").removesuffix(".git") != CANONICAL_UPSTREAM:
        raise ValueError("upstream .gitmodules URL must be the canonical Microsoft TypeScript repository")

    upstream = root / "upstream"
    if not (upstream / ".git").exists():
        raise ValueError("upstream submodule is not initialized; initialize it separately")
    if Path(git(upstream, "rev-parse", "--show-toplevel")).resolve() != upstream.resolve():
        raise ValueError("upstream is not its own initialized Git worktree")
    if git(upstream, "rev-parse", "HEAD") != pin:
        raise ValueError("upstream checkout does not match the provenance pin")
    if git(upstream, "status", "--porcelain", "--untracked-files=all", "--ignore-submodules=none"):
        raise ValueError("upstream checkout must have no tracked or untracked changes")
    if not (upstream / "tsc/go.mod").is_file() or not (upstream / "tsc/cmd/tsc/main.go").is_file():
        raise ValueError("upstream pin does not contain the expected Go compiler module")
    return upstream


def workspace(root):
    subprocess.run(
        ["cargo", "check", "--workspace", "--locked"],
        cwd=root,
        stdout=sys.stderr,
        stderr=sys.stderr,
        check=True,
    )
    return {"build": True}


def oracle(root):
    upstream = verify_upstream(root)
    env = os.environ.copy()
    # The declared run is a host executable using upstream's release cgo policy.
    for variable in ("GOOS", "GOARCH", "GOARM", "GOARM64", "GOAMD64", "GO386", "GOMIPS", "GOMIPS64", "GOPPC64", "GORISCV64"):
        env.pop(variable, None)
    env.update(CGO_ENABLED="0", GOWORK="off", GOFLAGS="")
    with tempfile.TemporaryDirectory(prefix="ts-rust-oracle-") as temp:
        executable = Path(temp) / ("tsc.exe" if os.name == "nt" else "tsc")
        subprocess.run(
            ["go", "build", "-mod=readonly", "-o", str(executable), "./cmd/tsc"],
            cwd=upstream / "tsc",
            env=env,
            stdout=sys.stderr,
            stderr=sys.stderr,
            check=True,
        )
        result = subprocess.run(
            [str(executable), "--version"],
            cwd=upstream / "tsc",
            env=env,
            stdout=subprocess.PIPE,
            stderr=sys.stderr,
            text=True,
            check=True,
        )
        print(result.stdout, file=sys.stderr, end="")
        if not re.fullmatch(r"Version [0-9]+\.[0-9]+\.[0-9]+[^\r\n]*\r?\n?", result.stdout):
            raise ValueError("oracle --version did not emit a TypeScript version")
    # A successful build must not have changed the checkout or raced a pin edit.
    verify_upstream(root)
    return {"build": True, "smoke": True}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("check", choices=("workspace", "oracle"))
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    try:
        metrics = {"workspace": workspace, "oracle": oracle}[args.check](root)
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        print(f"{args.check} check failed: {error}", file=sys.stderr)
        return 1
    print(json.dumps({"metrics": metrics}, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
