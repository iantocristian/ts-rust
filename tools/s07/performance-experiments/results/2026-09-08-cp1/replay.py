#!/usr/bin/env python3
"""Package/replay the CP1 core-read screen without executing captured binaries."""
import argparse
from contextlib import redirect_stderr
import hashlib
import importlib.util
import io
import json
from pathlib import Path, PurePosixPath
import sys
import re
import shlex
import tarfile
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
sys.path.insert(0, str(ROOT / "scripts"))
from s04_common import strict_json_loads
from s06_protocol import canonical
from s07_benchmark_graph import check_record, compare_rows
from s06_ownership import validate_output
from s07_ownership import COMMON, GROUPS, MODES, validate_manifest, publish_metrics

SPEC = importlib.util.spec_from_file_location("cp1_archive_runner", ROOT / "tools/s07/performance-experiments/runner.py")
runner = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(runner)

MANIFEST_SHAS = {
    "control": "124956f668540814b95e6f677bdeae98bb2cad4f7586d3cb87f869e5c5af118f",
    "candidate": "3a57976f667b9857891edbe9f76de5261c43480ac8c62d946b4545d9ee19d931",
}
E3_SHA = "75dde69056152a7d7e876c4e28f4200ca3be9f4f3b90be07f4ce7dc7dd35f77a"
E3_INVENTORY_SHA = "01ef62ff462331d94ea45cfac26c1bacc934f23dad3983790851a350804ced71"
BINDER_SHA = "4a778e65f27886a66c0b6bbd2bd956d41ef6c4ae8e3bbaa671a7235c0082b72d"
BLOCKED_BINDER_SHA = "ef3c5fdbf138ab11d4804ee21f2e56101448de1e6b37a79b4c5b3a306c6f8c3b"
BINDER_RAW_SHAS = {"report.json": "646abeedced80ebf0d2ceb91049b4df622f45a16118a0aa99a7be8adad40d0f9",
                   "requests.ndjson": "3322a8d2afbad685fce923143f9e914a70f0d82a9ea6772200e7ae86bb79ba8f"}
PROOF_LOGS = ("cp1-node-read-build.log", "cp1-node-read-graphs.log", "cp1-node-read-screen.log",
              "cp1-node-read-screen-replay.json", "cp1-binder-validation.log",
              "cp1-binder-validation-cache-access.log", "cp1-e3-validation.log")
POLICY = {
    "checkpoint": "CP1 exclusive-core node read",
    "diagnostic_only": True,
    "full_workload_files": 13094,
    "worker_counts": [1, 8],
    "pairs_per_metric_mode": 7,
    "warmups": 8,
    "samples": 56,
    "target_metrics": ["wall_time_ns"],
    "maximum_relative_mad": 0.05,
    "maximum_timing_upper_95_ratio": 1.02,
    "maximum_memory_median_ratio": 1.02,
    "targeted_win_maximum_median_ratio": 0.95,
    "targeted_timing_win_upper_ratio_strictly_below": 1.0,
    "infrastructure_exception_allowed": False,
    "final_go_relative_gates_established": False,
}


def require(value, message):
    if not value:
        raise ValueError(message)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def identity(raw):
    return {"sha256": sha(raw), "bytes": len(raw)}


def safe_name(name):
    require(type(name) is str and bool(name), "unsafe archive path")
    path = PurePosixPath(name)
    require(not path.is_absolute()
            and ".." not in path.parts and str(path) == name, "unsafe archive path")
    return name


def helper_files():
    # Capture the actually imported project-module closure, including helpers
    # outside runner.tool_fingerprint(). No module is executed from the archive.
    files = {Path(__file__).resolve(), HERE / "test_replay.py",
             ROOT / "tools/s07/performance-experiments/runner.py"}
    # Some capture helpers run as child scripts and never enter sys.modules.
    # Include both the declared capture tools and the imported replay closure.
    files.update(Path(path).resolve() for path in runner.tool_fingerprint())
    for module in tuple(sys.modules.values()):
        filename = getattr(module, "__file__", None)
        if filename:
            path = Path(filename).resolve()
            if path.suffix == ".py" and path.is_relative_to(ROOT / "scripts"):
                files.add(path)
    return {str(path.relative_to(ROOT)): sha(path.read_bytes()) for path in sorted(files)}


