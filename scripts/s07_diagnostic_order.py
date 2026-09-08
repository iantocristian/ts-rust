#!/usr/bin/env python3
"""Freeze or verify direct diagnostic-order helper observations from a fresh canonical Go export."""
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
    adapter = ROOT / "tools/s07/diagnostic-order/export_test.go"
    with oracle_export() as (checkout, env, pin):
        shutil.copyfile(adapter, checkout / "tsc/internal/ast/s07_diagnostic_order_test.go")
        observed = checkout / "diagnostic-order.tsv"
        env["S07_DIAGNOSTIC_ORDER_OUTPUT"] = str(observed)
        repo_path = f"-gcflags=github.com/microsoft/TypeScript/tsc/internal/repo=-trimpath={checkout}/s07-unmatched-prefix"
        command(["go", "test", "-trimpath", "-mod=readonly", repo_path, "./internal/ast", "-run",
                 "^TestS07DiagnosticOrder$", "-count=1"], cwd=checkout / "tsc", env=env)
        data = observed.read_bytes()
        manifest = {"upstream_pin": pin,
                    "source_sha256": hashlib.sha256((checkout / "tsc/internal/ast/diagnostic.go").read_bytes()).hexdigest(),
                    "adapter_sha256": hashlib.sha256(adapter.read_bytes()).hexdigest(),
                    "observation_sha256": hashlib.sha256(data).hexdigest(),
                    "rows": len(data.splitlines())}
    for name, raw in {"diagnostic-order-observations.tsv": data,
                      "diagnostic-order-observations.json": (json.dumps(manifest, indent=2, sort_keys=True)+"\n").encode()}.items():
        target = ROOT / "data/s07" / name
        if args.check:
            if target.read_bytes() != raw:
                raise ValueError("pinned diagnostic-order helper observations changed: " + name)
        else:
            target.write_bytes(raw)
    print(f"{manifest['rows']} direct Go diagnostic-order helper rows")


if __name__ == "__main__":
    main()
