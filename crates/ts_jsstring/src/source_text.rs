//! port: tsc/internal/vfs/internal/internal.go (file decoding)
//!
//! `SourceText` owns a file's bytes after the same BOM handling as upstream's
//! `decodeBytes` (docs/design/text.md, 2.1).

use std::sync::Arc;

use crate::jsstring::JsString;
use crate::wtf8::{surrogate_pair_to_code_point, write_rune, Rune};

/// port: tsc/internal/vfs/internal/internal.go:decodeBytes
///
/// A UTF-16 LE or BE BOM selects `decode_utf16`; a UTF-8 BOM is dropped; any
/// other bytes are returned unchanged and nothing validates UTF-8.
pub fn decode_bytes(s: &[u8]) -> Vec<u8> {
    if s.len() >= 2 {
        match [s[0], s[1]] {
            [0xFF, 0xFE] => return decode_utf16(&s[2..], Endian::Little),
            [0xFE, 0xFF] => return decode_utf16(&s[2..], Endian::Big),
            _ => {}
        }
    }
    if s.len() >= 3 && s[0] == 0xEF && s[1] == 0xBB && s[2] == 0xBF {
        return s[3..].to_vec();
    }
    s.to_vec()
}

#[derive(Clone, Copy)]
pub enum Endian {
    Little,
    Big,
}

/// port: tsc/internal/vfs/internal/internal.go:decodeUtf16
///
/// Go reads `len(s)/2` units, so a trailing odd byte is dropped, then
/// `utf16.Decode` replaces every unpaired surrogate with U+FFFD and `string`
/// encodes the result as UTF-8.
pub fn decode_utf16(s: &[u8], order: Endian) -> Vec<u8> {
    let units: Vec<u16> = s
        .chunks_exact(2)
        .map(|pair| match order {
            Endian::Little => u16::from_le_bytes([pair[0], pair[1]]),
            Endian::Big => u16::from_be_bytes([pair[0], pair[1]]),
        })
        .collect();
    let mut out = Vec::with_capacity(units.len());
    let mut i = 0;
    while i < units.len() {
        let u = Rune::from(units[i]);
        if (0xD800..0xDC00).contains(&u)
            && i + 1 < units.len()
            && (0xDC00..0xE000).contains(&Rune::from(units[i + 1]))
        {
            write_rune(
                &mut out,
                surrogate_pair_to_code_point(u, Rune::from(units[i + 1])),
            );
            i += 2;
        } else {
            // write_rune maps a lone surrogate to U+FFFD, as utf16.Decode does.
            write_rune(&mut out, u);
            i += 1;
        }
    }
    out
}

/// Immutable, shared file text after BOM handling. Positions into it are byte
/// offsets; slicing by an offset that is not a character boundary is allowed and
/// yields a reclassified [`JsString`].
#[derive(Clone)]
pub struct SourceText {
    bytes: Arc<[u8]>,
    valid_utf8: bool,
}

impl SourceText {
    /// Decodes raw file bytes exactly as upstream does before scanning.
    pub fn from_file_bytes(raw: &[u8]) -> Self {
        Self::from_decoded(decode_bytes(raw))
    }

    /// Wraps bytes that have already been decoded.
    pub fn from_decoded(bytes: impl Into<Arc<[u8]>>) -> Self {
        let bytes: Arc<[u8]> = bytes.into();
        let valid_utf8 = std::str::from_utf8(&bytes).is_ok();
        Self { bytes, valid_utf8 }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn is_valid_utf8(&self) -> bool {
        self.valid_utf8
    }

    /// The whole text as `&str`, only when it is valid UTF-8.
    pub fn as_str(&self) -> Option<&str> {
        if self.valid_utf8 {
            std::str::from_utf8(&self.bytes).ok()
        } else {
            None
        }
    }

    /// A byte range as a reclassified string.
    ///
    /// # Panics
    /// When the range is out of bounds.
    pub fn slice(&self, range: std::ops::Range<usize>) -> JsString {
        JsString::from_bytes(&self.bytes[range])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boms() {
        assert_eq!(decode_bytes(b"\xEF\xBB\xBFabc"), b"abc");
        assert_eq!(decode_bytes(b"\xFF\xFEa\x00b\x00"), b"ab");
        assert_eq!(decode_bytes(b"\xFE\xFF\x00a\x00b"), b"ab");
        assert_eq!(
            decode_bytes(b"\xFF\xFE\x00\xD8a\x00"),
            "\u{FFFD}a".as_bytes(),
            "unpaired surrogate under a UTF-16 BOM"
        );
        assert_eq!(decode_bytes(b"\xFF\xFE\x3D\xD8\x00\xDE"), "😀".as_bytes());
        assert_eq!(
            decode_bytes(b"\xFF\xFEa\x00b"),
            b"a",
            "odd trailing byte is dropped"
        );
        assert_eq!(
            decode_bytes(b"\xFFabc"),
            b"\xFFabc",
            "no BOM: bytes unchanged, nothing validated"
        );
    }
}
