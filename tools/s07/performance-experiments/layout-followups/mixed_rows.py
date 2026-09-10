#!/usr/bin/env python3
"""Additive mixed-row projection using the unchanged archived CP0 inputs."""
from collections import Counter
import argparse
import gzip
import hashlib
import json
from pathlib import Path
from statistics import median
import subprocess
import sys

HERE = Path(__file__).resolve().parent
PHASES = HERE.parent / "phases"
ROOT = HERE.parents[3]
sys.path.insert(0, str(PHASES))
import project_layout as previous


def sha(data):
    return hashlib.sha256(data).hexdigest()


def inputs():
    paths = [PHASES / "layout-projection.json.gz", PHASES / "core-shapes.ndjson.gz",
             PHASES / "core-shapes-provenance.json.gz", PHASES / "project_layout.py",
             PHASES / "directory_sketch.rs", HERE.parent / "layout/baseline-model.json"]
    frozen = {str(path.relative_to(ROOT)): sha(path.read_bytes()) for path in paths}
    recorded = previous.strict_json_loads(gzip.decompress(paths[0].read_bytes()))
    raw = gzip.decompress(paths[1].read_bytes())
    proof = previous.strict_json_loads(gzip.decompress(paths[2].read_bytes()))
    model = previous.strict_json_loads(paths[5].read_bytes())
    if (sha(raw) != recorded["provenance"]["core_shapes_sha256"]
            or sha(raw) != proof["capture"]["summary"]["core_shapes_sha256"]
            or frozen[str(paths[3].relative_to(ROOT))] != recorded["provenance"]["projection_script_sha256"]
            or frozen[str(paths[4].relative_to(ROOT))] != recorded["provenance"]["directory_compilation"]["source_sha256"]
            or frozen[str(paths[5].relative_to(ROOT))] != recorded["provenance"]["generated_layout_model_sha256"]):
        raise ValueError("historical model/census input changed")
    capture, build = proof["capture"], proof["build_manifest"]
    if (capture.get("status") != "complete" or capture.get("kind") != "owned_core_shapes"
            or capture["build_manifest_sha256"] != recorded["provenance"]["build_manifest_sha256"]
            or build["source_fingerprint"]["sha256"] != recorded["provenance"]["build_source_fingerprint"]):
        raise ValueError("historical census build identity changed")
    for stream in ("stdout", "stderr"):
        if sha(proof["raw_" + stream].encode()) != capture["raw_sha256"]["sample-0-consuming." + stream]:
            raise ValueError("historical census child bytes changed")
    child = previous.probe.validate_observation(previous.strict_json_loads(proof["raw_stdout"]),
        proof["build_manifest"]["expected_work"], "consuming", True)
    if child["loaded_input_sha256"] != recorded["provenance"]["loaded_input_sha256"]:
        raise ValueError("historical loaded input identity changed")
    rows = [previous.strict_json_loads(line) for line in raw.splitlines()]
    validate_rows(rows, child)
    return rows, model, recorded, frozen


def validate_rows(rows, expected):
    if len(rows) != expected["files"]:
        raise ValueError("census file count changed")
    total, exclusive = 0, 0
    for index, row in enumerate(rows):
        if (set(row) != {"index", "core_shapes", "bound_in_place"}
                or type(row["index"]) is not int or row["index"] != index
                or type(row["bound_in_place"]) is not bool or type(row["core_shapes"]) is not dict):
            raise ValueError("census file order or fields changed")
        for name, count in row["core_shapes"].items():
            if type(name) is not str or type(count) is not int or count < 1:
                raise ValueError("invalid census shape count")
            total += count
        exclusive += row["bound_in_place"]
    if total != expected["nodes"] or exclusive != expected["bound_in_place_files"]:
        raise ValueError("census physical nodes or binding paths changed")


