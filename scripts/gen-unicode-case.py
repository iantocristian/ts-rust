#!/usr/bin/env python3
"""Translate the pinned Go case tables into Rust.

`upstream/tsc/internal/stringutil/js_case_generated.go` is itself generated from
the @unicode/unicode-15.1.0 data by upstream's `generate-unicode-data.mts`. The
port keeps that pinned table rather than whatever Unicode version a Rust crate
ships (docs/design/text.md, section 2.5), so this script is a mechanical
translation of the pinned bytes, not a second derivation from Unicode data.

    python3 scripts/gen-unicode-case.py            # write the Rust table
    python3 scripts/gen-unicode-case.py --check    # fail if it would change
"""

import argparse
import re
import subprocess
import sys
from pathlib import Path

SOURCE = "upstream/tsc/internal/stringutil/js_case_generated.go"
TARGET = "crates/ts_jsstring/src/unicode_case_generated.rs"

MAPPING = re.compile(
    r"^\t(0x[0-9A-Fa-f]+):\s*\{(.*)\},$",
)
FIELD = re.compile(r'(\w+):\s*("(?:[^"\\]|\\.)*"|\w+)')
RANGE = re.compile(r"^\t\t\{(0x[0-9A-Fa-f]+|\d+), (0x[0-9A-Fa-f]+|\d+), (\d+)\},$")


def go_string(literal):
    """Decode a Go double-quoted string literal into its bytes."""
    out = bytearray()
    i = 1
    while i < len(literal) - 1:
        ch = literal[i]
        if ch != "\\":
            out.extend(ch.encode("utf-8"))
            i += 1
            continue
        kind = literal[i + 1]
        if kind == "u":
            out.extend(chr(int(literal[i + 2 : i + 6], 16)).encode("utf-8"))
            i += 6
        elif kind == "U":
            out.extend(chr(int(literal[i + 2 : i + 10], 16)).encode("utf-8"))
            i += 10
        elif kind == "x":
            out.append(int(literal[i + 2 : i + 4], 16))
            i += 4
        elif kind in "abfnrtv\\'\"":
            out.extend({"a": b"\a", "b": b"\b", "f": b"\f", "n": b"\n", "r": b"\r",
                        "t": b"\t", "v": b"\v", "\\": b"\\", "'": b"'", '"': b'"'}[kind])
            i += 2
        else:
            raise ValueError(f"unsupported Go escape in {literal!r}")
    return bytes(out)


def rust_bytes(data):
    return 'b"' + "".join(
        chr(b) if 0x20 <= b < 0x7F and chr(b) not in '"\\' else f"\\x{b:02x}" for b in data
    ) + '"'


def parse(text):
    mappings = []
    ranges = {}
    section = None
    table = None
    for line in text.splitlines():
        if line.startswith("var specialCasingMappings"):
            section = "mappings"
            continue
        if line.startswith("var unicodeCasedRanges"):
            section, table = "ranges", "cased"
            ranges[table] = {"r16": [], "r32": [], "latin_offset": 0}
            continue
        if line.startswith("var unicodeCaseIgnorableRanges"):
            section, table = "ranges", "ignorable"
            ranges[table] = {"r16": [], "r32": [], "latin_offset": 0}
            continue
        if line == "}":
            section = None
            continue
        if section == "mappings":
            match = MAPPING.match(line)
            if not match:
                continue
            fields = dict(FIELD.findall(match.group(2)))
            mappings.append(
                (
                    int(match.group(1), 16),
                    go_string(fields.get("lower", '""')),
                    go_string(fields.get("upper", '""')),
                    go_string(fields.get("conditionalLower", '""')),
                    fields.get("condition", "specialCasingConditionNone")
                    == "specialCasingConditionFinalSigma",
                )
            )
        elif section == "ranges":
            if "R16:" in line:
                ranges[table]["current"] = "r16"
            elif "R32:" in line:
                ranges[table]["current"] = "r32"
            elif "LatinOffset:" in line:
                ranges[table]["latin_offset"] = int(line.split(":")[1].strip().rstrip(","))
            else:
                match = RANGE.match(line)
                if match:
                    ranges[table][ranges[table]["current"]].append(
                        tuple(int(g, 0) for g in match.groups())
                    )
    return mappings, ranges


