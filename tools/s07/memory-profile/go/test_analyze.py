import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location("memory_analyze", Path(__file__).with_name("analyze.py"))
M = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(M)

RAW = """PeriodType: space bytes
Period: 65536
Samples:
alloc_objects/count alloc_space/bytes inuse_objects/count inuse_space/bytes
  4 256 2 128: 1 2 2
                bytes:[64]
  -1 -64 -1 -64: 3
                bytes:[64]
Locations
  1: 0x123 M=1 leaf leaf.go:2:0 s=1
  2: 0x456 M=1 caller caller.go:8:0 s=5
  3: 0x789 M=1 main.profiles main.go:12:0 s=10
Mappings
1: 0x100/0x900/0x0 executable [FN]
"""


class MemoryAnalysisTests(unittest.TestCase):
    def test_signed_values_and_recursive_inclusive(self):
        metadata, samples = M.parse_raw(RAW)
        self.assertEqual(metadata["period_bytes"], 65536)
        summary = M.summarize(samples, "inuse_space")
        self.assertEqual(summary["estimated_bytes"], 64)
        self.assertEqual(summary["positive_estimated_bytes"], 128)
        self.assertEqual(summary["negative_estimated_bytes"], -64)
        self.assertEqual({r["function"]: r["estimated_bytes"] for r in summary["top_inclusive_functions"]},
                         {"leaf": 128, "caller": 128, "main.profiles": -64})
        self.assertEqual(summary["exclusive_stack_domain_bytes"],
                         {"runtime_driver_other_or_truncated": 128, "diagnostic_profile_checkpoint": -64})

    def test_positive_period_and_exact_sample_types(self):
        for text in (RAW.replace("Period: 65536", "Period: 0"),
                     RAW.replace("inuse_space/bytes", "inuse_space/count")):
            with self.assertRaises(ValueError):
                M.parse_raw(text)

    def test_empty_missing_location_duplicate_label(self):
        for text in (RAW.replace("1 2 2", "1 2 999"),
                     RAW.replace("bytes:[64]", "bytes:[64]\n                bytes:[64]", 1),
                     RAW[:RAW.index("  4 256")] + RAW[RAW.index("Locations"):]):
            with self.assertRaises(ValueError):
                M.parse_raw(text)

    def test_actual_ancestors_win_over_generic_mentions(self):
        shape = "slices.Grow[go.shape." + M.INTERNAL + "ast.(*NodeFactory).NewIdentifier]"
        frames = [(shape, "slices/slices.go", 12), (M.INTERNAL + "binder.(*Binder).newFlowNode", "binder.go", 1)]
        self.assertEqual(M.factory(frames), M.INTERNAL + "binder.(*Binder).newFlowNode")
        self.assertEqual(M.domain(frames), "compiler_binder_ancestor")
        frames.append(("main.profiles", "main.go", 1))
        self.assertEqual(M.domain(frames), "diagnostic_profile_checkpoint")

    def test_profile_hash_and_report_identity(self):
        with tempfile.TemporaryDirectory() as directory:
            # Every required snapshot and both original profile encodings are bound.
            root = Path(directory)
            report = {"workers": 1, "diagnostic_only": True, "mem_profile_rate": 65536, "profiles": []}
            for name in ("pre_pipeline", "retained_endpoint", "retained_after_gc", "post_retirement_after_gc"):
                profile = {"name": name}
                for kind in ("heap", "allocs"):
                    path = root / (name + "." + kind)
                    path.write_bytes((name + kind).encode())
                    profile[kind + "_path"] = str(path)
                    profile[kind + "_sha256"] = M.sha(path)
                report["profiles"].append(profile)
            (root / "go-1-0-report.json").write_text(json.dumps(report))
            row = {"workers": 1, "repetition": 0, "report": report}
            M.profile_inputs(root, row)
            (root / "retained_after_gc.heap").write_bytes(b"changed")
            with self.assertRaisesRegex(ValueError, "profile hash mismatch"):
                M.profile_inputs(root, row)
            report["workers"] = 8
            with self.assertRaisesRegex(ValueError, "report differs"):
                M.profile_inputs(root, row)


if __name__ == "__main__":
    unittest.main()
