"""Challenge S06 aggregation independently of the real corpus and adapters."""
import copy
import hashlib
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from s06_compare import panic_identity
from s06_protocol import StreamCase, canonical, parser_request
from s06_results import compare_case, encoder_result, report


def frames(request, *, diagnostic=False, encoded="00", panic=None):
    records = [{"version": 1, "id": request["id"], "tag": "begin", "op": "parse"}]
    seq = stages = 0

    def observe(stage, kind, value):
        nonlocal seq
        records.append({"version": 1, "id": request["id"], "tag": "observation", "seq": seq, "stage": stage, "kind": kind, "value": value})
        seq += 1

    def end_stage(stage, outcome="ok", message=""):
        nonlocal stages
        records.append({"version": 1, "id": request["id"], "tag": "stage", "stage": stage, "outcome": outcome, "message_hex": message.encode().hex()})
        stages += 1

    if panic is not None:
        end_stage("parse", "panic", panic)
    else:
        observe("parse", "source_file", {"kind": 307, "pos": 0, "end": 0, "flags": 0, "node_count": 2, "text_count": 0, "identifier_count": 0, "script_kind": 3, "language_variant": 0, "declaration_file": False, "hash": "0"*32})
        if diagnostic:
            observe("parse", "diagnostic", {"collection": "parse", "index": 0, "code": 1000, "category": 1, "pos": 0, "end": 1, "key_hex": "61", "text_hex": "62", "args_hex": []})
        end_stage("parse")
        for stage in ("node_index_before", "encode_source_file", "node_index_after"):
            if stage == "encode_source_file":
                observe(stage, "bytes", {"length": len(encoded)//2})
                observe(stage, "chunk", {"offset": 0, "hex": encoded})
            else:
                identity = {"independent_order_equal": True, "cache_reused": True} if stage == "node_index_before" else {"before_reused": True, "encoded_reused": True}
                observe(stage, "identity", identity)
                observe(stage, "table", {"length": 1})
                observe(stage, "node", {"index": 0, "node": None})
                observe(stage, "absent_lookup", {"index": 0})
            end_stage(stage)
    records.append({"version": 1, "id": request["id"], "tag": "end", "observations": seq, "stages": stages})
    return records


def validated(request, records):
    state = StreamCase(request)
    for record in records:
        state.accept(record)
        yield record
    if not state.ended:
        raise ValueError("fixture omitted end")


def compare(request, left=None, right=None):
    return compare_case(request, validated(request, left or frames(request)), validated(request, right or frames(request)), panic_identity, {"oracle": set(), "rust": set()})


class ComparisonTests(unittest.TestCase):
    def setUp(self):
        self.request = parser_request("case", "physical", b"", "/a.ts", "/a.ts")

    def test_diagnostic_count_difference_does_not_misalign_encoder(self):
        result = compare(self.request, frames(self.request, diagnostic=True), frames(self.request))
        self.assertFalse(result["pass"])
        self.assertFalse(result["stages"]["parse"]["pass"])
        for stage in ("node_index_before", "encode_source_file", "node_index_after"):
            self.assertTrue(result["stages"][stage]["pass"])
        self.assertEqual(encoder_result(result, "encode_source_file"), (True, True))
        self.assertEqual(result["failure"]["stage"], "parse")

    def test_byte_difference_fails_bytes_but_not_success_outcome(self):
        result = compare(self.request, frames(self.request), frames(self.request, encoded="ff"))
        self.assertEqual(encoder_result(result, "encode_source_file"), (True, False))
        self.assertEqual(result["failure"]["stage"], "encode_source_file")

    def test_equal_unknown_panics_are_not_parity(self):
        records = frames(self.request, panic="unexpected assertion")
        result = compare(self.request, records, records)
        self.assertFalse(result["pass"])
        self.assertFalse(result["stages"]["parse"]["outcomes_match"])

    def test_matching_invented_parser_errors_cannot_skip_encoder_obligations(self):
        records = frames(self.request, panic="not a parser return contract")
        records[1]["outcome"] = "error"
        with self.assertRaisesRegex(ValueError, "parse cannot return an error"):
            compare(self.request, records, records)

    def test_matching_early_contract_panic_never_measures_encoder(self):
        request = copy.deepcopy(self.request)
        request["script_kind"] = 0
        records = frames(request, panic="ScriptKind must be specified when parsing source file: /a.ts")
        result = compare(request, records, records)
        self.assertTrue(result["pass"])
        self.assertFalse(result["stages"]["encode_source_file"]["measured"])
        self.assertEqual(encoder_result(result, "encode_source_file"), (False, False))

    def test_extra_unframed_record_fails_capture(self):
        records = frames(self.request)
        records.append(copy.deepcopy(records[-1]))
        with self.assertRaises(ValueError):
            compare(self.request, records, frames(self.request))


class DiagnosticQualificationTests(unittest.TestCase):
    def fixture(self, name="constructor", oracle_other=True):
        choices = {
            "constructor": (b"constructorabcdefghij x;", b"const ructorabcdefghij", b"constructor abcdefghij"),
            "typeof": (b"typeofabcdefghijklmno x;", b"type ofabcdefghijklmno", b"typeof abcdefghijklmno"),
        }
        source, deterministic, alternative = choices[name]
        request = parser_request("parser/text/split-keyword-"+name, None, source, "/s06/fixture.ts", "/s06/fixture.ts")
        left, right = frames(request, diagnostic=True), frames(request, diagnostic=True)
        for records, argument in ((left, alternative if oracle_other else deterministic), (right, deterministic)):
            value = next(record["value"] for record in records if record.get("kind") == "diagnostic")
            value.update(code=1435, end=21, key_hex=b"Unknown_keyword_or_identifier_Did_you_mean_0_1435".hex(), text_hex="", args_hex=[argument.hex()])
        return request, left, right

    def test_only_observed_go_variants_match_the_fixed_rust_choice(self):
        for name in ("constructor", "typeof"):
            for alternative in (False, True):
                request, left, right = self.fixture(name, alternative)
                result = compare(request, left, right)
                self.assertTrue(result["pass"])
                stage = result["stages"]["parse"]
                self.assertEqual(stage["observations_exact"], not alternative)
                self.assertEqual(stage["sha256"][0] == stage["sha256"][1], not alternative)
                self.assertEqual(len(stage["diagnostic_qualifications"]), 1)
                qualification = stage["diagnostic_qualifications"][0]
                self.assertEqual(qualification["oracle_args_hex"], left[2]["value"]["args_hex"])
                self.assertEqual(qualification["rust_args_hex"], right[2]["value"]["args_hex"])
                self.assertEqual(qualification["raw_exact"], not alternative)
                self.assertEqual(encoder_result(result, "encode_source_file"), (True, True))
                self.assertTrue(result["stages"]["encode_source_file"]["observations_exact"])
                self.assertEqual(result["stages"]["encode_source_file"]["diagnostic_qualifications"], [])

    def test_equal_alternative_rust_arguments_still_violate_determinism(self):
        request, left, _ = self.fixture()
        result = compare(request, left, copy.deepcopy(left))
        self.assertFalse(result["pass"])
        self.assertTrue(result["stages"]["parse"]["observations_exact"])
        self.assertEqual(result["stages"]["parse"]["diagnostic_qualifications"], [])

    def test_unobserved_arguments_and_changed_stable_fields_cannot_be_qualified(self):
        for side, field, value in ((0, "args_hex", [b"constructor broken".hex()]),
                                   (1, "args_hex", [b"const wrong".hex()]),
                                   (0, "pos", 1), (1, "code", 1000),
                                   (0, "text_hex", "61"), (1, "end", 20)):
            request, left, right = self.fixture()
            (left, right)[side][2]["value"][field] = value
            with self.subTest(side=side, field=field):
                result = compare(request, left, right)
                self.assertFalse(result["pass"])
                self.assertEqual(result["failure"]["stage"], "parse")
                self.assertEqual(result["stages"]["parse"]["diagnostic_qualifications"], [])

    def test_policy_is_bound_to_the_complete_frozen_request(self):
        for field, value in (("id", "other"), ("source_hex", b"constructorabcdefghij y;".hex()),
                             ("filename", "/other.ts"), ("path", "/other.ts"),
                             ("script_kind", 4), ("force", True), ("jsx", True),
                             ("primary", "physical")):
            request, left, right = self.fixture()
            request[field] = value
            for records in (left, right):
                for record in records:
                    record["id"] = request["id"]
            with self.subTest(field=field):
                result = compare(request, left, right)
                self.assertFalse(result["pass"])
                self.assertEqual(result["stages"]["parse"]["diagnostic_qualifications"], [])

    def test_other_diagnostics_in_the_named_request_remain_exact(self):
        request, left, right = self.fixture()
        left[2]["value"].update(collection="js", index=0)
        right[2]["value"].update(collection="js", index=0)
        result = compare(request, left, right)
        self.assertFalse(result["pass"])
        self.assertEqual(result["stages"]["parse"]["diagnostic_qualifications"], [])


class ReportTests(unittest.TestCase):
    def fixture(self):
        primary = [parser_request(f"case/{i}", "physical", b"", "/a.ts", "/a.ts") for i in range(2)]
        supplemental = [{"request": parser_request(group, None, b"", "/a.ts", "/a.ts"), "group": group, "provenance": "test"} for group in ("decoder", "ast_runtime", "depth", "encoder", "parser_regression")]
        items = [{"request": request, "group": "primary"} for request in primary] + supplemental
        results = [compare(item["request"]) for item in items]
        probes = {"planning_totals": {"primary_rows": 1}, "primary_requests": 2,
                  "request_sha256": hashlib.sha256(canonical(primary)).hexdigest(),
                  "supplemental": {"requests": 5, "groups": {item["group"]: 1 for item in supplemental}, "sha256": hashlib.sha256(canonical(supplemental)).hexdigest()}}
        return items, results, ["physical"], probes

    def test_every_option_variant_must_match_without_supplemental_dilution(self):
        items, results, rows, probes = self.fixture()
        request = items[1]["request"]
        results[1] = compare(request, frames(request), frames(request, encoded="ff"))
        output = report(items, results, rows, probes)
        self.assertEqual(output["tests"], {"physical": "fail"})
        self.assertTrue(output["metrics"]["encoder_success_error"])
        self.assertFalse(output["metrics"]["encoder_output_bytes"])
        self.assertEqual(output["metrics"]["primary_requests"], 2)

    def test_runtime_mismatch_cannot_hide_behind_identical_encoded_bytes(self):
        items, results, rows, probes = self.fixture()
        request = items[0]["request"]
        results[0] = compare(request, frames(request, diagnostic=True), frames(request))
        output = report(items, results, rows, probes)
        self.assertEqual(output["tests"], {"physical": "pass"})
        self.assertTrue(output["metrics"]["encoder_output_bytes"])
        self.assertFalse(output["metrics"]["ast_runtime"])

    def test_qualified_diagnostics_are_counted_without_byte_parity_claim(self):
        items, results, rows, probes = self.fixture()
        request, left, right = DiagnosticQualificationTests().fixture()
        index = next(index for index, item in enumerate(items) if item["group"] == "ast_runtime")
        items[index]["request"] = request
        results[index] = compare(request, left, right)
        supplemental = [item for item in items if item["request"]["primary"] is None]
        probes["supplemental"]["sha256"] = hashlib.sha256(canonical(supplemental)).hexdigest()
        output = report(items, results, rows, probes)
        self.assertEqual(output["metrics"]["diagnostic_qualified_observations"], 1)
        self.assertEqual(output["metrics"]["diagnostic_argument_differences"], 1)
        self.assertTrue(output["metrics"]["ast_runtime"])
        self.assertTrue(output["metrics"]["encoder_output_bytes"])
        self.assertNotIn("diagnostic_byte_parity", output["metrics"])

    def test_missing_duplicate_reordered_or_changed_requests_fail(self):
        for mutation in ("missing", "duplicate", "reordered", "changed", "scope"):
            items, results, rows, probes = self.fixture()
            if mutation == "missing": items.pop(); results.pop()
            elif mutation == "duplicate": items[1] = copy.deepcopy(items[0])
            elif mutation == "reordered": items[:2] = reversed(items[:2])
            elif mutation == "changed": items[0]["request"]["force"] = True
            else: items[-1]["provenance"] = "changed witness"
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                report(items, results, rows, probes)


if __name__ == "__main__":
    unittest.main()
