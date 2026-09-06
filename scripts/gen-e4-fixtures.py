#!/usr/bin/env python3
"""Generate the E4 fixture manifest.

The fixtures are the ones docs/design/text.md, section 3 names: BOM and UTF-16
decoding, WTF-8 sentinels, malformed bytes, astral characters, the two line-break
sets, byte slices that split a sequence, and the leaf helpers. Everything is hex
so a malformed byte survives the file.

The case sweep is taken from the pinned Go case table, so the comparison covers
every code point Unicode 15.1.0 gives a case mapping rather than a hand-picked
few.

    python3 scripts/gen-e4-fixtures.py            # write data/e4-fixtures.json
    python3 scripts/gen-e4-fixtures.py --check    # fail if it would change
"""

import argparse
import importlib.util
import json
import re
import sys
from pathlib import Path

TARGET = "data/e4-fixtures.json"
PINNED_CASE_TABLE = "upstream/tsc/internal/stringutil/js_case_generated.go"

# A lone high surrogate U+D800 and low surrogate U+DC00 as WTF-8 sentinels, and
# the halves of U+1F600.
HIGH = bytes([0xED, 0xA0, 0x80])
LOW = bytes([0xED, 0xB0, 0x80])
GRINNING_HIGH = bytes([0xED, 0xA0, 0xBD])
GRINNING_LOW = bytes([0xED, 0xB8, 0x80])
GRINNING = "\U0001F600".encode()


def u(text):
    return text.encode("utf-8")


FILES = [
    ("plain", b"abc", "no byte order mark; bytes are kept unchanged"),
    ("empty", b"", "an empty file"),
    ("utf8_bom", b"\xef\xbb\xbfabc", "a UTF-8 BOM is dropped"),
    ("utf8_bom_only", b"\xef\xbb\xbf", "a file that is nothing but a UTF-8 BOM"),
    ("utf8_bom_prefix", b"\xef\xbb", "an incomplete BOM prefix is data"),
    ("utf8_bom_then_sentinel", b"\xef\xbb\xbf" + HIGH, "a sentinel survives BOM removal"),
    ("utf16le_bom", b"\xff\xfea\x00b\x00", "UTF-16 LE decodes to UTF-8"),
    ("utf16be_bom", b"\xfe\xff\x00a\x00b", "UTF-16 BE decodes to UTF-8"),
    ("utf16le_pair", b"\xff\xfe\x3d\xd8\x00\xde", "a surrogate pair decodes to one code point"),
    ("utf16le_unpaired_high", b"\xff\xfe\x00\xd8", "an unpaired high surrogate becomes U+FFFD"),
    ("utf16be_unpaired_low", b"\xfe\xff\xdc\x00", "an unpaired low surrogate becomes U+FFFD"),
    ("utf16le_odd", b"\xff\xfea\x00b", "a trailing odd byte is dropped"),
    ("utf16le_bom_only", b"\xff\xfe", "a file that is nothing but a UTF-16 BOM"),
    ("malformed", b"a\xffb", "a malformed byte is kept; nothing validates UTF-8"),
    ("one_byte", b"\xff", "a one-byte file is too short for any BOM"),
    ("high_bytes", b"\xff\x00", "a two-byte file that is not a BOM"),
    ("sentinel", HIGH, "a WTF-8 sentinel in the file bytes"),
]

