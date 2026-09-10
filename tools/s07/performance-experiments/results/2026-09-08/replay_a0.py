#!/usr/bin/env python3
"""Replay initial A0 observations without executing binaries or loading sources."""
import hashlib
import importlib.util
import json
from pathlib import Path, PurePosixPath
import sys
import tarfile
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
sys.path.insert(0, str(ROOT / "scripts"))
from s04_common import strict_json_loads
from s06_protocol import canonical
from s07_benchmark_graph import check_record, compare_rows

SPEC = importlib.util.spec_from_file_location("bis_runner_replay", ROOT / "tools/s07/performance-experiments/runner.py")
runner = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(runner)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def replay(directory=HERE):
    directory = Path(directory)
    manifest_path = directory / "a0-initial-manifest.json"
    manifest = strict_json_loads(manifest_path.read_bytes())
    if (manifest.get("version") != 1 or manifest.get("diagnostic_only") is not True
            or manifest.get("promoted") is not False):
        raise ValueError("archive must preserve the initial non-promoted diagnostic result")
    archive = directory / manifest["archive"]["path"]
    if archive.stat().st_size != manifest["archive"]["bytes"] or sha(archive.read_bytes()) != manifest["archive"]["sha256"]:
        raise ValueError("archive bytes changed")
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
    if set(payloads) != set(manifest["members"]) or len(payloads) != manifest["member_count"]:
        raise ValueError("archive is incomplete")
    if sum(len(raw) for raw in payloads.values()) != manifest["uncompressed_bytes"]:
        raise ValueError("archive size inventory changed")
    for name, expected in manifest["external_frozen_obligations"].items():
        if sha((ROOT / name).read_bytes()) != expected:
            raise ValueError("independent frozen obligation changed: " + name)

    control = strict_json_loads(payloads["manifests/control.json"])
    candidate = strict_json_loads(payloads["manifests/candidate.json"])
    for label, member in (("control", "manifests/control.json"), ("candidate", "manifests/candidate.json")):
        if sha(payloads[member]) != manifest["source_manifest_sha256"][label]:
            raise ValueError("immutable variant manifest identity changed")
    graph = strict_json_loads(payloads["graphs/report.json"])
    if sha(payloads["graphs/report.json"]) != manifest["graph_report_sha256"]:
        raise ValueError("graph prerequisite identity changed")
    expected = control["expected_work"]
    frozen = strict_json_loads((ROOT / "data/s07/bindworkload-probes.json").read_bytes())
    graph_proof = []
    with tempfile.TemporaryDirectory(prefix="s07-bis-a0-replay-") as temporary:
        scratch = Path(temporary)
        report_path = scratch / "graph.json"
        report_path.write_bytes(payloads["graphs/report.json"])
        runner.validate_graph_report(report_path, control, candidate,
                                     manifest["source_manifest_sha256"]["control"],
                                     manifest["source_manifest_sha256"]["candidate"])
        for workers, recorded in zip((1, 8), graph["runs"], strict=True):
            rows = {}
            prefix = "graphs/workers-" + str(workers) + "/"
            for runtime in ("oracle", "rust"):
                rows[runtime] = [strict_json_loads(line) for line in payloads[prefix + runtime + ".ndjson"].splitlines()]
                if len(rows[runtime]) != len(frozen["requests"]):
                    raise ValueError("graph stream omitted a file")
                for index, (row, request) in enumerate(zip(rows[runtime], frozen["requests"], strict=True)):
                    check_record(row, index, workers, request)
            recomputed = compare_rows(rows["oracle"], rows["rust"], frozen["requests"], frozen, workers, scratch)
            recomputed["first_mismatch_witness"] = None
            if recomputed != recorded or (scratch / "failures.ndjson").read_bytes() != payloads[prefix + "failures.ndjson"]:
                raise ValueError("graph report differs from all raw per-file observations")
            paths = runner.validate_paths(strict_json_loads(payloads[prefix + "binding-paths.stdout"]), expected, workers)
            if paths != graph["binding_paths"][workers == 8]:
                raise ValueError("binding-path report differs from raw observation")
            graph_proof.append({"workers": workers, "files_per_runtime": len(rows["rust"]),
                                "comparison_sha256": sha(canonical(recomputed)), "parity": recomputed["parity"]})

    screen = strict_json_loads(payloads["screen/report.json"])
    capture = strict_json_loads(payloads["screen/capture.json"])
    if (screen.get("status") != "complete" or screen.get("diagnostic_only") is not True
            or screen.get("manifest_sha256") != manifest["source_manifest_sha256"]
            or any(screen.get(key) != value for key, value in capture.items())):
        raise ValueError("screen capture identity changed or is incomplete")
    if screen["graph_report"]["sha256"] != manifest["graph_report_sha256"]:
        raise ValueError("screen was paired with another graph prerequisite")
    raw_inventory = {name.removeprefix("screen/"): row for name, row in manifest["members"].items()
                     if name.startswith(("screen/sample-raw/", "screen/warmup-raw/"))}
    if raw_inventory != screen["raw_capture_inventory"]:
        raise ValueError("screen raw stdout/stderr inventory changed")
    observations = {}
    for name, warmup in (("samples", False), ("warmups", True)):
        raw = payloads["screen/" + name + ".ndjson"]
        if sha(raw) != screen[name + "_sha256"]:
            raise ValueError("sample ledger changed")
        observations[name] = [strict_json_loads(line) for line in raw.splitlines()]
        runner.validate_rows(observations[name], expected, warmup)
        for row in observations[name]:
            stem = "{workers}-{allocation}-{index}-{variant}".format(**row)
            folder = "warmup-raw" if warmup else "sample-raw"
            if strict_json_loads(payloads["screen/" + folder + "/" + stem + ".stdout"]) != row["sample"]:
                raise ValueError("sample differs from actual captured stdout")
    recomputed = runner.summarize(observations["samples"], expected, candidate["target_metrics"])
    if any(screen.get(key) != value for key, value in recomputed.items()) or screen["samples"] != len(observations["samples"]):
        raise ValueError("screen summary differs from every recorded sample")
    if (recomputed["screening_status"] != "regressing_or_uncertain"
            or recomputed["targeted_pipeline_win"] or recomputed["nonregression_conditions_met"]):
        raise ValueError("initial A0 non-promoted outcome changed")
    return {"version": 1, "diagnostic_only": True, "archive_sha256": manifest["archive"]["sha256"],
            "member_count": len(payloads), "uncompressed_bytes": manifest["uncompressed_bytes"],
            "graph_replay": graph_proof, "samples": len(observations["samples"]),
            "warmups": len(observations["warmups"]), "summary_sha256": sha(canonical(recomputed)),
            "screening_status": recomputed["screening_status"], "promoted": False,
            "scope": "member integrity plus graph/sample replay; no executable or source/workload bytes are present or executed"}


if __name__ == "__main__":
    try:
        print(json.dumps(replay(), indent=2, sort_keys=True))
    except (OSError, ValueError, KeyError, tarfile.TarError) as error:
        print("Initial A0 archive replay failed: " + str(error), file=sys.stderr)
        raise SystemExit(1) from error
