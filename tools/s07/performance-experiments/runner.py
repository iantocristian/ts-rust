#!/usr/bin/env python3
"""Immutable S07-bis control/candidate diagnostics; never emit tracker metrics."""
import argparse
import fcntl
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[3]
TOOLS = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT / "scripts"))
from s04_common import command, strict_json_loads
from s07_benchmark import (CACHE, build_rust, cargo_configuration, native_environment,
                           rust_native_toolchain, sha, source_fingerprint)
from s07_benchmark_graph import requests_from_frozen
from s07_benchmark_inputs import loaded_input_digest
from s07_benchmark_measure import (host_info, reject_concurrent_builds, validate_sample,
                                  validate_allocation_preflight, allocation_preflight)
from s07_benchmark_stats import ratio_summary

METRICS = ("wall_time_ns", "peak_rss_bytes", "allocated_bytes")
PAIRS = 7


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n")


def digest(path):
    return sha(Path(path).read_bytes())


def copy_file(source, destination):
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, destination)


def inventory(directory):
    return {str(path.relative_to(directory)): {"sha256": digest(path), "bytes": path.stat().st_size}
            for path in sorted(directory.rglob("*")) if path.is_file() and path != directory / "manifest.json"}


def seal(directory, manifest):
    manifest["inventory"] = inventory(directory)
    write_json(directory / "manifest.json", manifest)
    for path in sorted(directory.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() or path.parent == directory / "artifacts" else 0o444)
    directory.chmod(0o555)
    return digest(directory / "manifest.json")


def validate_bundle(directory, expected_sha):
    """An externally recorded manifest digest prevents rewriting identity in place."""
    directory = Path(directory).resolve()
    manifest_path = directory / "manifest.json"
    if digest(manifest_path) != expected_sha:
        raise ValueError("experiment manifest changed")
    manifest = strict_json_loads(manifest_path.read_bytes())
    if manifest.get("version") != 1 or manifest.get("diagnostic_only") is not True:
        raise ValueError("unsupported experiment manifest")
    for path in (directory, *directory.rglob("*")):
        if path.is_symlink() or path.stat().st_mode & 0o222:
            raise ValueError("experiment bundle is not immutable: " + str(path))
    if inventory(directory) != manifest["inventory"]:
        raise ValueError("experiment artifact/source/input inventory changed")
    artifacts = manifest["artifacts"]
    if set(artifacts) != {"normal", "allocation", "go"}:
        raise ValueError("missing normal, allocation or Go artifact")
    for artifact in artifacts.values():
        path = directory / artifact["path"]
        if not path.resolve().is_relative_to(directory) or digest(path) != artifact["sha256"]:
            raise ValueError("experiment binary overwritten or swapped")
        if not os.access(path, os.X_OK):
            raise ValueError("experiment binary is not executable")
    if artifacts["normal"]["sha256"] == artifacts["allocation"]["sha256"]:
        raise ValueError("normal and allocation builds cannot be the same executable")
    sources = manifest["source_fingerprint"]
    if sha(json.dumps(sources["files"], sort_keys=True, separators=(",", ":")).encode()) != sources["sha256"]:
        raise ValueError("invalid source fingerprint")
    for name, expected in sources["files"].items():
        path = directory / "source" / name
        if not path.resolve().is_relative_to(directory / "source") or digest(path) != expected:
            raise ValueError("snapshot does not match measured source")
    validate_allocation_preflight(manifest["allocation_preflight"])
    return manifest


def validate_inputs(directory, expected):
    # Every child also independently reports the digest of what it actually read.
    _, recipes = requests_from_frozen(Path(directory) / "inputs.json")
    if loaded_input_digest(recipes) != expected["loaded_input_sha256"]:
        raise ValueError("frozen workload bytes/options changed")