def prepare(directory):
    path = directory / "prepared.json"
    require(not path.exists(), "preparation already exists; do not replace its declared policy")
    directory.mkdir(parents=True, exist_ok=True)
    helpers = helper_files()
    for name, expected in helpers.items():
        raw = (ROOT / name).read_bytes()
        require(sha(raw) == expected, "helper changed during preparation")
        output = directory / "prepared-tools" / name
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_bytes(raw)
    value = {"version": 1, "policy": POLICY, "source_manifest_sha256": MANIFEST_SHAS,
             "helpers": helpers, "capture_tools": runner.tool_fingerprint()}
    runner.write_json(path, value)
    return {"prepared_sha256": sha(path.read_bytes()), "helper_count": len(helpers)}


def check_policy(summary, decision):
    require(decision in {"keep", "reject"}, "a concrete keep/reject decision is required")
    require(set(summary["modes"]) == {"1", "8"}, "screen omitted a worker mode")
    admissible, win, noisy = True, False, False
    for mode in summary["modes"].values():
        require(set(mode) == set(runner.METRICS), "screen omitted a metric")
        for metric, result in mode.items():
            require(result["samples_per_variant"] == 7, "screen changed sample count")
            quiet = result["control_relative_mad"] <= 0.05 and result["candidate_relative_mad"] <= 0.05
            timing = metric == "wall_time_ns"
            nonregressing = result["bootstrap"]["upper"] <= 1.02 if timing else result["ratio"] <= 1.02
            admissible &= quiet and nonregressing
            noisy |= not quiet
            win |= timing and quiet and result["ratio"] <= 0.95 and result["bootstrap"]["upper"] < 1.0
    status = ("eligible_for_review" if admissible and win else "no_demonstrated_win" if admissible
              else "inconclusive" if noisy else "regressing_or_uncertain")
    require(summary["screening_status"] == status
            and summary["nonregression_conditions_met"] is bool(admissible)
            and summary["targeted_pipeline_win"] is bool(win), "CP1 thresholds disagree with screen summary")
    require(decision != "keep" or admissible and win,
            "CP1 cannot keep a failed/neutral screen through the infrastructure exception")
    return {"screening_status": status, "screen_requires_rejection": not (admissible and win),
            "production_decision": decision, "infrastructure_exception_used": False}


def check_builds(members):
    builds = {}
    expected_members = set()
    for role, expected_sha in MANIFEST_SHAS.items():
        raw = members[role + "/manifest.json"]
        require(sha(raw) == expected_sha, "frozen variant manifest changed")
        build = strict_json_loads(raw)
        require(build["diagnostic_only"] is True and type(build["version"]) is int and build["version"] == 1,
                "invalid frozen variant identity")
        artifacts = build["artifacts"]
        require(set(artifacts) == {"normal", "allocation", "go"}, "missing binary identity")
        omitted = {record["path"] for record in artifacts.values()}
        require(len(omitted) == 3 and artifacts["normal"]["sha256"] != artifacts["allocation"]["sha256"],
                "binary modes collapsed")
        expected_members.add(role + "/manifest.json")
        for name, record in build["inventory"].items():
            safe_name(name)
            if name not in omitted:
                member = role + "/" + name
                require(identity(members[member]) == record, "frozen source/input/log member changed: " + member)
                expected_members.add(member)
        source = build["source_fingerprint"]
        require(sha(canonical(source["files"])) == source["sha256"], "source fingerprint changed")
        for name, expected in source["files"].items():
            require(sha(members[role + "/source/" + safe_name(name)]) == expected,
                    "source snapshot differs from its measured fingerprint")
        runner.validate_allocation_preflight(build["allocation_preflight"])
        require(strict_json_loads(members[role + "/evidence/allocation-preflight.json"]) == build["allocation_preflight"],
                "allocation preflight log differs from manifest")
        builds[role] = build
    require({name for name in members if name.startswith(("control/", "candidate/"))} == expected_members,
            "frozen build archive is incomplete or contains omitted binaries")
    control, candidate = builds["control"], builds["candidate"]
    require(candidate["control_manifest_sha256"] == MANIFEST_SHAS["control"], "candidate changed its declared control")
    require(candidate["target_metrics"] == POLICY["target_metrics"], "candidate changed its target metric")
    for key in ("expected_work", "rustc", "rust_profile"):
        require(control[key] == candidate[key], "variant workload/compiler/profile differ")
    require(control["artifacts"]["go"] == candidate["artifacts"]["go"], "variant oracle identities differ")
    require(control["expected_work"]["files"] == POLICY["full_workload_files"], "workload is a subset")
    # Frozen input paths are provenance, not an invitation to reopen workload files.
    frozen = strict_json_loads(members["candidate/source/data/s07/bindworkload-probes.json"])
    for role in builds:
        inputs = strict_json_loads(members[role + "/inputs.json"])
        require(len(inputs) == len(frozen["requests"]), "frozen transport omitted files")
        for recipe, request in zip(inputs, frozen["requests"], strict=True):
            require(set(recipe) == {"filename", "path", "local", "script_kind", "jsx", "force"}
                    and all(recipe[key] == request[key] for key in recipe if key != "local"),
                    "transport changed frozen order/options")
    return builds, frozen