def render(mappings, ranges):
    out = [
        "//! Case tables translated from the pinned Go tables by",
        "//! `scripts/gen-unicode-case.py`. DO NOT EDIT.",
        "//!",
        "//! Source: `upstream/tsc/internal/stringutil/js_case_generated.go`, itself",
        "//! generated from @unicode/unicode-15.1.0. Keeping the pinned table is what",
        "//! ties casing to Unicode 15.1.0 instead of the Rust toolchain's tables",
        "//! (docs/design/text.md, section 2.5).",
        "",
        "use crate::rune::Rune;",
        "",
        "/// The `condition` field of upstream's `specialCasingMapping`.",
        "#[derive(Clone, Copy, PartialEq, Eq, Debug)]",
        "pub enum SpecialCasingCondition {",
        "    None,",
        "    FinalSigma,",
        "}",
        "",
        "/// Upstream's `specialCasingMapping`, with byte strings instead of Go strings.",
        "pub struct SpecialCasingMapping {",
        "    pub lower: &'static [u8],",
        "    pub upper: &'static [u8],",
        "    pub conditional_lower: &'static [u8],",
        "    pub condition: SpecialCasingCondition,",
        "}",
        "",
        "/// `unicode.Range16`.",
        "pub struct Range16 {",
        "    pub lo: u16,",
        "    pub hi: u16,",
        "    pub stride: u16,",
        "}",
        "",
        "/// `unicode.Range32`.",
        "pub struct Range32 {",
        "    pub lo: u32,",
        "    pub hi: u32,",
        "    pub stride: u32,",
        "}",
        "",
        "/// `unicode.RangeTable`.",
        "pub struct RangeTable {",
        "    pub r16: &'static [Range16],",
        "    pub r32: &'static [Range32],",
        "    pub latin_offset: usize,",
        "}",
        "",
        f"/// The {len(mappings)} entries of upstream's `specialCasingMappings`, sorted by code point.",
        "pub static SPECIAL_CASING_MAPPINGS: &[(Rune, SpecialCasingMapping)] = &[",
    ]
    for code, lower, upper, conditional, final_sigma in sorted(mappings):
        condition = "FinalSigma" if final_sigma else "None"
        out.append(
            f"    (0x{code:04X}, SpecialCasingMapping {{ lower: {rust_bytes(lower)}, "
            f"upper: {rust_bytes(upper)}, conditional_lower: {rust_bytes(conditional)}, "
            f"condition: SpecialCasingCondition::{condition} }}),"
        )
    out.append("];")
    for name, key in (("UNICODE_CASED_RANGES", "cased"), ("UNICODE_CASE_IGNORABLE_RANGES", "ignorable")):
        table = ranges[key]
        out += [
            "",
            f"/// `unicode{'Cased' if key == 'cased' else 'CaseIgnorable'}Ranges` from the pinned table.",
            f"pub static {name}: RangeTable = RangeTable {{",
            "    r16: &[",
        ]
        out += [f"        Range16 {{ lo: 0x{lo:04X}, hi: 0x{hi:04X}, stride: {stride} }}," for lo, hi, stride in table["r16"]]
        out += ["    ],", "    r32: &["]
        out += [f"        Range32 {{ lo: 0x{lo:04X}, hi: 0x{hi:04X}, stride: {stride} }}," for lo, hi, stride in table["r32"]]
        out += ["    ],", f"    latin_offset: {table['latin_offset']},", "};"]
    return "\n".join(out) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="fail instead of writing")
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    mappings, ranges = parse((root / SOURCE).read_text())
    if len(mappings) < 1000 or not ranges.get("cased") or not ranges.get("ignorable"):
        print("pinned case tables did not parse as expected", file=sys.stderr)
        return 1
    # ts_jsstring derives Unicode 15.1.0's simple lowercase mapping from this
    # table for LowerFirstChar, which is only possible where the full lowercase
    # mapping is a single rune. U+0130 is the documented exception; a pin that
    # adds another must be reviewed rather than silently mistranslated.
    multi = [code for code, lower, _, _, _ in mappings if len(lower.decode("utf-8")) != 1]
    if multi != [0x130]:
        print(f"unexpected multi-rune lowercase mappings: {[hex(c) for c in multi]}", file=sys.stderr)
        return 1
    # The checked-in file is formatted, so `cargo fmt --check` and this
    # generator agree on one byte sequence instead of fighting over it.
    rendered = subprocess.run(
        ["rustfmt", "--edition", "2021", "--emit", "stdout", "--quiet"],
        input=render(mappings, ranges),
        stdout=subprocess.PIPE,
        text=True,
        check=True,
        cwd=root,
    ).stdout
    target = root / TARGET
    if args.check:
        if not target.is_file() or target.read_text() != rendered:
            print(f"{TARGET} is out of date; run scripts/gen-unicode-case.py", file=sys.stderr)
            return 1
        print(f"{TARGET} matches the pinned tables")
        return 0
    target.write_text(rendered)
    print(f"wrote {TARGET}: {len(mappings)} mappings")
    return 0


if __name__ == "__main__":
    sys.exit(main())
