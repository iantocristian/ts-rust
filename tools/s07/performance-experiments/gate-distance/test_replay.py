import copy
import gzip
from pathlib import Path
import tempfile
import unittest

import replay


class DistanceTests(unittest.TestCase):
    def test_wall_and_memory_have_distinct_thresholds_and_exact_fractional_budgets(self):
        wall = replay.distance(10, 8, "wall_time_ns")
        memory = replay.distance(10, 8, "allocated_bytes")
        self.assertEqual((wall["historical_budget"], wall["excess_to_historical_budget"]), (8, 2))
        self.assertEqual(memory["historical_budget_exact"], {"numerator": 28, "denominator": 5})
        self.assertEqual(memory["excess_to_historical_budget"], 4.4)
        self.assertEqual(replay.distance(5, 8, "peak_rss_bytes")["excess_to_historical_budget"], 0)
        for rust, go, domain in ((True, 8, "wall_time_ns"), (10, 0, "allocated_bytes"), (10, 8, "live_bytes")):
            with self.assertRaises(ValueError):
                replay.distance(rust, go, domain)

    def test_missing_domain_or_mode_and_bad_counts_are_not_partial_distance_reports(self):
        def source(name):
            return replay.strict_json(gzip.decompress((replay.HERE / "inputs" / (name + ".json.gz")).read_bytes()))
        original = source("original-go-report")
        screens = {label: source(names[0]) for label, names in replay.VARIANTS.items()}
        for mutate in (lambda s: s["CP1"]["modes"]["8"].pop("peak_rss_bytes"),
                       lambda s: s["A0-b"]["modes"].pop("1"),
                       lambda s: s["CP1"].update(samples=55),
                       lambda s: s["CP1-list-copy"].update(screening_status="eligible_for_review"),
                       lambda s: s["CP1-list-copy"].update(targeted_pipeline_win=True),
                       lambda s: s["CP1-list-copy"]["manifest_sha256"].update(candidate=replay.VARIANTS["CP1"][2]),
                       lambda s: s["CP1"]["modes"]["1"]["wall_time_ns"].update(samples_per_variant=6),
                       lambda s: s["CP1"]["modes"]["1"]["allocated_bytes"].update(candidate_median=1)):
            changed = copy.deepcopy(screens)
            mutate(changed)
            with self.assertRaises(ValueError):
                replay.calculate(original, changed)

    def test_ratio_to_old_go_does_not_manufacture_a_cross_runtime_confidence_bound(self):
        result = replay.replay()
        latest = result["checkpoints"]["CP1"]["workers"]["1"]
        self.assertEqual(latest["historical_distance"]["wall_time_ns"]["excess_to_historical_budget"], 1653226417)
        self.assertIsNone(latest["timing_bootstrap_upper_95_against_historical_go"])
        self.assertFalse(result["fresh_go_acceptance_measured"])
        self.assertLess(latest["same_screen_timing_bootstrap_upper_95_ratio"], 1)
        self.assertGreater(latest["historical_distance"]["wall_time_ns"]["median_ratio_to_historical_go"], 1)

    def test_rejected_candidate_and_same_capture_retained_control_are_distinct(self):
        result = replay.replay()
        rejected = result["checkpoints"]["CP1-list-copy"]
        self.assertEqual(rejected["production_disposition"]["decision"], "rejected")
        self.assertFalse(rejected["production_disposition"]["promoted"])
        self.assertEqual(rejected["source_manifests"]["control"], result["checkpoints"]["CP1"]["source_manifests"]["candidate"])
        one = rejected["workers"]["1"]
        eight = rejected["workers"]["8"]
        self.assertEqual(one["historical_distance_role"], "candidate")
        self.assertEqual(one["historical_distance"]["wall_time_ns"]["rust_median"], 4308220625)
        self.assertEqual(one["same_screen_control_historical_distance"]["wall_time_ns"]["rust_median"], 4344062333)
        self.assertEqual(eight["historical_distance"]["wall_time_ns"]["rust_median"], 968843792)
        self.assertEqual(eight["same_screen_control_historical_distance"]["wall_time_ns"]["rust_median"], 970697959)
        for label in ("A0-b", "CP1"):
            self.assertEqual(result["checkpoints"][label]["production_disposition"]["decision"], "accepted")
            self.assertTrue(result["checkpoints"][label]["production_disposition"]["promoted"])
        for worker in rejected["workers"].values():
            self.assertIsNone(worker["timing_bootstrap_upper_95_against_historical_go"])
            for domain in replay.DOMAINS:
                candidate = worker["historical_distance"][domain]
                control = worker["same_screen_control_historical_distance"][domain]
                self.assertEqual(candidate["historical_go_median"], control["historical_go_median"])
                self.assertEqual(candidate["historical_budget_exact"], control["historical_budget_exact"])

    def test_changed_pinned_source_fails_before_arithmetic(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            (directory / "inputs").mkdir()
            (directory / "inputs/original-go-report.json.gz").write_bytes(gzip.compress(b"{}"))
            with self.assertRaisesRegex(ValueError, "recorded input changed"):
                replay.replay(directory)

    def test_duplicate_or_nonfinite_json_is_rejected(self):
        for raw in (b'{"x":1,"x":2}', b'{"x":NaN}'):
            with self.assertRaises(ValueError):
                replay.strict_json(raw)


if __name__ == "__main__":
    unittest.main()
