"""Counterexamples for published benchmark provenance and wrapper overhead."""
import copy
from pathlib import Path
import sys
import tomllib
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from s04_runtime import load_toolchains
from s07_benchmark import ROOT
from s07_benchmark_measure import aggregate, metrics_from_summaries, RUST_PROFILE, MEASUREMENT_DOMAINS
from s07_benchmark_report import validate_metadata
import test_s07_benchmark as sample_fixtures


class Metadata(unittest.TestCase):
    def metadata(self):
        stable = tomllib.loads((ROOT / "rust-toolchain.toml").read_text())["toolchain"]["channel"]
        return {
            "allocator": "mimalloc 0.1.48", "allocation_wrapper": "cap 0.1.2/stats",
            "rust_profile": RUST_PROFILE, "measurement_domains": MEASUREMENT_DOMAINS,
            "go_gc": {"GOGC": 100, "GOMEMLIMIT": "unset", "GOMAXPROCS": "worker count"},
            "revision": "1" * 40, "transport_sha256": "2" * 64,
            "graph_report_sha256": "3" * 64, "samples_sha256": "4" * 64,
            "host": {"os": "darwin", "architecture": "arm64", "release": "25.0.0",
                     "cpu_capacity": 8, "physical_cpus": 8, "memory_bytes": 64 * 2**30,
                     "initial_load_average": [1.0, 1.0, 1.0]},
            "rustc": f"rustc {stable} (111111111 2026-07-14)\nbinary: rustc\ncommit-hash: {'1'*40}\ncommit-date: 2026-07-14\nhost: aarch64-apple-darwin\nrelease: {stable}\nLLVM version: 22.1.6\n",
            "go_version": f"go version {load_toolchains(ROOT)['go']} darwin/arm64\n",
        }

    def test_rejects_false_missing_and_mismatched_provenance(self):
        valid = self.metadata()
        validate_metadata(valid)
        for field, value in (("rustc", None), ("rustc", valid["rustc"].replace("release: ", "release: 0")),
                             ("rustc", valid["rustc"].replace("aarch64-apple", "x86_64-apple")),
                             ("go_version", None), ("go_version", valid["go_version"].replace("arm64", "amd64")),
                             ("go_version", "go version go0.0.0 darwin/arm64\n"),
                             ("revision", None), ("revision", "1"*39),
                             ("transport_sha256", True), ("transport_sha256", "x"*64),
                             ("measurement_domains", None), ("measurement_domains", {}),
                             ("host", None)):
            with self.subTest(field=field, value=value), self.assertRaises(ValueError):
                validate_metadata({**valid, field: value})
        for field, value in (("cpu_capacity", True), ("memory_bytes", 0),
                             ("architecture", "x86_64"), ("initial_load_average", [float("nan")]*3),
                             ("physical_cpus", False), ("unknown", 1)):
            with self.subTest(host_field=field), self.assertRaises(ValueError):
                validate_metadata({**valid, "host": {**valid["host"], field: value}})


class WrapperOverhead(unittest.TestCase):
    def test_reports_wrapper_cost_without_adjusting_gates_or_requiring_equal_batches(self):
        rows = sample_fixtures.NativeSamples().rows()
        normal = aggregate(rows)
        for row in rows:
            if row["runtime"] == "rust" and row["allocation"]:
                row["sample"]["report"]["wall_time_ns"] *= 2
                row["sample"]["peak_rss_bytes"] *= 3
        # The timing qualification can extend normal mode to 14/21 while the
        # allocation mode retains the fixed seven samples; no rows are discarded.
        rows.extend(copy.deepcopy([row for row in rows if not row["allocation"]]))
        measured = aggregate(rows)
        self.assertEqual(metrics_from_summaries(measured), metrics_from_summaries(normal))
        for mode in ("1", "8"):
            overhead = measured[mode]["rust_wrapper_overhead"]
            self.assertIs(overhead["informational"], True)
            for metric, ratio in (("wall_time_ns", 2), ("peak_rss_bytes", 3)):
                self.assertEqual(overhead[metric]["instrumented_over_normal"], ratio)
                self.assertEqual(overhead[metric]["normal_samples"], 14)
                self.assertEqual(overhead[metric]["instrumented_samples"], 7)


if __name__ == "__main__":
    unittest.main()
