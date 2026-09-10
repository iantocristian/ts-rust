//! CP0 storage envelopes, not production arenas or evidence of access speed.
#![forbid(unsafe_code)]
#![allow(dead_code)]
use std::mem::{align_of, size_of};
use std::sync::atomic::AtomicU32;

const SHAPES: usize = 192;

// The existing census measures40 bytes per Page directory element: Vec's24
// bytes and16 bytes of counter bookkeeping. Keep that charge in this model.
struct Page<T> {
    values: Vec<T>,
    counter_bookkeeping: [usize; 2],
}

enum PackedPages {
    Words(Vec<Page<u32>>),
    Wide(Vec<Page<[u64; 8]>>),
}
enum OneOrMany<T> {
    One(Page<T>),
    Many(Vec<Page<T>>),
}
struct InlineFirst<T> {
    first: Page<T>,
    rest: Vec<Page<T>>,
}

struct FixedDirectory {
    entries: [Vec<Page<u32>>; SHAPES],
}
struct OptionalDirectory {
    entries: [Option<Box<Vec<Page<u32>>>>; SHAPES],
}
struct PackedDirectory {
    shape_to_entry: [u16; SHAPES],
    entries: Vec<PackedPages>,
}
struct PackedWordDirectory {
    class_to_entry: [u16; 15],
    entries: Vec<PackedPages>,
}

// Alternative owned POD pages. The ledger envelope charges one Counters Arc
// and an explicit tracked-page count per active store; it does not implement
// counter/drop semantics. `used` distinguishes logical records from physically
// default-initialized spare slots in a Box page.
struct Ledger {
    counters_arc: usize,
    tracked_pages: usize,
}
struct Store<P> {
    pages: Vec<P>,
    used: usize,
    ledger: Ledger,
}
enum CompactOneOrMany<P> {
    One(P),
    Many(Vec<P>),
}
struct OneOrManyStore<P> {
    pages: CompactOneOrMany<P>,
    used: usize,
    ledger: Ledger,
}
struct InlineStore<P> {
    first: P,
    rest: Vec<P>,
    used: usize,
    ledger: Ledger,
}
#[repr(u8)]
enum PackedFatStore {
    Words(Store<Box<[u32]>>),
    Wide(Store<Box<[[u64; 8]]>>),
}
#[repr(u8)]
enum PackedThinStore {
    Words(Store<Box<[u32; 32]>>),
    Wide(Store<Box<[[u64; 8]; 32]>>),
}
struct FlatStore<T> {
    values: Vec<T>,
    ledger: Ledger,
}
#[repr(u8)]
enum PackedFlatStore {
    Words(FlatStore<u32>),
    Wide(FlatStore<[u64; 8]>),
}
struct OwnerLedgerStore<P> {
    pages: Vec<P>,
    used: usize,
}
struct OwnerLedgerOneOrMany<P> {
    pages: CompactOneOrMany<P>,
    used: usize,
}
struct OwnerLedgerInline<P> {
    first: P,
    rest: Vec<P>,
    used: usize,
}
#[repr(u8)]
enum PackedOwnerLedgerFatStore {
    Words(OwnerLedgerStore<Box<[u32]>>),
    Wide(OwnerLedgerStore<Box<[[u64; 8]]>>),
}
#[repr(u8)]
enum PackedOwnerLedgerThinStore {
    Words(OwnerLedgerStore<Box<[u32; 32]>>),
    Wide(OwnerLedgerStore<Box<[[u64; 8]; 32]>>),
}
#[repr(u8)]
enum PackedOwnerLedgerFlatStore {
    Words(Vec<u32>),
    Wide(Vec<[u64; 8]>),
}

