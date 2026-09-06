//! port: tsc/internal/printer/utilities.go (string literal escaping)
//!
//! The leaf escape helpers behind `getLiteralText`. Integration with the printer's
//! original-source reuse belongs to S08; these functions reproduce the byte output
//! of `escapeStringWorker` and its three exported and unexported wrappers.

use crate::wtf8::{decode_js_string_rune, is_digit, is_surrogate, Rune, RUNE_ERROR};

/// The `QuoteChar` type of tsc/internal/printer/utilities.go.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum QuoteChar {
    Single,
    Double,
    Backtick,
}

impl QuoteChar {
    pub fn byte(self) -> u8 {
        match self {
            QuoteChar::Single => b'\'',
            QuoteChar::Double => b'"',
            QuoteChar::Backtick => b'`',
        }
    }

    fn rune(self) -> Rune {
        Rune::from(self.byte())
    }
}

/// The `getLiteralTextFlags` type of tsc/internal/printer/utilities.go.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct LiteralTextFlags(u32);

impl LiteralTextFlags {
    pub const NONE: Self = Self(0);
    pub const NEVER_ASCII_ESCAPE: Self = Self(1 << 0);
    pub const JSX_ATTRIBUTE_ESCAPE: Self = Self(1 << 1);
    pub const TERMINATE_UNTERMINATED_LITERALS: Self = Self(1 << 2);
    pub const ALLOW_NUMERIC_SEPARATOR: Self = Self(1 << 3);

    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    pub const fn bits(self) -> u32 {
        self.0
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}

impl std::ops::BitOr for LiteralTextFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

fn jsx_escaped_chars_map(ch: Rune) -> Option<&'static [u8]> {
    match ch {
        0x22 => Some(b"&quot;"),
        0x27 => Some(b"&apos;"),
        _ => None,
    }
}

fn escaped_chars_map(ch: Rune) -> Option<&'static [u8]> {
    Some(match ch {
        0x09 => b"\\t",
        0x0B => b"\\v",
        0x0C => b"\\f",
        0x08 => b"\\b",
        0x0D => b"\\r",
        0x0A => b"\\n",
        0x5C => b"\\\\",
        0x22 => b"\\\"",
        0x27 => b"\\'",
        0x60 => b"\\`",
        0x24 => b"\\$",
        0x2028 => b"\\u2028",
        0x2029 => b"\\u2029",
        0x85 => b"\\u0085",
        _ => return None,
    })
}

/// port: tsc/internal/printer/utilities.go:encodeJsxCharacterEntity
fn encode_jsx_character_entity(b: &mut Vec<u8>, char_code: Rune) {
    b.extend_from_slice(b"&#x");
    b.extend_from_slice(format!("{char_code:X}").as_bytes());
    b.push(b';');
}

/// port: tsc/internal/printer/utilities.go:encodeUtf16EscapeSequence
fn encode_utf16_escape_sequence(b: &mut Vec<u8>, char_code: Rune) {
    let hex = format!("{char_code:X}");
    b.extend_from_slice(b"\\u");
    for _ in hex.len()..4 {
        b.push(b'0');
    }
    b.extend_from_slice(hex.as_bytes());
}

