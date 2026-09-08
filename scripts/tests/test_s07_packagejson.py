"""The one source-proven error rendering qualification cannot hide other drift."""
import contextlib
import copy
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import s07_packagejson as packagejson
from s07_program_helpers import packagejson_source_report


def encode(value):
    return (json.dumps(value, separators=(',', ':')) + '\n').encode()


class PackageJsonQualificationTests(unittest.TestCase):
    def setUp(self):
        self.rows = [{'id': 'array', 'parseable': False,
                      'error': 'json: cannot unmarshal JSON array into Go struct',
                      'fields': {'error': 'json: unable to preserve this user string'}}]
        self.frozen = encode(self.rows)
        self.alternate = copy.deepcopy(self.rows)
        self.alternate[0]['error'] = self.alternate[0]['error'].replace('cannot ', 'unable to ', 1)
        self.fresh = encode(self.alternate)
        self.manifest = {'version': 2, 'pin': 'a' * 40, 'requests': 1,
                         'source_sha256': {'packagejson.go': 'b' * 64},
                         'inputs_sha256': {'adapter.go': 'c' * 64}}

    def test_qualifies_only_the_demonstrated_prefix_and_retains_both_messages(self):
        report = packagejson.compare_observations(self.frozen, self.fresh)
        self.assertTrue(report['observations_equal'])
        self.assertFalse(report['raw_equal'])
        self.assertEqual(report['normalized_errors']['fresh'], [{
            'id': 'array', 'path': '/0/error', 'raw': self.alternate[0]['error'],
            'normalized': self.rows[0]['error']}])
        self.assertEqual(report['raw_diagnostics']['fresh'][0]['error'], self.alternate[0]['error'])
        self.assertEqual(packagejson.normalize_error_prefixes(self.fresh)[0], self.frozen)

    def test_changed_body_and_unrecognized_prefix_still_fail(self):
        for error in ('json: unable to unmarshal JSON string into Go struct',
                      'json: could not unmarshal JSON array into Go struct',
                      'json:\u00a0cannot unmarshal JSON array into Go struct',
                      'prefix json: unable to unmarshal JSON array into Go struct'):
            with self.subTest(error=error):
                rows = copy.deepcopy(self.rows)
                rows[0]['error'] = error
                self.assertFalse(packagejson.compare_observations(self.frozen, encode(rows))['observations_equal'])

    def test_fields_order_types_whitespace_and_success_state_remain_exact(self):
        changes = []
        for key, value in [('parseable', True), ('parseable', 0), ('fields', {}), ('id', 'changed')]:
            rows = copy.deepcopy(self.alternate)
            rows[0][key] = value
            changes.append(encode(rows))
        changes.extend([encode([dict(reversed(list(self.alternate[0].items())))]),
                        self.fresh.replace(b'"fields":', b'"fields" :')])
        for raw in changes:
            with self.subTest(raw=raw):
                try:
                    report = packagejson.compare_observations(self.frozen, raw)
                except ValueError:
                    continue
                self.assertFalse(report['observations_equal'])

    def test_nested_error_values_and_escaped_prefixes_are_not_normalized(self):
        changed = copy.deepcopy(self.rows)
        changed[0]['fields']['error'] = changed[0]['fields']['error'].replace('unable to ', 'cannot ')
        self.assertFalse(packagejson.compare_observations(self.frozen, encode(changed))['observations_equal'])
        escaped = self.fresh.replace(b'"error":"json: unable to unmarshal',
                                     b'"error":"\\u006ason: unable to unmarshal')
        self.assertFalse(packagejson.compare_observations(self.frozen, escaped)['observations_equal'])

    def test_successful_parse_error_is_not_qualified(self):
        expected = copy.deepcopy(self.rows)
        actual = copy.deepcopy(self.alternate)
        expected[0]['parseable'] = actual[0]['parseable'] = True
        self.assertEqual(packagejson.normalize_error_prefixes(encode(actual))[0], encode(actual))
        self.assertFalse(packagejson.compare_observations(encode(expected), encode(actual))['observations_equal'])

    def test_reordered_rows_cannot_pass(self):
        second = {**self.rows[0], 'id': 'second'}
        self.assertFalse(packagejson.compare_observations(
            encode([self.rows[0], second]), encode([second, self.alternate[0]]))['observations_equal'])

    def fixture(self, root):
        folder = root / 'data/s07'
        folder.mkdir(parents=True)
        (folder / 'packagejson-observations.json').write_bytes(self.frozen)
        (folder / 'packagejson-manifest.json').write_bytes(
            (json.dumps(self.manifest, sort_keys=True, separators=(',', ':')) + '\n').encode())

    def test_default_check_preserves_frozen_bytes_and_publishes_raw_source_report(self):
        with tempfile.TemporaryDirectory() as directory, contextlib.redirect_stdout(io.StringIO()):
            root = Path(directory)
            self.fixture(root)
            report = packagejson.publish_check(root, self.fresh, self.manifest)
            self.assertEqual((root / 'data/s07/packagejson-observations.json').read_bytes(), self.frozen)
            self.assertEqual(Path(report['fresh_artifact']).read_bytes(), self.fresh)
            self.assertEqual(packagejson_source_report(encode(report), self.manifest['pin']), report)
            self.assertEqual(json.loads((root / 'target/s07-packagejson/comparison.json').read_bytes()), report)

    def test_source_manifest_and_manifest_serialization_drift_fail_with_raw_output_retained(self):
        for field in ('source_sha256', 'inputs_sha256', 'serialization'):
            with self.subTest(field=field), tempfile.TemporaryDirectory() as directory, contextlib.redirect_stdout(io.StringIO()):
                root = Path(directory)
                self.fixture(root)
                current = copy.deepcopy(self.manifest)
                if field == 'serialization':
                    path = root / 'data/s07/packagejson-manifest.json'
                    path.write_bytes(path.read_bytes() + b' ')
                else:
                    current[field][next(iter(current[field]))] = 'd' * 64
                with self.assertRaisesRegex(ValueError, 'manifest'):
                    packagejson.publish_check(root, self.fresh, current)
                report = json.loads((root / 'target/s07-packagejson/comparison.json').read_bytes())
                self.assertFalse(report['manifest_current'])
                self.assertEqual(Path(report['fresh_artifact']).read_bytes(), self.fresh)

    def test_helper_rejects_missing_raw_diagnostics_or_changed_qualification(self):
        with tempfile.TemporaryDirectory() as directory, contextlib.redirect_stdout(io.StringIO()):
            root = Path(directory)
            self.fixture(root)
            report = packagejson.publish_check(root, self.fresh, self.manifest)
        for key, value in [('manifest_current', False), ('observations_equal', False),
                           ('qualification', {}), ('normalized_fresh_sha256', '0' * 64)]:
            with self.subTest(key=key):
                changed = {**report, key: value}
                with self.assertRaises(ValueError):
                    packagejson_source_report(encode(changed), self.manifest['pin'])
        del report['raw_diagnostics']
        with self.assertRaises(ValueError):
            packagejson_source_report(encode(report), self.manifest['pin'])


if __name__ == '__main__':
    unittest.main()
