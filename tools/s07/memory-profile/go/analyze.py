#!/usr/bin/env python3
"""Export pinned Go memory profiles and attribute sampled bytes to actual stacks."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "scripts"))
from s04_common import strict_json_loads
from s07_benchmark import go_native_environment

TYPES = ("alloc_objects", "alloc_space", "inuse_objects", "inuse_space")
INTERNAL = "github.com/microsoft/TypeScript/tsc/internal/"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def parse_raw(text):
    header, rest = text.split("Samples:\n", 1)
    body, rest = rest.split("\nLocations\n", 1)
    locations_text, _ = rest.split("\nMappings\n", 1)
    match = re.search(r"^Period: (\d+)$", header, re.M)
    if not match or int(match[1]) <= 0 or "PeriodType: space bytes\n" not in header:
        raise ValueError("invalid memory sample period")
    period = int(match[1])
    lines = body.splitlines()
    if lines.pop(0) != "alloc_objects/count alloc_space/bytes inuse_objects/count inuse_space/bytes":
        raise ValueError("unexpected memory sample types")
    samples = []
    for line in lines:
        if not line.strip():
            continue
        match = re.fullmatch(r"\s*(-?\d+)\s+(-?\d+)\s+(-?\d+)\s+(-?\d+):\s*([0-9 ]+)\s*", line)
        if match:
            samples.append({"values": tuple(int(match[i]) for i in range(1, 5)),
                            "locations": [int(value) for value in match[5].split()]})
        else:
            match = re.fullmatch(r"\s*bytes:\[(\d+)\]", line)
            if not match or not samples or "allocation_size" in samples[-1]:
                raise ValueError("unexpected/duplicate memory sample label: " + line)
            samples[-1]["allocation_size"] = int(match[1])
    if not samples:
        raise ValueError("empty memory profile")
    locations, current = {}, None
    for line in locations_text.splitlines():
        match = re.fullmatch(r"\s*(\d+): 0x[0-9a-f]+ M=\d+ (.+)", line)
        if match:
            current = int(match[1])
            if current in locations:
                raise ValueError("duplicate location")
            locations[current] = []
            frame = match[2]
        elif current is not None and line.startswith("             "):
            frame = line.strip()
        else:
            raise ValueError("unrecognized raw location: " + line)
        match = re.fullmatch(r"(.+) (\S+):(-?\d+):(-?\d+)(?: s=-?\d+)?", frame)
        if not match:
            raise ValueError("unrecognized source frame: " + frame)
        locations[current].append((match[1], match[2], int(match[3])))
    for sample in samples:
        if not sample["locations"] or any(index not in locations for index in sample["locations"]):
            raise ValueError("sample references missing location")
        sample["frames"] = [frame for index in sample.pop("locations") for frame in locations[index]]
    return {"period_bytes": period, "raw_header": header, "sample_records": len(samples)}, samples


def domain(frames):
    names = [frame[0] for frame in frames]
    if any(name in {"main.profiles", "main.snapshot", "main.checkpoint"} or name.startswith("runtime/pprof.") for name in names):
        return "diagnostic_profile_checkpoint"
    if any("memoryCensus" in name or name in {"main.census", INTERNAL + "ast.MemoryProfileCensus"} for name in names):
        return "diagnostic_census"
    if any(name.startswith(INTERNAL + "binder.") for name in names):
        return "compiler_binder_ancestor"
    if any(name.startswith(INTERNAL + "parser.") for name in names):
        return "compiler_parser_ancestor"
    if any(name in {"main.loadedDigest", "os.readFileContents"} for name in names):
        return "preload_digest"
    if any(name.startswith(INTERNAL) and not name.startswith(INTERNAL + "s07memoryprofile.") for name in names):
        return "compiler_other_ancestor"
    return "runtime_driver_other_or_truncated"


def factory(frames):
    # This grouping is an actual constructor ancestor, not inferred from the
    # long/hashed generic shape printed as the allocation leaf.
    prefix = INTERNAL + "ast.(*NodeFactory).New"
    for name, _, _ in frames:
        if name.startswith(prefix):
            return name
    for name, _, _ in frames:
        if name in {INTERNAL + "binder.(*Binder).newSymbol", INTERNAL + "binder.(*Binder).newFlowNode"}:
            return name
    return "<no recognized constructor ancestor>"


def summarize(samples, metric):
    index = TYPES.index(metric)
    flat, inclusive, domains, factories = Counter(), Counter(), Counter(), Counter()
    total = positive = negative = 0
    for sample in samples:
        value, frames = sample["values"][index], sample["frames"]
        total += value
        positive += max(value, 0)
        negative += min(value, 0)
        flat[frames[0]] += value
        for name in {frame[0] for frame in frames}:
            inclusive[name] += value
        domains[domain(frames)] += value
        factories[factory(frames)] += value
    if sum(flat.values()) != total or sum(domains.values()) != total or sum(factories.values()) != total:
        raise ValueError("sample attribution lost values")

    def ranked(values, key):
        return [{key: name, "estimated_bytes": value} for name, value in
                sorted(values.items(), key=lambda pair: (-abs(pair[1]), str(pair[0]))) if value][:40]

    return {"estimated_bytes": total, "positive_estimated_bytes": positive, "negative_estimated_bytes": negative,
            "exclusive_stack_domain_bytes": dict(domains), "exclusive_constructor_ancestor_bytes": dict(factories),
            "top_flat_source_sites": [{"function": name, "file": file, "line": line, "estimated_bytes": value}
                                      for (name, file, line), value in sorted(flat.items(), key=lambda pair: (-abs(pair[1]), pair[0])) if value][:40],
            "top_inclusive_functions": ranked(inclusive, "function")}


def profile_inputs(capture, row):
    if type(row.get("workers")) is not int or row["workers"] not in (1, 8) or type(row.get("repetition")) is not int or row["repetition"] < 0:
        raise ValueError("invalid Go run identity")
    report_path = capture / f"go-{row['workers']}-{row['repetition']}-report.json"
    report = strict_json_loads(report_path.read_bytes())
    if report != row["report"] or report.get("workers") != row["workers"] or report.get("diagnostic_only") is not True or report.get("mem_profile_rate") != 65536:
        raise ValueError("adapter report differs from capture")
    profiles = {item["name"]: item for item in report["profiles"]}
    if len(profiles) != len(report["profiles"]) or set(profiles) != {"pre_pipeline", "retained_endpoint", "retained_after_gc", "post_retirement_after_gc"}:
        raise ValueError("profile snapshot inventory mismatch")
    result = {}
    for name, profile in profiles.items():
        for kind in ("heap", "allocs"):
            path = capture / Path(profile[kind + "_path"]).name
            if sha(path) != profile[kind + "_sha256"]:
                raise ValueError("captured profile hash mismatch: " + str(path))
            result[(name, kind)] = path
    return report_path, report, result


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--capture", type=Path, required=True)
    args = ap.parse_args()
    capture = args.capture.resolve()
    capture_path = capture / "report.json"
    capture_hash = sha(capture_path)
    report = strict_json_loads(capture_path.read_bytes())
    if report.get("schema") != 1 or report.get("capture_complete") is not True or report.get("diagnostic_only") is not True:
        raise ValueError("requires a completed diagnostic capture")
    repetitions = report.get("repetitions")
    if type(repetitions) is not int or not 1 <= repetitions <= 5:
        raise ValueError("invalid capture repetitions")
    runs = [row for row in report["runs"] if row.get("runtime") == "go"]
    keys = [(row["workers"], row["repetition"]) for row in runs]
    if not keys or len(set(keys)) != len(keys) or set(keys) != {(worker, index) for worker in {key[0] for key in keys} for index in range(repetitions)}:
        raise ValueError("Go capture matrix incomplete or duplicated")
    env = go_native_environment()
    version = subprocess.check_output(["go", "version"], env=env, text=True).strip()
    output = capture / "go-memory-analysis"
    output.mkdir(exist_ok=True)
    results = []
    for row in runs:
        source_path, adapter, inputs = profile_inputs(capture, row)
        for key, value in report["expected_work"].items():
            if type(adapter.get(key)) is not type(value) or adapter[key] != value:
                raise ValueError("frozen workload denominator differs: " + key)
        prefix = f"go-{row['workers']}-{row['repetition']}"
        views = {}
        baseline, retained = inputs[("pre_pipeline", "heap")], inputs[("retained_after_gc", "heap")]
        for name, argv in (("baseline", [str(baseline)]), ("retained_after_gc", [str(retained)]),
                           ("retained_after_gc_minus_baseline", ["-base=" + str(baseline), str(retained)])):
            artifacts = {}
            for view, flags in (("raw", ["-raw"]), ("alloc-flat", ["-top", "-sample_index=alloc_space"]),
                                ("alloc-inclusive", ["-top", "-cum", "-sample_index=alloc_space"]),
                                ("inuse-flat", ["-top", "-sample_index=inuse_space"]),
                                ("inuse-inclusive", ["-top", "-cum", "-sample_index=inuse_space"])):
                command = ["go", "tool", "pprof", "-unit=bytes", "-nodecount=40", "-nodefraction=0", "-drop_negative=false", *flags, *argv]
                run = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, check=True)
                path = output / f"{prefix}-{name}-{view}.txt"
                path.write_bytes(run.stdout)
                stderr = path.with_suffix(".stderr")
                stderr.write_bytes(run.stderr)
                artifacts[view] = {"path": str(path), "sha256": sha(path), "stderr_sha256": sha(stderr), "command": command}
            metadata, samples = parse_raw(Path(artifacts["raw"]["path"]).read_text())
            if metadata["period_bytes"] != adapter["mem_profile_rate"]:
                raise ValueError("pprof sample rate differs from adapter")
            if name != "retained_after_gc_minus_baseline" and any(value < 0 for sample in samples for value in sample["values"]):
                raise ValueError("negative values in original memory profile")
            views[name] = {**metadata, "artifacts": artifacts, **{metric: summarize(samples, metric) for metric in ("alloc_space", "inuse_space")}}
        for metric in ("alloc_space", "inuse_space"):
            expected = views["retained_after_gc"][metric]["estimated_bytes"] - views["baseline"][metric]["estimated_bytes"]
            if views["retained_after_gc_minus_baseline"][metric]["estimated_bytes"] != expected:
                raise ValueError("native pprof subtraction changed the byte denominator")
        profile_inputs(capture, row)
        results.append({"workers": row["workers"], "repetition": row["repetition"],
                        "adapter_report_sha256": sha(source_path), "exact_pipeline_total_alloc_delta_bytes": adapter["allocated_bytes"],
                        "snapshots": adapter["snapshots"], "profiles": views})
        print("Analyzed " + prefix, file=sys.stderr)
    if sha(capture_path) != capture_hash:
        raise ValueError("capture changed during analysis")
    summary = {"schema": 1, "diagnostic_only": True, "capture_report_sha256": capture_hash, "go_version": version,
               "analyzer_sha256": sha(Path(__file__)), "runs": results,
               "method": "Pinned pprof exports all four memory sample values. Estimated byte sums preserve signed baseline subtraction without normalization or negative-value dropping. Flat sites use the first inline frame; inclusive functions count each function once per record. Constructor/domain grouping uses actual ancestors, never generic-shape spelling. Domain priority is diagnostic profile/checkpoint, census, binder, parser, preload/digest, other compiler, then other/truncated. This is allocation-stack attribution, not CPU-style phase labels.",
               "limits": ["64KiB statistical memory sampling yields weighted estimates, not exact requested bytes or RSS. Sample record/alloc_objects counts are not raw sampler-event counts.",
                          "Baseline and ordinary retained profiles can lag up to two GC cycles. Retained-after-GC is a separate deliberate snapshot. Baseline subtraction can retain negative freed values and delayed preload allocations; do not force reconciliation to pipeline TotalAlloc.",
                          "Post-GC alloc_space includes prior endpoint/profile/checkpoint diagnostic overhead; stack domains preserve visible overhead, and truncated stacks can remain unclassified. Post-retained census is excluded from these selected profiles.",
                          "Inclusive functions overlap and cannot be added. Concrete constructor ancestors identify source callsites; sampled allocation bytes can include whole arena backing arrays, unused slots and allocator rounding, unlike reachable-object sizeof census.",
                          "Heap live bytes, native RSS/footprint and census logical bytes are different domains. No difference of Rust/Go census totals is claimed to explain the actual retained-heap gap."]}
    path = capture / "go-memory-summary.json"
    path.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")
    print(json.dumps({"summary": str(path), "sha256": sha(path)}))


if __name__ == "__main__":
    main()
