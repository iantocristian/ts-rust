#!/usr/bin/env python3
"""Package/replay recorded chunk results without rerunning a native child."""
import argparse
import gzip
import io
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile

import run_chunks
import replay_lists as archive_checks

probe = run_chunks.backend

HERE = Path(__file__).resolve().parent
ROOT = probe.ROOT
BUILD_SHA = "1e1f3cfcf0c53967f5959a5c04bd6127592abdac2f26bc7450c314632102d007"
CAPTURE_SHA = "835a97fa3d176383388c3d67717c5a7bfaf3779bc3b8c8cd42753d5713397f4c"
CENSUS = HERE.parent / "owner-census/results/2026-09-08"


def identity(raw):
    return {"bytes": len(raw), "sha256": probe.runner.sha(raw)}


def replay_inputs():
    # Include the reused archive parser/inventory checker without changing it.
    return {path.name: probe.runner.digest(path) for path in (Path(__file__), HERE / "test_replay_chunks.py", HERE / "replay_lists.py")}


# Archive parsing and exact nonbinary inventory checks are shared unchanged.
validate_members = archive_checks.validate_members
read_members = archive_checks.read_members


def package(build_dir, capture_dir, output):
    # This replays recorded observations and verifies actual immutable binaries;
    # it never launches them. Their bytes are deliberately omitted below.
    probe.verify(capture_dir, CAPTURE_SHA, build_dir, BUILD_SHA)
    subprocess.run([sys.executable, str(CENSUS.parents[1] / "replay.py"), str(CENSUS)], check=True, stdout=subprocess.DEVNULL)
    build = probe.strict_json_loads((build_dir / "manifest.json").read_bytes())
    capture = probe.strict_json_loads((capture_dir / "manifest.json").read_bytes())
    output.mkdir(parents=True, exist_ok=False)
    omitted = {record["path"] for record in build["artifacts"].values()}
    members = {}
    for prefix, directory, manifest, excluded in (("build", build_dir, build, omitted), ("capture", capture_dir, capture, {"input.ndjson"})):
        for name in ["manifest.json", *manifest["inventory"]]:
            if name not in excluded:
                members[prefix + "/" + name] = (directory / name).read_bytes()
    validate_members(members, build, capture)
    archive_path = output / "recorded-results.tar.xz"
    with tarfile.open(archive_path, "w:xz", format=tarfile.PAX_FORMAT) as archive:
        for name, raw in sorted(members.items()):
            info = tarfile.TarInfo(name)
            info.size = len(raw)
            info.mode = 0o644
            archive.addfile(info, io.BytesIO(raw))
    reference = CENSUS / "owners.ndjson.gz"
    probe.require(probe.runner.sha(gzip.decompress(reference.read_bytes())) == capture["expected"]["input_sha256"], "durable census input differs from measured bytes")
    manifest = {"version": 1, "diagnostic_only": True, "kind": "s07_bis_chunk_distribution_archive", "scope": probe.SCOPE,
        "build_manifest_sha256": BUILD_SHA, "capture_manifest_sha256": CAPTURE_SHA,
        "files": {archive_path.name: identity(archive_path.read_bytes())}, "archive_members": len(members),
        "input_reference": {"path": str(reference.relative_to(ROOT)), **identity(reference.read_bytes()),
            "raw_sha256": capture["expected"]["input_sha256"], "raw_bytes": capture["inventory"]["input.ndjson"]["bytes"],
            "census_manifest_sha256": capture["census_manifest_sha256"]},
        "replay_inputs": replay_inputs(), "omitted_binaries": sorted(omitted),
        "native_child_reexecuted": False, "limitations": ["Offline replay verifies recorded observations and source/helper provenance, not execution of omitted binary bytes.", "The shared owner-census gzip is required; no original workload files or target build directory are needed."]}
    probe.runner.write_json(output / "manifest.json", manifest)
    return replay(output)


