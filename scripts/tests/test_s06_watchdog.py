"""External watchdog failure paths; fake processes never contribute runtime evidence."""
import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from s06_watchdog import ROOT, read_manifest, run_watchdogs

STUB = '''import sys,json,time
r=json.loads(sys.stdin.readline())
mode=sys.argv[1]
if mode=='early-exit':sys.exit(3)
begin={'version':1,'id':r['id'],'op':r['op'],'tag':'begin'}
if mode=='wrong-id':begin['id']='wrong'
print(json.dumps(begin),flush=True)
if mode=='exit':sys.exit(3)
if mode=='return':
 print(json.dumps({'version':1,'id':r['id'],'tag':'stage','stage':'decode','outcome':'error','message_hex':'6661696c'}),flush=True)
 time.sleep(2)
if mode=='partial':sys.stdout.write('{');sys.stdout.flush()
time.sleep(2)
'''

class WatchdogTests(unittest.TestCase):
    def run_stub(self, mode, directory):
        command=[sys.executable,'-u','-c',STUB,mode]
        return run_watchdogs({'oracle':command,'rust':command},directory)

    def test_only_ready_live_children_with_no_partial_output_satisfy_watchdog(self):
        with tempfile.TemporaryDirectory() as directory:
            result=self.run_stub('loop',directory)
            self.assertTrue(result['metric'])
            self.assertEqual((result['requests'],result['runtimes']),(1,2))
            self.assertEqual([item['outcome'] for item in result['cases']],['timeout','timeout'])
            self.assertTrue((Path(directory)/'000-rust.result.json').is_file())

    def test_return_is_measured_failure_and_retains_the_observed_frame(self):
        with tempfile.TemporaryDirectory() as directory:
            result=self.run_stub('return',directory)
            self.assertFalse(result['metric'])
            self.assertEqual([item['outcome'] for item in result['cases']],['returned','returned'])
            self.assertTrue((Path(directory)/'000-oracle.unexpected.json').is_file())

    def test_crashes_missing_readiness_and_partial_records_invalidate_capture(self):
        for mode in ('early-exit','exit','wrong-id','partial'):
            with self.subTest(mode=mode),tempfile.TemporaryDirectory() as directory:
                with self.assertRaises(RuntimeError):self.run_stub(mode,directory)
                self.assertTrue((Path(directory)/'000-oracle.invalid.txt').is_file())

    def test_missing_duplicate_changed_wire_and_unreviewed_deadlines_are_rejected(self):
        original=json.loads((ROOT/'data/s06/watchdogs.json').read_text())
        cases=[]
        changed=copy.deepcopy(original);changed['cases']=[];cases.append(changed)
        changed=copy.deepcopy(original);changed['cases']*=2;cases.append(changed)
        changed=copy.deepcopy(original);changed['cases'][0]['after_begin_deadline_seconds']=1;cases.append(changed)
        changed=copy.deepcopy(original);changed['cases'][0]['request']['wire_hex']='00'*128;cases.append(changed)
        changed=copy.deepcopy(original);changed['pin']='wrong';cases.append(changed)
        with tempfile.TemporaryDirectory() as directory:
            path=Path(directory)/'manifest.json'
            for changed in cases:
                path.write_text(json.dumps(changed))
                with self.assertRaises(ValueError):read_manifest(path)

if __name__=='__main__':unittest.main()
