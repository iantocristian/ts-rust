#!/usr/bin/env python3
"""Execute pinned baseline generation and preserve every requested source outcome.

This is a Go observation capture, not an E2 producer. --smoke is a fixed,
source-selected protocol trial; omitting it requests every frozen variant.
--review-capture verifies a completed full capture and lists unresolved outcomes.
"""
import argparse
from collections import Counter
import json
from pathlib import Path
import subprocess
import sys

from s04 import go_environment, same_json_value, verified_upstream
from s04_common import strict_json_loads
from s08_manifest import eligible
from s08_oracle import ROOT, canonical, digest
from s07_acceptance import load_partition, POLICY, OBSERVATIONS

BRIDGES = ROOT / "tools/s08/oracle"


def requests_from_subset(subset, smoke=False, case_id=None):
    rows = eligible(subset)
    tiers = {v["id"]:v["tier"] for v in load_partition(subset)["variants"]}
    if case_id is not None:
        rows = [(c, v) for c, v in rows if v["id"] == case_id]
        if len(rows) != 1:
            raise ValueError("case must name exactly one frozen eligible variant")
    if smoke:
        selectors = [lambda c, v: True,
                     lambda c, v: bool(v["options"].get("declaration") and v["options"].get("allowJs")),
                     lambda c, v: c["source"]["legacy_projection"] is not None,
                     lambda c, v: v["harness_options"]["CaptureSuggestions"],
                     lambda c, v: v["harness_options"]["NoTypesAndSymbols"]]
        selected = {next(v["id"] for c, v in rows if predicate(c, v)) for predicate in selectors}
        rows = [(c, v) for c, v in rows if v["id"] in selected]
    requests = []
    for case, variant in rows:
        name = Path(case["source"]["path"]).name
        extension = Path(name).suffix
        base = name[:-len(extension)]
        configured = variant["configured_name"]
        if not configured.startswith(base) or not configured.endswith(extension):
            raise ValueError("invalid configured filename")
        label = configured[len(base):-len(extension)]
        if label and (not label.startswith("(") or not label.endswith(")")):
            raise ValueError("invalid configuration label")
        requests.append({"id":variant["id"], "path":case["source"]["path"],
                         "acceptance_tier":tiers[variant["id"]],
                         "raw_sha256":case["source"]["raw_sha256"],
                         "loaded_sha256":case["source"]["loaded_sha256"],
                         "settings":case["source"]["configurations"][variant["configuration"]],
                         "configuration_name":label[1:-1] if label else "",
                         "configured_name":configured})
    return rows, requests


def replace_exact(source, before, after, count=1):
    if source.count(before) != count:
        raise ValueError(f"observation anchor drift: {before!r}")
    return source.replace(before, after)


def overlay_sources(upstream):
    harness = (upstream / "tsc/internal/testutil/harnessutil/harnessutil.go").read_text()
    harness = replace_exact(harness, "\terrors := postErrors\n",
                            "\tif S08ObserveDiagnostics != nil { S08ObserveDiagnostics(preErrors, postErrors) }\n\terrors := postErrors\n")
    walker = (upstream / "tsc/internal/testutil/tsbaseline/type_symbol_baseline.go").read_text()
    walker = replace_exact(walker, "fileChecker.GetTypeAtLocation(", "s08GetTypeAtLocation(walker, fileChecker, ", 2)
    walker = replace_exact(walker, "fileChecker.GetSymbolAtLocation(", "s08GetSymbolAtLocation(walker, fileChecker, ")
    initial = "\t\t\ttypeNode := builder.TypeToTypeNode"
    flags = "nodebuilder.Flags(typeFormatFlags&checker.TypeFormatFlagsNodeBuilderFlagsMask)|nodebuilder.FlagsIgnoreErrors"
    internal = "nodebuilder.InternalFlagsAllowUnresolvedNames"
    walker = replace_exact(walker, initial,
        f'\t\t\ts08RecordDisplay("TypeToTypeNode",walker,node.Parent,t,uint32({flags}),uint32({internal}))\n'+initial)
    alias = "\t\t\t\ttypeNode = builder.TypeToTypeNode"
    flags = "nodebuilder.Flags((typeFormatFlags|checker.TypeFormatFlagsInTypeAlias)&checker.TypeFormatFlagsNodeBuilderFlagsMask)|nodebuilder.FlagsIgnoreErrors"
    walker = replace_exact(walker, alias,
        f'\t\t\t\ts08RecordDisplay("TypeToTypeNode",walker,node.Parent,t,uint32({flags}),uint32({internal}))\n'+alias)
    symbol = '\tsymbolString.WriteString("Symbol(")\n'
    walker = replace_exact(walker,symbol,symbol+
        '\ts08RecordDisplay("SymbolToStringEx",walker,node.Parent,nil,uint32(checker.SymbolFormatFlagsAllowAnyNodeKind),0)\n')
    return {"testutil/harnessutil/harnessutil.go":harness,
            "testutil/harnessutil/s08_diagnostics_observer.go":(BRIDGES/"diagnostics_observer.go").read_text(),
            "testutil/tsbaseline/type_symbol_baseline.go":walker,
            "testutil/tsbaseline/s08_baselines_bridge.go":(BRIDGES/"baselines_bridge.go").read_text(),
            "testrunner/s08_baselines_test.go":(BRIDGES/"baselines_test.go").read_text()}


