#!/usr/bin/env python3
"""Freeze or verify direct scanner helper observations from a fresh canonical Go export."""
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
    adapter = ROOT / "tools/s07/scanner-helpers/export_test.go"
    with oracle_export() as (checkout, env, pin):
        shutil.copyfile(adapter, checkout / "tsc/internal/scanner/s07_scanner_helpers_test.go")
        observed = checkout / "scanner-helpers.tsv"
        env["S07_SCANNER_HELPERS_OUTPUT"] = str(observed)
        repo_path = f"-gcflags=github.com/microsoft/TypeScript/tsc/internal/repo=-trimpath={checkout}/s07-unmatched-prefix"
        command(["go", "test", "-trimpath", "-mod=readonly", repo_path, "./internal/scanner", "-run",
                 "^TestS07ScannerHelpers$", "-count=1"], cwd=checkout / "tsc", env=env)
        data = observed.read_bytes()
        manifest = {"upstream_pin": pin,
                    "source_sha256": hashlib.sha256((checkout / "tsc/internal/scanner/utilities.go").read_bytes()).hexdigest(),
                    "scanner_source_sha256": hashlib.sha256((checkout / "tsc/internal/scanner/scanner.go").read_bytes()).hexdigest(),
                    "adapter_sha256": hashlib.sha256(adapter.read_bytes()).hexdigest(),
                    "observation_sha256": hashlib.sha256(data).hexdigest(),
                    "rows": len(data.splitlines()),
                    "panic_payloads": json.loads(observed.with_name(observed.name + ".panics.json").read_bytes())}
    for name, raw in {"scanner-helper-observations.tsv": data,
                      "scanner-helper-observations.json": (json.dumps(manifest, indent=2, sort_keys=True)+"\n").encode()}.items():
        target = ROOT / "data/s07" / name
        if args.check:
            if target.read_bytes() != raw:
                raise ValueError("pinned scanner helper observations changed: " + name)
        else:
            target.write_bytes(raw)
    print(f"{manifest['rows']} direct Go scanner helper rows")


if __name__ == "__main__":
    main()