TEXTS = [
    ("empty", b"", "no text at all"),
    ("ascii", u("const a = 1;"), "the ASCII fast path"),
    ("latin1", u("a\u00e9b"), "a two-byte sequence, one UTF-16 unit"),
    ("bmp3", u("a\u20acb"), "a three-byte sequence, one UTF-16 unit"),
    ("astral", GRINNING, "a four-byte sequence, two UTF-16 units"),
    ("astral_in_line", u("a") + GRINNING + u("b"), "an astral character between ASCII"),
    ("sentinel", HIGH, "one WTF-8 sentinel: one API unit, three LSP units"),
    ("sentinel_pair", GRINNING_HIGH + GRINNING_LOW, "two adjacent sentinels"),
    ("sentinel_in_text", u("a") + HIGH + u("b"), "a sentinel between ASCII"),
    ("malformed", b"a\xffb", "a malformed byte counts as one unit everywhere"),
    ("cut_astral", GRINNING[:2], "half an astral sequence"),
    ("cut_sentinel", HIGH[:2], "half a sentinel"),
    ("lf", u("a\nb"), "a line feed"),
    ("cr", u("a\rb"), "a carriage return"),
    ("crlf", u("a\r\nb"), "a CRLF pair is one line break"),
    ("cr_lf_mixed", u("a\r\nb\rc\nd"), "every ASCII line break in one text"),
    ("line_separator", u("a\u2028b"), "U+2028 ends an ECMAScript line but not an LSP line"),
    ("paragraph_separator", u("a\u2029b"), "U+2029, likewise"),
    ("separators_and_lf", u("a\u2028b\nc\u2029d"), "the two line-break sets disagree twice"),
    ("trailing_lf", u("a\n"), "a trailing line break"),
    ("only_lf", u("\n"), "nothing but a line break"),
    ("multiline_astral", u("a\n") + GRINNING + u("\nb"), "an astral character on its own line"),
    ("nul_and_digit", b"\x000", "a null followed by a digit"),
]

HELPERS = [
    ("empty", b"", "no bytes"),
    ("ascii_lower", u("abc"), "the ASCII fast path with no mapping"),
    ("ascii_upper", u("ABC"), "the ASCII fast path with a mapping"),
    ("ascii_mixed", u("AbC0_"), "mixed ASCII"),
    ("sentinel_high", HIGH, "a complete high-surrogate sentinel"),
    ("sentinel_low", LOW, "a complete low-surrogate sentinel"),
    ("sentinel_pair", GRINNING_HIGH + GRINNING_LOW, "adjacent halves of U+1F600"),
    ("sentinel_reversed", GRINNING_LOW + GRINNING_HIGH, "a low half before a high half"),
    ("sentinel_then_ascii", HIGH + u("abc"), "a sentinel prefix"),
    ("cut_sentinel", HIGH[:2], "half a sentinel"),
    ("cut_sentinel_tail", HIGH[1:], "the tail of a sentinel"),
    ("malformed_ff", b"\xff", "a byte no decoder accepts"),
    ("malformed_in_ascii", b"a\xffb", "a malformed byte between ASCII"),
    ("cut_astral", GRINNING[:2], "half an astral sequence"),
    ("astral", GRINNING, "a whole astral character"),
    ("replacement", u("\ufffd"), "a real U+FFFD, three valid bytes"),
    ("latin1", u("\u00e9\u00c9"), "Latin-1 case pairs"),
    ("sharp_s", u("\u00df\u1e9e"), "the sharp s and its capital"),
    ("dotted_i", u("\u0130\u0131"), "the Turkish dotted and dotless I"),
    ("ligature", u("\ufb04"), "a ligature whose uppercase is three runes"),
    ("greek_final_sigma", u("\u039f\u0394\u039f\u03a3"), "a sigma in Final_Sigma context"),
    ("greek_medial_sigma", u("\u03a3\u03a3\u0391"), "a sigma that is not final"),
    ("greek_sigma_then_ignorable", u("\u039f\u03a3\u0301"), "a sigma followed by a case-ignorable mark"),
    ("combining", u("a\u0301"), "a combining mark"),
    ("quotes", u("a\"b'c`d"), "every quote character"),
    ("dollar_brace", u("a${b}"), "a template substitution opener"),
    ("backslash", u("a\\b"), "a backslash"),
    ("controls", b"\x00\x08\x09\x0b\x0c\x0d\x0a\x1f", "the escaped control characters"),
    ("crlf", u("a\r\nb"), "a CRLF pair, which templates escape as one unit"),
    ("nul_digit", b"a\x000b", "a null followed by a digit"),
    ("separators", u("\u2028\u2029\u0085"), "the separators that are always escaped"),
    ("non_ascii", u("\u00e9\u4e2d"), "non-ASCII that only some flags escape"),
    ("long_ascii", u("abcdefghij"), "a longer ASCII run for truncation"),
    ("mixed_widths", u("a\u00e9\u20ac") + GRINNING, "one, two, three and four byte sequences"),
]

