use super::*;

fn store(source: &[u8]) -> TextStore {
    TextStore::new(Arc::from(source))
}

#[test]
fn ordinary_reads_borrow_source_without_pooling_or_per_identifier_retains() {
    assert_eq!(size_of::<TextWord>(), 4);
    let source: Arc<[u8]> = Arc::from(&b"let alpha = alpha;"[..]);
    let mut owner = TextStore::new(Arc::clone(&source));
    let first = owner.insert(0, b"alpha", Some(9)).unwrap();
    let second = owner.insert(u32::MAX, b"alpha", Some(17)).unwrap();
    assert!(first.is_raw() && second.is_raw());
    for _ in 0..10 {
        assert_eq!(owner.bytes(0, first, 9).unwrap(), b"alpha");
        assert_eq!(
            owner.bytes(u32::MAX, second, 17).unwrap().as_ptr(),
            source[12..].as_ptr()
        );
    }
    assert_eq!(Arc::strong_count(&source), 2);
    let stats = owner.stats();
    assert_eq!(stats.pool_entries, 0);
    assert_eq!(stats.pool_capacity_bytes, 0);
    assert_eq!(stats.pool_entry_capacity_bytes, 0);
    assert_eq!(stats.extended_capacity_entries, 0);
}

#[test]
fn decoded_escapes_and_jsx_names_preserve_bytes() {
    let mut owner = store(br"\u0061 data-value");
    let escaped = owner.insert(1, b"a", Some(6)).unwrap();
    let jsx = owner.insert(2, b"data-value", Some(17)).unwrap();
    assert!(!escaped.is_raw());
    assert!(jsx.is_raw());
    assert_eq!(owner.bytes(1, escaped, 6).unwrap(), b"a");
    assert_eq!(owner.bytes(2, jsx, 17).unwrap(), b"data-value");
    assert_eq!(owner.stats().pool_bytes, 1);
}

#[test]
fn wtf8_malformed_bytes_and_partial_codepoints_are_not_decoded() {
    let source = b"\xed\xa0\x80 \xf0\x9f\x98\x80 \xff";
    let mut owner = store(source);
    for (slot, bytes, end) in [
        (0, &source[0..3], 3),
        (1, &source[5..7], 7),
        (2, &source[9..10], 10),
    ] {
        let word = owner.insert(slot, bytes, Some(end)).unwrap();
        assert!(word.is_raw());
        assert_eq!(owner.bytes(slot, word, end).unwrap(), bytes);
    }
    let pooled = owner.insert(3, b"\xed\xbf\xbf\xfe", Some(1)).unwrap();
    assert_eq!(owner.bytes(3, pooled, 1).unwrap(), b"\xed\xbf\xbf\xfe");
}

#[test]
fn factory_observation_precedes_range_and_finalization_does_not_erase_costs() {
    let mut owner = store(b"foo bar");
    let word = owner.insert(7, b"foo", None).unwrap();
    assert!(!word.is_raw());
    assert_eq!(owner.bytes(7, word, -1).unwrap(), b"foo");
    let word = owner.change_end(7, word, -1, 3).unwrap();
    assert!(word.is_raw());
    assert_eq!(owner.bytes(7, word, 3).unwrap(), b"foo");
    assert_eq!(owner.stats().pool_entries, 1);
    assert_eq!(owner.stats().pool_bytes, 3);
    let arbitrary = owner.insert(8, b"elsewhere", None).unwrap();
    let arbitrary = owner.change_end(8, arbitrary, -1, 3).unwrap();
    assert_eq!(owner.bytes(8, arbitrary, 3).unwrap(), b"elsewhere");
}

#[test]
fn range_edits_preserve_text_even_when_the_new_suffix_differs() {
    let mut owner = store(b"foo bar foo");
    let raw = owner.insert(4, b"foo", Some(3)).unwrap();
    let same = owner.change_end(4, raw, 3, 11).unwrap();
    assert!(same.is_raw());
    let moved = owner.change_end(4, same, 11, 7).unwrap();
    assert!(!moved.is_raw());
    assert_eq!(owner.bytes(4, moved, 7).unwrap(), b"foo");
    for end in [-1, -2, i32::MIN, i32::MAX, 0] {
        let moved = owner.change_end(4, moved, 7, end).unwrap();
        assert_eq!(owner.bytes(4, moved, end).unwrap(), b"foo");
    }
    assert_eq!(owner.stats().pool_entries, 1);
    assert_eq!(
        owner.change_end(4, raw, -1, 3),
        Err(TextError::InvalidRawRange)
    );
    assert_eq!(owner.bytes(4, raw, 3).unwrap(), b"foo");
}