def runtime_libraries(binary, destination):
    if sys.platform == "darwin":
        raw = command(["otool", "-L", str(binary)], cwd=ROOT)
        libraries = [line.strip().split(" (", 1)[0] for line in raw.decode().splitlines()[1:]]
        if any(not item.startswith(("/usr/lib/", "/System/Library/")) for item in libraries):
            raise ValueError("non-system runtime dependency must be bundled explicitly")
    elif sys.platform == "linux":
        # The pinned builds link dependencies statically apart from system libc.
        result = subprocess.run(["ldd", str(binary)], cwd=ROOT, stdout=subprocess.PIPE,
                                stderr=subprocess.PIPE, check=False)
        raw = result.stdout + result.stderr
        if result.returncode and not (result.returncode == 1 and b"not a dynamic executable" in raw):
            raise ValueError("cannot inspect native runtime dependencies")
        libraries = [word for word in raw.decode().split() if word.startswith("/")]
        if any(not item.startswith(("/lib/", "/lib64/", "/usr/lib/", "/usr/lib64/")) for item in libraries):
            raise ValueError("non-system runtime dependency must be bundled explicitly")
    else:
        raise ValueError("unsupported native diagnostic host")
    destination.write_bytes(raw)


def snapshot_sources(directory, fingerprint):
    for name, expected in fingerprint["files"].items():
        destination = directory / "source" / name
        copy_file(ROOT / name, destination)
        if digest(destination) != expected:
            raise ValueError("source changed while freezing build")


def freeze_control(output, baseline, graph):
    from s07_benchmark_report import read_capture
    report, _ = read_capture(baseline, graph)
    requests, recipes = requests_from_frozen(CACHE / "s07-benchmark/inputs.json")
    if loaded_input_digest(recipes) != report["expected_work"]["loaded_input_sha256"]:
        raise ValueError("baseline transport no longer names validated input bytes")
    output.mkdir(parents=True, exist_ok=False)
    (output / "artifacts").mkdir()
    (output / "evidence").mkdir()
    artifacts = {}
    for role, name, key in (("normal", "rust-benchmark", "rust"),
                            ("allocation", "rust-benchmark-allocation", "rust_allocation"),
                            ("go", "go-benchmark", "go")):
        destination = output / "artifacts" / name
        copy_file(CACHE / "s07-benchmark" / name, destination)
        if digest(destination) != report["binaries"][key]:
            raise ValueError("baseline executable changed during freeze")
        artifacts[role] = {"path": str(destination.relative_to(output)), "sha256": digest(destination)}
        runtime_libraries(destination, output / "evidence" / (role + "-runtime-libraries.txt"))
    for name in ("report.json", "samples.ndjson", "capture.json"):
        copy_file(baseline / name, output / "evidence" / ("baseline-" + name))
    copy_file(graph, output / "evidence/graph-report.json")
    snapshot_sources(output, report["source_fingerprint"])
    frozen = []
    for index, request in enumerate(requests):
        local = output / "workload" / str(index)
        copy_file(request["local"], local)
        if digest(local) != recipes[index]["source_sha256"]:
            raise ValueError("baseline input changed during freeze")
        frozen.append({**request, "local": str(local)})
    write_json(output / "inputs.json", frozen)
    if source_fingerprint() != report["source_fingerprint"] or cargo_configuration() != report["cargo_configuration"]:
        raise ValueError("baseline source/configuration changed during freeze")
    keys = ("source_fingerprint", "cargo_configuration", "expected_work", "rustc", "go_version",
            "rust_profile", "allocation_preflight", "measurement_domains")
    manifest = {key: report[key] for key in keys}
    manifest.update(version=1, kind="control", diagnostic_only=True, label="s07-published-binding",
                    artifacts=artifacts, original_capture_revision=report["revision"],
                    revision=command(["git", "rev-parse", "HEAD"], cwd=ROOT).decode().strip(),
                    frozen_at_unix=int(time.time()))
    return seal(output, manifest)


