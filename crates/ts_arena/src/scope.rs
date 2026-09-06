//! Resolution and the validated scope (docs/design/ownership.md, section 2.3).
//!
//! Resolution always goes through an object that holds `Arc` references to
//! arenas. Such an object is a **validated scope**: it can only map an `ArenaId`
//! to an arena it holds, so an id from an arena it does not hold is rejected,
//! never aliased. Because arena ids are never reused, a dropped arena's ids fail
//! here for every scope in the process.
//!
//! Two access paths exist, and the difference between them is the whole of the
//! check-elision rule:
//!
//! * [`Scope::import`] is the checked path for a raw id: ids read from caches,
//!   ids arriving through callbacks or reentrant paths, ids from API handles and
//!   any id whose arena is not the current one.
//! * [`with_scoped_arena`] is the scoped path. It checks a raw id once against
//!   one arena and its published bounds and mints a [`LocalNode`] carrying a
//!   fresh invariant brand. That handle cannot be constructed by callers, cannot
//!   be used with another arena, and cannot escape its scope, so repeated access
//!   through it may elide the owner check while keeping bounds safety. An
//!   ordinary copied `NodeId` carries no such proof and still takes the checked
//!   path.

use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::checker::{CheckerId, CheckerOwner, OperationPermit, PoolGeneration, StaleId};
use crate::core_arena::CoreArena;
use crate::ids::{ArenaId, NodeId, Slot};
use crate::lazy::LazyRef;
use crate::node::NodeData;
use crate::owner::{BundleOwner, FileOwner, ScratchOwner};

/// What kind of owner an arena belongs to. This is resolver metadata; it is
/// never encoded in an id, so no arena bit is stolen for it and owner kinds stay
/// distinct across the full 32-bit arena range.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OwnerKind {
    /// A file's core arena: parsed nodes, immutable after binding.
    FileCore,
    /// A file's lazy arena: JSDoc and tokens created after binding.
    FileLazy,
    /// A checker's own AST arena, with exact checker identity and generation.
    Checker {
        checker: CheckerId,
        generation: PoolGeneration,
    },
    /// Request scratch.
    Scratch,
}

/// A resolved node, still tied to the owner the scope retains.
pub enum NodeRef<'a> {
    Core(&'a NodeData),
    Lazy(LazyRef),
    Checker(NodeData),
}

impl NodeRef<'_> {
    pub fn data(&self) -> &NodeData {
        match self {
            NodeRef::Core(node) => node,
            NodeRef::Lazy(node) => node.node(),
            NodeRef::Checker(node) => node,
        }
    }
}

enum OwnerHandle {
    File(Arc<FileOwner>),
    Bundle(Arc<BundleOwner>, Arc<FileOwner>),
    Checker(Arc<CheckerOwner>),
    Scratch(Arc<ScratchOwner>),
}

struct ScopeEntry {
    kind: OwnerKind,
    owner: OwnerHandle,
}

/// A program's file set, an emit context, a node builder, a snapshot's data or
/// an API registry: anything that holds owners and resolves ids through them.
#[derive(Default)]
pub struct Scope {
    entries: BTreeMap<ArenaId, ScopeEntry>,
}

impl Scope {
    pub fn new() -> Self {
        Self::default()
    }

    /// Retain a file owner and both of its arenas.
    pub fn add_file(&mut self, file: &Arc<FileOwner>) {
        self.entries.insert(
            file.core().id(),
            ScopeEntry {
                kind: OwnerKind::FileCore,
                owner: OwnerHandle::File(Arc::clone(file)),
            },
        );
        self.entries.insert(
            file.lazy().id(),
            ScopeEntry {
                kind: OwnerKind::FileLazy,
                owner: OwnerHandle::File(Arc::clone(file)),
            },
        );
    }

    /// Retain a bundle. Its ownership and resolution set includes every member,
    /// which is why a standalone file `Arc` cannot follow the sibling links.
    pub fn add_bundle(&mut self, bundle: &Arc<BundleOwner>) {
        for file in bundle.members() {
            for (arena, kind) in [
                (file.core().id(), OwnerKind::FileCore),
                (file.lazy().id(), OwnerKind::FileLazy),
            ] {
                self.entries.insert(
                    arena,
                    ScopeEntry {
                        kind,
                        owner: OwnerHandle::Bundle(Arc::clone(bundle), Arc::clone(file)),
                    },
                );
            }
        }
    }

    /// Retain a checker's own AST arena, recorded with its exact identity and
    /// pool generation.
    pub fn add_checker(&mut self, checker: &Arc<CheckerOwner>) {
        self.entries.insert(
            checker.ast_arena_id(),
            ScopeEntry {
                kind: OwnerKind::Checker {
                    checker: checker.id(),
                    generation: checker.generation(),
                },
                owner: OwnerHandle::Checker(Arc::clone(checker)),
            },
        );
    }

    /// Retain request scratch for the duration of one operation.
    pub fn add_scratch(&mut self, scratch: &Arc<ScratchOwner>) {
        self.entries.insert(
            scratch.id(),
            ScopeEntry {
                kind: OwnerKind::Scratch,
                owner: OwnerHandle::Scratch(Arc::clone(scratch)),
            },
        );
    }

    /// The bundle that retains an arena, when the scope holds it as part of one.
    /// Following a canonical or supplemental link needs this: the ids in a
    /// file's mapper info own nothing, so only a bundle holder can resolve them.
    pub fn bundle(&self, arena: ArenaId) -> Option<&Arc<BundleOwner>> {
        match self.entries.get(&arena) {
            Some(ScopeEntry {
                owner: OwnerHandle::Bundle(bundle, _),
                ..
            }) => Some(bundle),
            _ => None,
        }
    }

