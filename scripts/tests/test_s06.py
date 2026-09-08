"""S06 preprocessing and frozen-denominator failure paths, without a Rust port."""

import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import s06_corpus as corpus
import s06_protocol as protocol
from s04_common import strict_json_loads


class RequestTests(unittest.TestCase):
    def test_parser_text_is_preserved_without_file_decoding(self):
        for source in (b"\xef\xbb\xbflet x;", b"\xff\xfeA\x00Z", b"\xed\xa0\x80"):
            request = protocol.parser_request("case", None, source, "/a.ts", "/a.ts")
            self.assertEqual(bytes.fromhex(request["source_hex"]), source)

    def test_domain_and_unknown_fields_cannot_change_the_obligation(self):
        request = protocol.parser_request("case", None, b"", "/a.ts", "/a.ts")
        for key, value in (("version", True), ("script_kind", True), ("script_kind", 2**31),
                           ("jsx", 1), ("force", None), ("source_hex", "FF"),
                           ("source_hex", "f"), ("operations", ["parse"]), ("id", ""),
                           ("op", "decode")):
            changed = copy.deepcopy(request); changed[key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                protocol.validate_request(changed)
        for key in request:
            changed = copy.deepcopy(request); del changed[key]
            with self.assertRaises(ValueError):
                protocol.validate_request(changed)
        changed = copy.deepcopy(request); changed["skip"] = True
        with self.assertRaises(ValueError):
            protocol.validate_request(changed)
        # Open script kinds are Go inputs, including the kind-zero panic input.
        for kind in (-(2**31), -1, 0, 7, 2**31-1):
            protocol.parser_request("kind", None, b"", "/a.ts", "/a.ts", kind)

    def test_duplicate_json_keys_and_nonfinite_numbers_fail(self):
        for raw in ('{"version":1,"version":1}', '{"x":NaN}', '{"x":1e999}', '{} {}'):
            with self.assertRaises(ValueError):
                strict_json_loads(raw)

    def test_request_digest_binds_content_options_and_operation_sequence(self):
        request = protocol.parser_request("case", "primary", b"source", "/a.ts", "/a.ts")
        original = protocol.sha256(protocol.canonical(request))
        for key, value in (("source_hex", "ff"), ("path", "/b.ts"), ("jsx", True),
                           ("force", True), ("script_kind", 1), ("primary", "other")):
            changed = copy.deepcopy(request); changed[key] = value
            self.assertNotEqual(protocol.sha256(protocol.canonical(changed)), original)


class InventoryTests(unittest.TestCase):
    def test_membership_is_the_git_tree_not_filesystem_globs(self):
        physical = "tsc/testdata/tests/cases/compiler/a.ts"
        library = "tsc/internal/bundled/libs/lib.d.ts"
        output = "\n".join((physical, library, "tsc/testdata/tests/cases/transpile/a.ts", "other.ts"))
        with patch.object(corpus, "EXPECTED", {"physical_cases": 1, "libraries": 1}), patch.object(corpus, "command", return_value=output.encode()) as command:
            self.assertEqual(corpus.membership(Path("upstream"), "pin"), ([physical], [library]))
            self.assertIn("ls-tree", command.call_args.args[0])

    def test_missing_duplicate_extra_and_reordered_rows_fail(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "output.ndjson"
            for paths in (("a",), ("a", "a"), ("a", "b", "c"), ("b", "a")):
                output.write_text("".join(json.dumps({"kind": "library", "path": p, "filename": "/"+p, "text_hex": ""})+"\n" for p in paths))
                with self.assertRaises(ValueError):
                    corpus.read_export(output, ["a"], ["b"])

    def test_verification_never_rewrites_drifted_or_partial_manifests(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            original = b'{"original":true}\n'
            (directory / "corpus.json").write_bytes(original)
            with self.assertRaises(ValueError):
                corpus.manifest_changes(directory, {"corpus.json": {"cases": [{"new": True}]}, "cases.json": ["a"]})
            self.assertEqual((directory / "corpus.json").read_bytes(), original)
            self.assertFalse((directory / "cases.json").exists())

    def test_supplemental_scope_is_not_a_primary_identifier(self):
        for path in ("fixtures/deep", "tsc/testdata/tests/cases/transpile/a.ts", "tsc/internal/parser/parser_test.go"):
            with self.assertRaises(ValueError):
                corpus.primary_id(path)



class StreamTests(unittest.TestCase):
    def request(self):
        return {"version": 1, "id": "x", "primary": None, "op": "path", "path_hex": "2f"}

    def frames(self):
        return [
            {"version": 1, "id": "x", "tag": "begin", "op": "path"},
            {"version": 1, "id": "x", "tag": "observation", "seq": 0, "stage": "path", "kind": "path", "value": {"encoded_root_length": 1, "normalized_hex": "2f", "declaration_file": False}},
            {"version": 1, "id": "x", "tag": "stage", "stage": "path", "outcome": "ok", "message_hex": ""},
            {"version": 1, "id": "x", "tag": "end", "observations": 1, "stages": 1},
        ]

    def test_valid_stream_and_measured_error(self):
        state = protocol.StreamCase(self.request())
        for frame in self.frames(): state.accept(frame)
        self.assertTrue(state.ended)
        state = protocol.StreamCase(self.request())
        frames = self.frames(); frames[2]["outcome"] = "panic"; frames[2]["message_hex"] = b"specific assertion".hex()
        for frame in frames: state.accept(frame)
        self.assertEqual(state.outcomes, [("path", "panic", b"specific assertion".hex())])

    def test_unknown_duplicate_missing_reordered_and_wrong_count_frames(self):
        good = self.frames()
        variants = [good[1:], good[:1]+good[2:], good[:2]+good[1:], good[:1]+[good[2],good[1],good[3]],good+[good[-1]]]
        for index,key,value in ((0,"id","wrong"),(1,"seq",1),(1,"kind","invented"),(2,"outcome",True),(3,"observations",0),(3,"stages",True)):
            frames = copy.deepcopy(good); frames[index][key] = value; variants.append(frames)
        frames = copy.deepcopy(good);frames[1]["value"]["declaration_file"] = 1;variants.append(frames)
        frames = copy.deepcopy(good);frames[1]["value"]["skip"] = True;variants.append(frames)
        for frames in variants:
            with self.subTest(frames=frames), self.assertRaises(ValueError):
                state = protocol.StreamCase(self.request())
                for frame in frames: state.accept(frame)

    def test_successful_stage_cannot_hide_all_members_or_bytes(self):
        request = protocol.parser_request("x", "primary", b"", "/x.ts", "/x.ts")
        for stage in ("parse", "node_index_before", "encode_source_file", "node_index_after"):
            state = protocol.StreamCase(request)
            state.stage_index = state.stages.index(stage)
            state.started = True
            with self.assertRaises(ValueError):
                state.accept({"version":1,"id":"x","tag":"stage","stage":stage,"outcome":"ok","message_hex":""})

    def test_only_actual_returned_error_stages_accept_error_frames(self):
        requests = [self.request(),
                    protocol.parser_request("x", None, b"", "/x.ts", "/x.ts"),
                    {"version":1,"id":"x","primary":None,"op":"kind_names","first":0,"count":1},
                    {"version":1,"id":"x","primary":None,"op":"factory","scenario":protocol.FACTORY_SCENARIOS[0]},
                    {"version":1,"id":"x","primary":None,"op":"codec","scenario":"raw-kind"}]
        for request in requests:
            for stage in protocol.stages_for(request):
                with self.subTest(stage=stage):
                    state = protocol.StreamCase(request)
                    state.started = True
                    state.stage_index = state.stages.index(stage)
                    record = {"version":1,"id":"x","tag":"stage","stage":stage,"outcome":"error","message_hex":b"source error".hex()}
                    if stage in protocol.RETURNED_ERROR_STAGES:
                        state.accept(record)
                        self.assertTrue(state.stopped)
                    else:
                        with self.assertRaisesRegex(ValueError, "cannot return an error"):
                            state.accept(record)

    def test_decoder_tree_cannot_silently_stop_after_returning_a_root(self):
        request = {"version":1,"id":"x","primary":None,"op":"decode","wire_hex":"","entrypoint":"nodes"}
        state = protocol.StreamCase(request)
        state.accept({"version":1,"id":"x","tag":"begin","op":"decode"})
        state.accept({"version":1,"id":"x","tag":"observation","seq":0,"stage":"decode","kind":"root","value":{"node":{"kind":0,"pos":0,"end":0,"flags":0}}})
        state.accept({"version":1,"id":"x","tag":"stage","stage":"decode","outcome":"ok","message_hex":""})
        with self.assertRaises(ValueError):
            state.accept({"version":1,"id":"x","tag":"stage","stage":"decoded_tree","outcome":"ok","message_hex":""})

    def test_absolute_transport_deadline_rejects_partial_byte_trickle(self):
        from s06_process import Process
        with tempfile.TemporaryDirectory() as temporary:
            code = "import os,time; [(os.write(1,b'x'),time.sleep(.03)) for _ in range(20)]"
            child = Process([sys.executable, "-c", code], Path(temporary)/"stderr", deadline=.12)
            try:
                with self.assertRaises(TimeoutError): child.read()
            finally: child.close()


class PanicTests(unittest.TestCase):
    def test_nil_root_requires_same_runtime_successful_nil_result(self):
        from s06_compare import panic_identity
        request={"op":"decode","entrypoint":"source_file","wire_hex":"ffff"}
        digest=protocol.sha256(bytes.fromhex(request["wire_hex"]))
        seen={"oracle":{digest},"rust":{digest}}
        go="runtime error: invalid memory address or nil pointer dereference"
        rust="nil SourceFile root returned by DecodeNodes"
        self.assertEqual(panic_identity(request,"decode",go,"oracle",seen),
                         panic_identity(request,"decode",rust,"rust",seen))
        for observed in ({}, {"oracle":{digest}}, {"rust":{"wrong-wire"}}):
            self.assertIsNone(panic_identity(request,"decode",rust,"rust",observed))
        self.assertIsNone(panic_identity(request,"decoded_tree",rust,"rust",seen))
        self.assertIsNone(panic_identity(request,"decode",go+" extra","oracle",seen))

    def test_bounds_never_accepts_assertion_or_partial_runtime_message(self):
        from s06_compare import panic_identity
        request={"op":"decode","entrypoint":"nodes","wire_hex":""}
        self.assertEqual(panic_identity(request,"decode","runtime error: index out of range [0] with length 0","oracle",{}),
                         panic_identity(request,"decode","index out of bounds: the len is 0 but the index is 0","rust",{}))
        for message in ("assertion failed: bounds", "attempt to add with overflow", "index out of bounds: custom", "SyntheticExpression should never be encoded"):
            self.assertIsNone(panic_identity(request,"decode",message,"rust",{}))


if __name__ == "__main__":
    unittest.main()