def check_graph_inventory(members):
    expected = {"graphs/report.json"} | {
        f"graphs/workers-{workers}/{name}" for workers in (1, 8)
        for name in ("oracle.ndjson", "rust.ndjson", "oracle.stderr", "rust.stderr",
                     "failures.ndjson", "binding-paths.stdout", "binding-paths.stderr")}
    require({name for name in members if name.startswith("graphs/")} == expected,
            "graph capture is incomplete or contains unexpected streams")


def check_graphs(members, builds, frozen, graph_sha):
    check_graph_inventory(members)
    raw = members["graphs/report.json"]
    require(sha(raw) == graph_sha, "graph prerequisite identity changed")
    graph = strict_json_loads(raw)
    expected = builds["control"]["expected_work"]
    proof = []
    with tempfile.TemporaryDirectory(prefix="s07-cp1-graph-replay-") as temporary:
        scratch = Path(temporary)
        path = scratch / "report.json"
        path.write_bytes(raw)
        runner.validate_graph_report(path, builds["control"], builds["candidate"],
                                     MANIFEST_SHAS["control"], MANIFEST_SHAS["candidate"])
        for position, (workers, recorded) in enumerate(zip((1, 8), graph["runs"], strict=True)):
            prefix = f"graphs/workers-{workers}/"
            rows = {}
            for runtime in ("oracle", "rust"):
                rows[runtime] = [strict_json_loads(line) for line in members[prefix + runtime + ".ndjson"].splitlines()]
                require(len(rows[runtime]) == len(frozen["requests"]), "graph stream omitted a file")
                for index, (row, request) in enumerate(zip(rows[runtime], frozen["requests"], strict=True)):
                    check_record(row, index, workers, request)
            result = compare_rows(rows["oracle"], rows["rust"], frozen["requests"], frozen, workers, scratch)
            result["first_mismatch_witness"] = None
            require(result == recorded and (scratch / "failures.ndjson").read_bytes() == members[prefix + "failures.ndjson"],
                    "graph result does not replay from every per-file observation")
            paths = runner.validate_paths(strict_json_loads(members[prefix + "binding-paths.stdout"]), expected, workers)
            require(paths == graph["binding_paths"][position], "binding-path report differs from raw output")
            proof.append({"workers": workers, "files_per_runtime": len(rows["rust"]),
                          "parity": result["parity"], "comparison_sha256": sha(canonical(result))})
    return graph, proof


def check_screen(members, builds, graph_sha, screen_sha, decision):
    raw = members["screen/report.json"]
    require(sha(raw) == screen_sha, "screen report identity changed")
    screen = strict_json_loads(raw)
    capture = strict_json_loads(members["screen/capture.json"])
    require(screen.get("status") == "complete" and screen.get("diagnostic_only") is True and "metrics" not in screen
            and screen["manifest_sha256"] == MANIFEST_SHAS
            and all(screen.get(key) == value for key, value in capture.items()), "screen capture identity changed")
    require(screen["graph_report"]["sha256"] == graph_sha, "screen uses another graph prerequisite")
    expected = builds["control"]["expected_work"]
    raw_inventory = {name.removeprefix("screen/"): identity(raw) for name, raw in members.items()
                     if name.startswith(("screen/warmup-raw/", "screen/sample-raw/"))}
    require(raw_inventory == screen["raw_capture_inventory"], "raw screen output inventory changed")
    ledgers = {}
    for name, warmup in (("samples", False), ("warmups", True)):
        ledger = members[f"screen/{name}.ndjson"]
        require(sha(ledger) == screen[name + "_sha256"], "screen observation ledger changed")
        rows = [strict_json_loads(line) for line in ledger.splitlines()]
        runner.validate_rows(rows, expected, warmup)
        for row in rows:
            stem = "{workers}-{allocation}-{index}-{variant}".format(**row)
            folder = "warmup-raw" if warmup else "sample-raw"
            require(strict_json_loads(members[f"screen/{folder}/{stem}.stdout"]) == row["sample"],
                    "screen observation differs from raw child stdout")
        ledgers[name] = rows
    require(len(ledgers["warmups"]) == 8 and len(ledgers["samples"]) == screen["samples"] == 56,
            "screen omitted or added observations")
    summary = runner.summarize(ledgers["samples"], expected, builds["candidate"]["target_metrics"])
    require(all(screen.get(key) == value for key, value in summary.items()), "screen summary changed")
    return screen, {**check_policy(summary, decision), "samples": 56, "warmups": 8,
                    "summary_sha256": sha(canonical(summary)), "summary": summary}


