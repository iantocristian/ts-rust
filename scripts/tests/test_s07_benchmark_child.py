"""Exercise actual per-child resource accounting and bounded failure cleanup."""
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import unittest
from s07_benchmark_child import capture

class ChildAccounting(unittest.TestCase):
    def test_previous_heavy_child_does_not_supply_the_next_peak(self):
        heavy = capture([sys.executable, "-c", "data=bytearray(80*1024*1024); print('{}')"])
        light = capture([sys.executable, "-c", "print('{}')"])
        self.assertGreater(heavy['peak_rss_bytes'], light['peak_rss_bytes'] + 40*1024*1024)
        self.assertGreater(light['process_time_ns'], 0)
        self.assertEqual(light['report'], {})

    def test_failure_timeout_and_oversized_output_cannot_be_samples(self):
        for code, timeout in (("raise SystemExit(1)", 3), ("import time; time.sleep(10)", .05), ("print('x'*65537)", 3), ("print('{bad json}')", 3)):
            with self.subTest(code=code), self.assertRaises(ValueError):
                capture([sys.executable, "-c", code], timeout=timeout)
