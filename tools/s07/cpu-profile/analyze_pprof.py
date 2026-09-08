#!/usr/bin/env python3
"""Retain pinned pprof views and partition actual CPU samples without double counting."""
import argparse
from collections import Counter, defaultdict
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "scripts"))
from s07_benchmark import go_native_environment
from s04_common import strict_json_loads


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def capture_profiles(capture):
    path = capture / "report.json"
    raw = path.read_bytes()
    report = strict_json_loads(raw)
    if (type(report) is not dict or report.get("schema") != 1
            or report.get("capture_complete") is not True or report.get("diagnostic_only") is not True
            or type(report.get("repetitions")) is not int or not 1 <= report["repetitions"] <= 10
            or type(report.get("runs")) is not list):
        raise ValueError("invalid completed capture manifest")
    repetitions = report["repetitions"]
    expected = {(workers, index) for workers in (1, 8) for index in range(repetitions)}
    profiles = {}
    for row in report["runs"]:
        if type(row) is not dict:
            raise ValueError("invalid capture run")
        if row.get("runtime") != "go":
            continue
        if (row.get("profiled") is not True or type(row.get("workers")) is not int
                or type(row.get("index")) is not int or type(row.get("artifacts")) is not dict):
            raise ValueError("invalid profiled Go run")
        key = (row["workers"], row["index"])
        digest = row["artifacts"].get("profile")
        if key not in expected or key in profiles:
            raise ValueError("duplicate or extra profiled Go run")
        if type(digest) is not str or not re.fullmatch(r"[0-9a-f]{64}", digest):
            raise ValueError("invalid captured profile hash")
        profiles[key] = digest
    if set(profiles) != expected:
        raise ValueError("missing profiled Go run")
    names = {f"go-{workers}-{index}.pprof" for workers, index in expected}
    if {path.name for path in capture.glob("go-*.pprof")} != names:
        raise ValueError("profile files differ from the captured run inventory")
    for (workers, index), digest in profiles.items():
        if sha(capture / f"go-{workers}-{index}.pprof") != digest:
            raise ValueError("profile bytes differ from the captured hash")
    return repetitions, profiles, hashlib.sha256(raw).hexdigest()


def parse_raw(text):
    header, remainder = text.split("Samples:\n", 1)
    samples_text, locations_text = remainder.split("\nLocations\n", 1)
    locations_text, _ = locations_text.split("\nMappings\n", 1)
    period = int(re.search(r"^Period: (-?\d+)$", header, re.M)[1])
    if period <= 0:
        raise ValueError("CPU sampling period must be positive")
    if "PeriodType: cpu nanoseconds" not in header:
        raise ValueError("unexpected CPU period unit")
    lines = samples_text.splitlines()
    if lines.pop(0) != "samples/count cpu/nanoseconds":
        raise ValueError("unexpected sample value types")
    samples = []
    for line in lines:
        if not line.strip():
            continue
        match = re.fullmatch(r"\s*(\d+)\s+(\d+):\s*([0-9 ]+)\s*", line)
        if match:
            count, cpu = int(match[1]), int(match[2])
            if count <= 0 or cpu != count * period:
                raise ValueError("sample count/CPU weight does not match period")
            samples.append({"count": count, "cpu_ns": cpu,
                            "locations": [int(value) for value in match[3].split()],
                            "phase": "unlabeled"})
        elif re.fullmatch(r"\s*phase:\[(parse|bind)\]", line):
            if not samples or samples[-1]["phase"] != "unlabeled":
                raise ValueError("duplicate or misplaced phase label")
            samples[-1]["phase"] = line.strip()[7:-1]
        else:
            raise ValueError("unrecognized raw sample line: " + line)
    if not samples:
        raise ValueError("empty CPU profile")
    locations = {}
    current = None
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
        match = re.fullmatch(r"(.+) (\S+):\d+:\d+(?: s=\d+)?", frame)
        if not match:
            raise ValueError("unrecognized function/source frame: " + frame)
        locations[current].append((match[1], match[2]))
    for sample in samples:
        if any(location not in locations for location in sample["locations"]):
            raise ValueError("sample references an unknown location")
        sample["frames"] = [frame for location in sample.pop("locations") for frame in locations[location]]
    # pprof's raw display truncates the formatted duration and drops its unit;
    # retain that text rather than misreading subsecond values as seconds.
    return {"period_ns": period, "raw_header": header,
            "sample_records": len(samples), "samples": samples}


