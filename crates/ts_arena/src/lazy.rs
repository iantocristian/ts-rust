//! Lazy storage for nodes created after binding (docs/design/ownership.md, 2.4).
//!
//! Fixed 256-slot pages whose addresses never move, a directory of pages, the
//! publication bound and the key cache all live under one `RwLock`. Readers take
//! the read guard; a miss drops it, takes the same lock's write guard and
//! rechecks before allocating. A writer initializes every node it creates before
//! publishing them and the cache entry together. Published nodes are immutable.
//! Each slot is a `OnceLock`, so this needs no unsafe code: unpublished slots are
//! written through shared references by the single writer, and published slots
//! are only ever read.

use std::collections::HashMap;
use std::hash::Hash;
use std::num::NonZeroU32;
use std::ops::Deref;
use std::sync::{Arc, OnceLock};

use parking_lot::RwLock;

use crate::counters::AllocationGuard;
use crate::id::{ArenaCounter, ArenaId, NodeId, SlotExhausted};

pub const PAGE_SIZE: usize = 256;

struct Page<T> {
    slots: [OnceLock<T>; PAGE_SIZE],
    _allocation: AllocationGuard,
}

impl<T> Page<T> {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            slots: std::array::from_fn(|_| OnceLock::new()),
            _allocation: AllocationGuard::new(),
        })
    }
}

struct LazyState<T, K> {
    pages: Vec<Arc<Page<T>>>,
    /// Number of published slots; slots beyond it may be initialized but are invisible.
    published: u32,
    cache: HashMap<K, NodeId>,
}

pub struct LazyArena<T, K> {
    id: ArenaId,
    first_slot: u32,
    state: RwLock<LazyState<T, K>>,
    _allocation: AllocationGuard,
}

/// A reference to a published lazy node. It keeps its page alive, so it stays
/// valid across directory growth and after the lock is released.
pub struct LazyRef<T> {
    page: Arc<Page<T>>,
    index: usize,
}

impl<T> Deref for LazyRef<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.page.slots[self.index]
            .get()
            .expect("a published lazy slot is initialized")
    }
}

impl<T> LazyRef<T> {
    /// The node's address, stable for the owner's lifetime.
    pub fn address(&self) -> usize {
        std::ptr::from_ref::<T>(self).addr()
    }
}

/// The allocator handed to an initializer; everything it allocates is published
/// together when the initializer returns.
pub struct LazyAlloc<'a, T> {
    arena: ArenaId,
    first_slot: u32,
    pages: &'a mut Vec<Arc<Page<T>>>,
    published: u32,
    staged: u32,
}

impl<T> LazyAlloc<'_, T> {
    /// Allocates and initializes one node, failing before the slot space could wrap.
    pub fn try_alloc(&mut self, value: T) -> Result<NodeId, SlotExhausted> {
        let index = u64::from(self.published) + u64::from(self.staged);
        let slot = u64::from(self.first_slot) + index;
        if slot > u64::from(u32::MAX) {
            return Err(SlotExhausted { arena: self.arena });
        }
        let index = index as usize;
        if index / PAGE_SIZE == self.pages.len() {
            self.pages.push(Page::new());
        }
        let page = &self.pages[index / PAGE_SIZE];
        assert!(
            page.slots[index % PAGE_SIZE].set(value).is_ok(),
            "an unpublished lazy slot was already initialized"
        );
        self.staged += 1;
        Ok(NodeId::new(
            self.arena,
            NonZeroU32::new(slot as u32).expect("slots start at 1"),
        ))
    }

    /// # Panics
    /// When slots are exhausted (ADR 0012).
    pub fn alloc(&mut self, value: T) -> NodeId {
        match self.try_alloc(value) {
            Ok(id) => id,
            Err(e) => panic!("{e}"),
        }
    }
}

impl<T, K: Hash + Eq> LazyArena<T, K> {
    pub fn new() -> Self {
        Self::with_counter(ArenaCounter::global())
    }

    pub fn with_counter(counter: &ArenaCounter) -> Self {
        Self::with_first_slot(counter, NonZeroU32::MIN)
    }