def validate_observations(rows, requests, observed, file_observations):
    if [row["id"] for row in observed] != [row["id"] for row in requests]:
        raise ValueError("missing, duplicate, extra or reordered baseline observation")
    mismatches = []
    for (case, variant), request, result in zip(rows, requests, observed, strict=True):
        if result["state"] not in ("executed", "upstream_failed", "upstream_skipped"):
            raise ValueError("unclassified source outcome")
        if result["acceptance_tier"] != request["acceptance_tier"] or request["acceptance_tier"] not in ("acceptance","informational"):
            raise ValueError("observation changed its acceptance tier")
        if result["state"] == "executed" and "panic" in result:
            raise ValueError("panicking observation cannot be executed successfully")
        for key in ("raw_sha256", "loaded_sha256"):
            if result[key] != request[key]:
                raise ValueError("source observation read a different input")
        for key, expected in (("options",variant["options"]),("harness_options",variant["harness_options"])):
            if key in result and not same_json_value(result[key], expected):
                mismatches.append({"id":request["id"], "field":key})
            elif key not in result and result["state"] == "executed":
                raise ValueError("executed observation lacks option identity")
        if ("pre_diagnostics" in result) != ("post_diagnostics" in result):
            raise ValueError("incomplete pre/post diagnostic observation")
        if "pre_diagnostics" in result:
            if not same_json_value(result["pre_diagnostics"], result["post_diagnostics"]):
                mismatches.append({"id":request["id"],"field":"pre_post_diagnostics"})
        elif result["state"] == "executed":
            raise ValueError("executed observation lacks pre/post diagnostics")
        if result["state"] != "executed":
            continue
        expected_files = [{"name":f["Name"],"path":f["Path"],"sha256":f["SHA256"],"bytes":f["Bytes"]}
                          for index in variant["dependency_closure"] for f in [file_observations[index]]]
        if not same_json_value(result["files"],expected_files):
            mismatches.append({"id":request["id"],"field":"loaded_files"})
        for key in ("types", "symbols", "errors"):
            item = result[key]
            if item["state"] not in ("content", "no_content", "disabled"):
                raise ValueError("unknown baseline outcome")
            if item["state"] == "content":
                bytes.fromhex(item["text_hex"])
            elif "text_hex" in item:
                raise ValueError("non-content outcome carries baseline text")
        disabled = variant["harness_options"]["NoTypesAndSymbols"]
        if any((result[key]["state"] == "disabled") != disabled for key in ("types", "symbols")):
            raise ValueError("baseline enablement differs from frozen policy")
        if result["errors"]["state"] == "disabled":
            raise ValueError("error baseline cannot be disabled")
        for query in result["queries"]:
            if query["operation"] not in ("GetTypeAtLocation","GetSymbolAtLocation","TypeToTypeNode","SymbolToStringEx"):
                raise ValueError("unknown walker operation")
            if not isinstance(query["file"],str) or not query["file"]:
                raise ValueError("query lacks a source file")
            if any(type(query[name]) is not int for name in ("kind","pos","end")):
                raise ValueError("malformed query coordinates")
            if "absent" in query and type(query["absent"]) is not bool:
                raise ValueError("malformed query outcome")
    return mismatches


