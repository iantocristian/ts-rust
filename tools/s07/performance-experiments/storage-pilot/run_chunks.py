#!/usr/bin/env python3
"""Fresh next-fit chunk capture with same-batch legacy and page256 controls."""
import importlib.util
from pathlib import Path

import run_lists as baseline

HERE = Path(__file__).resolve().parent
POLICIES = ("legacy", "page256", "chunk256", "chunk1024")

# Each original function resolves globals in this isolated module instance.
# The pristine run_lists module, its files and the first capture remain intact.
SPEC = importlib.util.spec_from_file_location("s07_chunk_capture_backend", HERE / "run_lists.py")
backend = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(backend)


def chunk_layout(lengths, width):
    """Next-fit completed backings: abandoned tails are never revisited."""
    baseline.integer(width, 1, 2**32 - 1)
    remaining, capacity, chunks, used = 0, 0, 0, 0
    for length in lengths:
        baseline.integer(length, 0, 2**32 - 1)
        if length == 0:
            continue
        if length > remaining:
            allocated = max(width, length)
            capacity += allocated
            chunks += 1
            remaining = allocated
        remaining -= length
        used += length
    return {"edge_chunks": chunks, "spare_edge_words": capacity - used}


def expected_from_raw(raw, *, validate_owners=False):
    expected = baseline.expected_from_raw(raw, validate_owners=validate_owners)
    expected["page_counts"] = {"page256": expected["page_counts"]["page256"]}
    chunks = {mode: {"edge_chunks": 0, "spare_edge_words": 0} for mode in POLICIES if mode.startswith("chunk")}
    for line in raw.splitlines():
        row = baseline.strict_json_loads(line)
        lengths = row["node_backings"]["physical_lengths_in_aux_order"]
        for mode, total in chunks.items():
            observed = chunk_layout(lengths, int(mode.removeprefix("chunk")))
            for key, value in observed.items():
                total[key] += value
    expected["chunk_storage"] = chunks
    return expected


def validate_child(value, expected, identity, sweeps):
    mode = identity["mode"]
    if mode in ("legacy", "page256"):
        return baseline.validate_child(value, expected, identity, sweeps)
    require = baseline.require
    integer = baseline.integer
    require(mode in ("chunk256", "chunk1024"), "unknown chunk policy")
    # Repeat the small common envelope check because the frozen original couples
    # it to paged storage. Do not fabricate a page-shaped result to reuse it.
    require(type(value) is dict and set(value) == {"version", "diagnostic_only", "mode", "scope", "input_sha256", "files", "backings", "edges", "checksum", "allocation", "timing", "storage"}, "unknown/missing chunk child fields")
    require(integer(value["version"]) == 1 and value["diagnostic_only"] is True and value["scope"] == baseline.SCOPE and value["mode"] == mode, "chunk child identity/scope changed")
    for key in ("files", "backings", "edges", "checksum"):
        require(integer(value[key]) == expected[key], "chunk child work/checksum differs: " + key)
    require(value["input_sha256"] == expected["input_sha256"], "chunk child loaded different input bytes")
    if identity["allocation"]:
        allocation = value["allocation"]
        require(value["timing"] is None and type(allocation) is dict and set(allocation) == {"requested_bytes", "retained_requested_bytes", "freed_or_superseded_requests", "drop_returns_to_start"}, "wrong chunk allocation metric presence")
        for key in ("requested_bytes", "retained_requested_bytes", "freed_or_superseded_requests"):
            integer(allocation[key], 0 if key == "freed_or_superseded_requests" else 1)
        require(allocation["drop_returns_to_start"] is True and allocation["requested_bytes"] - allocation["retained_requested_bytes"] == allocation["freed_or_superseded_requests"], "chunk allocation/drop accounting differs")
        require(allocation["retained_requested_bytes"] >= expected["edges"] * 4, "chunk retained counter is below stored edge bytes")
    else:
        timing = value["timing"]
        require(value["allocation"] is None and type(timing) is dict and set(timing) == {"construction_ns", "traversal_ns", "sweeps"}, "wrong chunk normal metric presence")
        for key in ("construction_ns", "traversal_ns"):
            integer(timing[key], 1, 600_000_000_000)
        require(integer(timing["sweeps"], 1, 100) == sweeps, "chunk traversal sweep count changed")
    storage = value["storage"]
    require(type(storage) is dict and set(storage) == {"edge_chunks", "spare_edge_words", "retained_scratch_words"}, "unknown/missing chunk storage fields")
    for count in storage.values():
        integer(count)
    require(all(storage[key] == count for key, count in expected["chunk_storage"][mode].items()), "chunk count/slack differs from next-fit distribution")
    require(storage["retained_scratch_words"] >= expected["scratch_minimum"], "chunk scratch cannot hold the largest backings")
    return value


def tool_files():
    return {**baseline.tool_files(), **{str(path.relative_to(baseline.ROOT)): baseline.runner.digest(path)
        for path in (Path(__file__), HERE / "test_run_chunks.py")}}


backend.POLICIES = POLICIES
backend.BUILD_KIND = "s07_bis_chunk_distribution_build"
backend.CAPTURE_KIND = "s07_bis_chunk_distribution_capture"
backend.expected_from_raw = expected_from_raw
backend.validate_child = validate_child
backend.tool_files = tool_files
backend.__doc__ = __doc__


if __name__ == "__main__":
    backend.main()
