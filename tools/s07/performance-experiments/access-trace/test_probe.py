"""Bounded output and semantic-control rejection paths."""
import io
from pathlib import Path
import tempfile
import unittest

from probe import CappedOutput, graph_match


class ProducerTests(unittest.TestCase):
    def test_cap_rejects_before_writing_over_budget(self):
        raw = io.BytesIO()
        sink = CappedOutput(raw, 5)
        sink.write(b'abc')
        with self.assertRaisesRegex(ValueError, 'incomplete'):
            sink.write(b'def')
        self.assertEqual(raw.getvalue(), b'abc')
        self.assertEqual(sink.written, 3)

    def test_graph_mismatch_and_missing_or_extra_file_fail(self):
        with tempfile.TemporaryDirectory() as temp:
            left, right = Path(temp) / 'left', Path(temp) / 'right'
            left.write_bytes(b'{"index":0,"value":1}\n')
            right.write_bytes(b'{"index":0,"value":1}\n')
            self.assertEqual(graph_match(left, right), 1)
            for value in (b'{"index":0,"value":2}\n', b'',
                          b'{"index":0,"value":1}\n{"index":1}\n'):
                right.write_bytes(value)
                with self.assertRaises(ValueError):
                    graph_match(left, right)


if __name__ == '__main__':
    unittest.main()
