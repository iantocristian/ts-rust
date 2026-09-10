"""Phase attribution cannot relabel CPU time, change work or mix backends."""
import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import probe

EXPECTED = {"files": 2, "loaded_bytes": 10, "nodes": 4, "symbols": 1,
            "parse_diagnostics": 0, "bind_diagnostics": 0, "loaded_input_sha256": "a" * 64}


def observation():
    return {"version": 1, "runtime": "rust", "diagnostic_only": True,
            "backend": "consuming", "workers": 1, **EXPECTED,
            "bound_in_place_files": 2, "published_files": 0, "shapes_emitted": False,
            "pipeline_wall_ns": 100, "elapsed_worker_totals": {"parse_ns": 30, "bind_and_publication_ns": 60},
            "timer_domain": "per-file elapsed"}


class PhaseContracts(unittest.TestCase):
    def test_exactly_one_warmup_and_seven_alternating_pairs(self):
        rows = list(probe.order())
        self.assertEqual(rows[:2], [(True, 0, "published"), (True, 0, "consuming")])
        self.assertEqual(len(rows), 16)
        self.assertEqual(rows[4:6], [(False, 1, "consuming"), (False, 1, "published")])

    def test_wrong_backend_and_loaded_bytes_fail(self):
        probe.validate_observation(observation(), EXPECTED, "consuming", False)
        for changed in [{"backend": "published"}, {"loaded_input_sha256": "b" * 64}, {"workers": 8}, {"shapes_emitted": True}]:
            with self.subTest(changed=changed), self.assertRaises(ValueError):
                probe.validate_observation({**observation(), **changed}, EXPECTED, "consuming", False)

    def test_publication_cannot_be_omitted_or_intervals_overlap(self):
        for totals in [{"parse_ns": 30, "bind_ns": 60}, {"parse_ns": 60, "bind_and_publication_ns": 60},
                       {"parse_ns": True, "bind_and_publication_ns": 60}]:
            with self.subTest(totals=totals), self.assertRaises(ValueError):
                probe.validate_observation({**observation(), "elapsed_worker_totals": totals}, EXPECTED, "consuming", False)

    def test_full_core_shapes_include_every_file_in_order(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "shapes.ndjson"
            rows = [{"index": 0, "core_shapes": {"Token": 1}, "bound_in_place": True},
                    {"index": 1, "core_shapes": {"Identifier": 3}, "bound_in_place": False}]
            path.write_text("\n".join(json.dumps(row) for row in rows))
            result = probe.validate_shapes(path, EXPECTED)
            self.assertEqual(result, {"files": 2, "core_nodes": 4, "bound_in_place_files": 1})
            for invalid in [rows[:1], rows[::-1], [rows[0], rows[0]]]:
                path.write_text("\n".join(json.dumps(row) for row in invalid))
                with self.assertRaises(ValueError):
                    probe.validate_shapes(path, EXPECTED)
            wrong = copy.deepcopy(rows)
            wrong[1]["core_shapes"]["Identifier"] = 2
            path.write_text("\n".join(json.dumps(row) for row in wrong))
            with self.assertRaisesRegex(ValueError, "allocated node obligations"):
                probe.validate_shapes(path, EXPECTED)

    def test_registry_versions_match_root_lock(self):
        self.assertGreater(probe.registry_lock()["registry_packages"], 0)


if __name__ == "__main__":
    unittest.main()