    /// The owner kind recorded for an arena, or `None` when this scope does not
    /// hold it.
    pub fn owner_kind(&self, arena: ArenaId) -> Option<OwnerKind> {
        self.entries.get(&arena).map(|entry| entry.kind)
    }

    pub fn arenas(&self) -> impl Iterator<Item = ArenaId> + '_ {
        self.entries.keys().copied()
    }

    /// The checked path. An absent arena means the owner is not retained here,
    /// so the id is rejected rather than aliased onto whatever now occupies
    /// those numbers.
    ///
    /// # Errors
    /// [`StaleId::UnknownArena`] for an arena this scope does not hold,
    /// [`StaleId::OutOfBounds`] for a slot past the arena's published bounds,
    /// and [`StaleId::WrongChecker`] for checker storage reached without that
    /// exact checker's operation permit.
    pub fn import(&self, id: NodeId) -> Result<NodeRef<'_>, StaleId> {
        self.import_inner(id, None)
    }

    /// The checked path for checker-owned storage, which additionally requires
    /// that exact checker's operation permit.
    ///
    /// # Errors
    /// As [`Scope::import`], plus [`StaleId::WrongChecker`] when the permit
    /// belongs to another checker and [`StaleId::RetiredGeneration`] when it
    /// belongs to another pool generation.
    pub fn import_with_permit(
        &self,
        id: NodeId,
        permit: &OperationPermit<'_>,
    ) -> Result<NodeRef<'_>, StaleId> {
        self.import_inner(id, Some(permit))
    }

    fn import_inner(
        &self,
        id: NodeId,
        permit: Option<&OperationPermit<'_>>,
    ) -> Result<NodeRef<'_>, StaleId> {
        let entry = self.entries.get(&id.arena()).ok_or(StaleId::UnknownArena)?;
        match (&entry.kind, &entry.owner) {
            (OwnerKind::FileCore, OwnerHandle::File(file) | OwnerHandle::Bundle(_, file)) => file
                .core()
                .get(id.slot())
                .map(NodeRef::Core)
                .ok_or(StaleId::OutOfBounds),
            (OwnerKind::FileLazy, OwnerHandle::File(file) | OwnerHandle::Bundle(_, file)) => file
                .lazy()
                .resolve(id)
                .map(NodeRef::Lazy)
                .ok_or(StaleId::OutOfBounds),
            (
                OwnerKind::Checker {
                    checker,
                    generation,
                },
                OwnerHandle::Checker(owner),
            ) => {
                // Checker-owned nodes need the exact checker's permit; two
                // checkers of one pool generation can hold equal numeric slots.
                let permit = permit.ok_or(StaleId::WrongChecker)?;
                if permit.checker() != *checker {
                    return Err(StaleId::WrongChecker);
                }
                if permit.generation() != *generation {
                    return Err(StaleId::RetiredGeneration);
                }
                owner
                    .node(id)
                    .map(NodeRef::Checker)
                    .ok_or(StaleId::OutOfBounds)
            }
            (OwnerKind::Scratch, OwnerHandle::Scratch(scratch)) => scratch
                .arena()
                .get(id.slot())
                .map(NodeRef::Core)
                .ok_or(StaleId::OutOfBounds),
            _ => Err(StaleId::UnknownArena),
        }
    }
}

/// An invariant lifetime brand. `fn(&'brand ()) -> &'brand ()` is invariant in
/// `'brand`, so two scopes can never unify and a handle from one cannot be used
/// with the other.
type Brand<'brand> = PhantomData<fn(&'brand ()) -> &'brand ()>;

/// One arena, opened for scoped access. The scope holds the owner, so storage
/// cannot be replaced and slots cannot be reused while it is open.
pub struct ScopedArena<'a, 'brand> {
    arena: &'a CoreArena,
    _brand: Brand<'brand>,
}

/// A slot proven to belong to one open [`ScopedArena`] and to be within its
/// published bounds. Private constructor, invariant brand: it cannot be built by
/// callers, used with another arena, or escape its scope.
#[derive(Clone, Copy, Debug)]
pub struct LocalNode<'brand> {
    slot: Slot,
    _brand: Brand<'brand>,
}

impl<'a, 'brand> ScopedArena<'a, 'brand> {
    /// Check a raw id against this arena and its published bounds, minting a
    /// local handle. This is the one owner check; later access through the
    /// handle may elide it.
    pub fn check(&self, id: NodeId) -> Option<LocalNode<'brand>> {
        if id.arena() != self.arena.id() || self.arena.get(id.slot()).is_none() {
            return None;
        }
        Some(LocalNode {
            slot: id.slot(),
            _brand: PhantomData,
        })
    }

    /// Resolve a local handle. The owner check is elided; the bounds check is
    /// not, because bounds safety is never elided.
    pub fn get(&self, local: LocalNode<'brand>) -> &'a NodeData {
        self.arena
            .get(local.slot)
            .expect("a local handle is checked against published bounds")
    }

    /// The id a local handle stands for, for storing it back into a cache.
    pub fn id(&self, local: LocalNode<'brand>) -> NodeId {
        NodeId::new(self.arena.id(), local.slot)
    }
}

/// Open one core arena for scoped access. The brand is fresh for each call, so
/// no handle can leak into another scope.
pub fn with_scoped_arena<R>(
    arena: &CoreArena,
    body: impl for<'brand> FnOnce(ScopedArena<'_, 'brand>) -> R,
) -> R {
    body(ScopedArena {
        arena,
        _brand: PhantomData,
    })
}
