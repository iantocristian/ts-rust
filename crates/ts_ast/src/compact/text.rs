//! Identifier words select a source suffix or a reusable exceptional string slot.
use super::FieldKey;
use crate::JsString;
use std::collections::HashMap;
use std::ops::Range;
use ts_jsstring::SourceText;

const EXTENDED: u32 = u32::MAX;
const POOL_LIMIT: usize = (u32::MAX >> 1) as usize;

#[derive(Default)]
pub(super) struct TextPool {
    entries: Vec<Option<JsString>>,
    free: Vec<u32>,
    extended: HashMap<FieldKey, JsString>,
}

fn raw_range(word: u32, end: i32, source: &SourceText) -> Range<usize> {
    let end = usize::try_from(end).expect("source-backed identifier has a nonnegative end");
    let start = end
        .checked_sub((word >> 1) as usize)
        .expect("source-backed identifier length");
    assert!(
        end <= source.len(),
        "source-backed identifier ends within source"
    );
    start..end
}

fn matching_raw(bytes: &[u8], end: i32, source: &SourceText) -> Option<u32> {
    let end = usize::try_from(end).ok()?;
    let start = end.checked_sub(bytes.len())?;
    if bytes.len() <= POOL_LIMIT && source.as_bytes().get(start..end)? == bytes {
        Some(u32::try_from(bytes.len()).expect("checked identifier length") << 1)
    } else {
        None
    }
}

impl TextPool {
    fn pooled(&self, key: FieldKey, word: u32) -> &JsString {
        if word == EXTENDED {
            self.extended
                .get(&key)
                .expect("extended text word belongs to its field")
        } else {
            self.entries
                .get((word >> 1) as usize)
                .and_then(Option::as_ref)
                .expect("text word names an occupied pool slot")
        }
    }

    pub(super) fn bytes<'a>(
        &'a self,
        key: FieldKey,
        word: u32,
        end: i32,
        source: &'a SourceText,
    ) -> &'a [u8] {
        if word & 1 == 0 {
            &source.as_bytes()[raw_range(word, end, source)]
        } else {
            self.pooled(key, word).as_bytes()
        }
    }

    pub(super) fn owned(
        &self,
        key: FieldKey,
        word: u32,
        end: i32,
        source: &SourceText,
    ) -> JsString {
        if word & 1 == 0 {
            source
                .slice(raw_range(word, end, source))
                .expect("validated source suffix")
        } else {
            self.pooled(key, word).clone()
        }
    }

    pub(super) fn insert(
        &mut self,
        key: FieldKey,
        value: JsString,
        end: i32,
        source: &SourceText,
    ) -> u32 {
        if crate::AstPayloadStore::is_identifier_text(key.shape, key.field) {
            if let Some(word) = matching_raw(value.as_bytes(), end, source) {
                return word;
            }
        }
        self.insert_pool(key, value, POOL_LIMIT)
    }

    fn insert_pool(&mut self, key: FieldKey, value: JsString, limit: usize) -> u32 {
        if let Some(index) = self.free.pop() {
            self.entries[index as usize] = Some(value);
            return (index << 1) | 1;
        }
        if self.entries.len() < limit.min(POOL_LIMIT) {
            let index = u32::try_from(self.entries.len()).expect("bounded text pool");
            self.entries.push(Some(value));
            (index << 1) | 1
        } else {
            self.extended.insert(key, value);
            EXTENDED
        }
    }

    pub(super) fn release(&mut self, key: FieldKey, word: u32) {
        if word & 1 == 0 {
            return;
        }
        if word == EXTENDED {
            self.extended.remove(&key);
        } else {
            let index = word >> 1;
            let slot = self
                .entries
                .get_mut(index as usize)
                .expect("owned text pool slot");
            assert!(
                slot.take().is_some(),
                "text pool slot released exactly once"
            );
            self.free.push(index);
        }
    }

    pub(super) fn change_end(
        &mut self,
        key: FieldKey,
        word: u32,
        old_end: i32,
        new_end: i32,
        source: &SourceText,
    ) -> u32 {
        if !crate::AstPayloadStore::is_identifier_text(key.shape, key.field) {
            return word;
        }
        if let Some(raw) = matching_raw(self.bytes(key, word, old_end, source), new_end, source) {
            self.release(key, word);
            return raw;
        }
        if word & 1 != 0 {
            return word;
        }
        let value = self.owned(key, word, old_end, source);
        self.insert_pool(key, value, POOL_LIMIT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn final_ranges_release_temporary_text_and_reuse_exception_slots() {
        let source = SourceText::from_loaded_bytes(b"alpha beta alpha".as_slice());
        let mut pool = TextPool::default();
        for row in 0..1000 {
            let key = FieldKey::new(1, row, 0);
            let word = pool.insert(key, JsString::from_bytes(b"alpha".as_slice()), -1, &source);
            assert_eq!(word & 1, 1);
            let raw = pool.change_end(key, word, -1, 5, &source);
            assert_eq!(raw, 10);
            assert_eq!(pool.bytes(key, raw, 5, &source), b"alpha");
            assert_eq!(pool.entries.len(), 1);
            assert!(pool.entries[0].is_none());
        }
        let key = FieldKey::new(1, 1001, 0);
        let raw = pool.insert(key, JsString::from_bytes(b"alpha".as_slice()), 5, &source);
        // Moving the range over different bytes preserves semantic text.
        let escaped = pool.change_end(key, raw, 5, 10, &source);
        assert_eq!(escaped & 1, 1);
        assert_eq!(pool.bytes(key, escaped, 10, &source), b"alpha");
        let raw_again = pool.change_end(key, escaped, 10, 16, &source);
        assert_eq!(raw_again, raw);
        assert_eq!(pool.bytes(key, raw_again, 16, &source), b"alpha");
        assert!(pool.entries[0].is_none());
    }

    #[test]
    fn decoded_wtf8_and_full_pool_escape_preserve_bytes_and_field_identity() {
        let source = SourceText::from_loaded_bytes(b"\\u0061 #a".as_slice());
        let mut pool = TextPool::default();
        let first = FieldKey::new(1, 0, 0);
        let second = FieldKey::new(2, 0, 0);
        let first_word = pool.insert(first, JsString::from_bytes(b"a".as_slice()), 6, &source);
        assert_eq!(first_word & 1, 1);
        let second_word = pool.insert_pool(
            second,
            JsString::from_bytes(b"\xed\xa0\x80\xff".as_slice()),
            0,
        );
        assert_eq!(second_word, EXTENDED);
        assert_eq!(pool.bytes(first, first_word, 6, &source), b"a");
        let owned = pool.owned(second, second_word, -1, &source);
        assert_eq!(owned.as_bytes(), b"\xed\xa0\x80\xff");
        assert_eq!(
            owned.as_bytes().as_ptr(),
            pool.bytes(second, second_word, -1, &source).as_ptr()
        );
        assert_eq!(
            owned.validity(),
            pool.pooled(second, second_word).validity()
        );
        pool.release(first, first_word);
        pool.release(second, second_word);
        assert!(pool.extended.is_empty());
        drop(pool);
        assert_eq!(owned.as_bytes(), b"\xed\xa0\x80\xff");
    }
}
