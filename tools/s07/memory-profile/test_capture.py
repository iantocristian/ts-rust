import importlib.util
import sys
from pathlib import Path
import unittest

HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
import capture

class FrozenWorkTests(unittest.TestCase):
    def test_exact_counters_and_worker_type(self):
        expected={'files':13,'loaded_input_sha256':'abc'}
        capture.validate_work({**expected,'workers':1,'diagnostic_only':True},expected,1)
        for bad in ({'files':12},{'files':True},{'workers':True},{'loaded_input_sha256':'def'},{'diagnostic_only':False}):
            with self.subTest(bad=bad),self.assertRaises(ValueError):
                capture.validate_work({**expected,'workers':1,'diagnostic_only':True,**bad},expected,1)
    def test_native_and_forced_gc_checkpoints_are_distinct(self):
        self.assertLess(capture.CHECKPOINTS['go'].index('retained_endpoint'),capture.CHECKPOINTS['go'].index('retained_after_gc'))
        for points in capture.CHECKPOINTS.values(): self.assertEqual(len(points),len(set(points)))

if __name__=='__main__':unittest.main()