def capture(directory, smoke=False, case_id=None, include_informational=False):
    directory = Path(directory).resolve()
    directory.mkdir(parents=True,exist_ok=False)
    upstream = verified_upstream()
    env = go_environment()
    subset = strict_json_loads((ROOT/"data/s07/subset.json").read_bytes())
    rows, requests = requests_from_subset(subset,smoke,case_id)
    raw = canonical(requests)+b"\n"
    (directory/"requests.json").write_bytes(raw)
    sources = overlay_sources(upstream)
    replacements = {}
    for name, source in sources.items():
        path = directory/"overlay"/name
        path.parent.mkdir(parents=True,exist_ok=True)
        path.write_text(source)
        replacements[str(upstream/"tsc/internal"/name)] = str(path)
    (directory/"overlay.json").write_bytes(canonical({"Replace":replacements}))
    inputs = {name:digest((ROOT/name).read_bytes()) for name in (
        "data/s07/subset.json","data/s08/baseline-requests.json","data/upstream.json",
        "scripts/s08_baselines.py","scripts/s04.py","scripts/s04_common.py","scripts/s04_runtime.py",
        "scripts/tracking-bootstrap.py","scripts/s08_manifest.py","scripts/s08_oracle.py","data/s04/toolchains.toml",
        "scripts/s07_acceptance.py",POLICY,OBSERVATIONS,"tools/s08/oracle/acceptance_policy_test.go")}
    inputs.update({str(path.relative_to(ROOT)):digest(path.read_bytes()) for path in BRIDGES.glob("*baseline*.go")})
    inputs["tools/s08/oracle/diagnostics_observer.go"] = digest((BRIDGES/"diagnostics_observer.go").read_bytes())
    for name in inputs:
        snapshot = directory/"source-snapshot"/name
        snapshot.parent.mkdir(parents=True,exist_ok=True)
        snapshot.write_bytes((ROOT/name).read_bytes())
    env.update(S08_REQUESTS=str(directory/"requests.json"),S08_OUTPUT=str(directory/"observations.ndjson"),S08_SUMMARY=str(directory/"go-summary.json"))
    env["S08_INCLUDE_INFORMATIONAL"] = "1" if include_informational else "0"
    # repo.RootPath intentionally requires this one package's physical filename.
    repo_flag = "-gcflags=github.com/microsoft/TypeScript/tsc/internal/repo=-trimpath="+str(directory/"unmatched-prefix")
    command = ["go","test","-trimpath","-mod=readonly",repo_flag,"-overlay",str(directory/"overlay.json"),"./internal/testrunner",
               "-run","^TestS08Baselines$","-count=1","-timeout=30m"]
    (directory/"command.json").write_bytes(canonical(command))
    with (directory/"go.stdout").open("wb") as stdout, (directory/"go.stderr").open("wb") as stderr:
        completed = subprocess.run(command,cwd=upstream/"tsc",env=env,stdout=stdout,stderr=stderr,timeout=1900,check=False)
    if not (directory/"go-summary.json").exists():
        raise ValueError(f"Go did not complete its observation inventory (exit {completed.returncode}); logs retained in {directory}")
    summary = strict_json_loads((directory/"go-summary.json").read_bytes())
    if summary["request_sha256"]!=digest(raw) or summary["rows"]!=len(requests):
        raise ValueError("Go did not observe the complete request")
    observed = [strict_json_loads(line) for line in (directory/"observations.ndjson").read_bytes().splitlines()]
    mismatches = validate_observations(rows, requests, observed, subset["file_observations"])
    states = Counter(row["state"] for row in observed)
    if completed.returncode not in (0,1) or (completed.returncode==1 and not states["upstream_failed"]):
        raise ValueError("unexplained Go test failure")
    verified_upstream()
    if any(digest((ROOT/name).read_bytes())!=value for name,value in inputs.items()):
        raise ValueError("capture inputs changed")
    report = {"version":2,"pin":subset["pin"],"smoke":smoke,"case_id":case_id,"source_inputs":inputs,
              "include_informational":include_informational,
              "request_sha256":digest(raw),"observation_sha256":digest((directory/"observations.ndjson").read_bytes()),
              "requests":len(requests),"states":dict(states),"test_exit":completed.returncode,
              "states_by_tier":{tier:dict(Counter(r["state"] for r in observed if r["acceptance_tier"]==tier)) for tier in ("acceptance","informational")},
              "query_operations":dict(Counter(query["operation"] for row in observed for query in row["queries"])),
              "mismatches":mismatches, "scope":"Pinned raw baseline generation and walker pulls; informational outcomes cannot affect E2; no Rust comparison or S08 acceptance claim",
              "go":summary}
    (directory/"report.json").write_bytes(canonical(report)+b"\n")
    print(json.dumps({key:value for key,value in report.items() if key not in ("source_inputs",)},sort_keys=True))


