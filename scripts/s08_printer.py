#!/usr/bin/env python3
"""Print synthetic type-display trees with the pinned Go printer and freeze the result.

The cases in data/s08/printer-cases.json are hand-written trees over the node
kinds the checker's node builder produces. The pinned printer's bytes for each
tree, plus the printer constants the Rust port transcribes, become
data/s08/printer-observations.json, which the Rust printer tests replay. This is
a Go observation, not an E2 producer and not a Rust parity result.
"""

import argparse
import json
from pathlib import Path
import sys

from s04_common import strict_json_loads
from s08_oracle import ROOT, canonical, digest, run_overlay

CASES = ROOT / "data/s08/printer-cases.json"
OBSERVATIONS = ROOT / "data/s08/printer-observations.json"
SOURCE = ROOT / "tools/s08/oracle/printer_test.go"


def capture(directory):
    request = strict_json_loads(CASES.read_bytes())
    pin = strict_json_loads((ROOT / "data/upstream.json").read_bytes())["pin"]
    if request["pin"] != pin:
        raise ValueError("printer cases target a different pin")
    names = [case["name"] for case in request["cases"]]
    if len(set(names)) != len(names):
        raise ValueError("duplicate printer case name")
    report = run_overlay(Path(directory) / "go-printer", "printer", SOURCE.read_text(), {"cases": request["cases"]},
                         "TestS08PrintTypeNodes")
    rows = report["rows"]
    if [row["name"] for row in rows] != names:
        raise ValueError("missing, extra, duplicate or reordered printer observation")
    for row in rows:
        if ("text_hex" in row) == ("panic" in row):
            raise ValueError(f"printer observation {row['name']} lacks exactly one outcome")
        if "text_hex" in row:
            bytes.fromhex(row["text_hex"])
    observation = {
        "version": 1, "pin": pin, "go": report["go"], "goos": report["goos"], "goarch": report["goarch"],
        "cases_sha256": digest(canonical(request)), "source_sha256": digest(SOURCE.read_bytes()),
        "scope": report["scope"],
        "derivation": "python3 scripts/s08_printer.py --output <new dir> [--freeze]; go test -overlay adds "
                      "tools/s08/oracle/printer_test.go to internal/printer without modifying the checkout",
        "rows": rows, "constants": report["constants"],
    }
    (Path(directory) / "printer-observations.candidate.json").write_bytes(canonical(observation) + b"\n")
    return observation


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True, help="new observation directory")
    parser.add_argument("--freeze", action="store_true", help="replace data/s08/printer-observations.json")
    args = parser.parse_args()
    observation = capture(args.output)
    content = canonical(observation) + b"\n"
    if args.freeze:
        OBSERVATIONS.write_bytes(content)
    elif OBSERVATIONS.exists() and OBSERVATIONS.read_bytes() != content:
        print("printer observations differ from the frozen fixture; review the candidate", file=sys.stderr)
        raise SystemExit(2)
    print(json.dumps({"cases": len(observation["rows"]), "panics": sum("panic" in row for row in observation["rows"]),
                      "constants": sum(len(values) for values in observation["constants"].values()),
                      "frozen": args.freeze}, sort_keys=True))


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, RuntimeError, KeyError, TypeError) as error:
        print(f"S08 printer observation failed: {error}", file=sys.stderr)
        raise SystemExit(1) from error