def build_candidate(output, control, control_sha, label, hypothesis, targets):
    baseline = validate_bundle(control, control_sha)
    validate_inputs(control, baseline["expected_work"])
    reject_concurrent_builds()
    before, configuration = source_fingerprint(), cargo_configuration()
    output.mkdir(parents=True, exist_ok=False)
    (output / "artifacts").mkdir()
    (output / "evidence").mkdir()
    preflight = allocation_preflight()
    write_json(output / "evidence/allocation-preflight.json", preflight)
    artifacts = {}
    # build_rust consumes Cargo's actual compiler-artifact path and enforces the
    # native profile. Copy immediately: shared helper destinations are mutable.
    for allocation, role in ((False, "normal"), (True, "allocation")):
        binary, env = build_rust(allocation)
        destination = output / "artifacts" / ("rust-benchmark-allocation" if allocation else "rust-benchmark")
        copy_file(binary, destination)
        artifacts[role] = {"path": str(destination.relative_to(output)), "sha256": digest(destination)}
        runtime_libraries(destination, output / "evidence" / (role + "-runtime-libraries.txt"))
        if source_fingerprint() != before or cargo_configuration() != configuration:
            raise ValueError("candidate source/configuration changed during build")
    destination = output / "artifacts/go-benchmark"
    copy_file(control / baseline["artifacts"]["go"]["path"], destination)
    artifacts["go"] = {"path": str(destination.relative_to(output)), "sha256": digest(destination)}
    copy_file(control / "inputs.json", output / "inputs.json")
    snapshot_sources(output, before)
    stable, _ = rust_native_toolchain(env)
    manifest = {key: baseline[key] for key in ("expected_work", "go_version", "rust_profile", "measurement_domains")}
    manifest.update(version=1, kind="candidate", diagnostic_only=True, label=label, hypothesis=hypothesis,
                    target_metrics=targets, control_manifest_sha256=control_sha,
                    source_fingerprint=before, cargo_configuration=configuration, artifacts=artifacts,
                    revision=command(["git", "rev-parse", "HEAD"], cwd=ROOT).decode().strip(),
                    rustc=command(["rustc", "+" + stable, "-Vv"], cwd=ROOT, env=env).decode(),
                    allocation_preflight=preflight,
                    frozen_at_unix=int(time.time()))
    if source_fingerprint() != before or cargo_configuration() != configuration:
        raise ValueError("candidate source/configuration changed while freezing")
    return seal(output, manifest)


def row_order(warmup=False):
    for workers in (1, 8):
        for allocation in (False, True):
            for index in range(1 if warmup else PAIRS):
                for variant in (("control", "candidate") if index % 2 == 0 else ("candidate", "control")):
                    yield {"workers": workers, "allocation": allocation, "index": index, "variant": variant}


def validate_rows(rows, expected, warmup=False):
    order = list(row_order(warmup))
    if len(rows) != len(order):
        raise ValueError("incomplete or extended diagnostic sample batch")
    for row, identity in zip(rows, order, strict=True):
        if set(row) != {*identity, "sample"} or {key: row[key] for key in identity} != identity:
            raise ValueError("duplicate, reordered or unknown sample identity")
        if any(type(row[key]) is not type(identity[key]) for key in identity):
            raise ValueError("invalid sample identity type")
        validate_sample(row["sample"], expected, row["workers"], "rust", row["allocation"])


def summarize(rows, expected, targets):
    validate_rows(rows, expected)
    summaries = {}
    admissible, win, noisy = True, False, False
    for workers in (1, 8):
        mode = {}
        for metric in METRICS:
            values = {}
            for variant in ("control", "candidate"):
                selected = [row["sample"] for row in rows if row["workers"] == workers
                            and row["variant"] == variant and row["allocation"] is (metric == "allocated_bytes")]
                values[variant] = [item[metric] if metric == "peak_rss_bytes" else item["report"][metric] for item in selected]
            timing = metric == "wall_time_ns"
            raw = ratio_summary(values["control"], values["candidate"], timing=timing)
            result = {key.replace("go_", "control_").replace("rust_", "candidate_"): value
                      for key, value in raw.items() if key not in {"stable", "needs_more"}}
            result["samples_per_variant"] = result.pop("samples_per_runtime")
            result["values"] = values
            result["ranges"] = {key: [min(value), max(value)] for key, value in values.items()}
            quiet = result["control_relative_mad"] <= 0.05 and result["candidate_relative_mad"] <= 0.05
            nonregressing = result["bootstrap"]["upper"] <= 1.02 if timing else result["ratio"] <= 1.02
            admissible &= quiet and nonregressing
            noisy |= not quiet
            win |= metric in targets and quiet and result["ratio"] <= 0.95 and (not timing or result["bootstrap"]["upper"] < 1.0)
            mode[metric] = result
        summaries[str(workers)] = mode
    status = "eligible_for_review" if admissible and win else "no_demonstrated_win" if admissible else "inconclusive" if noisy else "regressing_or_uncertain"
    return {"modes": summaries, "screening_status": status, "nonregression_conditions_met": bool(admissible),
            "targeted_pipeline_win": bool(win), "requires_separate_target_cost_and_semantic_review": True}


