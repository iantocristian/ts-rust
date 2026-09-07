//! Immutable JavaScript value bytes; each view carries its own validity tag.

use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::ops::Range;
use std::sync::Arc;

use crate::wtf8::{code_points, decode_rune, CodePoints, RUNE_ERROR};

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
        let length = storage.len();
        Self::from_shared_range(storage, 0..length)
    }

    pub(crate) fn from_shared_range(storage: Arc<[u8]>, range: Range<usize>) -> Self {
        let validity = classify(&storage[range.clone()]);
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
