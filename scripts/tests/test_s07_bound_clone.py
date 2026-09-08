"""The bound-source witness schema rejects type and field substitutions."""
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from s04_common import strict_json_loads
from s07_bound_clone import ROOT, validate_observation


class BoundCloneSchemaTests(unittest.TestCase):
    def setUp(self):
        self.observed = strict_json_loads((ROOT / "data/s07/bound-clone.json").read_bytes())

    def test_frozen_observation_has_exact_source_fields(self):
        validate_observation(self.observed)

    def test_equal_count_field_substitution_is_rejected(self):
        self.observed["unrelatedFlag"] = self.observed.pop("cloneFlags")
        with self.assertRaises(ValueError):
            validate_observation(self.observed)

    def test_bool_numeric_and_unsigned_substitutions_are_rejected(self):
        for field, value in (("cloneFlags", True), ("cloneFlags", 1.0),
                             ("cloneFlags", -1), ("cloneFlags", 2**32),
                             ("cloneBound", 0), ("cloneBound", "false")):
            with self.subTest(field=field, value=value), self.assertRaises(ValueError):
                validate_observation({**self.observed, field: value})


if __name__ == "__main__":
    unittest.main()