def check_e3(members):
    raw = members["e3/evidence.json"]
    require(sha(raw) == E3_SHA, "E3 evidence identity changed")
    evidence = strict_json_loads(raw)
    require(evidence["exit_code"] == 0 and evidence["valid_capture"] is True,
            "E3 is not a completed valid capture")
    for label in ("stdout", "stderr"):
        require(members["e3/" + label] == evidence[label].encode()
                and sha(members["e3/" + label]) == evidence[label + "_sha256"], "E3 raw output changed")
    inventory = validate_manifest(strict_json_loads(members["e3/ownership-cases.json"]))
    require(sha(members["e3/ownership-cases.json"]) == E3_INVENTORY_SHA, "exact CP1 ownership inventory changed")
    suites = {**inventory["common"], **inventory["groups"]}
    require(sum(len(suite["cases"]) for suite in suites.values()) == 29, "E3 inventory omitted CP1 cases")
    # The raw named-test output must cover every exact suite in all modes;
    # aggregate booleans and a successful producer exit are insufficient.
    text = evidence["stderr"]
    starts = list(re.finditer(r"^\+ (.+)$", text, re.MULTILINE))
    modes = {mode: {} for mode in MODES}
    for position, match in enumerate(starts):
        args = shlex.split(match.group(1))
        selected = [name for name, suite in suites.items() if suite["package"] in args and suite["filter"] in args]
        if not selected:
            continue
        require(len(selected) == 1, "ambiguous E3 ownership command")
        name = selected[0]
        mode = "miri" if "miri" in args else "address_sanitizer" if "-Zbuild-std" in args else "release" if "--release" in args else "debug"
        require(name not in modes[mode], "duplicate E3 suite/mode")
        stop = starts[position + 1].start() if position + 1 < len(starts) else len(text)
        with redirect_stderr(io.StringIO()):
            validate_output(text[match.end():stop].encode(), suites[name]["cases"], mode, "archived CP1 ownership")
        modes[mode][name] = True
    measured = {"metrics": {}}
    publish_metrics(measured, modes, inventory)
    original = strict_json_loads(evidence["stdout"])
    require(all(original["metrics"].get(key) == value for key, value in measured["metrics"].items())
            and measured["metrics"]["program_ownership_tests"] == 29
            and all(measured["metrics"][name] for name in GROUPS), "E3 named outcomes differ from metrics")
    return {"evidence_sha256": E3_SHA, "suites_per_mode": len(suites), "modes": sorted(modes),
            "distinct_cases": 29, "successful_named_test_observations": 116,
            "metrics_sha256": sha(canonical(measured["metrics"])),
            "scope": "Recorded S07 named observations only; not a new instrumentation run or full future E3 coverage"}


def check_binder_identity(members):
    for name, expected in BINDER_RAW_SHAS.items():
        require(sha(members["binder/" + name]) == expected, "binder raw capture identity changed: " + name)


