#!/usr/bin/env python3
"""Regenerate independent accessor observations from the clean canonical Go pin."""
import argparse
import shutil

from s04_common import command
from s06_build import ROOT, oracle_export


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    with oracle_export() as (checkout, env, pin):
        shutil.copyfile(ROOT / "scripts/s06_oracle/accessors_test.go",
                        checkout / "tsc/internal/ast/s06_accessors_test.go")
        observed = checkout / "accessors.tsv"
        env["S06_ACCESSORS_OUTPUT"] = str(observed)
        # Existing ast tests initialize repo.RootPath before test selection.
        # Preserve only that package's source path, as in corpus preprocessing.
        repo_path = f"-gcflags=github.com/microsoft/TypeScript/tsc/internal/repo=-trimpath={checkout}/s06-unmatched-prefix"
        command(["go", "test", "-trimpath", "-mod=readonly", repo_path, "./internal/ast", "-run",
                 "^TestS06Accessors$", "-count=1"], cwd=checkout / "tsc", env=env)
        data = b"# pin " + pin.encode() + b"\n" + observed.read_bytes()
    target = ROOT / "data/s06/accessor-observations.tsv"
    if args.check:
        if target.read_bytes() != data:
            raise ValueError("pinned AST front accessor observations changed")
    else:
        target.write_bytes(data)
    print(f"{len(data.splitlines())-1} observations: {target}")


if __name__ == "__main__":
    main()
