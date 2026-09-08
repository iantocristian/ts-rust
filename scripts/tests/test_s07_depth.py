"""The depth gate rejects incomplete or manufactured instrumentation."""
import copy
from contextlib import redirect_stderr, redirect_stdout
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import Mock, patch
import s07_depth
from s04_common import strict_json_loads
from s07_depth import native_rows

class DepthTests(unittest.TestCase):
    def setUp(self):
        self.inventory = {'small_stack':[{'id':'binary','source_hex':'78','require_binary':True},{'id':'body','source_hex':'78','require_growth':True}], 'constructed':[{'id':'unwind','require_growth':True,'terminal_failure':True}]}
        self.rows = [{'id':'binary','guard_entries':40005,'actual_segment_growths':0,'binary_nodes':20000,'max_binary_frames':40000},{'id':'body','guard_entries':900,'actual_segment_growths':1,'binary_nodes':0,'max_binary_frames':0},{'id':'unwind','guard_entries':900,'actual_segment_growths':1,'terminal_failure':True}]
    def wire(self, rows):
        return ('test name ... S07_BINDER_DEPTH:'+json.dumps(rows)+'\n').encode()
    def test_complete_rows(self):
        self.assertEqual(native_rows(self.wire(self.rows),self.inventory),self.rows)
    def test_missing_duplicate_extra(self):
        for rows in [self.rows[:-1],self.rows+self.rows[:1],self.rows+[{'id':'extra'}],[]]:
            with self.subTest(rows=rows),self.assertRaises(ValueError):native_rows(self.wire(rows),self.inventory)
    def test_counter_and_failure_gates(self):
        for index,key,value in [(0,'binary_nodes',0),(0,'max_binary_frames',20000),(1,'actual_segment_growths',0),(1,'actual_segment_growths',True),(2,'terminal_failure',False),(2,'guard_entries',0)]:
            rows=copy.deepcopy(self.rows);rows[index][key]=value
            with self.subTest(key=key,value=value),self.assertRaises(ValueError):native_rows(self.wire(rows),self.inventory)
    def test_unknown_counter_is_not_silently_consumed(self):
        rows=copy.deepcopy(self.rows);rows[0]['fake_growth']=1
        with self.assertRaises(ValueError):native_rows(self.wire(rows),self.inventory)
    def test_malformed_record(self):
        for wire in [b'S07_BINDER_DEPTH:{"id":"binary"}',b'S07_BINDER_DEPTH:[NaN]',b'S07_BINDER_DEPTH:[{"id":"a","id":"b"}]']:
            with self.subTest(wire=wire),self.assertRaises(ValueError):native_rows(wire,self.inventory)

    def test_embedded_capture_reserves_stdout_for_the_callers_single_json_report(self):
        # Only external processes are replaced. The real capture loop, request
        # validation, progress output and report serialization execute, for both
        # an equal graph and a failed comparison. This tests transport, not parity.
        inventory = strict_json_loads(s07_depth.CASES.read_bytes())
        inventory['graph_requests'] = inventory['graph_requests'][:1]
        request = inventory['graph_requests'][0]
        with tempfile.TemporaryDirectory(prefix='s07-depth-stdout-') as temporary:
            root = Path(temporary)
            cases = root/'cases.json'
            cases.write_text(json.dumps(inventory))
            binary = root/'binary'
            binary.write_bytes(b'process replacement for the output-channel regression')
            for equal in (True, False):
                oracle, rust = Mock(), Mock()
                comparison = {'equal':equal, 'first_difference':None if equal else {'field':'flags'},
                              'stages':{name:[{'outcome':'ok'}] for name in ('oracle','rust')}}
                stdout, stderr = io.StringIO(), io.StringIO()
                with self.subTest(equal=equal), patch.object(s07_depth,'CASES',cases), \
                     patch.object(s07_depth,'inputs',return_value={'source':'unchanged'}), \
                     patch.object(s07_depth,'native_capture',return_value={'transport_test_only':True}), \
                     patch.object(s07_depth,'build_oracle',return_value=(binary,'pin')), \
                     patch.object(s07_depth,'rust_binary',return_value=binary), \
                     patch.object(s07_depth,'Process',side_effect=[oracle,rust]), \
                     patch.object(s07_depth,'compare',return_value=comparison), \
                     redirect_stdout(stdout), redirect_stderr(stderr):
                    report = s07_depth.capture(root/str(equal))
                    print(json.dumps({'metrics':report['metrics']}))
                self.assertEqual(strict_json_loads(stdout.getvalue()), {'metrics':{'binder_depth':equal}})
                self.assertEqual(len(stdout.getvalue().splitlines()),1)
                self.assertIn(request['id'],stderr.getvalue())
                self.assertIn('passed' if equal else 'flags',stderr.getvalue())
                for process in (oracle,rust):
                    process.send.assert_called_once_with(request)
                    process.close.assert_called_once_with()

if __name__ == '__main__':unittest.main()
