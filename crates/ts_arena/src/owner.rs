//! Owners (docs/design/ownership.md, section 2.1).
//!
//! Owning references form a directed acyclic graph. A file owner never retains a
//! checker or builder; canonical and supplemental links are ids, not references,
//! so they may be cyclic. Disposal is reference counting: a file's storage goes
//! away when its last owner reference drops, and because arena ids are never
//! reused a dropped arena's ids fail `import` everywhere.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

use ts_jsstring::{compute_position_map, PositionMap, SourceText};

use crate::core_arena::CoreArena;
use crate::counters::OwnerGuard;
use crate::ids::{ArenaId, Exhausted, NodeId};
use crate::lazy::LazyArena;
use crate::node::NodeData;

/// A file's identity: its core arena id, which is never reused.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct FileId(pub ArenaId);

/// `ast.ContentMapperSourceFileInfo`: both directions of a bundle's links, held
/// as ids so the cycle owns nothing.
#[derive(Clone, Debug, Default)]
pub struct ContentMapperInfo {
    /// Set on a supplemental file: the canonical file it belongs to.
    pub canonical: Option<FileId>,
    /// Set on the canonical file: its supplemental outputs.
    pub supplemental: Vec<FileId>,
}

/// Builds a file's core arena before the owner is published.
///
/// The core arena is immutable after binding and readable without locks, so it
/// is filled here and frozen by [`FileOwnerBuilder::finish`].
pub struct FileOwnerBuilder {
    core: CoreArena,
    text: SourceText,
}

impl FileOwnerBuilder {
    /// # Errors
    /// [`Exhausted`] when the process-global arena counter is spent.
    pub fn new(text: SourceText) -> Result<Self, Exhausted> {
        Ok(Self {
            core: CoreArena::new()?,
            text,
        })
    }

    /// # Errors
    /// [`Exhausted`] when the arena's slots are spent.
    pub fn allocate(&mut self, node: NodeData) -> Result<NodeId, Exhausted> {
        self.core.allocate(node)
    }

    pub fn core(&self) -> &CoreArena {
        &self.core
    }

    /// # Errors
    /// [`Exhausted`] when the process-global arena counter is spent.
    pub fn finish(self) -> Result<Arc<FileOwner>, Exhausted> {
        let position_map = compute_position_map(self.text.as_bytes());
        Ok(Arc::new(FileOwner {
            file_id: FileId(self.core.id()),
            core: self.core,
            lazy: LazyArena::new()?,
            position_map,
            text: self.text,
            mapper: OnceLock::new(),
            bound: AtomicBool::new(false),
            _owner: OwnerGuard::new(),
        }))
    }
}

/// Everything one parsed file owns: its core arena, its lazy arena, its caches,
/// its position map and its source text.
pub struct FileOwner {
    file_id: FileId,
    core: CoreArena,
    lazy: LazyArena,
    position_map: PositionMap,
    text: SourceText,
    mapper: OnceLock<ContentMapperInfo>,
    bound: AtomicBool,
    _owner: OwnerGuard,
}

impl FileOwner {
    pub fn id(&self) -> FileId {
        self.file_id
    }

    pub fn core(&self) -> &CoreArena {
        &self.core
    }

    pub fn lazy(&self) -> &LazyArena {
        &self.lazy
    }

    pub fn text(&self) -> &SourceText {
        &self.text
    }

    pub fn position_map(&self) -> &PositionMap {
        &self.position_map
    }

    // port: tsc/internal/ast/ast.go:SourceFile.BindOnce
    /// Bind exactly once, whichever caller gets there first.
    pub fn bind_once(&self, bind: impl FnOnce()) {
        if self
            .bound
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            bind();
        }
    }

    pub fn is_bound(&self) -> bool {
        self.bound.load(Ordering::SeqCst)
    }

    /// Record this file's place in a content-mapped bundle. Set once, by the
    /// bundle's construction.
    pub fn set_mapper_info(&self, info: ContentMapperInfo) -> bool {
        self.mapper.set(info).is_ok()
    }

    pub fn mapper_info(&self) -> Option<&ContentMapperInfo> {
        self.mapper.get()
    }

    /// Nodes alive in this file's two arenas.
    pub fn live_nodes(&self) -> i64 {
        self.core.live_nodes() + self.lazy.live_nodes()
    }
}

impl std::fmt::Debug for FileOwner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FileOwner({:?})", self.file_id)
    }
}

/// A content-mapped file and its supplemental outputs, owned as one unit
/// (docs/design/ownership.md, section 2.5).
///
/// The canonical file's info stores the supplemental file ids and each
/// supplemental file's info points back; both directions are ids, and this owner
/// holds the `Arc`s. Programs, snapshots and escaped handles retain the bundle as
/// one unit, which is why a standalone file `Arc` is not sufficient for the
/// sibling links.
pub struct BundleOwner {
    canonical: Arc<FileOwner>,
    supplemental: Vec<Arc<FileOwner>>,
    _owner: OwnerGuard,
}

impl BundleOwner {
    /// Parse-time construction: the links are written in both directions before
    /// the bundle is published.
    pub fn new(canonical: Arc<FileOwner>, supplemental: Vec<Arc<FileOwner>>) -> Arc<Self> {
        canonical.set_mapper_info(ContentMapperInfo {
            canonical: None,
            supplemental: supplemental.iter().map(|file| file.id()).collect(),
        });
        for file in &supplemental {
            file.set_mapper_info(ContentMapperInfo {
                canonical: Some(canonical.id()),
                supplemental: Vec::new(),
            });
        }
        Arc::new(Self {
            canonical,
            supplemental,
            _owner: OwnerGuard::new(),
        })
    }

    pub fn canonical(&self) -> &Arc<FileOwner> {
        &self.canonical
    }

    pub fn supplemental(&self) -> &[Arc<FileOwner>] {
        &self.supplemental
    }

    /// Every member, canonical first. A bundle's ownership and resolution set
    /// includes all of them.
    pub fn members(&self) -> impl Iterator<Item = &Arc<FileOwner>> {
        std::iter::once(&self.canonical).chain(self.supplemental.iter())
    }

    /// Resolve a sibling link. Only a bundle holder can follow one.
    pub fn file(&self, id: FileId) -> Option<&Arc<FileOwner>> {
        self.members().find(|file| file.id() == id)
    }
}

impl std::fmt::Debug for BundleOwner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BundleOwner({:?} + {} supplemental)",
            self.canonical.id(),
            self.supplemental.len()
        )
    }
}

/// Request scratch: short-lived nodes for options, formatting and API print
/// requests. Nothing retains it after its operation returns.
pub struct ScratchOwner {
    core: CoreArena,
    _owner: OwnerGuard,
}

impl ScratchOwner {
    /// # Errors
    /// [`Exhausted`] when the process-global arena counter is spent.
    pub fn new() -> Result<Self, Exhausted> {
        Ok(Self {
            core: CoreArena::new()?,
            _owner: OwnerGuard::new(),
        })
    }

    /// # Errors
    /// [`Exhausted`] when the arena's slots are spent.
    pub fn allocate(&mut self, node: NodeData) -> Result<NodeId, Exhausted> {
        self.core.allocate(node)
    }

    /// Publish the finished scratch arena so a scope can retain it for the rest
    /// of the request.
    pub fn into_shared(self) -> Arc<Self> {
        Arc::new(self)
    }

    pub fn arena(&self) -> &CoreArena {
        &self.core
    }

    pub fn id(&self) -> ArenaId {
        self.core.id()
    }
}

impl std::fmt::Debug for ScratchOwner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ScratchOwner({:?})", self.core.id())
    }
}
