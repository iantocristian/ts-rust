#!/usr/bin/env python3
"""Fresh access-strategy replay with same-batch legacy and checked chunk controls."""
import importlib.util
from pathlib import Path

import run_chunks as chunks
import run_lists as baseline

HERE = Path(__file__).resolve().parent
POLICIES = ("legacy", "chunk256", "chunk256-owned", "chunk256-borrowed")
CORE_STORAGE = {"edge_chunks", "spare_edge_words", "retained_scratch_words"}
CACHE_STORAGE = {"borrowed_view_count", "borrowed_view_capacity_bytes"}

# Retain the original runner and both earlier experiments unchanged. Original
# functions resolve their scheduling/validation globals in this separate module.
SPEC = importlib.util.spec_from_file_location("s07_access_capture_backend", HERE / "run_lists.py")
backend = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(backend)
original_validate_observations = backend.validate_observations
original_capture = backend.capture


def expected_from_raw(raw, *, validate_owners=False):
    expected = baseline.expected_from_raw(raw, validate_owners=validate_owners)
    del expected["page_counts"], expected["wide_backings"]
    layout = {"edge_chunks": 0, "spare_edge_words": 0}
    for line in raw.splitlines():
        row = baseline.strict_json_loads(line)
        observed = chunks.chunk_layout(row["node_backings"]["physical_lengths_in_aux_order"], 256)
        for key, value in observed.items():
            layout[key] += value
    expected["chunk_storage"] = {"chunk256": layout}
    # This trial is the existing native64 target. Explicit with_capacity calls
    # reserve one slice descriptor per backing (including empty backings), plus
    # one Vec header per file in the outer cache. These are requested bytes,
    # not an estimate of allocator rounding or RSS.
    expected["borrowed_cache"] = {"borrowed_view_count": expected["backings"],
        "borrowed_view_capacity_bytes": expected["backings"] * 16 + expected["files"] * 24}
    return expected


def validate_child(value, expected, identity, sweeps):
    baseline.require(type(sweeps) is int and sweeps == 8, "access trial requires exactly eight sweeps")
    mode = identity["mode"]
    if mode == "legacy":
        return baseline.validate_child(value, expected, identity, sweeps)
    if mode == "chunk256":
        return chunks.validate_child(value, expected, identity, sweeps)
    require, integer = baseline.require, baseline.integer
    require(mode in ("chunk256-owned", "chunk256-borrowed"), "unknown access strategy")
    require(type(value) is dict and set(value) == {"version", "diagnostic_only", "mode", "scope", "input_sha256", "files", "backings", "edges", "checksum", "allocation", "timing", "storage"}, "unknown/missing access child fields")
    require(integer(value["version"]) == 1 and value["diagnostic_only"] is True and value["scope"] == baseline.SCOPE and value["mode"] == mode, "access child identity/scope changed")
    for key in ("files", "backings", "edges", "checksum"):
        require(integer(value[key]) == expected[key], "access child work/checksum differs: " + key)
    require(value["input_sha256"] == expected["input_sha256"], "access child loaded different input bytes")
    if identity["allocation"]:
        allocation = value["allocation"]
        require(value["timing"] is None and type(allocation) is dict and set(allocation) == {"requested_bytes", "retained_requested_bytes", "freed_or_superseded_requests", "drop_returns_to_start"}, "wrong access allocation metric presence")
        for key in ("requested_bytes", "retained_requested_bytes", "freed_or_superseded_requests"):
            integer(allocation[key], 0 if key == "freed_or_superseded_requests" else 1)
        require(allocation["drop_returns_to_start"] is True and allocation["requested_bytes"] - allocation["retained_requested_bytes"] == allocation["freed_or_superseded_requests"], "access allocation/drop accounting differs")
        cache_bytes = expected["borrowed_cache"]["borrowed_view_capacity_bytes"] if mode == "chunk256-borrowed" else 0
        require(allocation["retained_requested_bytes"] >= expected["edges"] * 4 + cache_bytes, "access retained counter omits stored edges or borrowed cache")
    else:
        timing = value["timing"]
        require(value["allocation"] is None and type(timing) is dict and set(timing) == {"construction_ns", "traversal_ns", "sweeps"}, "wrong access normal metric presence")
        for key in ("construction_ns", "traversal_ns"):
            integer(timing[key], 1, 600_000_000_000)
        require(integer(timing["sweeps"], 1, 100) == sweeps, "access traversal sweep count changed")
    storage = value["storage"]
    require(type(storage) is dict and set(storage) == CORE_STORAGE | CACHE_STORAGE, "unknown/missing access storage fields")
    for count in storage.values():
        integer(count)
    require(all(storage[key] == count for key, count in expected["chunk_storage"]["chunk256"].items()), "access changed core chunk distribution")
    require(storage["retained_scratch_words"] >= expected["scratch_minimum"], "access scratch cannot hold the largest backing")
    cache = expected["borrowed_cache"] if mode == "chunk256-borrowed" else dict.fromkeys(CACHE_STORAGE, 0)
    require(all(storage[key] == count for key, count in cache.items()), "access cache count/capacity differs from complete physical backings")
    return value


def validate_observations(report, directory, expected):
    original_validate_observations(report, directory, expected)
    controls = [row["child"]["storage"] for row in report["observations"] if row["identity"]["mode"] == "chunk256"]
    core = controls[0]
    for row in report["observations"]:
        if row["identity"]["mode"].startswith("chunk256"):
            baseline.require({key: row["child"]["storage"][key] for key in CORE_STORAGE} == core, "access strategy changed shared physical storage")
    measured = {(row["identity"]["mode"], row["identity"]["warmup"], row["identity"]["index"]): row["child"]["allocation"]
        for row in report["observations"] if row["identity"]["allocation"]}
    cache_bytes = expected["borrowed_cache"]["borrowed_view_capacity_bytes"]
    for (mode, warmup, index), checked in measured.items():
        if mode != "chunk256":
            continue
        baseline.require(measured[("chunk256-owned", warmup, index)] == checked, "owned access changed construction allocations")
        borrowed = measured[("chunk256-borrowed", warmup, index)]
        baseline.require(borrowed == {**checked,
            "requested_bytes": checked["requested_bytes"] + cache_bytes,
            "retained_requested_bytes": checked["retained_requested_bytes"] + cache_bytes}, "borrowed cache allocation is not charged at the measured endpoint")


def tool_files():
    return {**chunks.tool_files(), **{str(path.relative_to(baseline.ROOT)): baseline.runner.digest(path)
        for path in (Path(__file__), HERE / "test_run_access.py")}}


def capture(build_dir, build_sha, census_dir, census_sha, output, sweeps):
    baseline.require(type(sweeps) is int and sweeps == 8, "access trial requires exactly eight sweeps")
    return original_capture(build_dir, build_sha, census_dir, census_sha, output, sweeps)


backend.POLICIES = POLICIES
backend.BUILD_KIND = "s07_bis_list_access_build"
backend.CAPTURE_KIND = "s07_bis_list_access_capture"
backend.expected_from_raw = expected_from_raw
backend.validate_child = validate_child
backend.validate_observations = validate_observations
backend.capture = capture
backend.tool_files = tool_files
backend.__doc__ = __doc__


if __name__ == "__main__":
    backend.main()