# Code points handed to EncodeJSStringRune, including values it must fold to
# U+FFFD.
RUNES = [
    -1, 0, 0x41, 0x7F, 0x80, 0x7FF, 0x800, 0xD7FF, 0xD800, 0xDBFF, 0xDC00,
    0xDFFF, 0xE000, 0xFFFD, 0xFFFF, 0x10000, 0x1F600, 0x10FFFF, 0x110000,
]

TRUNCATIONS = [-1, 0, 1, 2, 3, 4, 5, 10, 40]

PROBES = {
    # Byte offsets outside every fixture text, to pin down each path's clamping,
    # arithmetic or panic behavior.
    "out_of_range_offsets": [-1, 64],
    # UTF-16 character offsets requested of the converters.
    "characters": [0, 1, 2, 3, 4, 8, 64],
    # Line numbers outside the line map.
    "out_of_range_lines": [-1, 64],
}


def pinned_tables(root):
    spec = importlib.util.spec_from_file_location("gen", root / "scripts/gen-unicode-case.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.parse((root / PINNED_CASE_TABLE).read_text())


def range_sweep(ranges):
    """Boundary code points of the Cased and Case_Ignorable range tables.

    ToLowerJS reads those tables only to decide the Final_Sigma context, so a
    transcription error in them is invisible unless a sigma is nearby. The
    harness surrounds each of these code points with a sigma in two contexts,
    which separates "case ignorable", "cased" and "neither" from each other.
    Surrogates are excluded: they are not scalar values, and their own handling
    is covered by the sentinel fixtures.
    """
    points = set()
    for table in ranges.values():
        for key, in_range in (("r16", None), ("r32", None)):
            for lo, hi, stride in table[key]:
                candidates = {lo - 1, lo, hi, hi + 1}
                if stride > 1:
                    candidates |= {lo + stride, hi - stride, lo + 1}
                elif hi > lo:
                    candidates |= {(lo + hi) // 2}
                points |= candidates
            _ = in_range
    return sorted(
        cp for cp in points if 0 <= cp <= 0x10FFFF and not 0xD800 <= cp <= 0xDFFF
    )


def build(root):
    mappings, ranges = pinned_tables(root)
    pin = re.search(r'"pin"\s*:\s*"([0-9a-f]{40})"', (root / "data/upstream.json").read_text())
    return {
        "version": 1,
        "generated_by": "scripts/gen-e4-fixtures.py",
        "upstream_pin": pin.group(1),
        "note": (
            "Fixtures for the E4 criteria S04 settles. Both the Go oracle and the "
            "Rust harness read this file and emit one value per derived probe; the "
            "producer compares them. Bytes are hex so malformed input survives."
        ),
        "files": [{"id": i, "bytes": b.hex(), "why": w} for i, b, w in FILES],
        "texts": [{"id": i, "bytes": b.hex(), "why": w} for i, b, w in TEXTS],
        "helpers": [{"id": i, "bytes": b.hex(), "why": w} for i, b, w in HELPERS],
        "runes": RUNES,
        "truncations": TRUNCATIONS,
        "probes": PROBES,
        "case_sweep": sorted(code for code, *_ in mappings),
        "range_sweep": range_sweep(ranges),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="fail instead of writing")
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    rendered = json.dumps(build(root), indent=2, sort_keys=True) + "\n"
    target = root / TARGET
    if args.check:
        if not target.is_file() or target.read_text() != rendered:
            print(f"{TARGET} is out of date; run scripts/gen-e4-fixtures.py", file=sys.stderr)
            return 1
        print(f"{TARGET} matches its generator")
        return 0
    target.write_text(rendered)
    print(f"wrote {TARGET}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
