#!/usr/bin/env python3
"""Replay A0b graphs, screening, grouped phase timers and recorded E3 tests."""
from contextlib import redirect_stderr
import importlib.util
import io
import json
from pathlib import Path, PurePosixPath
import re
import shlex
from statistics import median
import sys
import tarfile
import tempfile

from replay_a0 import ROOT, HERE, runner, sha, strict_json_loads, canonical, check_record, compare_rows
from s06_ownership import validate_output
from s07_ownership import COMMON, GROUPS, MODES, validate_manifest, publish_metrics


def unpack(directory):
    manifest = strict_json_loads((directory / "a0b-manifest.json").read_bytes())
    if (manifest.get("version") != 1 or manifest.get("diagnostic_only") is not True
            or manifest.get("retained_checkpoint") != "A0" or manifest.get("final_gates_passed") is not False):
        raise ValueError("A0b archive changed its checkpoint scope")
    archive = directory / manifest["archive"]["path"]
    if archive.stat().st_size != manifest["archive"]["bytes"] or sha(archive.read_bytes()) != manifest["archive"]["sha256"]:
        raise ValueError("A0b archive changed")
    payloads = {}
    with tarfile.open(archive, "r:xz") as stream:
        for member in stream:
            path = PurePosixPath(member.name)
            if (not member.isfile() or path.is_absolute() or ".." in path.parts
                    or member.name in payloads or member.name not in manifest["members"]):
                raise ValueError("unsafe, duplicate or unexpected archive member")
            raw = stream.extractfile(member).read()
            if {"sha256": sha(raw), "bytes": len(raw)} != manifest["members"][member.name]:
                raise ValueError("archive member changed: " + member.name)
            payloads[member.name] = raw
    if (set(payloads) != set(manifest["members"]) or len(payloads) != manifest["member_count"]
            or sum(len(raw) for raw in payloads.values()) != manifest["uncompressed_bytes"]):
        raise ValueError("archive member inventory incomplete")
    for name, expected in manifest["external_frozen_obligations"].items():
        if sha((ROOT / name).read_bytes()) != expected:
            raise ValueError("independent graph obligation changed")
    return manifest, payloads


def native_replay(manifest, payloads):
    control = strict_json_loads(payloads["manifests/control.json"])
    candidate = strict_json_loads(payloads["manifests/candidate.json"])
    for label in ("control", "candidate"):
        if sha(payloads["manifests/" + label + ".json"]) != manifest["source_manifest_sha256"][label]:
            raise ValueError("variant manifest identity changed")
    expected = control["expected_work"]
    graph = strict_json_loads(payloads["graphs/report.json"])
    if sha(payloads["graphs/report.json"]) != manifest["graph_report_sha256"]:
        raise ValueError("graph report identity changed")
    frozen = strict_json_loads((ROOT / "data/s07/bindworkload-probes.json").read_bytes())
    graph_proof = []
    with tempfile.TemporaryDirectory(prefix="s07-bis-a0b-replay-") as temporary:
        scratch = Path(temporary)
        report_path = scratch / "graph.json"
        report_path.write_bytes(payloads["graphs/report.json"])
        runner.validate_graph_report(report_path, control, candidate,
                                     manifest["source_manifest_sha256"]["control"],
                                     manifest["source_manifest_sha256"]["candidate"])
        for mode_index, (workers, recorded) in enumerate(zip((1, 8), graph["runs"], strict=True)):
            prefix = "graphs/workers-" + str(workers) + "/"
            rows = {}
            for runtime in ("oracle", "rust"):
                rows[runtime] = [strict_json_loads(line) for line in payloads[prefix + runtime + ".ndjson"].splitlines()]
                if len(rows[runtime]) != len(frozen["requests"]):
                    raise ValueError("graph stream omitted a file")
                for index, (row, request) in enumerate(zip(rows[runtime], frozen["requests"], strict=True)):
                    check_record(row, index, workers, request)
            recomputed = compare_rows(rows["oracle"], rows["rust"], frozen["requests"], frozen, workers, scratch)
            recomputed["first_mismatch_witness"] = None
            if recomputed != recorded or (scratch / "failures.ndjson").read_bytes() != payloads[prefix + "failures.ndjson"]:
                raise ValueError("graph report differs from its raw per-file observations")
            paths = runner.validate_paths(strict_json_loads(payloads[prefix + "binding-paths.stdout"]), expected, workers)
            if paths != graph["binding_paths"][mode_index]:
                raise ValueError("binding paths differ from raw observations")
            graph_proof.append({"workers": workers, "files_per_runtime": len(rows["rust"]),
                                "comparison_sha256": sha(canonical(recomputed)), "parity": recomputed["parity"]})

    screen = strict_json_loads(payloads["screen/report.json"])
    capture = strict_json_loads(payloads["screen/capture.json"])
    if (screen.get("status") != "complete" or screen.get("diagnostic_only") is not True
            or screen.get("manifest_sha256") != manifest["source_manifest_sha256"]
            or any(screen.get(key) != value for key, value in capture.items())):
        raise ValueError("screen capture identity changed")
    if screen["graph_report"]["sha256"] != manifest["graph_report_sha256"]:
        raise ValueError("screen names another graph prerequisite")
    raw_inventory = {name.removeprefix("screen/"): row for name, row in manifest["members"].items()
                     if name.startswith(("screen/sample-raw/", "screen/warmup-raw/"))}
    if raw_inventory != screen["raw_capture_inventory"]:
        raise ValueError("screen raw output inventory changed")
    observations = {}
    for name, warmup in (("samples", False), ("warmups", True)):
        raw = payloads["screen/" + name + ".ndjson"]
        if sha(raw) != screen[name + "_sha256"]:
            raise ValueError("screen ledger changed")
        observations[name] = [strict_json_loads(line) for line in raw.splitlines()]
        runner.validate_rows(observations[name], expected, warmup)
        for row in observations[name]:
            stem = "{workers}-{allocation}-{index}-{variant}".format(**row)
            folder = "warmup-raw" if warmup else "sample-raw"
            if strict_json_loads(payloads["screen/" + folder + "/" + stem + ".stdout"]) != row["sample"]:
                raise ValueError("screen row differs from raw stdout")
    summary = runner.summarize(observations["samples"], expected, candidate["target_metrics"])
    if any(screen.get(key) != value for key, value in summary.items()) or screen["samples"] != len(observations["samples"]):
        raise ValueError("screen summary differs from complete observations")
    if (summary["screening_status"] != "eligible_for_review" or not summary["targeted_pipeline_win"]
            or not summary["nonregression_conditions_met"]):
        raise ValueError("A0b screening outcome changed")
    return {"graph_replay": graph_proof, "samples": len(observations["samples"]),
            "warmups": len(observations["warmups"]), "summary_sha256": sha(canonical(summary)),
            "screening_status": summary["screening_status"]}