def review_capture(directory, output):
    directory = Path(directory)
    report = strict_json_loads((directory/"report.json").read_bytes())
    for name, expected in report["source_inputs"].items():
        if digest((directory/"source-snapshot"/name).read_bytes()) != expected:
            raise ValueError("captured source snapshot differs from its fingerprint")
    request_raw = (directory/"requests.json").read_bytes()
    observations_raw = (directory/"observations.ndjson").read_bytes()
    if digest(request_raw) != report["request_sha256"] or digest(observations_raw) != report["observation_sha256"]:
        raise ValueError("captured requests or observations changed")
    subset_raw = (ROOT/"data/s07/subset.json").read_bytes()
    if digest(subset_raw) != report["source_inputs"]["data/s07/subset.json"]:
        raise ValueError("review must use the captured frozen subset")
    subset = strict_json_loads(subset_raw)
    rows, requests = requests_from_subset(subset)
    if not same_json_value(strict_json_loads(request_raw), requests):
        raise ValueError("review requires the complete ordered frozen subset")
    observed = [strict_json_loads(line) for line in observations_raw.splitlines()]
    mismatches = validate_observations(rows, requests, observed, subset["file_observations"])
    discrepancies = []
    for mismatch in mismatches:
        if mismatch["field"] == "loaded_files":
            index = next(i for i, row in enumerate(requests) if row["id"] == mismatch["id"])
            variant = rows[index][1]
            discrepancies.append({"id":mismatch["id"], "expected":[subset["file_observations"][i] for i in variant["dependency_closure"]],
                                  "actual":observed[index]["files"]})
    result = {
        "version":1, "pin":report["pin"], "capture_report_sha256":digest((directory/"report.json").read_bytes()),
        "observation_sha256":report["observation_sha256"], "request_sha256":report["request_sha256"],
        "reviewer_source_sha256":digest(Path(__file__).read_bytes()),
        "scope":"Observed Go execution inventory for review; not frozen expected outcomes or Rust parity",
        "requests":len(requests), "states":dict(Counter(r["state"] for r in observed)),
        "diagnostics_compared":sum("pre_diagnostics" in r for r in observed), "mismatches":mismatches,
        "query_operations":dict(Counter(q["operation"] for r in observed for q in r["queries"])),
        "baseline_outcomes":{key:dict(Counter(r[key]["state"] for r in observed if r["state"]=="executed")) for key in ("types","symbols","errors")},
        "upstream_skipped":[r["id"] for r in observed if r["state"]=="upstream_skipped"],
        "upstream_failed":[{"id":r["id"],"frozen_option_outcome":v["option_outcome"],"frozen_option_diagnostics":v["option_diagnostics"]}
                           for (_,v),r in zip(rows,observed,strict=True) if r["state"]=="upstream_failed"],
        "native_runner_skipped":[{"id":r["id"],"observed_state":r["state"]} for r in observed if r.get("native_runner_skipped")],
        "loaded_file_discrepancies":discrepancies,
    }
    output = Path(output)
    with output.open("xb") as destination:
        destination.write(canonical(result)+b"\n")
    print(json.dumps({key:result[key] for key in ("requests","states","diagnostics_compared","mismatches","query_operations")},sort_keys=True))


if __name__=="__main__":
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output",type=Path,required=True)
    parser.add_argument("--include-informational",action="store_true",help="collect available baselines beyond native option guards for informational cases only")
    selection=parser.add_mutually_exclusive_group()
    selection.add_argument("--smoke",action="store_true")
    selection.add_argument("--case",dest="case_id",help="one exact frozen variant ID, for diagnosis only")
    selection.add_argument("--review-capture",type=Path,help="verify a full capture and write a review inventory to --output")
    args=parser.parse_args()
    try:
        if args.review_capture:
            review_capture(args.review_capture,args.output)
        else:
            capture(args.output,args.smoke,args.case_id,args.include_informational)
    except (OSError,ValueError,RuntimeError,KeyError,TypeError,subprocess.TimeoutExpired) as error:
        print(f"S08 baseline capture failed: {error}",file=sys.stderr)
        raise SystemExit(1) from error
