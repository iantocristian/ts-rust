//! `SourceText`: a file's bytes after the same BOM handling `decodeBytes` does
//! (docs/design/text.md, section 2.1).
//!
//! Nothing validates UTF-8 upstream, so nothing validates it here either. The
//! bytes are kept and a validity flag is recorded once; positions stay byte
//! offsets in both cases.

use std::sync::Arc;

use crate::rune::{utf16_decode, write_rune};

/// The immutable, shared bytes of one file.
#[derive(Clone)]
pub struct SourceText {
    bytes: Arc<[u8]>,
    valid_utf8: bool,
}

/// Byte order marks recognized by `decodeBytes`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ByteOrderMark {
    None,
    Utf8,
    Utf16Le,
    Utf16Be,
}

impl SourceText {
    // port: tsc/internal/vfs/internal/internal.go:decodeBytes
    /// Decode a file's raw bytes: a UTF-16 BOM decodes through `utf16.Decode`,
    /// which replaces unpaired surrogates with U+FFFD; a UTF-8 BOM is dropped;
    /// everything else is kept byte for byte.
    pub fn decode(raw: &[u8]) -> Self {
        Self::from_bytes(Self::decode_bytes(raw).0)
    }

    /// [`SourceText::decode`] with the byte order mark it recognized.
    pub fn decode_with_bom(raw: &[u8]) -> (Self, ByteOrderMark) {
        let (bytes, bom) = Self::decode_bytes(raw);
        (Self::from_bytes(bytes), bom)
    }

    fn decode_bytes(raw: &[u8]) -> (Vec<u8>, ByteOrderMark) {
        if raw.len() >= 2 {
            match (raw[0], raw[1]) {
                (0xFF, 0xFE) => {
                    return (Self::decode_utf16(&raw[2..], false), ByteOrderMark::Utf16Le)
                }
                (0xFE, 0xFF) => {
                    return (Self::decode_utf16(&raw[2..], true), ByteOrderMark::Utf16Be)
                }
                _ => {}
            }
        }
        if raw.len() >= 3 && raw[0] == 0xEF && raw[1] == 0xBB && raw[2] == 0xBF {
            return (raw[3..].to_vec(), ByteOrderMark::Utf8);
        }
        (raw.to_vec(), ByteOrderMark::None)
    }

    // port: tsc/internal/vfs/internal/internal.go:decodeUtf16
    /// `binary.Read` fills `len(s)/2` code units, so a trailing odd byte is
    /// dropped; `utf16.Decode` then replaces unpaired surrogates with U+FFFD.
    fn decode_utf16(raw: &[u8], big_endian: bool) -> Vec<u8> {
        let units: Vec<u16> = raw
            .chunks_exact(2)
            .map(|pair| {
                if big_endian {
                    u16::from_be_bytes([pair[0], pair[1]])
                } else {
                    u16::from_le_bytes([pair[0], pair[1]])
                }
            })
            .collect();
        let mut out = Vec::with_capacity(units.len());
        for ch in utf16_decode(&units) {
            write_rune(&mut out, ch);
        }
        out
    }

    /// Take already-decoded bytes and record their validity.
    pub fn from_bytes(bytes: impl Into<Arc<[u8]>>) -> Self {
        let bytes = bytes.into();
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

    /// Whether the whole file is valid UTF-8. A valid whole file does not
    /// guarantee that a converted wire offset lies on a character boundary.
    pub fn is_valid_utf8(&self) -> bool {
        self.valid_utf8
    }

    /// `Some` only when the whole file is valid UTF-8.
    pub fn as_str(&self) -> Option<&str> {
        self.valid_utf8
            .then(|| std::str::from_utf8(&self.bytes).ok())
            .flatten()
    }

    /// A byte-offset slice with checked bounds. An `&str` view of the result
    /// requires both endpoints to be character boundaries, which this does not
    /// assume.
    pub fn slice(&self, start: usize, end: usize) -> Option<&[u8]> {
        if start > end || end > self.bytes.len() {
            return None;
        }
        Some(&self.bytes[start..end])
    }

    /// The shared bytes, for owners that hold the same text.
    pub fn shared(&self) -> Arc<[u8]> {
        Arc::clone(&self.bytes)
    }
}

impl std::fmt::Debug for SourceText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SourceText({} bytes, valid_utf8={})",
            self.bytes.len(),
            self.valid_utf8
        )
    }
}
