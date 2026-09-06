//! Owners: file, bundle and scratch (docs/design/ownership.md, 2.1, 2.5).
//!
//! Disposal is reference counting; there is no explicit free. Owning references
//! form a DAG: a bundle owns its member files, and canonical/supplemental links
//! inside a bundle are ids, not references.

use std::hash::Hash;
use std::sync::{Arc, OnceLock};

use crate::arena::Arena;
use crate::counters::OwnerGuard;
use crate::id::{ArenaId, NodeId};
use crate::lazy::LazyArena;

/// A file is identified by its core arena's id.
pub type FileId = ArenaId;

/// The bidirectional bundle links, as ids.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BundleLinks {
    pub canonical: FileId,
    pub supplementals: Vec<FileId>,
}

/// The immutable parsed core, the lazy storage and the bundle links of one file.
pub struct FileOwner<T, K> {
    core: Arena<T>,
    lazy: LazyArena<T, K>,
    bundle: OnceLock<BundleLinks>,
    _owner: OwnerGuard,
}

impl<T, K: Hash + Eq> FileOwner<T, K> {
    /// Takes the parsed core arena, seals it, and creates the lazy arena from the
    /// same counter family (the global counter).
    pub fn new(mut core: Arena<T>) -> Self {
        core.seal();
        Self {
            core,
            lazy: LazyArena::new(),
            bundle: OnceLock::new(),
            _owner: OwnerGuard::new(),
        }
    }

    /// As [`FileOwner::new`], with a caller-supplied lazy arena (tests inject counters).
    pub fn with_lazy(mut core: Arena<T>, lazy: LazyArena<T, K>) -> Self {
        core.seal();
        Self {
            core,
            lazy,
            bundle: OnceLock::new(),
            _owner: OwnerGuard::new(),
        }
    }

    pub fn file_id(&self) -> FileId {
        self.core.id()
    }

    pub fn core(&self) -> &Arena<T> {
        &self.core
    }

    pub fn lazy(&self) -> &LazyArena<T, K> {
        &self.lazy
    }

    pub fn lazy_arena_id(&self) -> ArenaId {
        self.lazy.id()
    }

    /// Whether `id` names a published node of this file's core or lazy arena.
    pub fn owns(&self, id: NodeId) -> bool {
        self.core.contains(id) || self.lazy.get(id).is_some()
    }

    pub fn bundle_links(&self) -> Option<&BundleLinks> {
        self.bundle.get()
    }

    fn set_bundle_links(&self, links: BundleLinks) {
        assert!(
            self.bundle.set(links).is_ok(),
            "a file belongs to at most one bundle"
        );
    }
}

/// A content-mapped canonical file and its supplemental files, owned together.
pub struct BundleOwner<T, K> {
    canonical: Arc<FileOwner<T, K>>,
    supplementals: Vec<Arc<FileOwner<T, K>>>,
    _owner: OwnerGuard,
}

impl<T, K: Hash + Eq> BundleOwner<T, K> {
    /// Groups the files and records the id links in both directions.
    pub fn new(
        canonical: Arc<FileOwner<T, K>>,
        supplementals: Vec<Arc<FileOwner<T, K>>>,
    ) -> Arc<Self> {
        let links = BundleLinks {
            canonical: canonical.file_id(),
            supplementals: supplementals.iter().map(|f| f.file_id()).collect(),
        };
        canonical.set_bundle_links(links.clone());
        for supplemental in &supplementals {
            supplemental.set_bundle_links(links.clone());
        }
        Arc::new(Self {
            canonical,
            supplementals,
            _owner: OwnerGuard::new(),
        })
    }

    pub fn canonical(&self) -> &Arc<FileOwner<T, K>> {
        &self.canonical
    }

    pub fn supplementals(&self) -> &[Arc<FileOwner<T, K>>] {
        &self.supplementals
    }

    pub fn members(&self) -> impl Iterator<Item = &Arc<FileOwner<T, K>>> {
        std::iter::once(&self.canonical).chain(self.supplementals.iter())
    }
}

/// Short-lived storage for options, formatting and API print/format requests.
/// Nothing retains it after the operation returns.
pub struct ScratchOwner<T> {
    arena: Arena<T>,
    _owner: OwnerGuard,
}

impl<T> ScratchOwner<T> {
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
            _owner: OwnerGuard::new(),
        }
    }

    pub fn arena_id(&self) -> ArenaId {
        self.arena.id()
    }

    pub fn alloc(&mut self, value: T) -> NodeId {
        self.arena.alloc(value)
    }

    pub fn get(&self, id: NodeId) -> Option<&T> {
        if id.arena() == self.arena.id() {
            self.arena.get(id.slot())
        } else {
            None
        }
    }

    pub fn arena(&self) -> &Arena<T> {
        &self.arena
    }
}

impl<T> Default for ScratchOwner<T> {
    fn default() -> Self {
        Self::new()
    }
}
