#!/usr/bin/env python3
"""Reproduce the scaffold's pinned Go constants and fixed record sizes."""

import argparse
import json
from pathlib import Path
import re

from s04_common import strict_json_loads
from s08_oracle import ROOT, canonical, digest, run_overlay


def identifier(value):
    if not isinstance(value, str) or not re.fullmatch(r"[A-Za-z_][A-Za-z_0-9]*", value):
        raise ValueError(f"invalid Go identifier: {value!r}")
    return value


def source(request, package):
    flags = []
    for group, names in sorted(request["flags"].items()):
        owner, family = group.split(".")
        if owner not in ("checker", "nodebuilder"):
            raise ValueError("unknown flag package")
        identifier(family)
        if owner == package:
            values = ",".join(f'{json.dumps(name)}:int64({identifier(name)})' for name in names)
            flags.append(f'{json.dumps(group)}:map[string]int64{{{values}}}')
    constants = ",".join(f'{json.dumps(name)}:int64({identifier(name)})' for name in request["consts"]) if package == "checker" else ""
    sizes = []
    if package == "checker":
        for name in request["sizes"]:
            owner, record = name.split(".")
            if owner not in ("ast", "checker"):
                raise ValueError("unknown size package")
            target = ("ast." if owner == "ast" else "") + identifier(record)
            sizes.append(f'{json.dumps(name)}:unsafe.Sizeof({target}{{}})')
    imports = ('"unsafe"; "github.com/microsoft/TypeScript/tsc/internal/ast";' if sizes else "")
    return f'''package {package}
import ("crypto/sha256"; "encoding/hex"; "encoding/json"; "os"; "runtime"; "testing"; {imports})
func TestS08FlagObservations(t *testing.T) {{
 raw, err := os.ReadFile(os.Getenv("S08_REQUESTS")); if err != nil {{t.Fatal(err)}}
 sum := sha256.Sum256(raw)
 report := map[string]any{{
  "request_sha256":hex.EncodeToString(sum[:]), "go":runtime.Version(), "goos":runtime.GOOS, "goarch":runtime.GOARCH,
  "flags":map[string]any{{{','.join(flags)}}}, "consts":map[string]int64{{{constants}}},
  "sizes":map[string]uintptr{{{','.join(sizes)}}},
 }}
 output, err := json.Marshal(report); if err != nil {{t.Fatal(err)}}
 if err := os.WriteFile(os.Getenv("S08_OUTPUT"),output,0600); err != nil {{t.Fatal(err)}}
}}
'''


def capture(directory):
    request = strict_json_loads((ROOT / "data/s08/flag-requests.json").read_bytes())
    expected = strict_json_loads((ROOT / "data/s08/checker-flag-observations.json").read_bytes())
    pin = strict_json_loads((ROOT / "data/upstream.json").read_bytes())["pin"]
    if request["pin"] != pin or expected["pin"] != pin:
        raise ValueError("flag requests or observations target a different pin")
    observed = {name: {} for name in ("flags", "consts", "sizes")}
    reports = []
    for package in ("checker", "nodebuilder"):
        report = run_overlay(Path(directory) / package, package, source(request, package), request, "TestS08FlagObservations")
        reports.append(report)
        for name in observed:
            if observed[name].keys() & report[name].keys():
                raise ValueError("duplicate observation across packages")
            observed[name].update(report[name])
    for field in ("go", "goos", "goarch"):
        if len({report[field] for report in reports}) != 1:
            raise ValueError("different Go environments within one observation")
    for key in observed:
        if observed[key] != expected[key]:
            raise ValueError(f"scaffold Go observation drift in {key}; raw outputs retained")
    summary = {"pin": pin, "request_sha256": digest(canonical(request)),
               "constants": sum(len(values) for values in observed["flags"].values()) + len(observed["consts"]),
               "record_sizes": len(observed["sizes"]), "matches_scaffold": True,
               "scope": "fixed Go constants/record sizes, not a live type-footprint census",
               **{key: reports[0][key] for key in ("go", "goos", "goarch")}}
    (Path(directory) / "report.json").write_bytes(canonical(summary) + b"\n")
    return summary


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    print(json.dumps(capture(parser.parse_args().output), sort_keys=True))
