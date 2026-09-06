//! `stringutil/js_case.go`: JavaScript case mapping over JS-string bytes.
//!
//! Decoder choice and output behavior are ported per helper, not selected from a
//! validity tag (docs/design/text.md, section 2.2): both mappers use the
//! sentinel-aware decoder, preserve complete surrogate sentinels verbatim, and
//! write U+FFFD for any other malformed byte because Go's `WriteRune` does.

use crate::rune::{decode_js_string_rune, encode_js_string_rune, is_surrogate, write_rune, Rune};
use crate::unicode_case_generated::{
    Range16, Range32, RangeTable, SpecialCasingCondition, SpecialCasingMapping,
    SPECIAL_CASING_MAPPINGS, UNICODE_CASED_RANGES, UNICODE_CASE_IGNORABLE_RANGES,
};

fn special_casing(ch: Rune) -> Option<&'static SpecialCasingMapping> {
    SPECIAL_CASING_MAPPINGS
        .binary_search_by_key(&ch, |(code, _)| *code)
        .ok()
        .map(|index| &SPECIAL_CASING_MAPPINGS[index].1)
}

/// `unicode.Is`, ported with its `is16`/`is32` search strategy.
fn unicode_is(table: &RangeTable, ch: Rune) -> bool {
    // Go compares as uint32 so that a negative rune misses every range.
    let value = ch as u32;
    let r16 = table.r16;
    if !r16.is_empty() && value <= u32::from(r16[r16.len() - 1].hi) {
        return is16(r16, value as u16);
    }
    let r32 = table.r32;
    if !r32.is_empty() && ch >= r32[0].lo as Rune {
        return is32(r32, value);
    }
    false
}

/// `unicode.linearMax`.
const LINEAR_MAX: usize = 18;
/// `unicode.MaxLatin1`.
const MAX_LATIN1: u16 = 0xFF;

fn is16(ranges: &[Range16], value: u16) -> bool {
    if ranges.len() <= LINEAR_MAX || value <= MAX_LATIN1 {
        for range in ranges {
            if value < range.lo {
                return false;
            }
            if value <= range.hi {
                return range.stride == 1 || (value - range.lo).is_multiple_of(range.stride);
            }
        }
        return false;
    }
    let (mut lo, mut hi) = (0, ranges.len());
    while lo < hi {
        let mid = usize::midpoint(lo, hi);
        let range = &ranges[mid];
        if range.lo <= value && value <= range.hi {
            return range.stride == 1 || (value - range.lo).is_multiple_of(range.stride);
        }
        if value < range.lo {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    false
}

fn is32(ranges: &[Range32], value: u32) -> bool {
    if ranges.len() <= LINEAR_MAX {
        for range in ranges {
            if value < range.lo {
                return false;
            }
            if value <= range.hi {
                return range.stride == 1 || (value - range.lo).is_multiple_of(range.stride);
            }
        }
        return false;
    }
    let (mut lo, mut hi) = (0, ranges.len());
    while lo < hi {
        let mid = usize::midpoint(lo, hi);
        let range = &ranges[mid];
        if range.lo <= value && value <= range.hi {
            return range.stride == 1 || (value - range.lo).is_multiple_of(range.stride);
        }
        if value < range.lo {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    false
}

// port: tsc/internal/stringutil/js_case.go:isSigmaCased
fn is_sigma_cased(ch: Rune) -> bool {
    unicode_is(&UNICODE_CASED_RANGES, ch)
}

// port: tsc/internal/stringutil/js_case.go:isUnicodeCaseIgnorable
fn is_unicode_case_ignorable(ch: Rune) -> bool {
    unicode_is(&UNICODE_CASE_IGNORABLE_RANGES, ch)
}

// port: tsc/internal/stringutil/js_case.go:hasSigmaCasedAfter
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

// port: tsc/internal/stringutil/js_case.go:isFinalSigmaContext
fn is_final_sigma_context(cased_before: bool, s: &[u8], after_offset: usize) -> bool {
    cased_before && !has_sigma_cased_after(s, after_offset)
}

// port: tsc/internal/stringutil/js_case.go:toLowerASCII
fn to_lower_ascii(s: &[u8]) -> Option<Vec<u8>> {
    let mut needs_mapping = false;
    for &ch in s {
        if ch >= 0x80 {
            return None;
        }
        needs_mapping = needs_mapping || ch.is_ascii_uppercase();
    }
    if !needs_mapping {
        return Some(s.to_vec());
    }
    Some(s.iter().map(u8::to_ascii_lowercase).collect())
}

// port: tsc/internal/stringutil/js_case.go:toUpperASCII
fn to_upper_ascii(s: &[u8]) -> Option<Vec<u8>> {
    let mut needs_mapping = false;
    for &ch in s {
        if ch >= 0x80 {
            return None;
        }
        needs_mapping = needs_mapping || ch.is_ascii_lowercase();
    }
    if !needs_mapping {
        return Some(s.to_vec());
    }
    Some(s.iter().map(u8::to_ascii_uppercase).collect())
}

// port: tsc/internal/stringutil/js_case.go:ToLowerJS
/// `String.prototype.toLowerCase`, including the Final_Sigma context.
pub fn to_lower_js(s: &[u8]) -> Vec<u8> {
    if let Some(ascii) = to_lower_ascii(s) {
        return ascii;
    }
    let mut out = Vec::with_capacity(s.len());
    // casedBefore tracks the backward half of the Final_Sigma context so the
    // scan never has to look behind.
    let mut cased_before = false;
    let mut i = 0;
    while i < s.len() {
        let (r, size) = decode_js_string_rune(&s[i..]);
        i += size;
        if is_surrogate(r) {
            // A lone surrogate has no case mapping; the sentinel bytes are
            // restored because write_rune would fold it to U+FFFD.
            out.extend_from_slice(&encode_js_string_rune(r));
        } else if let Some(mapping) = special_casing(r) {
            if mapping.condition == SpecialCasingCondition::FinalSigma
                && is_final_sigma_context(cased_before, s, i)
            {
                out.extend_from_slice(mapping.conditional_lower);
            } else {
                out.extend_from_slice(mapping.lower);
            }
        } else {
            write_rune(&mut out, r);
        }
        if !is_unicode_case_ignorable(r) {
            cased_before = is_sigma_cased(r);
        }
    }
    out
}

// port: tsc/internal/stringutil/js_case.go:ToUpperJS
/// `String.prototype.toUpperCase`.
pub fn to_upper_js(s: &[u8]) -> Vec<u8> {
    if let Some(ascii) = to_upper_ascii(s) {
        return ascii;
    }
    let mut out = Vec::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        let (r, size) = decode_js_string_rune(&s[i..]);
        if is_surrogate(r) {
            // Copy the sentinel bytes directly, as upstream does here.
            out.extend_from_slice(&s[i..i + size]);
        } else if let Some(mapping) = special_casing(r) {
            out.extend_from_slice(mapping.upper);
        } else {
            write_rune(&mut out, r);
        }
        i += size;
    }
    out
}
