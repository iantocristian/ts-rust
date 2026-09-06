//! Per-operation casing, truncation and literal escaping over arbitrary bytes.

use std::borrow::Cow;

use crate::case_tables::{CASED_RANGES, CASE_IGNORABLE_RANGES, CASE_MAPPINGS, SIMPLE_LOWER};
use crate::strings::{append_rune, decode_rune, decode_utf8, is_surrogate, RUNE_ERROR};

/// port: tsc/internal/stringutil/js_case.go:ToLowerJS
pub fn to_lower_js(bytes: &[u8]) -> Cow<'_, [u8]> {
    if let Some(ascii) = to_lower_ascii(bytes) {
        return ascii;
    }
    let mut result = Vec::with_capacity(bytes.len());
    let mut cased_before = false;
    let mut offset = 0;
    while offset < bytes.len() {
        let (rune, width) = decode_rune(&bytes[offset..]);
        offset += width;
        if is_surrogate(rune) {
            append_rune(&mut result, rune);
        } else if let Ok(index) = CASE_MAPPINGS.binary_search_by_key(&rune, |row| row.0) {
            let (_, lower, _, conditional) = CASE_MAPPINGS[index];
            let mapping = match conditional {
                Some(final_sigma) if is_final_sigma_context(cased_before, bytes, offset) => {
                    final_sigma
                }
                _ => lower,
            };
            result.extend_from_slice(mapping.as_bytes());
        } else {
            append_rune(&mut result, rune);
        }
        if !is_unicode_case_ignorable(rune) {
            cased_before = is_sigma_cased(rune);
        }
    }
    Cow::Owned(result)
}

/// port: tsc/internal/stringutil/js_case.go:ToUpperJS
pub fn to_upper_js(bytes: &[u8]) -> Cow<'_, [u8]> {
    if let Some(ascii) = to_upper_ascii(bytes) {
        return ascii;
    }
    let mut result = Vec::with_capacity(bytes.len());
    let mut offset = 0;
    while offset < bytes.len() {
        let (rune, width) = decode_rune(&bytes[offset..]);
        if is_surrogate(rune) {
            result.extend_from_slice(&bytes[offset..offset + width]);
        } else if let Ok(index) = CASE_MAPPINGS.binary_search_by_key(&rune, |row| row.0) {
            result.extend_from_slice(CASE_MAPPINGS[index].2.as_bytes());
        } else {
            append_rune(&mut result, rune);
        }
        offset += width;
    }
    Cow::Owned(result)
}

/// port: tsc/internal/stringutil/js_case.go:toLowerASCII
fn to_lower_ascii(bytes: &[u8]) -> Option<Cow<'_, [u8]>> {
    let mut needs_mapping = false;
    for byte in bytes {
        if !byte.is_ascii() {
            return None;
        }
        needs_mapping |= byte.is_ascii_uppercase();
    }
    Some(if needs_mapping {
        Cow::Owned(bytes.to_ascii_lowercase())
    } else {
        Cow::Borrowed(bytes)
    })
}

/// port: tsc/internal/stringutil/js_case.go:toUpperASCII
fn to_upper_ascii(bytes: &[u8]) -> Option<Cow<'_, [u8]>> {
    let mut needs_mapping = false;
    for byte in bytes {
        if !byte.is_ascii() {
            return None;
        }
        needs_mapping |= byte.is_ascii_lowercase();
    }
    Some(if needs_mapping {
        Cow::Owned(bytes.to_ascii_uppercase())
    } else {
        Cow::Borrowed(bytes)
    })
}

/// port: tsc/internal/stringutil/js_case.go:isFinalSigmaContext
fn is_final_sigma_context(cased_before: bool, bytes: &[u8], after_offset: usize) -> bool {
    cased_before && !has_sigma_cased_after(bytes, after_offset)
}

/// port: tsc/internal/stringutil/js_case.go:hasSigmaCasedAfter
fn has_sigma_cased_after(bytes: &[u8], start: usize) -> bool {
    let mut offset = start;
    while offset < bytes.len() {
        let (rune, width) = decode_rune(&bytes[offset..]);
        offset += width;
        if is_unicode_case_ignorable(rune) {
            continue;
        }
        return is_sigma_cased(rune);
    }
    false
}