def tool_fingerprint():
    names = ("runner.py",)
    files = {str(TOOLS / name): digest(TOOLS / name) for name in names}
    files.update({str(ROOT / "scripts" / name): digest(ROOT / "scripts" / name) for name in
                  ("s04_common.py", "s07_benchmark.py", "s07_benchmark_child.py", "s07_benchmark_graph.py",
                   "s07_benchmark_inputs.py", "s07_benchmark_measure.py", "s07_benchmark_stats.py")})
    return files


def raw_inventory(directory):
    return {str(path.relative_to(directory)): {"sha256": digest(path), "bytes": path.stat().st_size}
            for folder in (directory / "warmup-raw", directory / "sample-raw")
            for path in sorted(folder.rglob("*")) if path.is_file()}


def validate_paths(value, expected, workers):
    fields = {"version", "workers", "files", "bound_in_place_files", "fallback_files", "loaded_input_sha256"}
    if type(value) is not dict or set(value) != fields:
        raise ValueError("malformed binding-path observations")
    if any(type(value[name]) is not int or value[name] < 0 for name in fields - {"loaded_input_sha256"}):
        raise ValueError("invalid binding-path counter")
    if (value["version"] != 1 or value["workers"] != workers or value["files"] != expected["files"]
            or value["bound_in_place_files"] + value["fallback_files"] != value["files"]
            or value["loaded_input_sha256"] != expected["loaded_input_sha256"]):
        raise ValueError("binding-path observations changed work or lost a file")
    return value


def validate_graph_report(path, control, candidate, control_sha, candidate_sha):
    from s07_benchmark_graph import validate_measurement_prerequisite
    report = strict_json_loads(Path(path).read_bytes())
    if (report.get("diagnostic_only") is not True or report.get("kind") != "full_graph_diagnostic"
            or report.get("manifest_sha256") != {"control": control_sha, "candidate": candidate_sha}):
        raise ValueError("graph observations do not identify these immutable variants")
    binaries = {"go": control["artifacts"]["go"]["sha256"], "rust": candidate["artifacts"]["normal"]["sha256"]}
    expected = validate_measurement_prerequisite(report, candidate["source_fingerprint"], binaries)
    if expected != candidate["expected_work"] or expected != control["expected_work"]:
        raise ValueError("graph observations changed work")
    paths = report.get("binding_paths")
    if type(paths) is not list or len(paths) != 2:
        raise ValueError("missing binding-path worker mode")
    for value, workers in zip(paths, (1, 8), strict=True):
        validate_paths(value, expected, workers)
    return report


