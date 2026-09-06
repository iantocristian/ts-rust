//! port: tsc/internal/stringutil/util.go (WTF-8 sentinel codec and surrogate helpers)
//!
//! A JavaScript string value is a sequence of UTF-16 code units. Corsa stores it
//! as UTF-8 bytes, with a lone surrogate written as the three-byte sentinel that
//! UTF-8 would use if surrogates were encodable (`ED A0..BF 80..BF`). These are
//! the byte-level primitives; every helper that decodes chooses its decoder
//! explicitly, as the corresponding Go helper does (docs/design/text.md, 2.2).

/// A decoded code point in Go's sense: a Unicode scalar value, a lone surrogate
/// (only from the sentinel-aware decoder) or [`RUNE_ERROR`] for a malformed byte.
pub type Rune = u32;

/// Go's `utf8.RuneError`, U+FFFD.
pub const RUNE_ERROR: Rune = 0xFFFD;

/// Go's `utf8.RuneSelf`: bytes below it are single-byte code points.
pub const RUNE_SELF: u8 = 0x80;

/// The `SurrogateLowStart` constant of tsc/internal/stringutil/util.go.
pub const SURROGATE_LOW_START: Rune = 0xDC00;

const SURROGATE_UTF8_LEAD: u8 = 0xED;
const SURROGATE_UTF8_LEAD_BITS: Rune = 0xD000;
const UTF8_CONT_MARKER: u8 = 0x80;
const UTF8_CONT_MAX: u8 = 0xBF;
const UTF8_CONT_MASK: u8 = 0x3F;
const SURROGATE_UTF8_BYTE1_MIN: u8 = 0xA0;
const SURROGATE_UTF8_BYTE1_MAX: u8 = 0xBF;

/// port: tsc/internal/stringutil/util.go:IsHighSurrogate
pub fn is_high_surrogate(ch: Rune) -> bool {
    is_surrogate(ch) && ch < SURROGATE_LOW_START
}

/// port: tsc/internal/stringutil/util.go:IsLowSurrogate
pub fn is_low_surrogate(ch: Rune) -> bool {
    is_surrogate(ch) && ch >= SURROGATE_LOW_START
}

/// port: tsc/internal/stringutil/util.go:IsSurrogate
pub fn is_surrogate(ch: Rune) -> bool {
    (0xD800..=0xDFFF).contains(&ch)
}

/// port: tsc/internal/stringutil/util.go:SurrogatePairToCodePoint
///
/// Go's `utf16.DecodeRune`: a valid pair yields the supplementary code point,
/// anything else yields U+FFFD.
pub fn surrogate_pair_to_code_point(high: Rune, low: Rune) -> Rune {
    if is_high_surrogate(high) && is_low_surrogate(low) {
        0x1_0000 + ((high - 0xD800) << 10) + (low - 0xDC00)
    } else {
        RUNE_ERROR
    }
}

/// port: tsc/internal/stringutil/util.go:CodePointToSurrogatePair
///
/// Go's `utf16.EncodeRune`: outside the supplementary range both halves are U+FFFD.
pub fn code_point_to_surrogate_pair(ch: Rune) -> (Rune, Rune) {
    if !(0x1_0000..=0x10_FFFF).contains(&ch) {
        return (RUNE_ERROR, RUNE_ERROR);
    }
    let r = ch - 0x1_0000;
    (0xD800 + (r >> 10), 0xDC00 + (r & 0x3FF))
}

/// Go's `utf16.RuneLen`: UTF-16 code units for a code point, `-1` for a surrogate
/// or an out-of-range value.
pub fn utf16_rune_len(r: Rune) -> i64 {
    if r < 0xD800 || (0xE000..0x1_0000).contains(&r) {
        1
    } else if (0x1_0000..=0x10_FFFF).contains(&r) {
        2
    } else {
        -1
    }
}

/// Go's standard decoder, `utf8.DecodeRune` on a byte slice: the code point and
/// its size, `(RUNE_ERROR, 0)` on empty input and `(RUNE_ERROR, 1)` for a
/// malformed or truncated sequence, including a surrogate encoding.
pub fn decode_rune(s: &[u8]) -> (Rune, usize) {
    let Some(&b0) = s.first() else {
        return (RUNE_ERROR, 0);
    };
    if b0 < RUNE_SELF {
        return (Rune::from(b0), 1);
    }
    let cont = |i: usize| {
        s.get(i)
            .is_some_and(|&b| (UTF8_CONT_MARKER..=UTF8_CONT_MAX).contains(&b))
    };
    let in_range = |i: usize, lo: u8, hi: u8| s.get(i).is_some_and(|&b| (lo..=hi).contains(&b));
    let (size, ok) = match b0 {
        0xC2..=0xDF => (2, cont(1)),
        0xE0 => (3, in_range(1, 0xA0, 0xBF) && cont(2)),
        0xE1..=0xEC | 0xEE..=0xEF => (3, cont(1) && cont(2)),
        0xED => (3, in_range(1, 0x80, 0x9F) && cont(2)),
        0xF0 => (4, in_range(1, 0x90, 0xBF) && cont(2) && cont(3)),
        0xF1..=0xF3 => (4, cont(1) && cont(2) && cont(3)),
        0xF4 => (4, in_range(1, 0x80, 0x8F) && cont(2) && cont(3)),
        _ => (1, false),
    };
    if !ok {
        return (RUNE_ERROR, 1);
    }
    let r = match size {
        2 => (Rune::from(b0 & 0x1F) << 6) | Rune::from(s[1] & UTF8_CONT_MASK),
        3 => {
            (Rune::from(b0 & 0x0F) << 12)
                | (Rune::from(s[1] & UTF8_CONT_MASK) << 6)
                | Rune::from(s[2] & UTF8_CONT_MASK)
        }
        _ => {
            (Rune::from(b0 & 0x07) << 18)
                | (Rune::from(s[1] & UTF8_CONT_MASK) << 12)
                | (Rune::from(s[2] & UTF8_CONT_MASK) << 6)
                | Rune::from(s[3] & UTF8_CONT_MASK)
        }
    };
    (r, size)
}

