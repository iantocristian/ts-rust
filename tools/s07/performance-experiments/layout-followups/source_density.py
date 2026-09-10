#!/usr/bin/env python3
"""Replay source-byte density from frozen manifest sizes; no runtime benchmark."""
import gzip
import hashlib
import json
from pathlib import Path
import tarfile

HERE = Path(__file__).resolve().parent
EXPERIMENTS = HERE.parent
RESULTS = EXPERIMENTS / "results/2026-09-08"


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def summarize(values):
    """Linear interpolation at index (n - 1) * q; each file has equal weight."""
    ordered = sorted(values)
    result = {}
    for label, q in (("min", 0), ("p05", .05), ("p25", .25), ("median", .5),
                     ("p75", .75), ("p95", .95), ("max", 1)):
        rank = (len(ordered) - 1) * q
        lower = int(rank)
        upper = min(lower + 1, len(ordered) - 1)
        result[label] = ordered[lower] + (ordered[upper] - ordered[lower]) * (rank - lower)
    return result


def report():
    metadata = json.loads((RESULTS / "a0b-manifest.json").read_bytes())
    archive = RESULTS / metadata["archive"]["path"]
    if digest(archive.read_bytes()) != metadata["archive"]["sha256"]:
        raise ValueError("archived manifest envelope changed")
    with tarfile.open(archive) as bundle:
        manifest_raw = bundle.extractfile("manifests/control.json").read()
    member = metadata["members"]["manifests/control.json"]
    if len(manifest_raw) != member["bytes"] or digest(manifest_raw) != member["sha256"]:
        raise ValueError("frozen control manifest changed")
    manifest = json.loads(manifest_raw)
    census_path = EXPERIMENTS / "phases/core-shapes.ndjson.gz"
    census_raw = gzip.decompress(census_path.read_bytes())
    provenance = json.loads(gzip.decompress(
        (EXPERIMENTS / "phases/core-shapes-provenance.json.gz").read_bytes()))
    if digest(census_raw) != provenance["capture"]["summary"]["core_shapes_sha256"]:
        raise ValueError("physical core census changed")
    rows = [json.loads(line) for line in census_raw.splitlines()]
    expected = manifest["expected_work"]
    if provenance["build_manifest"]["expected_work"] != expected:
        raise ValueError("source and physical census workloads differ")
    if len(rows) != expected["files"]:
        raise ValueError("missing physical core census records")
    sizes, nodes = [], []
    for index, row in enumerate(rows):
        if row["index"] != index:
            raise ValueError("census order changed")
        count = sum(row["core_shapes"].values())
        size = manifest["inventory"][f"workload/{index}"]["bytes"]
        if count <= 0 or size < 0:
            raise ValueError("invalid census/input size")
        nodes.append(count)
        sizes.append(size)
    if sum(nodes) != expected["nodes"] or sum(sizes) != expected["loaded_bytes"]:
        raise ValueError("source sizes and physical node obligations differ")
    density = sum(sizes) / sum(nodes)
    nonempty = [(size, count) for size, count in zip(sizes, nodes) if size]
    # Use the global density to predict each file's node count. This deliberately
    # prices neither minimum chunks nor headers, payloads, alignment or slack.
    predictions = [size / density for size in sizes]
    return {
        "version": 1, "diagnostic_only": True,
        "script_sha256": digest(Path(__file__).read_bytes()),
        "domain": "arithmetic over archived input sizes and physical core counts",
        "archive_sha256": metadata["archive"]["sha256"],
        "control_manifest_sha256": digest(manifest_raw),
        "core_census_sha256": digest(census_raw),
        "files": len(rows), "loaded_bytes": sum(sizes), "core_nodes": sum(nodes),
        "zero_byte_files": len(rows) - len(nonempty),
        "global_bytes_per_node": density,
        "file_weighted_bytes_per_node": summarize([size / count for size, count in zip(sizes, nodes)]),
        "predicted_over_actual_nodes_nonempty_files": summarize(
            [size / density / count for size, count in nonempty]),
        "files_with_prediction_below_actual": sum(p < n for p, n in zip(predictions, nodes)),
        "files_with_prediction_above_twice_actual": sum(p > 2 * n for p, n in zip(predictions, nodes)),
        "limitations": [
            "A global ratio is not a per-file bound or an implemented initial-chunk policy.",
            "No allocation ordering, row padding, list interleaving or chunk-tail slack is observed.",
            "No allocation call, native request, CPU or RSS measurement is produced.",
        ],
    }


if __name__ == "__main__":
    print(json.dumps(report(), indent=2, sort_keys=True))
