//! `printer/utilities.go`'s literal escaping, as a leaf helper.
//!
//! The escape rules live here because they are pure JS-string byte semantics;
//! the printer's decision between reusing eligible original source text and
//! regenerating a literal is a separate S08 concern (docs/design/text.md,
//! section 2.2). A high-surrogate sentinel becomes `\uD800` and a stray
//! malformed byte becomes `�`; the final writer adds no normalization.

use crate::rune::{decode_js_string_rune, Rune, RUNE_ERROR};

/// `printer.QuoteChar`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum QuoteChar {
    SingleQuote,
    DoubleQuote,
    Backtick,
}

impl QuoteChar {
    pub fn rune(self) -> Rune {
        match self {
            QuoteChar::SingleQuote => 0x27,
            QuoteChar::DoubleQuote => 0x22,
            QuoteChar::Backtick => 0x60,
        }
    }
}

/// `printer.getLiteralTextFlags`. Only the flags `escapeStringWorker` reads are
/// interpreted here; the rest belong to `getLiteralText`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct LiteralTextFlags(pub u32);

impl LiteralTextFlags {
    pub const NONE: Self = Self(0);
    pub const NEVER_ASCII_ESCAPE: Self = Self(1 << 0);
    pub const JSX_ATTRIBUTE_ESCAPE: Self = Self(1 << 1);
    pub const TERMINATE_UNTERMINATED_LITERALS: Self = Self(1 << 2);
    pub const ALLOW_NUMERIC_SEPARATOR: Self = Self(1 << 3);

    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    #[must_use]
    pub fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

// port: tsc/internal/printer/utilities.go:encodeJsxCharacterEntity
fn encode_jsx_character_entity(out: &mut Vec<u8>, char_code: Rune) {
    out.extend_from_slice(b"&#x");
    out.extend_from_slice(format!("{char_code:X}").as_bytes());
    out.push(b';');
}

// port: tsc/internal/printer/utilities.go:encodeUtf16EscapeSequence
fn encode_utf16_escape_sequence(out: &mut Vec<u8>, char_code: Rune) {
    let hex = format!("{char_code:X}");
    out.extend_from_slice(b"\\u");
    for _ in hex.len()..4 {
        out.push(b'0');
    }
    out.extend_from_slice(hex.as_bytes());
}

/// Upstream's `jsxEscapedCharsMap` table (`printer/utilities.go`).
fn jsx_escaped_char(ch: Rune) -> Option<&'static [u8]> {
    match ch {
        0x22 => Some(b"&quot;"),
        0x27 => Some(b"&apos;"),
        _ => None,
    }
}

/// Upstream's `escapedCharsMap` table (`printer/utilities.go`).
fn escaped_char(ch: Rune) -> Option<&'static [u8]> {
    match ch {
        0x09 => Some(b"\\t"),
        0x0B => Some(b"\\v"),
        0x0C => Some(b"\\f"),
        0x08 => Some(b"\\b"),
        0x0D => Some(b"\\r"),
        0x0A => Some(b"\\n"),
        0x5C => Some(b"\\\\"),
        0x22 => Some(b"\\\""),
        0x27 => Some(b"\\'"),
        0x60 => Some(b"\\`"),
        0x24 => Some(b"\\$"),
        0x2028 => Some(b"\\u2028"),
        0x2029 => Some(b"\\u2029"),
        0x0085 => Some(b"\\u0085"),
        _ => None,
    }
}

