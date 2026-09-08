#!/usr/bin/env python3
"""Arithmetic distance to historical Go budgets; never an acceptance producer."""
import argparse
from fractions import Fraction
import gzip
import hashlib
import json
from pathlib import Path
from statistics import median

HERE = Path(__file__).resolve().parent
DOMAINS = ("wall_time_ns", "allocated_bytes", "peak_rss_bytes")
WORKERS = ("1", "8")
INPUTS = {
    "original-go-report": "1c8dc74fb5c3d6d3ef5e6a8f37595c8d9dbacc7ba1f1ed0b7bef21bfb5fad5dd",
    "original-control-manifest": "c8e5dd18abb62a9cdd3c3af47051cc933c7744995a9a7029ac20a53bf8ea6363",
    "a0b-screen-report": "0f70cf1705dc7006695d5622bd00356b3912ab29aedc051f57ef6e1e18fe513f",
    "cp1-screen-report": "66b6547a29aafaddbbc01cc834ee7460a79126dcb0dba133d5e6606b144cfc6b",
}
VARIANTS = {
    "A0-b": ("a0b-screen-report", "c8e5dd18abb62a9cdd3c3af47051cc933c7744995a9a7029ac20a53bf8ea6363",
             "124956f668540814b95e6f677bdeae98bb2cad4f7586d3cb87f869e5c5af118f"),
    "CP1": ("cp1-screen-report", "124956f668540814b95e6f677bdeae98bb2cad4f7586d3cb87f869e5c5af118f",
            "3a57976f667b9857891edbe9f76de5261c43480ac8c62d946b4545d9ee19d931"),
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def strict_json(raw):
    def object_pairs(pairs):
        result = {}
        for key, value in pairs:
            require(key not in result, "duplicate JSON key")
            result[key] = value
        return result
    def constant(value):
        raise ValueError("nonfinite JSON value: " + value)
    return json.loads(raw, object_pairs_hook=object_pairs, parse_constant=constant)


def positive_integer(value):
    require(type(value) is int and value > 0, "expected a positive measured integer")
    return value


def distance(rust, go, domain):
    require(domain in DOMAINS, "unknown measurement domain")
    positive_integer(rust)
    positive_integer(go)
    threshold = Fraction(1) if domain == "wall_time_ns" else Fraction(7, 10)
    budget = go * threshold
    gap = max(Fraction(0), rust - budget)
    return {
        "rust_median": rust,
        "historical_go_median": go,
        "median_ratio_to_historical_go": float(Fraction(rust, go)),
        "required_ratio": float(threshold),
        "historical_budget": float(budget),
        "historical_budget_exact": {"numerator": budget.numerator, "denominator": budget.denominator},
        "excess_to_historical_budget": float(gap),
        "remaining_reduction_fraction": float(gap / rust),
    }


def calculate(original, screens):
    require(type(original["samples"]) is int and original["samples"] == 56
            and original["expected_work"]["files"] == 13094, "historical baseline changed workload/count")
    require(set(original["summaries"]) == set(WORKERS), "historical baseline omitted a worker mode")
    require(set(screens) == set(VARIANTS), "missing/extra checkpoint")
    for mode in original["summaries"].values():
        require(set(mode) == {*DOMAINS, "rust_wrapper_overhead"}, "historical baseline changed measurement domains")
        for domain in DOMAINS:
            row = mode[domain]
            require(type(row["samples_per_runtime"]) is int and row["samples_per_runtime"] == 7,
                    "historical baseline changed repetition count")
            positive_integer(row["go_median"])
    result = {}
    for label, screen in screens.items():
        _, control, candidate = VARIANTS[label]
        require(screen["status"] == "complete" and screen["diagnostic_only"] is True
                and type(screen["samples"]) is int and screen["samples"] == 56,
                "checkpoint is incomplete or has a different sample count")
        require(screen["manifest_sha256"] == {"control": control, "candidate": candidate},
                "checkpoint control/candidate identities changed")
        require(set(screen["modes"]) == set(WORKERS), "checkpoint omitted a worker mode")
        workers = {}
        for worker, mode in screen["modes"].items():
            require(set(mode) == set(DOMAINS), "checkpoint omitted a measurement domain")
            metrics = {}
            for domain, row in mode.items():
                require(type(row["samples_per_variant"]) is int and row["samples_per_variant"] == 7,
                        "checkpoint changed repetition count")
                require(set(row["values"]) == {"control", "candidate"}, "checkpoint changed variant roles")
                for role in ("control", "candidate"):
                    values = row["values"][role]
                    require(type(values) is list and len(values) == 7, "checkpoint omitted measured values")
                    for value in values:
                        positive_integer(value)
                    require(median(values) == row[role + "_median"], "checkpoint median differs from recorded values")
                metrics[domain] = distance(row["candidate_median"], original["summaries"][worker][domain]["go_median"], domain)
            wall = mode["wall_time_ns"]
            workers[worker] = {"historical_distance": metrics,
                "same_screen_rust_control_wall_ns": wall["control_median"],
                "same_screen_wall_median_ratio": wall["ratio"],
                "same_screen_timing_bootstrap_upper_95_ratio": wall["bootstrap"]["upper"],
                "timing_bootstrap_upper_95_against_historical_go": None}
        result[label] = {"source_report": VARIANTS[label][0], "source_manifests": screen["manifest_sha256"],
                         "workers": workers}
    return {"version": 1, "diagnostic_only": True, "source_report_sha256": INPUTS,
        "units": {"wall_time_ns": "nanoseconds", "allocated_bytes": "requested bytes", "peak_rss_bytes": "lifetime peak RSS bytes"},
        "scope": "Arithmetic between retained checkpoint medians and older Go medians; no fresh cross-runtime sample or acceptance claim",
        "fresh_go_acceptance_measured": False, "native_child_executed": False,
        "final_timing_requirements": {"median_ratio_maximum": 1.0, "bootstrap_upper_95_ratio_maximum": 1.0,
                                      "both_runtimes_relative_mad_maximum": 0.05, "required_worker_counts": [1, 8]},
        "final_memory_requirements": {"maximum_worker_median_ratio": 0.7, "required_domains": ["allocated_bytes", "peak_rss_bytes"]},
        "checkpoints": result,
        "limitations": ["Older Go denominators contextualize distance; fresh Go captures determine final acceptance.",
            "Same-screen bootstrap bounds compare Rust candidates with Rust controls, not with Go.",
            "Separate Rust screen percentages are not compounded into a measured cumulative improvement.",
            "No current phase profile attributes the remaining wall gap to binding, parsing or orchestration.",
            "Live requested bytes, allocation traffic and lifetime RSS are distinct domains; live storage is not a gate denominator."]}


def replay(directory=HERE):
    inputs = {}
    for name, expected in INPUTS.items():
        raw = gzip.decompress((directory / "inputs" / (name + ".json.gz")).read_bytes())
        require(hashlib.sha256(raw).hexdigest() == expected, "recorded input changed: " + name)
        inputs[name] = strict_json(raw)
    recorded = inputs["original-control-manifest"]["inventory"]["evidence/baseline-report.json"]
    require(recorded["sha256"] == INPUTS["original-go-report"], "historical Go report is not the immutable control's baseline")
    return calculate(inputs["original-go-report"], {label: inputs[names[0]] for label, names in VARIANTS.items()})


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true", help="write the derived result after validating pinned inputs")
    args = parser.parse_args()
    result = replay()
    path = HERE / "result.json"
    if args.write:
        path.write_text(json.dumps(result, indent=2, sort_keys=True, allow_nan=False) + "\n")
    else:
        require(strict_json(path.read_bytes()) == result, "recorded distance result differs from pinned-input arithmetic")
    print(json.dumps({"verified": True, "diagnostic_only": True, "checkpoints": list(result["checkpoints"]),
                      "native_child_executed": False}, sort_keys=True))