/// Go's `utf8.AppendRune` / `strings.Builder.WriteRune`: a surrogate or an
/// out-of-range value is written as U+FFFD.
pub fn write_rune(out: &mut Vec<u8>, r: Rune) {
    let c = char::from_u32(r).unwrap_or('\u{FFFD}');
    let mut buf = [0u8; 4];
    out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
}

/// port: tsc/internal/stringutil/util.go:EncodeJSStringRune
pub fn encode_js_string_rune(out: &mut Vec<u8>, ch: Rune) {
    if is_surrogate(ch) {
        out.push(SURROGATE_UTF8_LEAD);
        out.push(UTF8_CONT_MARKER | (((ch >> 6) & Rune::from(UTF8_CONT_MASK)) as u8));
        out.push(UTF8_CONT_MARKER | ((ch & Rune::from(UTF8_CONT_MASK)) as u8));
    } else {
        write_rune(out, ch);
    }
}

/// port: tsc/internal/stringutil/util.go:DecodeJSStringRune
///
/// The sentinel-aware decoder: a surrogate sentinel decodes to the surrogate with
/// size 3; everything else defers to [`decode_rune`].
pub fn decode_js_string_rune(s: &[u8]) -> (Rune, usize) {
    if s.len() >= 3
        && s[0] == SURROGATE_UTF8_LEAD
        && (SURROGATE_UTF8_BYTE1_MIN..=SURROGATE_UTF8_BYTE1_MAX).contains(&s[1])
        && (UTF8_CONT_MARKER..=UTF8_CONT_MAX).contains(&s[2])
    {
        return (
            SURROGATE_UTF8_LEAD_BITS
                | (Rune::from(s[1] & UTF8_CONT_MASK) << 6)
                | Rune::from(s[2] & UTF8_CONT_MASK),
            3,
        );
    }
    decode_rune(s)
}

/// port: tsc/internal/stringutil/util.go:CombineSurrogatePairs
///
/// Merges adjacent high and low sentinels into one supplementary code point; the
/// common sentinel-free case is returned unchanged.
pub fn combine_surrogate_pairs(s: &[u8]) -> std::borrow::Cow<'_, [u8]> {
    if !s.contains(&SURROGATE_UTF8_LEAD) {
        return std::borrow::Cow::Borrowed(s);
    }
    let mut b = Vec::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        let (r, size) = decode_js_string_rune(&s[i..]);
        if is_high_surrogate(r) {
            let (low, low_size) = decode_js_string_rune(&s[i + size..]);
            if is_low_surrogate(low) {
                write_rune(&mut b, surrogate_pair_to_code_point(r, low));
                i += size + low_size;
                continue;
            }
        }
        b.extend_from_slice(&s[i..i + size]);
        i += size;
    }
    std::borrow::Cow::Owned(b)
}

/// port: tsc/internal/stringutil/util.go:IsLineBreak
pub fn is_line_break(ch: Rune) -> bool {
    matches!(ch, 0x0A | 0x0D | 0x2028 | 0x2029)
}

/// port: tsc/internal/stringutil/util.go:IsDigit
pub fn is_digit(ch: Rune) -> bool {
    (Rune::from(b'0')..=Rune::from(b'9')).contains(&ch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_decoder_matches_go_edge_cases() {
        assert_eq!(decode_rune(b""), (RUNE_ERROR, 0));
        assert_eq!(decode_rune(b"a"), (0x61, 1));
        assert_eq!(decode_rune("é".as_bytes()), (0xE9, 2));
        assert_eq!(decode_rune("😀".as_bytes()), (0x1F600, 4));
        assert_eq!(decode_rune(&[0xFF]), (RUNE_ERROR, 1));
        assert_eq!(
            decode_rune(&[0xED, 0xA0, 0x80]),
            (RUNE_ERROR, 1),
            "surrogate encodings are malformed to the standard decoder"
        );
        assert_eq!(
            decode_rune(&[0xE2, 0x82]),
            (RUNE_ERROR, 1),
            "truncated sequence"
        );
        assert_eq!(decode_rune(&[0xC0, 0x80]), (RUNE_ERROR, 1), "overlong");
        assert_eq!(
            decode_rune(&[0xEF, 0xBF, 0xBD]),
            (RUNE_ERROR, 3),
            "an encoded U+FFFD is a normal three-byte character"
        );
    }

    #[test]
    fn sentinel_round_trip_and_combination() {
        let mut out = Vec::new();
        encode_js_string_rune(&mut out, 0xD800);
        assert_eq!(out, [0xED, 0xA0, 0x80]);
        assert_eq!(decode_js_string_rune(&out), (0xD800, 3));
        let mut pair = Vec::new();
        encode_js_string_rune(&mut pair, 0xD83D);
        encode_js_string_rune(&mut pair, 0xDE00);
        assert_eq!(combine_surrogate_pairs(&pair).as_ref(), "😀".as_bytes());
        assert!(matches!(
            combine_surrogate_pairs(b"plain"),
            std::borrow::Cow::Borrowed(_)
        ));
    }
}
