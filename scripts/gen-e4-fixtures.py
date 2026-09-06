#!/usr/bin/env python3
"""Generate tests/e4/fixtures.json, the E4 fixture set from docs/design/text.md.

Inputs are hex-encoded bytes so malformed bytes and WTF-8 sentinels are
representable. The Go oracle (tools/oracle-e4) and the Rust harness
(crates/e4_harness) both read this file; the harness compares its results with
the oracle's. Regenerate with this script; the output is committed and declared
as the e4 producer's input.
"""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "tests/e4/fixtures.json"

HIGH = b"\xed\xa0\x80"  # U+D800 sentinel
HIGH_SMILE = b"\xed\xa0\xbd"  # U+D83D
LOW_SMILE = b"\xed\xb8\x80"  # U+DE00
SMILE = "\U0001F600".encode()
LS = " ".encode()
PS = " ".encode()


def hx(b: bytes) -> str:
    return b.hex().upper()


def fixture(id_, data, **extra):
    return {"id": id_, "input": hx(data), **extra}


decode = [
    fixture("plain", b"abc"),
    fixture("utf8-bom", b"\xef\xbb\xbfabc"),
    fixture("utf16le-bom", b"\xff\xfea\x00b\x00"),
    fixture("utf16be-bom", b"\xfe\xff\x00a\x00b"),
    fixture("utf16le-astral", b"\xff\xfe\x3d\xd8\x00\xde"),
    fixture("utf16le-lone-high", b"\xff\xfe\x00\xd8a\x00"),
    fixture("utf16be-lone-low", b"\xfe\xff\xdc\x00\x00a"),
    fixture("utf16le-odd-trailing", b"\xff\xfea\x00b"),
    fixture("utf16le-empty", b"\xff\xfe"),
    fixture("malformed-no-bom", b"\xffabc\xed\xa0\x80"),
    fixture("short", b"\xef"),
    fixture("bom-only", b"\xef\xbb\xbf"),
]

helper_inputs = [
    ("ascii-mixed", b"Hello World"),
    ("ascii-upper", b"ABC"),
    ("sentinel-high", b"A" + HIGH + b"B"),
    ("sentinel-pair", HIGH_SMILE + LOW_SMILE),
    ("sentinel-only", HIGH),
    ("malformed-ff", b"A\xff"),
    ("malformed-prefix", b"\xffabc"),
    ("sentinel-prefix", HIGH + b"xyz"),
    ("truncated-sequence", b"a\xe2\x82"),
    ("final-sigma", "ΟΔΟΣ".encode()),
    ("sigma-not-final", "ΟΔΟΣ Σ".encode()),
    ("sigma-ignorable", "ΟΔΟΣ.".encode()),
    ("sharp-s", "ß".encode()),
    ("dotted-i", "İstanbul".encode()),
    ("astral", SMILE + b"a"),
    ("empty", b""),
    ("mixed", "éÉ".encode() + HIGH + b"\xff" + SMILE),
]
helpers = [fixture(id_, data, truncate=[-1, 0, 1, 2, 3, len(data), len(data) + 1]) for id_, data in helper_inputs]

escape_inputs = [
    ("quotes", b"a\"b'c`d"),
    ("backslash", b"a\\b"),
    ("newlines", b"a\r\nb\nc\rd"),
    ("template-dollar", b"${x} $y"),
    ("nul-digit", b"\x001"),
    ("nul-letter", b"\x00a"),
    ("control", b"\x01\x1f\x7f"),
    ("separators", b"a" + LS + b"b" + PS + b"c\xc2\x85d"),
    ("non-ascii", "é".encode()),
    ("astral", SMILE),
    ("sentinel-high", HIGH),
    ("sentinel-pair", HIGH_SMILE + LOW_SMILE),
    ("malformed", b"a\xffb"),
    ("mixed", b"\"" + HIGH + b"\xff" + SMILE + b"\\\n"),
]
escape = []
for id_, data in escape_inputs:
    for quote in ('"', "'", "`"):
        for flags in (0, 1, 3):
            escape.append(fixture(f"{id_}/{quote}/{flags}", data, quote=quote, flags=flags))

slice_inputs = [
    ("astral", b"a" + SMILE + b"b"),
    ("sentinel", HIGH + b"x"),
    ("two-byte", "aéb".encode()),
    ("malformed", b"a\xffb"),
]
slices = []
for id_, data in slice_inputs:
    n = len(data)
    ranges = [[a, b] for a in range(n + 1) for b in range(a, n + 1)]
    slices.append(fixture(id_, data, ranges=ranges))

text_inputs = [
    ("empty", b""),
    ("ascii", b"ab\ncd"),
    ("crlf-mix", b"a\r\nb\rc\nd\r\n"),
    ("astral", SMILE),
    ("astral-lines", b"x" + SMILE + b"y\n" + SMILE + SMILE + b"\nz"),
    ("ls", b"a" + LS + b"b"),
    ("ps", b"a" + PS + b"b"),
    ("sentinel", b"x" + HIGH + b"y"),
    ("sentinel-lines", HIGH + b"\n" + HIGH + HIGH + b"a"),
    ("malformed", b"a\xffb\n\xff"),
    ("two-byte", "aé\nébé".encode()),
    ("mixed", "é".encode() + SMILE + b"\n" + HIGH + b"\n\xff\r\n" + b"end"),
    ("trailing-newline", b"a\n"),
    ("only-newlines", b"\n\n"),
]
texts = []
for id_, data in text_inputs:
    n = len(data)
    byte_offsets = list(range(0, n + 3))
    utf16_offsets = list(range(0, n + 3))
    lines = max(1, data.count(b"\n") + data.count(b"\r") + 3)
    positions = [[line, char] for line in range(-1, lines + 1) for char in range(0, n + 3)]
    texts.append({"id": id_, "text": hx(data), "utf16_offsets": utf16_offsets, "byte_offsets": byte_offsets, "positions": positions})

OUT.write_text(json.dumps({"decode": decode, "helpers": helpers, "escape": escape, "slices": slices, "texts": texts}, indent=1) + "\n")
print(f"{OUT}: {len(decode)} decode, {len(helpers)} helper, {len(escape)} escape, {len(slices)} slice, {len(texts)} text fixtures")
