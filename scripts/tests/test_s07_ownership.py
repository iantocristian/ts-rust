import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import copy
import unittest

from s07_ownership import GROUPS, load_cases, publish_metrics


class OwnershipScope(unittest.TestCase):
    def setUp(self):
        self.manifest = load_cases(Path(__file__).resolve().parents[2])
        self.modes = {mode: {group: True for group in GROUPS} for mode in ("debug", "release", "miri", "address_sanitizer")}

    def test_group_failure_does_not_certify_it_or_later_checker_work(self):
        self.modes["miri"]["retained_snapshot_edit"] = False
        report = {"metrics": {}}
        publish_metrics(report, self.modes, self.manifest)
        self.assertTrue(report["metrics"]["shared_bound_file"])
        self.assertFalse(report["metrics"]["retained_snapshot_edit"])
        self.assertNotIn("independent_checker_merges", report["metrics"])
        self.assertNotIn("type_footprint_ratio", report["metrics"])

    def test_missing_instrumentation_or_group_cannot_publish_success(self):
        for missing in self.modes:
            modes = copy.deepcopy(self.modes)
            del modes[missing]
            with self.assertRaises(ValueError):
                publish_metrics({"metrics": {}}, modes, self.manifest)
        del self.modes["miri"]["shared_bound_file"]
        with self.assertRaises(ValueError):
            publish_metrics({"metrics": {}}, self.modes, self.manifest)


if __name__ == "__main__":
    unittest.main()
