//! Validated scopes (docs/design/ownership.md, 2.3).
//!
//! Resolution goes through an object holding `Arc` references to arenas. It can
//! only map an arena id to storage it retains, so a foreign, stale or recycled
//! id is rejected rather than aliased, in release builds too. Scoped access
//! through a branded local handle elides the owner check after one validation.

use std::collections::BTreeMap;
use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;

use crate::arena::Arena;
use crate::id::{ArenaId, NodeId};
use crate::lazy::LazyRef;
use crate::owner::{BundleOwner, FileOwner, ScratchOwner};

/// Why an id could not be resolved here.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StaleId {
    /// The scope retains no arena with this id: wrong owner, dropped, or never held.
    WrongOwner { arena: ArenaId },
    /// The arena is held but the slot is not published in it.
    UnpublishedSlot { id: NodeId },
}

impl fmt::Display for StaleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StaleId::WrongOwner { arena } => {
                write!(f, "arena {} is not retained by this scope", arena.get())
            }
            StaleId::UnpublishedSlot { id } => write!(f, "{id:?} is not a published slot"),
        }
    }
}

impl std::error::Error for StaleId {}

/// A resolved node: a core-arena borrow or a page-retaining lazy reference.
pub enum NodeRef<'a, T> {
    Core(&'a T),
    Lazy(LazyRef<T>),
}

impl<T: std::fmt::Debug> std::fmt::Debug for NodeRef<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&**self, f)
    }
}

impl<T> Deref for NodeRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        match self {
            NodeRef::Core(node) => node,
            NodeRef::Lazy(node) => node,
        }
    }
}

enum Entry<T, K> {
    Core(Arc<FileOwner<T, K>>),
    Lazy(Arc<FileOwner<T, K>>),
    Scratch(Arc<ScratchOwner<T>>),
}

/// A validated scope: the owner table of one program, emit context, builder,
/// snapshot or registry.
pub struct Scope<T, K> {
    entries: BTreeMap<ArenaId, Entry<T, K>>,
    bundles: Vec<Arc<BundleOwner<T, K>>>,
}

impl<T, K: Hash + Eq> Default for Scope<T, K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, K: Hash + Eq> Scope<T, K> {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            bundles: Vec::new(),
        }
    }

    /// Retains a standalone file: its core and lazy arenas become resolvable.
    pub fn retain_file(&mut self, file: &Arc<FileOwner<T, K>>) {
        self.entries
            .insert(file.file_id(), Entry::Core(Arc::clone(file)));
        self.entries
            .insert(file.lazy_arena_id(), Entry::Lazy(Arc::clone(file)));
    }

    /// Retains a bundle as one unit: every member file plus the bundle itself.
    pub fn retain_bundle(&mut self, bundle: &Arc<BundleOwner<T, K>>) {
        for member in bundle.members() {
            self.retain_file(member);
        }
        self.bundles.push(Arc::clone(bundle));
    }

    pub fn retain_scratch(&mut self, scratch: &Arc<ScratchOwner<T>>) {
        self.entries
            .insert(scratch.arena_id(), Entry::Scratch(Arc::clone(scratch)));
    }

    /// Drops the scope's references to a file; the storage lives on if retained elsewhere.
    pub fn release_file(&mut self, file: &FileOwner<T, K>) {
        self.entries.remove(&file.file_id());
        self.entries.remove(&file.lazy_arena_id());
    }

    pub fn holds(&self, arena: ArenaId) -> bool {
        self.entries.contains_key(&arena)
    }

    pub fn arena_count(&self) -> usize {
        self.entries.len()
    }

    /// Imported access: checked resolution of any id, including ids from caches,
    /// callbacks and other scopes.
    pub fn import(&self, id: NodeId) -> Result<NodeRef<'_, T>, StaleId> {
        let arena = id.arena();
        match self.entries.get(&arena) {
            None => Err(StaleId::WrongOwner { arena }),
            Some(Entry::Core(file)) => file
                .core()
                .get(id.slot())
                .map(NodeRef::Core)
                .ok_or(StaleId::UnpublishedSlot { id }),
            Some(Entry::Lazy(file)) => file
                .lazy()
                .get(id)
                .map(NodeRef::Lazy)
                .ok_or(StaleId::UnpublishedSlot { id }),
            Some(Entry::Scratch(scratch)) => scratch
                .get(id)
                .map(NodeRef::Core)
                .ok_or(StaleId::UnpublishedSlot { id }),
        }
    }

    /// Scoped access to one core arena. The closure receives a branded handle;
    /// slots it validates cannot be used with another arena or escape the closure.
    pub fn with_core_arena<R>(
        &self,
        arena: ArenaId,
        f: impl for<'id> FnOnce(LocalArena<'id, '_, T>) -> R,
    ) -> Option<R> {
        let core = match self.entries.get(&arena)? {
            Entry::Core(file) => file.core(),
            Entry::Scratch(scratch) => scratch.arena(),
            Entry::Lazy(_) => return None,
        };
        Some(f(LocalArena {
            core,
            _brand: PhantomData,
        }))
    }
}