def phase_replay(manifest, payloads):
    build = strict_json_loads(payloads["phase-build/manifest.json"])
    report = strict_json_loads(payloads["phase-capture/report.json"])
    if (sha(payloads["phase-build/manifest.json"]) != manifest["phase_build_manifest_sha256"]
            or report["build_manifest_sha256"] != manifest["phase_build_manifest_sha256"]
            or report["status"] != "complete" or report["kind"] != "phase_attribution"):
        raise ValueError("phase build/capture identity changed")
    for phase, record in (("build", build), ("capture", report)):
        for name, expected in record["tool_inputs"].items():
            if sha(payloads["phase-" + phase + "-tools/" + name]) != expected:
                raise ValueError("original phase tool snapshot changed")
    for name in ("inputs.json", "cargo-messages.ndjson", "runtime-libraries.txt"):
        raw = payloads["phase-build/" + name]
        if {"bytes": len(raw), "sha256": sha(raw)} != build["inventory"][name]:
            raise ValueError("original phase build input changed")
    recipes = strict_json_loads(payloads["phase-build/inputs.json"])
    frozen = strict_json_loads((ROOT / "data/s07/bindworkload-probes.json").read_bytes())
    if len(recipes) != len(frozen["requests"]):
        raise ValueError("phase transport omitted a file")
    for recipe, request in zip(recipes, frozen["requests"], strict=True):
        if (set(recipe) != {"filename", "path", "local", "script_kind", "jsx", "force"}
                or any(recipe[key] != request[key] for key in recipe if key != "local")):
            raise ValueError("phase transport changed frozen options or file order")

    probe_path = ROOT / "tools/s07/performance-experiments/phases/probe.py"
    if sha(probe_path.read_bytes()) != report["tool_inputs"][str(probe_path.relative_to(ROOT))]:
        raise ValueError("phase replay requires the recorded probe validator version")
    specification = importlib.util.spec_from_file_location("a0b_phase_validator", probe_path)
    phase = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(phase)
    raw = payloads["phase-capture/observations.json"]
    if sha(raw) != report["observations_sha256"]:
        raise ValueError("phase observation ledger changed")
    observations = strict_json_loads(raw)
    schedule = list(phase.order())
    if len(observations) != len(schedule):
        raise ValueError("phase observations are incomplete or extended")
    for row, (warmup, index, backend) in zip(observations, schedule, strict=True):
        if (set(row) != {"warmup", "index", "backend", "report"} or type(row["warmup"]) is not bool
                or type(row["index"]) is not int or (row["warmup"], row["index"], row["backend"]) != (warmup, index, backend)):
            raise ValueError("phase observations changed order or identity")
        phase.validate_observation(row["report"], build["expected_work"], backend, False)
        stem = f"{'warmup' if warmup else 'sample'}-{index}-{backend}"
        if strict_json_loads(payloads["phase-capture/" + stem + ".stdout"]) != row["report"]:
            raise ValueError("phase observation differs from captured stdout")
    raw_hashes = {name.removeprefix("phase-capture/"): sha(raw) for name, raw in payloads.items()
                  if name.startswith("phase-capture/") and name.endswith((".stdout", ".stderr"))}
    if raw_hashes != report["raw_sha256"]:
        raise ValueError("phase raw output inventory changed")
    summary = {}
    for timer in (*phase.TIMERS, "pipeline_wall_ns"):
        values = {backend: [row["report"][timer] if timer == "pipeline_wall_ns" else row["report"]["elapsed_worker_totals"][timer]
                            for row in observations if row["backend"] == backend and not row["warmup"]] for backend in phase.BACKENDS}
        medians = {backend: median(samples) for backend, samples in values.items()}
        summary[timer] = {"values": values, "medians": medians, "consuming_over_published": medians["consuming"] / medians["published"]}
    if summary != report["summary"]:
        raise ValueError("phase summary differs from complete observations")
    return {"samples": 14, "warmups": 2, "summary_sha256": sha(canonical(summary)),
            "tool_snapshots_verified": sum(len(record["tool_inputs"]) for record in (build, report)),
            "domain": "one-worker per-file elapsed, same revision; binding plus publication grouped"}


