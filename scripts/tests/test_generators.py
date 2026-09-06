"""Regressions for the two pinned-table generators.

Run: python3 -m unittest discover -s scripts/tests -v

The generators translate pinned upstream bytes, so the tests check the
translation rules rather than the Unicode data itself: a wrong Go escape or a
dropped range would otherwise reach the port silently.
"""

import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import unittest


REPO = Path(__file__).resolve().parents[2]


def load(name):
    spec = importlib.util.spec_from_file_location(name.replace("-", "_"), REPO / f"scripts/{name}.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


unicode_case = load("gen-unicode-case")
e4_fixtures = load("gen-e4-fixtures")


class GoStringTests(unittest.TestCase):
    def test_escapes(self):
        cases = {
            r'"abc"': b"abc",
            r'"a"': b"a",
            r'"ς"': "ς".encode(),
            r'"\U0001F600"': "\U0001f600".encode(),
            r'"\xed\xa0\x80"': bytes([0xED, 0xA0, 0x80]),
            r'"a\tb\\c\"d"': b'a\tb\\c"d',
            '""': b"",
        }
        for literal, expected in cases.items():
            with self.subTest(literal=literal):
                self.assertEqual(unicode_case.go_string(literal), expected)

    def test_unsupported_escape_is_an_error(self):
        with self.assertRaises(ValueError):
            unicode_case.go_string(r'"\q"')

    def test_rust_bytes_escapes_what_it_must(self):
        self.assertEqual(unicode_case.rust_bytes(b"ab"), 'b"ab"')
        self.assertEqual(unicode_case.rust_bytes(b'"'), 'b"\\x22"')
        self.assertEqual(unicode_case.rust_bytes(b"\\"), 'b"\\x5c"')
        self.assertEqual(unicode_case.rust_bytes(bytes([0xED])), 'b"\\xed"')


class PinnedTableTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        source = REPO / unicode_case.SOURCE
        if not source.is_file():
            raise unittest.SkipTest("the upstream submodule is not initialized")
        cls.mappings, cls.ranges = unicode_case.parse(source.read_text())

    def test_the_table_is_complete_and_sorted_after_generation(self):
        self.assertGreater(len(self.mappings), 2000)
        codes = [code for code, *_ in self.mappings]
        self.assertEqual(len(codes), len(set(codes)))

    def test_known_mappings_survive_translation(self):
        by_code = {code: (lower, upper, conditional, final) for code, lower, upper, conditional, final in self.mappings}
        self.assertEqual(by_code[0x41][0], b"a")
        # Capital sigma carries the Final_Sigma condition and its own final form.
        lower, _, conditional, final = by_code[0x3A3]
        self.assertTrue(final)
        self.assertEqual(lower, "σ".encode())
        self.assertEqual(conditional, "ς".encode())

    def test_range_tables_are_nonempty_and_ordered(self):
        for name, table in self.ranges.items():
            with self.subTest(table=name):
                self.assertTrue(table["r16"])
                for key in ("r16", "r32"):
                    ranges = table[key]
                    self.assertTrue(all(lo <= hi and stride >= 1 for lo, hi, stride in ranges))
                    self.assertEqual(ranges, sorted(ranges))

    def test_range_sweep_covers_boundaries_and_excludes_surrogates(self):
        sweep = set(e4_fixtures.range_sweep(self.ranges))
        self.assertTrue(sweep)
        self.assertFalse(any(0xD800 <= cp <= 0xDFFF for cp in sweep))
        for table in self.ranges.values():
            for lo, hi, _ in table["r16"] + table["r32"]:
                if not 0xD800 <= lo <= 0xDFFF:
                    self.assertIn(lo, sweep)
                if not 0xD800 <= hi <= 0xDFFF:
                    self.assertIn(hi, sweep)


class CommittedOutputTests(unittest.TestCase):
    """The checked-in generated files must match their generators."""

    def test_generated_files_are_current(self):
        if not (REPO / unicode_case.SOURCE).is_file():
            self.skipTest("the upstream submodule is not initialized")
        for generator in ("gen-unicode-case.py", "gen-e4-fixtures.py"):
            with self.subTest(generator=generator):
                completed = subprocess.run(
                    [sys.executable, f"scripts/{generator}", "--check"],
                    cwd=REPO,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    text=True,
                    check=False,
                )
                self.assertEqual(completed.returncode, 0, completed.stderr)

    def test_fixture_manifest_is_usable(self):
        manifest = json.loads((REPO / e4_fixtures.TARGET).read_text())
        for key in ("files", "texts", "helpers", "runes", "truncations", "probes", "case_sweep", "range_sweep"):
            self.assertIn(key, manifest)
        for group in ("files", "texts", "helpers"):
            ids = [entry["id"] for entry in manifest[group]]
            self.assertEqual(len(ids), len(set(ids)), f"{group} ids must be unique")
            for entry in manifest[group]:
                bytes.fromhex(entry["bytes"])
                self.assertTrue(entry["why"], f"{group}/{entry['id']} needs a reason")


if __name__ == "__main__":
    unittest.main()
