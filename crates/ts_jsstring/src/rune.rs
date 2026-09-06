//! Go's `unicode/utf8` and `unicode/utf16` primitives, plus the WTF-8 sentinel
//! encoder and decoder from `stringutil`.
//!
//! Everything here works on bytes and on Go `rune` values (`i32`), not on Rust
//! `char`, because a JavaScript string can hold a lone surrogate and a source
//! file can hold a byte that is not valid UTF-8 at all. The decoders reproduce
//! Go's exact classification, including `RuneError` with size 1 for a malformed
//! byte, which several upstream behaviors depend on (docs/design/text.md,
//! section 1, item 2).

/// A Go `rune`. Negative and out-of-range values are representable, as in Go.
pub type Rune = i32;

/// `utf8.RuneError`, the Unicode replacement character U+FFFD.
pub const RUNE_ERROR: Rune = 0xFFFD;
/// `utf8.RuneSelf`: below this, a byte is its own code point.
pub const RUNE_SELF: u8 = 0x80;
/// `utf8.MaxRune`.
pub const MAX_RUNE: Rune = 0x0010_FFFF;

const SURR1: Rune = 0xD800;
const SURR2: Rune = 0xDC00;
const SURR3: Rune = 0xE000;
const SURR_SELF: Rune = 0x0001_0000;

// port: tsc/internal/stringutil/util.go:IsSurrogate
/// `utf16.IsSurrogate`: the code point is in the UTF-16 surrogate range.
pub fn is_surrogate(ch: Rune) -> bool {
    (SURR1..SURR3).contains(&ch)
}

// port: tsc/internal/stringutil/util.go:IsHighSurrogate
pub fn is_high_surrogate(ch: Rune) -> bool {
    is_surrogate(ch) && ch < SURROGATE_LOW_START
}

// port: tsc/internal/stringutil/util.go:IsLowSurrogate
pub fn is_low_surrogate(ch: Rune) -> bool {
    is_surrogate(ch) && ch >= SURROGATE_LOW_START
}

/// `stringutil.SurrogateLowStart`.
pub const SURROGATE_LOW_START: Rune = 0xDC00;

// port: tsc/internal/stringutil/util.go:SurrogatePairToCodePoint
/// `utf16.DecodeRune`: U+FFFD unless the pair is a valid high/low surrogate pair.
pub fn surrogate_pair_to_code_point(high: Rune, low: Rune) -> Rune {
    if (SURR1..SURR2).contains(&high) && (SURR2..SURR3).contains(&low) {
        (((high - SURR1) << 10) | (low - SURR2)) + SURR_SELF
    } else {
        RUNE_ERROR
    }
}

// port: tsc/internal/stringutil/util.go:CodePointToSurrogatePair
/// `utf16.EncodeRune`: (U+FFFD, U+FFFD) when the code point needs no pair.
pub fn code_point_to_surrogate_pair(ch: Rune) -> (Rune, Rune) {
    if !(SURR_SELF..=MAX_RUNE).contains(&ch) {
        return (RUNE_ERROR, RUNE_ERROR);
    }
    let ch = ch - SURR_SELF;
    (SURR1 + ((ch >> 10) & 0x3FF), SURR2 + (ch & 0x3FF))
}

/// `utf16.RuneLen`: UTF-16 code units, or -1 when the rune cannot be encoded.
pub fn utf16_rune_len(ch: Rune) -> i32 {
    if (0..SURR1).contains(&ch) || (SURR3..SURR_SELF).contains(&ch) {
        1
    } else if (SURR_SELF..=MAX_RUNE).contains(&ch) {
        2
    } else {
        -1
    }
}

/// `utf8.DecodeRuneInString` over bytes: `(RuneError, 0)` for an empty input and
/// `(RuneError, 1)` for any malformed sequence, including surrogates, overlong
/// encodings and code points above U+10FFFF.
pub fn decode_rune(s: &[u8]) -> (Rune, usize) {
    let Some(&b0) = s.first() else {
        return (RUNE_ERROR, 0);
    };
    if b0 < RUNE_SELF {
        return (Rune::from(b0), 1);
    }
    // Go's acceptance ranges: the second byte's bounds pin down overlong
    // encodings, surrogates and values above U+10FFFF.
    let (size, lo, hi) = match b0 {
        0xC2..=0xDF => (2, 0x80, 0xBF),
        0xE0 => (3, 0xA0, 0xBF),
        // 0xED is split out from its neighbours because its narrower second
        // byte is exactly what rejects the surrogate range.
        0xED => (3, 0x80, 0x9F),
        0xE1..=0xEC | 0xEE..=0xEF => (3, 0x80, 0xBF),
        0xF0 => (4, 0x90, 0xBF),
        0xF1..=0xF3 => (4, 0x80, 0xBF),
        0xF4 => (4, 0x80, 0x8F),
        _ => return (RUNE_ERROR, 1),
    };
    if s.len() < size {
        return (RUNE_ERROR, 1);
    }
    let b1 = s[1];
    if b1 < lo || b1 > hi {
        return (RUNE_ERROR, 1);
    }
    if size == 2 {
        return ((Rune::from(b0 & 0x1F) << 6) | Rune::from(b1 & 0x3F), 2);
    }
    let b2 = s[2];
    if !(0x80..=0xBF).contains(&b2) {
        return (RUNE_ERROR, 1);
    }
    if size == 3 {
        return (
            (Rune::from(b0 & 0x0F) << 12) | (Rune::from(b1 & 0x3F) << 6) | Rune::from(b2 & 0x3F),
            3,
        );
    }
    let b3 = s[3];
    if !(0x80..=0xBF).contains(&b3) {
        return (RUNE_ERROR, 1);
    }
    (
        (Rune::from(b0 & 0x07) << 18)
            | (Rune::from(b1 & 0x3F) << 12)
            | (Rune::from(b2 & 0x3F) << 6)
            | Rune::from(b3 & 0x3F),
        4,
    )
}