def e3_replay(manifest, payloads):
    raw = payloads["e3/evidence.json"]
    if sha(raw) != manifest["e3_evidence_sha256"]:
        raise ValueError("recorded E3 evidence identity changed")
    evidence = strict_json_loads(raw)
    if evidence["exit_code"] != 0 or evidence["valid_capture"] is not True:
        raise ValueError("E3 is not a completed valid capture")
    for label, member in (("stdout", "e3/stdout.json"), ("stderr", "e3/stderr.txt")):
        if sha(payloads[member]) != evidence[label + "_sha256"] or payloads[member] != evidence[label].encode():
            raise ValueError("E3 raw output changed")
    inventory = validate_manifest(strict_json_loads(payloads["e3/data/s07/ownership-cases.json"]))
    suites = {**inventory["common"], **inventory["groups"]}
    text = evidence["stderr"]
    starts = list(re.finditer(r"^\+ (.+)$", text, re.MULTILINE))
    modes = {mode: {} for mode in MODES}
    for position, match in enumerate(starts):
        args = shlex.split(match.group(1))
        selected = [name for name, suite in suites.items() if suite["package"] in args and suite["filter"] in args]
        if not selected:
            continue
        if len(selected) != 1:
            raise ValueError("ambiguous E3 ownership command")
        name = selected[0]
        mode = "miri" if "miri" in args else "address_sanitizer" if "-Zbuild-std" in args else "release" if "--release" in args else "debug"
        if name in modes[mode]:
            raise ValueError("duplicate E3 suite/mode")
        stop = starts[position + 1].start() if position + 1 < len(starts) else len(text)
        with redirect_stderr(io.StringIO()):
            validate_output(text[match.end():stop].encode(), suites[name]["cases"], mode, "archived S07 ownership")
        modes[mode][name] = True
    measured = {"metrics": {}}
    publish_metrics(measured, modes, inventory)
    original = strict_json_loads(evidence["stdout"])
    if (any(original["metrics"].get(key) != value for key, value in measured["metrics"].items())
            or measured["metrics"]["program_ownership_tests"] != 27
            or not all(measured["metrics"][name] for name in GROUPS)):
        raise ValueError("E3 metrics differ from actual named suite output")
    return {"suites_per_mode": len(suites), "modes": sorted(modes), "distinct_cases": 27,
            "successful_named_test_observations": 108, "metrics_sha256": sha(canonical(measured["metrics"])),
            "scope": "replayed the 27 S07 cases; retained complete E3 output does not imply all future E3 criteria"}


def replay(directory=HERE):
    manifest, payloads = unpack(Path(directory))
    return {"version": 1, "diagnostic_only": True, "archive_sha256": manifest["archive"]["sha256"],
            "member_count": manifest["member_count"], "uncompressed_bytes": manifest["uncompressed_bytes"],
            "native": native_replay(manifest, payloads), "phases": phase_replay(manifest, payloads),
            "e3": e3_replay(manifest, payloads), "retained_checkpoint": "A0", "final_gates_passed": False,
            "scope": "member integrity and recorded-observation replay; no captured executable is run"}


if __name__ == "__main__":
    try:
        print(json.dumps(replay(), indent=2, sort_keys=True))
    except (OSError, ValueError, KeyError, tarfile.TarError) as error:
        print("A0b archive replay failed: " + str(error), file=sys.stderr)
        raise SystemExit(1) from error