def check_binder(members, builds):
    check_binder_identity(members)
    raw = members["binder/evidence.json"]
    require(sha(raw) == BINDER_SHA, "binder evidence identity changed")
    evidence = strict_json_loads(raw)
    require(evidence["exit_code"] == 0 and evidence["valid_capture"] is True, "binder capture failed")
    for label in ("stdout", "stderr"):
        require(sha(evidence[label].encode()) == evidence[label + "_sha256"], "binder evidence output changed")
    report = strict_json_loads(members["binder/report.json"])
    require(report["source_stable"] is True, "binder source changed during capture")
    source = builds["candidate"]["source_fingerprint"]["files"]
    for name, expected in report["production_inputs"].items():
        if name in source:
            require(source[name] == expected, "binder production source differs from frozen candidate: " + name)
        else:
            require(sha(members["binder/source/" + safe_name(name)]) == expected,
                    "additional binder source snapshot changed: " + name)
    for name in ("oracle.stderr", "rust.stderr"):
        require("binder/" + name in members, "binder capture omitted raw child stderr")
    rows = [strict_json_loads(line) for line in members["binder/requests.ndjson"].splitlines()]
    require(len(rows) == report["requests"] == 22361 and len({row["id"] for row in rows}) == len(rows),
            "binder request observations are missing or duplicated")
    primary, reached = {}, {"oracle": 0, "rust": 0}
    for index, row in enumerate(rows):
        require(row["equal"] is True and row["first_difference"] is None, "binder request failed")
        if index < 22343:
            require(type(row["primary"]) is str, "primary binder observation lost its row")
            primary[row["primary"]] = primary.get(row["primary"], True) and row["equal"]
            for runtime in reached:
                reached[runtime] += any(stage["stage"] == "bind" for stage in row["stages"][runtime])
        else:
            require(row["primary"] is None, "supplemental observation became primary")
    require(len(primary) == report["primary_rows"] == report["passed_rows"] == 12829
            and report["primary_requests"] == 22343 and primary == report["tests"]
            and reached == report["reached_bind"] == {"oracle": 22343, "rust": 22343}
            and report["supplemental"] == {"requests": 18, "passed": 18, "parity": 1.0}
            and report["failed_requests"] == 0 and report["parity"] == 1.0
            and members["binder/failures.ndjson"] == b"", "binder aggregate differs from recorded observations")
    stdout = strict_json_loads(evidence["stdout"])
    require(stdout["tests"] == {name: "pass" for name in primary}, "binder evidence differs from observed rows")
    blocked_raw = members["binder/blocked-attempt-evidence.json"]
    require(sha(blocked_raw) == BLOCKED_BINDER_SHA, "original blocked attempt changed")
    blocked = strict_json_loads(blocked_raw)
    require(blocked["exit_code"] != 0 and blocked["valid_capture"] is False
            and "operation not permitted" in blocked["stderr"].lower(), "blocked attempt was not the recorded environment failure")
    return {"evidence_sha256": BINDER_SHA, "production_inputs_verified": len(report["production_inputs"]),
            "primary_requests": 22343, "primary_rows": len(primary), "supplemental_requests": 18,
            "reached_bind": reached, "blocked_attempt_evidence_sha256": BLOCKED_BINDER_SHA,
            "scope": "Replays per-request comparison observations and row membership; original full binder graph streams are not archived"}


def check_proofs(members, builds):
    for name in PROOF_LOGS:
        require("proofs/" + name in members, "required validation/build log is absent")
    receipt = strict_json_loads(members["generated-code/receipt.json"])
    expected = {"control-bind-node": "control", "candidate-bind-node": "candidate", "control-ast-view-node": "control"}
    require(len(receipt["records"]) == 3 and {row["name"] for row in receipt["records"]} == set(expected),
            "generated-code inspection inventory changed")
    for row in receipt["records"]:
        require(row["returncode"] == 0
                and row["binary_sha256"] == builds[expected[row["name"]]]["artifacts"]["normal"]["sha256"],
                "generated-code inspection used another binary or failed")
        for label in ("stdout", "stderr"):
            require(sha(members["generated-code/" + row["name"] + "." + label]) == row[label + "_sha256"],
                    "generated-code raw output changed")


def read_archive(directory):
    manifest = strict_json_loads((directory / "manifest.json").read_bytes())
    require(type(manifest["version"]) is int and manifest["version"] == 1
            and manifest["kind"] == "s07_bis_cp1_core_read_archive" and manifest["policy"] == POLICY
            and manifest["source_manifest_sha256"] == MANIFEST_SHAS, "archive identity/policy changed")
    path = directory / safe_name(manifest["archive"]["path"])
    require(identity(path.read_bytes()) == {key: manifest["archive"][key] for key in ("bytes", "sha256")}, "archive bytes changed")
    members = {}
    with tarfile.open(path, "r:xz") as stream:
        for entry in stream:
            name = safe_name(entry.name)
            require(entry.isfile() and name not in members and name in manifest["members"], "invalid/duplicate/extra archive member")
            raw = stream.extractfile(entry).read()
            require(identity(raw) == manifest["members"][name], "archive member changed: " + name)
            members[name] = raw
    require(set(members) == set(manifest["members"]) and len(members) == manifest["member_count"]
            and sum(len(raw) for raw in members.values()) == manifest["uncompressed_bytes"], "archive member inventory incomplete")
    return manifest, members