/// Go's `for i, r := range s` over a byte string: the same decoder as
/// [`decode_rune`], one malformed byte at a time.
pub struct Runes<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Runes<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }
}

impl Iterator for Runes<'_> {
    /// `(byte offset, rune, size)`.
    type Item = (usize, Rune, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.bytes.len() {
            return None;
        }
        let start = self.pos;
        let (r, size) = decode_rune(&self.bytes[start..]);
        self.pos += size;
        Some((start, r, size))
    }
}

/// Go's `strings.Builder.WriteRune` and `string(rune)` conversion: an invalid
/// rune is written as U+FFFD.
pub fn write_rune(out: &mut Vec<u8>, ch: Rune) {
    let ch = if (0..SURR1).contains(&ch) || (SURR3..=MAX_RUNE).contains(&ch) {
        ch
    } else {
        RUNE_ERROR
    };
    let ch = ch as u32;
    match ch {
        0x0000..=0x007F => out.push(ch as u8),
        0x0080..=0x07FF => {
            out.push(0xC0 | (ch >> 6) as u8);
            out.push(0x80 | (ch & 0x3F) as u8);
        }
        0x0800..=0xFFFF => {
            out.push(0xE0 | (ch >> 12) as u8);
            out.push(0x80 | ((ch >> 6) & 0x3F) as u8);
            out.push(0x80 | (ch & 0x3F) as u8);
        }
        _ => {
            out.push(0xF0 | (ch >> 18) as u8);
            out.push(0x80 | ((ch >> 12) & 0x3F) as u8);
            out.push(0x80 | ((ch >> 6) & 0x3F) as u8);
            out.push(0x80 | (ch & 0x3F) as u8);
        }
    }
}

// The WTF-8 sentinel constants, spelled out as upstream spells them.
const SURROGATE_UTF8_LEAD: u8 = 0xED;
const SURROGATE_UTF8_LEAD_BITS: Rune = 0xD000;
const UTF8_CONT_MARKER: u8 = 0x80;
const UTF8_CONT_MAX: u8 = 0xBF;
const UTF8_CONT_MASK: u8 = 0x3F;
const SURROGATE_UTF8_BYTE1_MIN: u8 = 0xA0;
const SURROGATE_UTF8_BYTE1_MAX: u8 = 0xBF;

// port: tsc/internal/stringutil/util.go:EncodeJSStringRune
/// A lone surrogate becomes the three-byte WTF-8 sentinel; anything else is
/// written as Go's `string(ch)` would write it.
pub fn encode_js_string_rune(ch: Rune) -> Vec<u8> {
    if is_surrogate(ch) {
        return vec![
            SURROGATE_UTF8_LEAD,
            UTF8_CONT_MARKER | ((ch >> 6) as u8 & UTF8_CONT_MASK),
            UTF8_CONT_MARKER | (ch as u8 & UTF8_CONT_MASK),
        ];
    }
    let mut out = Vec::new();
    write_rune(&mut out, ch);
    out
}

// port: tsc/internal/stringutil/util.go:DecodeJSStringRune
/// The sentinel-aware decoder: a WTF-8 sentinel yields its surrogate code point
/// with size 3; everything else defers to [`decode_rune`].
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

/// [`decode_js_string_rune`] applied repeatedly, as `(offset, rune, size)`.
pub struct JsRunes<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> JsRunes<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }
}

impl Iterator for JsRunes<'_> {
    type Item = (usize, Rune, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.bytes.len() {
            return None;
        }
        let start = self.pos;
        let (r, size) = decode_js_string_rune(&self.bytes[start..]);
        self.pos += size;
        Some((start, r, size))
    }
}

// port: tsc/internal/stringutil/util.go:CombineSurrogatePairs
/// Merge adjacent high and low sentinels into the supplementary code point they
/// represent. Applied wherever separately scanned values are joined.
pub fn combine_surrogate_pairs(s: &[u8]) -> Vec<u8> {
    if !s.contains(&SURROGATE_UTF8_LEAD) {
        return s.to_vec();
    }
    let mut out = Vec::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        let (r, size) = decode_js_string_rune(&s[i..]);
        if is_high_surrogate(r) {
            let (low, low_size) = decode_js_string_rune(&s[i + size..]);
            if is_low_surrogate(low) {
                write_rune(&mut out, surrogate_pair_to_code_point(r, low));
                i += size + low_size;
                continue;
            }
        }
        out.extend_from_slice(&s[i..i + size]);
        i += size;
    }
    out
}

/// `utf16.Decode`: unpaired surrogates become U+FFFD, as `decodeUtf16` relies on.
pub fn utf16_decode(units: &[u16]) -> Vec<Rune> {
    let mut out = Vec::with_capacity(units.len());
    let mut i = 0;
    while i < units.len() {
        let u = Rune::from(units[i]);
        if !(SURR1..SURR3).contains(&u) {
            out.push(u);
            i += 1;
        } else if (SURR1..SURR2).contains(&u)
            && i + 1 < units.len()
            && (SURR2..SURR3).contains(&Rune::from(units[i + 1]))
        {
            out.push(surrogate_pair_to_code_point(u, Rune::from(units[i + 1])));
            i += 2;
        } else {
            out.push(RUNE_ERROR);
            i += 1;
        }
    }
    out
}
