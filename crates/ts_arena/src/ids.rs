//! Node and symbol identity (docs/design/ownership.md, section 2.2).
//!
//! A `NodeId` is 64 bits: the high 32 are an `ArenaId`, the low 32 a slot. Arena
//! ids come from one process-global counter and are never reused, so "generation"
//! for node storage *is* the arena id: no separate generation field and no ABA
//! window. Arena 0 and slot 0 are reserved, which is what lets `Option<NodeId>`
//! represent absence in one word.

use std::fmt;
use std::num::{NonZeroU32, NonZeroU64};
use std::sync::atomic::{AtomicU64, Ordering};

/// Allocating past this many nonzero ids fails before the counter can wrap.
pub const MAX_ID: u32 = u32::MAX;

/// An arena's identity. Never reused within the process.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArenaId(NonZeroU32);

impl ArenaId {
    pub fn get(self) -> u32 {
        self.0.get()
    }

    /// An arena id from a raw value.
    ///
    /// Production storage always takes its id from [`allocate_arena_id`]. This
    /// exists for the harnesses that inject counters at the exhaustion
    /// boundaries, which is how E3 exercises them rather than relying on an
    /// assumed session lifetime. Arena 0 is reserved, so zero yields `None`.
    pub fn from_raw(value: u32) -> Option<Self> {
        NonZeroU32::new(value).map(ArenaId)
    }
}

impl fmt::Debug for ArenaId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ArenaId({})", self.0)
    }
}

/// A slot within one arena. Never reused; arenas are append-only.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Slot(NonZeroU32);

impl Slot {
    /// A slot from a raw value. Slot 0 is reserved, so zero yields `None`.
    /// Production allocation goes through [`SlotCounter`].
    pub fn from_raw(value: u32) -> Option<Self> {
        NonZeroU32::new(value).map(Slot)
    }

    pub fn get(self) -> u32 {
        self.0.get()
    }

    pub fn index(self) -> usize {
        self.0.get() as usize - 1
    }
}

/// Allocation was refused before the counter could wrap or reuse a value.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Exhausted {
    /// What ran out: `"arena"` or `"slot"`.
    pub what: &'static str,
}

impl fmt::Display for Exhausted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ids exhausted: refusing to allocate beyond {MAX_ID} before wrap or reuse",
            self.what
        )
    }
}

/// A checked, never-wrapping counter of nonzero 32-bit ids.
///
/// The counter is held as 64 bits so the saturated state (`MAX_ID + 1`) is
/// representable and the 32-bit space can never wrap back onto a live id. E3
/// injects a starting value to exercise the boundary instead of relying on an
/// assumed session lifetime.
#[derive(Debug)]
pub struct IdCounter {
    next: AtomicU64,
    what: &'static str,
}

impl IdCounter {
    pub const fn new(what: &'static str) -> Self {
        Self {
            next: AtomicU64::new(1),
            what,
        }
    }

    /// An injected counter, for the exhaustion boundaries E3 exercises.
    pub fn starting_at(what: &'static str, first: u32) -> Self {
        Self {
            next: AtomicU64::new(u64::from(first.max(1))),
            what,
        }
    }

    /// The next value that would be issued, saturating one past [`MAX_ID`].
    pub fn peek(&self) -> u64 {
        self.next.load(Ordering::SeqCst)
    }

    /// Issue the next id.
    ///
    /// # Errors
    /// [`Exhausted`] once [`MAX_ID`] ids have been issued. The counter saturates
    /// one past the limit, so it can never wrap back onto a live id and every
    /// later attempt fails the same way.
    pub fn allocate(&self) -> Result<u32, Exhausted> {
        self.next
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
                (value <= u64::from(MAX_ID)).then(|| value + 1)
            })
            .map(|value| value as u32)
            .map_err(|_| Exhausted { what: self.what })
    }
}

/// The one process-global arena counter. Every node or symbol arena created in
/// the process, file core, lazy, checker, transform, builder generation or
/// scratch, takes the next value.
static ARENA_IDS: IdCounter = IdCounter::new("arena");

