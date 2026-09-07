"""Failure-path checks for the JSON-only TypeScript client renderer."""

import copy
import json
import unittest

from render_typescript import render, unique_object


def example_contract():
    return {
        "schemaVersion": 1,
        "generator": "tools/gen-proto",
        "types": [{
            "id": "api.Params", "kind": "dto", "typescript": "Params",
            "typescriptDoc": "", "fields": [{
                "jsonName": "content-type", "typescript": "readonly string[]",
                "typescriptDoc": "    /** Source documentation. */\n",
                "optional": True, "clientVisible": True,
            }, {
                "jsonName": "private", "clientVisible": False,
            }],
        }],
        "declarationOrder": ["api.Params"],
        "aliases": [],
        "imports": [],
        "methods": [{
            "id": "get-data", "params": {"typescript": "Params"},
            "result": {"typescript": "number | null"},
        }],
    }


class ContractRenderingTests(unittest.TestCase):
    def test_only_client_visible_fields_and_authoritative_wire_spellings(self):
        output = render(example_contract())
        self.assertIn('"get-data": APIMethod<Params, number | null>', output)
        self.assertIn('"content-type"?: readonly string[];', output)
        self.assertIn("/** Source documentation. */", output)
        self.assertNotIn("private", output)

    def test_missing_dto_declaration_is_an_error(self):
        contract = example_contract()
        contract["declarationOrder"] = []
        with self.assertRaisesRegex(ValueError, "does not cover"):
            render(contract)

    def test_duplicate_dto_and_method_are_errors(self):
        contract = example_contract()
        contract["types"].append(copy.deepcopy(contract["types"][0]))
        with self.assertRaisesRegex(ValueError, "duplicate exported type"):
            render(contract)
        contract = example_contract()
        contract["methods"].append(copy.deepcopy(contract["methods"][0]))
        with self.assertRaisesRegex(ValueError, "duplicate method ID"):
            render(contract)

    def test_duplicate_json_metadata_is_an_error(self):
        with self.assertRaisesRegex(ValueError, "duplicate JSON key"):
            json.loads('{"schemaVersion":1,"schemaVersion":2}', object_pairs_hook=unique_object)

    def test_wrong_export_version_is_an_error(self):
        contract = example_contract()
        contract["schemaVersion"] = 2
        with self.assertRaisesRegex(ValueError, "unsupported"):
            render(contract)


if __name__ == "__main__":
    unittest.main()