def replay(directory):
    manifest, members = read_archive(directory)
    preparation = strict_json_loads(members["prepared.json"])
    require(type(preparation["version"]) is int and preparation["version"] == 1
            and preparation["policy"] == POLICY and preparation["source_manifest_sha256"] == MANIFEST_SHAS,
            "predeclared policy/variant identities changed")
    require(preparation["helpers"] == helper_files(), "archive replay implementation/helper closure changed")
    for name, expected in preparation["helpers"].items():
        require(sha(members["tools/" + safe_name(name)]) == expected, "prepared helper snapshot changed")
    builds, frozen = check_builds(members)
    graph, graph_proof = check_graphs(members, builds, frozen, manifest["graph_report_sha256"])
    screen, screen_proof = check_screen(members, builds, manifest["graph_report_sha256"],
                                        manifest["screen_report_sha256"], manifest["production_decision"])
    require(manifest["screening_status"] == screen_proof["screening_status"], "archive decision summary changed")
    e3_proof = check_e3(members)
    binder_proof = check_binder(members, builds)
    check_proofs(members, builds)
    for record in (graph, screen):
        require(record["tool_fingerprint"] == preparation["capture_tools"], "capture helper identity differs from preparation")
        for path, expected in record["tool_fingerprint"].items():
            relative = Path(path).relative_to(ROOT).as_posix()
            require(sha(members["tools/" + relative]) == expected, "capture helper snapshot changed")
    return {"version": 1, "diagnostic_only": True, "archive_sha256": manifest["archive"]["sha256"],
            "members_verified": len(members), "source_manifest_sha256": MANIFEST_SHAS,
            "graph_replay": graph_proof, "screen_replay": screen_proof, "e3_replay": e3_proof,
            "binder_replay": binder_proof,
            "native_child_reexecuted": False, "final_go_relative_gates_established": False,
            "scope": "recorded-observation replay, complete nonbinary build closure and helper provenance"}


