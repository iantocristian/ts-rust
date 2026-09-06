//! port: tsc/internal/stringutil/js_case.go
//! port: tsc/internal/stringutil/util.go (case and truncation helpers)
//!
//! Each helper keeps its own decoder and replacement behavior, as the Go helper
//! does (docs/design/text.md, 2.2): case mapping uses the sentinel-aware decoder
//! and preserves lone surrogates; truncation and first-character lowering use the
//! standard decoder and can split a sentinel.

use crate::js_case_generated::{
    Condition, SpecialCasing, CASED_RANGES, CASE_IGNORABLE_RANGES, SPECIAL_CASING,
};
use crate::wtf8::{
    decode_js_string_rune, decode_rune, encode_js_string_rune, is_surrogate, write_rune, Rune,
    RUNE_SELF,
};

/// Go's `unicode.Is` over a stride-encoded range table.
fn in_range_table(table: &[(u32, u32, u32)], r: Rune) -> bool {
    let idx = table.partition_point(|&(lo, _, _)| lo <= r);
    idx > 0 && {
        let (lo, hi, stride) = table[idx - 1];
        r <= hi && (r - lo).is_multiple_of(stride)
    }
}

fn special_casing(r: Rune) -> Option<&'static SpecialCasing> {
    SPECIAL_CASING
        .binary_search_by_key(&r, |m| m.code_point)
        .ok()
        .map(|i| &SPECIAL_CASING[i])
}

/// port: tsc/internal/stringutil/js_case.go:isSigmaCased
fn is_sigma_cased(r: Rune) -> bool {
    in_range_table(CASED_RANGES, r)
}

/// port: tsc/internal/stringutil/js_case.go:isUnicodeCaseIgnorable
fn is_unicode_case_ignorable(r: Rune) -> bool {
    in_range_table(CASE_IGNORABLE_RANGES, r)
}

/// port: tsc/internal/stringutil/js_case.go:hasSigmaCasedAfter
fn has_sigma_cased_after(s: &[u8], start: usize) -> bool {
    let mut i = start;
    while i < s.len() {
        let (r, size) = decode_js_string_rune(&s[i..]);
        i += size;
        if is_unicode_case_ignorable(r) {
            continue;
        }
        return is_sigma_cased(r);
    }
    false
}

/// port: tsc/internal/stringutil/js_case.go:isFinalSigmaContext
fn is_final_sigma_context(cased_before: bool, s: &[u8], after_offset: usize) -> bool {
    cased_before && !has_sigma_cased_after(s, after_offset)
}

/// port: tsc/internal/stringutil/js_case.go:toLowerASCII
fn to_lower_ascii(s: &[u8]) -> Option<Vec<u8>> {
    let mut needs_mapping = false;
    for &ch in s {
        if ch >= RUNE_SELF {
            return None;
        }
        needs_mapping = needs_mapping || ch.is_ascii_uppercase();
    }
    if !needs_mapping {
        return Some(s.to_vec());
    }
    Some(s.iter().map(u8::to_ascii_lowercase).collect())
}

/// port: tsc/internal/stringutil/js_case.go:toUpperASCII
fn to_upper_ascii(s: &[u8]) -> Option<Vec<u8>> {
    let mut needs_mapping = false;
    for &ch in s {
        if ch >= RUNE_SELF {
            return None;
        }
        needs_mapping = needs_mapping || ch.is_ascii_lowercase();
    }
    if !needs_mapping {
        return Some(s.to_vec());
    }
    Some(s.iter().map(u8::to_ascii_uppercase).collect())
}

/// port: tsc/internal/stringutil/js_case.go:ToLowerJS
pub fn to_lower_js(s: &[u8]) -> Vec<u8> {
    if let Some(ascii) = to_lower_ascii(s) {
        return ascii;
    }
    let mut b = Vec::with_capacity(s.len());
    let mut cased_before = false;
    let mut i = 0;
    while i < s.len() {
        let (r, size) = decode_js_string_rune(&s[i..]);
        i += size;
        if is_surrogate(r) {
            encode_js_string_rune(&mut b, r);
        } else if let Some(mapping) = special_casing(r) {
            if mapping.condition == Condition::FinalSigma
                && is_final_sigma_context(cased_before, s, i)
            {
                b.extend_from_slice(mapping.conditional_lower.as_bytes());
            } else {
                b.extend_from_slice(mapping.lower.as_bytes());
            }
        } else {
            write_rune(&mut b, r);
        }
        if !is_unicode_case_ignorable(r) {
            cased_before = is_sigma_cased(r);
        }
    }
    b
}

