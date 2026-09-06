//! `JsString`: the type of every JavaScript string value (docs/design/text.md,
//! section 2.2).
//!
//! The bytes are immutable and cheaply shared, and a validity tag is computed
//! once at construction. There is no implicit conversion to `String`, `&str` or
//! `Cow<str>`: code that needs an `&str` asks for it and handles `None`.

use std::fmt;
use std::sync::Arc;

use crate::rune::{decode_js_string_rune, is_surrogate, JsRunes, Rune, RUNE_ERROR};

/// How the bytes of a [`JsString`] relate to UTF-8.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Validity {
    /// Valid UTF-8, the common case. `&str` and byte views are available.
    Utf8,
    /// Valid UTF-8 except for WTF-8 sentinels encoding lone surrogates. Bytes
    /// and sentinel-aware code-point iteration are available; `&str` is not.
    Wtf8,
    /// Bytes that are neither, including scanner raw-copy results and fragments
    /// produced by byte slicing. Bytes only.
    Raw,
}

/// Classify bytes without constructing a string.
///
/// `Wtf8` means every position decodes through the sentinel-aware decoder as
/// either a valid UTF-8 sequence or a lone-surrogate sentinel; a byte the
/// standard decoder rejects and the sentinel decoder does not recognize makes
/// the whole string `Raw`.
pub fn classify(bytes: &[u8]) -> Validity {
    if std::str::from_utf8(bytes).is_ok() {
        return Validity::Utf8;
    }
    let mut i = 0;
    let mut saw_sentinel = false;
    while i < bytes.len() {
        let (r, size) = decode_js_string_rune(&bytes[i..]);
        if r == RUNE_ERROR && size == 1 {
            return Validity::Raw;
        }
        saw_sentinel = saw_sentinel || is_surrogate(r);
        i += size;
    }
    // Not valid UTF-8, yet every sequence decoded: the difference is sentinels.
    debug_assert!(saw_sentinel);
    Validity::Wtf8
}

/// A JavaScript string value: literal values, template text, identifier and
/// symbol names, string-literal types, property keys and encoder strings.
#[derive(Clone)]
pub struct JsString {
    bytes: Arc<[u8]>,
    validity: Validity,
}

impl JsString {
    /// Classify and take ownership of the bytes.
    pub fn new(bytes: impl Into<Arc<[u8]>>) -> Self {
        let bytes = bytes.into();
        let validity = classify(&bytes);
        Self { bytes, validity }
    }

    /// The empty string.
    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    /// A `&str` is already known to be valid UTF-8, so no classification is
    /// needed. Not `FromStr`: this cannot fail and takes no parsing decision.
    pub fn from_text(text: &str) -> Self {
        Self {
            bytes: Arc::from(text.as_bytes()),
            validity: Validity::Utf8,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn validity(&self) -> Validity {
        self.validity
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// `Some` only for `Utf8` bytes. There is no lossy fallback.
    pub fn as_str(&self) -> Option<&str> {
        match self.validity {
            Validity::Utf8 => std::str::from_utf8(&self.bytes).ok(),
            Validity::Wtf8 | Validity::Raw => None,
        }
    }

    /// Code points through the sentinel-aware decoder, as `(offset, rune, size)`.
    pub fn code_points(&self) -> JsRunes<'_> {
        JsRunes::new(&self.bytes)
    }

    /// A byte-offset slice with checked bounds. The result is classified afresh:
    /// slicing a `Utf8` or `Wtf8` string can produce `Raw` bytes.
    pub fn slice(&self, start: usize, end: usize) -> Option<JsString> {
        if start > end || end > self.bytes.len() {
            return None;
        }
        Some(JsString::new(self.bytes[start..end].to_vec()))
    }

    /// Concatenation without any surrogate joining. Call
    /// [`crate::rune::combine_surrogate_pairs`] at the join points upstream does.
    #[must_use]
    pub fn concat(&self, other: &JsString) -> JsString {
        let mut bytes = Vec::with_capacity(self.len() + other.len());
        bytes.extend_from_slice(&self.bytes);
        bytes.extend_from_slice(&other.bytes);
        JsString::new(bytes)
    }
}

impl PartialEq for JsString {
    /// Byte-wise, as Go's string comparison is.
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

impl std::hash::Hash for JsString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}

impl fmt::Debug for JsString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JsString({:?}, ", self.validity)?;
        for &b in self.bytes.iter() {
            if b.is_ascii_graphic() || b == b' ' {
                write!(f, "{}", b as char)?;
            } else {
                write!(f, "\\x{b:02x}")?;
            }
        }
        write!(f, ")")
    }
}

/// The code point at a byte offset, for callers that walk by offset.
pub fn code_point_at(bytes: &[u8], offset: usize) -> (Rune, usize) {
    if offset >= bytes.len() {
        return (RUNE_ERROR, 0);
    }
    decode_js_string_rune(&bytes[offset..])
}
