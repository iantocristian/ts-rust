//! The decoder is chosen by the operation, independently of a string's validity tag.

use std::borrow::Cow;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::ops::Range;
use std::sync::Arc;

pub const RUNE_ERROR: i32 = 0xfffd;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Validity {
    Utf8,
    Wtf8,
    Raw,
}

/// Immutable byte string; clones and byte slices share the original allocation.
#[derive(Clone, Debug)]
pub struct JsString {
    storage: Arc<[u8]>,
    range: Range<usize>,
    validity: Validity,
}

impl JsString {
    pub fn from_bytes(bytes: impl Into<Arc<[u8]>>) -> Self {
        let storage = bytes.into();
        let validity = classify(&storage);
        let range = 0..storage.len();
        Self {
            storage,
            range,
            validity,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.storage[self.range.clone()]
    }

    pub fn as_str(&self) -> Option<&str> {
        if self.validity == Validity::Utf8 {
            // Revalidate the safe view; the crate does not need unchecked UTF-8 access.
            std::str::from_utf8(self.as_bytes()).ok()
        } else {
            None
        }
    }

    pub fn validity(&self) -> Validity {
        self.validity
    }
    pub fn len(&self) -> usize {
        self.range.len()
    }
    pub fn is_empty(&self) -> bool {
        self.range.is_empty()
    }

    /// Bounds are checked in bytes; endpoints need not be character boundaries.
    pub fn slice(&self, range: Range<usize>) -> Option<Self> {
        let bytes = self.as_bytes().get(range.clone())?;
        Some(Self {
            storage: Arc::clone(&self.storage),
            range: self.range.start + range.start..self.range.start + range.end,
            validity: classify(bytes),
        })
    }

    pub fn code_points(&self) -> CodePoints<'_> {
        code_points(self.as_bytes())
    }
}

impl Default for JsString {
    fn default() -> Self {
        Self::from_bytes(&[][..])
    }
}

impl AsRef<[u8]> for JsString {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl PartialEq for JsString {
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}
impl Eq for JsString {}
impl PartialOrd for JsString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for JsString {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_bytes().cmp(other.as_bytes())
    }
}
impl Hash for JsString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state);
    }
}

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
            let validity = classify(&bytes[start..]);
            let end = bytes.len();
            Self(JsString {
                storage: bytes,
                range: start..end,
                validity,
            })
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
        .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
        .collect::<String>()
        .into_bytes()
}

pub fn classify(bytes: &[u8]) -> Validity {
    if std::str::from_utf8(bytes).is_ok() {
        return Validity::Utf8;
    }
    let mut offset = 0;
    while offset < bytes.len() {
        let (rune, width) = decode_rune(&bytes[offset..]);
        if rune == RUNE_ERROR && width == 1 {
            return Validity::Raw;
        }
        offset += width;
    }
    Validity::Wtf8
}

/// Go's unicode/utf8.DecodeRuneInString: invalid input consumes one byte.
/// The empty input returns (RuneError, 0), unlike a malformed nonempty prefix.
pub fn decode_utf8(bytes: &[u8]) -> (i32, usize) {
    let Some(&first) = bytes.first() else {
        return (RUNE_ERROR, 0);
    };
    if first < 0x80 {
        return (i32::from(first), 1);
    }
    let (width, second_min, second_max, mask) = match first {
        0xc2..=0xdf => (2, 0x80, 0xbf, 0x1f),
        0xe0 => (3, 0xa0, 0xbf, 0x0f),
        0xe1..=0xec | 0xee..=0xef => (3, 0x80, 0xbf, 0x0f),
        0xed => (3, 0x80, 0x9f, 0x0f),
        0xf0 => (4, 0x90, 0xbf, 0x07),
        0xf1..=0xf3 => (4, 0x80, 0xbf, 0x07),
        0xf4 => (4, 0x80, 0x8f, 0x07),
        _ => return (RUNE_ERROR, 1),
    };
    if bytes.len() < width || !(second_min..=second_max).contains(&bytes[1]) {
        return (RUNE_ERROR, 1);
    }
    if bytes[2..width].iter().any(|b| !(0x80..=0xbf).contains(b)) {
        return (RUNE_ERROR, 1);
    }
    let mut rune = i32::from(first & mask);
    for byte in &bytes[1..width] {
        rune = (rune << 6) | i32::from(byte & 0x3f);
    }
    (rune, width)
}

