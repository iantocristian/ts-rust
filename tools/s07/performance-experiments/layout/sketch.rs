//! CP0 size and identity experiment; this is not a production storage proposal.
#![forbid(unsafe_code)]
#![allow(dead_code)]

use std::collections::BTreeMap;

#[repr(C)]
struct Header24 {
    kind: i16,
    shape: u16,
    flags: u32,
    pos: i32,
    end: i32,
    parent: u32,
    payload_ordinal: u32,
}

#[repr(C)]
struct Header32 {
    kind: i16,
    shape: u16,
    flags: u32,
    pos: i32,
    end: i32,
    parent: u64,
    payload_ordinal: u32,
}

#[repr(C)]
struct Text8 {
    offset: u32,
    len: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FullId {
    arena: u32,
    slot: u32,
}

/// Slot+1 encodes ordinary local links; no public slot bit is stolen. A separate
/// bitmap marks exceptions, including the local u32::MAX slot and all foreign
/// or lazy links. This intentionally pays for both bitmap and exception map.
/// It establishes round-trip identity only, not ownership or a fast AST API.
#[derive(Default)]
struct Links {
    words: Vec<u32>,
    escaped: Vec<u64>,
    exceptions: BTreeMap<usize, FullId>,
}

impl Links {
    fn push(&mut self, owner: u32, value: Option<FullId>, is_lazy: bool) {
        let field = self.words.len();
        let direct = value.filter(|id| id.arena == owner && !is_lazy)
            .and_then(|id| id.slot.checked_add(1));
        self.words.push(direct.unwrap_or(0));
        if let Some(id) = value.filter(|_| direct.is_none()) {
            self.escaped.resize(field / 64 + 1, 0);
            self.escaped[field / 64] |= 1 << (field % 64);
            self.exceptions.insert(field, id);
        }
    }

    fn get(&self, owner: u32, field: usize) -> Option<FullId> {
        // Bounds are checked even when the exception bitmap is empty.
        let word = self.words[field];
        if self.escaped.get(field / 64).is_some_and(|bits| bits & (1 << (field % 64)) != 0) {
            return Some(*self.exceptions.get(&field).expect("marked escape must exist"));
        }
        word.checked_sub(1).map(|slot| FullId { arena: owner, slot })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_slot_and_owner_domain_survives() {
        for owner in [1, 17, u32::MAX] {
            let mut links = Links::default();
            let mut expected = Vec::new();
            for arena in [1, 17, u32::MAX] {
                for slot in [0, 1, (1 << 31) - 1, 1 << 31, u32::MAX - 1, u32::MAX] {
                    for lazy in [false, true] {
                        let id = Some(FullId { arena, slot });
                        links.push(owner, id, lazy);
                        expected.push(id);
                    }
                }
            }
            links.push(owner, None, false);
            expected.push(None);
            for (field, id) in expected.into_iter().enumerate() {
                assert_eq!(links.get(owner, field), id);
            }
        }
    }

    #[test]
    fn open_kind_and_payload_shape_are_independent() {
        let header = Header24 { kind: i16::MIN, shape: 7, flags: u32::MAX,
            pos: -1, end: i32::MAX, parent: 0, payload_ordinal: u32::MAX };
        assert_eq!(header.kind, i16::MIN);
        assert_eq!(header.shape, 7);
        assert_eq!(std::mem::size_of::<Header24>(), 24);
        assert_eq!(std::mem::size_of::<Header32>(), 32);
    }

    #[test]
    #[should_panic]
    fn raw_field_still_requires_valid_bounds() {
        Links::default().get(1, 0);
    }
}