def classify(frames):
    names = {name for name, _ in frames}
    if any(name.startswith("runtime.gcAssistAlloc") for name in names):
        return "gc_assist"
    if any(name.startswith("runtime.gcBgMarkWorker") for name in names):
        return "gc_background_mark"
    # Exact source-file GC implementation frames; ordinary mallocgc/mheap and
    # typedmemmove are deliberately not GC classifiers. Assists win above.
    if (any(path.startswith("runtime/mgc") or path == "runtime/mwbbuf.go" for _, path in frames)
            or any(name.startswith(("runtime.gcWriteBarrier", "gcWriteBarrier")) for name in names)):
        return "gc_other"
    if "runtime.madvise" in names:
        return "heap_madvise"
    if any(name.startswith(("runtime.mallocgc", "runtime.(*mheap).alloc", "runtime.(*mcentral).cacheSpan")) for name in names):
        return "allocation_other"
    if frames[0][0].startswith(("runtime.", "internal/runtime/")):
        return "runtime_other"
    return "other"


def summarize(samples):
    total = sum(sample["cpu_ns"] for sample in samples)
    phases, categories = Counter(), Counter()
    cross = defaultdict(Counter)
    self_time, inclusive = Counter(), Counter()
    phase_self, phase_inclusive = defaultdict(Counter), defaultdict(Counter)
    madvise_stacks = Counter()
    gc_stacks = Counter()
    for sample in samples:
        phase, cpu, frames = sample["phase"], sample["cpu_ns"], sample["frames"]
        category = classify(frames)
        phases[phase] += cpu
        categories[category] += cpu
        cross[phase][category] += cpu
        if "runtime.madvise" in {name for name, _ in frames}:
            madvise_stacks[(phase, tuple(name for name, _ in frames))] += cpu
        if category.startswith("gc_"):
            gc_stacks[(phase, category, tuple(name for name, _ in frames))] += cpu
        self_time[frames[0][0]] += cpu
        phase_self[phase][frames[0][0]] += cpu
        for name in {name for name, _ in frames}:
            inclusive[name] += cpu
            phase_inclusive[phase][name] += cpu
    if sum(phases.values()) != total or sum(categories.values()) != total:
        raise ValueError("CPU partition lost samples")

    def top(values):
        return [{"function": name, "cpu_ns": cpu, "percent_total_cpu": 100 * cpu / total}
                for name, cpu in sorted(values.items(), key=lambda row: (-row[1], row[0]))[:30]]

    return {"sample_count": sum(sample["count"] for sample in samples), "cpu_ns": total,
            "phase_cpu_ns": dict(phases), "exclusive_category_cpu_ns": dict(categories),
            "phase_category_cpu_ns": {phase: dict(values) for phase, values in cross.items()},
            "madvise_stacks": [{"phase": phase, "frames": list(frames), "cpu_ns": cpu}
                               for (phase, frames), cpu in sorted(madvise_stacks.items())],
            "gc_stacks": [{"phase": phase, "category": category, "frames": list(frames), "cpu_ns": cpu}
                          for (phase, category, frames), cpu in sorted(gc_stacks.items())],
            "top_self": top(self_time), "top_inclusive": top(inclusive),
            "phase_top_self": {phase: top(values) for phase, values in phase_self.items()},
            "phase_top_inclusive": {phase: top(values) for phase, values in phase_inclusive.items()}}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--capture", type=Path, default=ROOT / "target/s07-cpu-profiles/capture")
    args = parser.parse_args()
    repetitions, profiles, capture_hash = capture_profiles(args.capture)
    env = go_native_environment()
    version = subprocess.check_output(["go", "version"], env=env, text=True).strip()
    goroot = Path(subprocess.check_output(["go", "env", "GOROOT"], env=env, text=True).strip())
    runtime_sources = {}
    for name, start, end in (("mem_darwin.go", 29, 35), ("sys_darwin.go", 202, 211),
                             ("mheap.go", 1385, 1401), ("os_darwin.go", 457, 467)):
        path = goroot / "src/runtime" / name
        runtime_sources[name] = {"path": str(path), "sha256": sha(path), "start_line": start,
                                 "text": "\n".join(path.read_text().splitlines()[start - 1:end])}
    rows, combined = [], defaultdict(list)
    views = {"raw": ["-raw"], "top": ["-top", "-nodecount=40", "-unit=ms"],
             "top-cum": ["-top", "-cum", "-nodecount=40", "-unit=ms"], "tags": ["-tags"],
             "bind-top": ["-top", "-tagfocus=phase=bind", "-nodecount=30", "-unit=ms"],
             "bind-top-cum": ["-top", "-cum", "-tagfocus=phase=bind", "-nodecount=30", "-unit=ms"]}
    for workers in (1, 8):
        for index in range(repetitions):
            profile = args.capture / f"go-{workers}-{index}.pprof"
            before = profiles[(workers, index)]
            artifacts = {}
            for view, flags in views.items():
                command = ["go", "tool", "pprof", "-sample_index=cpu", *flags, str(profile)]
                result = subprocess.run(command, env=env, cwd=ROOT, capture_output=True, check=True)
                path = profile.with_suffix(f".pprof.{view}.txt")
                path.write_bytes(result.stdout)
                path.with_suffix(".stderr").write_bytes(result.stderr)
                artifacts[view] = {"path": str(path), "sha256": sha(path), "command": command}
            parsed = parse_raw(profile.with_suffix(".pprof.raw.txt").read_text())
            if before != sha(profile):
                raise ValueError("profile changed during analysis")
            samples = parsed.pop("samples")
            top_text = profile.with_suffix(".pprof.top.txt").read_text()
            displayed_cpu = int(re.search(r"Total samples = (\d+)ms", top_text)[1]) * 1_000_000
            if sum(sample["cpu_ns"] for sample in samples) != displayed_cpu:
                raise ValueError("raw sample CPU differs from native pprof total")
            rows.append({"workers": workers, "index": index, "profile_sha256": before,
                         **parsed, **summarize(samples), "artifacts": artifacts})
            combined[workers].extend(samples)
            print(f"Analyzed Go workers={workers} repetition={index}", file=sys.stderr)
    if capture_profiles(args.capture) != (repetitions, profiles, capture_hash):
        raise ValueError("capture manifest/profile inventory changed during analysis")
    report = {"schema": 1, "diagnostic_only": True, "go_version": version,
              "repetitions": repetitions, "capture_report_sha256": capture_hash,
              "analyzer_sha256": sha(Path(__file__)),
              "runtime_sources": runtime_sources,
              "madvise_contract": "At this Go pin on Darwin, sysUsedOS calls MADV_FREE_REUSE for kernel accounting. The exact observed madvise stacks are retained in each result. Their reported PC/path is not an attribution to GC, waiting, or a particular amount of kernel CPU. Labels retain parse/bind scope despite systemstack hiding user ancestors.",
              "comparison_limit": "Go samples use a 10ms CPU period; the Rust Time Profiler capture uses a different sampler at 1ms. Whole-process raw percentages from these different tools are not directly comparable; inspect independently delimited phases and retain sample uncertainty.",
              "method": "CPU weights and counts from pinned pprof -raw; phase labels partition the full denominator, including systemstack samples with no user ancestor. Self uses the innermost inline frame; inclusive counts each distinct function once per sample and must not be summed across functions. Exclusive categories use GC assist, background mark, other GC source frames, madvise, other allocation, other runtime, then remaining work in that priority. Categories and phases each sum to total CPU. GC is a subset of phases, not additional CPU. Sampling resolution is 10ms; labels/profiler overhead and runtime/system stacks limit source attribution.",
              "profiles": rows, "pooled": {str(workers): summarize(samples) for workers, samples in combined.items()}}
    output = args.capture / "go-pprof-summary.json"
    output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    print(json.dumps({"summary": str(output), "sha256": sha(output)}))


if __name__ == "__main__":
    main()
