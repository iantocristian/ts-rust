"""Counterexamples and replay for the bounded storage arithmetic."""
import gzip
import hashlib
import json
from pathlib import Path
import tempfile
import unittest

import project_layout as layout


class LayoutProjectionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = json.loads(gzip.decompress((layout.HERE / "layout-projection.json.gz").read_bytes()))
        cls.model = json.loads((layout.HERE.parent / "layout/baseline-model.json").read_text())
        cls.raw = gzip.decompress((layout.HERE / "core-shapes.ndjson.gz").read_bytes())
        cls.rows = [json.loads(line) for line in cls.raw.splitlines()]
        cls.sizes = cls.report["directory_layouts"]

    def test_page_boundaries_match_an_independent_push_simulation(self):
        for first, maximum in [(1, 1), (1, 32), (2, 64), (4, 4), (32, 32), (2, 256)]:
            capacities = []
            for count in range(700):
                if count > sum(capacities):
                    capacities.append(first if not capacities else min(capacities[-1] * 2, maximum))
                self.assertEqual(layout.page_plan(count, first, maximum), (sum(capacities), len(capacities)))
        for args in [(-1, 1, 32), (1, 0, 32), (1, 4, 2), (1, 3, 32), (1, 1, 33)]:
            with self.assertRaises(ValueError):
                layout.page_plan(*args)

    def test_vector_requests_include_every_replacement(self):
        self.assertEqual(layout.vector_plan(0, 8), (0, 0, 0))
        self.assertEqual(layout.vector_plan(5, 8), (64, 96, 2))
        self.assertEqual(layout.vector_plan(9, 8, 1), (128, 248, 5))
        live, requests, calls = layout.vector_plan(513, 12, 1)
        self.assertEqual((live, requests - live, calls), (12_288, 12_276, 11))
        with self.assertRaises(ValueError):
            layout.vector_plan(-1, 8)

    def test_frozen_census_is_bound_to_raw_child_and_inputs(self):
        proof = json.loads(gzip.decompress((layout.HERE / "core-shapes-provenance.json.gz").read_bytes()))
        capture, build = proof["capture"], proof["build_manifest"]
        self.assertEqual(hashlib.sha256(self.raw).hexdigest(), capture["summary"]["core_shapes_sha256"])
        self.assertEqual(capture["build_manifest_sha256"], self.report["provenance"]["build_manifest_sha256"])
        for name in ("stdout", "stderr"):
            self.assertEqual(hashlib.sha256(proof["raw_" + name].encode()).hexdigest(),
                             capture["raw_sha256"]["sample-0-consuming." + name])
        child = layout.probe.validate_observation(json.loads(proof["raw_stdout"]), build["expected_work"], "consuming", True)
        self.assertEqual(sum(sum(row["core_shapes"].values()) for row in self.rows), child["nodes"])
        self.assertEqual(sum(row["bound_in_place"] for row in self.rows), child["bound_in_place_files"])
        self.assertEqual(child["loaded_input_sha256"], self.report["provenance"]["loaded_input_sha256"])
        self.assertEqual(build["source_fingerprint"]["sha256"], self.report["provenance"]["build_source_fingerprint"])

    def test_compiled_types_match_recorded_envelopes(self):
        with tempfile.TemporaryDirectory() as temporary:
            sizes, _ = layout.compile_directory_sketch(Path(temporary))
        self.assertEqual(sizes, self.sizes)
        self.assertEqual(sizes["ThinPage"]["size"], 8)
        self.assertEqual(sizes["FatPage"]["size"], 16)
        self.assertEqual(sizes["AtomicWord16"], sizes["Word16"])

    def test_owner_ledger_saves_exactly_one_ledger_per_active_store(self):
        best = self.report["compact_box_pages"]["best_by_representation_and_ledger"]
        for kind in ("Fat", "Thin"):
            store, owner = best[kind + "_store"], best[kind + "_owner"]
            for key in ("payload_policy", "header_policy", "directory_strategy"):
                self.assertEqual(store[key], owner[key])
            self.assertEqual(store["live_accounted_bytes"] - owner["live_accounted_bytes"],
                             owner["payloads"]["active_shapes"] * self.sizes["Ledger"]["size"])
            self.assertTrue(owner["header"]["root_includes_used_length_and_ledger"])

    def test_odd_class_count_rounds_packed_root_to_vector_alignment(self):
        empty = {"active_shapes": 0, "directory_vec_live": 0, "directory_vec_requests": 0, "directory_vec_allocations": 0}
        result = layout.lean_directory_costs("packed_u16_index", empty, [0], self.sizes, "Thin", "owner", 15)
        self.assertEqual(result["shape_roots"], self.sizes["PackedWordDirectory"]["size"])
        self.assertEqual(result["shape_roots"], 56)

    def test_word_atomic_removal_does_not_hide_table_capacity_or_directories(self):
        word = self.report["word_class_pages"]
        atomic = word["atomic_word_variant"]["lowest_accounted_candidate"]
        scalar = next(row for row in word["candidates"] if all(row[key] == atomic[key]
            for key in ("payload_policy", "header_policy", "directory_strategy")) and row["facts_policy"]["first"] == 32)
        self.assertGreater(scalar["live_accounted_bytes"] - atomic["live_accounted_bytes"],
                           word["composite_nodes"] * self.sizes["AtomicFacts"]["size"])
        self.assertEqual(atomic["syntax_attributing_all_shared_directories_and_spare_to_syntax"]
                         + atomic["binding_used_increment"], atomic["live_accounted_bytes"])
        for row in word["atomic_word_variant"]["candidates"]:
            self.assertEqual(row["requests_accounted_bytes"] - row["live_accounted_bytes"], row["directory_superseded_requests"])

    def test_every_recorded_projection_replays_from_raw_census(self):
        replay = layout.project(self.rows, self.model, self.sizes)
        self.assertEqual(replay, {key: value for key, value in self.report.items() if key != "provenance"})
        self.assertEqual(layout.probe.runner.digest(Path(layout.__file__)), self.report["provenance"]["projection_script_sha256"])
        self.assertEqual(layout.probe.runner.digest(layout.HERE.parent / "layout/baseline-model.json"),
                         self.report["provenance"]["generated_layout_model_sha256"])


if __name__ == "__main__":
    unittest.main()
