"""Protocol and accounting counterexamples for the diagnostic pprof reader."""
import unittest
import hashlib
import json
from pathlib import Path
import tempfile

from analyze_pprof import capture_profiles, parse_raw, summarize


RAW = """PeriodType: cpu nanoseconds
Period: 10000000
Time: synthetic
Duration: 1.00
Samples:
samples/count cpu/nanoseconds
          2   20000000: 1 2 1 2
                phase:[bind]
          1   10000000: 3
          1   10000000: 4
                phase:[parse]
Locations
     1: 0x1 M=1 runtime.gcAssistAlloc runtime/mgcmark.go:1:0 s=1
     2: 0x2 M=1 app.recursive app/file.go:1:0 s=1
     3: 0x3 M=1 runtime.gcBgMarkWorker runtime/mgc.go:1:0 s=1
     4: 0x4 M=1 app.parse app/file.go:2:0 s=1
Mappings
1: synthetic
"""


class RawProfileTests(unittest.TestCase):
    def make_capture(self, folder, repetitions):
        runs = []
        for workers in (1, 8):
            for index in range(repetitions):
                raw = f"synthetic profile {workers} {index}".encode()
                (folder / f"go-{workers}-{index}.pprof").write_bytes(raw)
                runs.append({"runtime": "go", "profiled": True, "workers": workers, "index": index,
                             "artifacts": {"profile": hashlib.sha256(raw).hexdigest()}})
        report = {"schema": 1, "diagnostic_only": True, "capture_complete": True,
                  "repetitions": repetitions, "runs": runs}
        (folder / "report.json").write_text(json.dumps(report))
        return report

    def test_capture_declares_entire_repetition_inventory(self):
        for repetitions in (1, 4, 10):
            with self.subTest(repetitions=repetitions), tempfile.TemporaryDirectory() as temporary:
                folder = Path(temporary)
                self.make_capture(folder, repetitions)
                count, profiles, _ = capture_profiles(folder)
                self.assertEqual(count, repetitions)
                self.assertEqual(set(profiles), {(worker, index) for worker in (1, 8) for index in range(count)})

    def test_capture_rejects_missing_duplicate_extra_or_changed_profiles(self):
        for mutation in ("missing", "duplicate", "extra", "extra_file", "changed_bytes"):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as temporary:
                folder = Path(temporary)
                report = self.make_capture(folder, 1)
                if mutation == "missing":
                    report["runs"].pop()
                elif mutation == "duplicate":
                    report["runs"].append(report["runs"][0])
                elif mutation == "extra":
                    report["runs"].append({**report["runs"][0], "index": 1})
                elif mutation == "extra_file":
                    (folder / "go-1-1.pprof").write_bytes(b"extra")
                else:
                    (folder / "go-1-0.pprof").write_bytes(b"changed")
                (folder / "report.json").write_text(json.dumps(report))
                with self.assertRaises(ValueError):
                    capture_profiles(folder)

    def test_positive_period_required(self):
        for value in ("0", "-1"):
            with self.subTest(value=value), self.assertRaisesRegex(ValueError, "period must be positive"):
                parse_raw(RAW.replace("Period: 10000000", "Period: " + value))

    def test_nonempty_samples_required(self):
        prefix, rest = RAW.split("samples/count cpu/nanoseconds\n")
        locations = rest.split("Locations\n", 1)[1]
        with self.assertRaisesRegex(ValueError, "empty CPU profile"):
            parse_raw(prefix + "samples/count cpu/nanoseconds\n\nLocations\n" + locations)

    def test_weight_must_match_count_and_period(self):
        with self.assertRaisesRegex(ValueError, "count/CPU weight"):
            parse_raw(RAW.replace("2   20000000:", "2   10000000:"))

    def test_duplicate_labels_rejected(self):
        with self.assertRaisesRegex(ValueError, "duplicate or misplaced"):
            parse_raw(RAW.replace("phase:[bind]", "phase:[bind]\n                phase:[parse]"))

    def test_missing_location_rejected(self):
        with self.assertRaisesRegex(ValueError, "unknown location"):
            parse_raw(RAW.replace("1 2 1 2", "1 999"))

    def test_recursion_and_gc_partition(self):
        report = summarize(parse_raw(RAW)["samples"])
        self.assertEqual(report["sample_count"], 4)
        self.assertEqual(report["cpu_ns"], 40000000)
        inclusive = {row["function"]: row["cpu_ns"] for row in report["top_inclusive"]}
        self.assertEqual(inclusive["app.recursive"], 20000000)
        self.assertEqual(inclusive["runtime.gcAssistAlloc"], 20000000)
        self.assertEqual(report["phase_cpu_ns"], {"bind": 20000000, "parse": 10000000, "unlabeled": 10000000})
        self.assertEqual(report["exclusive_category_cpu_ns"], {"gc_assist": 20000000, "gc_background_mark": 10000000, "other": 10000000})
        self.assertEqual(report["phase_category_cpu_ns"]["bind"], {"gc_assist": 20000000})
        self.assertEqual(sum(report["exclusive_category_cpu_ns"].values()), report["cpu_ns"])


if __name__ == "__main__":
    unittest.main()