fn in_ranges(rune: i32, ranges: &[(i32, i32, i32)]) -> bool {
    let index = ranges.partition_point(|&(_, last, _)| last < rune);
    ranges.get(index).is_some_and(|&(first, last, stride)| {
        rune >= first && rune <= last && (rune - first) % stride == 0
    })
}

/// port: tsc/internal/stringutil/js_case.go:isSigmaCased
fn is_sigma_cased(rune: i32) -> bool {
    in_ranges(rune, CASED_RANGES)
}

/// port: tsc/internal/stringutil/js_case.go:isUnicodeCaseIgnorable
fn is_unicode_case_ignorable(rune: i32) -> bool {
    in_ranges(rune, CASE_IGNORABLE_RANGES)
}

/// port: tsc/internal/stringutil/util.go:LowerFirstChar
pub fn lower_first_char(bytes: &[u8]) -> Vec<u8> {
    let (rune, width) = decode_utf8(bytes);
    if width == 0 {
        return bytes.to_vec();
    }
    let rune = SIMPLE_LOWER
        .binary_search_by_key(&rune, |row| row.0)
        .map_or(rune, |index| SIMPLE_LOWER[index].1);
    let mut result = Vec::with_capacity(bytes.len() + 2);
    append_rune(&mut result, rune);
    result.extend_from_slice(&bytes[width..]);
    result
}