// port: tsc/internal/printer/utilities.go:escapeStringWorker
/// ECMA-262's `QuoteJSONString`, augmented as upstream augments it. Does not add
/// the surrounding quotes.
pub fn escape_string_worker(
    s: &[u8],
    quote_char: QuoteChar,
    flags: LiteralTextFlags,
    out: &mut Vec<u8>,
) {
    let quote = quote_char.rune();
    let mut pos = 0usize;
    let mut i = 0usize;
    while i < s.len() {
        let (ch, mut size) = decode_js_string_rune(&s[i..]);

        let mut escape = false;
        if (0xD800..=0xDFFF).contains(&ch) {
            escape = true;
        } else if ch == RUNE_ERROR && size == 1 {
            // A stray byte that is not valid UTF-8, for example a fragment of a
            // sentinel left behind by byte slicing. Escaped as the replacement
            // character so the output is always well formed.
            escape = true;
        }

        match ch {
            0x5C => {
                if !flags.contains(LiteralTextFlags::JSX_ATTRIBUTE_ESCAPE) {
                    escape = true;
                }
            }
            0x24 => {
                if quote_char == QuoteChar::Backtick && i + 1 < s.len() && s[i + 1] == b'{' {
                    escape = true;
                }
            }
            0x0A => {
                if quote_char != QuoteChar::Backtick {
                    // Template strings preserve simple LF newlines and still
                    // encode CRLF or CR.
                    escape = true;
                }
            }
            // Upstream lists the quote character and the separators in one
            // case and falls through to the default's range test; both only
            // ever set the flag, so they are one condition here.
            _ => {
                if ch == quote
                    || ch == 0x2028
                    || ch == 0x2029
                    || ch == 0x0085
                    || ch == 0x0D
                    || ch <= 0x1F
                    || (!flags.contains(LiteralTextFlags::NEVER_ASCII_ESCAPE) && ch > 0x7F)
                {
                    escape = true;
                }
            }
        }

        if escape {
            if pos < i {
                out.extend_from_slice(&s[pos..i]);
            }
            if flags.contains(LiteralTextFlags::JSX_ATTRIBUTE_ESCAPE) {
                if ch == 0 {
                    out.extend_from_slice(b"&#0;");
                } else if let Some(entity) = jsx_escaped_char(ch) {
                    out.extend_from_slice(entity);
                } else {
                    encode_jsx_character_entity(out, ch);
                }
            } else if ch == 0x0D
                && quote_char == QuoteChar::Backtick
                && i + 1 < s.len()
                && s[i + 1] == b'\n'
            {
                // Left alone, the `\r` and `\n` cases would escape CRLF as two
                // independent characters.
                size += 1;
                out.extend_from_slice(b"\\r\\n");
            } else if ch > 0xFFFF {
                let ch = ch - 0x1_0000;
                encode_utf16_escape_sequence(
                    out,
                    ((ch & 0b1111_1111_1100_0000_0000) >> 10) + 0xD800,
                );
                encode_utf16_escape_sequence(out, (ch & 0b0000_0000_0011_1111_1111) + 0xDC00);
            } else if (0xD800..=0xDFFF).contains(&ch) {
                encode_utf16_escape_sequence(out, ch);
            } else if ch == 0 {
                if i + 1 < s.len() && s[i + 1].is_ascii_digit() {
                    // A null followed by digits is printed as a hex escape so the
                    // result cannot parse as an octal escape in strict mode.
                    out.extend_from_slice(b"\\x00");
                } else {
                    out.extend_from_slice(b"\\0");
                }
            } else if let Some(escaped) = escaped_char(ch) {
                out.extend_from_slice(escaped);
            } else {
                encode_utf16_escape_sequence(out, ch);
            }
            pos = i + size;
        }

        i += size;
    }

    if pos < i {
        out.extend_from_slice(&s[pos..]);
    }
}

// port: tsc/internal/printer/utilities.go:EscapeString
/// `printer.EscapeString`: escape without ASCII-escaping non-ASCII characters.
pub fn escape_string(s: &[u8], quote_char: QuoteChar) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len() + 2);
    escape_string_worker(
        s,
        quote_char,
        LiteralTextFlags::NEVER_ASCII_ESCAPE,
        &mut out,
    );
    out
}

// port: tsc/internal/printer/utilities.go:escapeNonAsciiString
/// Escape non-ASCII characters as well.
pub fn escape_non_ascii_string(s: &[u8], quote_char: QuoteChar) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len() + 2);
    escape_string_worker(s, quote_char, LiteralTextFlags::NONE, &mut out);
    out
}

// port: tsc/internal/printer/utilities.go:escapeJsxAttributeString
/// Escape as a JSX attribute value.
pub fn escape_jsx_attribute_string(s: &[u8], quote_char: QuoteChar) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len() + 2);
    escape_string_worker(
        s,
        quote_char,
        LiteralTextFlags::JSX_ATTRIBUTE_ESCAPE.union(LiteralTextFlags::NEVER_ASCII_ESCAPE),
        &mut out,
    );
    out
}
