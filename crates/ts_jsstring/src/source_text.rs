//! BOM decoding for immutable compiler source bytes.

use std::ops::Range;
use std::sync::Arc;

use crate::jsstring::{JsString, Validity};

/// A file's immutable bytes after Go's BOM decoding, including raw malformed UTF-8.
#[derive(Clone, Debug, Default)]
pub struct SourceText(JsString);

impl SourceText {
    /// port: tsc/internal/vfs/internal/internal.go:decodeBytes
    pub fn from_bytes(bytes: impl Into<Arc<[u8]>>) -> Self {
        let bytes = bytes.into();
        if bytes.starts_with(&[0xff, 0xfe]) {
            Self(JsString::from_bytes(decode_utf16(&bytes[2..], true)))
        } else if bytes.starts_with(&[0xfe, 0xff]) {
            Self(JsString::from_bytes(decode_utf16(&bytes[2..], false)))
        } else {
            let start = if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
                3
            } else {
                0
            };
            let end = bytes.len();
            Self(JsString::from_shared_range(bytes, start..end))
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
    pub fn as_str(&self) -> Option<&str> {
        self.0.as_str()
    }
    pub fn validity(&self) -> Validity {
        self.0.validity()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn slice(&self, range: Range<usize>) -> Option<JsString> {
        self.0.slice(range)
    }
}

/// port: tsc/internal/vfs/internal/internal.go:decodeUtf16
fn decode_utf16(bytes: &[u8], little_endian: bool) -> Vec<u8> {
    // Go allocates len(bytes)/2 words and binary.Read reads that exact word count.
    // Consequently an odd final byte is discarded, including a one-byte payload.
    let units = bytes.chunks_exact(2).map(|pair| {
        let pair = [pair[0], pair[1]];
        if little_endian {
            u16::from_le_bytes(pair)
        } else {
            u16::from_be_bytes(pair)
        }
    });
    char::decode_utf16(units)
        .map(|rune| rune.unwrap_or(char::REPLACEMENT_CHARACTER))
        .collect::<String>()
        .into_bytes()
}