/// port: tsc/internal/stringutil/util.go:TruncateByRunes
pub fn truncate_by_runes(bytes: &[u8], max_length: isize) -> &[u8] {
    if max_length <= 0 {
        return &bytes[..0];
    }
    if bytes.len() < max_length as usize {
        return bytes;
    }
    let mut offset = 0;
    for _ in 0..max_length {
        if offset == bytes.len() {
            break;
        }
        offset += decode_utf8(&bytes[offset..]).1;
    }
    &bytes[..offset]
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum QuoteChar {
    Single = b'\'',
    Double = b'"',
    Backtick = b'`',
}

/// The first two flags affect this leaf worker; the other two are consumed by
/// the future original-source/printer integration and are inert in the worker.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LiteralEscapeFlags(u8);

impl LiteralEscapeFlags {
    pub const NONE: Self = Self(0);
    pub const NEVER_ASCII_ESCAPE: Self = Self(1);
    pub const JSX_ATTRIBUTE_ESCAPE: Self = Self(2);
    pub const TERMINATE_UNTERMINATED_LITERALS: Self = Self(4);
    pub const ALLOW_NUMERIC_SEPARATOR: Self = Self(8);

    pub fn from_bits(bits: u8) -> Option<Self> {
        (bits & !15 == 0).then_some(Self(bits))
    }
    pub fn bits(self) -> u8 {
        self.0
    }
    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl std::ops::BitOr for LiteralEscapeFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// port: tsc/internal/printer/utilities.go:encodeJsxCharacterEntity
fn encode_jsx_character_entity(output: &mut Vec<u8>, rune: i32) {
    output.extend_from_slice(format!("&#x{rune:X};").as_bytes());
}

/// port: tsc/internal/printer/utilities.go:encodeUtf16EscapeSequence
fn encode_utf16_escape_sequence(output: &mut Vec<u8>, rune: i32) {
    output.extend_from_slice(format!("\\u{rune:04X}").as_bytes());
}

fn canonical_escape(rune: i32) -> Option<&'static [u8]> {
    match rune {
        9 => Some(br"\t"),
        11 => Some(br"\v"),
        12 => Some(br"\f"),
        8 => Some(br"\b"),
        13 => Some(br"\r"),
        10 => Some(br"\n"),
        92 => Some(br"\\"),
        34 => Some(b"\\\""),
        39 => Some(br"\'"),
        96 => Some(br"\`"),
        36 => Some(br"\$"),
        0x2028 => Some(br"\u2028"),
        0x2029 => Some(br"\u2029"),
        0x85 => Some(br"\u0085"),
        _ => None,
    }
}

/// Escape literal contents, without surrounding quotes.
/// port: tsc/internal/printer/utilities.go:escapeStringWorker
pub fn escape_string_with_flags(
    bytes: &[u8],
    quote: QuoteChar,
    flags: LiteralEscapeFlags,
) -> Vec<u8> {
    let mut output = Vec::with_capacity(bytes.len());
    let mut copied_until = 0;
    let mut offset = 0;
    let jsx = flags.contains(LiteralEscapeFlags::JSX_ATTRIBUTE_ESCAPE);
    let never_ascii_escape = flags.contains(LiteralEscapeFlags::NEVER_ASCII_ESCAPE);
    while offset < bytes.len() {
        let (mut rune, mut width) = decode_rune(&bytes[offset..]);
        let mut escape = is_surrogate(rune) || rune == RUNE_ERROR && width == 1;
        match rune {
            92 => {
                if !jsx {
                    escape = true;
                }
            }
            36 => {
                if quote == QuoteChar::Backtick && bytes.get(offset + 1) == Some(&b'{') {
                    escape = true;
                }
            }
            r if r == i32::from(quote as u8) || matches!(r, 0x2028 | 0x2029 | 0x85 | 13) => {
                escape = true;
            }
            10 => {
                if quote != QuoteChar::Backtick {
                    escape = true;
                }
            }
            _ => {
                if rune <= 0x1f || !never_ascii_escape && rune > 0x7f {
                    escape = true;
                }
            }
        }
        if escape {
            output.extend_from_slice(&bytes[copied_until..offset]);
            if jsx {
                match rune {
                    0 => output.extend_from_slice(b"&#0;"),
                    34 => output.extend_from_slice(b"&quot;"),
                    39 => output.extend_from_slice(b"&apos;"),
                    _ => encode_jsx_character_entity(&mut output, rune),
                }
            } else if rune == 13
                && quote == QuoteChar::Backtick
                && bytes.get(offset + 1) == Some(&b'\n')
            {
                width += 1;
                output.extend_from_slice(br"\r\n");
            } else if rune > 0xffff {
                rune -= 0x10000;
                encode_utf16_escape_sequence(&mut output, (rune >> 10) + 0xd800);
                encode_utf16_escape_sequence(&mut output, (rune & 0x3ff) + 0xdc00);
            } else if is_surrogate(rune) {
                encode_utf16_escape_sequence(&mut output, rune);
            } else if rune == 0 {
                if bytes.get(offset + 1).is_some_and(u8::is_ascii_digit) {
                    output.extend_from_slice(br"\x00");
                } else {
                    output.extend_from_slice(br"\0");
                }
            } else if let Some(escaped) = canonical_escape(rune) {
                output.extend_from_slice(escaped);
            } else {
                encode_utf16_escape_sequence(&mut output, rune);
            }
            copied_until = offset + width;
        }
        offset += width;
    }
    output.extend_from_slice(&bytes[copied_until..]);
    output
}

/// port: tsc/internal/printer/utilities.go:EscapeString
pub fn escape_string(bytes: &[u8], quote: QuoteChar) -> Vec<u8> {
    escape_string_with_flags(bytes, quote, LiteralEscapeFlags::NEVER_ASCII_ESCAPE)
}

/// port: tsc/internal/printer/utilities.go:escapeNonAsciiString
pub fn escape_non_ascii_string(bytes: &[u8], quote: QuoteChar) -> Vec<u8> {
    escape_string_with_flags(bytes, quote, LiteralEscapeFlags::NONE)
}

/// port: tsc/internal/printer/utilities.go:escapeJsxAttributeString
pub fn escape_jsx_attribute_string(bytes: &[u8], quote: QuoteChar) -> Vec<u8> {
    escape_string_with_flags(
        bytes,
        quote,
        LiteralEscapeFlags::JSX_ATTRIBUTE_ESCAPE | LiteralEscapeFlags::NEVER_ASCII_ESCAPE,
    )
}
