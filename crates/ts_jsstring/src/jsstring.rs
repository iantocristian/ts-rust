//! `JsString`: the type of every JavaScript string value (docs/design/text.md, 2.2).

use std::borrow::Cow;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::wtf8::{combine_surrogate_pairs, decode_js_string_rune, Rune, RUNE_ERROR};

/// The validity tag computed once at construction.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Validity {
    /// Valid UTF-8, the common case.
    Utf8,
    /// Valid UTF-8 except for WTF-8 sentinels standing for lone surrogates.
    Wtf8,
    /// Contains bytes that are neither: scanner raw copies and byte-slice fragments.
    Raw,
}

/// Classify bytes without assuming anything about their origin.
pub fn classify(bytes: &[u8]) -> Validity {
    if std::str::from_utf8(bytes).is_ok() {
        return Validity::Utf8;
    }
    let mut i = 0;
    while i < bytes.len() {
        let (r, size) = decode_js_string_rune(&bytes[i..]);
        if r == RUNE_ERROR && size == 1 {
            return Validity::Raw;
        }
        i += size;
    }
    Validity::Wtf8
}

/// Cheaply shared immutable bytes plus their validity tag. Equality, ordering and
/// hashing are byte-wise, as Go string comparisons are. There is deliberately no
/// `Deref<Target = str>`, `Display` or `From<JsString> for String`.
#[derive(Clone)]
pub struct JsString {
    bytes: Arc<[u8]>,
    tag: Validity,
}

impl JsString {
    /// Classifies and shares the bytes.
    pub fn from_bytes(bytes: impl Into<Arc<[u8]>>) -> Self {
        let bytes: Arc<[u8]> = bytes.into();
        let tag = classify(&bytes);
        Self { bytes, tag }
    }

    /// A `&str` is valid UTF-8 by construction; no scan is needed.
    pub fn from_text(s: &str) -> Self {
        Self {
            bytes: Arc::from(s.as_bytes()),
            tag: Validity::Utf8,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn tag(&self) -> Validity {
        self.tag
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// The `&str` view exists only for `Utf8`-tagged strings.
    pub fn as_str(&self) -> Option<&str> {
        match self.tag {
            Validity::Utf8 => std::str::from_utf8(&self.bytes).ok(),
            Validity::Wtf8 | Validity::Raw => None,
        }
    }

    /// A byte slice with checked bounds. The result does not inherit this
    /// string's tag: it is reclassified, and can be `Raw` even when the parent
    /// is `Utf8`, because the endpoints may split a sequence.
    ///
    /// # Panics
    /// When the range is out of bounds, like a Go slice expression.
    #[must_use]
    pub fn slice(&self, range: std::ops::Range<usize>) -> Self {
        Self::from_bytes(&self.bytes[range])
    }

    /// The `&str` view of a byte range, available only when both endpoints are
    /// character boundaries of a `Utf8` string.
    pub fn slice_str(&self, range: std::ops::Range<usize>) -> Option<&str> {
        let s = self.as_str()?;
        if s.is_char_boundary(range.start) && s.is_char_boundary(range.end) {
            s.get(range)
        } else {
            None
        }
    }

    /// Code points through the sentinel-aware decoder, each with its byte size.
    pub fn runes(&self) -> Runes<'_> {
        Runes {
            bytes: &self.bytes,
            pos: 0,
        }
    }

    /// Joins separately produced values the way JavaScript concatenation does:
    /// adjacent lone surrogates become one supplementary code point.
    pub fn concat<'a>(parts: impl IntoIterator<Item = &'a JsString>) -> Self {
        let mut joined = Vec::new();
        for part in parts {
            joined.extend_from_slice(&part.bytes);
        }
        match combine_surrogate_pairs(&joined) {
            Cow::Borrowed(_) => Self::from_bytes(joined),
            Cow::Owned(combined) => Self::from_bytes(combined),
        }
    }
}

impl PartialEq for JsString {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

impl Eq for JsString {}

impl PartialOrd for JsString {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for JsString {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.bytes.cmp(&other.bytes)
    }
}

impl Hash for JsString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}

impl std::fmt::Debug for JsString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JsString({:?}, {})", self.tag, self.bytes.escape_ascii())
    }
}

/// Iterator over `(rune, byte size)` pairs.
pub struct Runes<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl Iterator for Runes<'_> {
    type Item = (Rune, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.bytes.len() {
            return None;
        }
        let (r, size) = decode_js_string_rune(&self.bytes[self.pos..]);
        self.pos += size;
        Some((r, size))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_and_views() {
        let utf8 = JsString::from_bytes("héllo".as_bytes());
        assert_eq!(utf8.tag(), Validity::Utf8);
        assert_eq!(utf8.as_str(), Some("héllo"));
        let wtf8 = JsString::from_bytes(&b"a\xED\xA0\x80b"[..]);
        assert_eq!(wtf8.tag(), Validity::Wtf8);
        assert_eq!(wtf8.as_str(), None);
        let raw = JsString::from_bytes(&b"a\xFFb"[..]);
        assert_eq!(raw.tag(), Validity::Raw);
    }

    #[test]
    fn slices_are_reclassified() {
        let s = JsString::from_text("a😀b");
        assert_eq!(s.slice(0..1).tag(), Validity::Utf8);
        assert_eq!(
            s.slice(0..2).tag(),
            Validity::Raw,
            "cuts inside the astral character"
        );
        assert_eq!(s.slice(2..6).tag(), Validity::Raw);
        assert_eq!(s.slice(0..2).as_bytes(), &[b'a', 0xF0]);
        assert!(s.slice_str(0..2).is_none());
        assert_eq!(s.slice_str(1..5), Some("😀"));
        let w = JsString::from_bytes(&b"\xED\xA0\x80x"[..]);
        assert_eq!(w.slice(1..4).tag(), Validity::Raw, "splits the sentinel");
        assert_eq!(w.slice(3..4).tag(), Validity::Utf8);
    }

    #[test]
    fn concat_combines_pairs() {
        let high = JsString::from_bytes(&b"\xED\xA0\xBD"[..]);
        let low = JsString::from_bytes(&b"\xED\xB8\x80"[..]);
        let joined = JsString::concat([&high, &low]);
        assert_eq!(joined.as_str(), Some("😀"));
    }
}