/// port: tsc/internal/printer/utilities.go:escapeStringWorker
///
/// Does not wrap the output in quotes. A lone surrogate is written as `\uD800`
/// style escapes and a stray malformed byte as `�`.
pub fn escape_string_worker(
    s: &[u8],
    quote_char: QuoteChar,
    flags: LiteralTextFlags,
    b: &mut Vec<u8>,
) {
    let jsx = flags.contains(LiteralTextFlags::JSX_ATTRIBUTE_ESCAPE);
    let never_ascii_escape = flags.contains(LiteralTextFlags::NEVER_ASCII_ESCAPE);
    let mut pos = 0;
    let mut i = 0;
    while i < s.len() {
        let (ch, mut size) = decode_js_string_rune(&s[i..]);

        let mut escape = is_surrogate(ch) || (ch == RUNE_ERROR && size == 1);

        if ch == 0x5C {
            if !jsx {
                escape = true;
            }
        } else if ch == 0x24 {
            if quote_char == QuoteChar::Backtick && i + 1 < s.len() && s[i + 1] == b'{' {
                escape = true;
            }
        } else if ch == quote_char.rune() || matches!(ch, 0x2028 | 0x2029 | 0x85 | 0x0D) {
            escape = true;
        } else if ch == 0x0A {
            if quote_char != QuoteChar::Backtick {
                escape = true;
            }
        } else if ch <= 0x1F || (!never_ascii_escape && ch > 0x7F) {
            escape = true;
        }

        if escape {
            if pos < i {
                b.extend_from_slice(&s[pos..i]);
            }
            if jsx {
                if ch == 0 {
                    b.extend_from_slice(b"&#0;");
                } else if let Some(m) = jsx_escaped_chars_map(ch) {
                    b.extend_from_slice(m);
                } else {
                    encode_jsx_character_entity(b, ch);
                }
            } else if ch == 0x0D
                && quote_char == QuoteChar::Backtick
                && i + 1 < s.len()
                && s[i + 1] == b'\n'
            {
                size += 1;
                b.extend_from_slice(b"\\r\\n");
            } else if ch > 0xFFFF {
                let c = ch - 0x1_0000;
                encode_utf16_escape_sequence(b, ((c & 0b1111_1111_1100_0000_0000) >> 10) + 0xD800);
                encode_utf16_escape_sequence(b, (c & 0b0000_0000_0011_1111_1111) + 0xDC00);
            } else if is_surrogate(ch) {
                encode_utf16_escape_sequence(b, ch);
            } else if ch == 0 {
                if i + 1 < s.len() && is_digit(Rune::from(s[i + 1])) {
                    b.extend_from_slice(b"\\x00");
                } else {
                    b.extend_from_slice(b"\\0");
                }
            } else if let Some(m) = escaped_chars_map(ch) {
                b.extend_from_slice(m);
            } else {
                encode_utf16_escape_sequence(b, ch);
            }
            pos = i + size;
        }

        i += size;
    }

    if pos < i {
        b.extend_from_slice(&s[pos..]);
    }
}

/// port: tsc/internal/printer/utilities.go:EscapeString
pub fn escape_string(s: &[u8], quote_char: QuoteChar) -> Vec<u8> {
    let mut b = Vec::with_capacity(s.len() + 2);
    escape_string_worker(s, quote_char, LiteralTextFlags::NEVER_ASCII_ESCAPE, &mut b);
    b
}

/// port: tsc/internal/printer/utilities.go:escapeNonAsciiString
pub fn escape_non_ascii_string(s: &[u8], quote_char: QuoteChar) -> Vec<u8> {
    let mut b = Vec::with_capacity(s.len() + 2);
    escape_string_worker(s, quote_char, LiteralTextFlags::NONE, &mut b);
    b
}

/// port: tsc/internal/printer/utilities.go:escapeJsxAttributeString
pub fn escape_jsx_attribute_string(s: &[u8], quote_char: QuoteChar) -> Vec<u8> {
    let mut b = Vec::with_capacity(s.len() + 2);
    escape_string_worker(
        s,
        quote_char,
        LiteralTextFlags::JSX_ATTRIBUTE_ESCAPE | LiteralTextFlags::NEVER_ASCII_ESCAPE,
        &mut b,
    );
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_match_upstream_shapes() {
        assert_eq!(escape_string(b"a\"b", QuoteChar::Double), b"a\\\"b");
        assert_eq!(escape_string(b"a\"b", QuoteChar::Single), b"a\"b");
        assert_eq!(
            escape_string(b"\xED\xA0\x80", QuoteChar::Double),
            b"\\uD800"
        );
        assert_eq!(escape_string(b"\xFF", QuoteChar::Double), b"\\uFFFD");
        assert_eq!(
            escape_non_ascii_string("é".as_bytes(), QuoteChar::Double),
            b"\\u00E9"
        );
        assert_eq!(
            escape_non_ascii_string("😀".as_bytes(), QuoteChar::Double),
            b"\\uD83D\\uDE00"
        );
        assert_eq!(escape_string(b"\r\n", QuoteChar::Backtick), b"\\r\\n");
        assert_eq!(escape_string(b"\n", QuoteChar::Backtick), b"\n");
        assert_eq!(escape_string(b"${x}", QuoteChar::Backtick), b"\\${x}");
        assert_eq!(escape_string(b"\x001", QuoteChar::Double), b"\\x001");
        assert_eq!(escape_string(b"\x00a", QuoteChar::Double), b"\\0a");
        assert_eq!(
            escape_jsx_attribute_string(b"a\"\\\x00", QuoteChar::Double),
            b"a&quot;\\&#0;"
        );
        assert_eq!(
            escape_jsx_attribute_string(b"\x01", QuoteChar::Double),
            b"&#x1;"
        );
    }
}
