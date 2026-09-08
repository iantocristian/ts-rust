"""An identical failure, missing row or false integer cannot certify verification."""
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import unittest
from s07_verify_compare import compare

class OptionVerification(unittest.TestCase):
    def test_identity_and_complete_source_observation_are_required(self):
        requests = [{'id':'a'}, {'id':'b'}]
        source = [dict(id=name, diagnostics=[], includes=[], blocked=[]) for name in ('a','b')]
        for rows in (source[:1], source[::-1], source+source[:1]):
            with self.assertRaises(ValueError): compare(requests, source, rows)
        with self.assertRaises(ValueError): compare(requests, [{'id':'a','Panic':'x'},source[1]], source)
        for changes in ({'Panic':'x'}, {'Error':'unsupported'}, {'diagnostics':None}, {'blocked':False}, {'includes':[{'code':True}]}):
            rows = [{**source[0], **changes}, source[1]]
            self.assertFalse(compare(requests, source, rows)[0]['passed'])

    def test_config_diagnostic_positions_and_order_are_observed(self):
        source = [dict(id='a',diagnostics=[{'Pos':1,'End':2,'Args':['a','b']}],includes=[],blocked=[])]
        for changed in ({'Pos':True,'End':2,'Args':['a','b']}, {'Pos':1,'End':2,'Args':['b','a']}):
            actual=[{**source[0],'diagnostics':[changed]}]
            self.assertFalse(compare([{'id':'a'}],source,actual)[0]['passed'])
