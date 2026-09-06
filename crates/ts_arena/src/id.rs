//! Arena and node identities (docs/design/ownership.md, 2.2; docs/design/symbols.md, 2.2).
//!
//! An id is 64 bits: the high 32 bits are a process-global, never-reused arena
//! id, the low 32 bits a slot. Arena 0 and slot 0 are reserved so the wrapper is
//! a `NonZeroU64` and `Option<Id>` needs no extra word. There is no owner-kind
//! bit; owner kind is resolver metadata.

use std::fmt;
use std::num::{NonZeroU32, NonZeroU64};
use std::sync::atomic::{AtomicU64, Ordering};

/// A never-reused arena identity, 1..=u32::MAX.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ArenaId(NonZeroU32);

impl ArenaId {
    pub fn get(self) -> u32 {
        self.0.get()
    }
}

macro_rules! packed_id {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(NonZeroU64);

        impl $name {
            /// Packs an arena id and a nonzero slot. Arenas mint ids; callers never do.
            pub fn new(arena: ArenaId, slot: NonZeroU32) -> Self {
                let packed = (u64::from(arena.get()) << 32) | u64::from(slot.get());
                Self(
                    NonZeroU64::new(packed)
                        .expect("a nonzero arena id makes the packed id nonzero"),
                )
            }

            pub fn arena(self) -> ArenaId {
                ArenaId(NonZeroU32::new((self.0.get() >> 32) as u32).expect("arena 0 is reserved"))
            }

            pub fn slot(self) -> u32 {
                self.0.get() as u32
            }

            /// The packed 64-bit value; never sent on the wire (ADR 0007).
            pub fn raw(self) -> u64 {
                self.0.get()
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    "{}({}.{})",
                    stringify!($name),
                    self.arena().get(),
                    self.slot()
                )
            }
        }
    };
}

packed_id!(
    NodeId,
    "A node identity: arena id in the high 32 bits, slot in the low 32 bits."
);
packed_id!(
    SymbolId,
    "A symbol identity with the same encoding as `NodeId`."
);

/// No arena id is left: the counter reached `u32::MAX` and must not wrap.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ArenaExhausted;

impl fmt::Display for ArenaExhausted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("arena ids exhausted: the process-global counter reached u32::MAX and ids are never reused")
    }
}

/// No slot is left in this arena: the next slot would exceed `u32::MAX`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SlotExhausted {
    pub arena: ArenaId,
}

impl fmt::Display for SlotExhausted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "arena {} exhausted its 32-bit slot space; slots are never reused",
            self.arena.get()
        )
    }
}

/// The checked allocator of arena ids. One process-global instance serves every
/// arena; tests inject a counter that starts near a boundary.
pub struct ArenaCounter {
    /// The next id to hand out, kept in 64 bits so the check happens before any
    /// 32-bit wrap could occur.
    next: AtomicU64,
}

impl ArenaCounter {
    /// A counter whose first id is `first`; `first` must be nonzero.
    pub const fn starting_at(first: u32) -> Self {
        assert!(first != 0, "arena 0 is reserved");
        Self {
            next: AtomicU64::new(first as u64),
        }
    }

    /// The process-global counter.
    pub fn global() -> &'static ArenaCounter {
        static GLOBAL: ArenaCounter = ArenaCounter::starting_at(1);
        &GLOBAL
    }

    /// Allocates the next id, or fails before the counter can pass `u32::MAX`.
    pub fn try_allocate(&self) -> Result<ArenaId, ArenaExhausted> {
        let mut current = self.next.load(Ordering::Relaxed);
        loop {
            if current > u64::from(u32::MAX) {
                return Err(ArenaExhausted);
            }
            match self.next.compare_exchange_weak(
                current,
                current + 1,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    return Ok(ArenaId(
                        NonZeroU32::new(current as u32).expect("counter starts at 1"),
                    ))
                }
                Err(observed) => current = observed,
            }
        }
    }

    /// Allocates the next id; exhaustion is an invariant failure (ADR 0012).
    ///
    /// # Panics
    /// When ids are exhausted.
    pub fn allocate(&self) -> ArenaId {
        match self.try_allocate() {
            Ok(id) => id,
            Err(e) => panic!("{e}"),
        }
    }

    /// The value the next allocation would receive; `u32::MAX + 1` once exhausted.
    pub fn peek_next(&self) -> u64 {
        self.next.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packing() {
        let counter = ArenaCounter::starting_at(0x8000_0000);
        let arena = counter.allocate();
        let id = NodeId::new(arena, NonZeroU32::new(7).unwrap());
        assert_eq!(id.arena(), arena);
        assert_eq!(id.slot(), 7);
        assert_eq!(id.raw(), 0x8000_0000_0000_0007);
        assert_eq!(std::mem::size_of::<Option<NodeId>>(), 8);
    }

    #[test]
    fn exhaustion_stops_before_wrap() {
        let counter = ArenaCounter::starting_at(u32::MAX - 1);
        assert_eq!(counter.allocate().get(), u32::MAX - 1);
        assert_eq!(counter.allocate().get(), u32::MAX);
        assert_eq!(counter.try_allocate(), Err(ArenaExhausted));
        assert_eq!(counter.try_allocate(), Err(ArenaExhausted));
        assert_eq!(counter.peek_next(), u64::from(u32::MAX) + 1);
    }
}
