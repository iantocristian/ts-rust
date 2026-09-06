#!/usr/bin/env python3
"""Translate Corsa's pinned Unicode 15.1 JS casing tables to Rust."""

from __future__ import annotations

import argparse
import ast
import re
from pathlib import Path


MAP = re.compile(
    r'^\s*(0x[0-9A-F]+):\s+\{lower: "(.*?)", upper: "(.*?)"'
    r'(?:, conditionalLower: "(.*?)")?, condition: ([A-Za-z0-9_]+)\},$'
)
RANGE = re.compile(r"^\s*\{(0x[0-9A-F]+), (0x[0-9A-F]+), ([0-9]+)\},$")


def go_string(value: str) -> str:
    return ast.literal_eval(f'"{value}"')


def rust_string(value: str) -> str:
    pieces = ['"']
    for char in value:
        code = ord(char)
        if char == '"':
            pieces.append(r'\"')
        elif char == "\\":
            pieces.append(r"\\")
        elif code < 0x20 or code >= 0x7F:
            pieces.append(f"\\u{{{code:X}}}")
        else:
            pieces.append(char)
    pieces.append('"')
    return "".join(pieces)


def ranges(text: str, name: str) -> list[tuple[str, str, str]]:
    start = text.index(f"var {name} = &unicode.RangeTable{{")
    end = text.index("\n}\n", start)
    found = []
    for line in text[start:end].splitlines():
        if match := RANGE.match(line):
            found.append(match.groups())
    return found


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("source", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    text = args.source.read_text()

    mappings = []
    for line in text.splitlines():
        if match := MAP.match(line):
            code, lower, upper, conditional, condition = match.groups()
            mappings.append(
                (
                    code,
                    rust_string(go_string(lower)),
                    rust_string(go_string(upper)),
                    "None" if conditional is None else f"Some({rust_string(go_string(conditional))})",
                    condition == "specialCasingConditionFinalSigma",
                )
            )

    cased = ranges(text, "unicodeCasedRanges")
    ignorable = ranges(text, "unicodeCaseIgnorableRanges")
    if not mappings or not cased or not ignorable:
        raise SystemExit("failed to parse one or more casing tables")

    lines = [
        "// @generated from the pinned Corsa Unicode 15.1 tables; do not edit.",
        "// Source: tsc/internal/stringutil/js_case_generated.go",
        "",
        "#[derive(Clone, Copy)]",
        "pub(crate) struct Mapping {",
        "    pub(crate) code_point: u32,",
        "    pub(crate) lower: &'static str,",
        "    pub(crate) upper: &'static str,",
        "    pub(crate) conditional_lower: Option<&'static str>,",
        "    pub(crate) final_sigma: bool,",
        "}",
        "",
        "const MAPPINGS: &[Mapping] = &[",
    ]
    for code, lower, upper, conditional, final in mappings:
        lines.extend(
            [
                "    Mapping {",
                f"        code_point: {code},",
                f"        lower: {lower},",
                f"        upper: {upper},",
                f"        conditional_lower: {conditional},",
                f"        final_sigma: {str(final).lower()},",
                "    },",
            ]
        )
    lines.extend(
        [
            "];",
            "",
            "const CASED_RANGES: &[(u32, u32, u32)] = &[",
        ]
    )
    lines.extend(f"    ({lo}, {hi}, {stride})," for lo, hi, stride in cased)
    lines.extend(
        [
            "];",
            "",
            "const CASE_IGNORABLE_RANGES: &[(u32, u32, u32)] = &[",
        ]
    )
    lines.extend(f"    ({lo}, {hi}, {stride})," for lo, hi, stride in ignorable)
    lines.extend(
        [
            "];",
            "",
            "pub(crate) fn mapping(code_point: u32) -> Option<&'static Mapping> {",
            "    MAPPINGS",
            "        .binary_search_by_key(&code_point, |mapping| mapping.code_point)",
            "        .ok()",
            "        .map(|index| &MAPPINGS[index])",
            "}",
            "",
            "pub(crate) fn is_cased(code_point: u32) -> bool {",
            "    in_ranges(code_point, CASED_RANGES)",
            "}",
            "",
            "pub(crate) fn is_case_ignorable(code_point: u32) -> bool {",
            "    in_ranges(code_point, CASE_IGNORABLE_RANGES)",
            "}",
            "",
            "fn in_ranges(code_point: u32, ranges: &[(u32, u32, u32)]) -> bool {",
            "    let index = ranges.partition_point(|&(low, _, _)| low <= code_point);",
            "    index.checked_sub(1).is_some_and(|index| {",
            "        let (low, high, stride) = ranges[index];",
            "        code_point <= high && (code_point - low).is_multiple_of(stride)",
            "    })",
            "}",
            "",
        ]
    )
    args.output.write_text("\n".join(lines))


if __name__ == "__main__":
    main()
