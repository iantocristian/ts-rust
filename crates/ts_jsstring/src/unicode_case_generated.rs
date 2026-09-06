//! Case tables translated from the pinned Go tables by
//! `scripts/gen-unicode-case.py`. DO NOT EDIT.
//!
//! Source: `upstream/tsc/internal/stringutil/js_case_generated.go`, itself
//! generated from @unicode/unicode-15.1.0. Keeping the pinned table is what
//! ties casing to Unicode 15.1.0 instead of the Rust toolchain's tables
//! (docs/design/text.md, section 2.5).

use crate::rune::Rune;

/// The `condition` field of upstream's `specialCasingMapping`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpecialCasingCondition {
    None,
    FinalSigma,
}

/// Upstream's `specialCasingMapping`, with byte strings instead of Go strings.
pub struct SpecialCasingMapping {
    pub lower: &'static [u8],
    pub upper: &'static [u8],
    pub conditional_lower: &'static [u8],
    pub condition: SpecialCasingCondition,
}

/// `unicode.Range16`.
pub struct Range16 {
    pub lo: u16,
    pub hi: u16,
    pub stride: u16,
}

/// `unicode.Range32`.
pub struct Range32 {
    pub lo: u32,
    pub hi: u32,
    pub stride: u32,
}

/// `unicode.RangeTable`.
pub struct RangeTable {
    pub r16: &'static [Range16],
    pub r32: &'static [Range32],
    pub latin_offset: usize,
}

/// The 2927 entries of upstream's `specialCasingMappings`, sorted by code point.
pub static SPECIAL_CASING_MAPPINGS: &[(Rune, SpecialCasingMapping)] = &[
    (
        0x0041,
        SpecialCasingMapping {
            lower: b"a",
            upper: b"A",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0042,
        SpecialCasingMapping {
            lower: b"b",
            upper: b"B",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0043,
        SpecialCasingMapping {
            lower: b"c",
            upper: b"C",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0044,
        SpecialCasingMapping {
            lower: b"d",
            upper: b"D",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0045,
        SpecialCasingMapping {
            lower: b"e",
            upper: b"E",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0046,
        SpecialCasingMapping {
            lower: b"f",
            upper: b"F",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0047,
        SpecialCasingMapping {
            lower: b"g",
            upper: b"G",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0048,
        SpecialCasingMapping {
            lower: b"h",
            upper: b"H",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0049,
        SpecialCasingMapping {
            lower: b"i",
            upper: b"I",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x004A,
        SpecialCasingMapping {
            lower: b"j",
            upper: b"J",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x004B,
        SpecialCasingMapping {
            lower: b"k",
            upper: b"K",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x004C,
        SpecialCasingMapping {
            lower: b"l",
            upper: b"L",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x004D,
        SpecialCasingMapping {
            lower: b"m",
            upper: b"M",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x004E,
        SpecialCasingMapping {
            lower: b"n",
            upper: b"N",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x004F,
        SpecialCasingMapping {
            lower: b"o",
            upper: b"O",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0050,
        SpecialCasingMapping {
            lower: b"p",
            upper: b"P",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0051,
        SpecialCasingMapping {
            lower: b"q",
            upper: b"Q",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0052,
        SpecialCasingMapping {
            lower: b"r",
            upper: b"R",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0053,
        SpecialCasingMapping {
            lower: b"s",
            upper: b"S",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0054,
        SpecialCasingMapping {
            lower: b"t",
            upper: b"T",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0055,
        SpecialCasingMapping {
            lower: b"u",
            upper: b"U",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0056,
        SpecialCasingMapping {
            lower: b"v",
            upper: b"V",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0057,
        SpecialCasingMapping {
            lower: b"w",
            upper: b"W",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0058,
        SpecialCasingMapping {
            lower: b"x",
            upper: b"X",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0059,
        SpecialCasingMapping {
            lower: b"y",
            upper: b"Y",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x005A,
        SpecialCasingMapping {
            lower: b"z",
            upper: b"Z",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0061,
        SpecialCasingMapping {
            lower: b"a",
            upper: b"A",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0062,
        SpecialCasingMapping {
            lower: b"b",
            upper: b"B",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0063,
        SpecialCasingMapping {
            lower: b"c",
            upper: b"C",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0064,
        SpecialCasingMapping {
            lower: b"d",
            upper: b"D",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0065,
        SpecialCasingMapping {
            lower: b"e",
            upper: b"E",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0066,
        SpecialCasingMapping {
            lower: b"f",
            upper: b"F",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0067,
        SpecialCasingMapping {
            lower: b"g",
            upper: b"G",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0068,
        SpecialCasingMapping {
            lower: b"h",
            upper: b"H",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0069,
        SpecialCasingMapping {
            lower: b"i",
            upper: b"I",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x006A,
        SpecialCasingMapping {
            lower: b"j",
            upper: b"J",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x006B,
        SpecialCasingMapping {
            lower: b"k",
            upper: b"K",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x006C,
        SpecialCasingMapping {
            lower: b"l",
            upper: b"L",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x006D,
        SpecialCasingMapping {
            lower: b"m",
            upper: b"M",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x006E,
        SpecialCasingMapping {
            lower: b"n",
            upper: b"N",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x006F,
        SpecialCasingMapping {
            lower: b"o",
            upper: b"O",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0070,
        SpecialCasingMapping {
            lower: b"p",
            upper: b"P",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0071,
        SpecialCasingMapping {
            lower: b"q",
            upper: b"Q",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0072,
        SpecialCasingMapping {
            lower: b"r",
            upper: b"R",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0073,
        SpecialCasingMapping {
            lower: b"s",
            upper: b"S",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0074,
        SpecialCasingMapping {
            lower: b"t",
            upper: b"T",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0075,
        SpecialCasingMapping {
            lower: b"u",
            upper: b"U",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0076,
        SpecialCasingMapping {
            lower: b"v",
            upper: b"V",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0077,
        SpecialCasingMapping {
            lower: b"w",
            upper: b"W",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0078,
        SpecialCasingMapping {
            lower: b"x",
            upper: b"X",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0079,
        SpecialCasingMapping {
            lower: b"y",
            upper: b"Y",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x007A,
        SpecialCasingMapping {
            lower: b"z",
            upper: b"Z",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00B5,
        SpecialCasingMapping {
            lower: b"\xc2\xb5",
            upper: b"\xce\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00C0,
        SpecialCasingMapping {
            lower: b"\xc3\xa0",
            upper: b"\xc3\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00C1,
        SpecialCasingMapping {
            lower: b"\xc3\xa1",
            upper: b"\xc3\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00C2,
        SpecialCasingMapping {
            lower: b"\xc3\xa2",
            upper: b"\xc3\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00C3,
        SpecialCasingMapping {
            lower: b"\xc3\xa3",
            upper: b"\xc3\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00C4,
        SpecialCasingMapping {
            lower: b"\xc3\xa4",
            upper: b"\xc3\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00C5,
        SpecialCasingMapping {
            lower: b"\xc3\xa5",
            upper: b"\xc3\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00C6,
        SpecialCasingMapping {
            lower: b"\xc3\xa6",
            upper: b"\xc3\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00C7,
        SpecialCasingMapping {
            lower: b"\xc3\xa7",
            upper: b"\xc3\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00C8,
        SpecialCasingMapping {
            lower: b"\xc3\xa8",
            upper: b"\xc3\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00C9,
        SpecialCasingMapping {
            lower: b"\xc3\xa9",
            upper: b"\xc3\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00CA,
        SpecialCasingMapping {
            lower: b"\xc3\xaa",
            upper: b"\xc3\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00CB,
        SpecialCasingMapping {
            lower: b"\xc3\xab",
            upper: b"\xc3\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00CC,
        SpecialCasingMapping {
            lower: b"\xc3\xac",
            upper: b"\xc3\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00CD,
        SpecialCasingMapping {
            lower: b"\xc3\xad",
            upper: b"\xc3\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00CE,
        SpecialCasingMapping {
            lower: b"\xc3\xae",
            upper: b"\xc3\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00CF,
        SpecialCasingMapping {
            lower: b"\xc3\xaf",
            upper: b"\xc3\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00D0,
        SpecialCasingMapping {
            lower: b"\xc3\xb0",
            upper: b"\xc3\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00D1,
        SpecialCasingMapping {
            lower: b"\xc3\xb1",
            upper: b"\xc3\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00D2,
        SpecialCasingMapping {
            lower: b"\xc3\xb2",
            upper: b"\xc3\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00D3,
        SpecialCasingMapping {
            lower: b"\xc3\xb3",
            upper: b"\xc3\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00D4,
        SpecialCasingMapping {
            lower: b"\xc3\xb4",
            upper: b"\xc3\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00D5,
        SpecialCasingMapping {
            lower: b"\xc3\xb5",
            upper: b"\xc3\x95",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00D6,
        SpecialCasingMapping {
            lower: b"\xc3\xb6",
            upper: b"\xc3\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00D8,
        SpecialCasingMapping {
            lower: b"\xc3\xb8",
            upper: b"\xc3\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00D9,
        SpecialCasingMapping {
            lower: b"\xc3\xb9",
            upper: b"\xc3\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00DA,
        SpecialCasingMapping {
            lower: b"\xc3\xba",
            upper: b"\xc3\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00DB,
        SpecialCasingMapping {
            lower: b"\xc3\xbb",
            upper: b"\xc3\x9b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00DC,
        SpecialCasingMapping {
            lower: b"\xc3\xbc",
            upper: b"\xc3\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00DD,
        SpecialCasingMapping {
            lower: b"\xc3\xbd",
            upper: b"\xc3\x9d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00DE,
        SpecialCasingMapping {
            lower: b"\xc3\xbe",
            upper: b"\xc3\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00DF,
        SpecialCasingMapping {
            lower: b"\xc3\x9f",
            upper: b"SS",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00E0,
        SpecialCasingMapping {
            lower: b"\xc3\xa0",
            upper: b"\xc3\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00E1,
        SpecialCasingMapping {
            lower: b"\xc3\xa1",
            upper: b"\xc3\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00E2,
        SpecialCasingMapping {
            lower: b"\xc3\xa2",
            upper: b"\xc3\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00E3,
        SpecialCasingMapping {
            lower: b"\xc3\xa3",
            upper: b"\xc3\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00E4,
        SpecialCasingMapping {
            lower: b"\xc3\xa4",
            upper: b"\xc3\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00E5,
        SpecialCasingMapping {
            lower: b"\xc3\xa5",
            upper: b"\xc3\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00E6,
        SpecialCasingMapping {
            lower: b"\xc3\xa6",
            upper: b"\xc3\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00E7,
        SpecialCasingMapping {
            lower: b"\xc3\xa7",
            upper: b"\xc3\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00E8,
        SpecialCasingMapping {
            lower: b"\xc3\xa8",
            upper: b"\xc3\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00E9,
        SpecialCasingMapping {
            lower: b"\xc3\xa9",
            upper: b"\xc3\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00EA,
        SpecialCasingMapping {
            lower: b"\xc3\xaa",
            upper: b"\xc3\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00EB,
        SpecialCasingMapping {
            lower: b"\xc3\xab",
            upper: b"\xc3\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00EC,
        SpecialCasingMapping {
            lower: b"\xc3\xac",
            upper: b"\xc3\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00ED,
        SpecialCasingMapping {
            lower: b"\xc3\xad",
            upper: b"\xc3\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00EE,
        SpecialCasingMapping {
            lower: b"\xc3\xae",
            upper: b"\xc3\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00EF,
        SpecialCasingMapping {
            lower: b"\xc3\xaf",
            upper: b"\xc3\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00F0,
        SpecialCasingMapping {
            lower: b"\xc3\xb0",
            upper: b"\xc3\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00F1,
        SpecialCasingMapping {
            lower: b"\xc3\xb1",
            upper: b"\xc3\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00F2,
        SpecialCasingMapping {
            lower: b"\xc3\xb2",
            upper: b"\xc3\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00F3,
        SpecialCasingMapping {
            lower: b"\xc3\xb3",
            upper: b"\xc3\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00F4,
        SpecialCasingMapping {
            lower: b"\xc3\xb4",
            upper: b"\xc3\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00F5,
        SpecialCasingMapping {
            lower: b"\xc3\xb5",
            upper: b"\xc3\x95",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00F6,
        SpecialCasingMapping {
            lower: b"\xc3\xb6",
            upper: b"\xc3\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00F8,
        SpecialCasingMapping {
            lower: b"\xc3\xb8",
            upper: b"\xc3\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00F9,
        SpecialCasingMapping {
            lower: b"\xc3\xb9",
            upper: b"\xc3\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00FA,
        SpecialCasingMapping {
            lower: b"\xc3\xba",
            upper: b"\xc3\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00FB,
        SpecialCasingMapping {
            lower: b"\xc3\xbb",
            upper: b"\xc3\x9b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00FC,
        SpecialCasingMapping {
            lower: b"\xc3\xbc",
            upper: b"\xc3\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00FD,
        SpecialCasingMapping {
            lower: b"\xc3\xbd",
            upper: b"\xc3\x9d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00FE,
        SpecialCasingMapping {
            lower: b"\xc3\xbe",
            upper: b"\xc3\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x00FF,
        SpecialCasingMapping {
            lower: b"\xc3\xbf",
            upper: b"\xc5\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0100,
        SpecialCasingMapping {
            lower: b"\xc4\x81",
            upper: b"\xc4\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0101,
        SpecialCasingMapping {
            lower: b"\xc4\x81",
            upper: b"\xc4\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0102,
        SpecialCasingMapping {
            lower: b"\xc4\x83",
            upper: b"\xc4\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0103,
        SpecialCasingMapping {
            lower: b"\xc4\x83",
            upper: b"\xc4\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0104,
        SpecialCasingMapping {
            lower: b"\xc4\x85",
            upper: b"\xc4\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0105,
        SpecialCasingMapping {
            lower: b"\xc4\x85",
            upper: b"\xc4\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0106,
        SpecialCasingMapping {
            lower: b"\xc4\x87",
            upper: b"\xc4\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0107,
        SpecialCasingMapping {
            lower: b"\xc4\x87",
            upper: b"\xc4\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0108,
        SpecialCasingMapping {
            lower: b"\xc4\x89",
            upper: b"\xc4\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0109,
        SpecialCasingMapping {
            lower: b"\xc4\x89",
            upper: b"\xc4\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x010A,
        SpecialCasingMapping {
            lower: b"\xc4\x8b",
            upper: b"\xc4\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x010B,
        SpecialCasingMapping {
            lower: b"\xc4\x8b",
            upper: b"\xc4\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x010C,
        SpecialCasingMapping {
            lower: b"\xc4\x8d",
            upper: b"\xc4\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x010D,
        SpecialCasingMapping {
            lower: b"\xc4\x8d",
            upper: b"\xc4\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x010E,
        SpecialCasingMapping {
            lower: b"\xc4\x8f",
            upper: b"\xc4\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x010F,
        SpecialCasingMapping {
            lower: b"\xc4\x8f",
            upper: b"\xc4\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0110,
        SpecialCasingMapping {
            lower: b"\xc4\x91",
            upper: b"\xc4\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0111,
        SpecialCasingMapping {
            lower: b"\xc4\x91",
            upper: b"\xc4\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0112,
        SpecialCasingMapping {
            lower: b"\xc4\x93",
            upper: b"\xc4\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0113,
        SpecialCasingMapping {
            lower: b"\xc4\x93",
            upper: b"\xc4\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0114,
        SpecialCasingMapping {
            lower: b"\xc4\x95",
            upper: b"\xc4\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0115,
        SpecialCasingMapping {
            lower: b"\xc4\x95",
            upper: b"\xc4\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0116,
        SpecialCasingMapping {
            lower: b"\xc4\x97",
            upper: b"\xc4\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0117,
        SpecialCasingMapping {
            lower: b"\xc4\x97",
            upper: b"\xc4\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0118,
        SpecialCasingMapping {
            lower: b"\xc4\x99",
            upper: b"\xc4\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0119,
        SpecialCasingMapping {
            lower: b"\xc4\x99",
            upper: b"\xc4\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x011A,
        SpecialCasingMapping {
            lower: b"\xc4\x9b",
            upper: b"\xc4\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x011B,
        SpecialCasingMapping {
            lower: b"\xc4\x9b",
            upper: b"\xc4\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x011C,
        SpecialCasingMapping {
            lower: b"\xc4\x9d",
            upper: b"\xc4\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x011D,
        SpecialCasingMapping {
            lower: b"\xc4\x9d",
            upper: b"\xc4\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x011E,
        SpecialCasingMapping {
            lower: b"\xc4\x9f",
            upper: b"\xc4\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x011F,
        SpecialCasingMapping {
            lower: b"\xc4\x9f",
            upper: b"\xc4\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0120,
        SpecialCasingMapping {
            lower: b"\xc4\xa1",
            upper: b"\xc4\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0121,
        SpecialCasingMapping {
            lower: b"\xc4\xa1",
            upper: b"\xc4\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0122,
        SpecialCasingMapping {
            lower: b"\xc4\xa3",
            upper: b"\xc4\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0123,
        SpecialCasingMapping {
            lower: b"\xc4\xa3",
            upper: b"\xc4\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0124,
        SpecialCasingMapping {
            lower: b"\xc4\xa5",
            upper: b"\xc4\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0125,
        SpecialCasingMapping {
            lower: b"\xc4\xa5",
            upper: b"\xc4\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0126,
        SpecialCasingMapping {
            lower: b"\xc4\xa7",
            upper: b"\xc4\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0127,
        SpecialCasingMapping {
            lower: b"\xc4\xa7",
            upper: b"\xc4\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0128,
        SpecialCasingMapping {
            lower: b"\xc4\xa9",
            upper: b"\xc4\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0129,
        SpecialCasingMapping {
            lower: b"\xc4\xa9",
            upper: b"\xc4\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x012A,
        SpecialCasingMapping {
            lower: b"\xc4\xab",
            upper: b"\xc4\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x012B,
        SpecialCasingMapping {
            lower: b"\xc4\xab",
            upper: b"\xc4\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x012C,
        SpecialCasingMapping {
            lower: b"\xc4\xad",
            upper: b"\xc4\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x012D,
        SpecialCasingMapping {
            lower: b"\xc4\xad",
            upper: b"\xc4\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x012E,
        SpecialCasingMapping {
            lower: b"\xc4\xaf",
            upper: b"\xc4\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x012F,
        SpecialCasingMapping {
            lower: b"\xc4\xaf",
            upper: b"\xc4\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0130,
        SpecialCasingMapping {
            lower: b"i\xcc\x87",
            upper: b"\xc4\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0131,
        SpecialCasingMapping {
            lower: b"\xc4\xb1",
            upper: b"I",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0132,
        SpecialCasingMapping {
            lower: b"\xc4\xb3",
            upper: b"\xc4\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0133,
        SpecialCasingMapping {
            lower: b"\xc4\xb3",
            upper: b"\xc4\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0134,
        SpecialCasingMapping {
            lower: b"\xc4\xb5",
            upper: b"\xc4\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0135,
        SpecialCasingMapping {
            lower: b"\xc4\xb5",
            upper: b"\xc4\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0136,
        SpecialCasingMapping {
            lower: b"\xc4\xb7",
            upper: b"\xc4\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0137,
        SpecialCasingMapping {
            lower: b"\xc4\xb7",
            upper: b"\xc4\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0139,
        SpecialCasingMapping {
            lower: b"\xc4\xba",
            upper: b"\xc4\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x013A,
        SpecialCasingMapping {
            lower: b"\xc4\xba",
            upper: b"\xc4\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x013B,
        SpecialCasingMapping {
            lower: b"\xc4\xbc",
            upper: b"\xc4\xbb",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x013C,
        SpecialCasingMapping {
            lower: b"\xc4\xbc",
            upper: b"\xc4\xbb",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x013D,
        SpecialCasingMapping {
            lower: b"\xc4\xbe",
            upper: b"\xc4\xbd",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x013E,
        SpecialCasingMapping {
            lower: b"\xc4\xbe",
            upper: b"\xc4\xbd",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x013F,
        SpecialCasingMapping {
            lower: b"\xc5\x80",
            upper: b"\xc4\xbf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0140,
        SpecialCasingMapping {
            lower: b"\xc5\x80",
            upper: b"\xc4\xbf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0141,
        SpecialCasingMapping {
            lower: b"\xc5\x82",
            upper: b"\xc5\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0142,
        SpecialCasingMapping {
            lower: b"\xc5\x82",
            upper: b"\xc5\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0143,
        SpecialCasingMapping {
            lower: b"\xc5\x84",
            upper: b"\xc5\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0144,
        SpecialCasingMapping {
            lower: b"\xc5\x84",
            upper: b"\xc5\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0145,
        SpecialCasingMapping {
            lower: b"\xc5\x86",
            upper: b"\xc5\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0146,
        SpecialCasingMapping {
            lower: b"\xc5\x86",
            upper: b"\xc5\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0147,
        SpecialCasingMapping {
            lower: b"\xc5\x88",
            upper: b"\xc5\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0148,
        SpecialCasingMapping {
            lower: b"\xc5\x88",
            upper: b"\xc5\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0149,
        SpecialCasingMapping {
            lower: b"\xc5\x89",
            upper: b"\xca\xbcN",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x014A,
        SpecialCasingMapping {
            lower: b"\xc5\x8b",
            upper: b"\xc5\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x014B,
        SpecialCasingMapping {
            lower: b"\xc5\x8b",
            upper: b"\xc5\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x014C,
        SpecialCasingMapping {
            lower: b"\xc5\x8d",
            upper: b"\xc5\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x014D,
        SpecialCasingMapping {
            lower: b"\xc5\x8d",
            upper: b"\xc5\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x014E,
        SpecialCasingMapping {
            lower: b"\xc5\x8f",
            upper: b"\xc5\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x014F,
        SpecialCasingMapping {
            lower: b"\xc5\x8f",
            upper: b"\xc5\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0150,
        SpecialCasingMapping {
            lower: b"\xc5\x91",
            upper: b"\xc5\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0151,
        SpecialCasingMapping {
            lower: b"\xc5\x91",
            upper: b"\xc5\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0152,
        SpecialCasingMapping {
            lower: b"\xc5\x93",
            upper: b"\xc5\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0153,
        SpecialCasingMapping {
            lower: b"\xc5\x93",
            upper: b"\xc5\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0154,
        SpecialCasingMapping {
            lower: b"\xc5\x95",
            upper: b"\xc5\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0155,
        SpecialCasingMapping {
            lower: b"\xc5\x95",
            upper: b"\xc5\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0156,
        SpecialCasingMapping {
            lower: b"\xc5\x97",
            upper: b"\xc5\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0157,
        SpecialCasingMapping {
            lower: b"\xc5\x97",
            upper: b"\xc5\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0158,
        SpecialCasingMapping {
            lower: b"\xc5\x99",
            upper: b"\xc5\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0159,
        SpecialCasingMapping {
            lower: b"\xc5\x99",
            upper: b"\xc5\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x015A,
        SpecialCasingMapping {
            lower: b"\xc5\x9b",
            upper: b"\xc5\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x015B,
        SpecialCasingMapping {
            lower: b"\xc5\x9b",
            upper: b"\xc5\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x015C,
        SpecialCasingMapping {
            lower: b"\xc5\x9d",
            upper: b"\xc5\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x015D,
        SpecialCasingMapping {
            lower: b"\xc5\x9d",
            upper: b"\xc5\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x015E,
        SpecialCasingMapping {
            lower: b"\xc5\x9f",
            upper: b"\xc5\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x015F,
        SpecialCasingMapping {
            lower: b"\xc5\x9f",
            upper: b"\xc5\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0160,
        SpecialCasingMapping {
            lower: b"\xc5\xa1",
            upper: b"\xc5\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0161,
        SpecialCasingMapping {
            lower: b"\xc5\xa1",
            upper: b"\xc5\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0162,
        SpecialCasingMapping {
            lower: b"\xc5\xa3",
            upper: b"\xc5\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0163,
        SpecialCasingMapping {
            lower: b"\xc5\xa3",
            upper: b"\xc5\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0164,
        SpecialCasingMapping {
            lower: b"\xc5\xa5",
            upper: b"\xc5\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0165,
        SpecialCasingMapping {
            lower: b"\xc5\xa5",
            upper: b"\xc5\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0166,
        SpecialCasingMapping {
            lower: b"\xc5\xa7",
            upper: b"\xc5\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0167,
        SpecialCasingMapping {
            lower: b"\xc5\xa7",
            upper: b"\xc5\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0168,
        SpecialCasingMapping {
            lower: b"\xc5\xa9",
            upper: b"\xc5\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0169,
        SpecialCasingMapping {
            lower: b"\xc5\xa9",
            upper: b"\xc5\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x016A,
        SpecialCasingMapping {
            lower: b"\xc5\xab",
            upper: b"\xc5\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x016B,
        SpecialCasingMapping {
            lower: b"\xc5\xab",
            upper: b"\xc5\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x016C,
        SpecialCasingMapping {
            lower: b"\xc5\xad",
            upper: b"\xc5\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x016D,
        SpecialCasingMapping {
            lower: b"\xc5\xad",
            upper: b"\xc5\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x016E,
        SpecialCasingMapping {
            lower: b"\xc5\xaf",
            upper: b"\xc5\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x016F,
        SpecialCasingMapping {
            lower: b"\xc5\xaf",
            upper: b"\xc5\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0170,
        SpecialCasingMapping {
            lower: b"\xc5\xb1",
            upper: b"\xc5\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0171,
        SpecialCasingMapping {
            lower: b"\xc5\xb1",
            upper: b"\xc5\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0172,
        SpecialCasingMapping {
            lower: b"\xc5\xb3",
            upper: b"\xc5\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0173,
        SpecialCasingMapping {
            lower: b"\xc5\xb3",
            upper: b"\xc5\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0174,
        SpecialCasingMapping {
            lower: b"\xc5\xb5",
            upper: b"\xc5\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0175,
        SpecialCasingMapping {
            lower: b"\xc5\xb5",
            upper: b"\xc5\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0176,
        SpecialCasingMapping {
            lower: b"\xc5\xb7",
            upper: b"\xc5\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0177,
        SpecialCasingMapping {
            lower: b"\xc5\xb7",
            upper: b"\xc5\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0178,
        SpecialCasingMapping {
            lower: b"\xc3\xbf",
            upper: b"\xc5\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0179,
        SpecialCasingMapping {
            lower: b"\xc5\xba",
            upper: b"\xc5\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x017A,
        SpecialCasingMapping {
            lower: b"\xc5\xba",
            upper: b"\xc5\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x017B,
        SpecialCasingMapping {
            lower: b"\xc5\xbc",
            upper: b"\xc5\xbb",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x017C,
        SpecialCasingMapping {
            lower: b"\xc5\xbc",
            upper: b"\xc5\xbb",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x017D,
        SpecialCasingMapping {
            lower: b"\xc5\xbe",
            upper: b"\xc5\xbd",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x017E,
        SpecialCasingMapping {
            lower: b"\xc5\xbe",
            upper: b"\xc5\xbd",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x017F,
        SpecialCasingMapping {
            lower: b"\xc5\xbf",
            upper: b"S",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0180,
        SpecialCasingMapping {
            lower: b"\xc6\x80",
            upper: b"\xc9\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0181,
        SpecialCasingMapping {
            lower: b"\xc9\x93",
            upper: b"\xc6\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0182,
        SpecialCasingMapping {
            lower: b"\xc6\x83",
            upper: b"\xc6\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0183,
        SpecialCasingMapping {
            lower: b"\xc6\x83",
            upper: b"\xc6\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0184,
        SpecialCasingMapping {
            lower: b"\xc6\x85",
            upper: b"\xc6\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0185,
        SpecialCasingMapping {
            lower: b"\xc6\x85",
            upper: b"\xc6\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0186,
        SpecialCasingMapping {
            lower: b"\xc9\x94",
            upper: b"\xc6\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0187,
        SpecialCasingMapping {
            lower: b"\xc6\x88",
            upper: b"\xc6\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0188,
        SpecialCasingMapping {
            lower: b"\xc6\x88",
            upper: b"\xc6\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0189,
        SpecialCasingMapping {
            lower: b"\xc9\x96",
            upper: b"\xc6\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x018A,
        SpecialCasingMapping {
            lower: b"\xc9\x97",
            upper: b"\xc6\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x018B,
        SpecialCasingMapping {
            lower: b"\xc6\x8c",
            upper: b"\xc6\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x018C,
        SpecialCasingMapping {
            lower: b"\xc6\x8c",
            upper: b"\xc6\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x018E,
        SpecialCasingMapping {
            lower: b"\xc7\x9d",
            upper: b"\xc6\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x018F,
        SpecialCasingMapping {
            lower: b"\xc9\x99",
            upper: b"\xc6\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0190,
        SpecialCasingMapping {
            lower: b"\xc9\x9b",
            upper: b"\xc6\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0191,
        SpecialCasingMapping {
            lower: b"\xc6\x92",
            upper: b"\xc6\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0192,
        SpecialCasingMapping {
            lower: b"\xc6\x92",
            upper: b"\xc6\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0193,
        SpecialCasingMapping {
            lower: b"\xc9\xa0",
            upper: b"\xc6\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0194,
        SpecialCasingMapping {
            lower: b"\xc9\xa3",
            upper: b"\xc6\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0195,
        SpecialCasingMapping {
            lower: b"\xc6\x95",
            upper: b"\xc7\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0196,
        SpecialCasingMapping {
            lower: b"\xc9\xa9",
            upper: b"\xc6\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0197,
        SpecialCasingMapping {
            lower: b"\xc9\xa8",
            upper: b"\xc6\x97",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0198,
        SpecialCasingMapping {
            lower: b"\xc6\x99",
            upper: b"\xc6\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0199,
        SpecialCasingMapping {
            lower: b"\xc6\x99",
            upper: b"\xc6\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x019A,
        SpecialCasingMapping {
            lower: b"\xc6\x9a",
            upper: b"\xc8\xbd",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x019C,
        SpecialCasingMapping {
            lower: b"\xc9\xaf",
            upper: b"\xc6\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x019D,
        SpecialCasingMapping {
            lower: b"\xc9\xb2",
            upper: b"\xc6\x9d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x019E,
        SpecialCasingMapping {
            lower: b"\xc6\x9e",
            upper: b"\xc8\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x019F,
        SpecialCasingMapping {
            lower: b"\xc9\xb5",
            upper: b"\xc6\x9f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01A0,
        SpecialCasingMapping {
            lower: b"\xc6\xa1",
            upper: b"\xc6\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01A1,
        SpecialCasingMapping {
            lower: b"\xc6\xa1",
            upper: b"\xc6\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01A2,
        SpecialCasingMapping {
            lower: b"\xc6\xa3",
            upper: b"\xc6\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01A3,
        SpecialCasingMapping {
            lower: b"\xc6\xa3",
            upper: b"\xc6\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01A4,
        SpecialCasingMapping {
            lower: b"\xc6\xa5",
            upper: b"\xc6\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01A5,
        SpecialCasingMapping {
            lower: b"\xc6\xa5",
            upper: b"\xc6\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01A6,
        SpecialCasingMapping {
            lower: b"\xca\x80",
            upper: b"\xc6\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01A7,
        SpecialCasingMapping {
            lower: b"\xc6\xa8",
            upper: b"\xc6\xa7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01A8,
        SpecialCasingMapping {
            lower: b"\xc6\xa8",
            upper: b"\xc6\xa7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01A9,
        SpecialCasingMapping {
            lower: b"\xca\x83",
            upper: b"\xc6\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01AC,
        SpecialCasingMapping {
            lower: b"\xc6\xad",
            upper: b"\xc6\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01AD,
        SpecialCasingMapping {
            lower: b"\xc6\xad",
            upper: b"\xc6\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01AE,
        SpecialCasingMapping {
            lower: b"\xca\x88",
            upper: b"\xc6\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01AF,
        SpecialCasingMapping {
            lower: b"\xc6\xb0",
            upper: b"\xc6\xaf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01B0,
        SpecialCasingMapping {
            lower: b"\xc6\xb0",
            upper: b"\xc6\xaf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01B1,
        SpecialCasingMapping {
            lower: b"\xca\x8a",
            upper: b"\xc6\xb1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01B2,
        SpecialCasingMapping {
            lower: b"\xca\x8b",
            upper: b"\xc6\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01B3,
        SpecialCasingMapping {
            lower: b"\xc6\xb4",
            upper: b"\xc6\xb3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01B4,
        SpecialCasingMapping {
            lower: b"\xc6\xb4",
            upper: b"\xc6\xb3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01B5,
        SpecialCasingMapping {
            lower: b"\xc6\xb6",
            upper: b"\xc6\xb5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01B6,
        SpecialCasingMapping {
            lower: b"\xc6\xb6",
            upper: b"\xc6\xb5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01B7,
        SpecialCasingMapping {
            lower: b"\xca\x92",
            upper: b"\xc6\xb7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01B8,
        SpecialCasingMapping {
            lower: b"\xc6\xb9",
            upper: b"\xc6\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01B9,
        SpecialCasingMapping {
            lower: b"\xc6\xb9",
            upper: b"\xc6\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01BC,
        SpecialCasingMapping {
            lower: b"\xc6\xbd",
            upper: b"\xc6\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01BD,
        SpecialCasingMapping {
            lower: b"\xc6\xbd",
            upper: b"\xc6\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01BF,
        SpecialCasingMapping {
            lower: b"\xc6\xbf",
            upper: b"\xc7\xb7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01C4,
        SpecialCasingMapping {
            lower: b"\xc7\x86",
            upper: b"\xc7\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01C5,
        SpecialCasingMapping {
            lower: b"\xc7\x86",
            upper: b"\xc7\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01C6,
        SpecialCasingMapping {
            lower: b"\xc7\x86",
            upper: b"\xc7\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01C7,
        SpecialCasingMapping {
            lower: b"\xc7\x89",
            upper: b"\xc7\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01C8,
        SpecialCasingMapping {
            lower: b"\xc7\x89",
            upper: b"\xc7\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01C9,
        SpecialCasingMapping {
            lower: b"\xc7\x89",
            upper: b"\xc7\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01CA,
        SpecialCasingMapping {
            lower: b"\xc7\x8c",
            upper: b"\xc7\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01CB,
        SpecialCasingMapping {
            lower: b"\xc7\x8c",
            upper: b"\xc7\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01CC,
        SpecialCasingMapping {
            lower: b"\xc7\x8c",
            upper: b"\xc7\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01CD,
        SpecialCasingMapping {
            lower: b"\xc7\x8e",
            upper: b"\xc7\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01CE,
        SpecialCasingMapping {
            lower: b"\xc7\x8e",
            upper: b"\xc7\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01CF,
        SpecialCasingMapping {
            lower: b"\xc7\x90",
            upper: b"\xc7\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01D0,
        SpecialCasingMapping {
            lower: b"\xc7\x90",
            upper: b"\xc7\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01D1,
        SpecialCasingMapping {
            lower: b"\xc7\x92",
            upper: b"\xc7\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01D2,
        SpecialCasingMapping {
            lower: b"\xc7\x92",
            upper: b"\xc7\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01D3,
        SpecialCasingMapping {
            lower: b"\xc7\x94",
            upper: b"\xc7\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01D4,
        SpecialCasingMapping {
            lower: b"\xc7\x94",
            upper: b"\xc7\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01D5,
        SpecialCasingMapping {
            lower: b"\xc7\x96",
            upper: b"\xc7\x95",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01D6,
        SpecialCasingMapping {
            lower: b"\xc7\x96",
            upper: b"\xc7\x95",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01D7,
        SpecialCasingMapping {
            lower: b"\xc7\x98",
            upper: b"\xc7\x97",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01D8,
        SpecialCasingMapping {
            lower: b"\xc7\x98",
            upper: b"\xc7\x97",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01D9,
        SpecialCasingMapping {
            lower: b"\xc7\x9a",
            upper: b"\xc7\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01DA,
        SpecialCasingMapping {
            lower: b"\xc7\x9a",
            upper: b"\xc7\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01DB,
        SpecialCasingMapping {
            lower: b"\xc7\x9c",
            upper: b"\xc7\x9b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01DC,
        SpecialCasingMapping {
            lower: b"\xc7\x9c",
            upper: b"\xc7\x9b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01DD,
        SpecialCasingMapping {
            lower: b"\xc7\x9d",
            upper: b"\xc6\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01DE,
        SpecialCasingMapping {
            lower: b"\xc7\x9f",
            upper: b"\xc7\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01DF,
        SpecialCasingMapping {
            lower: b"\xc7\x9f",
            upper: b"\xc7\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01E0,
        SpecialCasingMapping {
            lower: b"\xc7\xa1",
            upper: b"\xc7\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01E1,
        SpecialCasingMapping {
            lower: b"\xc7\xa1",
            upper: b"\xc7\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01E2,
        SpecialCasingMapping {
            lower: b"\xc7\xa3",
            upper: b"\xc7\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01E3,
        SpecialCasingMapping {
            lower: b"\xc7\xa3",
            upper: b"\xc7\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01E4,
        SpecialCasingMapping {
            lower: b"\xc7\xa5",
            upper: b"\xc7\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01E5,
        SpecialCasingMapping {
            lower: b"\xc7\xa5",
            upper: b"\xc7\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01E6,
        SpecialCasingMapping {
            lower: b"\xc7\xa7",
            upper: b"\xc7\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01E7,
        SpecialCasingMapping {
            lower: b"\xc7\xa7",
            upper: b"\xc7\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01E8,
        SpecialCasingMapping {
            lower: b"\xc7\xa9",
            upper: b"\xc7\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01E9,
        SpecialCasingMapping {
            lower: b"\xc7\xa9",
            upper: b"\xc7\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01EA,
        SpecialCasingMapping {
            lower: b"\xc7\xab",
            upper: b"\xc7\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01EB,
        SpecialCasingMapping {
            lower: b"\xc7\xab",
            upper: b"\xc7\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01EC,
        SpecialCasingMapping {
            lower: b"\xc7\xad",
            upper: b"\xc7\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01ED,
        SpecialCasingMapping {
            lower: b"\xc7\xad",
            upper: b"\xc7\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01EE,
        SpecialCasingMapping {
            lower: b"\xc7\xaf",
            upper: b"\xc7\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01EF,
        SpecialCasingMapping {
            lower: b"\xc7\xaf",
            upper: b"\xc7\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01F0,
        SpecialCasingMapping {
            lower: b"\xc7\xb0",
            upper: b"J\xcc\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01F1,
        SpecialCasingMapping {
            lower: b"\xc7\xb3",
            upper: b"\xc7\xb1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01F2,
        SpecialCasingMapping {
            lower: b"\xc7\xb3",
            upper: b"\xc7\xb1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01F3,
        SpecialCasingMapping {
            lower: b"\xc7\xb3",
            upper: b"\xc7\xb1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01F4,
        SpecialCasingMapping {
            lower: b"\xc7\xb5",
            upper: b"\xc7\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01F5,
        SpecialCasingMapping {
            lower: b"\xc7\xb5",
            upper: b"\xc7\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01F6,
        SpecialCasingMapping {
            lower: b"\xc6\x95",
            upper: b"\xc7\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01F7,
        SpecialCasingMapping {
            lower: b"\xc6\xbf",
            upper: b"\xc7\xb7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01F8,
        SpecialCasingMapping {
            lower: b"\xc7\xb9",
            upper: b"\xc7\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01F9,
        SpecialCasingMapping {
            lower: b"\xc7\xb9",
            upper: b"\xc7\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01FA,
        SpecialCasingMapping {
            lower: b"\xc7\xbb",
            upper: b"\xc7\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01FB,
        SpecialCasingMapping {
            lower: b"\xc7\xbb",
            upper: b"\xc7\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01FC,
        SpecialCasingMapping {
            lower: b"\xc7\xbd",
            upper: b"\xc7\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01FD,
        SpecialCasingMapping {
            lower: b"\xc7\xbd",
            upper: b"\xc7\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01FE,
        SpecialCasingMapping {
            lower: b"\xc7\xbf",
            upper: b"\xc7\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x01FF,
        SpecialCasingMapping {
            lower: b"\xc7\xbf",
            upper: b"\xc7\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0200,
        SpecialCasingMapping {
            lower: b"\xc8\x81",
            upper: b"\xc8\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0201,
        SpecialCasingMapping {
            lower: b"\xc8\x81",
            upper: b"\xc8\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0202,
        SpecialCasingMapping {
            lower: b"\xc8\x83",
            upper: b"\xc8\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0203,
        SpecialCasingMapping {
            lower: b"\xc8\x83",
            upper: b"\xc8\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0204,
        SpecialCasingMapping {
            lower: b"\xc8\x85",
            upper: b"\xc8\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0205,
        SpecialCasingMapping {
            lower: b"\xc8\x85",
            upper: b"\xc8\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0206,
        SpecialCasingMapping {
            lower: b"\xc8\x87",
            upper: b"\xc8\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0207,
        SpecialCasingMapping {
            lower: b"\xc8\x87",
            upper: b"\xc8\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0208,
        SpecialCasingMapping {
            lower: b"\xc8\x89",
            upper: b"\xc8\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0209,
        SpecialCasingMapping {
            lower: b"\xc8\x89",
            upper: b"\xc8\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x020A,
        SpecialCasingMapping {
            lower: b"\xc8\x8b",
            upper: b"\xc8\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x020B,
        SpecialCasingMapping {
            lower: b"\xc8\x8b",
            upper: b"\xc8\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x020C,
        SpecialCasingMapping {
            lower: b"\xc8\x8d",
            upper: b"\xc8\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x020D,
        SpecialCasingMapping {
            lower: b"\xc8\x8d",
            upper: b"\xc8\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x020E,
        SpecialCasingMapping {
            lower: b"\xc8\x8f",
            upper: b"\xc8\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x020F,
        SpecialCasingMapping {
            lower: b"\xc8\x8f",
            upper: b"\xc8\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0210,
        SpecialCasingMapping {
            lower: b"\xc8\x91",
            upper: b"\xc8\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0211,
        SpecialCasingMapping {
            lower: b"\xc8\x91",
            upper: b"\xc8\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0212,
        SpecialCasingMapping {
            lower: b"\xc8\x93",
            upper: b"\xc8\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0213,
        SpecialCasingMapping {
            lower: b"\xc8\x93",
            upper: b"\xc8\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0214,
        SpecialCasingMapping {
            lower: b"\xc8\x95",
            upper: b"\xc8\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0215,
        SpecialCasingMapping {
            lower: b"\xc8\x95",
            upper: b"\xc8\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0216,
        SpecialCasingMapping {
            lower: b"\xc8\x97",
            upper: b"\xc8\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0217,
        SpecialCasingMapping {
            lower: b"\xc8\x97",
            upper: b"\xc8\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0218,
        SpecialCasingMapping {
            lower: b"\xc8\x99",
            upper: b"\xc8\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0219,
        SpecialCasingMapping {
            lower: b"\xc8\x99",
            upper: b"\xc8\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x021A,
        SpecialCasingMapping {
            lower: b"\xc8\x9b",
            upper: b"\xc8\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x021B,
        SpecialCasingMapping {
            lower: b"\xc8\x9b",
            upper: b"\xc8\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x021C,
        SpecialCasingMapping {
            lower: b"\xc8\x9d",
            upper: b"\xc8\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x021D,
        SpecialCasingMapping {
            lower: b"\xc8\x9d",
            upper: b"\xc8\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x021E,
        SpecialCasingMapping {
            lower: b"\xc8\x9f",
            upper: b"\xc8\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x021F,
        SpecialCasingMapping {
            lower: b"\xc8\x9f",
            upper: b"\xc8\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0220,
        SpecialCasingMapping {
            lower: b"\xc6\x9e",
            upper: b"\xc8\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0222,
        SpecialCasingMapping {
            lower: b"\xc8\xa3",
            upper: b"\xc8\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0223,
        SpecialCasingMapping {
            lower: b"\xc8\xa3",
            upper: b"\xc8\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0224,
        SpecialCasingMapping {
            lower: b"\xc8\xa5",
            upper: b"\xc8\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0225,
        SpecialCasingMapping {
            lower: b"\xc8\xa5",
            upper: b"\xc8\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0226,
        SpecialCasingMapping {
            lower: b"\xc8\xa7",
            upper: b"\xc8\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0227,
        SpecialCasingMapping {
            lower: b"\xc8\xa7",
            upper: b"\xc8\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0228,
        SpecialCasingMapping {
            lower: b"\xc8\xa9",
            upper: b"\xc8\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0229,
        SpecialCasingMapping {
            lower: b"\xc8\xa9",
            upper: b"\xc8\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x022A,
        SpecialCasingMapping {
            lower: b"\xc8\xab",
            upper: b"\xc8\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x022B,
        SpecialCasingMapping {
            lower: b"\xc8\xab",
            upper: b"\xc8\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x022C,
        SpecialCasingMapping {
            lower: b"\xc8\xad",
            upper: b"\xc8\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x022D,
        SpecialCasingMapping {
            lower: b"\xc8\xad",
            upper: b"\xc8\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x022E,
        SpecialCasingMapping {
            lower: b"\xc8\xaf",
            upper: b"\xc8\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x022F,
        SpecialCasingMapping {
            lower: b"\xc8\xaf",
            upper: b"\xc8\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0230,
        SpecialCasingMapping {
            lower: b"\xc8\xb1",
            upper: b"\xc8\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0231,
        SpecialCasingMapping {
            lower: b"\xc8\xb1",
            upper: b"\xc8\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0232,
        SpecialCasingMapping {
            lower: b"\xc8\xb3",
            upper: b"\xc8\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0233,
        SpecialCasingMapping {
            lower: b"\xc8\xb3",
            upper: b"\xc8\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x023A,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\xa5",
            upper: b"\xc8\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x023B,
        SpecialCasingMapping {
            lower: b"\xc8\xbc",
            upper: b"\xc8\xbb",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x023C,
        SpecialCasingMapping {
            lower: b"\xc8\xbc",
            upper: b"\xc8\xbb",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x023D,
        SpecialCasingMapping {
            lower: b"\xc6\x9a",
            upper: b"\xc8\xbd",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x023E,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\xa6",
            upper: b"\xc8\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x023F,
        SpecialCasingMapping {
            lower: b"\xc8\xbf",
            upper: b"\xe2\xb1\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0240,
        SpecialCasingMapping {
            lower: b"\xc9\x80",
            upper: b"\xe2\xb1\xbf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0241,
        SpecialCasingMapping {
            lower: b"\xc9\x82",
            upper: b"\xc9\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0242,
        SpecialCasingMapping {
            lower: b"\xc9\x82",
            upper: b"\xc9\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0243,
        SpecialCasingMapping {
            lower: b"\xc6\x80",
            upper: b"\xc9\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0244,
        SpecialCasingMapping {
            lower: b"\xca\x89",
            upper: b"\xc9\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0245,
        SpecialCasingMapping {
            lower: b"\xca\x8c",
            upper: b"\xc9\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0246,
        SpecialCasingMapping {
            lower: b"\xc9\x87",
            upper: b"\xc9\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0247,
        SpecialCasingMapping {
            lower: b"\xc9\x87",
            upper: b"\xc9\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0248,
        SpecialCasingMapping {
            lower: b"\xc9\x89",
            upper: b"\xc9\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0249,
        SpecialCasingMapping {
            lower: b"\xc9\x89",
            upper: b"\xc9\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x024A,
        SpecialCasingMapping {
            lower: b"\xc9\x8b",
            upper: b"\xc9\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x024B,
        SpecialCasingMapping {
            lower: b"\xc9\x8b",
            upper: b"\xc9\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x024C,
        SpecialCasingMapping {
            lower: b"\xc9\x8d",
            upper: b"\xc9\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x024D,
        SpecialCasingMapping {
            lower: b"\xc9\x8d",
            upper: b"\xc9\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x024E,
        SpecialCasingMapping {
            lower: b"\xc9\x8f",
            upper: b"\xc9\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x024F,
        SpecialCasingMapping {
            lower: b"\xc9\x8f",
            upper: b"\xc9\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0250,
        SpecialCasingMapping {
            lower: b"\xc9\x90",
            upper: b"\xe2\xb1\xaf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0251,
        SpecialCasingMapping {
            lower: b"\xc9\x91",
            upper: b"\xe2\xb1\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0252,
        SpecialCasingMapping {
            lower: b"\xc9\x92",
            upper: b"\xe2\xb1\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0253,
        SpecialCasingMapping {
            lower: b"\xc9\x93",
            upper: b"\xc6\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0254,
        SpecialCasingMapping {
            lower: b"\xc9\x94",
            upper: b"\xc6\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0256,
        SpecialCasingMapping {
            lower: b"\xc9\x96",
            upper: b"\xc6\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0257,
        SpecialCasingMapping {
            lower: b"\xc9\x97",
            upper: b"\xc6\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0259,
        SpecialCasingMapping {
            lower: b"\xc9\x99",
            upper: b"\xc6\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x025B,
        SpecialCasingMapping {
            lower: b"\xc9\x9b",
            upper: b"\xc6\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x025C,
        SpecialCasingMapping {
            lower: b"\xc9\x9c",
            upper: b"\xea\x9e\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0260,
        SpecialCasingMapping {
            lower: b"\xc9\xa0",
            upper: b"\xc6\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0261,
        SpecialCasingMapping {
            lower: b"\xc9\xa1",
            upper: b"\xea\x9e\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0263,
        SpecialCasingMapping {
            lower: b"\xc9\xa3",
            upper: b"\xc6\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0265,
        SpecialCasingMapping {
            lower: b"\xc9\xa5",
            upper: b"\xea\x9e\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0266,
        SpecialCasingMapping {
            lower: b"\xc9\xa6",
            upper: b"\xea\x9e\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0268,
        SpecialCasingMapping {
            lower: b"\xc9\xa8",
            upper: b"\xc6\x97",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0269,
        SpecialCasingMapping {
            lower: b"\xc9\xa9",
            upper: b"\xc6\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x026A,
        SpecialCasingMapping {
            lower: b"\xc9\xaa",
            upper: b"\xea\x9e\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x026B,
        SpecialCasingMapping {
            lower: b"\xc9\xab",
            upper: b"\xe2\xb1\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x026C,
        SpecialCasingMapping {
            lower: b"\xc9\xac",
            upper: b"\xea\x9e\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x026F,
        SpecialCasingMapping {
            lower: b"\xc9\xaf",
            upper: b"\xc6\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0271,
        SpecialCasingMapping {
            lower: b"\xc9\xb1",
            upper: b"\xe2\xb1\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0272,
        SpecialCasingMapping {
            lower: b"\xc9\xb2",
            upper: b"\xc6\x9d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0275,
        SpecialCasingMapping {
            lower: b"\xc9\xb5",
            upper: b"\xc6\x9f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x027D,
        SpecialCasingMapping {
            lower: b"\xc9\xbd",
            upper: b"\xe2\xb1\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0280,
        SpecialCasingMapping {
            lower: b"\xca\x80",
            upper: b"\xc6\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0282,
        SpecialCasingMapping {
            lower: b"\xca\x82",
            upper: b"\xea\x9f\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0283,
        SpecialCasingMapping {
            lower: b"\xca\x83",
            upper: b"\xc6\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0287,
        SpecialCasingMapping {
            lower: b"\xca\x87",
            upper: b"\xea\x9e\xb1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0288,
        SpecialCasingMapping {
            lower: b"\xca\x88",
            upper: b"\xc6\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0289,
        SpecialCasingMapping {
            lower: b"\xca\x89",
            upper: b"\xc9\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x028A,
        SpecialCasingMapping {
            lower: b"\xca\x8a",
            upper: b"\xc6\xb1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x028B,
        SpecialCasingMapping {
            lower: b"\xca\x8b",
            upper: b"\xc6\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x028C,
        SpecialCasingMapping {
            lower: b"\xca\x8c",
            upper: b"\xc9\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0292,
        SpecialCasingMapping {
            lower: b"\xca\x92",
            upper: b"\xc6\xb7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x029D,
        SpecialCasingMapping {
            lower: b"\xca\x9d",
            upper: b"\xea\x9e\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x029E,
        SpecialCasingMapping {
            lower: b"\xca\x9e",
            upper: b"\xea\x9e\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0345,
        SpecialCasingMapping {
            lower: b"\xcd\x85",
            upper: b"\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0370,
        SpecialCasingMapping {
            lower: b"\xcd\xb1",
            upper: b"\xcd\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0371,
        SpecialCasingMapping {
            lower: b"\xcd\xb1",
            upper: b"\xcd\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0372,
        SpecialCasingMapping {
            lower: b"\xcd\xb3",
            upper: b"\xcd\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0373,
        SpecialCasingMapping {
            lower: b"\xcd\xb3",
            upper: b"\xcd\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0376,
        SpecialCasingMapping {
            lower: b"\xcd\xb7",
            upper: b"\xcd\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0377,
        SpecialCasingMapping {
            lower: b"\xcd\xb7",
            upper: b"\xcd\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x037B,
        SpecialCasingMapping {
            lower: b"\xcd\xbb",
            upper: b"\xcf\xbd",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x037C,
        SpecialCasingMapping {
            lower: b"\xcd\xbc",
            upper: b"\xcf\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x037D,
        SpecialCasingMapping {
            lower: b"\xcd\xbd",
            upper: b"\xcf\xbf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x037F,
        SpecialCasingMapping {
            lower: b"\xcf\xb3",
            upper: b"\xcd\xbf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0386,
        SpecialCasingMapping {
            lower: b"\xce\xac",
            upper: b"\xce\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0388,
        SpecialCasingMapping {
            lower: b"\xce\xad",
            upper: b"\xce\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0389,
        SpecialCasingMapping {
            lower: b"\xce\xae",
            upper: b"\xce\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x038A,
        SpecialCasingMapping {
            lower: b"\xce\xaf",
            upper: b"\xce\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x038C,
        SpecialCasingMapping {
            lower: b"\xcf\x8c",
            upper: b"\xce\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x038E,
        SpecialCasingMapping {
            lower: b"\xcf\x8d",
            upper: b"\xce\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x038F,
        SpecialCasingMapping {
            lower: b"\xcf\x8e",
            upper: b"\xce\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0390,
        SpecialCasingMapping {
            lower: b"\xce\x90",
            upper: b"\xce\x99\xcc\x88\xcc\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0391,
        SpecialCasingMapping {
            lower: b"\xce\xb1",
            upper: b"\xce\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0392,
        SpecialCasingMapping {
            lower: b"\xce\xb2",
            upper: b"\xce\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0393,
        SpecialCasingMapping {
            lower: b"\xce\xb3",
            upper: b"\xce\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0394,
        SpecialCasingMapping {
            lower: b"\xce\xb4",
            upper: b"\xce\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0395,
        SpecialCasingMapping {
            lower: b"\xce\xb5",
            upper: b"\xce\x95",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0396,
        SpecialCasingMapping {
            lower: b"\xce\xb6",
            upper: b"\xce\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0397,
        SpecialCasingMapping {
            lower: b"\xce\xb7",
            upper: b"\xce\x97",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0398,
        SpecialCasingMapping {
            lower: b"\xce\xb8",
            upper: b"\xce\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0399,
        SpecialCasingMapping {
            lower: b"\xce\xb9",
            upper: b"\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x039A,
        SpecialCasingMapping {
            lower: b"\xce\xba",
            upper: b"\xce\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x039B,
        SpecialCasingMapping {
            lower: b"\xce\xbb",
            upper: b"\xce\x9b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x039C,
        SpecialCasingMapping {
            lower: b"\xce\xbc",
            upper: b"\xce\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x039D,
        SpecialCasingMapping {
            lower: b"\xce\xbd",
            upper: b"\xce\x9d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x039E,
        SpecialCasingMapping {
            lower: b"\xce\xbe",
            upper: b"\xce\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x039F,
        SpecialCasingMapping {
            lower: b"\xce\xbf",
            upper: b"\xce\x9f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03A0,
        SpecialCasingMapping {
            lower: b"\xcf\x80",
            upper: b"\xce\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03A1,
        SpecialCasingMapping {
            lower: b"\xcf\x81",
            upper: b"\xce\xa1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03A3,
        SpecialCasingMapping {
            lower: b"\xcf\x83",
            upper: b"\xce\xa3",
            conditional_lower: b"\xcf\x82",
            condition: SpecialCasingCondition::FinalSigma,
        },
    ),
    (
        0x03A4,
        SpecialCasingMapping {
            lower: b"\xcf\x84",
            upper: b"\xce\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03A5,
        SpecialCasingMapping {
            lower: b"\xcf\x85",
            upper: b"\xce\xa5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03A6,
        SpecialCasingMapping {
            lower: b"\xcf\x86",
            upper: b"\xce\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03A7,
        SpecialCasingMapping {
            lower: b"\xcf\x87",
            upper: b"\xce\xa7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03A8,
        SpecialCasingMapping {
            lower: b"\xcf\x88",
            upper: b"\xce\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03A9,
        SpecialCasingMapping {
            lower: b"\xcf\x89",
            upper: b"\xce\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03AA,
        SpecialCasingMapping {
            lower: b"\xcf\x8a",
            upper: b"\xce\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03AB,
        SpecialCasingMapping {
            lower: b"\xcf\x8b",
            upper: b"\xce\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03AC,
        SpecialCasingMapping {
            lower: b"\xce\xac",
            upper: b"\xce\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03AD,
        SpecialCasingMapping {
            lower: b"\xce\xad",
            upper: b"\xce\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03AE,
        SpecialCasingMapping {
            lower: b"\xce\xae",
            upper: b"\xce\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03AF,
        SpecialCasingMapping {
            lower: b"\xce\xaf",
            upper: b"\xce\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03B0,
        SpecialCasingMapping {
            lower: b"\xce\xb0",
            upper: b"\xce\xa5\xcc\x88\xcc\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03B1,
        SpecialCasingMapping {
            lower: b"\xce\xb1",
            upper: b"\xce\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03B2,
        SpecialCasingMapping {
            lower: b"\xce\xb2",
            upper: b"\xce\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03B3,
        SpecialCasingMapping {
            lower: b"\xce\xb3",
            upper: b"\xce\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03B4,
        SpecialCasingMapping {
            lower: b"\xce\xb4",
            upper: b"\xce\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03B5,
        SpecialCasingMapping {
            lower: b"\xce\xb5",
            upper: b"\xce\x95",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03B6,
        SpecialCasingMapping {
            lower: b"\xce\xb6",
            upper: b"\xce\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03B7,
        SpecialCasingMapping {
            lower: b"\xce\xb7",
            upper: b"\xce\x97",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03B8,
        SpecialCasingMapping {
            lower: b"\xce\xb8",
            upper: b"\xce\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03B9,
        SpecialCasingMapping {
            lower: b"\xce\xb9",
            upper: b"\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03BA,
        SpecialCasingMapping {
            lower: b"\xce\xba",
            upper: b"\xce\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03BB,
        SpecialCasingMapping {
            lower: b"\xce\xbb",
            upper: b"\xce\x9b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03BC,
        SpecialCasingMapping {
            lower: b"\xce\xbc",
            upper: b"\xce\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03BD,
        SpecialCasingMapping {
            lower: b"\xce\xbd",
            upper: b"\xce\x9d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03BE,
        SpecialCasingMapping {
            lower: b"\xce\xbe",
            upper: b"\xce\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03BF,
        SpecialCasingMapping {
            lower: b"\xce\xbf",
            upper: b"\xce\x9f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03C0,
        SpecialCasingMapping {
            lower: b"\xcf\x80",
            upper: b"\xce\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03C1,
        SpecialCasingMapping {
            lower: b"\xcf\x81",
            upper: b"\xce\xa1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03C2,
        SpecialCasingMapping {
            lower: b"\xcf\x82",
            upper: b"\xce\xa3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03C3,
        SpecialCasingMapping {
            lower: b"\xcf\x83",
            upper: b"\xce\xa3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03C4,
        SpecialCasingMapping {
            lower: b"\xcf\x84",
            upper: b"\xce\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03C5,
        SpecialCasingMapping {
            lower: b"\xcf\x85",
            upper: b"\xce\xa5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03C6,
        SpecialCasingMapping {
            lower: b"\xcf\x86",
            upper: b"\xce\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03C7,
        SpecialCasingMapping {
            lower: b"\xcf\x87",
            upper: b"\xce\xa7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03C8,
        SpecialCasingMapping {
            lower: b"\xcf\x88",
            upper: b"\xce\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03C9,
        SpecialCasingMapping {
            lower: b"\xcf\x89",
            upper: b"\xce\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03CA,
        SpecialCasingMapping {
            lower: b"\xcf\x8a",
            upper: b"\xce\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03CB,
        SpecialCasingMapping {
            lower: b"\xcf\x8b",
            upper: b"\xce\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03CC,
        SpecialCasingMapping {
            lower: b"\xcf\x8c",
            upper: b"\xce\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03CD,
        SpecialCasingMapping {
            lower: b"\xcf\x8d",
            upper: b"\xce\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03CE,
        SpecialCasingMapping {
            lower: b"\xcf\x8e",
            upper: b"\xce\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03CF,
        SpecialCasingMapping {
            lower: b"\xcf\x97",
            upper: b"\xcf\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03D0,
        SpecialCasingMapping {
            lower: b"\xcf\x90",
            upper: b"\xce\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03D1,
        SpecialCasingMapping {
            lower: b"\xcf\x91",
            upper: b"\xce\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03D5,
        SpecialCasingMapping {
            lower: b"\xcf\x95",
            upper: b"\xce\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03D6,
        SpecialCasingMapping {
            lower: b"\xcf\x96",
            upper: b"\xce\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03D7,
        SpecialCasingMapping {
            lower: b"\xcf\x97",
            upper: b"\xcf\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03D8,
        SpecialCasingMapping {
            lower: b"\xcf\x99",
            upper: b"\xcf\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03D9,
        SpecialCasingMapping {
            lower: b"\xcf\x99",
            upper: b"\xcf\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03DA,
        SpecialCasingMapping {
            lower: b"\xcf\x9b",
            upper: b"\xcf\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03DB,
        SpecialCasingMapping {
            lower: b"\xcf\x9b",
            upper: b"\xcf\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03DC,
        SpecialCasingMapping {
            lower: b"\xcf\x9d",
            upper: b"\xcf\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03DD,
        SpecialCasingMapping {
            lower: b"\xcf\x9d",
            upper: b"\xcf\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03DE,
        SpecialCasingMapping {
            lower: b"\xcf\x9f",
            upper: b"\xcf\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03DF,
        SpecialCasingMapping {
            lower: b"\xcf\x9f",
            upper: b"\xcf\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03E0,
        SpecialCasingMapping {
            lower: b"\xcf\xa1",
            upper: b"\xcf\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03E1,
        SpecialCasingMapping {
            lower: b"\xcf\xa1",
            upper: b"\xcf\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03E2,
        SpecialCasingMapping {
            lower: b"\xcf\xa3",
            upper: b"\xcf\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03E3,
        SpecialCasingMapping {
            lower: b"\xcf\xa3",
            upper: b"\xcf\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03E4,
        SpecialCasingMapping {
            lower: b"\xcf\xa5",
            upper: b"\xcf\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03E5,
        SpecialCasingMapping {
            lower: b"\xcf\xa5",
            upper: b"\xcf\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03E6,
        SpecialCasingMapping {
            lower: b"\xcf\xa7",
            upper: b"\xcf\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03E7,
        SpecialCasingMapping {
            lower: b"\xcf\xa7",
            upper: b"\xcf\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03E8,
        SpecialCasingMapping {
            lower: b"\xcf\xa9",
            upper: b"\xcf\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03E9,
        SpecialCasingMapping {
            lower: b"\xcf\xa9",
            upper: b"\xcf\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03EA,
        SpecialCasingMapping {
            lower: b"\xcf\xab",
            upper: b"\xcf\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03EB,
        SpecialCasingMapping {
            lower: b"\xcf\xab",
            upper: b"\xcf\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03EC,
        SpecialCasingMapping {
            lower: b"\xcf\xad",
            upper: b"\xcf\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03ED,
        SpecialCasingMapping {
            lower: b"\xcf\xad",
            upper: b"\xcf\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03EE,
        SpecialCasingMapping {
            lower: b"\xcf\xaf",
            upper: b"\xcf\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03EF,
        SpecialCasingMapping {
            lower: b"\xcf\xaf",
            upper: b"\xcf\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03F0,
        SpecialCasingMapping {
            lower: b"\xcf\xb0",
            upper: b"\xce\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03F1,
        SpecialCasingMapping {
            lower: b"\xcf\xb1",
            upper: b"\xce\xa1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03F2,
        SpecialCasingMapping {
            lower: b"\xcf\xb2",
            upper: b"\xcf\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03F3,
        SpecialCasingMapping {
            lower: b"\xcf\xb3",
            upper: b"\xcd\xbf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03F4,
        SpecialCasingMapping {
            lower: b"\xce\xb8",
            upper: b"\xcf\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03F5,
        SpecialCasingMapping {
            lower: b"\xcf\xb5",
            upper: b"\xce\x95",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03F7,
        SpecialCasingMapping {
            lower: b"\xcf\xb8",
            upper: b"\xcf\xb7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03F8,
        SpecialCasingMapping {
            lower: b"\xcf\xb8",
            upper: b"\xcf\xb7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03F9,
        SpecialCasingMapping {
            lower: b"\xcf\xb2",
            upper: b"\xcf\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03FA,
        SpecialCasingMapping {
            lower: b"\xcf\xbb",
            upper: b"\xcf\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03FB,
        SpecialCasingMapping {
            lower: b"\xcf\xbb",
            upper: b"\xcf\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03FD,
        SpecialCasingMapping {
            lower: b"\xcd\xbb",
            upper: b"\xcf\xbd",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03FE,
        SpecialCasingMapping {
            lower: b"\xcd\xbc",
            upper: b"\xcf\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x03FF,
        SpecialCasingMapping {
            lower: b"\xcd\xbd",
            upper: b"\xcf\xbf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0400,
        SpecialCasingMapping {
            lower: b"\xd1\x90",
            upper: b"\xd0\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0401,
        SpecialCasingMapping {
            lower: b"\xd1\x91",
            upper: b"\xd0\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0402,
        SpecialCasingMapping {
            lower: b"\xd1\x92",
            upper: b"\xd0\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0403,
        SpecialCasingMapping {
            lower: b"\xd1\x93",
            upper: b"\xd0\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0404,
        SpecialCasingMapping {
            lower: b"\xd1\x94",
            upper: b"\xd0\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0405,
        SpecialCasingMapping {
            lower: b"\xd1\x95",
            upper: b"\xd0\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0406,
        SpecialCasingMapping {
            lower: b"\xd1\x96",
            upper: b"\xd0\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0407,
        SpecialCasingMapping {
            lower: b"\xd1\x97",
            upper: b"\xd0\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0408,
        SpecialCasingMapping {
            lower: b"\xd1\x98",
            upper: b"\xd0\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0409,
        SpecialCasingMapping {
            lower: b"\xd1\x99",
            upper: b"\xd0\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x040A,
        SpecialCasingMapping {
            lower: b"\xd1\x9a",
            upper: b"\xd0\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x040B,
        SpecialCasingMapping {
            lower: b"\xd1\x9b",
            upper: b"\xd0\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x040C,
        SpecialCasingMapping {
            lower: b"\xd1\x9c",
            upper: b"\xd0\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x040D,
        SpecialCasingMapping {
            lower: b"\xd1\x9d",
            upper: b"\xd0\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x040E,
        SpecialCasingMapping {
            lower: b"\xd1\x9e",
            upper: b"\xd0\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x040F,
        SpecialCasingMapping {
            lower: b"\xd1\x9f",
            upper: b"\xd0\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0410,
        SpecialCasingMapping {
            lower: b"\xd0\xb0",
            upper: b"\xd0\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0411,
        SpecialCasingMapping {
            lower: b"\xd0\xb1",
            upper: b"\xd0\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0412,
        SpecialCasingMapping {
            lower: b"\xd0\xb2",
            upper: b"\xd0\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0413,
        SpecialCasingMapping {
            lower: b"\xd0\xb3",
            upper: b"\xd0\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0414,
        SpecialCasingMapping {
            lower: b"\xd0\xb4",
            upper: b"\xd0\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0415,
        SpecialCasingMapping {
            lower: b"\xd0\xb5",
            upper: b"\xd0\x95",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0416,
        SpecialCasingMapping {
            lower: b"\xd0\xb6",
            upper: b"\xd0\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0417,
        SpecialCasingMapping {
            lower: b"\xd0\xb7",
            upper: b"\xd0\x97",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0418,
        SpecialCasingMapping {
            lower: b"\xd0\xb8",
            upper: b"\xd0\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0419,
        SpecialCasingMapping {
            lower: b"\xd0\xb9",
            upper: b"\xd0\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x041A,
        SpecialCasingMapping {
            lower: b"\xd0\xba",
            upper: b"\xd0\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x041B,
        SpecialCasingMapping {
            lower: b"\xd0\xbb",
            upper: b"\xd0\x9b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x041C,
        SpecialCasingMapping {
            lower: b"\xd0\xbc",
            upper: b"\xd0\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x041D,
        SpecialCasingMapping {
            lower: b"\xd0\xbd",
            upper: b"\xd0\x9d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x041E,
        SpecialCasingMapping {
            lower: b"\xd0\xbe",
            upper: b"\xd0\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x041F,
        SpecialCasingMapping {
            lower: b"\xd0\xbf",
            upper: b"\xd0\x9f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0420,
        SpecialCasingMapping {
            lower: b"\xd1\x80",
            upper: b"\xd0\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0421,
        SpecialCasingMapping {
            lower: b"\xd1\x81",
            upper: b"\xd0\xa1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0422,
        SpecialCasingMapping {
            lower: b"\xd1\x82",
            upper: b"\xd0\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0423,
        SpecialCasingMapping {
            lower: b"\xd1\x83",
            upper: b"\xd0\xa3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0424,
        SpecialCasingMapping {
            lower: b"\xd1\x84",
            upper: b"\xd0\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0425,
        SpecialCasingMapping {
            lower: b"\xd1\x85",
            upper: b"\xd0\xa5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0426,
        SpecialCasingMapping {
            lower: b"\xd1\x86",
            upper: b"\xd0\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0427,
        SpecialCasingMapping {
            lower: b"\xd1\x87",
            upper: b"\xd0\xa7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0428,
        SpecialCasingMapping {
            lower: b"\xd1\x88",
            upper: b"\xd0\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0429,
        SpecialCasingMapping {
            lower: b"\xd1\x89",
            upper: b"\xd0\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x042A,
        SpecialCasingMapping {
            lower: b"\xd1\x8a",
            upper: b"\xd0\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x042B,
        SpecialCasingMapping {
            lower: b"\xd1\x8b",
            upper: b"\xd0\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x042C,
        SpecialCasingMapping {
            lower: b"\xd1\x8c",
            upper: b"\xd0\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x042D,
        SpecialCasingMapping {
            lower: b"\xd1\x8d",
            upper: b"\xd0\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x042E,
        SpecialCasingMapping {
            lower: b"\xd1\x8e",
            upper: b"\xd0\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x042F,
        SpecialCasingMapping {
            lower: b"\xd1\x8f",
            upper: b"\xd0\xaf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0430,
        SpecialCasingMapping {
            lower: b"\xd0\xb0",
            upper: b"\xd0\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0431,
        SpecialCasingMapping {
            lower: b"\xd0\xb1",
            upper: b"\xd0\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0432,
        SpecialCasingMapping {
            lower: b"\xd0\xb2",
            upper: b"\xd0\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0433,
        SpecialCasingMapping {
            lower: b"\xd0\xb3",
            upper: b"\xd0\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0434,
        SpecialCasingMapping {
            lower: b"\xd0\xb4",
            upper: b"\xd0\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0435,
        SpecialCasingMapping {
            lower: b"\xd0\xb5",
            upper: b"\xd0\x95",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0436,
        SpecialCasingMapping {
            lower: b"\xd0\xb6",
            upper: b"\xd0\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0437,
        SpecialCasingMapping {
            lower: b"\xd0\xb7",
            upper: b"\xd0\x97",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0438,
        SpecialCasingMapping {
            lower: b"\xd0\xb8",
            upper: b"\xd0\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0439,
        SpecialCasingMapping {
            lower: b"\xd0\xb9",
            upper: b"\xd0\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x043A,
        SpecialCasingMapping {
            lower: b"\xd0\xba",
            upper: b"\xd0\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x043B,
        SpecialCasingMapping {
            lower: b"\xd0\xbb",
            upper: b"\xd0\x9b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x043C,
        SpecialCasingMapping {
            lower: b"\xd0\xbc",
            upper: b"\xd0\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x043D,
        SpecialCasingMapping {
            lower: b"\xd0\xbd",
            upper: b"\xd0\x9d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x043E,
        SpecialCasingMapping {
            lower: b"\xd0\xbe",
            upper: b"\xd0\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x043F,
        SpecialCasingMapping {
            lower: b"\xd0\xbf",
            upper: b"\xd0\x9f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0440,
        SpecialCasingMapping {
            lower: b"\xd1\x80",
            upper: b"\xd0\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0441,
        SpecialCasingMapping {
            lower: b"\xd1\x81",
            upper: b"\xd0\xa1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0442,
        SpecialCasingMapping {
            lower: b"\xd1\x82",
            upper: b"\xd0\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0443,
        SpecialCasingMapping {
            lower: b"\xd1\x83",
            upper: b"\xd0\xa3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0444,
        SpecialCasingMapping {
            lower: b"\xd1\x84",
            upper: b"\xd0\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0445,
        SpecialCasingMapping {
            lower: b"\xd1\x85",
            upper: b"\xd0\xa5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0446,
        SpecialCasingMapping {
            lower: b"\xd1\x86",
            upper: b"\xd0\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0447,
        SpecialCasingMapping {
            lower: b"\xd1\x87",
            upper: b"\xd0\xa7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0448,
        SpecialCasingMapping {
            lower: b"\xd1\x88",
            upper: b"\xd0\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0449,
        SpecialCasingMapping {
            lower: b"\xd1\x89",
            upper: b"\xd0\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x044A,
        SpecialCasingMapping {
            lower: b"\xd1\x8a",
            upper: b"\xd0\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x044B,
        SpecialCasingMapping {
            lower: b"\xd1\x8b",
            upper: b"\xd0\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x044C,
        SpecialCasingMapping {
            lower: b"\xd1\x8c",
            upper: b"\xd0\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x044D,
        SpecialCasingMapping {
            lower: b"\xd1\x8d",
            upper: b"\xd0\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x044E,
        SpecialCasingMapping {
            lower: b"\xd1\x8e",
            upper: b"\xd0\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x044F,
        SpecialCasingMapping {
            lower: b"\xd1\x8f",
            upper: b"\xd0\xaf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0450,
        SpecialCasingMapping {
            lower: b"\xd1\x90",
            upper: b"\xd0\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0451,
        SpecialCasingMapping {
            lower: b"\xd1\x91",
            upper: b"\xd0\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0452,
        SpecialCasingMapping {
            lower: b"\xd1\x92",
            upper: b"\xd0\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0453,
        SpecialCasingMapping {
            lower: b"\xd1\x93",
            upper: b"\xd0\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0454,
        SpecialCasingMapping {
            lower: b"\xd1\x94",
            upper: b"\xd0\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0455,
        SpecialCasingMapping {
            lower: b"\xd1\x95",
            upper: b"\xd0\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0456,
        SpecialCasingMapping {
            lower: b"\xd1\x96",
            upper: b"\xd0\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0457,
        SpecialCasingMapping {
            lower: b"\xd1\x97",
            upper: b"\xd0\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0458,
        SpecialCasingMapping {
            lower: b"\xd1\x98",
            upper: b"\xd0\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0459,
        SpecialCasingMapping {
            lower: b"\xd1\x99",
            upper: b"\xd0\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x045A,
        SpecialCasingMapping {
            lower: b"\xd1\x9a",
            upper: b"\xd0\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x045B,
        SpecialCasingMapping {
            lower: b"\xd1\x9b",
            upper: b"\xd0\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x045C,
        SpecialCasingMapping {
            lower: b"\xd1\x9c",
            upper: b"\xd0\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x045D,
        SpecialCasingMapping {
            lower: b"\xd1\x9d",
            upper: b"\xd0\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x045E,
        SpecialCasingMapping {
            lower: b"\xd1\x9e",
            upper: b"\xd0\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x045F,
        SpecialCasingMapping {
            lower: b"\xd1\x9f",
            upper: b"\xd0\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0460,
        SpecialCasingMapping {
            lower: b"\xd1\xa1",
            upper: b"\xd1\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0461,
        SpecialCasingMapping {
            lower: b"\xd1\xa1",
            upper: b"\xd1\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0462,
        SpecialCasingMapping {
            lower: b"\xd1\xa3",
            upper: b"\xd1\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0463,
        SpecialCasingMapping {
            lower: b"\xd1\xa3",
            upper: b"\xd1\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0464,
        SpecialCasingMapping {
            lower: b"\xd1\xa5",
            upper: b"\xd1\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0465,
        SpecialCasingMapping {
            lower: b"\xd1\xa5",
            upper: b"\xd1\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0466,
        SpecialCasingMapping {
            lower: b"\xd1\xa7",
            upper: b"\xd1\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0467,
        SpecialCasingMapping {
            lower: b"\xd1\xa7",
            upper: b"\xd1\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0468,
        SpecialCasingMapping {
            lower: b"\xd1\xa9",
            upper: b"\xd1\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0469,
        SpecialCasingMapping {
            lower: b"\xd1\xa9",
            upper: b"\xd1\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x046A,
        SpecialCasingMapping {
            lower: b"\xd1\xab",
            upper: b"\xd1\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x046B,
        SpecialCasingMapping {
            lower: b"\xd1\xab",
            upper: b"\xd1\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x046C,
        SpecialCasingMapping {
            lower: b"\xd1\xad",
            upper: b"\xd1\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x046D,
        SpecialCasingMapping {
            lower: b"\xd1\xad",
            upper: b"\xd1\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x046E,
        SpecialCasingMapping {
            lower: b"\xd1\xaf",
            upper: b"\xd1\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x046F,
        SpecialCasingMapping {
            lower: b"\xd1\xaf",
            upper: b"\xd1\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0470,
        SpecialCasingMapping {
            lower: b"\xd1\xb1",
            upper: b"\xd1\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0471,
        SpecialCasingMapping {
            lower: b"\xd1\xb1",
            upper: b"\xd1\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0472,
        SpecialCasingMapping {
            lower: b"\xd1\xb3",
            upper: b"\xd1\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0473,
        SpecialCasingMapping {
            lower: b"\xd1\xb3",
            upper: b"\xd1\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0474,
        SpecialCasingMapping {
            lower: b"\xd1\xb5",
            upper: b"\xd1\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0475,
        SpecialCasingMapping {
            lower: b"\xd1\xb5",
            upper: b"\xd1\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0476,
        SpecialCasingMapping {
            lower: b"\xd1\xb7",
            upper: b"\xd1\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0477,
        SpecialCasingMapping {
            lower: b"\xd1\xb7",
            upper: b"\xd1\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0478,
        SpecialCasingMapping {
            lower: b"\xd1\xb9",
            upper: b"\xd1\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0479,
        SpecialCasingMapping {
            lower: b"\xd1\xb9",
            upper: b"\xd1\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x047A,
        SpecialCasingMapping {
            lower: b"\xd1\xbb",
            upper: b"\xd1\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x047B,
        SpecialCasingMapping {
            lower: b"\xd1\xbb",
            upper: b"\xd1\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x047C,
        SpecialCasingMapping {
            lower: b"\xd1\xbd",
            upper: b"\xd1\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x047D,
        SpecialCasingMapping {
            lower: b"\xd1\xbd",
            upper: b"\xd1\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x047E,
        SpecialCasingMapping {
            lower: b"\xd1\xbf",
            upper: b"\xd1\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x047F,
        SpecialCasingMapping {
            lower: b"\xd1\xbf",
            upper: b"\xd1\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0480,
        SpecialCasingMapping {
            lower: b"\xd2\x81",
            upper: b"\xd2\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0481,
        SpecialCasingMapping {
            lower: b"\xd2\x81",
            upper: b"\xd2\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x048A,
        SpecialCasingMapping {
            lower: b"\xd2\x8b",
            upper: b"\xd2\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x048B,
        SpecialCasingMapping {
            lower: b"\xd2\x8b",
            upper: b"\xd2\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x048C,
        SpecialCasingMapping {
            lower: b"\xd2\x8d",
            upper: b"\xd2\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x048D,
        SpecialCasingMapping {
            lower: b"\xd2\x8d",
            upper: b"\xd2\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x048E,
        SpecialCasingMapping {
            lower: b"\xd2\x8f",
            upper: b"\xd2\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x048F,
        SpecialCasingMapping {
            lower: b"\xd2\x8f",
            upper: b"\xd2\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0490,
        SpecialCasingMapping {
            lower: b"\xd2\x91",
            upper: b"\xd2\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0491,
        SpecialCasingMapping {
            lower: b"\xd2\x91",
            upper: b"\xd2\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0492,
        SpecialCasingMapping {
            lower: b"\xd2\x93",
            upper: b"\xd2\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0493,
        SpecialCasingMapping {
            lower: b"\xd2\x93",
            upper: b"\xd2\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0494,
        SpecialCasingMapping {
            lower: b"\xd2\x95",
            upper: b"\xd2\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0495,
        SpecialCasingMapping {
            lower: b"\xd2\x95",
            upper: b"\xd2\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0496,
        SpecialCasingMapping {
            lower: b"\xd2\x97",
            upper: b"\xd2\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0497,
        SpecialCasingMapping {
            lower: b"\xd2\x97",
            upper: b"\xd2\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0498,
        SpecialCasingMapping {
            lower: b"\xd2\x99",
            upper: b"\xd2\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0499,
        SpecialCasingMapping {
            lower: b"\xd2\x99",
            upper: b"\xd2\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x049A,
        SpecialCasingMapping {
            lower: b"\xd2\x9b",
            upper: b"\xd2\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x049B,
        SpecialCasingMapping {
            lower: b"\xd2\x9b",
            upper: b"\xd2\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x049C,
        SpecialCasingMapping {
            lower: b"\xd2\x9d",
            upper: b"\xd2\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x049D,
        SpecialCasingMapping {
            lower: b"\xd2\x9d",
            upper: b"\xd2\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x049E,
        SpecialCasingMapping {
            lower: b"\xd2\x9f",
            upper: b"\xd2\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x049F,
        SpecialCasingMapping {
            lower: b"\xd2\x9f",
            upper: b"\xd2\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04A0,
        SpecialCasingMapping {
            lower: b"\xd2\xa1",
            upper: b"\xd2\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04A1,
        SpecialCasingMapping {
            lower: b"\xd2\xa1",
            upper: b"\xd2\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04A2,
        SpecialCasingMapping {
            lower: b"\xd2\xa3",
            upper: b"\xd2\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04A3,
        SpecialCasingMapping {
            lower: b"\xd2\xa3",
            upper: b"\xd2\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04A4,
        SpecialCasingMapping {
            lower: b"\xd2\xa5",
            upper: b"\xd2\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04A5,
        SpecialCasingMapping {
            lower: b"\xd2\xa5",
            upper: b"\xd2\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04A6,
        SpecialCasingMapping {
            lower: b"\xd2\xa7",
            upper: b"\xd2\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04A7,
        SpecialCasingMapping {
            lower: b"\xd2\xa7",
            upper: b"\xd2\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04A8,
        SpecialCasingMapping {
            lower: b"\xd2\xa9",
            upper: b"\xd2\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04A9,
        SpecialCasingMapping {
            lower: b"\xd2\xa9",
            upper: b"\xd2\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04AA,
        SpecialCasingMapping {
            lower: b"\xd2\xab",
            upper: b"\xd2\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04AB,
        SpecialCasingMapping {
            lower: b"\xd2\xab",
            upper: b"\xd2\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04AC,
        SpecialCasingMapping {
            lower: b"\xd2\xad",
            upper: b"\xd2\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04AD,
        SpecialCasingMapping {
            lower: b"\xd2\xad",
            upper: b"\xd2\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04AE,
        SpecialCasingMapping {
            lower: b"\xd2\xaf",
            upper: b"\xd2\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04AF,
        SpecialCasingMapping {
            lower: b"\xd2\xaf",
            upper: b"\xd2\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04B0,
        SpecialCasingMapping {
            lower: b"\xd2\xb1",
            upper: b"\xd2\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04B1,
        SpecialCasingMapping {
            lower: b"\xd2\xb1",
            upper: b"\xd2\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04B2,
        SpecialCasingMapping {
            lower: b"\xd2\xb3",
            upper: b"\xd2\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04B3,
        SpecialCasingMapping {
            lower: b"\xd2\xb3",
            upper: b"\xd2\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04B4,
        SpecialCasingMapping {
            lower: b"\xd2\xb5",
            upper: b"\xd2\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04B5,
        SpecialCasingMapping {
            lower: b"\xd2\xb5",
            upper: b"\xd2\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04B6,
        SpecialCasingMapping {
            lower: b"\xd2\xb7",
            upper: b"\xd2\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04B7,
        SpecialCasingMapping {
            lower: b"\xd2\xb7",
            upper: b"\xd2\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04B8,
        SpecialCasingMapping {
            lower: b"\xd2\xb9",
            upper: b"\xd2\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04B9,
        SpecialCasingMapping {
            lower: b"\xd2\xb9",
            upper: b"\xd2\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04BA,
        SpecialCasingMapping {
            lower: b"\xd2\xbb",
            upper: b"\xd2\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04BB,
        SpecialCasingMapping {
            lower: b"\xd2\xbb",
            upper: b"\xd2\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04BC,
        SpecialCasingMapping {
            lower: b"\xd2\xbd",
            upper: b"\xd2\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04BD,
        SpecialCasingMapping {
            lower: b"\xd2\xbd",
            upper: b"\xd2\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04BE,
        SpecialCasingMapping {
            lower: b"\xd2\xbf",
            upper: b"\xd2\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04BF,
        SpecialCasingMapping {
            lower: b"\xd2\xbf",
            upper: b"\xd2\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04C0,
        SpecialCasingMapping {
            lower: b"\xd3\x8f",
            upper: b"\xd3\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04C1,
        SpecialCasingMapping {
            lower: b"\xd3\x82",
            upper: b"\xd3\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04C2,
        SpecialCasingMapping {
            lower: b"\xd3\x82",
            upper: b"\xd3\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04C3,
        SpecialCasingMapping {
            lower: b"\xd3\x84",
            upper: b"\xd3\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04C4,
        SpecialCasingMapping {
            lower: b"\xd3\x84",
            upper: b"\xd3\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04C5,
        SpecialCasingMapping {
            lower: b"\xd3\x86",
            upper: b"\xd3\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04C6,
        SpecialCasingMapping {
            lower: b"\xd3\x86",
            upper: b"\xd3\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04C7,
        SpecialCasingMapping {
            lower: b"\xd3\x88",
            upper: b"\xd3\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04C8,
        SpecialCasingMapping {
            lower: b"\xd3\x88",
            upper: b"\xd3\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04C9,
        SpecialCasingMapping {
            lower: b"\xd3\x8a",
            upper: b"\xd3\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04CA,
        SpecialCasingMapping {
            lower: b"\xd3\x8a",
            upper: b"\xd3\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04CB,
        SpecialCasingMapping {
            lower: b"\xd3\x8c",
            upper: b"\xd3\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04CC,
        SpecialCasingMapping {
            lower: b"\xd3\x8c",
            upper: b"\xd3\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04CD,
        SpecialCasingMapping {
            lower: b"\xd3\x8e",
            upper: b"\xd3\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04CE,
        SpecialCasingMapping {
            lower: b"\xd3\x8e",
            upper: b"\xd3\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04CF,
        SpecialCasingMapping {
            lower: b"\xd3\x8f",
            upper: b"\xd3\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04D0,
        SpecialCasingMapping {
            lower: b"\xd3\x91",
            upper: b"\xd3\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04D1,
        SpecialCasingMapping {
            lower: b"\xd3\x91",
            upper: b"\xd3\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04D2,
        SpecialCasingMapping {
            lower: b"\xd3\x93",
            upper: b"\xd3\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04D3,
        SpecialCasingMapping {
            lower: b"\xd3\x93",
            upper: b"\xd3\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04D4,
        SpecialCasingMapping {
            lower: b"\xd3\x95",
            upper: b"\xd3\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04D5,
        SpecialCasingMapping {
            lower: b"\xd3\x95",
            upper: b"\xd3\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04D6,
        SpecialCasingMapping {
            lower: b"\xd3\x97",
            upper: b"\xd3\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04D7,
        SpecialCasingMapping {
            lower: b"\xd3\x97",
            upper: b"\xd3\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04D8,
        SpecialCasingMapping {
            lower: b"\xd3\x99",
            upper: b"\xd3\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04D9,
        SpecialCasingMapping {
            lower: b"\xd3\x99",
            upper: b"\xd3\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04DA,
        SpecialCasingMapping {
            lower: b"\xd3\x9b",
            upper: b"\xd3\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04DB,
        SpecialCasingMapping {
            lower: b"\xd3\x9b",
            upper: b"\xd3\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04DC,
        SpecialCasingMapping {
            lower: b"\xd3\x9d",
            upper: b"\xd3\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04DD,
        SpecialCasingMapping {
            lower: b"\xd3\x9d",
            upper: b"\xd3\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04DE,
        SpecialCasingMapping {
            lower: b"\xd3\x9f",
            upper: b"\xd3\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04DF,
        SpecialCasingMapping {
            lower: b"\xd3\x9f",
            upper: b"\xd3\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04E0,
        SpecialCasingMapping {
            lower: b"\xd3\xa1",
            upper: b"\xd3\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04E1,
        SpecialCasingMapping {
            lower: b"\xd3\xa1",
            upper: b"\xd3\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04E2,
        SpecialCasingMapping {
            lower: b"\xd3\xa3",
            upper: b"\xd3\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04E3,
        SpecialCasingMapping {
            lower: b"\xd3\xa3",
            upper: b"\xd3\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04E4,
        SpecialCasingMapping {
            lower: b"\xd3\xa5",
            upper: b"\xd3\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04E5,
        SpecialCasingMapping {
            lower: b"\xd3\xa5",
            upper: b"\xd3\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04E6,
        SpecialCasingMapping {
            lower: b"\xd3\xa7",
            upper: b"\xd3\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04E7,
        SpecialCasingMapping {
            lower: b"\xd3\xa7",
            upper: b"\xd3\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04E8,
        SpecialCasingMapping {
            lower: b"\xd3\xa9",
            upper: b"\xd3\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04E9,
        SpecialCasingMapping {
            lower: b"\xd3\xa9",
            upper: b"\xd3\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04EA,
        SpecialCasingMapping {
            lower: b"\xd3\xab",
            upper: b"\xd3\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04EB,
        SpecialCasingMapping {
            lower: b"\xd3\xab",
            upper: b"\xd3\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04EC,
        SpecialCasingMapping {
            lower: b"\xd3\xad",
            upper: b"\xd3\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04ED,
        SpecialCasingMapping {
            lower: b"\xd3\xad",
            upper: b"\xd3\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04EE,
        SpecialCasingMapping {
            lower: b"\xd3\xaf",
            upper: b"\xd3\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04EF,
        SpecialCasingMapping {
            lower: b"\xd3\xaf",
            upper: b"\xd3\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04F0,
        SpecialCasingMapping {
            lower: b"\xd3\xb1",
            upper: b"\xd3\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04F1,
        SpecialCasingMapping {
            lower: b"\xd3\xb1",
            upper: b"\xd3\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04F2,
        SpecialCasingMapping {
            lower: b"\xd3\xb3",
            upper: b"\xd3\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04F3,
        SpecialCasingMapping {
            lower: b"\xd3\xb3",
            upper: b"\xd3\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04F4,
        SpecialCasingMapping {
            lower: b"\xd3\xb5",
            upper: b"\xd3\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04F5,
        SpecialCasingMapping {
            lower: b"\xd3\xb5",
            upper: b"\xd3\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04F6,
        SpecialCasingMapping {
            lower: b"\xd3\xb7",
            upper: b"\xd3\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04F7,
        SpecialCasingMapping {
            lower: b"\xd3\xb7",
            upper: b"\xd3\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04F8,
        SpecialCasingMapping {
            lower: b"\xd3\xb9",
            upper: b"\xd3\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04F9,
        SpecialCasingMapping {
            lower: b"\xd3\xb9",
            upper: b"\xd3\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04FA,
        SpecialCasingMapping {
            lower: b"\xd3\xbb",
            upper: b"\xd3\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04FB,
        SpecialCasingMapping {
            lower: b"\xd3\xbb",
            upper: b"\xd3\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04FC,
        SpecialCasingMapping {
            lower: b"\xd3\xbd",
            upper: b"\xd3\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04FD,
        SpecialCasingMapping {
            lower: b"\xd3\xbd",
            upper: b"\xd3\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04FE,
        SpecialCasingMapping {
            lower: b"\xd3\xbf",
            upper: b"\xd3\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x04FF,
        SpecialCasingMapping {
            lower: b"\xd3\xbf",
            upper: b"\xd3\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0500,
        SpecialCasingMapping {
            lower: b"\xd4\x81",
            upper: b"\xd4\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0501,
        SpecialCasingMapping {
            lower: b"\xd4\x81",
            upper: b"\xd4\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0502,
        SpecialCasingMapping {
            lower: b"\xd4\x83",
            upper: b"\xd4\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0503,
        SpecialCasingMapping {
            lower: b"\xd4\x83",
            upper: b"\xd4\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0504,
        SpecialCasingMapping {
            lower: b"\xd4\x85",
            upper: b"\xd4\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0505,
        SpecialCasingMapping {
            lower: b"\xd4\x85",
            upper: b"\xd4\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0506,
        SpecialCasingMapping {
            lower: b"\xd4\x87",
            upper: b"\xd4\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0507,
        SpecialCasingMapping {
            lower: b"\xd4\x87",
            upper: b"\xd4\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0508,
        SpecialCasingMapping {
            lower: b"\xd4\x89",
            upper: b"\xd4\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0509,
        SpecialCasingMapping {
            lower: b"\xd4\x89",
            upper: b"\xd4\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x050A,
        SpecialCasingMapping {
            lower: b"\xd4\x8b",
            upper: b"\xd4\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x050B,
        SpecialCasingMapping {
            lower: b"\xd4\x8b",
            upper: b"\xd4\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x050C,
        SpecialCasingMapping {
            lower: b"\xd4\x8d",
            upper: b"\xd4\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x050D,
        SpecialCasingMapping {
            lower: b"\xd4\x8d",
            upper: b"\xd4\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x050E,
        SpecialCasingMapping {
            lower: b"\xd4\x8f",
            upper: b"\xd4\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x050F,
        SpecialCasingMapping {
            lower: b"\xd4\x8f",
            upper: b"\xd4\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0510,
        SpecialCasingMapping {
            lower: b"\xd4\x91",
            upper: b"\xd4\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0511,
        SpecialCasingMapping {
            lower: b"\xd4\x91",
            upper: b"\xd4\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0512,
        SpecialCasingMapping {
            lower: b"\xd4\x93",
            upper: b"\xd4\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0513,
        SpecialCasingMapping {
            lower: b"\xd4\x93",
            upper: b"\xd4\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0514,
        SpecialCasingMapping {
            lower: b"\xd4\x95",
            upper: b"\xd4\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0515,
        SpecialCasingMapping {
            lower: b"\xd4\x95",
            upper: b"\xd4\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0516,
        SpecialCasingMapping {
            lower: b"\xd4\x97",
            upper: b"\xd4\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0517,
        SpecialCasingMapping {
            lower: b"\xd4\x97",
            upper: b"\xd4\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0518,
        SpecialCasingMapping {
            lower: b"\xd4\x99",
            upper: b"\xd4\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0519,
        SpecialCasingMapping {
            lower: b"\xd4\x99",
            upper: b"\xd4\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x051A,
        SpecialCasingMapping {
            lower: b"\xd4\x9b",
            upper: b"\xd4\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x051B,
        SpecialCasingMapping {
            lower: b"\xd4\x9b",
            upper: b"\xd4\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x051C,
        SpecialCasingMapping {
            lower: b"\xd4\x9d",
            upper: b"\xd4\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x051D,
        SpecialCasingMapping {
            lower: b"\xd4\x9d",
            upper: b"\xd4\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x051E,
        SpecialCasingMapping {
            lower: b"\xd4\x9f",
            upper: b"\xd4\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x051F,
        SpecialCasingMapping {
            lower: b"\xd4\x9f",
            upper: b"\xd4\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0520,
        SpecialCasingMapping {
            lower: b"\xd4\xa1",
            upper: b"\xd4\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0521,
        SpecialCasingMapping {
            lower: b"\xd4\xa1",
            upper: b"\xd4\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0522,
        SpecialCasingMapping {
            lower: b"\xd4\xa3",
            upper: b"\xd4\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0523,
        SpecialCasingMapping {
            lower: b"\xd4\xa3",
            upper: b"\xd4\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0524,
        SpecialCasingMapping {
            lower: b"\xd4\xa5",
            upper: b"\xd4\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0525,
        SpecialCasingMapping {
            lower: b"\xd4\xa5",
            upper: b"\xd4\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0526,
        SpecialCasingMapping {
            lower: b"\xd4\xa7",
            upper: b"\xd4\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0527,
        SpecialCasingMapping {
            lower: b"\xd4\xa7",
            upper: b"\xd4\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0528,
        SpecialCasingMapping {
            lower: b"\xd4\xa9",
            upper: b"\xd4\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0529,
        SpecialCasingMapping {
            lower: b"\xd4\xa9",
            upper: b"\xd4\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x052A,
        SpecialCasingMapping {
            lower: b"\xd4\xab",
            upper: b"\xd4\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x052B,
        SpecialCasingMapping {
            lower: b"\xd4\xab",
            upper: b"\xd4\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x052C,
        SpecialCasingMapping {
            lower: b"\xd4\xad",
            upper: b"\xd4\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x052D,
        SpecialCasingMapping {
            lower: b"\xd4\xad",
            upper: b"\xd4\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x052E,
        SpecialCasingMapping {
            lower: b"\xd4\xaf",
            upper: b"\xd4\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x052F,
        SpecialCasingMapping {
            lower: b"\xd4\xaf",
            upper: b"\xd4\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0531,
        SpecialCasingMapping {
            lower: b"\xd5\xa1",
            upper: b"\xd4\xb1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0532,
        SpecialCasingMapping {
            lower: b"\xd5\xa2",
            upper: b"\xd4\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0533,
        SpecialCasingMapping {
            lower: b"\xd5\xa3",
            upper: b"\xd4\xb3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0534,
        SpecialCasingMapping {
            lower: b"\xd5\xa4",
            upper: b"\xd4\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0535,
        SpecialCasingMapping {
            lower: b"\xd5\xa5",
            upper: b"\xd4\xb5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0536,
        SpecialCasingMapping {
            lower: b"\xd5\xa6",
            upper: b"\xd4\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0537,
        SpecialCasingMapping {
            lower: b"\xd5\xa7",
            upper: b"\xd4\xb7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0538,
        SpecialCasingMapping {
            lower: b"\xd5\xa8",
            upper: b"\xd4\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0539,
        SpecialCasingMapping {
            lower: b"\xd5\xa9",
            upper: b"\xd4\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x053A,
        SpecialCasingMapping {
            lower: b"\xd5\xaa",
            upper: b"\xd4\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x053B,
        SpecialCasingMapping {
            lower: b"\xd5\xab",
            upper: b"\xd4\xbb",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x053C,
        SpecialCasingMapping {
            lower: b"\xd5\xac",
            upper: b"\xd4\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x053D,
        SpecialCasingMapping {
            lower: b"\xd5\xad",
            upper: b"\xd4\xbd",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x053E,
        SpecialCasingMapping {
            lower: b"\xd5\xae",
            upper: b"\xd4\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x053F,
        SpecialCasingMapping {
            lower: b"\xd5\xaf",
            upper: b"\xd4\xbf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0540,
        SpecialCasingMapping {
            lower: b"\xd5\xb0",
            upper: b"\xd5\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0541,
        SpecialCasingMapping {
            lower: b"\xd5\xb1",
            upper: b"\xd5\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0542,
        SpecialCasingMapping {
            lower: b"\xd5\xb2",
            upper: b"\xd5\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0543,
        SpecialCasingMapping {
            lower: b"\xd5\xb3",
            upper: b"\xd5\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0544,
        SpecialCasingMapping {
            lower: b"\xd5\xb4",
            upper: b"\xd5\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0545,
        SpecialCasingMapping {
            lower: b"\xd5\xb5",
            upper: b"\xd5\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0546,
        SpecialCasingMapping {
            lower: b"\xd5\xb6",
            upper: b"\xd5\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0547,
        SpecialCasingMapping {
            lower: b"\xd5\xb7",
            upper: b"\xd5\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0548,
        SpecialCasingMapping {
            lower: b"\xd5\xb8",
            upper: b"\xd5\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0549,
        SpecialCasingMapping {
            lower: b"\xd5\xb9",
            upper: b"\xd5\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x054A,
        SpecialCasingMapping {
            lower: b"\xd5\xba",
            upper: b"\xd5\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x054B,
        SpecialCasingMapping {
            lower: b"\xd5\xbb",
            upper: b"\xd5\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x054C,
        SpecialCasingMapping {
            lower: b"\xd5\xbc",
            upper: b"\xd5\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x054D,
        SpecialCasingMapping {
            lower: b"\xd5\xbd",
            upper: b"\xd5\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x054E,
        SpecialCasingMapping {
            lower: b"\xd5\xbe",
            upper: b"\xd5\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x054F,
        SpecialCasingMapping {
            lower: b"\xd5\xbf",
            upper: b"\xd5\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0550,
        SpecialCasingMapping {
            lower: b"\xd6\x80",
            upper: b"\xd5\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0551,
        SpecialCasingMapping {
            lower: b"\xd6\x81",
            upper: b"\xd5\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0552,
        SpecialCasingMapping {
            lower: b"\xd6\x82",
            upper: b"\xd5\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0553,
        SpecialCasingMapping {
            lower: b"\xd6\x83",
            upper: b"\xd5\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0554,
        SpecialCasingMapping {
            lower: b"\xd6\x84",
            upper: b"\xd5\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0555,
        SpecialCasingMapping {
            lower: b"\xd6\x85",
            upper: b"\xd5\x95",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0556,
        SpecialCasingMapping {
            lower: b"\xd6\x86",
            upper: b"\xd5\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0561,
        SpecialCasingMapping {
            lower: b"\xd5\xa1",
            upper: b"\xd4\xb1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0562,
        SpecialCasingMapping {
            lower: b"\xd5\xa2",
            upper: b"\xd4\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0563,
        SpecialCasingMapping {
            lower: b"\xd5\xa3",
            upper: b"\xd4\xb3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0564,
        SpecialCasingMapping {
            lower: b"\xd5\xa4",
            upper: b"\xd4\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0565,
        SpecialCasingMapping {
            lower: b"\xd5\xa5",
            upper: b"\xd4\xb5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0566,
        SpecialCasingMapping {
            lower: b"\xd5\xa6",
            upper: b"\xd4\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0567,
        SpecialCasingMapping {
            lower: b"\xd5\xa7",
            upper: b"\xd4\xb7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0568,
        SpecialCasingMapping {
            lower: b"\xd5\xa8",
            upper: b"\xd4\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0569,
        SpecialCasingMapping {
            lower: b"\xd5\xa9",
            upper: b"\xd4\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x056A,
        SpecialCasingMapping {
            lower: b"\xd5\xaa",
            upper: b"\xd4\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x056B,
        SpecialCasingMapping {
            lower: b"\xd5\xab",
            upper: b"\xd4\xbb",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x056C,
        SpecialCasingMapping {
            lower: b"\xd5\xac",
            upper: b"\xd4\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x056D,
        SpecialCasingMapping {
            lower: b"\xd5\xad",
            upper: b"\xd4\xbd",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x056E,
        SpecialCasingMapping {
            lower: b"\xd5\xae",
            upper: b"\xd4\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x056F,
        SpecialCasingMapping {
            lower: b"\xd5\xaf",
            upper: b"\xd4\xbf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0570,
        SpecialCasingMapping {
            lower: b"\xd5\xb0",
            upper: b"\xd5\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0571,
        SpecialCasingMapping {
            lower: b"\xd5\xb1",
            upper: b"\xd5\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0572,
        SpecialCasingMapping {
            lower: b"\xd5\xb2",
            upper: b"\xd5\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0573,
        SpecialCasingMapping {
            lower: b"\xd5\xb3",
            upper: b"\xd5\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0574,
        SpecialCasingMapping {
            lower: b"\xd5\xb4",
            upper: b"\xd5\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0575,
        SpecialCasingMapping {
            lower: b"\xd5\xb5",
            upper: b"\xd5\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0576,
        SpecialCasingMapping {
            lower: b"\xd5\xb6",
            upper: b"\xd5\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0577,
        SpecialCasingMapping {
            lower: b"\xd5\xb7",
            upper: b"\xd5\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0578,
        SpecialCasingMapping {
            lower: b"\xd5\xb8",
            upper: b"\xd5\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0579,
        SpecialCasingMapping {
            lower: b"\xd5\xb9",
            upper: b"\xd5\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x057A,
        SpecialCasingMapping {
            lower: b"\xd5\xba",
            upper: b"\xd5\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x057B,
        SpecialCasingMapping {
            lower: b"\xd5\xbb",
            upper: b"\xd5\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x057C,
        SpecialCasingMapping {
            lower: b"\xd5\xbc",
            upper: b"\xd5\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x057D,
        SpecialCasingMapping {
            lower: b"\xd5\xbd",
            upper: b"\xd5\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x057E,
        SpecialCasingMapping {
            lower: b"\xd5\xbe",
            upper: b"\xd5\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x057F,
        SpecialCasingMapping {
            lower: b"\xd5\xbf",
            upper: b"\xd5\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0580,
        SpecialCasingMapping {
            lower: b"\xd6\x80",
            upper: b"\xd5\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0581,
        SpecialCasingMapping {
            lower: b"\xd6\x81",
            upper: b"\xd5\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0582,
        SpecialCasingMapping {
            lower: b"\xd6\x82",
            upper: b"\xd5\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0583,
        SpecialCasingMapping {
            lower: b"\xd6\x83",
            upper: b"\xd5\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0584,
        SpecialCasingMapping {
            lower: b"\xd6\x84",
            upper: b"\xd5\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0585,
        SpecialCasingMapping {
            lower: b"\xd6\x85",
            upper: b"\xd5\x95",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0586,
        SpecialCasingMapping {
            lower: b"\xd6\x86",
            upper: b"\xd5\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x0587,
        SpecialCasingMapping {
            lower: b"\xd6\x87",
            upper: b"\xd4\xb5\xd5\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10A0,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x80",
            upper: b"\xe1\x82\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10A1,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x81",
            upper: b"\xe1\x82\xa1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10A2,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x82",
            upper: b"\xe1\x82\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10A3,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x83",
            upper: b"\xe1\x82\xa3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10A4,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x84",
            upper: b"\xe1\x82\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10A5,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x85",
            upper: b"\xe1\x82\xa5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10A6,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x86",
            upper: b"\xe1\x82\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10A7,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x87",
            upper: b"\xe1\x82\xa7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10A8,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x88",
            upper: b"\xe1\x82\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10A9,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x89",
            upper: b"\xe1\x82\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10AA,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x8a",
            upper: b"\xe1\x82\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10AB,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x8b",
            upper: b"\xe1\x82\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10AC,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x8c",
            upper: b"\xe1\x82\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10AD,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x8d",
            upper: b"\xe1\x82\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10AE,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x8e",
            upper: b"\xe1\x82\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10AF,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x8f",
            upper: b"\xe1\x82\xaf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10B0,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x90",
            upper: b"\xe1\x82\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10B1,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x91",
            upper: b"\xe1\x82\xb1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10B2,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x92",
            upper: b"\xe1\x82\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10B3,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x93",
            upper: b"\xe1\x82\xb3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10B4,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x94",
            upper: b"\xe1\x82\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10B5,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x95",
            upper: b"\xe1\x82\xb5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10B6,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x96",
            upper: b"\xe1\x82\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10B7,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x97",
            upper: b"\xe1\x82\xb7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10B8,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x98",
            upper: b"\xe1\x82\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10B9,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x99",
            upper: b"\xe1\x82\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10BA,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x9a",
            upper: b"\xe1\x82\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10BB,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x9b",
            upper: b"\xe1\x82\xbb",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10BC,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x9c",
            upper: b"\xe1\x82\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10BD,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x9d",
            upper: b"\xe1\x82\xbd",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10BE,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x9e",
            upper: b"\xe1\x82\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10BF,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x9f",
            upper: b"\xe1\x82\xbf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C0,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\xa0",
            upper: b"\xe1\x83\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C1,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\xa1",
            upper: b"\xe1\x83\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C2,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\xa2",
            upper: b"\xe1\x83\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C3,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\xa3",
            upper: b"\xe1\x83\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C4,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\xa4",
            upper: b"\xe1\x83\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C5,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\xa5",
            upper: b"\xe1\x83\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C7,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\xa7",
            upper: b"\xe1\x83\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CD,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\xad",
            upper: b"\xe1\x83\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10D0,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x90",
            upper: b"\xe1\xb2\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10D1,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x91",
            upper: b"\xe1\xb2\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10D2,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x92",
            upper: b"\xe1\xb2\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10D3,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x93",
            upper: b"\xe1\xb2\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10D4,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x94",
            upper: b"\xe1\xb2\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10D5,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x95",
            upper: b"\xe1\xb2\x95",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10D6,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x96",
            upper: b"\xe1\xb2\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10D7,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x97",
            upper: b"\xe1\xb2\x97",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10D8,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x98",
            upper: b"\xe1\xb2\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10D9,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x99",
            upper: b"\xe1\xb2\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10DA,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x9a",
            upper: b"\xe1\xb2\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10DB,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x9b",
            upper: b"\xe1\xb2\x9b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10DC,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x9c",
            upper: b"\xe1\xb2\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10DD,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x9d",
            upper: b"\xe1\xb2\x9d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10DE,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x9e",
            upper: b"\xe1\xb2\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10DF,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x9f",
            upper: b"\xe1\xb2\x9f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10E0,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xa0",
            upper: b"\xe1\xb2\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10E1,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xa1",
            upper: b"\xe1\xb2\xa1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10E2,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xa2",
            upper: b"\xe1\xb2\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10E3,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xa3",
            upper: b"\xe1\xb2\xa3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10E4,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xa4",
            upper: b"\xe1\xb2\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10E5,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xa5",
            upper: b"\xe1\xb2\xa5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10E6,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xa6",
            upper: b"\xe1\xb2\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10E7,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xa7",
            upper: b"\xe1\xb2\xa7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10E8,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xa8",
            upper: b"\xe1\xb2\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10E9,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xa9",
            upper: b"\xe1\xb2\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10EA,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xaa",
            upper: b"\xe1\xb2\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10EB,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xab",
            upper: b"\xe1\xb2\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10EC,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xac",
            upper: b"\xe1\xb2\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10ED,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xad",
            upper: b"\xe1\xb2\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10EE,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xae",
            upper: b"\xe1\xb2\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10EF,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xaf",
            upper: b"\xe1\xb2\xaf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10F0,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xb0",
            upper: b"\xe1\xb2\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10F1,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xb1",
            upper: b"\xe1\xb2\xb1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10F2,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xb2",
            upper: b"\xe1\xb2\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10F3,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xb3",
            upper: b"\xe1\xb2\xb3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10F4,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xb4",
            upper: b"\xe1\xb2\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10F5,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xb5",
            upper: b"\xe1\xb2\xb5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10F6,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xb6",
            upper: b"\xe1\xb2\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10F7,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xb7",
            upper: b"\xe1\xb2\xb7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10F8,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xb8",
            upper: b"\xe1\xb2\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10F9,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xb9",
            upper: b"\xe1\xb2\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10FA,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xba",
            upper: b"\xe1\xb2\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10FD,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xbd",
            upper: b"\xe1\xb2\xbd",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10FE,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xbe",
            upper: b"\xe1\xb2\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10FF,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xbf",
            upper: b"\xe1\xb2\xbf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13A0,
        SpecialCasingMapping {
            lower: b"\xea\xad\xb0",
            upper: b"\xe1\x8e\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13A1,
        SpecialCasingMapping {
            lower: b"\xea\xad\xb1",
            upper: b"\xe1\x8e\xa1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13A2,
        SpecialCasingMapping {
            lower: b"\xea\xad\xb2",
            upper: b"\xe1\x8e\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13A3,
        SpecialCasingMapping {
            lower: b"\xea\xad\xb3",
            upper: b"\xe1\x8e\xa3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13A4,
        SpecialCasingMapping {
            lower: b"\xea\xad\xb4",
            upper: b"\xe1\x8e\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13A5,
        SpecialCasingMapping {
            lower: b"\xea\xad\xb5",
            upper: b"\xe1\x8e\xa5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13A6,
        SpecialCasingMapping {
            lower: b"\xea\xad\xb6",
            upper: b"\xe1\x8e\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13A7,
        SpecialCasingMapping {
            lower: b"\xea\xad\xb7",
            upper: b"\xe1\x8e\xa7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13A8,
        SpecialCasingMapping {
            lower: b"\xea\xad\xb8",
            upper: b"\xe1\x8e\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13A9,
        SpecialCasingMapping {
            lower: b"\xea\xad\xb9",
            upper: b"\xe1\x8e\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13AA,
        SpecialCasingMapping {
            lower: b"\xea\xad\xba",
            upper: b"\xe1\x8e\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13AB,
        SpecialCasingMapping {
            lower: b"\xea\xad\xbb",
            upper: b"\xe1\x8e\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13AC,
        SpecialCasingMapping {
            lower: b"\xea\xad\xbc",
            upper: b"\xe1\x8e\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13AD,
        SpecialCasingMapping {
            lower: b"\xea\xad\xbd",
            upper: b"\xe1\x8e\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13AE,
        SpecialCasingMapping {
            lower: b"\xea\xad\xbe",
            upper: b"\xe1\x8e\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13AF,
        SpecialCasingMapping {
            lower: b"\xea\xad\xbf",
            upper: b"\xe1\x8e\xaf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13B0,
        SpecialCasingMapping {
            lower: b"\xea\xae\x80",
            upper: b"\xe1\x8e\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13B1,
        SpecialCasingMapping {
            lower: b"\xea\xae\x81",
            upper: b"\xe1\x8e\xb1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13B2,
        SpecialCasingMapping {
            lower: b"\xea\xae\x82",
            upper: b"\xe1\x8e\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13B3,
        SpecialCasingMapping {
            lower: b"\xea\xae\x83",
            upper: b"\xe1\x8e\xb3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13B4,
        SpecialCasingMapping {
            lower: b"\xea\xae\x84",
            upper: b"\xe1\x8e\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13B5,
        SpecialCasingMapping {
            lower: b"\xea\xae\x85",
            upper: b"\xe1\x8e\xb5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13B6,
        SpecialCasingMapping {
            lower: b"\xea\xae\x86",
            upper: b"\xe1\x8e\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13B7,
        SpecialCasingMapping {
            lower: b"\xea\xae\x87",
            upper: b"\xe1\x8e\xb7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13B8,
        SpecialCasingMapping {
            lower: b"\xea\xae\x88",
            upper: b"\xe1\x8e\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13B9,
        SpecialCasingMapping {
            lower: b"\xea\xae\x89",
            upper: b"\xe1\x8e\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13BA,
        SpecialCasingMapping {
            lower: b"\xea\xae\x8a",
            upper: b"\xe1\x8e\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13BB,
        SpecialCasingMapping {
            lower: b"\xea\xae\x8b",
            upper: b"\xe1\x8e\xbb",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13BC,
        SpecialCasingMapping {
            lower: b"\xea\xae\x8c",
            upper: b"\xe1\x8e\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13BD,
        SpecialCasingMapping {
            lower: b"\xea\xae\x8d",
            upper: b"\xe1\x8e\xbd",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13BE,
        SpecialCasingMapping {
            lower: b"\xea\xae\x8e",
            upper: b"\xe1\x8e\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13BF,
        SpecialCasingMapping {
            lower: b"\xea\xae\x8f",
            upper: b"\xe1\x8e\xbf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13C0,
        SpecialCasingMapping {
            lower: b"\xea\xae\x90",
            upper: b"\xe1\x8f\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13C1,
        SpecialCasingMapping {
            lower: b"\xea\xae\x91",
            upper: b"\xe1\x8f\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13C2,
        SpecialCasingMapping {
            lower: b"\xea\xae\x92",
            upper: b"\xe1\x8f\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13C3,
        SpecialCasingMapping {
            lower: b"\xea\xae\x93",
            upper: b"\xe1\x8f\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13C4,
        SpecialCasingMapping {
            lower: b"\xea\xae\x94",
            upper: b"\xe1\x8f\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13C5,
        SpecialCasingMapping {
            lower: b"\xea\xae\x95",
            upper: b"\xe1\x8f\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13C6,
        SpecialCasingMapping {
            lower: b"\xea\xae\x96",
            upper: b"\xe1\x8f\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13C7,
        SpecialCasingMapping {
            lower: b"\xea\xae\x97",
            upper: b"\xe1\x8f\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13C8,
        SpecialCasingMapping {
            lower: b"\xea\xae\x98",
            upper: b"\xe1\x8f\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13C9,
        SpecialCasingMapping {
            lower: b"\xea\xae\x99",
            upper: b"\xe1\x8f\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13CA,
        SpecialCasingMapping {
            lower: b"\xea\xae\x9a",
            upper: b"\xe1\x8f\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13CB,
        SpecialCasingMapping {
            lower: b"\xea\xae\x9b",
            upper: b"\xe1\x8f\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13CC,
        SpecialCasingMapping {
            lower: b"\xea\xae\x9c",
            upper: b"\xe1\x8f\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13CD,
        SpecialCasingMapping {
            lower: b"\xea\xae\x9d",
            upper: b"\xe1\x8f\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13CE,
        SpecialCasingMapping {
            lower: b"\xea\xae\x9e",
            upper: b"\xe1\x8f\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13CF,
        SpecialCasingMapping {
            lower: b"\xea\xae\x9f",
            upper: b"\xe1\x8f\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13D0,
        SpecialCasingMapping {
            lower: b"\xea\xae\xa0",
            upper: b"\xe1\x8f\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13D1,
        SpecialCasingMapping {
            lower: b"\xea\xae\xa1",
            upper: b"\xe1\x8f\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13D2,
        SpecialCasingMapping {
            lower: b"\xea\xae\xa2",
            upper: b"\xe1\x8f\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13D3,
        SpecialCasingMapping {
            lower: b"\xea\xae\xa3",
            upper: b"\xe1\x8f\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13D4,
        SpecialCasingMapping {
            lower: b"\xea\xae\xa4",
            upper: b"\xe1\x8f\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13D5,
        SpecialCasingMapping {
            lower: b"\xea\xae\xa5",
            upper: b"\xe1\x8f\x95",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13D6,
        SpecialCasingMapping {
            lower: b"\xea\xae\xa6",
            upper: b"\xe1\x8f\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13D7,
        SpecialCasingMapping {
            lower: b"\xea\xae\xa7",
            upper: b"\xe1\x8f\x97",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13D8,
        SpecialCasingMapping {
            lower: b"\xea\xae\xa8",
            upper: b"\xe1\x8f\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13D9,
        SpecialCasingMapping {
            lower: b"\xea\xae\xa9",
            upper: b"\xe1\x8f\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13DA,
        SpecialCasingMapping {
            lower: b"\xea\xae\xaa",
            upper: b"\xe1\x8f\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13DB,
        SpecialCasingMapping {
            lower: b"\xea\xae\xab",
            upper: b"\xe1\x8f\x9b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13DC,
        SpecialCasingMapping {
            lower: b"\xea\xae\xac",
            upper: b"\xe1\x8f\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13DD,
        SpecialCasingMapping {
            lower: b"\xea\xae\xad",
            upper: b"\xe1\x8f\x9d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13DE,
        SpecialCasingMapping {
            lower: b"\xea\xae\xae",
            upper: b"\xe1\x8f\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13DF,
        SpecialCasingMapping {
            lower: b"\xea\xae\xaf",
            upper: b"\xe1\x8f\x9f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13E0,
        SpecialCasingMapping {
            lower: b"\xea\xae\xb0",
            upper: b"\xe1\x8f\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13E1,
        SpecialCasingMapping {
            lower: b"\xea\xae\xb1",
            upper: b"\xe1\x8f\xa1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13E2,
        SpecialCasingMapping {
            lower: b"\xea\xae\xb2",
            upper: b"\xe1\x8f\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13E3,
        SpecialCasingMapping {
            lower: b"\xea\xae\xb3",
            upper: b"\xe1\x8f\xa3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13E4,
        SpecialCasingMapping {
            lower: b"\xea\xae\xb4",
            upper: b"\xe1\x8f\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13E5,
        SpecialCasingMapping {
            lower: b"\xea\xae\xb5",
            upper: b"\xe1\x8f\xa5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13E6,
        SpecialCasingMapping {
            lower: b"\xea\xae\xb6",
            upper: b"\xe1\x8f\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13E7,
        SpecialCasingMapping {
            lower: b"\xea\xae\xb7",
            upper: b"\xe1\x8f\xa7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13E8,
        SpecialCasingMapping {
            lower: b"\xea\xae\xb8",
            upper: b"\xe1\x8f\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13E9,
        SpecialCasingMapping {
            lower: b"\xea\xae\xb9",
            upper: b"\xe1\x8f\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13EA,
        SpecialCasingMapping {
            lower: b"\xea\xae\xba",
            upper: b"\xe1\x8f\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13EB,
        SpecialCasingMapping {
            lower: b"\xea\xae\xbb",
            upper: b"\xe1\x8f\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13EC,
        SpecialCasingMapping {
            lower: b"\xea\xae\xbc",
            upper: b"\xe1\x8f\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13ED,
        SpecialCasingMapping {
            lower: b"\xea\xae\xbd",
            upper: b"\xe1\x8f\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13EE,
        SpecialCasingMapping {
            lower: b"\xea\xae\xbe",
            upper: b"\xe1\x8f\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13EF,
        SpecialCasingMapping {
            lower: b"\xea\xae\xbf",
            upper: b"\xe1\x8f\xaf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13F0,
        SpecialCasingMapping {
            lower: b"\xe1\x8f\xb8",
            upper: b"\xe1\x8f\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13F1,
        SpecialCasingMapping {
            lower: b"\xe1\x8f\xb9",
            upper: b"\xe1\x8f\xb1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13F2,
        SpecialCasingMapping {
            lower: b"\xe1\x8f\xba",
            upper: b"\xe1\x8f\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13F3,
        SpecialCasingMapping {
            lower: b"\xe1\x8f\xbb",
            upper: b"\xe1\x8f\xb3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13F4,
        SpecialCasingMapping {
            lower: b"\xe1\x8f\xbc",
            upper: b"\xe1\x8f\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13F5,
        SpecialCasingMapping {
            lower: b"\xe1\x8f\xbd",
            upper: b"\xe1\x8f\xb5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13F8,
        SpecialCasingMapping {
            lower: b"\xe1\x8f\xb8",
            upper: b"\xe1\x8f\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13F9,
        SpecialCasingMapping {
            lower: b"\xe1\x8f\xb9",
            upper: b"\xe1\x8f\xb1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13FA,
        SpecialCasingMapping {
            lower: b"\xe1\x8f\xba",
            upper: b"\xe1\x8f\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13FB,
        SpecialCasingMapping {
            lower: b"\xe1\x8f\xbb",
            upper: b"\xe1\x8f\xb3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13FC,
        SpecialCasingMapping {
            lower: b"\xe1\x8f\xbc",
            upper: b"\xe1\x8f\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x13FD,
        SpecialCasingMapping {
            lower: b"\xe1\x8f\xbd",
            upper: b"\xe1\x8f\xb5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1C80,
        SpecialCasingMapping {
            lower: b"\xe1\xb2\x80",
            upper: b"\xd0\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1C81,
        SpecialCasingMapping {
            lower: b"\xe1\xb2\x81",
            upper: b"\xd0\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1C82,
        SpecialCasingMapping {
            lower: b"\xe1\xb2\x82",
            upper: b"\xd0\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1C83,
        SpecialCasingMapping {
            lower: b"\xe1\xb2\x83",
            upper: b"\xd0\xa1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1C84,
        SpecialCasingMapping {
            lower: b"\xe1\xb2\x84",
            upper: b"\xd0\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1C85,
        SpecialCasingMapping {
            lower: b"\xe1\xb2\x85",
            upper: b"\xd0\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1C86,
        SpecialCasingMapping {
            lower: b"\xe1\xb2\x86",
            upper: b"\xd0\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1C87,
        SpecialCasingMapping {
            lower: b"\xe1\xb2\x87",
            upper: b"\xd1\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1C88,
        SpecialCasingMapping {
            lower: b"\xe1\xb2\x88",
            upper: b"\xea\x99\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1C90,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x90",
            upper: b"\xe1\xb2\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1C91,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x91",
            upper: b"\xe1\xb2\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1C92,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x92",
            upper: b"\xe1\xb2\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1C93,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x93",
            upper: b"\xe1\xb2\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1C94,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x94",
            upper: b"\xe1\xb2\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1C95,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x95",
            upper: b"\xe1\xb2\x95",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1C96,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x96",
            upper: b"\xe1\xb2\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1C97,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x97",
            upper: b"\xe1\xb2\x97",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1C98,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x98",
            upper: b"\xe1\xb2\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1C99,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x99",
            upper: b"\xe1\xb2\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1C9A,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x9a",
            upper: b"\xe1\xb2\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1C9B,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x9b",
            upper: b"\xe1\xb2\x9b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1C9C,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x9c",
            upper: b"\xe1\xb2\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1C9D,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x9d",
            upper: b"\xe1\xb2\x9d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1C9E,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x9e",
            upper: b"\xe1\xb2\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1C9F,
        SpecialCasingMapping {
            lower: b"\xe1\x83\x9f",
            upper: b"\xe1\xb2\x9f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CA0,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xa0",
            upper: b"\xe1\xb2\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CA1,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xa1",
            upper: b"\xe1\xb2\xa1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CA2,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xa2",
            upper: b"\xe1\xb2\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CA3,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xa3",
            upper: b"\xe1\xb2\xa3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CA4,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xa4",
            upper: b"\xe1\xb2\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CA5,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xa5",
            upper: b"\xe1\xb2\xa5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CA6,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xa6",
            upper: b"\xe1\xb2\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CA7,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xa7",
            upper: b"\xe1\xb2\xa7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CA8,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xa8",
            upper: b"\xe1\xb2\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CA9,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xa9",
            upper: b"\xe1\xb2\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CAA,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xaa",
            upper: b"\xe1\xb2\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CAB,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xab",
            upper: b"\xe1\xb2\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CAC,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xac",
            upper: b"\xe1\xb2\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CAD,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xad",
            upper: b"\xe1\xb2\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CAE,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xae",
            upper: b"\xe1\xb2\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CAF,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xaf",
            upper: b"\xe1\xb2\xaf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CB0,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xb0",
            upper: b"\xe1\xb2\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CB1,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xb1",
            upper: b"\xe1\xb2\xb1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CB2,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xb2",
            upper: b"\xe1\xb2\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CB3,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xb3",
            upper: b"\xe1\xb2\xb3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CB4,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xb4",
            upper: b"\xe1\xb2\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CB5,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xb5",
            upper: b"\xe1\xb2\xb5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CB6,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xb6",
            upper: b"\xe1\xb2\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CB7,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xb7",
            upper: b"\xe1\xb2\xb7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CB8,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xb8",
            upper: b"\xe1\xb2\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CB9,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xb9",
            upper: b"\xe1\xb2\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CBA,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xba",
            upper: b"\xe1\xb2\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CBD,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xbd",
            upper: b"\xe1\xb2\xbd",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CBE,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xbe",
            upper: b"\xe1\xb2\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1CBF,
        SpecialCasingMapping {
            lower: b"\xe1\x83\xbf",
            upper: b"\xe1\xb2\xbf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1D79,
        SpecialCasingMapping {
            lower: b"\xe1\xb5\xb9",
            upper: b"\xea\x9d\xbd",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1D7D,
        SpecialCasingMapping {
            lower: b"\xe1\xb5\xbd",
            upper: b"\xe2\xb1\xa3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1D8E,
        SpecialCasingMapping {
            lower: b"\xe1\xb6\x8e",
            upper: b"\xea\x9f\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E00,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x81",
            upper: b"\xe1\xb8\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E01,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x81",
            upper: b"\xe1\xb8\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E02,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x83",
            upper: b"\xe1\xb8\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E03,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x83",
            upper: b"\xe1\xb8\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E04,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x85",
            upper: b"\xe1\xb8\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E05,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x85",
            upper: b"\xe1\xb8\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E06,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x87",
            upper: b"\xe1\xb8\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E07,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x87",
            upper: b"\xe1\xb8\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E08,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x89",
            upper: b"\xe1\xb8\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E09,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x89",
            upper: b"\xe1\xb8\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E0A,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x8b",
            upper: b"\xe1\xb8\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E0B,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x8b",
            upper: b"\xe1\xb8\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E0C,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x8d",
            upper: b"\xe1\xb8\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E0D,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x8d",
            upper: b"\xe1\xb8\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E0E,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x8f",
            upper: b"\xe1\xb8\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E0F,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x8f",
            upper: b"\xe1\xb8\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E10,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x91",
            upper: b"\xe1\xb8\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E11,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x91",
            upper: b"\xe1\xb8\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E12,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x93",
            upper: b"\xe1\xb8\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E13,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x93",
            upper: b"\xe1\xb8\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E14,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x95",
            upper: b"\xe1\xb8\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E15,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x95",
            upper: b"\xe1\xb8\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E16,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x97",
            upper: b"\xe1\xb8\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E17,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x97",
            upper: b"\xe1\xb8\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E18,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x99",
            upper: b"\xe1\xb8\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E19,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x99",
            upper: b"\xe1\xb8\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E1A,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x9b",
            upper: b"\xe1\xb8\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E1B,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x9b",
            upper: b"\xe1\xb8\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E1C,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x9d",
            upper: b"\xe1\xb8\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E1D,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x9d",
            upper: b"\xe1\xb8\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E1E,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x9f",
            upper: b"\xe1\xb8\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E1F,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\x9f",
            upper: b"\xe1\xb8\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E20,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xa1",
            upper: b"\xe1\xb8\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E21,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xa1",
            upper: b"\xe1\xb8\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E22,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xa3",
            upper: b"\xe1\xb8\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E23,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xa3",
            upper: b"\xe1\xb8\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E24,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xa5",
            upper: b"\xe1\xb8\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E25,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xa5",
            upper: b"\xe1\xb8\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E26,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xa7",
            upper: b"\xe1\xb8\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E27,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xa7",
            upper: b"\xe1\xb8\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E28,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xa9",
            upper: b"\xe1\xb8\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E29,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xa9",
            upper: b"\xe1\xb8\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E2A,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xab",
            upper: b"\xe1\xb8\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E2B,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xab",
            upper: b"\xe1\xb8\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E2C,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xad",
            upper: b"\xe1\xb8\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E2D,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xad",
            upper: b"\xe1\xb8\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E2E,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xaf",
            upper: b"\xe1\xb8\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E2F,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xaf",
            upper: b"\xe1\xb8\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E30,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xb1",
            upper: b"\xe1\xb8\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E31,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xb1",
            upper: b"\xe1\xb8\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E32,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xb3",
            upper: b"\xe1\xb8\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E33,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xb3",
            upper: b"\xe1\xb8\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E34,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xb5",
            upper: b"\xe1\xb8\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E35,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xb5",
            upper: b"\xe1\xb8\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E36,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xb7",
            upper: b"\xe1\xb8\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E37,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xb7",
            upper: b"\xe1\xb8\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E38,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xb9",
            upper: b"\xe1\xb8\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E39,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xb9",
            upper: b"\xe1\xb8\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E3A,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xbb",
            upper: b"\xe1\xb8\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E3B,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xbb",
            upper: b"\xe1\xb8\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E3C,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xbd",
            upper: b"\xe1\xb8\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E3D,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xbd",
            upper: b"\xe1\xb8\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E3E,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xbf",
            upper: b"\xe1\xb8\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E3F,
        SpecialCasingMapping {
            lower: b"\xe1\xb8\xbf",
            upper: b"\xe1\xb8\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E40,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x81",
            upper: b"\xe1\xb9\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E41,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x81",
            upper: b"\xe1\xb9\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E42,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x83",
            upper: b"\xe1\xb9\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E43,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x83",
            upper: b"\xe1\xb9\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E44,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x85",
            upper: b"\xe1\xb9\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E45,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x85",
            upper: b"\xe1\xb9\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E46,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x87",
            upper: b"\xe1\xb9\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E47,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x87",
            upper: b"\xe1\xb9\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E48,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x89",
            upper: b"\xe1\xb9\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E49,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x89",
            upper: b"\xe1\xb9\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E4A,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x8b",
            upper: b"\xe1\xb9\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E4B,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x8b",
            upper: b"\xe1\xb9\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E4C,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x8d",
            upper: b"\xe1\xb9\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E4D,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x8d",
            upper: b"\xe1\xb9\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E4E,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x8f",
            upper: b"\xe1\xb9\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E4F,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x8f",
            upper: b"\xe1\xb9\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E50,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x91",
            upper: b"\xe1\xb9\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E51,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x91",
            upper: b"\xe1\xb9\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E52,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x93",
            upper: b"\xe1\xb9\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E53,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x93",
            upper: b"\xe1\xb9\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E54,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x95",
            upper: b"\xe1\xb9\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E55,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x95",
            upper: b"\xe1\xb9\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E56,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x97",
            upper: b"\xe1\xb9\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E57,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x97",
            upper: b"\xe1\xb9\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E58,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x99",
            upper: b"\xe1\xb9\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E59,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x99",
            upper: b"\xe1\xb9\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E5A,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x9b",
            upper: b"\xe1\xb9\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E5B,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x9b",
            upper: b"\xe1\xb9\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E5C,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x9d",
            upper: b"\xe1\xb9\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E5D,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x9d",
            upper: b"\xe1\xb9\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E5E,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x9f",
            upper: b"\xe1\xb9\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E5F,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\x9f",
            upper: b"\xe1\xb9\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E60,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xa1",
            upper: b"\xe1\xb9\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E61,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xa1",
            upper: b"\xe1\xb9\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E62,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xa3",
            upper: b"\xe1\xb9\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E63,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xa3",
            upper: b"\xe1\xb9\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E64,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xa5",
            upper: b"\xe1\xb9\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E65,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xa5",
            upper: b"\xe1\xb9\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E66,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xa7",
            upper: b"\xe1\xb9\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E67,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xa7",
            upper: b"\xe1\xb9\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E68,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xa9",
            upper: b"\xe1\xb9\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E69,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xa9",
            upper: b"\xe1\xb9\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E6A,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xab",
            upper: b"\xe1\xb9\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E6B,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xab",
            upper: b"\xe1\xb9\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E6C,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xad",
            upper: b"\xe1\xb9\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E6D,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xad",
            upper: b"\xe1\xb9\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E6E,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xaf",
            upper: b"\xe1\xb9\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E6F,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xaf",
            upper: b"\xe1\xb9\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E70,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xb1",
            upper: b"\xe1\xb9\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E71,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xb1",
            upper: b"\xe1\xb9\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E72,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xb3",
            upper: b"\xe1\xb9\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E73,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xb3",
            upper: b"\xe1\xb9\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E74,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xb5",
            upper: b"\xe1\xb9\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E75,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xb5",
            upper: b"\xe1\xb9\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E76,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xb7",
            upper: b"\xe1\xb9\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E77,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xb7",
            upper: b"\xe1\xb9\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E78,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xb9",
            upper: b"\xe1\xb9\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E79,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xb9",
            upper: b"\xe1\xb9\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E7A,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xbb",
            upper: b"\xe1\xb9\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E7B,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xbb",
            upper: b"\xe1\xb9\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E7C,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xbd",
            upper: b"\xe1\xb9\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E7D,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xbd",
            upper: b"\xe1\xb9\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E7E,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xbf",
            upper: b"\xe1\xb9\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E7F,
        SpecialCasingMapping {
            lower: b"\xe1\xb9\xbf",
            upper: b"\xe1\xb9\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E80,
        SpecialCasingMapping {
            lower: b"\xe1\xba\x81",
            upper: b"\xe1\xba\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E81,
        SpecialCasingMapping {
            lower: b"\xe1\xba\x81",
            upper: b"\xe1\xba\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E82,
        SpecialCasingMapping {
            lower: b"\xe1\xba\x83",
            upper: b"\xe1\xba\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E83,
        SpecialCasingMapping {
            lower: b"\xe1\xba\x83",
            upper: b"\xe1\xba\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E84,
        SpecialCasingMapping {
            lower: b"\xe1\xba\x85",
            upper: b"\xe1\xba\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E85,
        SpecialCasingMapping {
            lower: b"\xe1\xba\x85",
            upper: b"\xe1\xba\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E86,
        SpecialCasingMapping {
            lower: b"\xe1\xba\x87",
            upper: b"\xe1\xba\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E87,
        SpecialCasingMapping {
            lower: b"\xe1\xba\x87",
            upper: b"\xe1\xba\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E88,
        SpecialCasingMapping {
            lower: b"\xe1\xba\x89",
            upper: b"\xe1\xba\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E89,
        SpecialCasingMapping {
            lower: b"\xe1\xba\x89",
            upper: b"\xe1\xba\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E8A,
        SpecialCasingMapping {
            lower: b"\xe1\xba\x8b",
            upper: b"\xe1\xba\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E8B,
        SpecialCasingMapping {
            lower: b"\xe1\xba\x8b",
            upper: b"\xe1\xba\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E8C,
        SpecialCasingMapping {
            lower: b"\xe1\xba\x8d",
            upper: b"\xe1\xba\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E8D,
        SpecialCasingMapping {
            lower: b"\xe1\xba\x8d",
            upper: b"\xe1\xba\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E8E,
        SpecialCasingMapping {
            lower: b"\xe1\xba\x8f",
            upper: b"\xe1\xba\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E8F,
        SpecialCasingMapping {
            lower: b"\xe1\xba\x8f",
            upper: b"\xe1\xba\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E90,
        SpecialCasingMapping {
            lower: b"\xe1\xba\x91",
            upper: b"\xe1\xba\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E91,
        SpecialCasingMapping {
            lower: b"\xe1\xba\x91",
            upper: b"\xe1\xba\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E92,
        SpecialCasingMapping {
            lower: b"\xe1\xba\x93",
            upper: b"\xe1\xba\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E93,
        SpecialCasingMapping {
            lower: b"\xe1\xba\x93",
            upper: b"\xe1\xba\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E94,
        SpecialCasingMapping {
            lower: b"\xe1\xba\x95",
            upper: b"\xe1\xba\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E95,
        SpecialCasingMapping {
            lower: b"\xe1\xba\x95",
            upper: b"\xe1\xba\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E96,
        SpecialCasingMapping {
            lower: b"\xe1\xba\x96",
            upper: b"H\xcc\xb1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E97,
        SpecialCasingMapping {
            lower: b"\xe1\xba\x97",
            upper: b"T\xcc\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E98,
        SpecialCasingMapping {
            lower: b"\xe1\xba\x98",
            upper: b"W\xcc\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E99,
        SpecialCasingMapping {
            lower: b"\xe1\xba\x99",
            upper: b"Y\xcc\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E9A,
        SpecialCasingMapping {
            lower: b"\xe1\xba\x9a",
            upper: b"A\xca\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E9B,
        SpecialCasingMapping {
            lower: b"\xe1\xba\x9b",
            upper: b"\xe1\xb9\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E9E,
        SpecialCasingMapping {
            lower: b"\xc3\x9f",
            upper: b"\xe1\xba\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EA0,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xa1",
            upper: b"\xe1\xba\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EA1,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xa1",
            upper: b"\xe1\xba\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EA2,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xa3",
            upper: b"\xe1\xba\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EA3,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xa3",
            upper: b"\xe1\xba\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EA4,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xa5",
            upper: b"\xe1\xba\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EA5,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xa5",
            upper: b"\xe1\xba\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EA6,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xa7",
            upper: b"\xe1\xba\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EA7,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xa7",
            upper: b"\xe1\xba\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EA8,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xa9",
            upper: b"\xe1\xba\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EA9,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xa9",
            upper: b"\xe1\xba\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EAA,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xab",
            upper: b"\xe1\xba\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EAB,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xab",
            upper: b"\xe1\xba\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EAC,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xad",
            upper: b"\xe1\xba\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EAD,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xad",
            upper: b"\xe1\xba\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EAE,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xaf",
            upper: b"\xe1\xba\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EAF,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xaf",
            upper: b"\xe1\xba\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EB0,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xb1",
            upper: b"\xe1\xba\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EB1,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xb1",
            upper: b"\xe1\xba\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EB2,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xb3",
            upper: b"\xe1\xba\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EB3,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xb3",
            upper: b"\xe1\xba\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EB4,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xb5",
            upper: b"\xe1\xba\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EB5,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xb5",
            upper: b"\xe1\xba\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EB6,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xb7",
            upper: b"\xe1\xba\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EB7,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xb7",
            upper: b"\xe1\xba\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EB8,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xb9",
            upper: b"\xe1\xba\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EB9,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xb9",
            upper: b"\xe1\xba\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EBA,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xbb",
            upper: b"\xe1\xba\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EBB,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xbb",
            upper: b"\xe1\xba\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EBC,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xbd",
            upper: b"\xe1\xba\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EBD,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xbd",
            upper: b"\xe1\xba\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EBE,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xbf",
            upper: b"\xe1\xba\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EBF,
        SpecialCasingMapping {
            lower: b"\xe1\xba\xbf",
            upper: b"\xe1\xba\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EC0,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x81",
            upper: b"\xe1\xbb\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EC1,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x81",
            upper: b"\xe1\xbb\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EC2,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x83",
            upper: b"\xe1\xbb\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EC3,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x83",
            upper: b"\xe1\xbb\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EC4,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x85",
            upper: b"\xe1\xbb\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EC5,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x85",
            upper: b"\xe1\xbb\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EC6,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x87",
            upper: b"\xe1\xbb\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EC7,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x87",
            upper: b"\xe1\xbb\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EC8,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x89",
            upper: b"\xe1\xbb\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EC9,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x89",
            upper: b"\xe1\xbb\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1ECA,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x8b",
            upper: b"\xe1\xbb\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1ECB,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x8b",
            upper: b"\xe1\xbb\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1ECC,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x8d",
            upper: b"\xe1\xbb\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1ECD,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x8d",
            upper: b"\xe1\xbb\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1ECE,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x8f",
            upper: b"\xe1\xbb\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1ECF,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x8f",
            upper: b"\xe1\xbb\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1ED0,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x91",
            upper: b"\xe1\xbb\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1ED1,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x91",
            upper: b"\xe1\xbb\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1ED2,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x93",
            upper: b"\xe1\xbb\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1ED3,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x93",
            upper: b"\xe1\xbb\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1ED4,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x95",
            upper: b"\xe1\xbb\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1ED5,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x95",
            upper: b"\xe1\xbb\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1ED6,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x97",
            upper: b"\xe1\xbb\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1ED7,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x97",
            upper: b"\xe1\xbb\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1ED8,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x99",
            upper: b"\xe1\xbb\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1ED9,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x99",
            upper: b"\xe1\xbb\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EDA,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x9b",
            upper: b"\xe1\xbb\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EDB,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x9b",
            upper: b"\xe1\xbb\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EDC,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x9d",
            upper: b"\xe1\xbb\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EDD,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x9d",
            upper: b"\xe1\xbb\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EDE,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x9f",
            upper: b"\xe1\xbb\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EDF,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\x9f",
            upper: b"\xe1\xbb\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EE0,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xa1",
            upper: b"\xe1\xbb\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EE1,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xa1",
            upper: b"\xe1\xbb\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EE2,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xa3",
            upper: b"\xe1\xbb\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EE3,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xa3",
            upper: b"\xe1\xbb\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EE4,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xa5",
            upper: b"\xe1\xbb\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EE5,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xa5",
            upper: b"\xe1\xbb\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EE6,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xa7",
            upper: b"\xe1\xbb\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EE7,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xa7",
            upper: b"\xe1\xbb\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EE8,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xa9",
            upper: b"\xe1\xbb\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EE9,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xa9",
            upper: b"\xe1\xbb\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EEA,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xab",
            upper: b"\xe1\xbb\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EEB,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xab",
            upper: b"\xe1\xbb\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EEC,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xad",
            upper: b"\xe1\xbb\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EED,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xad",
            upper: b"\xe1\xbb\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EEE,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xaf",
            upper: b"\xe1\xbb\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EEF,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xaf",
            upper: b"\xe1\xbb\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EF0,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xb1",
            upper: b"\xe1\xbb\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EF1,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xb1",
            upper: b"\xe1\xbb\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EF2,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xb3",
            upper: b"\xe1\xbb\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EF3,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xb3",
            upper: b"\xe1\xbb\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EF4,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xb5",
            upper: b"\xe1\xbb\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EF5,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xb5",
            upper: b"\xe1\xbb\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EF6,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xb7",
            upper: b"\xe1\xbb\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EF7,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xb7",
            upper: b"\xe1\xbb\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EF8,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xb9",
            upper: b"\xe1\xbb\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EF9,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xb9",
            upper: b"\xe1\xbb\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EFA,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xbb",
            upper: b"\xe1\xbb\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EFB,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xbb",
            upper: b"\xe1\xbb\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EFC,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xbd",
            upper: b"\xe1\xbb\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EFD,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xbd",
            upper: b"\xe1\xbb\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EFE,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xbf",
            upper: b"\xe1\xbb\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1EFF,
        SpecialCasingMapping {
            lower: b"\xe1\xbb\xbf",
            upper: b"\xe1\xbb\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F00,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\x80",
            upper: b"\xe1\xbc\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F01,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\x81",
            upper: b"\xe1\xbc\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F02,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\x82",
            upper: b"\xe1\xbc\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F03,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\x83",
            upper: b"\xe1\xbc\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F04,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\x84",
            upper: b"\xe1\xbc\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F05,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\x85",
            upper: b"\xe1\xbc\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F06,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\x86",
            upper: b"\xe1\xbc\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F07,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\x87",
            upper: b"\xe1\xbc\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F08,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\x80",
            upper: b"\xe1\xbc\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F09,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\x81",
            upper: b"\xe1\xbc\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F0A,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\x82",
            upper: b"\xe1\xbc\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F0B,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\x83",
            upper: b"\xe1\xbc\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F0C,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\x84",
            upper: b"\xe1\xbc\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F0D,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\x85",
            upper: b"\xe1\xbc\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F0E,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\x86",
            upper: b"\xe1\xbc\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F0F,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\x87",
            upper: b"\xe1\xbc\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F10,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\x90",
            upper: b"\xe1\xbc\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F11,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\x91",
            upper: b"\xe1\xbc\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F12,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\x92",
            upper: b"\xe1\xbc\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F13,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\x93",
            upper: b"\xe1\xbc\x9b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F14,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\x94",
            upper: b"\xe1\xbc\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F15,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\x95",
            upper: b"\xe1\xbc\x9d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F18,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\x90",
            upper: b"\xe1\xbc\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F19,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\x91",
            upper: b"\xe1\xbc\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F1A,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\x92",
            upper: b"\xe1\xbc\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F1B,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\x93",
            upper: b"\xe1\xbc\x9b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F1C,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\x94",
            upper: b"\xe1\xbc\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F1D,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\x95",
            upper: b"\xe1\xbc\x9d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F20,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xa0",
            upper: b"\xe1\xbc\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F21,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xa1",
            upper: b"\xe1\xbc\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F22,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xa2",
            upper: b"\xe1\xbc\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F23,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xa3",
            upper: b"\xe1\xbc\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F24,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xa4",
            upper: b"\xe1\xbc\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F25,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xa5",
            upper: b"\xe1\xbc\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F26,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xa6",
            upper: b"\xe1\xbc\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F27,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xa7",
            upper: b"\xe1\xbc\xaf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F28,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xa0",
            upper: b"\xe1\xbc\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F29,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xa1",
            upper: b"\xe1\xbc\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F2A,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xa2",
            upper: b"\xe1\xbc\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F2B,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xa3",
            upper: b"\xe1\xbc\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F2C,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xa4",
            upper: b"\xe1\xbc\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F2D,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xa5",
            upper: b"\xe1\xbc\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F2E,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xa6",
            upper: b"\xe1\xbc\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F2F,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xa7",
            upper: b"\xe1\xbc\xaf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F30,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xb0",
            upper: b"\xe1\xbc\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F31,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xb1",
            upper: b"\xe1\xbc\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F32,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xb2",
            upper: b"\xe1\xbc\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F33,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xb3",
            upper: b"\xe1\xbc\xbb",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F34,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xb4",
            upper: b"\xe1\xbc\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F35,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xb5",
            upper: b"\xe1\xbc\xbd",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F36,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xb6",
            upper: b"\xe1\xbc\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F37,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xb7",
            upper: b"\xe1\xbc\xbf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F38,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xb0",
            upper: b"\xe1\xbc\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F39,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xb1",
            upper: b"\xe1\xbc\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F3A,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xb2",
            upper: b"\xe1\xbc\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F3B,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xb3",
            upper: b"\xe1\xbc\xbb",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F3C,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xb4",
            upper: b"\xe1\xbc\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F3D,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xb5",
            upper: b"\xe1\xbc\xbd",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F3E,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xb6",
            upper: b"\xe1\xbc\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F3F,
        SpecialCasingMapping {
            lower: b"\xe1\xbc\xb7",
            upper: b"\xe1\xbc\xbf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F40,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\x80",
            upper: b"\xe1\xbd\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F41,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\x81",
            upper: b"\xe1\xbd\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F42,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\x82",
            upper: b"\xe1\xbd\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F43,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\x83",
            upper: b"\xe1\xbd\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F44,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\x84",
            upper: b"\xe1\xbd\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F45,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\x85",
            upper: b"\xe1\xbd\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F48,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\x80",
            upper: b"\xe1\xbd\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F49,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\x81",
            upper: b"\xe1\xbd\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F4A,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\x82",
            upper: b"\xe1\xbd\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F4B,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\x83",
            upper: b"\xe1\xbd\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F4C,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\x84",
            upper: b"\xe1\xbd\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F4D,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\x85",
            upper: b"\xe1\xbd\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F50,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\x90",
            upper: b"\xce\xa5\xcc\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F51,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\x91",
            upper: b"\xe1\xbd\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F52,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\x92",
            upper: b"\xce\xa5\xcc\x93\xcc\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F53,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\x93",
            upper: b"\xe1\xbd\x9b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F54,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\x94",
            upper: b"\xce\xa5\xcc\x93\xcc\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F55,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\x95",
            upper: b"\xe1\xbd\x9d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F56,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\x96",
            upper: b"\xce\xa5\xcc\x93\xcd\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F57,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\x97",
            upper: b"\xe1\xbd\x9f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F59,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\x91",
            upper: b"\xe1\xbd\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F5B,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\x93",
            upper: b"\xe1\xbd\x9b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F5D,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\x95",
            upper: b"\xe1\xbd\x9d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F5F,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\x97",
            upper: b"\xe1\xbd\x9f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F60,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xa0",
            upper: b"\xe1\xbd\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F61,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xa1",
            upper: b"\xe1\xbd\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F62,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xa2",
            upper: b"\xe1\xbd\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F63,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xa3",
            upper: b"\xe1\xbd\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F64,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xa4",
            upper: b"\xe1\xbd\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F65,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xa5",
            upper: b"\xe1\xbd\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F66,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xa6",
            upper: b"\xe1\xbd\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F67,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xa7",
            upper: b"\xe1\xbd\xaf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F68,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xa0",
            upper: b"\xe1\xbd\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F69,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xa1",
            upper: b"\xe1\xbd\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F6A,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xa2",
            upper: b"\xe1\xbd\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F6B,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xa3",
            upper: b"\xe1\xbd\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F6C,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xa4",
            upper: b"\xe1\xbd\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F6D,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xa5",
            upper: b"\xe1\xbd\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F6E,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xa6",
            upper: b"\xe1\xbd\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F6F,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xa7",
            upper: b"\xe1\xbd\xaf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F70,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xb0",
            upper: b"\xe1\xbe\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F71,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xb1",
            upper: b"\xe1\xbe\xbb",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F72,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xb2",
            upper: b"\xe1\xbf\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F73,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xb3",
            upper: b"\xe1\xbf\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F74,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xb4",
            upper: b"\xe1\xbf\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F75,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xb5",
            upper: b"\xe1\xbf\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F76,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xb6",
            upper: b"\xe1\xbf\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F77,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xb7",
            upper: b"\xe1\xbf\x9b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F78,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xb8",
            upper: b"\xe1\xbf\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F79,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xb9",
            upper: b"\xe1\xbf\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F7A,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xba",
            upper: b"\xe1\xbf\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F7B,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xbb",
            upper: b"\xe1\xbf\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F7C,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xbc",
            upper: b"\xe1\xbf\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F7D,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xbd",
            upper: b"\xe1\xbf\xbb",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F80,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x80",
            upper: b"\xe1\xbc\x88\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F81,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x81",
            upper: b"\xe1\xbc\x89\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F82,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x82",
            upper: b"\xe1\xbc\x8a\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F83,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x83",
            upper: b"\xe1\xbc\x8b\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F84,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x84",
            upper: b"\xe1\xbc\x8c\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F85,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x85",
            upper: b"\xe1\xbc\x8d\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F86,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x86",
            upper: b"\xe1\xbc\x8e\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F87,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x87",
            upper: b"\xe1\xbc\x8f\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F88,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x80",
            upper: b"\xe1\xbc\x88\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F89,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x81",
            upper: b"\xe1\xbc\x89\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F8A,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x82",
            upper: b"\xe1\xbc\x8a\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F8B,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x83",
            upper: b"\xe1\xbc\x8b\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F8C,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x84",
            upper: b"\xe1\xbc\x8c\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F8D,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x85",
            upper: b"\xe1\xbc\x8d\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F8E,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x86",
            upper: b"\xe1\xbc\x8e\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F8F,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x87",
            upper: b"\xe1\xbc\x8f\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F90,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x90",
            upper: b"\xe1\xbc\xa8\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F91,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x91",
            upper: b"\xe1\xbc\xa9\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F92,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x92",
            upper: b"\xe1\xbc\xaa\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F93,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x93",
            upper: b"\xe1\xbc\xab\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F94,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x94",
            upper: b"\xe1\xbc\xac\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F95,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x95",
            upper: b"\xe1\xbc\xad\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F96,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x96",
            upper: b"\xe1\xbc\xae\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F97,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x97",
            upper: b"\xe1\xbc\xaf\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F98,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x90",
            upper: b"\xe1\xbc\xa8\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F99,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x91",
            upper: b"\xe1\xbc\xa9\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F9A,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x92",
            upper: b"\xe1\xbc\xaa\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F9B,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x93",
            upper: b"\xe1\xbc\xab\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F9C,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x94",
            upper: b"\xe1\xbc\xac\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F9D,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x95",
            upper: b"\xe1\xbc\xad\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F9E,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x96",
            upper: b"\xe1\xbc\xae\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1F9F,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\x97",
            upper: b"\xe1\xbc\xaf\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FA0,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\xa0",
            upper: b"\xe1\xbd\xa8\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FA1,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\xa1",
            upper: b"\xe1\xbd\xa9\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FA2,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\xa2",
            upper: b"\xe1\xbd\xaa\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FA3,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\xa3",
            upper: b"\xe1\xbd\xab\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FA4,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\xa4",
            upper: b"\xe1\xbd\xac\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FA5,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\xa5",
            upper: b"\xe1\xbd\xad\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FA6,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\xa6",
            upper: b"\xe1\xbd\xae\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FA7,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\xa7",
            upper: b"\xe1\xbd\xaf\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FA8,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\xa0",
            upper: b"\xe1\xbd\xa8\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FA9,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\xa1",
            upper: b"\xe1\xbd\xa9\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FAA,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\xa2",
            upper: b"\xe1\xbd\xaa\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FAB,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\xa3",
            upper: b"\xe1\xbd\xab\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FAC,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\xa4",
            upper: b"\xe1\xbd\xac\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FAD,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\xa5",
            upper: b"\xe1\xbd\xad\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FAE,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\xa6",
            upper: b"\xe1\xbd\xae\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FAF,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\xa7",
            upper: b"\xe1\xbd\xaf\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FB0,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\xb0",
            upper: b"\xe1\xbe\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FB1,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\xb1",
            upper: b"\xe1\xbe\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FB2,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\xb2",
            upper: b"\xe1\xbe\xba\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FB3,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\xb3",
            upper: b"\xce\x91\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FB4,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\xb4",
            upper: b"\xce\x86\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FB6,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\xb6",
            upper: b"\xce\x91\xcd\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FB7,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\xb7",
            upper: b"\xce\x91\xcd\x82\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FB8,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\xb0",
            upper: b"\xe1\xbe\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FB9,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\xb1",
            upper: b"\xe1\xbe\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FBA,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xb0",
            upper: b"\xe1\xbe\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FBB,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xb1",
            upper: b"\xe1\xbe\xbb",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FBC,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\xb3",
            upper: b"\xce\x91\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FBE,
        SpecialCasingMapping {
            lower: b"\xe1\xbe\xbe",
            upper: b"\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FC2,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\x82",
            upper: b"\xe1\xbf\x8a\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FC3,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\x83",
            upper: b"\xce\x97\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FC4,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\x84",
            upper: b"\xce\x89\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FC6,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\x86",
            upper: b"\xce\x97\xcd\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FC7,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\x87",
            upper: b"\xce\x97\xcd\x82\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FC8,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xb2",
            upper: b"\xe1\xbf\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FC9,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xb3",
            upper: b"\xe1\xbf\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FCA,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xb4",
            upper: b"\xe1\xbf\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FCB,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xb5",
            upper: b"\xe1\xbf\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FCC,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\x83",
            upper: b"\xce\x97\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FD0,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\x90",
            upper: b"\xe1\xbf\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FD1,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\x91",
            upper: b"\xe1\xbf\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FD2,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\x92",
            upper: b"\xce\x99\xcc\x88\xcc\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FD3,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\x93",
            upper: b"\xce\x99\xcc\x88\xcc\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FD6,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\x96",
            upper: b"\xce\x99\xcd\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FD7,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\x97",
            upper: b"\xce\x99\xcc\x88\xcd\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FD8,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\x90",
            upper: b"\xe1\xbf\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FD9,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\x91",
            upper: b"\xe1\xbf\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FDA,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xb6",
            upper: b"\xe1\xbf\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FDB,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xb7",
            upper: b"\xe1\xbf\x9b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FE0,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\xa0",
            upper: b"\xe1\xbf\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FE1,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\xa1",
            upper: b"\xe1\xbf\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FE2,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\xa2",
            upper: b"\xce\xa5\xcc\x88\xcc\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FE3,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\xa3",
            upper: b"\xce\xa5\xcc\x88\xcc\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FE4,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\xa4",
            upper: b"\xce\xa1\xcc\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FE5,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\xa5",
            upper: b"\xe1\xbf\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FE6,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\xa6",
            upper: b"\xce\xa5\xcd\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FE7,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\xa7",
            upper: b"\xce\xa5\xcc\x88\xcd\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FE8,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\xa0",
            upper: b"\xe1\xbf\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FE9,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\xa1",
            upper: b"\xe1\xbf\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FEA,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xba",
            upper: b"\xe1\xbf\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FEB,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xbb",
            upper: b"\xe1\xbf\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FEC,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\xa5",
            upper: b"\xe1\xbf\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FF2,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\xb2",
            upper: b"\xe1\xbf\xba\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FF3,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\xb3",
            upper: b"\xce\xa9\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FF4,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\xb4",
            upper: b"\xce\x8f\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FF6,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\xb6",
            upper: b"\xce\xa9\xcd\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FF7,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\xb7",
            upper: b"\xce\xa9\xcd\x82\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FF8,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xb8",
            upper: b"\xe1\xbf\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FF9,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xb9",
            upper: b"\xe1\xbf\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FFA,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xbc",
            upper: b"\xe1\xbf\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FFB,
        SpecialCasingMapping {
            lower: b"\xe1\xbd\xbd",
            upper: b"\xe1\xbf\xbb",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1FFC,
        SpecialCasingMapping {
            lower: b"\xe1\xbf\xb3",
            upper: b"\xce\xa9\xce\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2126,
        SpecialCasingMapping {
            lower: b"\xcf\x89",
            upper: b"\xe2\x84\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x212A,
        SpecialCasingMapping {
            lower: b"k",
            upper: b"\xe2\x84\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x212B,
        SpecialCasingMapping {
            lower: b"\xc3\xa5",
            upper: b"\xe2\x84\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2132,
        SpecialCasingMapping {
            lower: b"\xe2\x85\x8e",
            upper: b"\xe2\x84\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x214E,
        SpecialCasingMapping {
            lower: b"\xe2\x85\x8e",
            upper: b"\xe2\x84\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2160,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xb0",
            upper: b"\xe2\x85\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2161,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xb1",
            upper: b"\xe2\x85\xa1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2162,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xb2",
            upper: b"\xe2\x85\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2163,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xb3",
            upper: b"\xe2\x85\xa3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2164,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xb4",
            upper: b"\xe2\x85\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2165,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xb5",
            upper: b"\xe2\x85\xa5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2166,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xb6",
            upper: b"\xe2\x85\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2167,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xb7",
            upper: b"\xe2\x85\xa7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2168,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xb8",
            upper: b"\xe2\x85\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2169,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xb9",
            upper: b"\xe2\x85\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x216A,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xba",
            upper: b"\xe2\x85\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x216B,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xbb",
            upper: b"\xe2\x85\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x216C,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xbc",
            upper: b"\xe2\x85\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x216D,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xbd",
            upper: b"\xe2\x85\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x216E,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xbe",
            upper: b"\xe2\x85\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x216F,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xbf",
            upper: b"\xe2\x85\xaf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2170,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xb0",
            upper: b"\xe2\x85\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2171,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xb1",
            upper: b"\xe2\x85\xa1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2172,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xb2",
            upper: b"\xe2\x85\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2173,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xb3",
            upper: b"\xe2\x85\xa3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2174,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xb4",
            upper: b"\xe2\x85\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2175,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xb5",
            upper: b"\xe2\x85\xa5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2176,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xb6",
            upper: b"\xe2\x85\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2177,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xb7",
            upper: b"\xe2\x85\xa7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2178,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xb8",
            upper: b"\xe2\x85\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2179,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xb9",
            upper: b"\xe2\x85\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x217A,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xba",
            upper: b"\xe2\x85\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x217B,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xbb",
            upper: b"\xe2\x85\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x217C,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xbc",
            upper: b"\xe2\x85\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x217D,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xbd",
            upper: b"\xe2\x85\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x217E,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xbe",
            upper: b"\xe2\x85\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x217F,
        SpecialCasingMapping {
            lower: b"\xe2\x85\xbf",
            upper: b"\xe2\x85\xaf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2183,
        SpecialCasingMapping {
            lower: b"\xe2\x86\x84",
            upper: b"\xe2\x86\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2184,
        SpecialCasingMapping {
            lower: b"\xe2\x86\x84",
            upper: b"\xe2\x86\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24B6,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x90",
            upper: b"\xe2\x92\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24B7,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x91",
            upper: b"\xe2\x92\xb7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24B8,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x92",
            upper: b"\xe2\x92\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24B9,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x93",
            upper: b"\xe2\x92\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24BA,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x94",
            upper: b"\xe2\x92\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24BB,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x95",
            upper: b"\xe2\x92\xbb",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24BC,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x96",
            upper: b"\xe2\x92\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24BD,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x97",
            upper: b"\xe2\x92\xbd",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24BE,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x98",
            upper: b"\xe2\x92\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24BF,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x99",
            upper: b"\xe2\x92\xbf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24C0,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x9a",
            upper: b"\xe2\x93\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24C1,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x9b",
            upper: b"\xe2\x93\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24C2,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x9c",
            upper: b"\xe2\x93\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24C3,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x9d",
            upper: b"\xe2\x93\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24C4,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x9e",
            upper: b"\xe2\x93\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24C5,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x9f",
            upper: b"\xe2\x93\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24C6,
        SpecialCasingMapping {
            lower: b"\xe2\x93\xa0",
            upper: b"\xe2\x93\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24C7,
        SpecialCasingMapping {
            lower: b"\xe2\x93\xa1",
            upper: b"\xe2\x93\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24C8,
        SpecialCasingMapping {
            lower: b"\xe2\x93\xa2",
            upper: b"\xe2\x93\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24C9,
        SpecialCasingMapping {
            lower: b"\xe2\x93\xa3",
            upper: b"\xe2\x93\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24CA,
        SpecialCasingMapping {
            lower: b"\xe2\x93\xa4",
            upper: b"\xe2\x93\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24CB,
        SpecialCasingMapping {
            lower: b"\xe2\x93\xa5",
            upper: b"\xe2\x93\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24CC,
        SpecialCasingMapping {
            lower: b"\xe2\x93\xa6",
            upper: b"\xe2\x93\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24CD,
        SpecialCasingMapping {
            lower: b"\xe2\x93\xa7",
            upper: b"\xe2\x93\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24CE,
        SpecialCasingMapping {
            lower: b"\xe2\x93\xa8",
            upper: b"\xe2\x93\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24CF,
        SpecialCasingMapping {
            lower: b"\xe2\x93\xa9",
            upper: b"\xe2\x93\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24D0,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x90",
            upper: b"\xe2\x92\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24D1,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x91",
            upper: b"\xe2\x92\xb7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24D2,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x92",
            upper: b"\xe2\x92\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24D3,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x93",
            upper: b"\xe2\x92\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24D4,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x94",
            upper: b"\xe2\x92\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24D5,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x95",
            upper: b"\xe2\x92\xbb",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24D6,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x96",
            upper: b"\xe2\x92\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24D7,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x97",
            upper: b"\xe2\x92\xbd",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24D8,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x98",
            upper: b"\xe2\x92\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24D9,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x99",
            upper: b"\xe2\x92\xbf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24DA,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x9a",
            upper: b"\xe2\x93\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24DB,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x9b",
            upper: b"\xe2\x93\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24DC,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x9c",
            upper: b"\xe2\x93\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24DD,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x9d",
            upper: b"\xe2\x93\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24DE,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x9e",
            upper: b"\xe2\x93\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24DF,
        SpecialCasingMapping {
            lower: b"\xe2\x93\x9f",
            upper: b"\xe2\x93\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24E0,
        SpecialCasingMapping {
            lower: b"\xe2\x93\xa0",
            upper: b"\xe2\x93\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24E1,
        SpecialCasingMapping {
            lower: b"\xe2\x93\xa1",
            upper: b"\xe2\x93\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24E2,
        SpecialCasingMapping {
            lower: b"\xe2\x93\xa2",
            upper: b"\xe2\x93\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24E3,
        SpecialCasingMapping {
            lower: b"\xe2\x93\xa3",
            upper: b"\xe2\x93\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24E4,
        SpecialCasingMapping {
            lower: b"\xe2\x93\xa4",
            upper: b"\xe2\x93\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24E5,
        SpecialCasingMapping {
            lower: b"\xe2\x93\xa5",
            upper: b"\xe2\x93\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24E6,
        SpecialCasingMapping {
            lower: b"\xe2\x93\xa6",
            upper: b"\xe2\x93\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24E7,
        SpecialCasingMapping {
            lower: b"\xe2\x93\xa7",
            upper: b"\xe2\x93\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24E8,
        SpecialCasingMapping {
            lower: b"\xe2\x93\xa8",
            upper: b"\xe2\x93\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x24E9,
        SpecialCasingMapping {
            lower: b"\xe2\x93\xa9",
            upper: b"\xe2\x93\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C00,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xb0",
            upper: b"\xe2\xb0\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C01,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xb1",
            upper: b"\xe2\xb0\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C02,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xb2",
            upper: b"\xe2\xb0\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C03,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xb3",
            upper: b"\xe2\xb0\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C04,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xb4",
            upper: b"\xe2\xb0\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C05,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xb5",
            upper: b"\xe2\xb0\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C06,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xb6",
            upper: b"\xe2\xb0\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C07,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xb7",
            upper: b"\xe2\xb0\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C08,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xb8",
            upper: b"\xe2\xb0\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C09,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xb9",
            upper: b"\xe2\xb0\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C0A,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xba",
            upper: b"\xe2\xb0\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C0B,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xbb",
            upper: b"\xe2\xb0\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C0C,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xbc",
            upper: b"\xe2\xb0\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C0D,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xbd",
            upper: b"\xe2\xb0\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C0E,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xbe",
            upper: b"\xe2\xb0\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C0F,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xbf",
            upper: b"\xe2\xb0\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C10,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x80",
            upper: b"\xe2\xb0\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C11,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x81",
            upper: b"\xe2\xb0\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C12,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x82",
            upper: b"\xe2\xb0\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C13,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x83",
            upper: b"\xe2\xb0\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C14,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x84",
            upper: b"\xe2\xb0\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C15,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x85",
            upper: b"\xe2\xb0\x95",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C16,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x86",
            upper: b"\xe2\xb0\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C17,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x87",
            upper: b"\xe2\xb0\x97",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C18,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x88",
            upper: b"\xe2\xb0\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C19,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x89",
            upper: b"\xe2\xb0\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C1A,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x8a",
            upper: b"\xe2\xb0\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C1B,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x8b",
            upper: b"\xe2\xb0\x9b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C1C,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x8c",
            upper: b"\xe2\xb0\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C1D,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x8d",
            upper: b"\xe2\xb0\x9d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C1E,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x8e",
            upper: b"\xe2\xb0\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C1F,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x8f",
            upper: b"\xe2\xb0\x9f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C20,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x90",
            upper: b"\xe2\xb0\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C21,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x91",
            upper: b"\xe2\xb0\xa1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C22,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x92",
            upper: b"\xe2\xb0\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C23,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x93",
            upper: b"\xe2\xb0\xa3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C24,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x94",
            upper: b"\xe2\xb0\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C25,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x95",
            upper: b"\xe2\xb0\xa5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C26,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x96",
            upper: b"\xe2\xb0\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C27,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x97",
            upper: b"\xe2\xb0\xa7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C28,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x98",
            upper: b"\xe2\xb0\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C29,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x99",
            upper: b"\xe2\xb0\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C2A,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x9a",
            upper: b"\xe2\xb0\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C2B,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x9b",
            upper: b"\xe2\xb0\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C2C,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x9c",
            upper: b"\xe2\xb0\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C2D,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x9d",
            upper: b"\xe2\xb0\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C2E,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x9e",
            upper: b"\xe2\xb0\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C2F,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x9f",
            upper: b"\xe2\xb0\xaf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C30,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xb0",
            upper: b"\xe2\xb0\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C31,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xb1",
            upper: b"\xe2\xb0\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C32,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xb2",
            upper: b"\xe2\xb0\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C33,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xb3",
            upper: b"\xe2\xb0\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C34,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xb4",
            upper: b"\xe2\xb0\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C35,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xb5",
            upper: b"\xe2\xb0\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C36,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xb6",
            upper: b"\xe2\xb0\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C37,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xb7",
            upper: b"\xe2\xb0\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C38,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xb8",
            upper: b"\xe2\xb0\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C39,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xb9",
            upper: b"\xe2\xb0\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C3A,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xba",
            upper: b"\xe2\xb0\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C3B,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xbb",
            upper: b"\xe2\xb0\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C3C,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xbc",
            upper: b"\xe2\xb0\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C3D,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xbd",
            upper: b"\xe2\xb0\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C3E,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xbe",
            upper: b"\xe2\xb0\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C3F,
        SpecialCasingMapping {
            lower: b"\xe2\xb0\xbf",
            upper: b"\xe2\xb0\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C40,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x80",
            upper: b"\xe2\xb0\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C41,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x81",
            upper: b"\xe2\xb0\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C42,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x82",
            upper: b"\xe2\xb0\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C43,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x83",
            upper: b"\xe2\xb0\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C44,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x84",
            upper: b"\xe2\xb0\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C45,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x85",
            upper: b"\xe2\xb0\x95",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C46,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x86",
            upper: b"\xe2\xb0\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C47,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x87",
            upper: b"\xe2\xb0\x97",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C48,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x88",
            upper: b"\xe2\xb0\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C49,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x89",
            upper: b"\xe2\xb0\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C4A,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x8a",
            upper: b"\xe2\xb0\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C4B,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x8b",
            upper: b"\xe2\xb0\x9b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C4C,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x8c",
            upper: b"\xe2\xb0\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C4D,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x8d",
            upper: b"\xe2\xb0\x9d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C4E,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x8e",
            upper: b"\xe2\xb0\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C4F,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x8f",
            upper: b"\xe2\xb0\x9f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C50,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x90",
            upper: b"\xe2\xb0\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C51,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x91",
            upper: b"\xe2\xb0\xa1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C52,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x92",
            upper: b"\xe2\xb0\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C53,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x93",
            upper: b"\xe2\xb0\xa3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C54,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x94",
            upper: b"\xe2\xb0\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C55,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x95",
            upper: b"\xe2\xb0\xa5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C56,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x96",
            upper: b"\xe2\xb0\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C57,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x97",
            upper: b"\xe2\xb0\xa7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C58,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x98",
            upper: b"\xe2\xb0\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C59,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x99",
            upper: b"\xe2\xb0\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C5A,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x9a",
            upper: b"\xe2\xb0\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C5B,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x9b",
            upper: b"\xe2\xb0\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C5C,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x9c",
            upper: b"\xe2\xb0\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C5D,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x9d",
            upper: b"\xe2\xb0\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C5E,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x9e",
            upper: b"\xe2\xb0\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C5F,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\x9f",
            upper: b"\xe2\xb0\xaf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C60,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\xa1",
            upper: b"\xe2\xb1\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C61,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\xa1",
            upper: b"\xe2\xb1\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C62,
        SpecialCasingMapping {
            lower: b"\xc9\xab",
            upper: b"\xe2\xb1\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C63,
        SpecialCasingMapping {
            lower: b"\xe1\xb5\xbd",
            upper: b"\xe2\xb1\xa3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C64,
        SpecialCasingMapping {
            lower: b"\xc9\xbd",
            upper: b"\xe2\xb1\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C65,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\xa5",
            upper: b"\xc8\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C66,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\xa6",
            upper: b"\xc8\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C67,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\xa8",
            upper: b"\xe2\xb1\xa7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C68,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\xa8",
            upper: b"\xe2\xb1\xa7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C69,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\xaa",
            upper: b"\xe2\xb1\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C6A,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\xaa",
            upper: b"\xe2\xb1\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C6B,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\xac",
            upper: b"\xe2\xb1\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C6C,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\xac",
            upper: b"\xe2\xb1\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C6D,
        SpecialCasingMapping {
            lower: b"\xc9\x91",
            upper: b"\xe2\xb1\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C6E,
        SpecialCasingMapping {
            lower: b"\xc9\xb1",
            upper: b"\xe2\xb1\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C6F,
        SpecialCasingMapping {
            lower: b"\xc9\x90",
            upper: b"\xe2\xb1\xaf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C70,
        SpecialCasingMapping {
            lower: b"\xc9\x92",
            upper: b"\xe2\xb1\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C72,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\xb3",
            upper: b"\xe2\xb1\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C73,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\xb3",
            upper: b"\xe2\xb1\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C75,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\xb6",
            upper: b"\xe2\xb1\xb5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C76,
        SpecialCasingMapping {
            lower: b"\xe2\xb1\xb6",
            upper: b"\xe2\xb1\xb5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C7E,
        SpecialCasingMapping {
            lower: b"\xc8\xbf",
            upper: b"\xe2\xb1\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C7F,
        SpecialCasingMapping {
            lower: b"\xc9\x80",
            upper: b"\xe2\xb1\xbf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C80,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x81",
            upper: b"\xe2\xb2\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C81,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x81",
            upper: b"\xe2\xb2\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C82,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x83",
            upper: b"\xe2\xb2\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C83,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x83",
            upper: b"\xe2\xb2\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C84,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x85",
            upper: b"\xe2\xb2\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C85,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x85",
            upper: b"\xe2\xb2\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C86,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x87",
            upper: b"\xe2\xb2\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C87,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x87",
            upper: b"\xe2\xb2\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C88,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x89",
            upper: b"\xe2\xb2\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C89,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x89",
            upper: b"\xe2\xb2\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C8A,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x8b",
            upper: b"\xe2\xb2\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C8B,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x8b",
            upper: b"\xe2\xb2\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C8C,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x8d",
            upper: b"\xe2\xb2\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C8D,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x8d",
            upper: b"\xe2\xb2\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C8E,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x8f",
            upper: b"\xe2\xb2\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C8F,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x8f",
            upper: b"\xe2\xb2\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C90,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x91",
            upper: b"\xe2\xb2\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C91,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x91",
            upper: b"\xe2\xb2\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C92,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x93",
            upper: b"\xe2\xb2\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C93,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x93",
            upper: b"\xe2\xb2\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C94,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x95",
            upper: b"\xe2\xb2\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C95,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x95",
            upper: b"\xe2\xb2\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C96,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x97",
            upper: b"\xe2\xb2\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C97,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x97",
            upper: b"\xe2\xb2\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C98,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x99",
            upper: b"\xe2\xb2\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C99,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x99",
            upper: b"\xe2\xb2\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C9A,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x9b",
            upper: b"\xe2\xb2\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C9B,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x9b",
            upper: b"\xe2\xb2\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C9C,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x9d",
            upper: b"\xe2\xb2\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C9D,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x9d",
            upper: b"\xe2\xb2\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C9E,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x9f",
            upper: b"\xe2\xb2\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2C9F,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\x9f",
            upper: b"\xe2\xb2\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CA0,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xa1",
            upper: b"\xe2\xb2\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CA1,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xa1",
            upper: b"\xe2\xb2\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CA2,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xa3",
            upper: b"\xe2\xb2\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CA3,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xa3",
            upper: b"\xe2\xb2\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CA4,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xa5",
            upper: b"\xe2\xb2\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CA5,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xa5",
            upper: b"\xe2\xb2\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CA6,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xa7",
            upper: b"\xe2\xb2\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CA7,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xa7",
            upper: b"\xe2\xb2\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CA8,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xa9",
            upper: b"\xe2\xb2\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CA9,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xa9",
            upper: b"\xe2\xb2\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CAA,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xab",
            upper: b"\xe2\xb2\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CAB,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xab",
            upper: b"\xe2\xb2\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CAC,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xad",
            upper: b"\xe2\xb2\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CAD,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xad",
            upper: b"\xe2\xb2\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CAE,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xaf",
            upper: b"\xe2\xb2\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CAF,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xaf",
            upper: b"\xe2\xb2\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CB0,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xb1",
            upper: b"\xe2\xb2\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CB1,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xb1",
            upper: b"\xe2\xb2\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CB2,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xb3",
            upper: b"\xe2\xb2\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CB3,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xb3",
            upper: b"\xe2\xb2\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CB4,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xb5",
            upper: b"\xe2\xb2\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CB5,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xb5",
            upper: b"\xe2\xb2\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CB6,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xb7",
            upper: b"\xe2\xb2\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CB7,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xb7",
            upper: b"\xe2\xb2\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CB8,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xb9",
            upper: b"\xe2\xb2\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CB9,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xb9",
            upper: b"\xe2\xb2\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CBA,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xbb",
            upper: b"\xe2\xb2\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CBB,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xbb",
            upper: b"\xe2\xb2\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CBC,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xbd",
            upper: b"\xe2\xb2\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CBD,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xbd",
            upper: b"\xe2\xb2\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CBE,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xbf",
            upper: b"\xe2\xb2\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CBF,
        SpecialCasingMapping {
            lower: b"\xe2\xb2\xbf",
            upper: b"\xe2\xb2\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CC0,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x81",
            upper: b"\xe2\xb3\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CC1,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x81",
            upper: b"\xe2\xb3\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CC2,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x83",
            upper: b"\xe2\xb3\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CC3,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x83",
            upper: b"\xe2\xb3\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CC4,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x85",
            upper: b"\xe2\xb3\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CC5,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x85",
            upper: b"\xe2\xb3\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CC6,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x87",
            upper: b"\xe2\xb3\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CC7,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x87",
            upper: b"\xe2\xb3\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CC8,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x89",
            upper: b"\xe2\xb3\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CC9,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x89",
            upper: b"\xe2\xb3\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CCA,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x8b",
            upper: b"\xe2\xb3\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CCB,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x8b",
            upper: b"\xe2\xb3\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CCC,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x8d",
            upper: b"\xe2\xb3\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CCD,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x8d",
            upper: b"\xe2\xb3\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CCE,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x8f",
            upper: b"\xe2\xb3\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CCF,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x8f",
            upper: b"\xe2\xb3\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CD0,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x91",
            upper: b"\xe2\xb3\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CD1,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x91",
            upper: b"\xe2\xb3\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CD2,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x93",
            upper: b"\xe2\xb3\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CD3,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x93",
            upper: b"\xe2\xb3\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CD4,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x95",
            upper: b"\xe2\xb3\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CD5,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x95",
            upper: b"\xe2\xb3\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CD6,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x97",
            upper: b"\xe2\xb3\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CD7,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x97",
            upper: b"\xe2\xb3\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CD8,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x99",
            upper: b"\xe2\xb3\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CD9,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x99",
            upper: b"\xe2\xb3\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CDA,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x9b",
            upper: b"\xe2\xb3\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CDB,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x9b",
            upper: b"\xe2\xb3\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CDC,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x9d",
            upper: b"\xe2\xb3\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CDD,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x9d",
            upper: b"\xe2\xb3\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CDE,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x9f",
            upper: b"\xe2\xb3\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CDF,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\x9f",
            upper: b"\xe2\xb3\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CE0,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\xa1",
            upper: b"\xe2\xb3\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CE1,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\xa1",
            upper: b"\xe2\xb3\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CE2,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\xa3",
            upper: b"\xe2\xb3\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CE3,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\xa3",
            upper: b"\xe2\xb3\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CEB,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\xac",
            upper: b"\xe2\xb3\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CEC,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\xac",
            upper: b"\xe2\xb3\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CED,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\xae",
            upper: b"\xe2\xb3\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CEE,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\xae",
            upper: b"\xe2\xb3\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CF2,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\xb3",
            upper: b"\xe2\xb3\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2CF3,
        SpecialCasingMapping {
            lower: b"\xe2\xb3\xb3",
            upper: b"\xe2\xb3\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D00,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x80",
            upper: b"\xe1\x82\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D01,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x81",
            upper: b"\xe1\x82\xa1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D02,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x82",
            upper: b"\xe1\x82\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D03,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x83",
            upper: b"\xe1\x82\xa3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D04,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x84",
            upper: b"\xe1\x82\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D05,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x85",
            upper: b"\xe1\x82\xa5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D06,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x86",
            upper: b"\xe1\x82\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D07,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x87",
            upper: b"\xe1\x82\xa7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D08,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x88",
            upper: b"\xe1\x82\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D09,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x89",
            upper: b"\xe1\x82\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D0A,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x8a",
            upper: b"\xe1\x82\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D0B,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x8b",
            upper: b"\xe1\x82\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D0C,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x8c",
            upper: b"\xe1\x82\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D0D,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x8d",
            upper: b"\xe1\x82\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D0E,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x8e",
            upper: b"\xe1\x82\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D0F,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x8f",
            upper: b"\xe1\x82\xaf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D10,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x90",
            upper: b"\xe1\x82\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D11,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x91",
            upper: b"\xe1\x82\xb1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D12,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x92",
            upper: b"\xe1\x82\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D13,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x93",
            upper: b"\xe1\x82\xb3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D14,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x94",
            upper: b"\xe1\x82\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D15,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x95",
            upper: b"\xe1\x82\xb5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D16,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x96",
            upper: b"\xe1\x82\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D17,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x97",
            upper: b"\xe1\x82\xb7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D18,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x98",
            upper: b"\xe1\x82\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D19,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x99",
            upper: b"\xe1\x82\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D1A,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x9a",
            upper: b"\xe1\x82\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D1B,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x9b",
            upper: b"\xe1\x82\xbb",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D1C,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x9c",
            upper: b"\xe1\x82\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D1D,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x9d",
            upper: b"\xe1\x82\xbd",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D1E,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x9e",
            upper: b"\xe1\x82\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D1F,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\x9f",
            upper: b"\xe1\x82\xbf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D20,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\xa0",
            upper: b"\xe1\x83\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D21,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\xa1",
            upper: b"\xe1\x83\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D22,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\xa2",
            upper: b"\xe1\x83\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D23,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\xa3",
            upper: b"\xe1\x83\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D24,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\xa4",
            upper: b"\xe1\x83\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D25,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\xa5",
            upper: b"\xe1\x83\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D27,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\xa7",
            upper: b"\xe1\x83\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x2D2D,
        SpecialCasingMapping {
            lower: b"\xe2\xb4\xad",
            upper: b"\xe1\x83\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA640,
        SpecialCasingMapping {
            lower: b"\xea\x99\x81",
            upper: b"\xea\x99\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA641,
        SpecialCasingMapping {
            lower: b"\xea\x99\x81",
            upper: b"\xea\x99\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA642,
        SpecialCasingMapping {
            lower: b"\xea\x99\x83",
            upper: b"\xea\x99\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA643,
        SpecialCasingMapping {
            lower: b"\xea\x99\x83",
            upper: b"\xea\x99\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA644,
        SpecialCasingMapping {
            lower: b"\xea\x99\x85",
            upper: b"\xea\x99\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA645,
        SpecialCasingMapping {
            lower: b"\xea\x99\x85",
            upper: b"\xea\x99\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA646,
        SpecialCasingMapping {
            lower: b"\xea\x99\x87",
            upper: b"\xea\x99\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA647,
        SpecialCasingMapping {
            lower: b"\xea\x99\x87",
            upper: b"\xea\x99\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA648,
        SpecialCasingMapping {
            lower: b"\xea\x99\x89",
            upper: b"\xea\x99\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA649,
        SpecialCasingMapping {
            lower: b"\xea\x99\x89",
            upper: b"\xea\x99\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA64A,
        SpecialCasingMapping {
            lower: b"\xea\x99\x8b",
            upper: b"\xea\x99\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA64B,
        SpecialCasingMapping {
            lower: b"\xea\x99\x8b",
            upper: b"\xea\x99\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA64C,
        SpecialCasingMapping {
            lower: b"\xea\x99\x8d",
            upper: b"\xea\x99\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA64D,
        SpecialCasingMapping {
            lower: b"\xea\x99\x8d",
            upper: b"\xea\x99\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA64E,
        SpecialCasingMapping {
            lower: b"\xea\x99\x8f",
            upper: b"\xea\x99\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA64F,
        SpecialCasingMapping {
            lower: b"\xea\x99\x8f",
            upper: b"\xea\x99\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA650,
        SpecialCasingMapping {
            lower: b"\xea\x99\x91",
            upper: b"\xea\x99\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA651,
        SpecialCasingMapping {
            lower: b"\xea\x99\x91",
            upper: b"\xea\x99\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA652,
        SpecialCasingMapping {
            lower: b"\xea\x99\x93",
            upper: b"\xea\x99\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA653,
        SpecialCasingMapping {
            lower: b"\xea\x99\x93",
            upper: b"\xea\x99\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA654,
        SpecialCasingMapping {
            lower: b"\xea\x99\x95",
            upper: b"\xea\x99\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA655,
        SpecialCasingMapping {
            lower: b"\xea\x99\x95",
            upper: b"\xea\x99\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA656,
        SpecialCasingMapping {
            lower: b"\xea\x99\x97",
            upper: b"\xea\x99\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA657,
        SpecialCasingMapping {
            lower: b"\xea\x99\x97",
            upper: b"\xea\x99\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA658,
        SpecialCasingMapping {
            lower: b"\xea\x99\x99",
            upper: b"\xea\x99\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA659,
        SpecialCasingMapping {
            lower: b"\xea\x99\x99",
            upper: b"\xea\x99\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA65A,
        SpecialCasingMapping {
            lower: b"\xea\x99\x9b",
            upper: b"\xea\x99\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA65B,
        SpecialCasingMapping {
            lower: b"\xea\x99\x9b",
            upper: b"\xea\x99\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA65C,
        SpecialCasingMapping {
            lower: b"\xea\x99\x9d",
            upper: b"\xea\x99\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA65D,
        SpecialCasingMapping {
            lower: b"\xea\x99\x9d",
            upper: b"\xea\x99\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA65E,
        SpecialCasingMapping {
            lower: b"\xea\x99\x9f",
            upper: b"\xea\x99\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA65F,
        SpecialCasingMapping {
            lower: b"\xea\x99\x9f",
            upper: b"\xea\x99\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA660,
        SpecialCasingMapping {
            lower: b"\xea\x99\xa1",
            upper: b"\xea\x99\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA661,
        SpecialCasingMapping {
            lower: b"\xea\x99\xa1",
            upper: b"\xea\x99\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA662,
        SpecialCasingMapping {
            lower: b"\xea\x99\xa3",
            upper: b"\xea\x99\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA663,
        SpecialCasingMapping {
            lower: b"\xea\x99\xa3",
            upper: b"\xea\x99\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA664,
        SpecialCasingMapping {
            lower: b"\xea\x99\xa5",
            upper: b"\xea\x99\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA665,
        SpecialCasingMapping {
            lower: b"\xea\x99\xa5",
            upper: b"\xea\x99\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA666,
        SpecialCasingMapping {
            lower: b"\xea\x99\xa7",
            upper: b"\xea\x99\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA667,
        SpecialCasingMapping {
            lower: b"\xea\x99\xa7",
            upper: b"\xea\x99\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA668,
        SpecialCasingMapping {
            lower: b"\xea\x99\xa9",
            upper: b"\xea\x99\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA669,
        SpecialCasingMapping {
            lower: b"\xea\x99\xa9",
            upper: b"\xea\x99\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA66A,
        SpecialCasingMapping {
            lower: b"\xea\x99\xab",
            upper: b"\xea\x99\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA66B,
        SpecialCasingMapping {
            lower: b"\xea\x99\xab",
            upper: b"\xea\x99\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA66C,
        SpecialCasingMapping {
            lower: b"\xea\x99\xad",
            upper: b"\xea\x99\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA66D,
        SpecialCasingMapping {
            lower: b"\xea\x99\xad",
            upper: b"\xea\x99\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA680,
        SpecialCasingMapping {
            lower: b"\xea\x9a\x81",
            upper: b"\xea\x9a\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA681,
        SpecialCasingMapping {
            lower: b"\xea\x9a\x81",
            upper: b"\xea\x9a\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA682,
        SpecialCasingMapping {
            lower: b"\xea\x9a\x83",
            upper: b"\xea\x9a\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA683,
        SpecialCasingMapping {
            lower: b"\xea\x9a\x83",
            upper: b"\xea\x9a\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA684,
        SpecialCasingMapping {
            lower: b"\xea\x9a\x85",
            upper: b"\xea\x9a\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA685,
        SpecialCasingMapping {
            lower: b"\xea\x9a\x85",
            upper: b"\xea\x9a\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA686,
        SpecialCasingMapping {
            lower: b"\xea\x9a\x87",
            upper: b"\xea\x9a\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA687,
        SpecialCasingMapping {
            lower: b"\xea\x9a\x87",
            upper: b"\xea\x9a\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA688,
        SpecialCasingMapping {
            lower: b"\xea\x9a\x89",
            upper: b"\xea\x9a\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA689,
        SpecialCasingMapping {
            lower: b"\xea\x9a\x89",
            upper: b"\xea\x9a\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA68A,
        SpecialCasingMapping {
            lower: b"\xea\x9a\x8b",
            upper: b"\xea\x9a\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA68B,
        SpecialCasingMapping {
            lower: b"\xea\x9a\x8b",
            upper: b"\xea\x9a\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA68C,
        SpecialCasingMapping {
            lower: b"\xea\x9a\x8d",
            upper: b"\xea\x9a\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA68D,
        SpecialCasingMapping {
            lower: b"\xea\x9a\x8d",
            upper: b"\xea\x9a\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA68E,
        SpecialCasingMapping {
            lower: b"\xea\x9a\x8f",
            upper: b"\xea\x9a\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA68F,
        SpecialCasingMapping {
            lower: b"\xea\x9a\x8f",
            upper: b"\xea\x9a\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA690,
        SpecialCasingMapping {
            lower: b"\xea\x9a\x91",
            upper: b"\xea\x9a\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA691,
        SpecialCasingMapping {
            lower: b"\xea\x9a\x91",
            upper: b"\xea\x9a\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA692,
        SpecialCasingMapping {
            lower: b"\xea\x9a\x93",
            upper: b"\xea\x9a\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA693,
        SpecialCasingMapping {
            lower: b"\xea\x9a\x93",
            upper: b"\xea\x9a\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA694,
        SpecialCasingMapping {
            lower: b"\xea\x9a\x95",
            upper: b"\xea\x9a\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA695,
        SpecialCasingMapping {
            lower: b"\xea\x9a\x95",
            upper: b"\xea\x9a\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA696,
        SpecialCasingMapping {
            lower: b"\xea\x9a\x97",
            upper: b"\xea\x9a\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA697,
        SpecialCasingMapping {
            lower: b"\xea\x9a\x97",
            upper: b"\xea\x9a\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA698,
        SpecialCasingMapping {
            lower: b"\xea\x9a\x99",
            upper: b"\xea\x9a\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA699,
        SpecialCasingMapping {
            lower: b"\xea\x9a\x99",
            upper: b"\xea\x9a\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA69A,
        SpecialCasingMapping {
            lower: b"\xea\x9a\x9b",
            upper: b"\xea\x9a\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA69B,
        SpecialCasingMapping {
            lower: b"\xea\x9a\x9b",
            upper: b"\xea\x9a\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA722,
        SpecialCasingMapping {
            lower: b"\xea\x9c\xa3",
            upper: b"\xea\x9c\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA723,
        SpecialCasingMapping {
            lower: b"\xea\x9c\xa3",
            upper: b"\xea\x9c\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA724,
        SpecialCasingMapping {
            lower: b"\xea\x9c\xa5",
            upper: b"\xea\x9c\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA725,
        SpecialCasingMapping {
            lower: b"\xea\x9c\xa5",
            upper: b"\xea\x9c\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA726,
        SpecialCasingMapping {
            lower: b"\xea\x9c\xa7",
            upper: b"\xea\x9c\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA727,
        SpecialCasingMapping {
            lower: b"\xea\x9c\xa7",
            upper: b"\xea\x9c\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA728,
        SpecialCasingMapping {
            lower: b"\xea\x9c\xa9",
            upper: b"\xea\x9c\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA729,
        SpecialCasingMapping {
            lower: b"\xea\x9c\xa9",
            upper: b"\xea\x9c\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA72A,
        SpecialCasingMapping {
            lower: b"\xea\x9c\xab",
            upper: b"\xea\x9c\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA72B,
        SpecialCasingMapping {
            lower: b"\xea\x9c\xab",
            upper: b"\xea\x9c\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA72C,
        SpecialCasingMapping {
            lower: b"\xea\x9c\xad",
            upper: b"\xea\x9c\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA72D,
        SpecialCasingMapping {
            lower: b"\xea\x9c\xad",
            upper: b"\xea\x9c\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA72E,
        SpecialCasingMapping {
            lower: b"\xea\x9c\xaf",
            upper: b"\xea\x9c\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA72F,
        SpecialCasingMapping {
            lower: b"\xea\x9c\xaf",
            upper: b"\xea\x9c\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA732,
        SpecialCasingMapping {
            lower: b"\xea\x9c\xb3",
            upper: b"\xea\x9c\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA733,
        SpecialCasingMapping {
            lower: b"\xea\x9c\xb3",
            upper: b"\xea\x9c\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA734,
        SpecialCasingMapping {
            lower: b"\xea\x9c\xb5",
            upper: b"\xea\x9c\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA735,
        SpecialCasingMapping {
            lower: b"\xea\x9c\xb5",
            upper: b"\xea\x9c\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA736,
        SpecialCasingMapping {
            lower: b"\xea\x9c\xb7",
            upper: b"\xea\x9c\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA737,
        SpecialCasingMapping {
            lower: b"\xea\x9c\xb7",
            upper: b"\xea\x9c\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA738,
        SpecialCasingMapping {
            lower: b"\xea\x9c\xb9",
            upper: b"\xea\x9c\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA739,
        SpecialCasingMapping {
            lower: b"\xea\x9c\xb9",
            upper: b"\xea\x9c\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA73A,
        SpecialCasingMapping {
            lower: b"\xea\x9c\xbb",
            upper: b"\xea\x9c\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA73B,
        SpecialCasingMapping {
            lower: b"\xea\x9c\xbb",
            upper: b"\xea\x9c\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA73C,
        SpecialCasingMapping {
            lower: b"\xea\x9c\xbd",
            upper: b"\xea\x9c\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA73D,
        SpecialCasingMapping {
            lower: b"\xea\x9c\xbd",
            upper: b"\xea\x9c\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA73E,
        SpecialCasingMapping {
            lower: b"\xea\x9c\xbf",
            upper: b"\xea\x9c\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA73F,
        SpecialCasingMapping {
            lower: b"\xea\x9c\xbf",
            upper: b"\xea\x9c\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA740,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x81",
            upper: b"\xea\x9d\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA741,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x81",
            upper: b"\xea\x9d\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA742,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x83",
            upper: b"\xea\x9d\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA743,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x83",
            upper: b"\xea\x9d\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA744,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x85",
            upper: b"\xea\x9d\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA745,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x85",
            upper: b"\xea\x9d\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA746,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x87",
            upper: b"\xea\x9d\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA747,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x87",
            upper: b"\xea\x9d\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA748,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x89",
            upper: b"\xea\x9d\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA749,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x89",
            upper: b"\xea\x9d\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA74A,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x8b",
            upper: b"\xea\x9d\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA74B,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x8b",
            upper: b"\xea\x9d\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA74C,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x8d",
            upper: b"\xea\x9d\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA74D,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x8d",
            upper: b"\xea\x9d\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA74E,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x8f",
            upper: b"\xea\x9d\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA74F,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x8f",
            upper: b"\xea\x9d\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA750,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x91",
            upper: b"\xea\x9d\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA751,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x91",
            upper: b"\xea\x9d\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA752,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x93",
            upper: b"\xea\x9d\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA753,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x93",
            upper: b"\xea\x9d\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA754,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x95",
            upper: b"\xea\x9d\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA755,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x95",
            upper: b"\xea\x9d\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA756,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x97",
            upper: b"\xea\x9d\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA757,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x97",
            upper: b"\xea\x9d\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA758,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x99",
            upper: b"\xea\x9d\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA759,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x99",
            upper: b"\xea\x9d\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA75A,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x9b",
            upper: b"\xea\x9d\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA75B,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x9b",
            upper: b"\xea\x9d\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA75C,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x9d",
            upper: b"\xea\x9d\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA75D,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x9d",
            upper: b"\xea\x9d\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA75E,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x9f",
            upper: b"\xea\x9d\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA75F,
        SpecialCasingMapping {
            lower: b"\xea\x9d\x9f",
            upper: b"\xea\x9d\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA760,
        SpecialCasingMapping {
            lower: b"\xea\x9d\xa1",
            upper: b"\xea\x9d\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA761,
        SpecialCasingMapping {
            lower: b"\xea\x9d\xa1",
            upper: b"\xea\x9d\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA762,
        SpecialCasingMapping {
            lower: b"\xea\x9d\xa3",
            upper: b"\xea\x9d\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA763,
        SpecialCasingMapping {
            lower: b"\xea\x9d\xa3",
            upper: b"\xea\x9d\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA764,
        SpecialCasingMapping {
            lower: b"\xea\x9d\xa5",
            upper: b"\xea\x9d\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA765,
        SpecialCasingMapping {
            lower: b"\xea\x9d\xa5",
            upper: b"\xea\x9d\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA766,
        SpecialCasingMapping {
            lower: b"\xea\x9d\xa7",
            upper: b"\xea\x9d\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA767,
        SpecialCasingMapping {
            lower: b"\xea\x9d\xa7",
            upper: b"\xea\x9d\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA768,
        SpecialCasingMapping {
            lower: b"\xea\x9d\xa9",
            upper: b"\xea\x9d\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA769,
        SpecialCasingMapping {
            lower: b"\xea\x9d\xa9",
            upper: b"\xea\x9d\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA76A,
        SpecialCasingMapping {
            lower: b"\xea\x9d\xab",
            upper: b"\xea\x9d\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA76B,
        SpecialCasingMapping {
            lower: b"\xea\x9d\xab",
            upper: b"\xea\x9d\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA76C,
        SpecialCasingMapping {
            lower: b"\xea\x9d\xad",
            upper: b"\xea\x9d\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA76D,
        SpecialCasingMapping {
            lower: b"\xea\x9d\xad",
            upper: b"\xea\x9d\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA76E,
        SpecialCasingMapping {
            lower: b"\xea\x9d\xaf",
            upper: b"\xea\x9d\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA76F,
        SpecialCasingMapping {
            lower: b"\xea\x9d\xaf",
            upper: b"\xea\x9d\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA779,
        SpecialCasingMapping {
            lower: b"\xea\x9d\xba",
            upper: b"\xea\x9d\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA77A,
        SpecialCasingMapping {
            lower: b"\xea\x9d\xba",
            upper: b"\xea\x9d\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA77B,
        SpecialCasingMapping {
            lower: b"\xea\x9d\xbc",
            upper: b"\xea\x9d\xbb",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA77C,
        SpecialCasingMapping {
            lower: b"\xea\x9d\xbc",
            upper: b"\xea\x9d\xbb",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA77D,
        SpecialCasingMapping {
            lower: b"\xe1\xb5\xb9",
            upper: b"\xea\x9d\xbd",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA77E,
        SpecialCasingMapping {
            lower: b"\xea\x9d\xbf",
            upper: b"\xea\x9d\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA77F,
        SpecialCasingMapping {
            lower: b"\xea\x9d\xbf",
            upper: b"\xea\x9d\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA780,
        SpecialCasingMapping {
            lower: b"\xea\x9e\x81",
            upper: b"\xea\x9e\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA781,
        SpecialCasingMapping {
            lower: b"\xea\x9e\x81",
            upper: b"\xea\x9e\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA782,
        SpecialCasingMapping {
            lower: b"\xea\x9e\x83",
            upper: b"\xea\x9e\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA783,
        SpecialCasingMapping {
            lower: b"\xea\x9e\x83",
            upper: b"\xea\x9e\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA784,
        SpecialCasingMapping {
            lower: b"\xea\x9e\x85",
            upper: b"\xea\x9e\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA785,
        SpecialCasingMapping {
            lower: b"\xea\x9e\x85",
            upper: b"\xea\x9e\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA786,
        SpecialCasingMapping {
            lower: b"\xea\x9e\x87",
            upper: b"\xea\x9e\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA787,
        SpecialCasingMapping {
            lower: b"\xea\x9e\x87",
            upper: b"\xea\x9e\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA78B,
        SpecialCasingMapping {
            lower: b"\xea\x9e\x8c",
            upper: b"\xea\x9e\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA78C,
        SpecialCasingMapping {
            lower: b"\xea\x9e\x8c",
            upper: b"\xea\x9e\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA78D,
        SpecialCasingMapping {
            lower: b"\xc9\xa5",
            upper: b"\xea\x9e\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA790,
        SpecialCasingMapping {
            lower: b"\xea\x9e\x91",
            upper: b"\xea\x9e\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA791,
        SpecialCasingMapping {
            lower: b"\xea\x9e\x91",
            upper: b"\xea\x9e\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA792,
        SpecialCasingMapping {
            lower: b"\xea\x9e\x93",
            upper: b"\xea\x9e\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA793,
        SpecialCasingMapping {
            lower: b"\xea\x9e\x93",
            upper: b"\xea\x9e\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA794,
        SpecialCasingMapping {
            lower: b"\xea\x9e\x94",
            upper: b"\xea\x9f\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA796,
        SpecialCasingMapping {
            lower: b"\xea\x9e\x97",
            upper: b"\xea\x9e\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA797,
        SpecialCasingMapping {
            lower: b"\xea\x9e\x97",
            upper: b"\xea\x9e\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA798,
        SpecialCasingMapping {
            lower: b"\xea\x9e\x99",
            upper: b"\xea\x9e\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA799,
        SpecialCasingMapping {
            lower: b"\xea\x9e\x99",
            upper: b"\xea\x9e\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA79A,
        SpecialCasingMapping {
            lower: b"\xea\x9e\x9b",
            upper: b"\xea\x9e\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA79B,
        SpecialCasingMapping {
            lower: b"\xea\x9e\x9b",
            upper: b"\xea\x9e\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA79C,
        SpecialCasingMapping {
            lower: b"\xea\x9e\x9d",
            upper: b"\xea\x9e\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA79D,
        SpecialCasingMapping {
            lower: b"\xea\x9e\x9d",
            upper: b"\xea\x9e\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA79E,
        SpecialCasingMapping {
            lower: b"\xea\x9e\x9f",
            upper: b"\xea\x9e\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA79F,
        SpecialCasingMapping {
            lower: b"\xea\x9e\x9f",
            upper: b"\xea\x9e\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7A0,
        SpecialCasingMapping {
            lower: b"\xea\x9e\xa1",
            upper: b"\xea\x9e\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7A1,
        SpecialCasingMapping {
            lower: b"\xea\x9e\xa1",
            upper: b"\xea\x9e\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7A2,
        SpecialCasingMapping {
            lower: b"\xea\x9e\xa3",
            upper: b"\xea\x9e\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7A3,
        SpecialCasingMapping {
            lower: b"\xea\x9e\xa3",
            upper: b"\xea\x9e\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7A4,
        SpecialCasingMapping {
            lower: b"\xea\x9e\xa5",
            upper: b"\xea\x9e\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7A5,
        SpecialCasingMapping {
            lower: b"\xea\x9e\xa5",
            upper: b"\xea\x9e\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7A6,
        SpecialCasingMapping {
            lower: b"\xea\x9e\xa7",
            upper: b"\xea\x9e\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7A7,
        SpecialCasingMapping {
            lower: b"\xea\x9e\xa7",
            upper: b"\xea\x9e\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7A8,
        SpecialCasingMapping {
            lower: b"\xea\x9e\xa9",
            upper: b"\xea\x9e\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7A9,
        SpecialCasingMapping {
            lower: b"\xea\x9e\xa9",
            upper: b"\xea\x9e\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7AA,
        SpecialCasingMapping {
            lower: b"\xc9\xa6",
            upper: b"\xea\x9e\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7AB,
        SpecialCasingMapping {
            lower: b"\xc9\x9c",
            upper: b"\xea\x9e\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7AC,
        SpecialCasingMapping {
            lower: b"\xc9\xa1",
            upper: b"\xea\x9e\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7AD,
        SpecialCasingMapping {
            lower: b"\xc9\xac",
            upper: b"\xea\x9e\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7AE,
        SpecialCasingMapping {
            lower: b"\xc9\xaa",
            upper: b"\xea\x9e\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7B0,
        SpecialCasingMapping {
            lower: b"\xca\x9e",
            upper: b"\xea\x9e\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7B1,
        SpecialCasingMapping {
            lower: b"\xca\x87",
            upper: b"\xea\x9e\xb1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7B2,
        SpecialCasingMapping {
            lower: b"\xca\x9d",
            upper: b"\xea\x9e\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7B3,
        SpecialCasingMapping {
            lower: b"\xea\xad\x93",
            upper: b"\xea\x9e\xb3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7B4,
        SpecialCasingMapping {
            lower: b"\xea\x9e\xb5",
            upper: b"\xea\x9e\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7B5,
        SpecialCasingMapping {
            lower: b"\xea\x9e\xb5",
            upper: b"\xea\x9e\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7B6,
        SpecialCasingMapping {
            lower: b"\xea\x9e\xb7",
            upper: b"\xea\x9e\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7B7,
        SpecialCasingMapping {
            lower: b"\xea\x9e\xb7",
            upper: b"\xea\x9e\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7B8,
        SpecialCasingMapping {
            lower: b"\xea\x9e\xb9",
            upper: b"\xea\x9e\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7B9,
        SpecialCasingMapping {
            lower: b"\xea\x9e\xb9",
            upper: b"\xea\x9e\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7BA,
        SpecialCasingMapping {
            lower: b"\xea\x9e\xbb",
            upper: b"\xea\x9e\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7BB,
        SpecialCasingMapping {
            lower: b"\xea\x9e\xbb",
            upper: b"\xea\x9e\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7BC,
        SpecialCasingMapping {
            lower: b"\xea\x9e\xbd",
            upper: b"\xea\x9e\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7BD,
        SpecialCasingMapping {
            lower: b"\xea\x9e\xbd",
            upper: b"\xea\x9e\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7BE,
        SpecialCasingMapping {
            lower: b"\xea\x9e\xbf",
            upper: b"\xea\x9e\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7BF,
        SpecialCasingMapping {
            lower: b"\xea\x9e\xbf",
            upper: b"\xea\x9e\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7C0,
        SpecialCasingMapping {
            lower: b"\xea\x9f\x81",
            upper: b"\xea\x9f\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7C1,
        SpecialCasingMapping {
            lower: b"\xea\x9f\x81",
            upper: b"\xea\x9f\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7C2,
        SpecialCasingMapping {
            lower: b"\xea\x9f\x83",
            upper: b"\xea\x9f\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7C3,
        SpecialCasingMapping {
            lower: b"\xea\x9f\x83",
            upper: b"\xea\x9f\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7C4,
        SpecialCasingMapping {
            lower: b"\xea\x9e\x94",
            upper: b"\xea\x9f\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7C5,
        SpecialCasingMapping {
            lower: b"\xca\x82",
            upper: b"\xea\x9f\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7C6,
        SpecialCasingMapping {
            lower: b"\xe1\xb6\x8e",
            upper: b"\xea\x9f\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7C7,
        SpecialCasingMapping {
            lower: b"\xea\x9f\x88",
            upper: b"\xea\x9f\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7C8,
        SpecialCasingMapping {
            lower: b"\xea\x9f\x88",
            upper: b"\xea\x9f\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7C9,
        SpecialCasingMapping {
            lower: b"\xea\x9f\x8a",
            upper: b"\xea\x9f\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7CA,
        SpecialCasingMapping {
            lower: b"\xea\x9f\x8a",
            upper: b"\xea\x9f\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7D0,
        SpecialCasingMapping {
            lower: b"\xea\x9f\x91",
            upper: b"\xea\x9f\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7D1,
        SpecialCasingMapping {
            lower: b"\xea\x9f\x91",
            upper: b"\xea\x9f\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7D6,
        SpecialCasingMapping {
            lower: b"\xea\x9f\x97",
            upper: b"\xea\x9f\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7D7,
        SpecialCasingMapping {
            lower: b"\xea\x9f\x97",
            upper: b"\xea\x9f\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7D8,
        SpecialCasingMapping {
            lower: b"\xea\x9f\x99",
            upper: b"\xea\x9f\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7D9,
        SpecialCasingMapping {
            lower: b"\xea\x9f\x99",
            upper: b"\xea\x9f\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7F5,
        SpecialCasingMapping {
            lower: b"\xea\x9f\xb6",
            upper: b"\xea\x9f\xb5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xA7F6,
        SpecialCasingMapping {
            lower: b"\xea\x9f\xb6",
            upper: b"\xea\x9f\xb5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB53,
        SpecialCasingMapping {
            lower: b"\xea\xad\x93",
            upper: b"\xea\x9e\xb3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB70,
        SpecialCasingMapping {
            lower: b"\xea\xad\xb0",
            upper: b"\xe1\x8e\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB71,
        SpecialCasingMapping {
            lower: b"\xea\xad\xb1",
            upper: b"\xe1\x8e\xa1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB72,
        SpecialCasingMapping {
            lower: b"\xea\xad\xb2",
            upper: b"\xe1\x8e\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB73,
        SpecialCasingMapping {
            lower: b"\xea\xad\xb3",
            upper: b"\xe1\x8e\xa3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB74,
        SpecialCasingMapping {
            lower: b"\xea\xad\xb4",
            upper: b"\xe1\x8e\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB75,
        SpecialCasingMapping {
            lower: b"\xea\xad\xb5",
            upper: b"\xe1\x8e\xa5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB76,
        SpecialCasingMapping {
            lower: b"\xea\xad\xb6",
            upper: b"\xe1\x8e\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB77,
        SpecialCasingMapping {
            lower: b"\xea\xad\xb7",
            upper: b"\xe1\x8e\xa7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB78,
        SpecialCasingMapping {
            lower: b"\xea\xad\xb8",
            upper: b"\xe1\x8e\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB79,
        SpecialCasingMapping {
            lower: b"\xea\xad\xb9",
            upper: b"\xe1\x8e\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB7A,
        SpecialCasingMapping {
            lower: b"\xea\xad\xba",
            upper: b"\xe1\x8e\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB7B,
        SpecialCasingMapping {
            lower: b"\xea\xad\xbb",
            upper: b"\xe1\x8e\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB7C,
        SpecialCasingMapping {
            lower: b"\xea\xad\xbc",
            upper: b"\xe1\x8e\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB7D,
        SpecialCasingMapping {
            lower: b"\xea\xad\xbd",
            upper: b"\xe1\x8e\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB7E,
        SpecialCasingMapping {
            lower: b"\xea\xad\xbe",
            upper: b"\xe1\x8e\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB7F,
        SpecialCasingMapping {
            lower: b"\xea\xad\xbf",
            upper: b"\xe1\x8e\xaf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB80,
        SpecialCasingMapping {
            lower: b"\xea\xae\x80",
            upper: b"\xe1\x8e\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB81,
        SpecialCasingMapping {
            lower: b"\xea\xae\x81",
            upper: b"\xe1\x8e\xb1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB82,
        SpecialCasingMapping {
            lower: b"\xea\xae\x82",
            upper: b"\xe1\x8e\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB83,
        SpecialCasingMapping {
            lower: b"\xea\xae\x83",
            upper: b"\xe1\x8e\xb3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB84,
        SpecialCasingMapping {
            lower: b"\xea\xae\x84",
            upper: b"\xe1\x8e\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB85,
        SpecialCasingMapping {
            lower: b"\xea\xae\x85",
            upper: b"\xe1\x8e\xb5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB86,
        SpecialCasingMapping {
            lower: b"\xea\xae\x86",
            upper: b"\xe1\x8e\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB87,
        SpecialCasingMapping {
            lower: b"\xea\xae\x87",
            upper: b"\xe1\x8e\xb7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB88,
        SpecialCasingMapping {
            lower: b"\xea\xae\x88",
            upper: b"\xe1\x8e\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB89,
        SpecialCasingMapping {
            lower: b"\xea\xae\x89",
            upper: b"\xe1\x8e\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB8A,
        SpecialCasingMapping {
            lower: b"\xea\xae\x8a",
            upper: b"\xe1\x8e\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB8B,
        SpecialCasingMapping {
            lower: b"\xea\xae\x8b",
            upper: b"\xe1\x8e\xbb",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB8C,
        SpecialCasingMapping {
            lower: b"\xea\xae\x8c",
            upper: b"\xe1\x8e\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB8D,
        SpecialCasingMapping {
            lower: b"\xea\xae\x8d",
            upper: b"\xe1\x8e\xbd",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB8E,
        SpecialCasingMapping {
            lower: b"\xea\xae\x8e",
            upper: b"\xe1\x8e\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB8F,
        SpecialCasingMapping {
            lower: b"\xea\xae\x8f",
            upper: b"\xe1\x8e\xbf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB90,
        SpecialCasingMapping {
            lower: b"\xea\xae\x90",
            upper: b"\xe1\x8f\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB91,
        SpecialCasingMapping {
            lower: b"\xea\xae\x91",
            upper: b"\xe1\x8f\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB92,
        SpecialCasingMapping {
            lower: b"\xea\xae\x92",
            upper: b"\xe1\x8f\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB93,
        SpecialCasingMapping {
            lower: b"\xea\xae\x93",
            upper: b"\xe1\x8f\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB94,
        SpecialCasingMapping {
            lower: b"\xea\xae\x94",
            upper: b"\xe1\x8f\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB95,
        SpecialCasingMapping {
            lower: b"\xea\xae\x95",
            upper: b"\xe1\x8f\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB96,
        SpecialCasingMapping {
            lower: b"\xea\xae\x96",
            upper: b"\xe1\x8f\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB97,
        SpecialCasingMapping {
            lower: b"\xea\xae\x97",
            upper: b"\xe1\x8f\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB98,
        SpecialCasingMapping {
            lower: b"\xea\xae\x98",
            upper: b"\xe1\x8f\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB99,
        SpecialCasingMapping {
            lower: b"\xea\xae\x99",
            upper: b"\xe1\x8f\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB9A,
        SpecialCasingMapping {
            lower: b"\xea\xae\x9a",
            upper: b"\xe1\x8f\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB9B,
        SpecialCasingMapping {
            lower: b"\xea\xae\x9b",
            upper: b"\xe1\x8f\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB9C,
        SpecialCasingMapping {
            lower: b"\xea\xae\x9c",
            upper: b"\xe1\x8f\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB9D,
        SpecialCasingMapping {
            lower: b"\xea\xae\x9d",
            upper: b"\xe1\x8f\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB9E,
        SpecialCasingMapping {
            lower: b"\xea\xae\x9e",
            upper: b"\xe1\x8f\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xAB9F,
        SpecialCasingMapping {
            lower: b"\xea\xae\x9f",
            upper: b"\xe1\x8f\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABA0,
        SpecialCasingMapping {
            lower: b"\xea\xae\xa0",
            upper: b"\xe1\x8f\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABA1,
        SpecialCasingMapping {
            lower: b"\xea\xae\xa1",
            upper: b"\xe1\x8f\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABA2,
        SpecialCasingMapping {
            lower: b"\xea\xae\xa2",
            upper: b"\xe1\x8f\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABA3,
        SpecialCasingMapping {
            lower: b"\xea\xae\xa3",
            upper: b"\xe1\x8f\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABA4,
        SpecialCasingMapping {
            lower: b"\xea\xae\xa4",
            upper: b"\xe1\x8f\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABA5,
        SpecialCasingMapping {
            lower: b"\xea\xae\xa5",
            upper: b"\xe1\x8f\x95",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABA6,
        SpecialCasingMapping {
            lower: b"\xea\xae\xa6",
            upper: b"\xe1\x8f\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABA7,
        SpecialCasingMapping {
            lower: b"\xea\xae\xa7",
            upper: b"\xe1\x8f\x97",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABA8,
        SpecialCasingMapping {
            lower: b"\xea\xae\xa8",
            upper: b"\xe1\x8f\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABA9,
        SpecialCasingMapping {
            lower: b"\xea\xae\xa9",
            upper: b"\xe1\x8f\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABAA,
        SpecialCasingMapping {
            lower: b"\xea\xae\xaa",
            upper: b"\xe1\x8f\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABAB,
        SpecialCasingMapping {
            lower: b"\xea\xae\xab",
            upper: b"\xe1\x8f\x9b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABAC,
        SpecialCasingMapping {
            lower: b"\xea\xae\xac",
            upper: b"\xe1\x8f\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABAD,
        SpecialCasingMapping {
            lower: b"\xea\xae\xad",
            upper: b"\xe1\x8f\x9d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABAE,
        SpecialCasingMapping {
            lower: b"\xea\xae\xae",
            upper: b"\xe1\x8f\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABAF,
        SpecialCasingMapping {
            lower: b"\xea\xae\xaf",
            upper: b"\xe1\x8f\x9f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABB0,
        SpecialCasingMapping {
            lower: b"\xea\xae\xb0",
            upper: b"\xe1\x8f\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABB1,
        SpecialCasingMapping {
            lower: b"\xea\xae\xb1",
            upper: b"\xe1\x8f\xa1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABB2,
        SpecialCasingMapping {
            lower: b"\xea\xae\xb2",
            upper: b"\xe1\x8f\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABB3,
        SpecialCasingMapping {
            lower: b"\xea\xae\xb3",
            upper: b"\xe1\x8f\xa3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABB4,
        SpecialCasingMapping {
            lower: b"\xea\xae\xb4",
            upper: b"\xe1\x8f\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABB5,
        SpecialCasingMapping {
            lower: b"\xea\xae\xb5",
            upper: b"\xe1\x8f\xa5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABB6,
        SpecialCasingMapping {
            lower: b"\xea\xae\xb6",
            upper: b"\xe1\x8f\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABB7,
        SpecialCasingMapping {
            lower: b"\xea\xae\xb7",
            upper: b"\xe1\x8f\xa7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABB8,
        SpecialCasingMapping {
            lower: b"\xea\xae\xb8",
            upper: b"\xe1\x8f\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABB9,
        SpecialCasingMapping {
            lower: b"\xea\xae\xb9",
            upper: b"\xe1\x8f\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABBA,
        SpecialCasingMapping {
            lower: b"\xea\xae\xba",
            upper: b"\xe1\x8f\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABBB,
        SpecialCasingMapping {
            lower: b"\xea\xae\xbb",
            upper: b"\xe1\x8f\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABBC,
        SpecialCasingMapping {
            lower: b"\xea\xae\xbc",
            upper: b"\xe1\x8f\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABBD,
        SpecialCasingMapping {
            lower: b"\xea\xae\xbd",
            upper: b"\xe1\x8f\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABBE,
        SpecialCasingMapping {
            lower: b"\xea\xae\xbe",
            upper: b"\xe1\x8f\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xABBF,
        SpecialCasingMapping {
            lower: b"\xea\xae\xbf",
            upper: b"\xe1\x8f\xaf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFB00,
        SpecialCasingMapping {
            lower: b"\xef\xac\x80",
            upper: b"FF",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFB01,
        SpecialCasingMapping {
            lower: b"\xef\xac\x81",
            upper: b"FI",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFB02,
        SpecialCasingMapping {
            lower: b"\xef\xac\x82",
            upper: b"FL",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFB03,
        SpecialCasingMapping {
            lower: b"\xef\xac\x83",
            upper: b"FFI",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFB04,
        SpecialCasingMapping {
            lower: b"\xef\xac\x84",
            upper: b"FFL",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFB05,
        SpecialCasingMapping {
            lower: b"\xef\xac\x85",
            upper: b"ST",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFB06,
        SpecialCasingMapping {
            lower: b"\xef\xac\x86",
            upper: b"ST",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFB13,
        SpecialCasingMapping {
            lower: b"\xef\xac\x93",
            upper: b"\xd5\x84\xd5\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFB14,
        SpecialCasingMapping {
            lower: b"\xef\xac\x94",
            upper: b"\xd5\x84\xd4\xb5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFB15,
        SpecialCasingMapping {
            lower: b"\xef\xac\x95",
            upper: b"\xd5\x84\xd4\xbb",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFB16,
        SpecialCasingMapping {
            lower: b"\xef\xac\x96",
            upper: b"\xd5\x8e\xd5\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFB17,
        SpecialCasingMapping {
            lower: b"\xef\xac\x97",
            upper: b"\xd5\x84\xd4\xbd",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF21,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x81",
            upper: b"\xef\xbc\xa1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF22,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x82",
            upper: b"\xef\xbc\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF23,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x83",
            upper: b"\xef\xbc\xa3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF24,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x84",
            upper: b"\xef\xbc\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF25,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x85",
            upper: b"\xef\xbc\xa5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF26,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x86",
            upper: b"\xef\xbc\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF27,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x87",
            upper: b"\xef\xbc\xa7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF28,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x88",
            upper: b"\xef\xbc\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF29,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x89",
            upper: b"\xef\xbc\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF2A,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x8a",
            upper: b"\xef\xbc\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF2B,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x8b",
            upper: b"\xef\xbc\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF2C,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x8c",
            upper: b"\xef\xbc\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF2D,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x8d",
            upper: b"\xef\xbc\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF2E,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x8e",
            upper: b"\xef\xbc\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF2F,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x8f",
            upper: b"\xef\xbc\xaf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF30,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x90",
            upper: b"\xef\xbc\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF31,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x91",
            upper: b"\xef\xbc\xb1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF32,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x92",
            upper: b"\xef\xbc\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF33,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x93",
            upper: b"\xef\xbc\xb3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF34,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x94",
            upper: b"\xef\xbc\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF35,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x95",
            upper: b"\xef\xbc\xb5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF36,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x96",
            upper: b"\xef\xbc\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF37,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x97",
            upper: b"\xef\xbc\xb7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF38,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x98",
            upper: b"\xef\xbc\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF39,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x99",
            upper: b"\xef\xbc\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF3A,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x9a",
            upper: b"\xef\xbc\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF41,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x81",
            upper: b"\xef\xbc\xa1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF42,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x82",
            upper: b"\xef\xbc\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF43,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x83",
            upper: b"\xef\xbc\xa3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF44,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x84",
            upper: b"\xef\xbc\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF45,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x85",
            upper: b"\xef\xbc\xa5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF46,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x86",
            upper: b"\xef\xbc\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF47,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x87",
            upper: b"\xef\xbc\xa7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF48,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x88",
            upper: b"\xef\xbc\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF49,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x89",
            upper: b"\xef\xbc\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF4A,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x8a",
            upper: b"\xef\xbc\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF4B,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x8b",
            upper: b"\xef\xbc\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF4C,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x8c",
            upper: b"\xef\xbc\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF4D,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x8d",
            upper: b"\xef\xbc\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF4E,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x8e",
            upper: b"\xef\xbc\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF4F,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x8f",
            upper: b"\xef\xbc\xaf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF50,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x90",
            upper: b"\xef\xbc\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF51,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x91",
            upper: b"\xef\xbc\xb1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF52,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x92",
            upper: b"\xef\xbc\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF53,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x93",
            upper: b"\xef\xbc\xb3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF54,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x94",
            upper: b"\xef\xbc\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF55,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x95",
            upper: b"\xef\xbc\xb5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF56,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x96",
            upper: b"\xef\xbc\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF57,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x97",
            upper: b"\xef\xbc\xb7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF58,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x98",
            upper: b"\xef\xbc\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF59,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x99",
            upper: b"\xef\xbc\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0xFF5A,
        SpecialCasingMapping {
            lower: b"\xef\xbd\x9a",
            upper: b"\xef\xbc\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10400,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xa8",
            upper: b"\xf0\x90\x90\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10401,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xa9",
            upper: b"\xf0\x90\x90\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10402,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xaa",
            upper: b"\xf0\x90\x90\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10403,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xab",
            upper: b"\xf0\x90\x90\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10404,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xac",
            upper: b"\xf0\x90\x90\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10405,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xad",
            upper: b"\xf0\x90\x90\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10406,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xae",
            upper: b"\xf0\x90\x90\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10407,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xaf",
            upper: b"\xf0\x90\x90\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10408,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xb0",
            upper: b"\xf0\x90\x90\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10409,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xb1",
            upper: b"\xf0\x90\x90\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1040A,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xb2",
            upper: b"\xf0\x90\x90\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1040B,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xb3",
            upper: b"\xf0\x90\x90\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1040C,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xb4",
            upper: b"\xf0\x90\x90\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1040D,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xb5",
            upper: b"\xf0\x90\x90\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1040E,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xb6",
            upper: b"\xf0\x90\x90\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1040F,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xb7",
            upper: b"\xf0\x90\x90\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10410,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xb8",
            upper: b"\xf0\x90\x90\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10411,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xb9",
            upper: b"\xf0\x90\x90\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10412,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xba",
            upper: b"\xf0\x90\x90\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10413,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xbb",
            upper: b"\xf0\x90\x90\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10414,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xbc",
            upper: b"\xf0\x90\x90\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10415,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xbd",
            upper: b"\xf0\x90\x90\x95",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10416,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xbe",
            upper: b"\xf0\x90\x90\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10417,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xbf",
            upper: b"\xf0\x90\x90\x97",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10418,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x80",
            upper: b"\xf0\x90\x90\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10419,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x81",
            upper: b"\xf0\x90\x90\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1041A,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x82",
            upper: b"\xf0\x90\x90\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1041B,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x83",
            upper: b"\xf0\x90\x90\x9b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1041C,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x84",
            upper: b"\xf0\x90\x90\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1041D,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x85",
            upper: b"\xf0\x90\x90\x9d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1041E,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x86",
            upper: b"\xf0\x90\x90\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1041F,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x87",
            upper: b"\xf0\x90\x90\x9f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10420,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x88",
            upper: b"\xf0\x90\x90\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10421,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x89",
            upper: b"\xf0\x90\x90\xa1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10422,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x8a",
            upper: b"\xf0\x90\x90\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10423,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x8b",
            upper: b"\xf0\x90\x90\xa3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10424,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x8c",
            upper: b"\xf0\x90\x90\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10425,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x8d",
            upper: b"\xf0\x90\x90\xa5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10426,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x8e",
            upper: b"\xf0\x90\x90\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10427,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x8f",
            upper: b"\xf0\x90\x90\xa7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10428,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xa8",
            upper: b"\xf0\x90\x90\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10429,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xa9",
            upper: b"\xf0\x90\x90\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1042A,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xaa",
            upper: b"\xf0\x90\x90\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1042B,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xab",
            upper: b"\xf0\x90\x90\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1042C,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xac",
            upper: b"\xf0\x90\x90\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1042D,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xad",
            upper: b"\xf0\x90\x90\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1042E,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xae",
            upper: b"\xf0\x90\x90\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1042F,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xaf",
            upper: b"\xf0\x90\x90\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10430,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xb0",
            upper: b"\xf0\x90\x90\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10431,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xb1",
            upper: b"\xf0\x90\x90\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10432,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xb2",
            upper: b"\xf0\x90\x90\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10433,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xb3",
            upper: b"\xf0\x90\x90\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10434,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xb4",
            upper: b"\xf0\x90\x90\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10435,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xb5",
            upper: b"\xf0\x90\x90\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10436,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xb6",
            upper: b"\xf0\x90\x90\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10437,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xb7",
            upper: b"\xf0\x90\x90\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10438,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xb8",
            upper: b"\xf0\x90\x90\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10439,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xb9",
            upper: b"\xf0\x90\x90\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1043A,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xba",
            upper: b"\xf0\x90\x90\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1043B,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xbb",
            upper: b"\xf0\x90\x90\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1043C,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xbc",
            upper: b"\xf0\x90\x90\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1043D,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xbd",
            upper: b"\xf0\x90\x90\x95",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1043E,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xbe",
            upper: b"\xf0\x90\x90\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1043F,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x90\xbf",
            upper: b"\xf0\x90\x90\x97",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10440,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x80",
            upper: b"\xf0\x90\x90\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10441,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x81",
            upper: b"\xf0\x90\x90\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10442,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x82",
            upper: b"\xf0\x90\x90\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10443,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x83",
            upper: b"\xf0\x90\x90\x9b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10444,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x84",
            upper: b"\xf0\x90\x90\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10445,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x85",
            upper: b"\xf0\x90\x90\x9d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10446,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x86",
            upper: b"\xf0\x90\x90\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10447,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x87",
            upper: b"\xf0\x90\x90\x9f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10448,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x88",
            upper: b"\xf0\x90\x90\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10449,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x89",
            upper: b"\xf0\x90\x90\xa1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1044A,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x8a",
            upper: b"\xf0\x90\x90\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1044B,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x8b",
            upper: b"\xf0\x90\x90\xa3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1044C,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x8c",
            upper: b"\xf0\x90\x90\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1044D,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x8d",
            upper: b"\xf0\x90\x90\xa5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1044E,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x8e",
            upper: b"\xf0\x90\x90\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1044F,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x91\x8f",
            upper: b"\xf0\x90\x90\xa7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104B0,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\x98",
            upper: b"\xf0\x90\x92\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104B1,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\x99",
            upper: b"\xf0\x90\x92\xb1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104B2,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\x9a",
            upper: b"\xf0\x90\x92\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104B3,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\x9b",
            upper: b"\xf0\x90\x92\xb3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104B4,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\x9c",
            upper: b"\xf0\x90\x92\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104B5,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\x9d",
            upper: b"\xf0\x90\x92\xb5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104B6,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\x9e",
            upper: b"\xf0\x90\x92\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104B7,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\x9f",
            upper: b"\xf0\x90\x92\xb7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104B8,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xa0",
            upper: b"\xf0\x90\x92\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104B9,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xa1",
            upper: b"\xf0\x90\x92\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104BA,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xa2",
            upper: b"\xf0\x90\x92\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104BB,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xa3",
            upper: b"\xf0\x90\x92\xbb",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104BC,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xa4",
            upper: b"\xf0\x90\x92\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104BD,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xa5",
            upper: b"\xf0\x90\x92\xbd",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104BE,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xa6",
            upper: b"\xf0\x90\x92\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104BF,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xa7",
            upper: b"\xf0\x90\x92\xbf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104C0,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xa8",
            upper: b"\xf0\x90\x93\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104C1,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xa9",
            upper: b"\xf0\x90\x93\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104C2,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xaa",
            upper: b"\xf0\x90\x93\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104C3,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xab",
            upper: b"\xf0\x90\x93\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104C4,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xac",
            upper: b"\xf0\x90\x93\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104C5,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xad",
            upper: b"\xf0\x90\x93\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104C6,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xae",
            upper: b"\xf0\x90\x93\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104C7,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xaf",
            upper: b"\xf0\x90\x93\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104C8,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xb0",
            upper: b"\xf0\x90\x93\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104C9,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xb1",
            upper: b"\xf0\x90\x93\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104CA,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xb2",
            upper: b"\xf0\x90\x93\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104CB,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xb3",
            upper: b"\xf0\x90\x93\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104CC,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xb4",
            upper: b"\xf0\x90\x93\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104CD,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xb5",
            upper: b"\xf0\x90\x93\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104CE,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xb6",
            upper: b"\xf0\x90\x93\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104CF,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xb7",
            upper: b"\xf0\x90\x93\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104D0,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xb8",
            upper: b"\xf0\x90\x93\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104D1,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xb9",
            upper: b"\xf0\x90\x93\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104D2,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xba",
            upper: b"\xf0\x90\x93\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104D3,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xbb",
            upper: b"\xf0\x90\x93\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104D8,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\x98",
            upper: b"\xf0\x90\x92\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104D9,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\x99",
            upper: b"\xf0\x90\x92\xb1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104DA,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\x9a",
            upper: b"\xf0\x90\x92\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104DB,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\x9b",
            upper: b"\xf0\x90\x92\xb3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104DC,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\x9c",
            upper: b"\xf0\x90\x92\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104DD,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\x9d",
            upper: b"\xf0\x90\x92\xb5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104DE,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\x9e",
            upper: b"\xf0\x90\x92\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104DF,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\x9f",
            upper: b"\xf0\x90\x92\xb7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104E0,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xa0",
            upper: b"\xf0\x90\x92\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104E1,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xa1",
            upper: b"\xf0\x90\x92\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104E2,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xa2",
            upper: b"\xf0\x90\x92\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104E3,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xa3",
            upper: b"\xf0\x90\x92\xbb",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104E4,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xa4",
            upper: b"\xf0\x90\x92\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104E5,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xa5",
            upper: b"\xf0\x90\x92\xbd",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104E6,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xa6",
            upper: b"\xf0\x90\x92\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104E7,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xa7",
            upper: b"\xf0\x90\x92\xbf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104E8,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xa8",
            upper: b"\xf0\x90\x93\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104E9,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xa9",
            upper: b"\xf0\x90\x93\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104EA,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xaa",
            upper: b"\xf0\x90\x93\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104EB,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xab",
            upper: b"\xf0\x90\x93\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104EC,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xac",
            upper: b"\xf0\x90\x93\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104ED,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xad",
            upper: b"\xf0\x90\x93\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104EE,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xae",
            upper: b"\xf0\x90\x93\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104EF,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xaf",
            upper: b"\xf0\x90\x93\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104F0,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xb0",
            upper: b"\xf0\x90\x93\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104F1,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xb1",
            upper: b"\xf0\x90\x93\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104F2,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xb2",
            upper: b"\xf0\x90\x93\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104F3,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xb3",
            upper: b"\xf0\x90\x93\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104F4,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xb4",
            upper: b"\xf0\x90\x93\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104F5,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xb5",
            upper: b"\xf0\x90\x93\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104F6,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xb6",
            upper: b"\xf0\x90\x93\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104F7,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xb7",
            upper: b"\xf0\x90\x93\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104F8,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xb8",
            upper: b"\xf0\x90\x93\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104F9,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xb9",
            upper: b"\xf0\x90\x93\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104FA,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xba",
            upper: b"\xf0\x90\x93\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x104FB,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x93\xbb",
            upper: b"\xf0\x90\x93\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10570,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\x97",
            upper: b"\xf0\x90\x95\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10571,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\x98",
            upper: b"\xf0\x90\x95\xb1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10572,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\x99",
            upper: b"\xf0\x90\x95\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10573,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\x9a",
            upper: b"\xf0\x90\x95\xb3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10574,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\x9b",
            upper: b"\xf0\x90\x95\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10575,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\x9c",
            upper: b"\xf0\x90\x95\xb5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10576,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\x9d",
            upper: b"\xf0\x90\x95\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10577,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\x9e",
            upper: b"\xf0\x90\x95\xb7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10578,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\x9f",
            upper: b"\xf0\x90\x95\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10579,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xa0",
            upper: b"\xf0\x90\x95\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1057A,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xa1",
            upper: b"\xf0\x90\x95\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1057C,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xa3",
            upper: b"\xf0\x90\x95\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1057D,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xa4",
            upper: b"\xf0\x90\x95\xbd",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1057E,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xa5",
            upper: b"\xf0\x90\x95\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1057F,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xa6",
            upper: b"\xf0\x90\x95\xbf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10580,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xa7",
            upper: b"\xf0\x90\x96\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10581,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xa8",
            upper: b"\xf0\x90\x96\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10582,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xa9",
            upper: b"\xf0\x90\x96\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10583,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xaa",
            upper: b"\xf0\x90\x96\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10584,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xab",
            upper: b"\xf0\x90\x96\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10585,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xac",
            upper: b"\xf0\x90\x96\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10586,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xad",
            upper: b"\xf0\x90\x96\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10587,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xae",
            upper: b"\xf0\x90\x96\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10588,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xaf",
            upper: b"\xf0\x90\x96\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10589,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xb0",
            upper: b"\xf0\x90\x96\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1058A,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xb1",
            upper: b"\xf0\x90\x96\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1058C,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xb3",
            upper: b"\xf0\x90\x96\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1058D,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xb4",
            upper: b"\xf0\x90\x96\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1058E,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xb5",
            upper: b"\xf0\x90\x96\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1058F,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xb6",
            upper: b"\xf0\x90\x96\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10590,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xb7",
            upper: b"\xf0\x90\x96\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10591,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xb8",
            upper: b"\xf0\x90\x96\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10592,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xb9",
            upper: b"\xf0\x90\x96\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10594,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xbb",
            upper: b"\xf0\x90\x96\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10595,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xbc",
            upper: b"\xf0\x90\x96\x95",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10597,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\x97",
            upper: b"\xf0\x90\x95\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10598,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\x98",
            upper: b"\xf0\x90\x95\xb1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10599,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\x99",
            upper: b"\xf0\x90\x95\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1059A,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\x9a",
            upper: b"\xf0\x90\x95\xb3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1059B,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\x9b",
            upper: b"\xf0\x90\x95\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1059C,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\x9c",
            upper: b"\xf0\x90\x95\xb5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1059D,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\x9d",
            upper: b"\xf0\x90\x95\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1059E,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\x9e",
            upper: b"\xf0\x90\x95\xb7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1059F,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\x9f",
            upper: b"\xf0\x90\x95\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x105A0,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xa0",
            upper: b"\xf0\x90\x95\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x105A1,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xa1",
            upper: b"\xf0\x90\x95\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x105A3,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xa3",
            upper: b"\xf0\x90\x95\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x105A4,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xa4",
            upper: b"\xf0\x90\x95\xbd",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x105A5,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xa5",
            upper: b"\xf0\x90\x95\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x105A6,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xa6",
            upper: b"\xf0\x90\x95\xbf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x105A7,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xa7",
            upper: b"\xf0\x90\x96\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x105A8,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xa8",
            upper: b"\xf0\x90\x96\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x105A9,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xa9",
            upper: b"\xf0\x90\x96\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x105AA,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xaa",
            upper: b"\xf0\x90\x96\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x105AB,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xab",
            upper: b"\xf0\x90\x96\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x105AC,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xac",
            upper: b"\xf0\x90\x96\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x105AD,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xad",
            upper: b"\xf0\x90\x96\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x105AE,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xae",
            upper: b"\xf0\x90\x96\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x105AF,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xaf",
            upper: b"\xf0\x90\x96\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x105B0,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xb0",
            upper: b"\xf0\x90\x96\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x105B1,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xb1",
            upper: b"\xf0\x90\x96\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x105B3,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xb3",
            upper: b"\xf0\x90\x96\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x105B4,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xb4",
            upper: b"\xf0\x90\x96\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x105B5,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xb5",
            upper: b"\xf0\x90\x96\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x105B6,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xb6",
            upper: b"\xf0\x90\x96\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x105B7,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xb7",
            upper: b"\xf0\x90\x96\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x105B8,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xb8",
            upper: b"\xf0\x90\x96\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x105B9,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xb9",
            upper: b"\xf0\x90\x96\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x105BB,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xbb",
            upper: b"\xf0\x90\x96\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x105BC,
        SpecialCasingMapping {
            lower: b"\xf0\x90\x96\xbc",
            upper: b"\xf0\x90\x96\x95",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C80,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x80",
            upper: b"\xf0\x90\xb2\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C81,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x81",
            upper: b"\xf0\x90\xb2\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C82,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x82",
            upper: b"\xf0\x90\xb2\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C83,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x83",
            upper: b"\xf0\x90\xb2\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C84,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x84",
            upper: b"\xf0\x90\xb2\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C85,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x85",
            upper: b"\xf0\x90\xb2\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C86,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x86",
            upper: b"\xf0\x90\xb2\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C87,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x87",
            upper: b"\xf0\x90\xb2\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C88,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x88",
            upper: b"\xf0\x90\xb2\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C89,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x89",
            upper: b"\xf0\x90\xb2\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C8A,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x8a",
            upper: b"\xf0\x90\xb2\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C8B,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x8b",
            upper: b"\xf0\x90\xb2\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C8C,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x8c",
            upper: b"\xf0\x90\xb2\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C8D,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x8d",
            upper: b"\xf0\x90\xb2\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C8E,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x8e",
            upper: b"\xf0\x90\xb2\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C8F,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x8f",
            upper: b"\xf0\x90\xb2\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C90,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x90",
            upper: b"\xf0\x90\xb2\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C91,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x91",
            upper: b"\xf0\x90\xb2\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C92,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x92",
            upper: b"\xf0\x90\xb2\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C93,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x93",
            upper: b"\xf0\x90\xb2\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C94,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x94",
            upper: b"\xf0\x90\xb2\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C95,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x95",
            upper: b"\xf0\x90\xb2\x95",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C96,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x96",
            upper: b"\xf0\x90\xb2\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C97,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x97",
            upper: b"\xf0\x90\xb2\x97",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C98,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x98",
            upper: b"\xf0\x90\xb2\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C99,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x99",
            upper: b"\xf0\x90\xb2\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C9A,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x9a",
            upper: b"\xf0\x90\xb2\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C9B,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x9b",
            upper: b"\xf0\x90\xb2\x9b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C9C,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x9c",
            upper: b"\xf0\x90\xb2\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C9D,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x9d",
            upper: b"\xf0\x90\xb2\x9d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C9E,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x9e",
            upper: b"\xf0\x90\xb2\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10C9F,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x9f",
            upper: b"\xf0\x90\xb2\x9f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CA0,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xa0",
            upper: b"\xf0\x90\xb2\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CA1,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xa1",
            upper: b"\xf0\x90\xb2\xa1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CA2,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xa2",
            upper: b"\xf0\x90\xb2\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CA3,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xa3",
            upper: b"\xf0\x90\xb2\xa3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CA4,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xa4",
            upper: b"\xf0\x90\xb2\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CA5,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xa5",
            upper: b"\xf0\x90\xb2\xa5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CA6,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xa6",
            upper: b"\xf0\x90\xb2\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CA7,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xa7",
            upper: b"\xf0\x90\xb2\xa7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CA8,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xa8",
            upper: b"\xf0\x90\xb2\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CA9,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xa9",
            upper: b"\xf0\x90\xb2\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CAA,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xaa",
            upper: b"\xf0\x90\xb2\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CAB,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xab",
            upper: b"\xf0\x90\xb2\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CAC,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xac",
            upper: b"\xf0\x90\xb2\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CAD,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xad",
            upper: b"\xf0\x90\xb2\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CAE,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xae",
            upper: b"\xf0\x90\xb2\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CAF,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xaf",
            upper: b"\xf0\x90\xb2\xaf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CB0,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xb0",
            upper: b"\xf0\x90\xb2\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CB1,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xb1",
            upper: b"\xf0\x90\xb2\xb1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CB2,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xb2",
            upper: b"\xf0\x90\xb2\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CC0,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x80",
            upper: b"\xf0\x90\xb2\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CC1,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x81",
            upper: b"\xf0\x90\xb2\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CC2,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x82",
            upper: b"\xf0\x90\xb2\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CC3,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x83",
            upper: b"\xf0\x90\xb2\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CC4,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x84",
            upper: b"\xf0\x90\xb2\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CC5,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x85",
            upper: b"\xf0\x90\xb2\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CC6,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x86",
            upper: b"\xf0\x90\xb2\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CC7,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x87",
            upper: b"\xf0\x90\xb2\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CC8,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x88",
            upper: b"\xf0\x90\xb2\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CC9,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x89",
            upper: b"\xf0\x90\xb2\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CCA,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x8a",
            upper: b"\xf0\x90\xb2\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CCB,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x8b",
            upper: b"\xf0\x90\xb2\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CCC,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x8c",
            upper: b"\xf0\x90\xb2\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CCD,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x8d",
            upper: b"\xf0\x90\xb2\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CCE,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x8e",
            upper: b"\xf0\x90\xb2\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CCF,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x8f",
            upper: b"\xf0\x90\xb2\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CD0,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x90",
            upper: b"\xf0\x90\xb2\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CD1,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x91",
            upper: b"\xf0\x90\xb2\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CD2,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x92",
            upper: b"\xf0\x90\xb2\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CD3,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x93",
            upper: b"\xf0\x90\xb2\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CD4,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x94",
            upper: b"\xf0\x90\xb2\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CD5,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x95",
            upper: b"\xf0\x90\xb2\x95",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CD6,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x96",
            upper: b"\xf0\x90\xb2\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CD7,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x97",
            upper: b"\xf0\x90\xb2\x97",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CD8,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x98",
            upper: b"\xf0\x90\xb2\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CD9,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x99",
            upper: b"\xf0\x90\xb2\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CDA,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x9a",
            upper: b"\xf0\x90\xb2\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CDB,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x9b",
            upper: b"\xf0\x90\xb2\x9b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CDC,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x9c",
            upper: b"\xf0\x90\xb2\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CDD,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x9d",
            upper: b"\xf0\x90\xb2\x9d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CDE,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x9e",
            upper: b"\xf0\x90\xb2\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CDF,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\x9f",
            upper: b"\xf0\x90\xb2\x9f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CE0,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xa0",
            upper: b"\xf0\x90\xb2\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CE1,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xa1",
            upper: b"\xf0\x90\xb2\xa1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CE2,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xa2",
            upper: b"\xf0\x90\xb2\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CE3,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xa3",
            upper: b"\xf0\x90\xb2\xa3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CE4,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xa4",
            upper: b"\xf0\x90\xb2\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CE5,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xa5",
            upper: b"\xf0\x90\xb2\xa5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CE6,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xa6",
            upper: b"\xf0\x90\xb2\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CE7,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xa7",
            upper: b"\xf0\x90\xb2\xa7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CE8,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xa8",
            upper: b"\xf0\x90\xb2\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CE9,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xa9",
            upper: b"\xf0\x90\xb2\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CEA,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xaa",
            upper: b"\xf0\x90\xb2\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CEB,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xab",
            upper: b"\xf0\x90\xb2\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CEC,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xac",
            upper: b"\xf0\x90\xb2\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CED,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xad",
            upper: b"\xf0\x90\xb2\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CEE,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xae",
            upper: b"\xf0\x90\xb2\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CEF,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xaf",
            upper: b"\xf0\x90\xb2\xaf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CF0,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xb0",
            upper: b"\xf0\x90\xb2\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CF1,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xb1",
            upper: b"\xf0\x90\xb2\xb1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x10CF2,
        SpecialCasingMapping {
            lower: b"\xf0\x90\xb3\xb2",
            upper: b"\xf0\x90\xb2\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118A0,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x80",
            upper: b"\xf0\x91\xa2\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118A1,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x81",
            upper: b"\xf0\x91\xa2\xa1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118A2,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x82",
            upper: b"\xf0\x91\xa2\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118A3,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x83",
            upper: b"\xf0\x91\xa2\xa3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118A4,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x84",
            upper: b"\xf0\x91\xa2\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118A5,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x85",
            upper: b"\xf0\x91\xa2\xa5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118A6,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x86",
            upper: b"\xf0\x91\xa2\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118A7,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x87",
            upper: b"\xf0\x91\xa2\xa7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118A8,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x88",
            upper: b"\xf0\x91\xa2\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118A9,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x89",
            upper: b"\xf0\x91\xa2\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118AA,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x8a",
            upper: b"\xf0\x91\xa2\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118AB,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x8b",
            upper: b"\xf0\x91\xa2\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118AC,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x8c",
            upper: b"\xf0\x91\xa2\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118AD,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x8d",
            upper: b"\xf0\x91\xa2\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118AE,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x8e",
            upper: b"\xf0\x91\xa2\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118AF,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x8f",
            upper: b"\xf0\x91\xa2\xaf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118B0,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x90",
            upper: b"\xf0\x91\xa2\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118B1,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x91",
            upper: b"\xf0\x91\xa2\xb1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118B2,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x92",
            upper: b"\xf0\x91\xa2\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118B3,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x93",
            upper: b"\xf0\x91\xa2\xb3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118B4,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x94",
            upper: b"\xf0\x91\xa2\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118B5,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x95",
            upper: b"\xf0\x91\xa2\xb5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118B6,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x96",
            upper: b"\xf0\x91\xa2\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118B7,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x97",
            upper: b"\xf0\x91\xa2\xb7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118B8,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x98",
            upper: b"\xf0\x91\xa2\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118B9,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x99",
            upper: b"\xf0\x91\xa2\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118BA,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x9a",
            upper: b"\xf0\x91\xa2\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118BB,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x9b",
            upper: b"\xf0\x91\xa2\xbb",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118BC,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x9c",
            upper: b"\xf0\x91\xa2\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118BD,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x9d",
            upper: b"\xf0\x91\xa2\xbd",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118BE,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x9e",
            upper: b"\xf0\x91\xa2\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118BF,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x9f",
            upper: b"\xf0\x91\xa2\xbf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118C0,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x80",
            upper: b"\xf0\x91\xa2\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118C1,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x81",
            upper: b"\xf0\x91\xa2\xa1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118C2,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x82",
            upper: b"\xf0\x91\xa2\xa2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118C3,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x83",
            upper: b"\xf0\x91\xa2\xa3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118C4,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x84",
            upper: b"\xf0\x91\xa2\xa4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118C5,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x85",
            upper: b"\xf0\x91\xa2\xa5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118C6,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x86",
            upper: b"\xf0\x91\xa2\xa6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118C7,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x87",
            upper: b"\xf0\x91\xa2\xa7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118C8,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x88",
            upper: b"\xf0\x91\xa2\xa8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118C9,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x89",
            upper: b"\xf0\x91\xa2\xa9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118CA,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x8a",
            upper: b"\xf0\x91\xa2\xaa",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118CB,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x8b",
            upper: b"\xf0\x91\xa2\xab",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118CC,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x8c",
            upper: b"\xf0\x91\xa2\xac",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118CD,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x8d",
            upper: b"\xf0\x91\xa2\xad",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118CE,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x8e",
            upper: b"\xf0\x91\xa2\xae",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118CF,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x8f",
            upper: b"\xf0\x91\xa2\xaf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118D0,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x90",
            upper: b"\xf0\x91\xa2\xb0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118D1,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x91",
            upper: b"\xf0\x91\xa2\xb1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118D2,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x92",
            upper: b"\xf0\x91\xa2\xb2",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118D3,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x93",
            upper: b"\xf0\x91\xa2\xb3",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118D4,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x94",
            upper: b"\xf0\x91\xa2\xb4",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118D5,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x95",
            upper: b"\xf0\x91\xa2\xb5",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118D6,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x96",
            upper: b"\xf0\x91\xa2\xb6",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118D7,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x97",
            upper: b"\xf0\x91\xa2\xb7",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118D8,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x98",
            upper: b"\xf0\x91\xa2\xb8",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118D9,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x99",
            upper: b"\xf0\x91\xa2\xb9",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118DA,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x9a",
            upper: b"\xf0\x91\xa2\xba",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118DB,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x9b",
            upper: b"\xf0\x91\xa2\xbb",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118DC,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x9c",
            upper: b"\xf0\x91\xa2\xbc",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118DD,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x9d",
            upper: b"\xf0\x91\xa2\xbd",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118DE,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x9e",
            upper: b"\xf0\x91\xa2\xbe",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x118DF,
        SpecialCasingMapping {
            lower: b"\xf0\x91\xa3\x9f",
            upper: b"\xf0\x91\xa2\xbf",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E40,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xa0",
            upper: b"\xf0\x96\xb9\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E41,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xa1",
            upper: b"\xf0\x96\xb9\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E42,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xa2",
            upper: b"\xf0\x96\xb9\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E43,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xa3",
            upper: b"\xf0\x96\xb9\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E44,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xa4",
            upper: b"\xf0\x96\xb9\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E45,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xa5",
            upper: b"\xf0\x96\xb9\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E46,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xa6",
            upper: b"\xf0\x96\xb9\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E47,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xa7",
            upper: b"\xf0\x96\xb9\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E48,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xa8",
            upper: b"\xf0\x96\xb9\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E49,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xa9",
            upper: b"\xf0\x96\xb9\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E4A,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xaa",
            upper: b"\xf0\x96\xb9\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E4B,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xab",
            upper: b"\xf0\x96\xb9\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E4C,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xac",
            upper: b"\xf0\x96\xb9\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E4D,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xad",
            upper: b"\xf0\x96\xb9\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E4E,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xae",
            upper: b"\xf0\x96\xb9\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E4F,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xaf",
            upper: b"\xf0\x96\xb9\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E50,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xb0",
            upper: b"\xf0\x96\xb9\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E51,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xb1",
            upper: b"\xf0\x96\xb9\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E52,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xb2",
            upper: b"\xf0\x96\xb9\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E53,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xb3",
            upper: b"\xf0\x96\xb9\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E54,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xb4",
            upper: b"\xf0\x96\xb9\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E55,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xb5",
            upper: b"\xf0\x96\xb9\x95",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E56,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xb6",
            upper: b"\xf0\x96\xb9\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E57,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xb7",
            upper: b"\xf0\x96\xb9\x97",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E58,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xb8",
            upper: b"\xf0\x96\xb9\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E59,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xb9",
            upper: b"\xf0\x96\xb9\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E5A,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xba",
            upper: b"\xf0\x96\xb9\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E5B,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xbb",
            upper: b"\xf0\x96\xb9\x9b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E5C,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xbc",
            upper: b"\xf0\x96\xb9\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E5D,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xbd",
            upper: b"\xf0\x96\xb9\x9d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E5E,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xbe",
            upper: b"\xf0\x96\xb9\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E5F,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xbf",
            upper: b"\xf0\x96\xb9\x9f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E60,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xa0",
            upper: b"\xf0\x96\xb9\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E61,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xa1",
            upper: b"\xf0\x96\xb9\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E62,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xa2",
            upper: b"\xf0\x96\xb9\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E63,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xa3",
            upper: b"\xf0\x96\xb9\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E64,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xa4",
            upper: b"\xf0\x96\xb9\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E65,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xa5",
            upper: b"\xf0\x96\xb9\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E66,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xa6",
            upper: b"\xf0\x96\xb9\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E67,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xa7",
            upper: b"\xf0\x96\xb9\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E68,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xa8",
            upper: b"\xf0\x96\xb9\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E69,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xa9",
            upper: b"\xf0\x96\xb9\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E6A,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xaa",
            upper: b"\xf0\x96\xb9\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E6B,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xab",
            upper: b"\xf0\x96\xb9\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E6C,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xac",
            upper: b"\xf0\x96\xb9\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E6D,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xad",
            upper: b"\xf0\x96\xb9\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E6E,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xae",
            upper: b"\xf0\x96\xb9\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E6F,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xaf",
            upper: b"\xf0\x96\xb9\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E70,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xb0",
            upper: b"\xf0\x96\xb9\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E71,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xb1",
            upper: b"\xf0\x96\xb9\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E72,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xb2",
            upper: b"\xf0\x96\xb9\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E73,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xb3",
            upper: b"\xf0\x96\xb9\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E74,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xb4",
            upper: b"\xf0\x96\xb9\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E75,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xb5",
            upper: b"\xf0\x96\xb9\x95",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E76,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xb6",
            upper: b"\xf0\x96\xb9\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E77,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xb7",
            upper: b"\xf0\x96\xb9\x97",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E78,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xb8",
            upper: b"\xf0\x96\xb9\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E79,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xb9",
            upper: b"\xf0\x96\xb9\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E7A,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xba",
            upper: b"\xf0\x96\xb9\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E7B,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xbb",
            upper: b"\xf0\x96\xb9\x9b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E7C,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xbc",
            upper: b"\xf0\x96\xb9\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E7D,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xbd",
            upper: b"\xf0\x96\xb9\x9d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E7E,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xbe",
            upper: b"\xf0\x96\xb9\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x16E7F,
        SpecialCasingMapping {
            lower: b"\xf0\x96\xb9\xbf",
            upper: b"\xf0\x96\xb9\x9f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E900,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xa2",
            upper: b"\xf0\x9e\xa4\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E901,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xa3",
            upper: b"\xf0\x9e\xa4\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E902,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xa4",
            upper: b"\xf0\x9e\xa4\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E903,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xa5",
            upper: b"\xf0\x9e\xa4\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E904,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xa6",
            upper: b"\xf0\x9e\xa4\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E905,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xa7",
            upper: b"\xf0\x9e\xa4\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E906,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xa8",
            upper: b"\xf0\x9e\xa4\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E907,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xa9",
            upper: b"\xf0\x9e\xa4\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E908,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xaa",
            upper: b"\xf0\x9e\xa4\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E909,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xab",
            upper: b"\xf0\x9e\xa4\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E90A,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xac",
            upper: b"\xf0\x9e\xa4\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E90B,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xad",
            upper: b"\xf0\x9e\xa4\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E90C,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xae",
            upper: b"\xf0\x9e\xa4\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E90D,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xaf",
            upper: b"\xf0\x9e\xa4\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E90E,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xb0",
            upper: b"\xf0\x9e\xa4\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E90F,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xb1",
            upper: b"\xf0\x9e\xa4\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E910,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xb2",
            upper: b"\xf0\x9e\xa4\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E911,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xb3",
            upper: b"\xf0\x9e\xa4\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E912,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xb4",
            upper: b"\xf0\x9e\xa4\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E913,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xb5",
            upper: b"\xf0\x9e\xa4\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E914,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xb6",
            upper: b"\xf0\x9e\xa4\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E915,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xb7",
            upper: b"\xf0\x9e\xa4\x95",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E916,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xb8",
            upper: b"\xf0\x9e\xa4\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E917,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xb9",
            upper: b"\xf0\x9e\xa4\x97",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E918,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xba",
            upper: b"\xf0\x9e\xa4\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E919,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xbb",
            upper: b"\xf0\x9e\xa4\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E91A,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xbc",
            upper: b"\xf0\x9e\xa4\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E91B,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xbd",
            upper: b"\xf0\x9e\xa4\x9b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E91C,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xbe",
            upper: b"\xf0\x9e\xa4\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E91D,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xbf",
            upper: b"\xf0\x9e\xa4\x9d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E91E,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa5\x80",
            upper: b"\xf0\x9e\xa4\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E91F,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa5\x81",
            upper: b"\xf0\x9e\xa4\x9f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E920,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa5\x82",
            upper: b"\xf0\x9e\xa4\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E921,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa5\x83",
            upper: b"\xf0\x9e\xa4\xa1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E922,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xa2",
            upper: b"\xf0\x9e\xa4\x80",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E923,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xa3",
            upper: b"\xf0\x9e\xa4\x81",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E924,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xa4",
            upper: b"\xf0\x9e\xa4\x82",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E925,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xa5",
            upper: b"\xf0\x9e\xa4\x83",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E926,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xa6",
            upper: b"\xf0\x9e\xa4\x84",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E927,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xa7",
            upper: b"\xf0\x9e\xa4\x85",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E928,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xa8",
            upper: b"\xf0\x9e\xa4\x86",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E929,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xa9",
            upper: b"\xf0\x9e\xa4\x87",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E92A,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xaa",
            upper: b"\xf0\x9e\xa4\x88",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E92B,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xab",
            upper: b"\xf0\x9e\xa4\x89",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E92C,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xac",
            upper: b"\xf0\x9e\xa4\x8a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E92D,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xad",
            upper: b"\xf0\x9e\xa4\x8b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E92E,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xae",
            upper: b"\xf0\x9e\xa4\x8c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E92F,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xaf",
            upper: b"\xf0\x9e\xa4\x8d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E930,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xb0",
            upper: b"\xf0\x9e\xa4\x8e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E931,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xb1",
            upper: b"\xf0\x9e\xa4\x8f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E932,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xb2",
            upper: b"\xf0\x9e\xa4\x90",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E933,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xb3",
            upper: b"\xf0\x9e\xa4\x91",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E934,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xb4",
            upper: b"\xf0\x9e\xa4\x92",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E935,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xb5",
            upper: b"\xf0\x9e\xa4\x93",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E936,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xb6",
            upper: b"\xf0\x9e\xa4\x94",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E937,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xb7",
            upper: b"\xf0\x9e\xa4\x95",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E938,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xb8",
            upper: b"\xf0\x9e\xa4\x96",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E939,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xb9",
            upper: b"\xf0\x9e\xa4\x97",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E93A,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xba",
            upper: b"\xf0\x9e\xa4\x98",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E93B,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xbb",
            upper: b"\xf0\x9e\xa4\x99",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E93C,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xbc",
            upper: b"\xf0\x9e\xa4\x9a",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E93D,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xbd",
            upper: b"\xf0\x9e\xa4\x9b",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E93E,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xbe",
            upper: b"\xf0\x9e\xa4\x9c",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E93F,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa4\xbf",
            upper: b"\xf0\x9e\xa4\x9d",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E940,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa5\x80",
            upper: b"\xf0\x9e\xa4\x9e",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E941,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa5\x81",
            upper: b"\xf0\x9e\xa4\x9f",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E942,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa5\x82",
            upper: b"\xf0\x9e\xa4\xa0",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
    (
        0x1E943,
        SpecialCasingMapping {
            lower: b"\xf0\x9e\xa5\x83",
            upper: b"\xf0\x9e\xa4\xa1",
            conditional_lower: b"",
            condition: SpecialCasingCondition::None,
        },
    ),
];

/// `unicodeCasedRanges` from the pinned table.
pub static UNICODE_CASED_RANGES: RangeTable = RangeTable {
    r16: &[
        Range16 {
            lo: 0x0041,
            hi: 0x005A,
            stride: 1,
        },
        Range16 {
            lo: 0x0061,
            hi: 0x007A,
            stride: 1,
        },
        Range16 {
            lo: 0x00AA,
            hi: 0x00B5,
            stride: 11,
        },
        Range16 {
            lo: 0x00BA,
            hi: 0x00C0,
            stride: 6,
        },
        Range16 {
            lo: 0x00C1,
            hi: 0x00D6,
            stride: 1,
        },
        Range16 {
            lo: 0x00D8,
            hi: 0x00F6,
            stride: 1,
        },
        Range16 {
            lo: 0x00F8,
            hi: 0x01BA,
            stride: 1,
        },
        Range16 {
            lo: 0x01BC,
            hi: 0x01BF,
            stride: 1,
        },
        Range16 {
            lo: 0x01C4,
            hi: 0x0293,
            stride: 1,
        },
        Range16 {
            lo: 0x0295,
            hi: 0x02B8,
            stride: 1,
        },
        Range16 {
            lo: 0x02C0,
            hi: 0x02C1,
            stride: 1,
        },
        Range16 {
            lo: 0x02E0,
            hi: 0x02E4,
            stride: 1,
        },
        Range16 {
            lo: 0x0345,
            hi: 0x0370,
            stride: 43,
        },
        Range16 {
            lo: 0x0371,
            hi: 0x0373,
            stride: 1,
        },
        Range16 {
            lo: 0x0376,
            hi: 0x0377,
            stride: 1,
        },
        Range16 {
            lo: 0x037A,
            hi: 0x037D,
            stride: 1,
        },
        Range16 {
            lo: 0x037F,
            hi: 0x0386,
            stride: 7,
        },
        Range16 {
            lo: 0x0388,
            hi: 0x038A,
            stride: 1,
        },
        Range16 {
            lo: 0x038C,
            hi: 0x038E,
            stride: 2,
        },
        Range16 {
            lo: 0x038F,
            hi: 0x03A1,
            stride: 1,
        },
        Range16 {
            lo: 0x03A3,
            hi: 0x03F5,
            stride: 1,
        },
        Range16 {
            lo: 0x03F7,
            hi: 0x0481,
            stride: 1,
        },
        Range16 {
            lo: 0x048A,
            hi: 0x052F,
            stride: 1,
        },
        Range16 {
            lo: 0x0531,
            hi: 0x0556,
            stride: 1,
        },
        Range16 {
            lo: 0x0560,
            hi: 0x0588,
            stride: 1,
        },
        Range16 {
            lo: 0x10A0,
            hi: 0x10C5,
            stride: 1,
        },
        Range16 {
            lo: 0x10C7,
            hi: 0x10CD,
            stride: 6,
        },
        Range16 {
            lo: 0x10D0,
            hi: 0x10FA,
            stride: 1,
        },
        Range16 {
            lo: 0x10FC,
            hi: 0x10FF,
            stride: 1,
        },
        Range16 {
            lo: 0x13A0,
            hi: 0x13F5,
            stride: 1,
        },
        Range16 {
            lo: 0x13F8,
            hi: 0x13FD,
            stride: 1,
        },
        Range16 {
            lo: 0x1C80,
            hi: 0x1C88,
            stride: 1,
        },
        Range16 {
            lo: 0x1C90,
            hi: 0x1CBA,
            stride: 1,
        },
        Range16 {
            lo: 0x1CBD,
            hi: 0x1CBF,
            stride: 1,
        },
        Range16 {
            lo: 0x1D00,
            hi: 0x1DBF,
            stride: 1,
        },
        Range16 {
            lo: 0x1E00,
            hi: 0x1F15,
            stride: 1,
        },
        Range16 {
            lo: 0x1F18,
            hi: 0x1F1D,
            stride: 1,
        },
        Range16 {
            lo: 0x1F20,
            hi: 0x1F45,
            stride: 1,
        },
        Range16 {
            lo: 0x1F48,
            hi: 0x1F4D,
            stride: 1,
        },
        Range16 {
            lo: 0x1F50,
            hi: 0x1F57,
            stride: 1,
        },
        Range16 {
            lo: 0x1F59,
            hi: 0x1F5F,
            stride: 2,
        },
        Range16 {
            lo: 0x1F60,
            hi: 0x1F7D,
            stride: 1,
        },
        Range16 {
            lo: 0x1F80,
            hi: 0x1FB4,
            stride: 1,
        },
        Range16 {
            lo: 0x1FB6,
            hi: 0x1FBC,
            stride: 1,
        },
        Range16 {
            lo: 0x1FBE,
            hi: 0x1FC2,
            stride: 4,
        },
        Range16 {
            lo: 0x1FC3,
            hi: 0x1FC4,
            stride: 1,
        },
        Range16 {
            lo: 0x1FC6,
            hi: 0x1FCC,
            stride: 1,
        },
        Range16 {
            lo: 0x1FD0,
            hi: 0x1FD3,
            stride: 1,
        },
        Range16 {
            lo: 0x1FD6,
            hi: 0x1FDB,
            stride: 1,
        },
        Range16 {
            lo: 0x1FE0,
            hi: 0x1FEC,
            stride: 1,
        },
        Range16 {
            lo: 0x1FF2,
            hi: 0x1FF4,
            stride: 1,
        },
        Range16 {
            lo: 0x1FF6,
            hi: 0x1FFC,
            stride: 1,
        },
        Range16 {
            lo: 0x2071,
            hi: 0x207F,
            stride: 14,
        },
        Range16 {
            lo: 0x2090,
            hi: 0x209C,
            stride: 1,
        },
        Range16 {
            lo: 0x2102,
            hi: 0x2107,
            stride: 5,
        },
        Range16 {
            lo: 0x210A,
            hi: 0x2113,
            stride: 1,
        },
        Range16 {
            lo: 0x2115,
            hi: 0x2119,
            stride: 4,
        },
        Range16 {
            lo: 0x211A,
            hi: 0x211D,
            stride: 1,
        },
        Range16 {
            lo: 0x2124,
            hi: 0x212A,
            stride: 2,
        },
        Range16 {
            lo: 0x212B,
            hi: 0x212D,
            stride: 1,
        },
        Range16 {
            lo: 0x212F,
            hi: 0x2134,
            stride: 1,
        },
        Range16 {
            lo: 0x2139,
            hi: 0x213C,
            stride: 3,
        },
        Range16 {
            lo: 0x213D,
            hi: 0x213F,
            stride: 1,
        },
        Range16 {
            lo: 0x2145,
            hi: 0x2149,
            stride: 1,
        },
        Range16 {
            lo: 0x214E,
            hi: 0x2160,
            stride: 18,
        },
        Range16 {
            lo: 0x2161,
            hi: 0x217F,
            stride: 1,
        },
        Range16 {
            lo: 0x2183,
            hi: 0x2184,
            stride: 1,
        },
        Range16 {
            lo: 0x24B6,
            hi: 0x24E9,
            stride: 1,
        },
        Range16 {
            lo: 0x2C00,
            hi: 0x2CE4,
            stride: 1,
        },
        Range16 {
            lo: 0x2CEB,
            hi: 0x2CEE,
            stride: 1,
        },
        Range16 {
            lo: 0x2CF2,
            hi: 0x2CF3,
            stride: 1,
        },
        Range16 {
            lo: 0x2D00,
            hi: 0x2D25,
            stride: 1,
        },
        Range16 {
            lo: 0x2D27,
            hi: 0x2D2D,
            stride: 6,
        },
        Range16 {
            lo: 0xA640,
            hi: 0xA66D,
            stride: 1,
        },
        Range16 {
            lo: 0xA680,
            hi: 0xA69D,
            stride: 1,
        },
        Range16 {
            lo: 0xA722,
            hi: 0xA787,
            stride: 1,
        },
        Range16 {
            lo: 0xA78B,
            hi: 0xA78E,
            stride: 1,
        },
        Range16 {
            lo: 0xA790,
            hi: 0xA7CA,
            stride: 1,
        },
        Range16 {
            lo: 0xA7D0,
            hi: 0xA7D1,
            stride: 1,
        },
        Range16 {
            lo: 0xA7D3,
            hi: 0xA7D5,
            stride: 2,
        },
        Range16 {
            lo: 0xA7D6,
            hi: 0xA7D9,
            stride: 1,
        },
        Range16 {
            lo: 0xA7F2,
            hi: 0xA7F6,
            stride: 1,
        },
        Range16 {
            lo: 0xA7F8,
            hi: 0xA7FA,
            stride: 1,
        },
        Range16 {
            lo: 0xAB30,
            hi: 0xAB5A,
            stride: 1,
        },
        Range16 {
            lo: 0xAB5C,
            hi: 0xAB69,
            stride: 1,
        },
        Range16 {
            lo: 0xAB70,
            hi: 0xABBF,
            stride: 1,
        },
        Range16 {
            lo: 0xFB00,
            hi: 0xFB06,
            stride: 1,
        },
        Range16 {
            lo: 0xFB13,
            hi: 0xFB17,
            stride: 1,
        },
        Range16 {
            lo: 0xFF21,
            hi: 0xFF3A,
            stride: 1,
        },
        Range16 {
            lo: 0xFF41,
            hi: 0xFF5A,
            stride: 1,
        },
    ],
    r32: &[
        Range32 {
            lo: 0x10400,
            hi: 0x1044F,
            stride: 1,
        },
        Range32 {
            lo: 0x104B0,
            hi: 0x104D3,
            stride: 1,
        },
        Range32 {
            lo: 0x104D8,
            hi: 0x104FB,
            stride: 1,
        },
        Range32 {
            lo: 0x10570,
            hi: 0x1057A,
            stride: 1,
        },
        Range32 {
            lo: 0x1057C,
            hi: 0x1058A,
            stride: 1,
        },
        Range32 {
            lo: 0x1058C,
            hi: 0x10592,
            stride: 1,
        },
        Range32 {
            lo: 0x10594,
            hi: 0x10595,
            stride: 1,
        },
        Range32 {
            lo: 0x10597,
            hi: 0x105A1,
            stride: 1,
        },
        Range32 {
            lo: 0x105A3,
            hi: 0x105B1,
            stride: 1,
        },
        Range32 {
            lo: 0x105B3,
            hi: 0x105B9,
            stride: 1,
        },
        Range32 {
            lo: 0x105BB,
            hi: 0x105BC,
            stride: 1,
        },
        Range32 {
            lo: 0x10780,
            hi: 0x10783,
            stride: 3,
        },
        Range32 {
            lo: 0x10784,
            hi: 0x10785,
            stride: 1,
        },
        Range32 {
            lo: 0x10787,
            hi: 0x107B0,
            stride: 1,
        },
        Range32 {
            lo: 0x107B2,
            hi: 0x107BA,
            stride: 1,
        },
        Range32 {
            lo: 0x10C80,
            hi: 0x10CB2,
            stride: 1,
        },
        Range32 {
            lo: 0x10CC0,
            hi: 0x10CF2,
            stride: 1,
        },
        Range32 {
            lo: 0x118A0,
            hi: 0x118DF,
            stride: 1,
        },
        Range32 {
            lo: 0x16E40,
            hi: 0x16E7F,
            stride: 1,
        },
        Range32 {
            lo: 0x1D400,
            hi: 0x1D454,
            stride: 1,
        },
        Range32 {
            lo: 0x1D456,
            hi: 0x1D49C,
            stride: 1,
        },
        Range32 {
            lo: 0x1D49E,
            hi: 0x1D49F,
            stride: 1,
        },
        Range32 {
            lo: 0x1D4A2,
            hi: 0x1D4A5,
            stride: 3,
        },
        Range32 {
            lo: 0x1D4A6,
            hi: 0x1D4A9,
            stride: 3,
        },
        Range32 {
            lo: 0x1D4AA,
            hi: 0x1D4AC,
            stride: 1,
        },
        Range32 {
            lo: 0x1D4AE,
            hi: 0x1D4B9,
            stride: 1,
        },
        Range32 {
            lo: 0x1D4BB,
            hi: 0x1D4BD,
            stride: 2,
        },
        Range32 {
            lo: 0x1D4BE,
            hi: 0x1D4C3,
            stride: 1,
        },
        Range32 {
            lo: 0x1D4C5,
            hi: 0x1D505,
            stride: 1,
        },
        Range32 {
            lo: 0x1D507,
            hi: 0x1D50A,
            stride: 1,
        },
        Range32 {
            lo: 0x1D50D,
            hi: 0x1D514,
            stride: 1,
        },
        Range32 {
            lo: 0x1D516,
            hi: 0x1D51C,
            stride: 1,
        },
        Range32 {
            lo: 0x1D51E,
            hi: 0x1D539,
            stride: 1,
        },
        Range32 {
            lo: 0x1D53B,
            hi: 0x1D53E,
            stride: 1,
        },
        Range32 {
            lo: 0x1D540,
            hi: 0x1D544,
            stride: 1,
        },
        Range32 {
            lo: 0x1D546,
            hi: 0x1D54A,
            stride: 4,
        },
        Range32 {
            lo: 0x1D54B,
            hi: 0x1D550,
            stride: 1,
        },
        Range32 {
            lo: 0x1D552,
            hi: 0x1D6A5,
            stride: 1,
        },
        Range32 {
            lo: 0x1D6A8,
            hi: 0x1D6C0,
            stride: 1,
        },
        Range32 {
            lo: 0x1D6C2,
            hi: 0x1D6DA,
            stride: 1,
        },
        Range32 {
            lo: 0x1D6DC,
            hi: 0x1D6FA,
            stride: 1,
        },
        Range32 {
            lo: 0x1D6FC,
            hi: 0x1D714,
            stride: 1,
        },
        Range32 {
            lo: 0x1D716,
            hi: 0x1D734,
            stride: 1,
        },
        Range32 {
            lo: 0x1D736,
            hi: 0x1D74E,
            stride: 1,
        },
        Range32 {
            lo: 0x1D750,
            hi: 0x1D76E,
            stride: 1,
        },
        Range32 {
            lo: 0x1D770,
            hi: 0x1D788,
            stride: 1,
        },
        Range32 {
            lo: 0x1D78A,
            hi: 0x1D7A8,
            stride: 1,
        },
        Range32 {
            lo: 0x1D7AA,
            hi: 0x1D7C2,
            stride: 1,
        },
        Range32 {
            lo: 0x1D7C4,
            hi: 0x1D7CB,
            stride: 1,
        },
        Range32 {
            lo: 0x1DF00,
            hi: 0x1DF09,
            stride: 1,
        },
        Range32 {
            lo: 0x1DF0B,
            hi: 0x1DF1E,
            stride: 1,
        },
        Range32 {
            lo: 0x1DF25,
            hi: 0x1DF2A,
            stride: 1,
        },
        Range32 {
            lo: 0x1E030,
            hi: 0x1E06D,
            stride: 1,
        },
        Range32 {
            lo: 0x1E900,
            hi: 0x1E943,
            stride: 1,
        },
        Range32 {
            lo: 0x1F130,
            hi: 0x1F149,
            stride: 1,
        },
        Range32 {
            lo: 0x1F150,
            hi: 0x1F169,
            stride: 1,
        },
        Range32 {
            lo: 0x1F170,
            hi: 0x1F189,
            stride: 1,
        },
    ],
    latin_offset: 6,
};

/// `unicodeCaseIgnorableRanges` from the pinned table.
pub static UNICODE_CASE_IGNORABLE_RANGES: RangeTable = RangeTable {
    r16: &[
        Range16 {
            lo: 0x0027,
            hi: 0x002E,
            stride: 7,
        },
        Range16 {
            lo: 0x003A,
            hi: 0x005E,
            stride: 36,
        },
        Range16 {
            lo: 0x0060,
            hi: 0x00A8,
            stride: 72,
        },
        Range16 {
            lo: 0x00AD,
            hi: 0x00AF,
            stride: 2,
        },
        Range16 {
            lo: 0x00B4,
            hi: 0x00B7,
            stride: 3,
        },
        Range16 {
            lo: 0x00B8,
            hi: 0x02B0,
            stride: 504,
        },
        Range16 {
            lo: 0x02B1,
            hi: 0x036F,
            stride: 1,
        },
        Range16 {
            lo: 0x0374,
            hi: 0x0375,
            stride: 1,
        },
        Range16 {
            lo: 0x037A,
            hi: 0x0384,
            stride: 10,
        },
        Range16 {
            lo: 0x0385,
            hi: 0x0387,
            stride: 2,
        },
        Range16 {
            lo: 0x0483,
            hi: 0x0489,
            stride: 1,
        },
        Range16 {
            lo: 0x0559,
            hi: 0x055F,
            stride: 6,
        },
        Range16 {
            lo: 0x0591,
            hi: 0x05BD,
            stride: 1,
        },
        Range16 {
            lo: 0x05BF,
            hi: 0x05C1,
            stride: 2,
        },
        Range16 {
            lo: 0x05C2,
            hi: 0x05C4,
            stride: 2,
        },
        Range16 {
            lo: 0x05C5,
            hi: 0x05C7,
            stride: 2,
        },
        Range16 {
            lo: 0x05F4,
            hi: 0x0600,
            stride: 12,
        },
        Range16 {
            lo: 0x0601,
            hi: 0x0605,
            stride: 1,
        },
        Range16 {
            lo: 0x0610,
            hi: 0x061A,
            stride: 1,
        },
        Range16 {
            lo: 0x061C,
            hi: 0x0640,
            stride: 36,
        },
        Range16 {
            lo: 0x064B,
            hi: 0x065F,
            stride: 1,
        },
        Range16 {
            lo: 0x0670,
            hi: 0x06D6,
            stride: 102,
        },
        Range16 {
            lo: 0x06D7,
            hi: 0x06DD,
            stride: 1,
        },
        Range16 {
            lo: 0x06DF,
            hi: 0x06E8,
            stride: 1,
        },
        Range16 {
            lo: 0x06EA,
            hi: 0x06ED,
            stride: 1,
        },
        Range16 {
            lo: 0x070F,
            hi: 0x0711,
            stride: 2,
        },
        Range16 {
            lo: 0x0730,
            hi: 0x074A,
            stride: 1,
        },
        Range16 {
            lo: 0x07A6,
            hi: 0x07B0,
            stride: 1,
        },
        Range16 {
            lo: 0x07EB,
            hi: 0x07F5,
            stride: 1,
        },
        Range16 {
            lo: 0x07FA,
            hi: 0x07FD,
            stride: 3,
        },
        Range16 {
            lo: 0x0816,
            hi: 0x082D,
            stride: 1,
        },
        Range16 {
            lo: 0x0859,
            hi: 0x085B,
            stride: 1,
        },
        Range16 {
            lo: 0x0888,
            hi: 0x0890,
            stride: 8,
        },
        Range16 {
            lo: 0x0891,
            hi: 0x0898,
            stride: 7,
        },
        Range16 {
            lo: 0x0899,
            hi: 0x089F,
            stride: 1,
        },
        Range16 {
            lo: 0x08C9,
            hi: 0x0902,
            stride: 1,
        },
        Range16 {
            lo: 0x093A,
            hi: 0x093C,
            stride: 2,
        },
        Range16 {
            lo: 0x0941,
            hi: 0x0948,
            stride: 1,
        },
        Range16 {
            lo: 0x094D,
            hi: 0x0951,
            stride: 4,
        },
        Range16 {
            lo: 0x0952,
            hi: 0x0957,
            stride: 1,
        },
        Range16 {
            lo: 0x0962,
            hi: 0x0963,
            stride: 1,
        },
        Range16 {
            lo: 0x0971,
            hi: 0x0981,
            stride: 16,
        },
        Range16 {
            lo: 0x09BC,
            hi: 0x09C1,
            stride: 5,
        },
        Range16 {
            lo: 0x09C2,
            hi: 0x09C4,
            stride: 1,
        },
        Range16 {
            lo: 0x09CD,
            hi: 0x09E2,
            stride: 21,
        },
        Range16 {
            lo: 0x09E3,
            hi: 0x09FE,
            stride: 27,
        },
        Range16 {
            lo: 0x0A01,
            hi: 0x0A02,
            stride: 1,
        },
        Range16 {
            lo: 0x0A3C,
            hi: 0x0A41,
            stride: 5,
        },
        Range16 {
            lo: 0x0A42,
            hi: 0x0A47,
            stride: 5,
        },
        Range16 {
            lo: 0x0A48,
            hi: 0x0A4B,
            stride: 3,
        },
        Range16 {
            lo: 0x0A4C,
            hi: 0x0A4D,
            stride: 1,
        },
        Range16 {
            lo: 0x0A51,
            hi: 0x0A70,
            stride: 31,
        },
        Range16 {
            lo: 0x0A71,
            hi: 0x0A75,
            stride: 4,
        },
        Range16 {
            lo: 0x0A81,
            hi: 0x0A82,
            stride: 1,
        },
        Range16 {
            lo: 0x0ABC,
            hi: 0x0AC1,
            stride: 5,
        },
        Range16 {
            lo: 0x0AC2,
            hi: 0x0AC5,
            stride: 1,
        },
        Range16 {
            lo: 0x0AC7,
            hi: 0x0AC8,
            stride: 1,
        },
        Range16 {
            lo: 0x0ACD,
            hi: 0x0AE2,
            stride: 21,
        },
        Range16 {
            lo: 0x0AE3,
            hi: 0x0AFA,
            stride: 23,
        },
        Range16 {
            lo: 0x0AFB,
            hi: 0x0AFF,
            stride: 1,
        },
        Range16 {
            lo: 0x0B01,
            hi: 0x0B3C,
            stride: 59,
        },
        Range16 {
            lo: 0x0B3F,
            hi: 0x0B41,
            stride: 2,
        },
        Range16 {
            lo: 0x0B42,
            hi: 0x0B44,
            stride: 1,
        },
        Range16 {
            lo: 0x0B4D,
            hi: 0x0B55,
            stride: 8,
        },
        Range16 {
            lo: 0x0B56,
            hi: 0x0B62,
            stride: 12,
        },
        Range16 {
            lo: 0x0B63,
            hi: 0x0B82,
            stride: 31,
        },
        Range16 {
            lo: 0x0BC0,
            hi: 0x0BCD,
            stride: 13,
        },
        Range16 {
            lo: 0x0C00,
            hi: 0x0C04,
            stride: 4,
        },
        Range16 {
            lo: 0x0C3C,
            hi: 0x0C3E,
            stride: 2,
        },
        Range16 {
            lo: 0x0C3F,
            hi: 0x0C40,
            stride: 1,
        },
        Range16 {
            lo: 0x0C46,
            hi: 0x0C48,
            stride: 1,
        },
        Range16 {
            lo: 0x0C4A,
            hi: 0x0C4D,
            stride: 1,
        },
        Range16 {
            lo: 0x0C55,
            hi: 0x0C56,
            stride: 1,
        },
        Range16 {
            lo: 0x0C62,
            hi: 0x0C63,
            stride: 1,
        },
        Range16 {
            lo: 0x0C81,
            hi: 0x0CBC,
            stride: 59,
        },
        Range16 {
            lo: 0x0CBF,
            hi: 0x0CC6,
            stride: 7,
        },
        Range16 {
            lo: 0x0CCC,
            hi: 0x0CCD,
            stride: 1,
        },
        Range16 {
            lo: 0x0CE2,
            hi: 0x0CE3,
            stride: 1,
        },
        Range16 {
            lo: 0x0D00,
            hi: 0x0D01,
            stride: 1,
        },
        Range16 {
            lo: 0x0D3B,
            hi: 0x0D3C,
            stride: 1,
        },
        Range16 {
            lo: 0x0D41,
            hi: 0x0D44,
            stride: 1,
        },
        Range16 {
            lo: 0x0D4D,
            hi: 0x0D62,
            stride: 21,
        },
        Range16 {
            lo: 0x0D63,
            hi: 0x0D81,
            stride: 30,
        },
        Range16 {
            lo: 0x0DCA,
            hi: 0x0DD2,
            stride: 8,
        },
        Range16 {
            lo: 0x0DD3,
            hi: 0x0DD4,
            stride: 1,
        },
        Range16 {
            lo: 0x0DD6,
            hi: 0x0E31,
            stride: 91,
        },
        Range16 {
            lo: 0x0E34,
            hi: 0x0E3A,
            stride: 1,
        },
        Range16 {
            lo: 0x0E46,
            hi: 0x0E4E,
            stride: 1,
        },
        Range16 {
            lo: 0x0EB1,
            hi: 0x0EB4,
            stride: 3,
        },
        Range16 {
            lo: 0x0EB5,
            hi: 0x0EBC,
            stride: 1,
        },
        Range16 {
            lo: 0x0EC6,
            hi: 0x0EC8,
            stride: 2,
        },
        Range16 {
            lo: 0x0EC9,
            hi: 0x0ECE,
            stride: 1,
        },
        Range16 {
            lo: 0x0F18,
            hi: 0x0F19,
            stride: 1,
        },
        Range16 {
            lo: 0x0F35,
            hi: 0x0F39,
            stride: 2,
        },
        Range16 {
            lo: 0x0F71,
            hi: 0x0F7E,
            stride: 1,
        },
        Range16 {
            lo: 0x0F80,
            hi: 0x0F84,
            stride: 1,
        },
        Range16 {
            lo: 0x0F86,
            hi: 0x0F87,
            stride: 1,
        },
        Range16 {
            lo: 0x0F8D,
            hi: 0x0F97,
            stride: 1,
        },
        Range16 {
            lo: 0x0F99,
            hi: 0x0FBC,
            stride: 1,
        },
        Range16 {
            lo: 0x0FC6,
            hi: 0x102D,
            stride: 103,
        },
        Range16 {
            lo: 0x102E,
            hi: 0x1030,
            stride: 1,
        },
        Range16 {
            lo: 0x1032,
            hi: 0x1037,
            stride: 1,
        },
        Range16 {
            lo: 0x1039,
            hi: 0x103A,
            stride: 1,
        },
        Range16 {
            lo: 0x103D,
            hi: 0x103E,
            stride: 1,
        },
        Range16 {
            lo: 0x1058,
            hi: 0x1059,
            stride: 1,
        },
        Range16 {
            lo: 0x105E,
            hi: 0x1060,
            stride: 1,
        },
        Range16 {
            lo: 0x1071,
            hi: 0x1074,
            stride: 1,
        },
        Range16 {
            lo: 0x1082,
            hi: 0x1085,
            stride: 3,
        },
        Range16 {
            lo: 0x1086,
            hi: 0x108D,
            stride: 7,
        },
        Range16 {
            lo: 0x109D,
            hi: 0x10FC,
            stride: 95,
        },
        Range16 {
            lo: 0x135D,
            hi: 0x135F,
            stride: 1,
        },
        Range16 {
            lo: 0x1712,
            hi: 0x1714,
            stride: 1,
        },
        Range16 {
            lo: 0x1732,
            hi: 0x1733,
            stride: 1,
        },
        Range16 {
            lo: 0x1752,
            hi: 0x1753,
            stride: 1,
        },
        Range16 {
            lo: 0x1772,
            hi: 0x1773,
            stride: 1,
        },
        Range16 {
            lo: 0x17B4,
            hi: 0x17B5,
            stride: 1,
        },
        Range16 {
            lo: 0x17B7,
            hi: 0x17BD,
            stride: 1,
        },
        Range16 {
            lo: 0x17C6,
            hi: 0x17C9,
            stride: 3,
        },
        Range16 {
            lo: 0x17CA,
            hi: 0x17D3,
            stride: 1,
        },
        Range16 {
            lo: 0x17D7,
            hi: 0x17DD,
            stride: 6,
        },
        Range16 {
            lo: 0x180B,
            hi: 0x180F,
            stride: 1,
        },
        Range16 {
            lo: 0x1843,
            hi: 0x1885,
            stride: 66,
        },
        Range16 {
            lo: 0x1886,
            hi: 0x18A9,
            stride: 35,
        },
        Range16 {
            lo: 0x1920,
            hi: 0x1922,
            stride: 1,
        },
        Range16 {
            lo: 0x1927,
            hi: 0x1928,
            stride: 1,
        },
        Range16 {
            lo: 0x1932,
            hi: 0x1939,
            stride: 7,
        },
        Range16 {
            lo: 0x193A,
            hi: 0x193B,
            stride: 1,
        },
        Range16 {
            lo: 0x1A17,
            hi: 0x1A18,
            stride: 1,
        },
        Range16 {
            lo: 0x1A1B,
            hi: 0x1A56,
            stride: 59,
        },
        Range16 {
            lo: 0x1A58,
            hi: 0x1A5E,
            stride: 1,
        },
        Range16 {
            lo: 0x1A60,
            hi: 0x1A62,
            stride: 2,
        },
        Range16 {
            lo: 0x1A65,
            hi: 0x1A6C,
            stride: 1,
        },
        Range16 {
            lo: 0x1A73,
            hi: 0x1A7C,
            stride: 1,
        },
        Range16 {
            lo: 0x1A7F,
            hi: 0x1AA7,
            stride: 40,
        },
        Range16 {
            lo: 0x1AB0,
            hi: 0x1ACE,
            stride: 1,
        },
        Range16 {
            lo: 0x1B00,
            hi: 0x1B03,
            stride: 1,
        },
        Range16 {
            lo: 0x1B34,
            hi: 0x1B36,
            stride: 2,
        },
        Range16 {
            lo: 0x1B37,
            hi: 0x1B3A,
            stride: 1,
        },
        Range16 {
            lo: 0x1B3C,
            hi: 0x1B42,
            stride: 6,
        },
        Range16 {
            lo: 0x1B6B,
            hi: 0x1B73,
            stride: 1,
        },
        Range16 {
            lo: 0x1B80,
            hi: 0x1B81,
            stride: 1,
        },
        Range16 {
            lo: 0x1BA2,
            hi: 0x1BA5,
            stride: 1,
        },
        Range16 {
            lo: 0x1BA8,
            hi: 0x1BA9,
            stride: 1,
        },
        Range16 {
            lo: 0x1BAB,
            hi: 0x1BAD,
            stride: 1,
        },
        Range16 {
            lo: 0x1BE6,
            hi: 0x1BE8,
            stride: 2,
        },
        Range16 {
            lo: 0x1BE9,
            hi: 0x1BED,
            stride: 4,
        },
        Range16 {
            lo: 0x1BEF,
            hi: 0x1BF1,
            stride: 1,
        },
        Range16 {
            lo: 0x1C2C,
            hi: 0x1C33,
            stride: 1,
        },
        Range16 {
            lo: 0x1C36,
            hi: 0x1C37,
            stride: 1,
        },
        Range16 {
            lo: 0x1C78,
            hi: 0x1C7D,
            stride: 1,
        },
        Range16 {
            lo: 0x1CD0,
            hi: 0x1CD2,
            stride: 1,
        },
        Range16 {
            lo: 0x1CD4,
            hi: 0x1CE0,
            stride: 1,
        },
        Range16 {
            lo: 0x1CE2,
            hi: 0x1CE8,
            stride: 1,
        },
        Range16 {
            lo: 0x1CED,
            hi: 0x1CF4,
            stride: 7,
        },
        Range16 {
            lo: 0x1CF8,
            hi: 0x1CF9,
            stride: 1,
        },
        Range16 {
            lo: 0x1D2C,
            hi: 0x1D6A,
            stride: 1,
        },
        Range16 {
            lo: 0x1D78,
            hi: 0x1D9B,
            stride: 35,
        },
        Range16 {
            lo: 0x1D9C,
            hi: 0x1DFF,
            stride: 1,
        },
        Range16 {
            lo: 0x1FBD,
            hi: 0x1FBF,
            stride: 2,
        },
        Range16 {
            lo: 0x1FC0,
            hi: 0x1FC1,
            stride: 1,
        },
        Range16 {
            lo: 0x1FCD,
            hi: 0x1FCF,
            stride: 1,
        },
        Range16 {
            lo: 0x1FDD,
            hi: 0x1FDF,
            stride: 1,
        },
        Range16 {
            lo: 0x1FED,
            hi: 0x1FEF,
            stride: 1,
        },
        Range16 {
            lo: 0x1FFD,
            hi: 0x1FFE,
            stride: 1,
        },
        Range16 {
            lo: 0x200B,
            hi: 0x200F,
            stride: 1,
        },
        Range16 {
            lo: 0x2018,
            hi: 0x2019,
            stride: 1,
        },
        Range16 {
            lo: 0x2024,
            hi: 0x202A,
            stride: 3,
        },
        Range16 {
            lo: 0x202B,
            hi: 0x202E,
            stride: 1,
        },
        Range16 {
            lo: 0x2060,
            hi: 0x2064,
            stride: 1,
        },
        Range16 {
            lo: 0x2066,
            hi: 0x206F,
            stride: 1,
        },
        Range16 {
            lo: 0x2071,
            hi: 0x207F,
            stride: 14,
        },
        Range16 {
            lo: 0x2090,
            hi: 0x209C,
            stride: 1,
        },
        Range16 {
            lo: 0x20D0,
            hi: 0x20F0,
            stride: 1,
        },
        Range16 {
            lo: 0x2C7C,
            hi: 0x2C7D,
            stride: 1,
        },
        Range16 {
            lo: 0x2CEF,
            hi: 0x2CF1,
            stride: 1,
        },
        Range16 {
            lo: 0x2D6F,
            hi: 0x2D7F,
            stride: 16,
        },
        Range16 {
            lo: 0x2DE0,
            hi: 0x2DFF,
            stride: 1,
        },
        Range16 {
            lo: 0x2E2F,
            hi: 0x3005,
            stride: 470,
        },
        Range16 {
            lo: 0x302A,
            hi: 0x302D,
            stride: 1,
        },
        Range16 {
            lo: 0x3031,
            hi: 0x3035,
            stride: 1,
        },
        Range16 {
            lo: 0x303B,
            hi: 0x3099,
            stride: 94,
        },
        Range16 {
            lo: 0x309A,
            hi: 0x309E,
            stride: 1,
        },
        Range16 {
            lo: 0x30FC,
            hi: 0x30FE,
            stride: 1,
        },
        Range16 {
            lo: 0xA015,
            hi: 0xA4F8,
            stride: 1251,
        },
        Range16 {
            lo: 0xA4F9,
            hi: 0xA4FD,
            stride: 1,
        },
        Range16 {
            lo: 0xA60C,
            hi: 0xA66F,
            stride: 99,
        },
        Range16 {
            lo: 0xA670,
            hi: 0xA672,
            stride: 1,
        },
        Range16 {
            lo: 0xA674,
            hi: 0xA67D,
            stride: 1,
        },
        Range16 {
            lo: 0xA67F,
            hi: 0xA69C,
            stride: 29,
        },
        Range16 {
            lo: 0xA69D,
            hi: 0xA69F,
            stride: 1,
        },
        Range16 {
            lo: 0xA6F0,
            hi: 0xA6F1,
            stride: 1,
        },
        Range16 {
            lo: 0xA700,
            hi: 0xA721,
            stride: 1,
        },
        Range16 {
            lo: 0xA770,
            hi: 0xA788,
            stride: 24,
        },
        Range16 {
            lo: 0xA789,
            hi: 0xA78A,
            stride: 1,
        },
        Range16 {
            lo: 0xA7F2,
            hi: 0xA7F4,
            stride: 1,
        },
        Range16 {
            lo: 0xA7F8,
            hi: 0xA7F9,
            stride: 1,
        },
        Range16 {
            lo: 0xA802,
            hi: 0xA806,
            stride: 4,
        },
        Range16 {
            lo: 0xA80B,
            hi: 0xA825,
            stride: 26,
        },
        Range16 {
            lo: 0xA826,
            hi: 0xA82C,
            stride: 6,
        },
        Range16 {
            lo: 0xA8C4,
            hi: 0xA8C5,
            stride: 1,
        },
        Range16 {
            lo: 0xA8E0,
            hi: 0xA8F1,
            stride: 1,
        },
        Range16 {
            lo: 0xA8FF,
            hi: 0xA926,
            stride: 39,
        },
        Range16 {
            lo: 0xA927,
            hi: 0xA92D,
            stride: 1,
        },
        Range16 {
            lo: 0xA947,
            hi: 0xA951,
            stride: 1,
        },
        Range16 {
            lo: 0xA980,
            hi: 0xA982,
            stride: 1,
        },
        Range16 {
            lo: 0xA9B3,
            hi: 0xA9B6,
            stride: 3,
        },
        Range16 {
            lo: 0xA9B7,
            hi: 0xA9B9,
            stride: 1,
        },
        Range16 {
            lo: 0xA9BC,
            hi: 0xA9BD,
            stride: 1,
        },
        Range16 {
            lo: 0xA9CF,
            hi: 0xA9E5,
            stride: 22,
        },
        Range16 {
            lo: 0xA9E6,
            hi: 0xAA29,
            stride: 67,
        },
        Range16 {
            lo: 0xAA2A,
            hi: 0xAA2E,
            stride: 1,
        },
        Range16 {
            lo: 0xAA31,
            hi: 0xAA32,
            stride: 1,
        },
        Range16 {
            lo: 0xAA35,
            hi: 0xAA36,
            stride: 1,
        },
        Range16 {
            lo: 0xAA43,
            hi: 0xAA4C,
            stride: 9,
        },
        Range16 {
            lo: 0xAA70,
            hi: 0xAA7C,
            stride: 12,
        },
        Range16 {
            lo: 0xAAB0,
            hi: 0xAAB2,
            stride: 2,
        },
        Range16 {
            lo: 0xAAB3,
            hi: 0xAAB4,
            stride: 1,
        },
        Range16 {
            lo: 0xAAB7,
            hi: 0xAAB8,
            stride: 1,
        },
        Range16 {
            lo: 0xAABE,
            hi: 0xAABF,
            stride: 1,
        },
        Range16 {
            lo: 0xAAC1,
            hi: 0xAADD,
            stride: 28,
        },
        Range16 {
            lo: 0xAAEC,
            hi: 0xAAED,
            stride: 1,
        },
        Range16 {
            lo: 0xAAF3,
            hi: 0xAAF4,
            stride: 1,
        },
        Range16 {
            lo: 0xAAF6,
            hi: 0xAB5B,
            stride: 101,
        },
        Range16 {
            lo: 0xAB5C,
            hi: 0xAB5F,
            stride: 1,
        },
        Range16 {
            lo: 0xAB69,
            hi: 0xAB6B,
            stride: 1,
        },
        Range16 {
            lo: 0xABE5,
            hi: 0xABE8,
            stride: 3,
        },
        Range16 {
            lo: 0xABED,
            hi: 0xFB1E,
            stride: 20273,
        },
        Range16 {
            lo: 0xFBB2,
            hi: 0xFBC2,
            stride: 1,
        },
        Range16 {
            lo: 0xFE00,
            hi: 0xFE0F,
            stride: 1,
        },
        Range16 {
            lo: 0xFE13,
            hi: 0xFE20,
            stride: 13,
        },
        Range16 {
            lo: 0xFE21,
            hi: 0xFE2F,
            stride: 1,
        },
        Range16 {
            lo: 0xFE52,
            hi: 0xFE55,
            stride: 3,
        },
        Range16 {
            lo: 0xFEFF,
            hi: 0xFF07,
            stride: 8,
        },
        Range16 {
            lo: 0xFF0E,
            hi: 0xFF1A,
            stride: 12,
        },
        Range16 {
            lo: 0xFF3E,
            hi: 0xFF40,
            stride: 2,
        },
        Range16 {
            lo: 0xFF70,
            hi: 0xFF9E,
            stride: 46,
        },
        Range16 {
            lo: 0xFF9F,
            hi: 0xFFE3,
            stride: 68,
        },
        Range16 {
            lo: 0xFFF9,
            hi: 0xFFFB,
            stride: 1,
        },
    ],
    r32: &[
        Range32 {
            lo: 0x101FD,
            hi: 0x102E0,
            stride: 227,
        },
        Range32 {
            lo: 0x10376,
            hi: 0x1037A,
            stride: 1,
        },
        Range32 {
            lo: 0x10780,
            hi: 0x10785,
            stride: 1,
        },
        Range32 {
            lo: 0x10787,
            hi: 0x107B0,
            stride: 1,
        },
        Range32 {
            lo: 0x107B2,
            hi: 0x107BA,
            stride: 1,
        },
        Range32 {
            lo: 0x10A01,
            hi: 0x10A03,
            stride: 1,
        },
        Range32 {
            lo: 0x10A05,
            hi: 0x10A06,
            stride: 1,
        },
        Range32 {
            lo: 0x10A0C,
            hi: 0x10A0F,
            stride: 1,
        },
        Range32 {
            lo: 0x10A38,
            hi: 0x10A3A,
            stride: 1,
        },
        Range32 {
            lo: 0x10A3F,
            hi: 0x10AE5,
            stride: 166,
        },
        Range32 {
            lo: 0x10AE6,
            hi: 0x10D24,
            stride: 574,
        },
        Range32 {
            lo: 0x10D25,
            hi: 0x10D27,
            stride: 1,
        },
        Range32 {
            lo: 0x10EAB,
            hi: 0x10EAC,
            stride: 1,
        },
        Range32 {
            lo: 0x10EFD,
            hi: 0x10EFF,
            stride: 1,
        },
        Range32 {
            lo: 0x10F46,
            hi: 0x10F50,
            stride: 1,
        },
        Range32 {
            lo: 0x10F82,
            hi: 0x10F85,
            stride: 1,
        },
        Range32 {
            lo: 0x11001,
            hi: 0x11038,
            stride: 55,
        },
        Range32 {
            lo: 0x11039,
            hi: 0x11046,
            stride: 1,
        },
        Range32 {
            lo: 0x11070,
            hi: 0x11073,
            stride: 3,
        },
        Range32 {
            lo: 0x11074,
            hi: 0x1107F,
            stride: 11,
        },
        Range32 {
            lo: 0x11080,
            hi: 0x11081,
            stride: 1,
        },
        Range32 {
            lo: 0x110B3,
            hi: 0x110B6,
            stride: 1,
        },
        Range32 {
            lo: 0x110B9,
            hi: 0x110BA,
            stride: 1,
        },
        Range32 {
            lo: 0x110BD,
            hi: 0x110C2,
            stride: 5,
        },
        Range32 {
            lo: 0x110CD,
            hi: 0x11100,
            stride: 51,
        },
        Range32 {
            lo: 0x11101,
            hi: 0x11102,
            stride: 1,
        },
        Range32 {
            lo: 0x11127,
            hi: 0x1112B,
            stride: 1,
        },
        Range32 {
            lo: 0x1112D,
            hi: 0x11134,
            stride: 1,
        },
        Range32 {
            lo: 0x11173,
            hi: 0x11180,
            stride: 13,
        },
        Range32 {
            lo: 0x11181,
            hi: 0x111B6,
            stride: 53,
        },
        Range32 {
            lo: 0x111B7,
            hi: 0x111BE,
            stride: 1,
        },
        Range32 {
            lo: 0x111C9,
            hi: 0x111CC,
            stride: 1,
        },
        Range32 {
            lo: 0x111CF,
            hi: 0x1122F,
            stride: 96,
        },
        Range32 {
            lo: 0x11230,
            hi: 0x11231,
            stride: 1,
        },
        Range32 {
            lo: 0x11234,
            hi: 0x11236,
            stride: 2,
        },
        Range32 {
            lo: 0x11237,
            hi: 0x1123E,
            stride: 7,
        },
        Range32 {
            lo: 0x11241,
            hi: 0x112DF,
            stride: 158,
        },
        Range32 {
            lo: 0x112E3,
            hi: 0x112EA,
            stride: 1,
        },
        Range32 {
            lo: 0x11300,
            hi: 0x11301,
            stride: 1,
        },
        Range32 {
            lo: 0x1133B,
            hi: 0x1133C,
            stride: 1,
        },
        Range32 {
            lo: 0x11340,
            hi: 0x11366,
            stride: 38,
        },
        Range32 {
            lo: 0x11367,
            hi: 0x1136C,
            stride: 1,
        },
        Range32 {
            lo: 0x11370,
            hi: 0x11374,
            stride: 1,
        },
        Range32 {
            lo: 0x11438,
            hi: 0x1143F,
            stride: 1,
        },
        Range32 {
            lo: 0x11442,
            hi: 0x11444,
            stride: 1,
        },
        Range32 {
            lo: 0x11446,
            hi: 0x1145E,
            stride: 24,
        },
        Range32 {
            lo: 0x114B3,
            hi: 0x114B8,
            stride: 1,
        },
        Range32 {
            lo: 0x114BA,
            hi: 0x114BF,
            stride: 5,
        },
        Range32 {
            lo: 0x114C0,
            hi: 0x114C2,
            stride: 2,
        },
        Range32 {
            lo: 0x114C3,
            hi: 0x115B2,
            stride: 239,
        },
        Range32 {
            lo: 0x115B3,
            hi: 0x115B5,
            stride: 1,
        },
        Range32 {
            lo: 0x115BC,
            hi: 0x115BD,
            stride: 1,
        },
        Range32 {
            lo: 0x115BF,
            hi: 0x115C0,
            stride: 1,
        },
        Range32 {
            lo: 0x115DC,
            hi: 0x115DD,
            stride: 1,
        },
        Range32 {
            lo: 0x11633,
            hi: 0x1163A,
            stride: 1,
        },
        Range32 {
            lo: 0x1163D,
            hi: 0x1163F,
            stride: 2,
        },
        Range32 {
            lo: 0x11640,
            hi: 0x116AB,
            stride: 107,
        },
        Range32 {
            lo: 0x116AD,
            hi: 0x116B0,
            stride: 3,
        },
        Range32 {
            lo: 0x116B1,
            hi: 0x116B5,
            stride: 1,
        },
        Range32 {
            lo: 0x116B7,
            hi: 0x1171D,
            stride: 102,
        },
        Range32 {
            lo: 0x1171E,
            hi: 0x1171F,
            stride: 1,
        },
        Range32 {
            lo: 0x11722,
            hi: 0x11725,
            stride: 1,
        },
        Range32 {
            lo: 0x11727,
            hi: 0x1172B,
            stride: 1,
        },
        Range32 {
            lo: 0x1182F,
            hi: 0x11837,
            stride: 1,
        },
        Range32 {
            lo: 0x11839,
            hi: 0x1183A,
            stride: 1,
        },
        Range32 {
            lo: 0x1193B,
            hi: 0x1193C,
            stride: 1,
        },
        Range32 {
            lo: 0x1193E,
            hi: 0x11943,
            stride: 5,
        },
        Range32 {
            lo: 0x119D4,
            hi: 0x119D7,
            stride: 1,
        },
        Range32 {
            lo: 0x119DA,
            hi: 0x119DB,
            stride: 1,
        },
        Range32 {
            lo: 0x119E0,
            hi: 0x11A01,
            stride: 33,
        },
        Range32 {
            lo: 0x11A02,
            hi: 0x11A0A,
            stride: 1,
        },
        Range32 {
            lo: 0x11A33,
            hi: 0x11A38,
            stride: 1,
        },
        Range32 {
            lo: 0x11A3B,
            hi: 0x11A3E,
            stride: 1,
        },
        Range32 {
            lo: 0x11A47,
            hi: 0x11A51,
            stride: 10,
        },
        Range32 {
            lo: 0x11A52,
            hi: 0x11A56,
            stride: 1,
        },
        Range32 {
            lo: 0x11A59,
            hi: 0x11A5B,
            stride: 1,
        },
        Range32 {
            lo: 0x11A8A,
            hi: 0x11A96,
            stride: 1,
        },
        Range32 {
            lo: 0x11A98,
            hi: 0x11A99,
            stride: 1,
        },
        Range32 {
            lo: 0x11C30,
            hi: 0x11C36,
            stride: 1,
        },
        Range32 {
            lo: 0x11C38,
            hi: 0x11C3D,
            stride: 1,
        },
        Range32 {
            lo: 0x11C3F,
            hi: 0x11C92,
            stride: 83,
        },
        Range32 {
            lo: 0x11C93,
            hi: 0x11CA7,
            stride: 1,
        },
        Range32 {
            lo: 0x11CAA,
            hi: 0x11CB0,
            stride: 1,
        },
        Range32 {
            lo: 0x11CB2,
            hi: 0x11CB3,
            stride: 1,
        },
        Range32 {
            lo: 0x11CB5,
            hi: 0x11CB6,
            stride: 1,
        },
        Range32 {
            lo: 0x11D31,
            hi: 0x11D36,
            stride: 1,
        },
        Range32 {
            lo: 0x11D3A,
            hi: 0x11D3C,
            stride: 2,
        },
        Range32 {
            lo: 0x11D3D,
            hi: 0x11D3F,
            stride: 2,
        },
        Range32 {
            lo: 0x11D40,
            hi: 0x11D45,
            stride: 1,
        },
        Range32 {
            lo: 0x11D47,
            hi: 0x11D90,
            stride: 73,
        },
        Range32 {
            lo: 0x11D91,
            hi: 0x11D95,
            stride: 4,
        },
        Range32 {
            lo: 0x11D97,
            hi: 0x11EF3,
            stride: 348,
        },
        Range32 {
            lo: 0x11EF4,
            hi: 0x11F00,
            stride: 12,
        },
        Range32 {
            lo: 0x11F01,
            hi: 0x11F36,
            stride: 53,
        },
        Range32 {
            lo: 0x11F37,
            hi: 0x11F3A,
            stride: 1,
        },
        Range32 {
            lo: 0x11F40,
            hi: 0x11F42,
            stride: 2,
        },
        Range32 {
            lo: 0x13430,
            hi: 0x13440,
            stride: 1,
        },
        Range32 {
            lo: 0x13447,
            hi: 0x13455,
            stride: 1,
        },
        Range32 {
            lo: 0x16AF0,
            hi: 0x16AF4,
            stride: 1,
        },
        Range32 {
            lo: 0x16B30,
            hi: 0x16B36,
            stride: 1,
        },
        Range32 {
            lo: 0x16B40,
            hi: 0x16B43,
            stride: 1,
        },
        Range32 {
            lo: 0x16F4F,
            hi: 0x16F8F,
            stride: 64,
        },
        Range32 {
            lo: 0x16F90,
            hi: 0x16F9F,
            stride: 1,
        },
        Range32 {
            lo: 0x16FE0,
            hi: 0x16FE1,
            stride: 1,
        },
        Range32 {
            lo: 0x16FE3,
            hi: 0x16FE4,
            stride: 1,
        },
        Range32 {
            lo: 0x1AFF0,
            hi: 0x1AFF3,
            stride: 1,
        },
        Range32 {
            lo: 0x1AFF5,
            hi: 0x1AFFB,
            stride: 1,
        },
        Range32 {
            lo: 0x1AFFD,
            hi: 0x1AFFE,
            stride: 1,
        },
        Range32 {
            lo: 0x1BC9D,
            hi: 0x1BC9E,
            stride: 1,
        },
        Range32 {
            lo: 0x1BCA0,
            hi: 0x1BCA3,
            stride: 1,
        },
        Range32 {
            lo: 0x1CF00,
            hi: 0x1CF2D,
            stride: 1,
        },
        Range32 {
            lo: 0x1CF30,
            hi: 0x1CF46,
            stride: 1,
        },
        Range32 {
            lo: 0x1D167,
            hi: 0x1D169,
            stride: 1,
        },
        Range32 {
            lo: 0x1D173,
            hi: 0x1D182,
            stride: 1,
        },
        Range32 {
            lo: 0x1D185,
            hi: 0x1D18B,
            stride: 1,
        },
        Range32 {
            lo: 0x1D1AA,
            hi: 0x1D1AD,
            stride: 1,
        },
        Range32 {
            lo: 0x1D242,
            hi: 0x1D244,
            stride: 1,
        },
        Range32 {
            lo: 0x1DA00,
            hi: 0x1DA36,
            stride: 1,
        },
        Range32 {
            lo: 0x1DA3B,
            hi: 0x1DA6C,
            stride: 1,
        },
        Range32 {
            lo: 0x1DA75,
            hi: 0x1DA84,
            stride: 15,
        },
        Range32 {
            lo: 0x1DA9B,
            hi: 0x1DA9F,
            stride: 1,
        },
        Range32 {
            lo: 0x1DAA1,
            hi: 0x1DAAF,
            stride: 1,
        },
        Range32 {
            lo: 0x1E000,
            hi: 0x1E006,
            stride: 1,
        },
        Range32 {
            lo: 0x1E008,
            hi: 0x1E018,
            stride: 1,
        },
        Range32 {
            lo: 0x1E01B,
            hi: 0x1E021,
            stride: 1,
        },
        Range32 {
            lo: 0x1E023,
            hi: 0x1E024,
            stride: 1,
        },
        Range32 {
            lo: 0x1E026,
            hi: 0x1E02A,
            stride: 1,
        },
        Range32 {
            lo: 0x1E030,
            hi: 0x1E06D,
            stride: 1,
        },
        Range32 {
            lo: 0x1E08F,
            hi: 0x1E130,
            stride: 161,
        },
        Range32 {
            lo: 0x1E131,
            hi: 0x1E13D,
            stride: 1,
        },
        Range32 {
            lo: 0x1E2AE,
            hi: 0x1E2EC,
            stride: 62,
        },
        Range32 {
            lo: 0x1E2ED,
            hi: 0x1E2EF,
            stride: 1,
        },
        Range32 {
            lo: 0x1E4EB,
            hi: 0x1E4EF,
            stride: 1,
        },
        Range32 {
            lo: 0x1E8D0,
            hi: 0x1E8D6,
            stride: 1,
        },
        Range32 {
            lo: 0x1E944,
            hi: 0x1E94B,
            stride: 1,
        },
        Range32 {
            lo: 0x1F3FB,
            hi: 0x1F3FF,
            stride: 1,
        },
        Range32 {
            lo: 0xE0001,
            hi: 0xE0020,
            stride: 31,
        },
        Range32 {
            lo: 0xE0021,
            hi: 0xE007F,
            stride: 1,
        },
        Range32 {
            lo: 0xE0100,
            hi: 0xE01EF,
            stride: 1,
        },
    ],
    latin_offset: 5,
};
