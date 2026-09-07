"""Bounded, stage-aware comparison and independently gated S06 measurements."""

from collections import Counter
import hashlib

from s06_diagnostics import compare_diagnostic
from s06_protocol import canonical, stages_for


class Cursor:
    """Keep one protocol record, not a source file's node table, in memory."""
    def __init__(self, records):
        self.records = iter(records)
        self.current = next(self.records)
        if self.current["tag"] != "begin":
            raise ValueError("validated stream lost its begin frame")
        self.advance()

    def advance(self):
        self.current = next(self.records, None)
        if self.current is None:
            raise ValueError("validated stream ended before its end frame")

    def at(self, stage):
        return self.current["tag"] != "end" and self.current["stage"] == stage

    def observation(self, stage):
        if not self.at(stage) or self.current["tag"] != "observation":
            return None
        record = self.current
        self.advance()
        # Sequence counters are checked independently by StreamCase. Different
        # diagnostic counts must not misalign later, independent codec stages.
        return {"kind": record["kind"], "value": record["value"]}

    def outcome(self, stage):
        if not self.at(stage):
            return None
        if self.current["tag"] != "stage":
            raise ValueError("comparison did not consume a stage's observations")
        result = {"outcome": self.current["outcome"], "message_hex": self.current["message_hex"]}
        self.advance()
        return result

    def finish(self):
        if self.current["tag"] != "end" or next(self.records, None) is not None:
            raise ValueError("comparison omitted stages or trailing frames")


def compare_case(request, oracle, rust, panic_identity, decoded_nil_wires):
    cursors = [Cursor(oracle), Cursor(rust)]
    totals = [0, 0]
    stages = {}
    first_failure = None
    for stage in stages_for(request):
        digests = [hashlib.sha256(), hashlib.sha256()]
        counts = [0, 0]
        matched = True
        exact = True
        qualifications = []
        nil_roots = [False, False]
        while True:
            pair = [cursor.observation(stage) for cursor in cursors]
            if pair == [None, None]:
                break
            for index, record in enumerate(pair):
                if record is not None:
                    digests[index].update(canonical(record)+b"\n")
                    counts[index] += 1
                    nil_roots[index] |= record["kind"] == "root" and record["value"]["node"] is None
            pair_exact = pair[0] == pair[1]
            exact &= pair_exact
            pair_matches = pair_exact
            qualification = compare_diagnostic(request, stage, pair)
            if qualification is not None:
                pair_matches = qualification["pass"]
                if pair_matches:
                    qualifications.append(qualification)
            if not pair_matches:
                matched = False
                first_failure = first_failure or {"stage": stage, "observation": counts.copy(),
                                                  "expected": pair[0], "actual": pair[1]}
        outcomes = [cursor.outcome(stage) for cursor in cursors]
        # A prior panic/error prevents a stage from running. Even matching early
        # failures cannot stand in for independently measured encoder outcomes.
        measured = all(outcome is not None for outcome in outcomes)
        outcomes_match = measured and outcomes[0]["outcome"] == outcomes[1]["outcome"]
        if outcomes_match:
            if outcomes[0]["outcome"] == "panic":
                identities = []
                for outcome, runtime in zip(outcomes, ("oracle", "rust")):
                    try:
                        message = bytes.fromhex(outcome["message_hex"]).decode("utf-8")
                    except UnicodeDecodeError:
                        identities.append(None)
                    else:
                        identities.append(panic_identity(request, stage, message, runtime, decoded_nil_wires))
                outcomes_match = identities[0] is not None and identities[0] == identities[1]
            else:
                outcomes_match = outcomes[0]["message_hex"] == outcomes[1]["message_hex"]
        if request["op"] == "decode" and request["entrypoint"] == "nodes" and stage == "decode":
            for index, runtime in enumerate(("oracle", "rust")):
                if nil_roots[index] and outcomes[index] is not None and outcomes[index]["outcome"] == "ok":
                    decoded_nil_wires[runtime].add(hashlib.sha256(bytes.fromhex(request["wire_hex"])).hexdigest())
        if request["op"] == "codec" and stage == "decode":
            for index, runtime in enumerate(("oracle", "rust")):
                if nil_roots[index] and outcomes[index] is not None and outcomes[index]["outcome"] == "ok":
                    decoded_nil_wires[runtime].add("codec:"+request["id"])
        if measured and not outcomes_match or (outcomes[0] is None) != (outcomes[1] is None):
            first_failure = first_failure or {"stage": stage, "expected": outcomes[0], "actual": outcomes[1]}
        # Matching terminal outcomes determine the case; absent later stages are
        # separately unmeasured and therefore cannot satisfy their own metrics.
        stage_pass = matched and (outcomes_match or outcomes == [None, None])
        stages[stage] = {"pass": stage_pass, "measured": measured, "observations_match": matched,
                         "observations_exact": exact, "diagnostic_qualifications": qualifications,
                         "outcomes_match": outcomes_match, "outcomes": outcomes,
                         "observations": counts, "sha256": [digest.hexdigest() for digest in digests]}
        totals = [total+count for total, count in zip(totals, counts)]
    for cursor in cursors:
        cursor.finish()
    return {"pass": all(stage["pass"] for stage in stages.values()), "stages": stages,
            "observations": totals, "failure": first_failure}


