#!/usr/bin/env python3
"""Freeze or verify direct AST helper observations from a fresh canonical Go export."""
import argparse
import hashlib
import json
import shutil

from s04_common import command
from s06_build import ROOT, oracle_export


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    adapter = ROOT / "tools/s07/ast-helpers/export_test.go"
    with oracle_export() as (checkout, env, pin):
        shutil.copyfile(adapter, checkout / "tsc/internal/ast/s07_helpers_test.go")
        observed = checkout / "ast-helpers.tsv"
        env["S07_AST_HELPERS_OUTPUT"] = str(observed)
        repo_path = f"-gcflags=github.com/microsoft/TypeScript/tsc/internal/repo=-trimpath={checkout}/s07-unmatched-prefix"
        command(["go", "test", "-trimpath", "-mod=readonly", repo_path, "./internal/ast", "-run",
                 "^TestS07ASTHelpers$", "-count=1"], cwd=checkout / "tsc", env=env)
        data = observed.read_bytes()
        manifest = {"upstream_pin": pin,
                    "source_sha256": hashlib.sha256((checkout / "tsc/internal/ast/utilities.go").read_bytes()).hexdigest(),
                    "adapter_sha256": hashlib.sha256(adapter.read_bytes()).hexdigest(),
                    "observation_sha256": hashlib.sha256(data).hexdigest(),
                    "rows": len(data.splitlines()),
                    "panic_payloads": json.loads(observed.with_name(observed.name + ".panics.json").read_bytes())}
    for name, raw in {"ast-helper-observations.tsv": data,
                      "ast-helper-observations.json": (json.dumps(manifest, indent=2, sort_keys=True)+"\n").encode()}.items():
        target = ROOT / "data/s07" / name
        if args.check:
            if target.read_bytes() != raw:
                raise ValueError("pinned AST helper observations changed: " + name)
        else:
            target.write_bytes(raw)
    print(f"{manifest['rows']} direct Go AST helper rows")


if __name__ == "__main__":
    main()
