//! Isolated byte-text prototype; local words are meaningful only in their owner.
//!
//! The enclosing node store must resolve a public handle's owner before calling
//! this crate-private API. It supplies the current stable slot, word and end;
//! copying a local word into another owner is not an import operation.

use std::collections::HashMap;
use std::fmt;
use std::mem::size_of;
use std::ops::Range;
use std::sync::Arc;

const MAX_TAGGED_VALUE: usize = (u32::MAX >> 1) as usize;
const EXTENDED_POOL_WORD: u32 = u32::MAX;

/// Four bytes in the node payload; positions already belong to its header.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub(crate) struct TextWord(u32);

impl TextWord {
    fn raw(byte_length: usize) -> Option<Self> {
        let length = u32::try_from(byte_length).ok()?;
        (byte_length <= MAX_TAGGED_VALUE).then_some(Self(length << 1))
    }

    fn pool(index: usize, inline_limit: usize) -> Option<Self> {
        if index >= inline_limit.min(MAX_TAGGED_VALUE) {
            return None;
        }
        let index = u32::try_from(index).ok()?;
        Some(Self((index << 1) | 1))
    }

    pub(crate) fn is_raw(self) -> bool {
        self.0 & 1 == 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TextError {
    InvalidRawRange,
    UnknownPoolEntry,
    UnknownExtendedSlot,
    LengthOverflow,
    AllocationFailed,
}

impl fmt::Display for TextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidRawRange => "identifier source suffix is out of bounds",
            Self::UnknownPoolEntry => "identifier pool entry does not exist",
            Self::UnknownExtendedSlot => "identifier slot has no extended pool entry",
            Self::LengthOverflow => "identifier pool byte length overflow",
            Self::AllocationFailed => "identifier pool allocation failed",
        })
    }
}

impl std::error::Error for TextError {}

#[derive(Clone, Copy, Debug)]
struct PoolEntry {
    byte_offset: usize,
    byte_length: usize,
}

#[derive(Default)]
struct TextPool {
    bytes: Vec<u8>,
    entries: Vec<PoolEntry>,
    extended: HashMap<u32, usize>,
}

impl TextPool {
    fn insert(&mut self, slot: u32, bytes: &[u8]) -> Result<TextWord, TextError> {
        self.insert_with_limit(slot, bytes, MAX_TAGGED_VALUE)
    }

    // The smaller limit in countertests exercises the actual overflow route
    // without allocating billions of entries. Production callers use the full
    // tag domain, reserving only the odd all-ones word for the slot resolver.
    fn insert_with_limit(
        &mut self,
        slot: u32,
        bytes: &[u8],
        inline_limit: usize,
    ) -> Result<TextWord, TextError> {
        self.bytes
            .len()
            .checked_add(bytes.len())
            .ok_or(TextError::LengthOverflow)?;
        let index = self.entries.len();
        let inline = TextWord::pool(index, inline_limit);
        self.entries
            .try_reserve(1)
            .map_err(|_| TextError::AllocationFailed)?;
        self.bytes
            .try_reserve(bytes.len())
            .map_err(|_| TextError::AllocationFailed)?;
        if inline.is_none() {
            self.extended
                .try_reserve(1)
                .map_err(|_| TextError::AllocationFailed)?;
        }

        // Publish no entry/word until all fallible reservations have succeeded.
        let byte_offset = self.bytes.len();
        self.bytes.extend_from_slice(bytes);
        self.entries.push(PoolEntry {
            byte_offset,
            byte_length: bytes.len(),
        });
        Ok(match inline {
            Some(word) => word,
            None => {
                self.extended.insert(slot, index);
                TextWord(EXTENDED_POOL_WORD)
            }
        })
    }

    fn get(&self, slot: u32, word: TextWord) -> Result<&[u8], TextError> {
        let index = if word.0 == EXTENDED_POOL_WORD {
            *self
                .extended
                .get(&slot)
                .ok_or(TextError::UnknownExtendedSlot)?
        } else {
            usize::try_from(word.0 >> 1).map_err(|_| TextError::UnknownPoolEntry)?
        };
        let entry = self.entries.get(index).ok_or(TextError::UnknownPoolEntry)?;
        let end = entry
            .byte_offset
            .checked_add(entry.byte_length)
            .ok_or(TextError::LengthOverflow)?;
        self.bytes
            .get(entry.byte_offset..end)
            .ok_or(TextError::UnknownPoolEntry)
    }
}

