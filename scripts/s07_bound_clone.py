#!/usr/bin/env python3
"""Freeze or verify bound-source factory copies from the unchanged Go pin."""
import argparse
import hashlib
import json
import shutil

from s04_common import command, strict_json_loads
from s06_build import ROOT, oracle_export


FLAG_FIELDS = {
    "parsedFlags", "boundFlags", "cloneFlags", "updateFlags",
    "parsedFunctionFlags", "boundFunctionFlags", "cloneFunctionFlags",
}
BOOL_FIELDS = {
    "commonJS", "cloneCommonJS", "updateCommonJS", "cloneBound", "updateBound",
    "sameCloneStatements", "sameUpdateChild", "sameCloneEOF", "sameUpdateEOF",
    "sameUnchangedUpdate",
}


def validate_observation(observed):
    if type(observed) is not dict or set(observed) != FLAG_FIELDS | BOOL_FIELDS:
        raise ValueError("bound-source witness observation fields changed")
    if any(type(observed[key]) is not int or not 0 <= observed[key] <= 0xFFFFFFFF for key in FLAG_FIELDS):
        raise ValueError("bound-source witness flags must be uint32")
    if any(type(observed[key]) is not bool for key in BOOL_FIELDS):
        raise ValueError("bound-source witness identity/state observations must be bool")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true", help="verify frozen observations (default)")
    mode.add_argument("--write", action="store_true", help="refresh frozen source observations")
    args = parser.parse_args()
    adapter = ROOT / "tools/s07/ownership/bound-clone.go"
    with oracle_export() as (checkout, env, pin):
        package = checkout / "tsc/internal/s07boundclone"
        package.mkdir()
        shutil.copyfile(adapter, package / "main.go")
        data = command(["go", "run", "-trimpath", "-mod=readonly", "./internal/s07boundclone"],
                       cwd=checkout / "tsc", env=env)
        observed = strict_json_loads(data)
        validate_observation(observed)
        manifest = {
            "upstream_pin": pin,
            "adapter_sha256": hashlib.sha256(adapter.read_bytes()).hexdigest(),
            "observation_sha256": hashlib.sha256(data).hexdigest(),
            "source_sha256": {
                name: hashlib.sha256((checkout / "tsc" / name).read_bytes()).hexdigest()
                for name in ("internal/ast/ast.go", "internal/ast/ast_generated.go",
                             "internal/binder/binder.go", "internal/parser/parser.go")
            },
            "observations": len(observed),
        }
    for name, raw in {
        "bound-clone.json": data,
        "bound-clone.manifest.json": (json.dumps(manifest, indent=2, sort_keys=True) + "\n").encode(),
    }.items():
        target = ROOT / "data/s07" / name
        if not args.write:
            if target.read_bytes() != raw:
                raise ValueError("pinned bound-source clone observations changed: " + name)
        else:
            target.write_bytes(raw)
    print(f"{len(observed)} direct Go bound-source clone/update observations")


if __name__ == "__main__":
    main()