def compile_sketch(output):
    output.mkdir(parents=True, exist_ok=True)
    source = HERE / "mixed_rows.rs"
    executable, tests = output / "mixed-rows", output / "mixed-row-tests"
    common = ["rustc", "--edition=2021", "-Dwarnings", str(source)]
    subprocess.run([*common, "-o", str(executable)], check=True)
    subprocess.run([*common, "--test", "-o", str(tests)], check=True)
    test_output = subprocess.run([str(tests)], capture_output=True, text=True, check=True).stdout
    raw = subprocess.run([str(executable)], capture_output=True, text=True, check=True).stdout
    layouts = {name: {"size": int(size), "alignment": int(alignment)}
               for name, size, alignment in map(str.split, raw.splitlines())}
    return layouts, {"rustc": subprocess.run(["rustc", "-vV"], capture_output=True, text=True, check=True).stdout,
        "source_sha256": sha(source.read_bytes()), "executable_sha256": sha(executable.read_bytes()),
        "test_output": test_output}


def group_shapes(rows, shapes, layouts, mixed):
    classes, shape_classes = {}, {}
    for name, shape in shapes.items():
        physical = shape["bound_layout"]
        if type(shape["facts"]) is not bool:
            raise ValueError("invalid composite classification: " + name)
        if physical["size"] % 4 or physical["alignment"] > 4:
            raise ValueError("unsupported word alignment/size: " + name)
        width = physical["size"] // 4
        if shape["facts"] and not width:
            raise ValueError("composite layout omitted its facts word")
        family = ("composite" if shape["facts"] else "plain") if mixed else "atomic"
        key = family + "." + str(width)
        shape_classes[name] = key if width else None
        if not width:
            continue
        compiled = layouts[key]
        if compiled != physical:
            raise ValueError("row layout differs from the previous bound shape: " + name)
        classes[key] = {"syntax_layout": compiled, "bound_layout": compiled,
            "total_words": width, "plain_scalar_words": width - int(shape["facts"]) if mixed else 0,
            "atomic_words": int(shape["facts"]) if mixed else width}
    grouped, observed, active = [], Counter(), []
    used, binding_used, atomics = 0, 0, 0
    for row in rows:
        counts = Counter()
        for name, count in row["core_shapes"].items():
            if name not in shapes:
                raise ValueError("unknown concrete shape: " + name)
            shape = shapes[name]
            key = shape_classes[name]
            if key is not None:
                counts[key] += count
                atomics += count * classes[key]["atomic_words"]
            used += count * shape["bound_layout"]["size"]
            binding_used += count * (shape["bound_layout"]["size"] - shape["syntax_layout"]["size"])
        observed.update(counts)
        grouped.append({"core_shapes": dict(counts)})
        active.append(len(counts))
    return grouped, classes, {"generated_nonempty_classes": len(classes), "observed_classes": len(observed),
        "class_records": dict(sorted(observed.items())), "shape_to_class": shape_classes,
        "active_file_class_pairs": sum(active), "active_per_file": {"minimum": min(active), "median": median(active), "maximum": max(active)},
        "used_bound_payload_bytes": used, "used_binding_increment": binding_used, "used_atomic_words": atomics}


