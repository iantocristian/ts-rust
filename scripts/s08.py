#!/usr/bin/env python3
"""S08 P0 request contracts. No experiment producer is registered by this tool."""

import argparse
import json
import sys
from pathlib import Path

from s08_manifest import prepare
from s08_oracle import ROOT, canonical


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=("prepare", "check", "freeze"))
    parser.add_argument("--output", type=Path, required=True, help="new observation directory")
    args = parser.parse_args()
    manifest = prepare(args.output)
    content = canonical(manifest) + b"\n"
    target = ROOT / "data/s08/baseline-requests.json"
    if args.operation == "check" and (not target.exists() or target.read_bytes() != content):
        raise ValueError("S08 request inventory drift; review the candidate before freezing")
    if args.operation == "freeze":
        target.write_bytes(content)
    print(json.dumps({"operation": args.operation, "counts": manifest["counts"],
                      "scope": manifest["scope"]}, sort_keys=True))


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, RuntimeError, KeyError, TypeError) as error:
        print(f"S08 preparation failed: {error}", file=sys.stderr)
        raise SystemExit(1) from error