fn main() {
    macro_rules! layout {
        ($name:literal, $ty:ty) => {
            ($name, size_of::<$ty>(), align_of::<$ty>())
        };
    }
    for (name, size, align) in [
        ("Page", size_of::<Page<u32>>(), align_of::<Page<u32>>()),
        ("VecHeader", size_of::<Vec<u32>>(), align_of::<Vec<u32>>()),
        (
            "OptionalPointer",
            size_of::<Option<Box<Vec<u32>>>>(),
            align_of::<Option<Box<Vec<u32>>>>(),
        ),
        (
            "PackedPages",
            size_of::<PackedPages>(),
            align_of::<PackedPages>(),
        ),
        (
            "OneOrMany",
            size_of::<OneOrMany<u32>>(),
            align_of::<OneOrMany<u32>>(),
        ),
        (
            "InlineFirst",
            size_of::<InlineFirst<u32>>(),
            align_of::<InlineFirst<u32>>(),
        ),
        (
            "FixedDirectory",
            size_of::<FixedDirectory>(),
            align_of::<FixedDirectory>(),
        ),
        (
            "OptionalDirectory",
            size_of::<OptionalDirectory>(),
            align_of::<OptionalDirectory>(),
        ),
        (
            "PackedDirectory",
            size_of::<PackedDirectory>(),
            align_of::<PackedDirectory>(),
        ),
        layout!("Ledger", Ledger),
        layout!("LogicalLength", usize),
        layout!("FatPage", Box<[u32]>),
        layout!("ThinPage", Box<[u32; 32]>),
        layout!("FatStore", Store<Box<[u32]>>),
        layout!("ThinStore", Store<Box<[u32; 32]>>),
        layout!("FatOneOrManyStore", OneOrManyStore<Box<[u32]>>),
        layout!("ThinOneOrManyStore", OneOrManyStore<Box<[u32; 32]>>),
        layout!("FatInlineStore", InlineStore<Box<[u32]>>),
        layout!("ThinInlineStore", InlineStore<Box<[u32; 32]>>),
        layout!("PackedFatStore", PackedFatStore),
        layout!("PackedThinStore", PackedThinStore),
        layout!("FlatStore", FlatStore<u32>),
        layout!("PackedFlatStore", PackedFlatStore),
        layout!("FatOwnerStore", OwnerLedgerStore<Box<[u32]>>),
        layout!("ThinOwnerStore", OwnerLedgerStore<Box<[u32; 32]>>),
        layout!("FatOwnerOneOrManyStore", OwnerLedgerOneOrMany<Box<[u32]>>),
        layout!("ThinOwnerOneOrManyStore", OwnerLedgerOneOrMany<Box<[u32; 32]>>),
        layout!("FatOwnerInlineStore", OwnerLedgerInline<Box<[u32]>>),
        layout!("ThinOwnerInlineStore", OwnerLedgerInline<Box<[u32; 32]>>),
        layout!("PackedFatOwnerStore", PackedOwnerLedgerFatStore),
        layout!("PackedThinOwnerStore", PackedOwnerLedgerThinStore),
        layout!("PackedFlatOwnerStore", PackedOwnerLedgerFlatStore),
        layout!("AtomicFacts", AtomicU32),
        layout!("Word1", [u32; 1]),
        layout!("Word2", [u32; 2]),
        layout!("Word3", [u32; 3]),
        layout!("Word4", [u32; 4]),
        layout!("Word5", [u32; 5]),
        layout!("Word6", [u32; 6]),
        layout!("Word7", [u32; 7]),
        layout!("Word8", [u32; 8]),
        layout!("Word9", [u32; 9]),
        layout!("Word10", [u32; 10]),
        layout!("Word11", [u32; 11]),
        layout!("Word12", [u32; 12]),
        layout!("Word13", [u32; 13]),
        layout!("Word14", [u32; 14]),
        layout!("Word15", [u32; 15]),
        layout!("Word16", [u32; 16]),
        layout!("AtomicWord1", [AtomicU32; 1]),
        layout!("AtomicWord16", [AtomicU32; 16]),
        layout!("PackedWordDirectory", PackedWordDirectory),
    ] {
        println!("{name} {size} {align}");
    }
}