def project(rows, model, recorded, layouts):
    sizes = recorded["directory_layouts"]
    for key in ("ThinPage", "ThinOwnerOneOrManyStore", "ThinStore"):
        if layouts[key] != sizes[key]:
            raise ValueError("mixed page/store layout differs from charged envelope: " + key)
    header = previous.lean_header_costs(rows, 32, 32, sizes, "Thin")
    families = {}
    for mixed in (False, True):
        family = "mixed_rows" if mixed else "all_atomic_rows"
        grouped, classes, counts = group_shapes(rows, model["shape_inventory"], layouts, mixed)
        candidates = []
        for page_size in (8, 16):
            costs, active = previous.page_costs(grouped, classes, page_size, page_size, sizes, sizes["ThinPage"]["size"])
            if costs["used_bound"] != counts["used_bound_payload_bytes"]:
                raise ValueError("grouping lost or duplicated physical payload bytes")
            for strategy in previous.STRATEGIES:
                directory = previous.lean_directory_costs(strategy, costs, active, sizes, "Thin", "owner", len(classes))
                live = sum((header["capacity"], header["directory_live"], costs["capacity_bound"], directory["live"]))
                requests = sum((header["capacity"], header["directory_requests"], costs["capacity_bound"], directory["requests"]))
                calls = sum((header["pages"], header["directory_allocations"], costs["payload_pages"], directory["allocations_excluding_embedded_root"]))
                result = {"page_rows": page_size, "header_page_rows": 32, "directory_strategy": strategy,
                    "ledger_scope": "owner", "live_accounted_bytes": live, "requests_accounted_bytes": requests,
                    "directory_superseded_requests": requests - live,
                    "allocation_calls_excluding_embedded_owner_root": calls,
                    "syntax_attributing_all_shared_directories_and_spare_to_syntax": live - counts["used_binding_increment"],
                    "binding_used_increment": counts["used_binding_increment"],
                    "payload_spare_storage_span": costs["capacity_bound"] - costs["used_bound"],
                    "header": header, "payloads": costs, "directory": directory}
                if not mixed:
                    old = next(item for item in recorded["word_class_pages"]["atomic_word_variant"]["candidates"]
                        if item["payload_policy"] == {"descriptor": "Thin", "first": page_size, "maximum": page_size}
                        and item["header_policy"] == {"descriptor": "Thin", "first": 32, "maximum": 32}
                        and item["directory_strategy"] == strategy)
                    for key in ("live_accounted_bytes", "requests_accounted_bytes", "directory_superseded_requests", "allocation_calls_excluding_embedded_owner_root"):
                        if result[key] != old[key]:
                            raise ValueError("unchanged all-atomic control did not replay: " + key)
                candidates.append(result)
        families[family] = {**counts, "classes": classes, "candidates": candidates}
    comparison = []
    for n in (8, 16):
        selected = {family: next(row for row in value["candidates"] if row["page_rows"] == n and row["directory_strategy"] == "optional_one_or_many")
                    for family, value in families.items()}
        comparison.append({"page_rows": n, "directory_strategy": "optional_one_or_many", **selected,
            "mixed_minus_atomic": {key: selected["mixed_rows"][key] - selected["all_atomic_rows"][key]
                for key in ("live_accounted_bytes", "requests_accounted_bytes", "directory_superseded_requests", "allocation_calls_excluding_embedded_owner_root", "payload_spare_storage_span")}})
    return {"version": 1, "diagnostic_only": True, "outcome": "feasibility_unproved", "cpu": "unmeasured",
        "families": families, "matched_policy_comparison": comparison,
        "layouts": layouts, "limitations": ["Same used row bytes do not establish equal page slack, directories, requests, allocation calls or CPU cost.",
            "Composite scalar count excludes the existing facts word; plain rows contain no atomics. No separate facts table/index is added.",
            "Header shape remains independent from open syntax kind; its full u32 ordinal indexes the statically selected class. No extra per-node class locator is assumed.",
            "Page initialization, allocator rounding/RSS, owner-ledger drop/panic accounting, runtime IDs, full-domain escapes, text pools and the whole-owner native residual remain unproved.",
            "The previous schema's explicitly unmodeled checker-only fields and fallback obligations are unchanged; this does not add missing field coverage.",
            "Only the unchanged 8/16-row thin payload policies, 32-row thin header policy and prior five directory strategies are compared; no complete storage implementation or timing capture is produced."]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--scratch", type=Path, default=ROOT / "target/s07-bis-mixed-rows")
    parser.add_argument("--output", type=Path, default=HERE / "mixed-rows-result.json.gz")
    args = parser.parse_args()
    rows, model, recorded, before = inputs()
    layouts, compilation = compile_sketch(args.scratch)
    report = project(rows, model, recorded, layouts)
    if inputs()[3] != before:
        raise ValueError("historical input changed during projection")
    report["provenance"] = {"historical_inputs_sha256": before, "census_provenance": recorded["provenance"],
        "projection_source_sha256": sha(Path(__file__).read_bytes()), "compilation": compilation}
    data = (json.dumps(report, sort_keys=True, indent=2) + "\n").encode()
    args.output.write_bytes(gzip.compress(data, mtime=0) if args.output.suffix == ".gz" else data)
    (args.scratch / "mixed-rows-result.json").write_bytes(data)
    print(json.dumps({"outcome": report["outcome"], "cpu": report["cpu"],
        "counts": {family: {key: values[key] for key in ("generated_nonempty_classes", "observed_classes", "active_file_class_pairs", "used_bound_payload_bytes", "used_atomic_words")}
                   for family, values in report["families"].items()},
        "matched_policy_comparison": [{"page_rows": row["page_rows"], "mixed_minus_atomic": row["mixed_minus_atomic"]}
                                      for row in report["matched_policy_comparison"]]}, indent=2))


if __name__ == "__main__":
    main()