/// port: tsc/internal/stringutil/js_case.go:ToUpperJS
pub fn to_upper_js(s: &[u8]) -> Vec<u8> {
    if let Some(ascii) = to_upper_ascii(s) {
        return ascii;
    }
    let mut b = Vec::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        let (r, size) = decode_js_string_rune(&s[i..]);
        if is_surrogate(r) {
            b.extend_from_slice(&s[i..i + size]);
        } else if let Some(mapping) = special_casing(r) {
            b.extend_from_slice(mapping.upper.as_bytes());
        } else {
            write_rune(&mut b, r);
        }
        i += size;
    }
    b
}

/// Go's `unicode.ToLower`: the simple (single code point) lowercase mapping.
///
/// Rust's `char::to_lowercase` is the full mapping; the two differ only where
/// SpecialCasing adds a multi-character lowercase form, which for unconditional
/// mappings is U+0130 alone. Unicode version skew between Go's tables and Rust's
/// is not corrected here; the E4 oracle arbitrates on the fixtures.
fn simple_to_lower(ch: Rune) -> Rune {
    if ch == 0x130 {
        return 0x69;
    }
    let Some(c) = char::from_u32(ch) else {
        return ch;
    };
    let mut lower = c.to_lowercase();
    match (lower.next(), lower.next()) {
        (Some(l), None) => Rune::from(l),
        _ => ch,
    }
}

/// port: tsc/internal/stringutil/util.go:LowerFirstChar
///
/// Standard decoding of the first character, so a sentinel or malformed prefix
/// becomes U+FFFD followed by the remaining bytes copied verbatim.
pub fn lower_first_char(s: &[u8]) -> Vec<u8> {
    let (ch, size) = decode_rune(s);
    if size > 0 {
        let mut out = Vec::with_capacity(s.len() + 2);
        write_rune(&mut out, simple_to_lower(ch));
        out.extend_from_slice(&s[size..]);
        return out;
    }
    s.to_vec()
}

/// port: tsc/internal/stringutil/util.go:TruncateByRunes
///
/// Go's `range` counts each malformed byte as one rune, so truncating the
/// sentinel bytes `ED A0 80` to one rune returns the single byte `ED`.
pub fn truncate_by_runes(s: &[u8], max_length: i64) -> &[u8] {
    if (s.len() as i64) < max_length {
        return s;
    }
    if max_length <= 0 {
        return &[];
    }
    let mut rune_count = 0i64;
    let mut i = 0;
    while i < s.len() {
        rune_count += 1;
        if rune_count > max_length {
            return &s[..i];
        }
        let (_, size) = decode_rune(&s[i..]);
        i += size;
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentinels_and_malformed_bytes() {
        assert_eq!(to_lower_js(b"A\xED\xA0\x80B"), b"a\xED\xA0\x80b");
        assert_eq!(to_upper_js(b"a\xED\xA0\x80b"), b"A\xED\xA0\x80B");
        assert_eq!(
            to_lower_js(b"A\xFF"),
            b"a\xEF\xBF\xBD",
            "a malformed byte becomes U+FFFD"
        );
        assert_eq!(truncate_by_runes(b"\xED\xA0\x80", 1), b"\xED");
        assert_eq!(lower_first_char(b"\xED\xA0\x80x"), b"\xEF\xBF\xBD\xA0\x80x");
    }

    #[test]
    fn final_sigma_and_special_casing() {
        assert_eq!(to_lower_js("ΟΔΟΣ".as_bytes()), "οδος".as_bytes());
        assert_eq!(to_lower_js("ΟΔΟΣ Σ".as_bytes()), "οδος σ".as_bytes());
        assert_eq!(to_upper_js("ß".as_bytes()), "SS".as_bytes());
        assert_eq!(to_lower_js("İ".as_bytes()), "i\u{307}".as_bytes());
        assert_eq!(
            lower_first_char("İx".as_bytes()),
            "ix".as_bytes(),
            "simple mapping for the first character"
        );
    }
}
