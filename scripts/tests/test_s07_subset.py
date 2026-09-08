import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

"""Failure-oriented tests for the normative subset boundary, not Go parity."""

import copy
import json
import unittest

import s07_subset as subset


class SubsetBoundaryTests(unittest.TestCase):
    def setUp(self):
        self.table = subset.table("syntax-table.json", "kind")
        self.variant = {"syntax": [{"unit": 0, "route": "virtual_file", "syntax": {
            "kinds": {"KindSourceFile": 1, "KindMissingDeclaration": 1},
            "first_positions": {"KindSourceFile": 0, "KindMissingDeclaration": 3},
            "nonempty_type_arguments": [], "diagnostics": [{"code": 1005}],
        }}], "project_references": False, "content_mappers": False, "request": {"options": {}}}

    def test_malformed_input_is_eligible_until_a_feature_rejects_it(self):
        self.assertEqual(subset.source_reasons(self.variant, self.table), [])
        syntax = self.variant["syntax"][0]["syntax"]
        syntax["kinds"]["KindConditionalType"] = 1
        syntax["first_positions"]["KindConditionalType"] = 8
        self.assertEqual(subset.source_reasons(self.variant, self.table)[0]["rule"], "conditional_types")

    def test_type_arguments_and_jsdoc_type_parameters_reject_independently(self):
        syntax = self.variant["syntax"][0]["syntax"]
        syntax["nonempty_type_arguments"] = [12]
        self.assertEqual(subset.source_reasons(self.variant, self.table)[0]["rule"], "nonempty_type_arguments")
        syntax["nonempty_type_arguments"] = []
        syntax["kinds"]["KindJSDocTemplateTag"] = 1
        syntax["first_positions"]["KindJSDocTemplateTag"] = 15
        self.assertEqual(subset.source_reasons(self.variant, self.table)[0]["rule"], "explicit_type_parameters")

    def test_unknown_or_malformed_syntax_is_an_error_not_an_exclusion(self):
        for patch in ({"kinds": {"KindFuture": 1}, "first_positions": {"KindFuture": 0}},
                      {"kinds": {"KindSourceFile": True, "KindMissingDeclaration": 1}},
                      {"nonempty_type_arguments": {}}, {"first_positions": {}}):
            variant = copy.deepcopy(self.variant)
            variant["syntax"][0]["syntax"].update(patch)
            with self.assertRaises(ValueError):
                subset.source_reasons(variant, self.table)

    def test_mapper_configuration_without_execution_is_retained(self):
        self.variant["content_mappers"] = True
        self.assertEqual(subset.source_reasons(self.variant, self.table), [])
        self.variant["request"]["options"]["runExternalCode"] = True
        self.assertEqual(subset.source_reasons(self.variant, self.table), [{"rule": "content_mapper_execution"}])

    def test_emit_exclusion_requires_the_exact_conjunction(self):
        variant = {"harness_options": {"NoTypesAndSymbols": True}}
        self.assertFalse(subset.emitted_only(variant, []))
        self.assertTrue(subset.emitted_only(variant, [{"kind": ".js"}]))
        for kind in (".errors.txt", ".types", ".symbols"):
            self.assertFalse(subset.emitted_only(variant, [{"kind": ".js"}, {"kind": kind}]))
        variant["harness_options"]["NoTypesAndSymbols"] = False
        self.assertFalse(subset.emitted_only(variant, [{"kind": ".js"}]))

    def test_canonicalization_preserves_source_ordered_option_maps(self):
        value = {"z": 1, "options": {"paths": {"z/*": ["first"], "a/*": ["second"]}},
                 "config_raw": {"compilerOptions": {"paths": {"z": ["first"], "a": ["second"]}}, "files": []}}
        decoded = json.loads(subset.json_bytes(value))
        self.assertEqual(list(decoded), sorted(value))
        self.assertEqual(list(decoded["options"]["paths"]), ["z/*", "a/*"])
        self.assertEqual(list(decoded["config_raw"]), ["compilerOptions", "files"])
        self.assertEqual(list(decoded["config_raw"]["compilerOptions"]["paths"]), ["z", "a"])


if __name__ == "__main__":
    unittest.main()