#[test]
fn foreign_and_synthetic_clones_survive_the_original_owner_and_temporary() {
    let mut destination = store(b"bar");
    let (foreign, synthetic) = {
        let mut original = store(b"foo");
        let word = original.insert(1, b"foo", Some(3)).unwrap();
        let foreign = destination
            .insert(2, original.bytes(1, word, 3).unwrap(), Some(3))
            .unwrap();
        let temporary = vec![0xed, 0xa0, 0x80, 0xff];
        let synthetic = destination.insert(3, &temporary, Some(-1)).unwrap();
        (foreign, synthetic)
    };
    assert_eq!(destination.bytes(2, foreign, 3).unwrap(), b"foo");
    assert_eq!(
        destination.bytes(3, synthetic, -1).unwrap(),
        b"\xed\xa0\x80\xff"
    );
    let retained = destination.retain(3, synthetic, -1).unwrap();
    drop(destination);
    assert_eq!(&*retained, b"\xed\xa0\x80\xff");
}

#[test]
fn raw_boundary_arithmetic_and_full_tag_domain_are_checked() {
    assert_eq!(
        TextWord::raw(MAX_TAGGED_VALUE),
        Some(TextWord(u32::MAX - 1))
    );
    assert_eq!(TextWord::raw(MAX_TAGGED_VALUE + 1), None);
    assert_eq!(TextWord::raw(usize::MAX), None);
    assert_eq!(
        TextWord::pool(MAX_TAGGED_VALUE - 1, MAX_TAGGED_VALUE),
        Some(TextWord(u32::MAX - 2))
    );
    assert_eq!(TextWord::pool(MAX_TAGGED_VALUE, MAX_TAGGED_VALUE), None);
    assert_eq!(TextWord::pool(usize::MAX, MAX_TAGGED_VALUE), None);
    let mut owner = store(b"abc");
    for end in [i32::MIN, -1, 0, 2, 4, i32::MAX] {
        let word = owner.insert(0, b"abc", Some(end)).unwrap();
        assert!(!word.is_raw());
        assert_eq!(owner.bytes(0, word, end).unwrap(), b"abc");
    }
    let empty = owner.insert(1, b"", Some(0)).unwrap();
    assert!(empty.is_raw());
    assert_eq!(owner.bytes(1, empty, 0).unwrap(), b"");
    let huge = TextWord::raw(MAX_TAGGED_VALUE).unwrap();
    assert_eq!(
        owner.bytes(1, huge, i32::MAX),
        Err(TextError::InvalidRawRange)
    );
}

#[test]
fn extended_pool_words_resolve_full_slot_domain_without_truncating_indices() {
    let mut owner = store(b"");
    let first = owner.pool.insert_with_limit(9, b"first", 1).unwrap();
    let zero = owner.pool.insert_with_limit(0, b"zero slot", 1).unwrap();
    let maximum = owner
        .pool
        .insert_with_limit(u32::MAX, b"max slot", 1)
        .unwrap();
    assert_eq!(first, TextWord(1));
    assert_eq!(zero, TextWord(EXTENDED_POOL_WORD));
    assert_eq!(maximum, TextWord(EXTENDED_POOL_WORD));
    assert_eq!(owner.bytes(0, zero, -1).unwrap(), b"zero slot");
    assert_eq!(owner.bytes(u32::MAX, maximum, -1).unwrap(), b"max slot");
    assert_eq!(
        owner.bytes(1, maximum, -1),
        Err(TextError::UnknownExtendedSlot)
    );
    assert_eq!(
        owner.bytes(9, TextWord(99), -1),
        Err(TextError::UnknownPoolEntry)
    );
    assert_eq!(owner.stats().extended_entries, 2);
    assert_eq!(owner.stats().pool_entries, 3);
}

#[test]
fn pool_growth_keeps_previously_issued_words_and_byte_ranges_valid() {
    let mut owner = store(b"");
    let original = owner.insert(0, b"\xfforiginal", None).unwrap();
    for slot in 1..300 {
        let bytes = slot.to_string();
        owner.insert(slot, bytes.as_bytes(), None).unwrap();
    }
    assert_eq!(owner.bytes(0, original, -1).unwrap(), b"\xfforiginal");
    let stats = owner.stats();
    assert_eq!(stats.pool_entries, 300);
    assert!(stats.pool_capacity_bytes >= stats.pool_bytes);
    assert!(stats.pool_entry_capacity_bytes >= stats.pool_entries * size_of::<PoolEntry>());
}