/// Invariant lifetime brand: `fn(&'id ()) -> &'id ()` is neither covariant nor
/// contravariant in `'id`, so two handles never unify.
type Brand<'id> = PhantomData<fn(&'id ()) -> &'id ()>;

/// A checked local view of one core arena.
pub struct LocalArena<'id, 'a, T> {
    core: &'a Arena<T>,
    _brand: Brand<'id>,
}

/// A slot proven to be published in the arena of the same brand.
#[derive(Clone, Copy)]
pub struct Slot<'id> {
    slot: u32,
    _brand: Brand<'id>,
}

impl<'id, 'a, T> LocalArena<'id, 'a, T> {
    pub fn id(&self) -> ArenaId {
        self.core.id()
    }

    /// One owner-and-bounds check that mints a branded slot.
    pub fn check(&self, id: NodeId) -> Option<Slot<'id>> {
        if id.arena() == self.core.id() && self.core.get(id.slot()).is_some() {
            Some(Slot {
                slot: id.slot(),
                _brand: PhantomData,
            })
        } else {
            None
        }
    }

    /// Access through a branded slot: no owner check, bounds safety kept.
    pub fn get(&self, slot: Slot<'id>) -> &'a T {
        debug_assert!(self.core.get(slot.slot).is_some());
        self.core
            .get(slot.slot)
            .expect("a branded slot was validated against this arena")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(values: &[u32]) -> Arc<FileOwner<u32, u32>> {
        let mut core = Arena::new();
        for &v in values {
            core.alloc(v);
        }
        Arc::new(FileOwner::new(core))
    }

    #[test]
    fn import_rejects_wrong_owner_and_resolves_held() {
        let a = file(&[1, 2]);
        let b = file(&[3]);
        let mut scope = Scope::new();
        scope.retain_file(&a);
        let id_a = NodeId::new(a.file_id(), std::num::NonZeroU32::new(2).unwrap());
        let id_b = NodeId::new(b.file_id(), std::num::NonZeroU32::new(1).unwrap());
        assert_eq!(*scope.import(id_a).unwrap(), 2);
        assert_eq!(
            scope.import(id_b).unwrap_err(),
            StaleId::WrongOwner { arena: b.file_id() }
        );
        let bad_slot = NodeId::new(a.file_id(), std::num::NonZeroU32::new(9).unwrap());
        assert_eq!(
            scope.import(bad_slot).unwrap_err(),
            StaleId::UnpublishedSlot { id: bad_slot }
        );
    }

    #[test]
    fn branded_access() {
        let a = file(&[10, 20]);
        let mut scope = Scope::new();
        scope.retain_file(&a);
        let id = NodeId::new(a.file_id(), std::num::NonZeroU32::new(2).unwrap());
        let value = scope.with_core_arena(a.file_id(), |local| {
            let slot = local.check(id).unwrap();
            *local.get(slot)
        });
        assert_eq!(value, Some(20));
        assert!(scope.with_core_arena(a.lazy_arena_id(), |_| ()).is_none());
    }
}