def graphs(control, candidate, control_sha, candidate_sha, output):
    """Run the existing complete graph protocol with immutable executables."""
    from s07_benchmark_graph import read_reports, compare_rows, capture_first_witness
    from s06_protocol import canonical
    baseline = validate_bundle(control, control_sha)
    variant = validate_bundle(candidate, candidate_sha)
    if variant.get("control_manifest_sha256") != control_sha:
        raise ValueError("candidate was not declared against this control")
    validate_inputs(control, baseline["expected_work"])
    validate_inputs(candidate, variant["expected_work"])
    inputs = candidate / "inputs.json"
    requests, recipes = requests_from_frozen(inputs)
    frozen = strict_json_loads((ROOT / "data/s07/bindworkload-probes.json").read_bytes())
    if frozen["input_sha256"] != sha(canonical(recipes)):
        raise ValueError("graph input recipes changed")
    env = native_environment()
    go = control / baseline["artifacts"]["go"]["path"], env
    rust = candidate / variant["artifacts"]["normal"]["path"], env
    output.mkdir(parents=True, exist_ok=False)
    write_json(output / "report.json", {"version": 1, "diagnostic_only": True, "status": "in_progress"})
    runs, paths = [], []
    try:
        for workers in (1, 8):
            directory = output / ("workers-" + str(workers))
            # These are untimed correctness captures, not screen samples.
            oracle_rows = read_reports(*go, inputs, requests, workers, directory, "oracle")
            rust_rows = read_reports(*rust, inputs, requests, workers, directory, "rust")
            run = compare_rows(oracle_rows, rust_rows, recipes, frozen, workers, directory)
            run["first_mismatch_witness"] = capture_first_witness(run, (("oracle", go), ("rust", rust)),
                                                                inputs, (oracle_rows, rust_rows), directory)
            runs.append(run)
            result = subprocess.run([str(rust[0]), str(inputs), str(workers), "--binding-paths"], cwd=ROOT,
                                    env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False, timeout=600)
            (directory / "binding-paths.stdout").write_bytes(result.stdout)
            (directory / "binding-paths.stderr").write_bytes(result.stderr)
            if result.returncode:
                raise ValueError("binding-path child failed; raw stdout/stderr retained")
            paths.append(validate_paths(strict_json_loads(result.stdout), variant["expected_work"], workers))
        validate_bundle(control, control_sha)
        validate_bundle(candidate, candidate_sha)
        validate_inputs(candidate, variant["expected_work"])
        binaries = {"oracle": baseline["artifacts"]["go"]["sha256"], "rust": variant["artifacts"]["normal"]["sha256"]}
        report = {"version": 1, "kind": "full_graph_diagnostic", "diagnostic_only": True,
                  "diagnostic_subset": False, "source_stable": True, "files": len(recipes),
                  "manifest_sha256": {"control": control_sha, "candidate": candidate_sha},
                  "parity": min(run["parity"] for run in runs), "expected_scalars": frozen["expected_scalars"],
                  "input_sha256": frozen["input_sha256"], "options_sha256": frozen["options_sha256"],
                  "workload_sha256": frozen["workload_sha256"], "source_fingerprint": variant["source_fingerprint"],
                  "source_fingerprint_after": variant["source_fingerprint"], "binary_sha256": binaries,
                  "binary_sha256_after": binaries, "runs": runs, "binding_paths": paths,
                  "provenance_scope": "immutable build snapshots; current checkout may differ"}
        write_json(output / "report.json", report)
        return report
    except BaseException as error:
        write_json(output / "report.json", {"version": 1, "diagnostic_only": True, "status": "failed", "error": str(error)})
        raise


def capture_sample(bundle, manifest, identity, raw_directory):
    role = "allocation" if identity["allocation"] else "normal"
    binary = bundle / manifest["artifacts"][role]["path"]
    argv = [sys.executable, str(ROOT / "scripts/s07_benchmark_child.py"), str(binary),
            str(bundle / "inputs.json"), str(identity["workers"])]
    completed = subprocess.run(argv, cwd=ROOT, env=native_environment(), stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, check=False)
    name = "{workers}-{allocation}-{index}-{variant}".format(**identity)
    (raw_directory / (name + ".stdout")).write_bytes(completed.stdout)
    (raw_directory / (name + ".stderr")).write_bytes(completed.stderr)
    if completed.returncode:
        raise ValueError(f"sample {name} failed ({completed.returncode}); raw stdout/stderr retained")
    value = strict_json_loads(completed.stdout)
    return validate_sample(value, manifest["expected_work"], identity["workers"], "rust", identity["allocation"])


