//! The core arena: append-only during parsing, immutable after sealing, readable
//! without locks (docs/design/ownership.md, 2.1 and 2.4).

use std::num::NonZeroU32;

use crate::counters::AllocationGuard;
use crate::id::{ArenaCounter, ArenaId, NodeId, SlotExhausted};

pub struct Arena<T> {
    id: ArenaId,
    first_slot: u32,
    nodes: Vec<T>,
    sealed: bool,
    _allocation: AllocationGuard,
}

impl<T> Arena<T> {
    /// A new arena with a fresh id from the global counter and slots from 1.
    pub fn new() -> Self {
        Self::with_counter(ArenaCounter::global())
    }

    pub fn with_counter(counter: &ArenaCounter) -> Self {
        Self::with_first_slot(counter, NonZeroU32::MIN)
    }

    /// Starts slot numbering at `first_slot`; tests use it to reach the slot boundary.
    pub fn with_first_slot(counter: &ArenaCounter, first_slot: NonZeroU32) -> Self {
        Self {
            id: counter.allocate(),
            first_slot: first_slot.get(),
            nodes: Vec::new(),
            sealed: false,
            _allocation: AllocationGuard::new(),
        }
    }

    pub fn id(&self) -> ArenaId {
        self.id
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn is_sealed(&self) -> bool {
        self.sealed
    }

    /// The slot the next allocation receives, as a 64-bit value so exhaustion is
    /// detected before any truncation.
    fn next_slot(&self) -> u64 {
        u64::from(self.first_slot) + self.nodes.len() as u64
    }

    /// Appends a node, failing before the slot space could wrap or a slot be reused.
    ///
    /// # Panics
    /// When the arena is sealed: allocation after binding is an invariant failure.
    pub fn try_alloc(&mut self, value: T) -> Result<NodeId, SlotExhausted> {
        assert!(
            !self.sealed,
            "allocation into a sealed core arena {}",
            self.id.get()
        );
        let slot = self.next_slot();
        if slot > u64::from(u32::MAX) {
            return Err(SlotExhausted { arena: self.id });
        }
        self.nodes.push(value);
        Ok(NodeId::new(
            self.id,
            NonZeroU32::new(slot as u32).expect("slots start at 1"),
        ))
    }

    /// Appends a node; exhaustion is an invariant failure (ADR 0012).
    ///
    /// # Panics
    /// When slots are exhausted or the arena is sealed.
    pub fn alloc(&mut self, value: T) -> NodeId {
        match self.try_alloc(value) {
            Ok(id) => id,
            Err(e) => panic!("{e}"),
        }
    }

    /// Ends allocation; the arena is immutable from here on.
    pub fn seal(&mut self) {
        self.sealed = true;
    }

    /// The node at a slot of this arena, if published.
    pub fn get(&self, slot: u32) -> Option<&T> {
        let index = slot.checked_sub(self.first_slot)?;
        self.nodes.get(index as usize)
    }

    /// Whether `id` names a published slot of this arena.
    pub fn contains(&self, id: NodeId) -> bool {
        id.arena() == self.id && self.get(id.slot()).is_some()
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_start_at_one_and_never_repeat() {
        let mut arena = Arena::new();
        let a = arena.alloc("a");
        let b = arena.alloc("b");
        assert_eq!(a.slot(), 1);
        assert_eq!(b.slot(), 2);
        assert_eq!(arena.get(0), None, "slot 0 is reserved");
        assert_eq!(arena.get(1), Some(&"a"));
        assert!(arena.contains(b));
    }

    #[test]
    fn slot_exhaustion_before_wrap() {
        let counter = ArenaCounter::starting_at(1);
        let mut arena = Arena::with_first_slot(&counter, NonZeroU32::new(u32::MAX - 1).unwrap());
        assert_eq!(arena.alloc(1).slot(), u32::MAX - 1);
        assert_eq!(arena.alloc(2).slot(), u32::MAX);
        assert!(arena.try_alloc(3).is_err());
        assert_eq!(arena.len(), 2);
        assert_eq!(arena.get(u32::MAX), Some(&2));
    }
}
