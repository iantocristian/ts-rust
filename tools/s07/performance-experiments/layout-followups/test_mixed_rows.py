"""Bounded mixed-row correctness checks and exact historical-control replay."""
import copy
import gzip
import json
from pathlib import Path
import tempfile
import unittest

import mixed_rows as model


class MixedRowsTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.rows, cls.baseline, cls.previous, cls.frozen = model.inputs()
        cls.report = json.loads(gzip.decompress((model.HERE / "mixed-rows-result.json.gz").read_bytes()))
        cls.layouts = cls.report["layouts"]

    def test_compiled_rows_have_plain_references_and_exact_width(self):
        with tempfile.TemporaryDirectory() as directory:
            layouts, compilation = model.compile_sketch(Path(directory))
        self.assertEqual(layouts, self.layouts)
        self.assertIn("2 passed; 0 failed", compilation["test_output"])
        for width in range(1, 17):
            self.assertEqual(layouts[f"plain.{width}"], layouts[f"composite.{width}"])
            self.assertEqual(layouts[f"plain.{width}"], layouts[f"atomic.{width}"])
        self.assertEqual(layouts["plain.0"]["size"], 0)

    def test_split_classes_change_capacity_even_when_used_bytes_match(self):
        shape = lambda facts: {"facts": facts, "syntax_layout": {"size": 8, "alignment": 4},
                               "bound_layout": {"size": 8, "alignment": 4}}
        shapes = {"Plain": shape(False), "Composite": shape(True)}
        rows = [{"core_shapes": {"Plain": 9, "Composite": 7}}]
        grouped, classes, count = model.group_shapes(rows, shapes, self.layouts, True)
        self.assertEqual(count["used_bound_payload_bytes"], 128)
        self.assertEqual(count["used_atomic_words"], 7)
        self.assertEqual(classes["composite.2"]["plain_scalar_words"], 1)
        self.assertEqual(classes["plain.2"]["atomic_words"], 0)
        split, _ = model.previous.page_costs(grouped, classes, 8, 8, self.previous["directory_layouts"], 8)
        grouped, classes, count = model.group_shapes(rows, shapes, self.layouts, False)
        combined, _ = model.previous.page_costs(grouped, classes, 8, 8, self.previous["directory_layouts"], 8)
        self.assertEqual((combined["capacity_bound"], split["capacity_bound"]), (128, 192))
        self.assertEqual((combined["payload_pages"], split["payload_pages"]), (2, 3))

    def test_double_counted_or_missing_composite_facts_is_rejected(self):
        changed = copy.deepcopy(self.layouts)
        changed["composite.16"]["size"] += 4
        with self.assertRaisesRegex(ValueError, "row layout differs"):
            model.group_shapes(self.rows, self.baseline["shape_inventory"], changed, True)
        shapes = copy.deepcopy(self.baseline["shape_inventory"])
        shapes["Token"]["facts"] = True
        with self.assertRaisesRegex(ValueError, "omitted its facts word"):
            model.group_shapes(self.rows, shapes, self.layouts, True)
        shapes["Token"]["facts"] = 1
        with self.assertRaisesRegex(ValueError, "invalid composite classification"):
            model.group_shapes(self.rows, shapes, self.layouts, True)

    def test_unknown_shape_or_alignment_does_not_silently_enter_a_class(self):
        with self.assertRaisesRegex(ValueError, "unknown concrete shape"):
            model.group_shapes([{"core_shapes": {"UnknownShape": 1}}], self.baseline["shape_inventory"], self.layouts, True)
        shapes = copy.deepcopy(self.baseline["shape_inventory"])
        shapes["Identifier"]["bound_layout"]["alignment"] = 8
        with self.assertRaisesRegex(ValueError, "unsupported word alignment"):
            model.group_shapes(self.rows, shapes, self.layouts, True)

    def test_census_order_types_counts_and_binding_path_remain_checked(self):
        expected = {"files": 1, "nodes": 1, "bound_in_place_files": 1}
        valid = [{"index": 0, "core_shapes": {"Token": 1}, "bound_in_place": True}]
        model.validate_rows(valid, expected)
        for changed in [[], valid * 2,
                        [{**valid[0], "index": True}],
                        [{**valid[0], "core_shapes": {"Token": True}}],
                        [{**valid[0], "core_shapes": {"Token": 2}}],
                        [{**valid[0], "bound_in_place": False}]]:
            with self.assertRaises(ValueError):
                model.validate_rows(changed, expected)

    def test_full_census_used_bytes_and_atomic_word_counts_reconcile(self):
        plain, mixed = self.report["families"]["all_atomic_rows"], self.report["families"]["mixed_rows"]
        self.assertEqual(plain["used_bound_payload_bytes"], mixed["used_bound_payload_bytes"])
        self.assertEqual(plain["used_atomic_words"] * 4, plain["used_bound_payload_bytes"])
        composites = sum(count for row in self.rows for name, count in row["core_shapes"].items()
                         if self.baseline["shape_inventory"][name]["facts"])
        self.assertEqual(mixed["used_atomic_words"], composites)
        self.assertEqual((plain["generated_nonempty_classes"], mixed["generated_nonempty_classes"]), (15, 23))
        for family in (plain, mixed):
            for candidate in family["candidates"]:
                self.assertEqual(candidate["requests_accounted_bytes"] - candidate["live_accounted_bytes"],
                                 candidate["directory_superseded_requests"])

    def test_all_candidates_replay_and_historical_files_are_unchanged(self):
        replay = model.project(self.rows, self.baseline, self.previous, self.layouts)
        self.assertEqual(replay, {key: value for key, value in self.report.items() if key != "provenance"})
        self.assertEqual(model.inputs()[3], self.frozen)
        self.assertEqual(self.frozen, self.report["provenance"]["historical_inputs_sha256"])
        self.assertEqual(model.sha(Path(model.__file__).read_bytes()), self.report["provenance"]["projection_source_sha256"])
        self.assertEqual(model.sha((model.HERE / "mixed_rows.rs").read_bytes()), self.report["provenance"]["compilation"]["source_sha256"])


if __name__ == "__main__":
    unittest.main()
