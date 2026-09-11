#!/usr/bin/env python3
"""Owner-approved E2 acceptance partition; source/parser membership is unchanged."""
import argparse
from collections import Counter
from pathlib import Path
import sys

from s04 import same_json_value, verified_upstream
from s04_common import strict_json_loads
from s08_oracle import ROOT, canonical, digest, run_overlay

AMENDMENT = "S07-3-2026-09-11-native-baseline-authority"
POLICY = "data/s07/e2-acceptance.json"
OBSERVATIONS = "data/s07/e2-policy-observations.json"
BRIDGE = "tools/s08/oracle/acceptance_policy_test.go"
SOURCES = ("scripts/s07_acceptance.py", BRIDGE, "scripts/s08_oracle.py", "scripts/s04.py",
           "scripts/s04_common.py", "scripts/s04_runtime.py", "data/s04/toolchains.toml")
NATIVE_SOURCES = ("tsc/internal/testrunner/compiler_runner.go", "tsc/internal/testutil/harnessutil/harnessutil.go")


def source_rows(subset):
    rows = [(case, variant) for case in subset["cases"] for variant in case["variants"] if variant["disposition"] == "eligible"]
    ids = [v["id"] for _,v in rows]
    if not ids or len(ids) != len(set(ids)):
        raise ValueError("empty or duplicate source inventory")
    return rows


def requests_for(rows):
    return [{"id":v["id"], "path":c["source"]["path"], "options":v["options"]} for c,v in rows]


def policy_observation(report):
    """Freeze portable policy facts; run_overlay retains full host provenance."""
    return {key:report[key] for key in ("request_sha256", "rows")}


def classify(rows, observation):
    requests = requests_for(rows)
    if observation["request_sha256"] != digest(canonical(requests)+b"\n"):
        raise ValueError("native eligibility observed different inputs")
    observed = observation["rows"]
    if [r["id"] for r in observed] != [r["id"] for r in requests]:
        raise ValueError("missing, duplicate, extra or reordered native eligibility")
    result = []
    for (_,v),r in zip(rows, observed, strict=True):
        if set(r) != {"id","filename_skip","option_guard"} or type(r["filename_skip"]) is not bool or r["option_guard"] not in ("allowed","skipped"):
            raise ValueError("unclassified native eligibility outcome")
        if v["option_outcome"] not in ("accepted","rejected") or bool(v["option_diagnostics"]) != (v["option_outcome"] == "rejected"):
            raise ValueError("rejected-option identity is inconsistent")
        reasons = []
        if v["option_outcome"] == "rejected": reasons.append("rejected_options")
        if r["option_guard"] == "skipped": reasons.append("native_option_guard")
        if r["filename_skip"]: reasons.append("native_filename_skip")
        result.append({"id":v["id"], "tier":"informational" if reasons else "acceptance", "reasons":reasons})
    return result


def counts(variants):
    return {"source_variants":len(variants), **dict(Counter(r["tier"] for r in variants)),
            "reason_memberships":dict(sorted(Counter(reason for r in variants for reason in r["reasons"]).items()))}


def fingerprints():
    upstream = verified_upstream()
    return ({name:digest((ROOT/name).read_bytes()) for name in SOURCES},
            {name:digest((upstream/name).read_bytes()) for name in NATIVE_SOURCES})


def load_partition(subset):
    policy = strict_json_loads((ROOT/POLICY).read_bytes())
    observation_raw = (ROOT/OBSERVATIONS).read_bytes()
    expected = classify(source_rows(subset), strict_json_loads(observation_raw))
    sources,native = fingerprints()
    if (policy["pin"] != subset["pin"] or policy["amendment"] != AMENDMENT
            or policy["status"] != "owner-approved" or policy["sources"] != sources
            or policy["native_sources"] != native or policy["observations_sha256"] != digest(observation_raw)
            or not same_json_value(policy["variants"], expected) or not same_json_value(policy["counts"], counts(expected))):
        raise ValueError("E2 acceptance partition is stale, changed or not owner-approved")
    return policy


def select_acceptance(results, variants):
    """Isolate the complete ordered acceptance inventory without gating on extras.

    Informational rows may be absent or report any outcome, including unavailable;
    they cannot replace a missing acceptance row or enter its comparison count.
    This selects observations only: it does not compute a synthetic parity metric.
    """
    ids = [v["id"] for v in variants]
    if len(ids) != len(set(ids)) or any(v["tier"] not in ("acceptance","informational") for v in variants):
        raise ValueError("invalid acceptance partition")
    required = [v["id"] for v in variants if v["tier"] == "acceptance"]
    if not required: raise ValueError("empty acceptance denominator")
    actual = [r["id"] for r in results]
    present = set(actual)
    if len(actual) != len(present) or actual != [i for i in ids if i in present]:
        raise ValueError("extra, duplicate or reordered source outcomes")
    required_ids = set(required)
    selected = [r for r in results if r["id"] in required_ids]
    if [r["id"] for r in selected] != required:
        raise ValueError("missing acceptance outcome")
    return selected


def prepare(directory):
    subset = strict_json_loads((ROOT/"data/s07/subset.json").read_bytes())
    rows = source_rows(subset)
    sources,native = fingerprints()
    observation = policy_observation(run_overlay(Path(directory)/"go-policy", "testrunner", (ROOT/BRIDGE).read_text(), requests_for(rows), "TestS08AcceptancePolicy"))
    variants = classify(rows, observation)
    if (sources,native) != fingerprints(): raise ValueError("eligibility sources changed during capture")
    raw = canonical(observation)+b"\n"
    policy = {"version":1, "pin":subset["pin"], "amendment":AMENDMENT, "status":"owner-approved",
              "decision":"Repository owner, 2026-09-11: retain native skips and rejected options as informational Go-versus-Rust differentials without an E2 gate",
              "review":"data/s07/subset-review.json", "sources":sources, "native_sources":native,
              "predicates":{"native_option_guard":"Execute pinned harnessutil.SkipUnsupportedCompilerOptions on frozen effective options; a skip is informational; fatal/unknown outcomes require review",
                            "native_filename_skip":"Pinned testrunner.skippedTests contains the physical source basename",
                            "rejected_options":"Frozen S07 option_outcome is rejected with nonempty original Go option_diagnostics"},
              "scope":"E2 checker acceptance only; S06/S07 source, parser, binder and loader membership and E7/E8 remain unchanged",
              "observations_sha256":digest(raw), "counts":counts(variants), "variants":variants}
    Path(directory,"e2-acceptance.candidate.json").write_bytes(canonical(policy)+b"\n")
    Path(directory,"e2-policy-observations.candidate.json").write_bytes(raw)
    return policy,raw


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation",choices=("prepare","freeze","check"))
    parser.add_argument("--output",type=Path,required=True)
    args=parser.parse_args()
    policy,observation=prepare(args.output)
    for name,raw in ((POLICY,canonical(policy)+b"\n"),(OBSERVATIONS,observation)):
        if args.operation=="freeze": (ROOT/name).write_bytes(raw)
        elif args.operation=="check" and (ROOT/name).read_bytes()!=raw: raise ValueError("E2 native policy drift: "+name)
    print(policy["counts"])


if __name__ == "__main__":
    try: main()
    except (OSError,ValueError,RuntimeError,KeyError,TypeError) as error:
        print(f"E2 eligibility failed: {error}",file=sys.stderr)
        raise SystemExit(1) from error