    pub fn with_first_slot(counter: &ArenaCounter, first_slot: NonZeroU32) -> Self {
        Self {
            id: counter.allocate(),
            first_slot: first_slot.get(),
            state: RwLock::new(LazyState {
                pages: Vec::new(),
                published: 0,
                cache: HashMap::new(),
            }),
            _allocation: AllocationGuard::new(),
        }
    }

    pub fn id(&self) -> ArenaId {
        self.id
    }

    /// The id cached for `key`, initializing it under the write lock exactly once.
    ///
    /// # Panics
    /// When the initializer exhausts the slot space (ADR 0012).
    pub fn get_or_init(
        &self,
        key: K,
        init: impl FnOnce(&mut LazyAlloc<'_, T>) -> NodeId,
    ) -> NodeId {
        match self.try_get_or_init(key, |alloc| Ok(init(alloc))) {
            Ok(id) => id,
            Err(e) => panic!("{e}"),
        }
    }

    /// As [`LazyArena::get_or_init`], but an initializer that runs out of slots
    /// fails before anything is published or cached.
    pub fn try_get_or_init(
        &self,
        key: K,
        init: impl FnOnce(&mut LazyAlloc<'_, T>) -> Result<NodeId, SlotExhausted>,
    ) -> Result<NodeId, SlotExhausted> {
        {
            let state = self.state.read();
            if let Some(&id) = state.cache.get(&key) {
                return Ok(id);
            }
        }
        let mut state = self.state.write();
        if let Some(&id) = state.cache.get(&key) {
            return Ok(id);
        }
        let published = state.published;
        let mut alloc = LazyAlloc {
            arena: self.id,
            first_slot: self.first_slot,
            pages: &mut state.pages,
            published,
            staged: 0,
        };
        let id = init(&mut alloc)?;
        let staged = alloc.staged;
        let slot = u64::from(id.slot());
        assert!(
            id.arena() == self.id
                && slot >= u64::from(self.first_slot)
                && slot < u64::from(self.first_slot) + u64::from(published) + u64::from(staged),
            "an initializer must return an id it allocated in this arena"
        );
        state.published = published + staged;
        state.cache.insert(key, id);
        Ok(id)
    }

    /// A published node of this arena.
    pub fn get(&self, id: NodeId) -> Option<LazyRef<T>> {
        if id.arena() != self.id {
            return None;
        }
        let state = self.state.read();
        let index = id.slot().checked_sub(self.first_slot)?;
        if index >= state.published {
            return None;
        }
        let index = index as usize;
        Some(LazyRef {
            page: Arc::clone(&state.pages[index / PAGE_SIZE]),
            index: index % PAGE_SIZE,
        })
    }

    pub fn published(&self) -> u32 {
        self.state.read().published
    }

    pub fn page_count(&self) -> usize {
        self.state.read().pages.len()
    }
}

impl<T, K: Hash + Eq> Default for LazyArena<T, K> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_allocation_per_key_and_stable_addresses() {
        let arena: LazyArena<u32, u32> = LazyArena::new();
        let first = arena.get_or_init(7, |a| a.alloc(70));
        let again = arena.get_or_init(7, |_| unreachable!("cached"));
        assert_eq!(first, again);
        let r = arena.get(first).unwrap();
        let address = r.address();
        for key in 0..1000 {
            arena.get_or_init(key + 100, |a| a.alloc(key));
        }
        assert!(arena.page_count() >= 4);
        assert_eq!(*r, 70);
        assert_eq!(r.address(), address);
        assert_eq!(*arena.get(first).unwrap(), 70);
    }

    #[test]
    fn graph_publishes_together() {
        let arena: LazyArena<&str, u8> = LazyArena::new();
        let root = arena.get_or_init(1, |a| {
            let _child = a.alloc("child");
            a.alloc("root")
        });
        assert_eq!(root.slot(), 2);
        assert_eq!(arena.published(), 2);
        assert_eq!(*arena.get(root).unwrap(), "root");
    }
}
