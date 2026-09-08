//! Safe candidate layouts and bounded contracts, not replacement compiler code.
#![forbid(unsafe_code)]
#![allow(dead_code)]

use std::mem::{align_of, size_of};
use std::num::NonZeroU32;
use std::sync::{atomic::AtomicU64, Mutex};

#[path = "../storage-pilot/text.rs"]
mod pilot_text;

/// Owner-local link word. Zero is nil; MAX remains an ordinary local slot.
/// Foreign/lazy identities require the separately budgeted escape mechanism.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct LocalLink(u32);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct DeclarationSlice {
    backing: LocalLink,
    start: u32,
    len: u32,
    capacity: u32,
}

impl DeclarationSlice {
    fn slice(self, start: u32, end: u32, capacity: u32) -> Option<Self> {
        if start > end || end > capacity || capacity > self.capacity {
            return None;
        }
        Some(Self {
            backing: self.backing,
            start: self.start.checked_add(start)?,
            len: end - start,
            capacity: capacity - start,
        })
    }

    fn same(self, other: Self) -> bool {
        self.len == other.len
            && (self.len == 0 || (self.backing == other.backing && self.start == other.start))
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Text8 {
    first: u32,
    second: u32,
}

#[repr(C)]
struct Symbol<Text> {
    flags: u32,
    check_flags: u32,
    name: Text,
    declarations: DeclarationSlice,
    value_declaration: LocalLink,
    members: LocalLink,
    exports: LocalLink,
    parent: LocalLink,
    export_symbol: LocalLink,
    runtime_id: AtomicU64,
}

#[repr(C)]
struct SymbolWithoutRuntimeId<Text> {
    flags: u32,
    check_flags: u32,
    name: Text,
    declarations: DeclarationSlice,
    value_declaration: LocalLink,
    members: LocalLink,
    exports: LocalLink,
    parent: LocalLink,
    export_symbol: LocalLink,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SwitchClause {
    switch_statement: LocalLink,
    clause_start: i32,
    clause_end: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ReduceLabel {
    target: LocalLink,
    antecedents: LocalLink,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FlowData {
    None,
    Ast(LocalLink),
    Switch(SwitchClause),
    Reduce(ReduceLabel),
}

#[repr(C)]
struct FlowInline {
    flags: u32,
    data: FlowData,
    antecedent: LocalLink,
    antecedents: LocalLink,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FlowTag {
    None,
    Ast,
    Switch,
    Reduce,
}

#[repr(C)]
struct FlowOutlined {
    flags: u32,
    tag: FlowTag,
    data_slot: u32,
    antecedent: LocalLink,
    antecedents: LocalLink,
}

#[repr(C)]
struct FlowPacked {
    flags: u32,
    data_word: u32,
    antecedent: LocalLink,
    antecedents: LocalLink,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FullLink {
    arena: NonZeroU32,
    slot: NonZeroU32,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FullFlowData {
    None,
    Ast(FullLink),
    Switch {
        node: Option<FullLink>,
        start: i32,
        end: i32,
    },
    Reduce {
        target: Option<FullLink>,
        antecedents: Option<FullLink>,
    },
}

#[repr(C)]
struct FlowEscape {
    flow_slot: NonZeroU32,
    data: FullFlowData,
}

struct FlowEscapeTable {
    slots: Box<[Option<FlowEscape>]>,
    len: usize,
}

const FLOW_ESCAPE_WORD: u32 = 1;
const FLOW_ORDINAL_LIMIT: u32 = 1 << 30;

/// The tag belongs to the data word, independently of every open flags bit.
fn pack_flow_word(tag: FlowTag, ordinal: u32) -> Option<u32> {
    let bits = match tag {
        FlowTag::None => return (ordinal == 0).then_some(0),
        FlowTag::Ast => 1,
        FlowTag::Switch => 2,
        FlowTag::Reduce => 3,
    };
    (ordinal > 0 && ordinal < FLOW_ORDINAL_LIMIT).then_some((bits << 30) | ordinal)
}

fn unpack_flow_word(word: u32) -> Option<(FlowTag, u32)> {
    if word == 0 {
        return Some((FlowTag::None, 0));
    }
    let ordinal = word & (FLOW_ORDINAL_LIMIT - 1);
    let tag = match word >> 30 {
        1 => FlowTag::Ast,
        2 => FlowTag::Switch,
        3 => FlowTag::Reduce,
        _ => return None,
    };
    (ordinal != 0).then_some((tag, ordinal))
}

fn local_link(value: FullLink, arena: NonZeroU32) -> Option<LocalLink> {
    (value.arena == arena).then_some(LocalLink(value.slot.get()))
}

/// Checked direct AST word or an exact full-identity escape keyed by flow slot.
fn pack_ast_data(
    flow_slot: NonZeroU32,
    node: FullLink,
    arena: NonZeroU32,
) -> (u32, Option<FlowEscape>) {
    if let Some(word) = local_link(node, arena).and_then(|n| pack_flow_word(FlowTag::Ast, n.0)) {
        (word, None)
    } else {
        (
            FLOW_ESCAPE_WORD,
            Some(FlowEscape {
                flow_slot,
                data: FullFlowData::Ast(node),
            }),
        )
    }
}

fn pack_synthetic_data(
    flow_slot: NonZeroU32,
    data: FullFlowData,
    ordinal: u32,
    ast_arena: NonZeroU32,
    flow_arena: NonZeroU32,
    list_arena: NonZeroU32,
) -> (u32, Option<FlowEscape>) {
    let (tag, local) = match data {
        FullFlowData::Switch { node, .. } => (
            FlowTag::Switch,
            node.map_or(true, |n| local_link(n, ast_arena).is_some()),
        ),
        FullFlowData::Reduce {
            target,
            antecedents,
        } => (
            FlowTag::Reduce,
            target.map_or(true, |n| local_link(n, flow_arena).is_some())
                && antecedents.map_or(true, |n| local_link(n, list_arena).is_some()),
        ),
        _ => return (FLOW_ESCAPE_WORD, Some(FlowEscape { flow_slot, data })),
    };
    if let Some(word) = local.then(|| pack_flow_word(tag, ordinal)).flatten() {
        (word, None)
    } else {
        (FLOW_ESCAPE_WORD, Some(FlowEscape { flow_slot, data }))
    }
}

#[repr(C)]
struct FlowList {
    flow: LocalLink,
    next: LocalLink,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct NodeSlice {
    backing: LocalLink,
    start: u32,
    len: u32,
}

#[repr(C)]
struct NodeList {
    pos: i32,
    end: i32,
    nodes: NodeSlice,
    modifier_flags: u32,
}

#[repr(C)]
struct IndirectNodeList {
    pos: i32,
    end: i32,
    slice_record: u32,
    modifier_flags: u32,
}

#[repr(C)]
struct AuxiliaryLocator {
    row: u32,
    kind: u32,
}

/// Hypothetical flattened backing; its cell store and growth are additional.
#[repr(C)]
struct FlatBacking {
    start: u32,
    len: u32,
}

/// Canonical owner-local byte-name identity; empty text still gets a nonzero ID.
/// The interner, byte equality and overflow/foreign fallback are not free.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NameKey(NonZeroU32);

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TableEntry {
    key: NameKey,
    symbol: LocalLink,
}

/// A concrete scalar-probing alternative to assuming a hash-overhead factor.
/// Empty slots use Option's nonzero-key niche; there is no omitted control array.
struct ScalarTable {
    slots: Box<[Option<TableEntry>]>,
    len: usize,
}

#[repr(C)]
struct RuntimeEntry {
    symbol_slot: NonZeroU32,
    id: AtomicU64,
}

struct RuntimeTable {
    slots: Box<[Option<RuntimeEntry>]>,
    len: usize,
}

impl ScalarTable {
    fn new(bucket_count: usize) -> Option<Self> {
        if bucket_count == 0 {
            return Some(Self {
                slots: Box::default(),
                len: 0,
            });
        }
        if bucket_count < 2 || !bucket_count.is_power_of_two() {
            return None;
        }
        Some(Self {
            slots: vec![None; bucket_count].into_boxed_slice(),
            len: 0,
        })
    }

    fn insert(&mut self, entry: TableEntry) -> Option<Option<LocalLink>> {
        if self.slots.is_empty() {
            return None;
        }
        let mask = self.slots.len() - 1;
        let start = usize::try_from(entry.key.0.get()).ok()? & mask;
        for distance in 0..self.slots.len() {
            let index = (start + distance) & mask;
            match self.slots[index] {
                Some(previous) if previous.key == entry.key => {
                    self.slots[index] = Some(entry);
                    return Some(Some(previous.symbol));
                }
                None => {
                    if self.len + 1 > self.slots.len() / 8 * 7 + self.slots.len() % 8 * 7 / 8 {
                        return None; // The caller must allocate and rehash a larger table.
                    }
                    self.slots[index] = Some(entry);
                    self.len += 1;
                    return Some(None);
                }
                _ => {}
            }
        }
        None
    }

    fn get(&self, key: NameKey) -> Option<LocalLink> {
        if self.slots.is_empty() {
            return None;
        }
        let mask = self.slots.len() - 1;
        let start = usize::try_from(key.0.get()).ok()? & mask;
        for distance in 0..self.slots.len() {
            match self.slots[(start + distance) & mask] {
                Some(entry) if entry.key == key => return Some(entry.symbol),
                None => return None,
                _ => {}
            }
        }
        None
    }
}

struct Page<T> {
    values: Vec<T>,
    tracking: [usize; 2],
}

struct OwnerLedger {
    counters_handle: usize,
    tracked_pages: usize,
}

struct ThinStore<T, const N: usize> {
    pages: Vec<Box<[T; N]>>,
    used: u32,
    arena_id: NonZeroU32,
}

fn main() {
    macro_rules! layout {
        ($name:literal, $ty:ty) => {
            println!("{} {} {}", $name, size_of::<$ty>(), align_of::<$ty>());
        };
    }
    layout!("DeclarationSlice", DeclarationSlice);
    layout!("SymbolText4", Symbol<u32>);
    layout!("SymbolText8", Symbol<Text8>);
    layout!("SymbolText4WithoutRuntimeId", SymbolWithoutRuntimeId<u32>);
    layout!("SymbolText8WithoutRuntimeId", SymbolWithoutRuntimeId<Text8>);
    layout!("RuntimeId", AtomicU64);
    layout!("RuntimeBucket", Option<RuntimeEntry>);
    layout!("RuntimeTable", Mutex<RuntimeTable>);
    layout!("FlowInline", FlowInline);
    layout!("FlowOutlined", FlowOutlined);
    layout!("FlowPacked", FlowPacked);
    layout!("FullFlowData", FullFlowData);
    layout!("FlowEscapeBucket", Option<FlowEscape>);
    layout!("FlowEscapeTable", FlowEscapeTable);
    layout!("SwitchClause", SwitchClause);
    layout!("ReduceLabel", ReduceLabel);
    layout!("FlowList", FlowList);
    layout!("NodeSlice", NodeSlice);
    layout!("NodeList", NodeList);
    layout!("IndirectNodeList", IndirectNodeList);
    layout!("AuxiliaryLocator", AuxiliaryLocator);
    layout!("FlatBacking", FlatBacking);
    layout!("BoxBacking", Box<[LocalLink]>);
    layout!("Cell", LocalLink);
    layout!("TableEntry", TableEntry);
    layout!("TableBucket", Option<TableEntry>);
    layout!("ScalarTable", ScalarTable);
    layout!("CurrentTableHeader", std::collections::HashMap<[usize; 4], u64>);
    layout!("CurrentPage", Page<u32>);
    layout!("ThinPage", Box<[u32; 32]>);
    layout!("FatPage", Box<[u32]>);
    layout!("ThinStore", ThinStore<u32, 32>);
    layout!("OwnerLedger", OwnerLedger);
    layout!("VecHeader", Vec<u32>);
    layout!("TextPrototypeOwner", pilot_text::TextStore);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declaration_headers_preserve_nil_identity_ranges_capacity_and_aliasing() {
        let nil = DeclarationSlice::default();
        let empty = DeclarationSlice {
            backing: LocalLink(1),
            ..nil
        };
        assert_ne!(nil.backing, empty.backing);
        assert!(nil.same(empty));
        let original = DeclarationSlice {
            backing: LocalLink(u32::MAX),
            start: 3,
            len: 1,
            capacity: 3,
        };
        let copied = original;
        let exposed_tail = copied.slice(1, 3, 3).unwrap();
        assert_eq!(
            (
                exposed_tail.backing.0,
                exposed_tail.start,
                exposed_tail.len,
                exposed_tail.capacity
            ),
            (u32::MAX, 4, 2, 2)
        );
        assert!(!original.same(exposed_tail));
        assert!(original.slice(0, 4, 4).is_none());
        assert!(original.slice(2, 1, 3).is_none());
        let overflow = DeclarationSlice {
            start: u32::MAX,
            ..original
        };
        assert!(overflow.slice(1, 2, 3).is_none());

        let mut physical_backings = [vec![7_u32, 0, 0].into_boxed_slice()];
        let head = DeclarationSlice {
            backing: LocalLink(1),
            start: 0,
            len: 1,
            capacity: 3,
        };
        let alias = head;
        physical_backings[0][1] = 11;
        let tail = alias.slice(1, 2, 3).unwrap();
        assert_eq!(
            physical_backings[0][usize::try_from(tail.start).unwrap()],
            11
        );
        assert_eq!(alias.len, 1); // Mutating backing never mutates a copied header.
    }

    #[test]
    fn flow_discriminants_do_not_borrow_bits_from_flags_or_links() {
        let data = FlowData::Switch(SwitchClause {
            switch_statement: LocalLink(u32::MAX),
            clause_start: i32::MIN,
            clause_end: i32::MAX,
        });
        let flow = FlowInline {
            flags: u32::MAX,
            data,
            antecedent: LocalLink(u32::MAX),
            antecedents: LocalLink(u32::MAX),
        };
        assert_eq!(flow.flags, u32::MAX);
        assert_eq!(flow.data, data);
        assert_ne!(FlowData::None, FlowData::Ast(LocalLink(0)));
        let outlined = FlowOutlined {
            flags: u32::MAX,
            tag: FlowTag::Reduce,
            data_slot: u32::MAX,
            antecedent: LocalLink(0),
            antecedents: LocalLink(1),
        };
        assert_eq!(outlined.data_slot, u32::MAX);
        assert_eq!(outlined.tag, FlowTag::Reduce);
    }

    #[test]
    fn packed_flow_words_roundtrip_tags_and_escape_full_or_foreign_identities() {
        let nonzero = |n| NonZeroU32::new(n).unwrap();
        for tag in [FlowTag::Ast, FlowTag::Switch, FlowTag::Reduce] {
            for ordinal in [1, 17, FLOW_ORDINAL_LIMIT - 1] {
                let word = pack_flow_word(tag, ordinal).unwrap();
                assert_eq!(unpack_flow_word(word), Some((tag, ordinal)));
            }
            assert!(pack_flow_word(tag, 0).is_none());
            assert!(pack_flow_word(tag, FLOW_ORDINAL_LIMIT).is_none());
            assert!(pack_flow_word(tag, u32::MAX).is_none());
        }
        assert_eq!(unpack_flow_word(0), Some((FlowTag::None, 0)));
        assert!(unpack_flow_word(FLOW_ESCAPE_WORD).is_none());
        assert!(unpack_flow_word(1 << 30).is_none());
        let flow_slot = nonzero(u32::MAX);
        for node in [
            FullLink {
                arena: nonzero(3),
                slot: nonzero(u32::MAX),
            },
            FullLink {
                arena: nonzero(u32::MAX),
                slot: nonzero(1),
            },
        ] {
            let (word, escape) = pack_ast_data(flow_slot, node, nonzero(3));
            assert_eq!(word, FLOW_ESCAPE_WORD);
            let escape = escape.unwrap();
            assert_eq!(escape.flow_slot, flow_slot);
            assert_eq!(escape.data, FullFlowData::Ast(node));
        }
        let node = FullLink {
            arena: nonzero(3),
            slot: nonzero(7),
        };
        let (word, escape) = pack_ast_data(flow_slot, node, nonzero(3));
        assert!(escape.is_none());
        assert_eq!(unpack_flow_word(word), Some((FlowTag::Ast, 7)));
        // Escaped synthetic payloads retain full links and signed ranges too;
        // their row and bucket are charged separately from the 16-byte flow.
        let foreign = FullLink {
            arena: nonzero(u32::MAX),
            slot: nonzero(u32::MAX),
        };
        for data in [
            FullFlowData::Switch {
                node: Some(foreign),
                start: i32::MIN,
                end: i32::MAX,
            },
            FullFlowData::Reduce {
                target: Some(foreign),
                antecedents: None,
            },
        ] {
            let (word, escape) =
                pack_synthetic_data(flow_slot, data, 1, nonzero(1), nonzero(2), nonzero(3));
            assert_eq!(word, FLOW_ESCAPE_WORD);
            let escape = escape.unwrap();
            assert_eq!(escape.flow_slot, flow_slot);
            assert_eq!(escape.data, data);
        }
        let local_switch = FullFlowData::Switch {
            node: Some(FullLink {
                arena: nonzero(1),
                slot: nonzero(u32::MAX),
            }),
            start: i32::MIN,
            end: i32::MAX,
        };
        let (word, escape) = pack_synthetic_data(
            flow_slot,
            local_switch,
            FLOW_ORDINAL_LIMIT - 1,
            nonzero(1),
            nonzero(2),
            nonzero(3),
        );
        assert!(escape.is_none());
        assert_eq!(
            unpack_flow_word(word),
            Some((FlowTag::Switch, FLOW_ORDINAL_LIMIT - 1))
        );
        let (word, escape) = pack_synthetic_data(
            flow_slot,
            local_switch,
            FLOW_ORDINAL_LIMIT,
            nonzero(1),
            nonzero(2),
            nonzero(3),
        );
        assert_eq!(word, FLOW_ESCAPE_WORD);
        assert_eq!(escape.unwrap().data, local_switch);
        assert_eq!(size_of::<Option<FlowEscape>>(), size_of::<FlowEscape>());
    }

    #[test]
    fn list_headers_keep_location_modifiers_and_distinct_slice_states() {
        let nil = NodeSlice::default();
        let missing = NodeSlice { start: 1, ..nil };
        let empty = NodeSlice {
            backing: LocalLink(1),
            ..nil
        };
        assert_ne!(nil, missing);
        assert_ne!(nil, empty);
        let list = NodeList {
            pos: i32::MIN,
            end: i32::MAX,
            nodes: missing,
            modifier_flags: u32::MAX,
        };
        assert_eq!(list.nodes, missing);
        assert_eq!(list.modifier_flags, u32::MAX);
    }

    #[test]
    fn scalar_table_keeps_nil_values_and_collision_chains_distinct_from_absence() {
        let key = |value| NameKey(NonZeroU32::new(value).unwrap());
        let mut table = ScalarTable::new(4).unwrap();
        assert_eq!(
            table.insert(TableEntry {
                key: key(1),
                symbol: LocalLink(0)
            }),
            Some(None)
        );
        assert_eq!(
            table.insert(TableEntry {
                key: key(5),
                symbol: LocalLink(u32::MAX)
            }),
            Some(None)
        );
        assert_eq!(table.get(key(1)), Some(LocalLink(0)));
        assert_eq!(table.get(key(5)), Some(LocalLink(u32::MAX)));
        assert_eq!(table.get(key(9)), None);
        assert_eq!(
            table.insert(TableEntry {
                key: key(1),
                symbol: LocalLink(7)
            }),
            Some(Some(LocalLink(0)))
        );
        assert_eq!(table.len, 2);
        assert_eq!(
            table.insert(TableEntry {
                key: key(9),
                symbol: LocalLink(3)
            }),
            Some(None)
        );
        assert!(table
            .insert(TableEntry {
                key: key(13),
                symbol: LocalLink(4)
            })
            .is_none());
        assert_eq!(table.len, 3);
        assert_eq!(size_of::<Option<TableEntry>>(), size_of::<TableEntry>());
        assert!(ScalarTable::new(3).is_none());
        let mut empty = ScalarTable::new(0).unwrap();
        assert_eq!(empty.get(key(1)), None);
        assert!(empty
            .insert(TableEntry {
                key: key(1),
                symbol: LocalLink(1)
            })
            .is_none());
    }
}