def replay(directory):
    manifest = probe.strict_json_loads((directory / "manifest.json").read_bytes())
    probe.require(type(manifest["version"]) is int and manifest["version"] == 1 and manifest["diagnostic_only"] is True and manifest["kind"] == "s07_bis_chunk_distribution_archive" and manifest["scope"] == probe.SCOPE, "invalid chunk archive identity")
    probe.require(manifest["build_manifest_sha256"] == BUILD_SHA and manifest["capture_manifest_sha256"] == CAPTURE_SHA, "archive changed the recorded experiment")
    probe.require(manifest["replay_inputs"] == replay_inputs(), "archive replay implementation changed")
    probe.require(set(manifest["files"]) == {"recorded-results.tar.xz"}, "unknown/missing archive files")
    for name, expected in manifest["files"].items():
        probe.require(identity((directory / name).read_bytes()) == expected, "archive bytes changed")
    members = read_members(directory / "recorded-results.tar.xz")
    probe.require(len(members) == manifest["archive_members"], "archive member count changed")
    probe.require(probe.runner.sha(members["build/manifest.json"]) == BUILD_SHA and probe.runner.sha(members["capture/manifest.json"]) == CAPTURE_SHA, "original manifest bytes changed")
    build = probe.strict_json_loads(members["build/manifest.json"])
    capture = probe.strict_json_loads(members["capture/manifest.json"])
    probe.require(validate_members(members, build, capture) == set(manifest["omitted_binaries"]), "omitted artifact set changed")
    probe.require(build["tool_sha256"] == capture["tool_sha256"] == probe.tool_files(), "recorded/current frozen validation helpers differ")
    for name, expected in build["tool_sha256"].items():
        probe.require(probe.runner.sha(members["build/tool-snapshot/" + name]) == expected, "helper source snapshot changed")
    for name, expected in build["source_sha256"].items():
        probe.require(probe.runner.sha(members["build/source/" + name]) == expected, "Rust source snapshot changed")
    observed_preflight = probe.strict_json_loads(gzip.decompress(members["build/allocation-preflight.stdout.gz"]))
    probe.require(probe.preflight(observed_preflight) == build["allocation_preflight"] == capture["allocation_preflight"], "exact allocator preflight differs")
    probe.require(capture["status"] == "complete" and capture["scope"] == probe.SCOPE and capture["build_manifest_sha256"] == BUILD_SHA and capture["artifacts"] == build["artifacts"], "capture provenance differs")
    reference = manifest["input_reference"]
    path = (ROOT / reference["path"]).resolve()
    probe.require(path.is_relative_to(ROOT) and identity(path.read_bytes()) == {key: reference[key] for key in ("bytes", "sha256")}, "shared census gzip changed")
    raw = gzip.decompress(path.read_bytes())
    probe.require(identity(raw) == {"bytes": reference["raw_bytes"], "sha256": reference["raw_sha256"]} == capture["inventory"]["input.ndjson"], "shared census raw bytes changed")
    census_raw = members["capture/census-manifest.json"]
    probe.require(probe.runner.sha(census_raw) == reference["census_manifest_sha256"] == capture["census_manifest_sha256"], "census origin manifest changed")
    census = probe.strict_json_loads(census_raw)
    expected = probe.expected_from_raw(raw, validate_owners=True)
    probe.require(expected == capture["expected"] and expected["input_sha256"] == census["raw_sha256"] and expected["files"] == census["child"]["files"], "recorded input distribution differs")
    probe.integer(capture["sweeps"], 1, 100)
    with tempfile.TemporaryDirectory(prefix="s07-chunk-replay-") as temporary:
        raw_directory = Path(temporary)
        (raw_directory / "raw").mkdir()
        for name, data in members.items():
            if name.startswith("capture/raw/"):
                (raw_directory / name.removeprefix("capture/")).write_bytes(data)
        probe.validate_observations(capture, raw_directory, expected)
    probe.require(probe.strict_json_loads(members["capture/report.json"]) == {key: value for key, value in capture.items() if key != "inventory"}, "report/manifest differ")
    return {"version": 1, "diagnostic_only": True, "scope": probe.SCOPE, "children": len(capture["observations"]),
        "warmups": sum(row["identity"]["warmup"] for row in capture["observations"]),
        "files": expected["files"], "backings": expected["backings"], "edges": expected["edges"],
        "build_manifest_sha256": BUILD_SHA, "capture_manifest_sha256": CAPTURE_SHA,
        "native_child_reexecuted": False, "archive_members_verified": len(members), "summary": capture["summary"]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("package", "replay"))
    parser.add_argument("--directory", type=Path, default=HERE / "results/2026-09-08-chunks")
    parser.add_argument("--build", type=Path, default=ROOT / "target/s07-bis/chunk-pilot-build")
    parser.add_argument("--capture", type=Path, default=ROOT / "target/s07-bis/chunk-pilot-capture")
    args = parser.parse_args()
    result = package(args.build.resolve(), args.capture.resolve(), args.directory.resolve()) if args.command == "package" else replay(args.directory.resolve())
    print(json.dumps(result, indent=2, sort_keys=True, allow_nan=False))


if __name__ == "__main__":
    main()