def encoder_result(result, name):
    stage = result["stages"][name]
    if not stage["measured"]:
        return False, False
    outcome = stage["outcomes_match"]
    # Exact bytes are required wherever Go encoded successfully. Returned errors
    # and recognized panics are checked by the outcome metric, not counted bytes.
    successful = stage["outcomes"][0]["outcome"] == "ok"
    return outcome, outcome and (not successful or stage["observations_match"])


def encoder_obligations(result):
    for name, stage in result["stages"].items():
        if name not in ("encode_source_file", "encode", "reencode"):
            continue
        if stage["outcomes"] == [None, None] and result["pass"]:
            # An independently recognized earlier failure (e.g. ScriptKind=0)
            # does not run an encoder. Count that fact separately in the report.
            continue
        yield encoder_result(result, name)


def report(items, results, primary_ids, probes):
    if not items or len(items) != len(results):
        raise ValueError("incomplete S06 result inventory")
    ids = [item["request"]["id"] for item in items]
    if len(ids) != len(set(ids)) or len(primary_ids) != len(set(primary_ids)):
        raise ValueError("duplicate S06 result or primary identity")
    rows = {identifier: [] for identifier in primary_ids}
    groups = Counter()
    supplemental = []
    primary_requests = []
    for item, result in zip(items, results):
        request = item["request"]
        if request["primary"] is not None:
            if request["primary"] not in rows or item["group"] != "primary":
                raise ValueError("unexpected primary membership")
            rows[request["primary"]].append(result)
            primary_requests.append(request)
        else:
            groups[item["group"]] += 1
            supplemental.append(item)
    if any(not values for values in rows.values()) or len(rows) != probes["planning_totals"]["primary_rows"]:
        raise ValueError("missing or extra primary row")
    if len(primary_requests) != probes["primary_requests"] or hashlib.sha256(canonical(primary_requests)).hexdigest() != probes["request_sha256"]:
        raise ValueError("primary request content or order changed")
    if dict(groups) != probes["supplemental"]["groups"] or len(supplemental) != probes["supplemental"]["requests"]:
        raise ValueError("missing or extra supplemental scope")
    if any(groups[group] == 0 for group in ("decoder", "ast_runtime", "depth", "encoder", "parser_regression")):
        raise ValueError("empty required supplemental scope")
    if hashlib.sha256(canonical(supplemental)).hexdigest() != probes["supplemental"]["sha256"]:
        raise ValueError("supplemental request or provenance changed")
    metrics = {"frozen_denominator": True, "primary_requests": len(primary_requests),
               "supplemental_requests": len(supplemental), "probes": len(items),
               "failed_requests": sum(not result["pass"] for result in results),
               "observations": sum(result["observations"][0] for result in results),
               "rust_observations": sum(result["observations"][1] for result in results)}
    qualifications = [qualification for result in results for stage in result["stages"].values()
                      for qualification in stage["diagnostic_qualifications"]]
    # This counts measured named qualifications, never diagnostic argument-byte
    # parity. Per-request artifacts retain both raw payloads and raw digests.
    metrics["diagnostic_qualified_observations"] = len(qualifications)
    metrics["diagnostic_argument_differences"] = sum(not item["raw_exact"] for item in qualifications)
    for group, metric in (("decoder", "decoder_parity"), ("ast_runtime", "ast_runtime"), ("depth", "depth"), ("parser_regression", "parser_regressions")):
        selected = [result for item, result in zip(items, results) if item["group"] == group or metric == "decoder_parity" and item["request"]["op"] == "codec"]
        metrics[metric] = all(result["pass"] for result in selected)
        metrics[metric+"_cases"] = len(selected)
    # AST runtime includes primary parser metadata and both cache/table traces.
    runtime_stages = ("parse", "node_index_before", "node_index_after")
    parse_results = [result for item, result in zip(items, results) if item["request"]["op"] == "parse"]
    metrics["ast_runtime"] &= all(all(stage["pass"] for name, stage in result["stages"].items() if name in runtime_stages) for result in parse_results)
    encoder = [obligation for result in results for obligation in encoder_obligations(result)]
    metrics["encoder_success_error"] = all(value[0] for value in encoder)
    metrics["encoder_output_bytes"] = all(value[1] for value in encoder)
    metrics["encoder_requests"] = len(encoder)
    metrics["encoder_stages_not_reached"] = sum(stage["outcomes"][0] is None for result in results for name, stage in result["stages"].items() if name in ("encode_source_file", "encode", "reencode"))
    # A row includes every virtual unit and option variant. Supplemental probes
    # never add rows to E1's denominator or dilute a failed physical case.
    tests = {identifier: "pass" if all(encoder_result(result, "encode_source_file") == (True, True) for result in values) else "fail" for identifier, values in rows.items()}
    return {"metrics": metrics, "tests": tests}