/// port: tsc/internal/stringutil/util.go:DecodeJSStringRune
pub fn decode_rune(bytes: &[u8]) -> (i32, usize) {
    if bytes.len() >= 3
        && bytes[0] == 0xed
        && (0xa0..=0xbf).contains(&bytes[1])
        && (0x80..=0xbf).contains(&bytes[2])
    {
        return (
            0xd000 | (i32::from(bytes[1] & 0x3f) << 6) | i32::from(bytes[2] & 0x3f),
            3,
        );
    }
    decode_utf8(bytes)
}

/// port: tsc/internal/stringutil/util.go:IsSurrogate
pub fn is_surrogate(rune: i32) -> bool {
    (0xd800..=0xdfff).contains(&rune)
}
/// port: tsc/internal/stringutil/util.go:IsHighSurrogate
pub fn is_high_surrogate(rune: i32) -> bool {
    (0xd800..=0xdbff).contains(&rune)
}
/// port: tsc/internal/stringutil/util.go:IsLowSurrogate
pub fn is_low_surrogate(rune: i32) -> bool {
    (0xdc00..=0xdfff).contains(&rune)
}

/// port: tsc/internal/stringutil/util.go:SurrogatePairToCodePoint
pub fn surrogate_pair_to_code_point(high: i32, low: i32) -> i32 {
    if is_high_surrogate(high) && is_low_surrogate(low) {
        ((high - 0xd800) << 10) + low - 0xdc00 + 0x10000
    } else {
        RUNE_ERROR
    }
}

/// port: tsc/internal/stringutil/util.go:CodePointToSurrogatePair
pub fn code_point_to_surrogate_pair(rune: i32) -> (i32, i32) {
    if (0x0001_0000..=0x0010_ffff).contains(&rune) {
        let value = rune - 0x10000;
        (0xd800 + (value >> 10), 0xdc00 + (value & 0x3ff))
    } else {
        (RUNE_ERROR, RUNE_ERROR)
    }
}

/// port: tsc/internal/stringutil/util.go:EncodeJSStringRune
pub fn encode_rune(rune: i32) -> Vec<u8> {
    let mut result = Vec::with_capacity(4);
    append_rune(&mut result, rune);
    result
}

pub(crate) fn append_rune(output: &mut Vec<u8>, rune: i32) {
    if is_surrogate(rune) {
        output.extend_from_slice(&[
            0xed,
            0x80 | ((rune >> 6) & 0x3f) as u8,
            0x80 | (rune & 0x3f) as u8,
        ]);
    } else {
        let character = char::from_u32(rune as u32).unwrap_or(char::REPLACEMENT_CHARACTER);
        let mut buffer = [0; 4];
        output.extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
    }
}

pub struct CodePoints<'a> {
    remaining: &'a [u8],
}

pub fn code_points(bytes: &[u8]) -> CodePoints<'_> {
    CodePoints { remaining: bytes }
}

impl Iterator for CodePoints<'_> {
    type Item = i32;
    fn next(&mut self) -> Option<Self::Item> {
        let (rune, width) = decode_rune(self.remaining);
        if width == 0 {
            return None;
        }
        self.remaining = &self.remaining[width..];
        Some(rune)
    }
}

/// port: tsc/internal/stringutil/util.go:CombineSurrogatePairs
pub fn combine_surrogate_pairs(bytes: &[u8]) -> Cow<'_, [u8]> {
    if !bytes.contains(&0xed) {
        return Cow::Borrowed(bytes);
    }
    let mut output = Vec::with_capacity(bytes.len());
    let mut offset = 0;
    while offset < bytes.len() {
        let (rune, width) = decode_rune(&bytes[offset..]);
        if is_high_surrogate(rune) {
            let (low, low_width) = decode_rune(&bytes[offset + width..]);
            if is_low_surrogate(low) {
                append_rune(&mut output, surrogate_pair_to_code_point(rune, low));
                offset += width + low_width;
                continue;
            }
        }
        output.extend_from_slice(&bytes[offset..offset + width]);
        offset += width;
    }
    Cow::Owned(output)
}