def package(args):
    directory = args.directory.resolve()
    require(not (directory / "manifest.json").exists() and not (directory / "recorded-results.tar.xz").exists(),
            "CP1 package already exists; do not overwrite captured results")
    preparation_raw = (directory / "prepared.json").read_bytes()
    preparation = strict_json_loads(preparation_raw)
    require(preparation["policy"] == POLICY and preparation["source_manifest_sha256"] == MANIFEST_SHAS
            and preparation["helpers"] == helper_files(), "preparation or replay implementation changed")
    require(runner.digest(args.graphs / "report.json") == args.graph_sha
            and runner.digest(args.screen / "report.json") == args.screen_sha, "externally recorded capture identity changed")
    # Read-only verification of the immutable binaries is possible at packaging;
    # offline replay deliberately omits their bytes and never executes them.
    builds = {role: runner.validate_bundle(path, MANIFEST_SHAS[role])
              for role, path in (("control", args.control), ("candidate", args.candidate))}
    runner.verify_report(args.screen)
    members = {"prepared.json": preparation_raw}
    for role, directory_path in (("control", args.control), ("candidate", args.candidate)):
        build = builds[role]
        omitted = {value["path"] for value in build["artifacts"].values()}
        for name in ("manifest.json", *build["inventory"]):
            if name not in omitted:
                members[role + "/" + safe_name(name)] = (directory_path / name).read_bytes()
    for role, directory_path in (("graphs", args.graphs), ("screen", args.screen)):
        for path in sorted(directory_path.rglob("*")):
            require(not path.is_symlink(), "capture contains symlink")
            if path.is_file():
                members[role + "/" + safe_name(path.relative_to(directory_path).as_posix())] = path.read_bytes()
    for name, expected in preparation["helpers"].items():
        raw = (directory / "prepared-tools" / name).read_bytes()
        require(sha(raw) == expected, "prepared helper bytes changed")
        members["tools/" + safe_name(name)] = raw
    for argument in args.proof:
        name, path = argument.split("=", 1)
        member = "proofs/" + safe_name(name)
        require(member not in members, "duplicate proof name")
        members[member] = Path(path).read_bytes()
    for name in PROOF_LOGS:
        require("proofs/" + name not in members, "automatic proof log cannot be replaced")
        members["proofs/" + name] = (ROOT / "target/s07-bis" / name).read_bytes()
    for name in ("receipt.json", *(stem + "." + extension for stem in
            ("control-bind-node", "candidate-bind-node", "control-ast-view-node") for extension in ("stdout", "stderr"))):
        members["generated-code/" + name] = (directory / "generated-code" / name).read_bytes()
    evidence_raw = (ROOT / "status/evidence" / (E3_SHA + ".json")).read_bytes()
    evidence = strict_json_loads(evidence_raw)
    members.update({"e3/evidence.json": evidence_raw, "e3/stdout": evidence["stdout"].encode(),
                    "e3/stderr": evidence["stderr"].encode(),
                    "e3/ownership-cases.json": (ROOT / "data/s07/ownership-cases.json").read_bytes()})
    check_e3(members)
    for name in ("report.json", "requests.ndjson", "failures.ndjson", "oracle.stderr", "rust.stderr"):
        members["binder/" + name] = (ROOT / "target/s07-binder-reports" / name).read_bytes()
    for name, expected in (("evidence.json", BINDER_SHA), ("blocked-attempt-evidence.json", BLOCKED_BINDER_SHA)):
        members["binder/" + name] = (ROOT / "status/evidence" / (expected + ".json")).read_bytes()
    for name, expected in strict_json_loads(members["binder/report.json"])["production_inputs"].items():
        if name not in builds["candidate"]["source_fingerprint"]["files"]:
            raw = (ROOT / name).read_bytes()
            require(sha(raw) == expected, "additional binder source changed before packaging")
            members["binder/source/" + safe_name(name)] = raw
    check_binder(members, builds)
    check_proofs(members, builds)
    builds, frozen = check_builds(members)
    _, screen_proof = check_screen(members, builds, args.graph_sha, args.screen_sha, args.decision)
    archive = directory / "recorded-results.tar.xz"
    with tarfile.open(archive, "w:xz", format=tarfile.PAX_FORMAT) as stream:
        for name, raw in sorted(members.items()):
            entry = tarfile.TarInfo(name)
            entry.size, entry.mode = len(raw), 0o444
            stream.addfile(entry, io.BytesIO(raw))
    manifest = {"version": 1, "kind": "s07_bis_cp1_core_read_archive", "policy": POLICY,
                "production_decision": args.decision, "source_manifest_sha256": MANIFEST_SHAS,
                "graph_report_sha256": args.graph_sha, "screen_report_sha256": args.screen_sha,
                "archive": {"path": archive.name, **identity(archive.read_bytes())},
                "members": {name: identity(raw) for name, raw in sorted(members.items())},
                "member_count": len(members), "uncompressed_bytes": sum(map(len, members.values())),
                "screening_status": screen_proof["screening_status"],
                "omitted": "Only the three executable artifacts per variant; original workload bytes were never copied into these bundles.",
                "scope": "No captured executable is run. Proof logs are retained verbatim; graph and screen outcomes are replayed."}
    runner.write_json(directory / "manifest.json", manifest)
    result = replay(directory)
    runner.write_json(directory / "replay.json", result)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("prepare", "package", "replay"))
    parser.add_argument("--directory", type=Path, default=HERE)
    parser.add_argument("--control", type=Path, default=ROOT / "target/s07-bis/a0b-candidate")
    parser.add_argument("--candidate", type=Path, default=ROOT / "target/s07-bis/cp1-node-read-candidate")
    parser.add_argument("--graphs", type=Path, default=ROOT / "target/s07-bis/cp1-node-read-graphs")
    parser.add_argument("--screen", type=Path, default=ROOT / "target/s07-bis/cp1-node-read-screen")
    parser.add_argument("--graph-sha")
    parser.add_argument("--screen-sha")
    parser.add_argument("--decision", choices=("keep", "reject"))
    parser.add_argument("--proof", action="append", default=[], metavar="ARCHIVE_NAME=PATH")
    args = parser.parse_args()
    result = prepare(args.directory) if args.command == "prepare" else package(args) if args.command == "package" else replay(args.directory)
    print(json.dumps(result, indent=2, sort_keys=True, allow_nan=False))


if __name__ == "__main__":
    main()
