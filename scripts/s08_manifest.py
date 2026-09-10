"""Freeze S08 baseline phase obligations without deriving success from absence."""

from collections import Counter
from pathlib import Path

from s04_common import strict_json_loads
from s08_oracle import ROOT, canonical, digest, run_overlay

INPUTS = ("data/s07/subset.json", "data/s07/subset-rule.json", "data/s07/subset-review.json",
          "data/s07/checker-obligations.json", "data/upstream.json")
PHASE_SOURCE = "tools/s08/oracle/phase_policy_test.go"
SOURCES = (PHASE_SOURCE, "scripts/s08.py", "scripts/s08_manifest.py", "scripts/s08_oracle.py",
           "scripts/s04.py", "scripts/s04_common.py", "scripts/s04_runtime.py",
           "scripts/tracking-bootstrap.py", "data/s04/toolchains.toml", ".gitmodules")


def optional_bool(options, name):
    value = options.get(name)
    if value is not None and type(value) is not bool:
        raise ValueError(f"non-boolean option {name}: {value!r}")
    return value


def eligible(subset):
    rows = []
    seen = set()
    for case in subset["cases"]:
        for variant in case["variants"]:
            if variant["disposition"] != "eligible":
                continue
            if variant["id"] in seen:
                raise ValueError("duplicate eligible variant")
            seen.add(variant["id"])
            rows.append((case, variant))
    if not rows:
        raise ValueError("empty eligible inventory")
    return rows


def phase_requests(rows):
    return [{"id": variant["id"],
             "declaration": optional_bool(variant["options"], "declaration"),
             "composite": optional_bool(variant["options"], "composite")}
            for _, variant in rows]


def validate_policy(requests, report):
    rows = report["rows"]
    if [row["id"] for row in rows] != [row["id"] for row in requests]:
        raise ValueError("missing, extra, duplicate or reordered phase observation")
    for row in rows:
        if set(row) != {"id", "declaration_requested"} or type(row["declaration_requested"]) is not bool:
            raise ValueError("malformed phase observation")
    return rows


def make_manifest(subset, rows, policy, inputs):
    requests = []
    for (case, variant), phase in zip(rows, policy, strict=True):
        if phase["id"] != variant["id"]:
            raise ValueError("phase joined to the wrong variant")
        harness = variant["harness_options"]
        for key in ("NoTypesAndSymbols", "CaptureSuggestions"):
            if type(harness[key]) is not bool:
                raise ValueError(f"invalid harness flag {key}")
        phases = ["config", "program", "syntactic", "semantic", "global"]
        if phase["declaration_requested"]:
            phases.append("declaration")
        if harness["CaptureSuggestions"]:
            phases.append("suggestion")
        references = [row for row in variant["baselines"] if row["kind"] in (".types", ".symbols", ".errors.txt")]
        requests.append({
            "id": variant["id"], "case": case["id"], "configuration": variant["configuration"],
            "configured_name": variant["configured_name"],
            "variant_sha256": digest(canonical(variant)), "source_sha256": case["source"]["raw_sha256"],
            "loading_request_sha256": variant["loading_request_sha256"],
            "options_sha256": digest(canonical(variant["options"])),
            "harness_options_sha256": digest(canonical(harness)),
            "dependency_closure_sha256": digest(canonical([
                subset["file_observations"][index] for index in variant["dependency_closure"]])),
            "diagnostic_phases": phases, "type_baseline_requested": not harness["NoTypesAndSymbols"],
            "reference_baselines": references,
        })
    counts = Counter(phase for row in requests for phase in row["diagnostic_phases"])
    return {
        "version": 1, "pin": subset["pin"], "inputs": inputs,
        "scope": "Required phases derived from frozen S07 variants and pinned Go option policy; no semantic execution or parity claim",
        "declaration_decision": "execute required declaration diagnostics inside S08",
        "result_contract": {
            "phase_outcomes": ["executed", "not_requested", "failed", "not_implemented"],
            "baseline_outcomes": ["content", "no_content", "disabled", "failed", "not_implemented"],
            "not_implemented_passes": False,
            "reference_absence_proves_no_content": False,
            "pre_post_emit_comparison": "complete sorted diagnostic payloads; equal counts alone are insufficient",
        },
        "counts": {"variants": len(requests), "diagnostic_phases": dict(sorted(counts.items())),
                   "type_baseline_requested": sum(row["type_baseline_requested"] for row in requests)},
        "requests": requests,
    }


def prepare(directory):
    directory = Path(directory)
    inputs = {name: digest((ROOT / name).read_bytes()) for name in (*INPUTS, *SOURCES)}
    subset = strict_json_loads((ROOT / INPUTS[0]).read_bytes())
    rule = strict_json_loads((ROOT / INPUTS[1]).read_bytes())
    pin = strict_json_loads((ROOT / "data/upstream.json").read_bytes())["pin"]
    if subset["pin"] != pin or rule["pin"] != pin or rule["state"] != "frozen":
        raise ValueError("unfrozen or wrong-pin subset")
    rows = eligible(subset)
    if len(rows) != rule["counts"]["eligible_variants"]:
        raise ValueError("eligible count differs from the frozen rule")
    requests = phase_requests(rows)
    report = run_overlay(directory / "go-phases", "core", (ROOT / PHASE_SOURCE).read_text(), requests, "TestS08PhasePolicy")
    policy = validate_policy(requests, report)
    manifest = make_manifest(subset, rows, policy, inputs)
    if inputs != {name: digest((ROOT / name).read_bytes()) for name in inputs}:
        raise ValueError("manifest inputs changed during observation")
    (directory / "baseline-requests.candidate.json").write_bytes(canonical(manifest) + b"\n")
    return manifest