/// Allocate the next process-global arena id.
///
/// # Errors
/// [`Exhausted`] once [`MAX_ID`] arena ids have been issued. The counter
/// saturates rather than wrapping, so a later attempt fails the same way and no
/// id is ever reused.
pub fn allocate_arena_id() -> Result<ArenaId, Exhausted> {
    ARENA_IDS
        .allocate()
        .and_then(|value| ArenaId::from_raw(value).ok_or(Exhausted { what: "arena" }))
}

/// A per-arena slot counter with the same checked, never-wrapping contract.
#[derive(Debug)]
pub struct SlotCounter(IdCounter);

impl SlotCounter {
    pub fn new() -> Self {
        Self(IdCounter::new("slot"))
    }

    /// An injected slot counter for the E3 exhaustion boundary.
    pub fn starting_at(first: u32) -> Self {
        Self(IdCounter::starting_at("slot", first))
    }

    /// # Errors
    /// [`Exhausted`] beyond [`MAX_ID`] slots, before truncation or reuse.
    pub fn allocate(&self) -> Result<Slot, Exhausted> {
        self.0.allocate().and_then(|value| {
            NonZeroU32::new(value)
                .map(Slot)
                .ok_or(Exhausted { what: "slot" })
        })
    }

    pub fn issued(&self) -> u64 {
        self.0.peek() - 1
    }
}

impl Default for SlotCounter {
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! entity_id {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(NonZeroU64);

        impl $name {
            /// Compose an id from its arena and slot. Both are nonzero, so the
            /// result has a niche and `Option<Self>` still costs one word.
            pub fn new(arena: ArenaId, slot: Slot) -> Self {
                let raw = (u64::from(arena.get()) << 32) | u64::from(slot.get());
                Self(NonZeroU64::new(raw).expect("arena and slot are nonzero"))
            }

            pub fn arena(self) -> ArenaId {
                ArenaId::from_raw((self.0.get() >> 32) as u32).expect("arena is nonzero")
            }

            pub fn slot(self) -> Slot {
                Slot(NonZeroU32::new(self.0.get() as u32).expect("slot is nonzero"))
            }

            pub fn raw(self) -> u64 {
                self.0.get()
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    concat!(stringify!($name), "({}:{})"),
                    self.arena().get(),
                    self.slot().get()
                )
            }
        }
    };
}

entity_id!(
    NodeId,
    "A node's identity: arena in the high 32 bits, slot in the low 32.\n\nIds never keep storage alive. Anything that stores one for later must retain\nthe owner as well."
);
entity_id!(
    SymbolId,
    "A symbol's identity, with the same layout and the same never-reused arena\nspace as [`NodeId`] (ADR 0007)."
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_survive_the_31_bit_boundary() {
        for arena in [1, 0x7FFF_FFFF, 0x8000_0000, MAX_ID] {
            for slot in [1, 0x7FFF_FFFF, 0x8000_0000, MAX_ID] {
                let id = NodeId::new(
                    ArenaId::from_raw(arena).unwrap(),
                    Slot::from_raw(slot).unwrap(),
                );
                assert_eq!(id.arena().get(), arena);
                assert_eq!(id.slot().get(), slot);
            }
        }
    }

    #[test]
    fn option_of_an_id_costs_one_word() {
        assert_eq!(
            std::mem::size_of::<Option<NodeId>>(),
            std::mem::size_of::<NodeId>()
        );
        assert_eq!(
            std::mem::size_of::<Option<SymbolId>>(),
            std::mem::size_of::<SymbolId>()
        );
    }

    #[test]
    fn a_counter_saturates_instead_of_wrapping() {
        let counter = IdCounter::starting_at("arena", MAX_ID);
        assert_eq!(counter.allocate(), Ok(MAX_ID));
        assert!(counter.allocate().is_err());
        assert!(counter.allocate().is_err());
        assert_eq!(counter.peek(), u64::from(MAX_ID) + 1);
    }
}