/// Capacity observations, not allocator/RSS measurements. Source bytes are
/// shared backing and must be charged only once by the enclosing owner census.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TextStats {
    pub(crate) source_bytes: usize,
    pub(crate) pool_bytes: usize,
    pub(crate) pool_capacity_bytes: usize,
    pub(crate) pool_entries: usize,
    pub(crate) pool_entry_capacity_bytes: usize,
    pub(crate) extended_entries: usize,
    pub(crate) extended_capacity_entries: usize,
}

/// One retained source allocation per owner. Exceptional values are copied into
/// the pool, so they outlive scanner temporaries and foreign source owners.
pub(crate) struct TextStore {
    source: Arc<[u8]>,
    pool: TextPool,
}

impl TextStore {
    pub(crate) fn new(source: Arc<[u8]>) -> Self {
        Self {
            source,
            pool: TextPool::default(),
        }
    }

    /// `None` is a factory node whose final range is not yet established. Its
    /// bytes remain immediately observable through the returned pooled word.
    /// If a parser already knows its end, insertion verifies the actual suffix.
    pub(crate) fn insert(
        &mut self,
        slot: u32,
        bytes: &[u8],
        end: Option<i32>,
    ) -> Result<TextWord, TextError> {
        if let Some(word) = end.and_then(|end| self.matching_raw(bytes, end)) {
            return Ok(word);
        }
        self.pool.insert(slot, bytes)
    }

    pub(crate) fn bytes(&self, slot: u32, word: TextWord, end: i32) -> Result<&[u8], TextError> {
        if word.is_raw() {
            Ok(&self.source[self.raw_range(word, end)?])
        } else {
            self.pool.get(slot, word)
        }
    }

    /// Compute the replacement word before the owner writes the new end. This
    /// also finalizes a factory range. An error leaves the existing word/end
    /// valid. Pool entries made obsolete by compaction remain charged in stats.
    pub(crate) fn change_end(
        &mut self,
        slot: u32,
        word: TextWord,
        old_end: i32,
        new_end: i32,
    ) -> Result<TextWord, TextError> {
        let bytes = self.bytes(slot, word, old_end)?;
        if let Some(raw) = self.matching_raw(bytes, new_end) {
            return Ok(raw);
        }
        if !word.is_raw() {
            return Ok(word);
        }
        let range = self.raw_range(word, old_end)?;
        self.pool.insert(slot, &self.source[range])
    }

    /// Explicit escape copies the selected bytes into independent ownership.
    /// Ordinary `bytes()` reads neither allocate nor increment an Arc count.
    pub(crate) fn retain(
        &self,
        slot: u32,
        word: TextWord,
        end: i32,
    ) -> Result<Arc<[u8]>, TextError> {
        Ok(Arc::from(self.bytes(slot, word, end)?))
    }

    pub(crate) fn stats(&self) -> TextStats {
        TextStats {
            source_bytes: self.source.len(),
            pool_bytes: self.pool.bytes.len(),
            pool_capacity_bytes: self.pool.bytes.capacity(),
            pool_entries: self.pool.entries.len(),
            pool_entry_capacity_bytes: self.pool.entries.capacity() * size_of::<PoolEntry>(),
            extended_entries: self.pool.extended.len(),
            extended_capacity_entries: self.pool.extended.capacity(),
        }
    }

    fn matching_raw(&self, bytes: &[u8], end: i32) -> Option<TextWord> {
        let word = TextWord::raw(bytes.len())?;
        let range = self.raw_range(word, end).ok()?;
        (self.source.get(range)? == bytes).then_some(word)
    }

    fn raw_range(&self, word: TextWord, end: i32) -> Result<Range<usize>, TextError> {
        let end = usize::try_from(end).map_err(|_| TextError::InvalidRawRange)?;
        let length = usize::try_from(word.0 >> 1).map_err(|_| TextError::InvalidRawRange)?;
        let start = end.checked_sub(length).ok_or(TextError::InvalidRawRange)?;
        if end > self.source.len() {
            return Err(TextError::InvalidRawRange);
        }
        Ok(start..end)
    }
}

#[cfg(test)]
#[path = "text-tests.rs"]
mod tests;
