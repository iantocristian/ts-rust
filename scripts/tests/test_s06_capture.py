"""S06 adapter startup failures must release every previously started child."""
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import s06
from s06_protocol import canonical, parser_request
from test_s06_results import compare, frames


class StartupTests(unittest.TestCase):
    def test_second_adapter_launch_failure_closes_the_first(self):
        request = parser_request('startup-test', None, b'', '/a.ts', '/a.ts')
        oracle = Mock()
        with tempfile.TemporaryDirectory() as temporary:
            with patch.object(s06, 'ROOT', Path(temporary)), \
                 patch.object(s06, 'command'), \
                 patch.object(s06, 'build_oracle', return_value=Path('/oracle')), \
                 patch.object(s06, 'parser_binary', return_value='/rust'), \
                 patch.object(s06, 'measure_utilities', return_value={'metric':True,'tests':2,'groups':4}), \
                 patch.object(s06, 'run_watchdogs', return_value={'metric':True,'requests':1,'runtimes':2}), \
                 patch.object(s06, 'Process', side_effect=[oracle, OSError('missing Rust adapter')]):
                with self.assertRaisesRegex(OSError, 'missing Rust adapter'):
                    s06.capture({'fixtures.json': {'fixtures': []}}, [request])
        oracle.close.assert_called_once_with()

    def test_watchdog_failure_is_measured_separately_and_does_not_skip_parity(self):
        request = parser_request('watchdog-test', None, b'', '/a.ts', '/a.ts')
        oracle, rust = Mock(), Mock()
        result = compare(request)
        with tempfile.TemporaryDirectory() as temporary:
            with patch.object(s06, 'ROOT', Path(temporary)), \
                 patch.object(s06, 'command'), \
                 patch.object(s06, 'verified_upstream'), \
                 patch.object(s06, 'build_oracle', return_value=Path('/oracle')), \
                 patch.object(s06, 'parser_binary', return_value='/rust'), \
                 patch.object(s06, 'measure_utilities', return_value={'metric':False,'tests':2,'groups':4}), \
                 patch.object(s06, 'run_watchdogs', return_value={'metric':False,'requests':1,'runtimes':2}) as watchdog, \
                 patch.object(s06, 'Process', side_effect=[oracle, rust]), \
                 patch.object(s06, 'compare_case', return_value=result), \
                 patch.object(s06, 'report', return_value={'metrics':{'encoder_output_bytes':True},'tests':{}}) as report:
                output = s06.capture({'fixtures.json': {'fixtures': []},'cases.json':[],'probes.json':{}}, [request])
                watchdog.assert_called_once_with({'oracle':['/oracle'],'rust':['/rust']}, Path(temporary)/'target/s06-reports/watchdogs')
                self.assertEqual(len(report.call_args.args[0]), 1)
                self.assertEqual(len(report.call_args.args[1]), 1)
                self.assertFalse(output['metrics']['decoder_watchdog'])
                self.assertFalse(output['metrics']['ast_utilities'])
                self.assertTrue(output['metrics']['encoder_output_bytes'])
                self.assertEqual(output['metrics']['decoder_watchdog_requests'], 1)
                self.assertEqual(output['metrics']['decoder_watchdog_runtimes'], 2)
                self.assertTrue((Path(temporary)/'target/s06-reports/watchdogs.json').is_file())
        oracle.finish.assert_called_once_with()
        rust.finish.assert_called_once_with()
        oracle.close.assert_called_once_with()
        rust.close.assert_called_once_with()


class DeadlineTests(unittest.TestCase):
    def request(self):
        return {'version':1,'id':'deadline','primary':None,'op':'path','path_hex':'2f'}

    def test_completed_records_cannot_keep_one_request_alive_forever(self):
        from s06_process import Process
        code = 'import sys,time; sys.stdin.readline(); [(print("{}",flush=True),time.sleep(.03)) for _ in range(40)]'
        with tempfile.TemporaryDirectory() as temporary:
            process = Process([sys.executable,'-u','-c',code],Path(temporary)/'stderr',deadline=.3)
            try:
                process.send(self.request())
                count = 0
                with self.assertRaises(TimeoutError):
                    while True:
                        process.read()
                        count += 1
                self.assertGreater(count, 1)
            finally:
                process.close()

    def test_buffered_records_are_subject_to_the_request_deadline(self):
        from s06_process import Process
        with tempfile.TemporaryDirectory() as temporary:
            process = Process([sys.executable,'-c','import time; time.sleep(10)'],Path(temporary)/'stderr')
            try:
                process.buffer.extend(b'{}\n')
                process.request_expires = 9
                with patch('s06_process.time.monotonic', return_value=10), self.assertRaisesRegex(TimeoutError,'request deadline'):
                    process.read()
                self.assertEqual(process.buffer, b'{}\n')
            finally:
                process.close()

    def test_validated_end_clears_the_deadline_before_the_next_request(self):
        from s06_process import Process
        request = parser_request('completed',None,b'', '/a.ts','/a.ts')
        with tempfile.TemporaryDirectory() as temporary:
            process = Process([sys.executable,'-c','import time; time.sleep(10)'],Path(temporary)/'stderr')
            try:
                process.send(request)
                with self.assertRaisesRegex(RuntimeError,'previous S06 request'):
                    process.send(request)
                process.buffer.extend(b''.join(canonical(record)+b'\n' for record in frames(request)))
                self.assertEqual(list(process.observations(request)), frames(request))
                self.assertIsNone(process.request_expires)
                process.send(request)
                self.assertIsNotNone(process.request_expires)
            finally:
                process.close()

    def test_launch_failure_closes_its_stderr_file(self):
        from s06_process import Process
        log = Mock()
        with patch('s05_protocol.open', return_value=log), patch('s05_protocol.subprocess.Popen', side_effect=OSError('missing adapter')):
            with self.assertRaisesRegex(OSError, 'missing adapter'):
                Process(['/missing'], Path('/unused'))
        log.close.assert_called_once_with()


if __name__ == '__main__':
    unittest.main()