def screen(control, candidate, control_sha, candidate_sha, output, graph_report=None):
    bundles = {"control": control, "candidate": candidate}
    hashes = {"control": control_sha, "candidate": candidate_sha}
    manifests = {key: validate_bundle(path, hashes[key]) for key, path in bundles.items()}
    if manifests["candidate"].get("control_manifest_sha256") != control_sha:
        raise ValueError("candidate was not declared against this control")
    if manifests["candidate"]["expected_work"] != manifests["control"]["expected_work"]:
        raise ValueError("control and candidate workloads differ")
    if control == candidate or not manifests["candidate"].get("target_metrics"):
        raise ValueError("screen requires two separately declared variants")
    if any(manifests["candidate"][key] != manifests["control"][key] for key in ("rustc", "rust_profile")):
        raise ValueError("control and candidate must use the same Rust compiler/profile")
    graph_sha = None
    if graph_report is not None:
        validate_graph_report(graph_report, manifests["control"], manifests["candidate"], control_sha, candidate_sha)
        graph_sha = digest(graph_report)
    output.mkdir(parents=True, exist_ok=False)
    lock_path = CACHE / "s07-benchmark/measurement.lock"
    with lock_path.open("a+") as lock:
        try:
            fcntl.flock(lock.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as error:
            raise ValueError("another performance capture owns this host") from error
        current, configuration, tools = source_fingerprint(), cargo_configuration(), tool_fingerprint()
        metadata = {"version": 1, "diagnostic_only": True, "bundles": {key: str(path) for key, path in bundles.items()},
                    "manifest_sha256": hashes, "host": host_info(), "tool_fingerprint": tools,
                    "current_source_fingerprint": current, "cargo_configuration": configuration}
        metadata["graph_report"] = {"path": str(graph_report), "sha256": graph_sha} if graph_report else None
        write_json(output / "capture.json", metadata)
        try:
            for key, path in bundles.items():
                validate_inputs(path, manifests[key]["expected_work"])
            # Warmups remain separately retained; no sample selection or batch extension.
            all_rows = []
            for warmup in (True, False):
                raw = output / ("warmup-raw" if warmup else "sample-raw")
                raw.mkdir()
                with (output / ("warmups.ndjson" if warmup else "samples.ndjson")).open("w") as stream:
                    previous_mode = None
                    for identity in row_order(warmup):
                        mode = identity["workers"], identity["allocation"]
                        if mode != previous_mode:
                            for key, path in bundles.items():
                                validate_bundle(path, hashes[key])
                            previous_mode = mode
                        reject_concurrent_builds()
                        variant = identity["variant"]
                        row = {**identity, "sample": capture_sample(bundles[variant], manifests[variant], identity, raw)}
                        stream.write(json.dumps(row, separators=(",", ":"), allow_nan=False) + "\n")
                        stream.flush()
                        if not warmup:
                            all_rows.append(row)
                    for key, path in bundles.items():
                        validate_bundle(path, hashes[key])
            for key, path in bundles.items():
                validate_inputs(path, manifests[key]["expected_work"])
            if source_fingerprint() != current or cargo_configuration() != configuration or tool_fingerprint() != tools:
                raise ValueError("source/configuration/measurement helpers changed during screen")
            if graph_report is not None and digest(graph_report) != graph_sha:
                raise ValueError("graph prerequisite changed during screen")
            summary = summarize(all_rows, manifests["control"]["expected_work"], manifests["candidate"]["target_metrics"])
            report = {**metadata, **summary, "status": "complete", "samples": len(all_rows),
                      "samples_sha256": digest(output / "samples.ndjson"), "warmups_sha256": digest(output / "warmups.ndjson"),
                      "raw_capture_inventory": raw_inventory(output)}
            write_json(output / "report.json", report)
            return report
        except BaseException as error:
            write_json(output / "report.json", {**metadata, "status": "failed", "error": str(error)})
            raise


def verify_report(directory):
    report = strict_json_loads((directory / "report.json").read_bytes())
    if report.get("status") != "complete" or report.get("diagnostic_only") is not True or "metrics" in report:
        raise ValueError("incomplete or non-diagnostic experiment report")
    manifests = {key: validate_bundle(Path(path), report["manifest_sha256"][key]) for key, path in report["bundles"].items()}
    if manifests["candidate"].get("control_manifest_sha256") != report["manifest_sha256"]["control"]:
        raise ValueError("report control differs from candidate declaration")
    if manifests["candidate"]["expected_work"] != manifests["control"]["expected_work"]:
        raise ValueError("report variants have different workload obligations")
    if raw_inventory(directory) != report["raw_capture_inventory"]:
        raise ValueError("raw stdout/stderr inventory changed")
    graph = report.get("graph_report")
    if graph is not None:
        if digest(graph["path"]) != graph["sha256"]:
            raise ValueError("graph prerequisite changed")
        validate_graph_report(graph["path"], manifests["control"], manifests["candidate"],
                              report["manifest_sha256"]["control"], report["manifest_sha256"]["candidate"])
    rows = {}
    expected = manifests["control"]["expected_work"]
    for name in ("samples", "warmups"):
        path = directory / (name + ".ndjson")
        if digest(path) != report[name + "_sha256"]:
            raise ValueError("raw observations changed")
        rows[name] = [strict_json_loads(line) for line in path.read_bytes().splitlines()]
        validate_rows(rows[name], expected, warmup=name == "warmups")
        folder = directory / ("warmup-raw" if name == "warmups" else "sample-raw")
        for row in rows[name]:
            stem = "{workers}-{allocation}-{index}-{variant}".format(**row)
            if strict_json_loads((folder / (stem + ".stdout")).read_bytes()) != row["sample"]:
                raise ValueError("raw child output differs from observation")
    if report["samples"] != len(rows["samples"]):
        raise ValueError("sample count changed")
    summary = summarize(rows["samples"], expected, manifests["candidate"]["target_metrics"])
    if any(report.get(key) != value for key, value in summary.items()):
        raise ValueError("published summaries differ from complete raw samples")
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="operation", required=True)
    freeze = sub.add_parser("freeze-control")
    freeze.add_argument("--output", type=Path, required=True)
    freeze.add_argument("--baseline", type=Path, default=ROOT / "target/s07-benchmark")
    freeze.add_argument("--graph", type=Path, default=ROOT / "target/s07-bindworkload/report.json")
    build = sub.add_parser("build")
    capture = sub.add_parser("screen")
    graph = sub.add_parser("graphs")
    for child in (build, capture, graph):
        child.add_argument("--control", type=Path, required=True)
        child.add_argument("--control-sha", required=True)
        child.add_argument("--output", type=Path, required=True)
    build.add_argument("--label", required=True)
    build.add_argument("--hypothesis", required=True)
    build.add_argument("--target-metric", choices=METRICS, action="append", required=True)
    for child in (capture, graph):
        child.add_argument("--candidate", type=Path, required=True)
        child.add_argument("--candidate-sha", required=True)
    capture.add_argument("--graph-report", type=Path)
    verify = sub.add_parser("verify")
    verify.add_argument("directory", type=Path)
    args = parser.parse_args()
    for name, value in vars(args).items():
        if isinstance(value, Path):
            setattr(args, name, value.resolve())
    if args.operation == "freeze-control":
        result = {"manifest_sha256": freeze_control(args.output, args.baseline, args.graph)}
    elif args.operation == "build":
        result = {"manifest_sha256": build_candidate(args.output, args.control, args.control_sha,
                                                     args.label, args.hypothesis, sorted(set(args.target_metric)))}
    elif args.operation == "screen":
        result = screen(args.control, args.candidate, args.control_sha, args.candidate_sha, args.output, args.graph_report)
    elif args.operation == "graphs":
        report = graphs(args.control, args.candidate, args.control_sha, args.candidate_sha, args.output)
        result = {key: report[key] for key in ("files", "parity", "binding_paths")}
        result["report"] = str(args.output / "report.json")
    else:
        result = verify_report(args.directory)
    print(json.dumps(result, sort_keys=True, allow_nan=False))


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        print("S07-bis experiment failed: " + str(error), file=sys.stderr)
        raise SystemExit(1) from error
