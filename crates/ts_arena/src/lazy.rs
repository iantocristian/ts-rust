//! Lazy storage: the nodes a file gains after binding, on fixed pages published
//! under one lock (docs/design/ownership.md, section 2.4).
//!
//! One `RwLock` protects both caches, the page directory, the allocation
//! counters and publication. Readers take its read guard; a miss drops that
//! guard, takes the *same* lock's write guard and rechecks before allocating.
//! The writer initializes the complete node, then publishes the slot and the
//! cache entry before unlocking. There is no separate append mutex that bypasses
//! cache readers. This is the deliberate simplification of Go's two separate
//! mutexes recorded in the note.
//!
//! Pages are separately allocated and reference counted, so a published node's
//! address never moves and directory growth cannot invalidate it. A resolved
//! reference retains its page and its file owner, which is what lets it outlive
//! the guard without any unsafe aliasing: published slots are `OnceLock`s that
//! are written exactly once, under the write guard, before publication.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

use crate::counters::AllocationGuard;
use crate::ids::{allocate_arena_id, ArenaId, Exhausted, NodeId, Slot, SlotCounter};
use crate::node::{NodeData, NodeFlags, NodeKind};

/// Nodes per page, as the note fixes it.
pub const PAGE_SIZE: usize = 256;

struct Page {
    slots: [OnceLock<NodeData>; PAGE_SIZE],
}

impl Page {
    fn new() -> Self {
        Self {
            slots: std::array::from_fn(|_| OnceLock::new()),
        }
    }
}

/// The `(parent, range)` key Go uses for the token cache.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TokenKey {
    pub parent: NodeId,
    pub pos: u32,
    pub end: u32,
}

/// What a lazy request can refuse to do.
///
/// The kind mismatch and reparsed-parent cases are upstream panics. They are
/// returned from the fallible entry points so the refusal happens after the
/// lock guard is released: a panic taken while holding the guard would poison
/// the lock and turn one contract violation into a dead file owner.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LazyError {
    /// The arena's slots or the process's arena ids are spent.
    Exhausted(Exhausted),
    /// A cached token's kind disagrees with the requested kind.
    KindMismatch {
        cached: NodeKind,
        requested: NodeKind,
    },
    /// The supplied parent is a reparsed node, which can never own a lazily
    /// created token.
    ReparsedParent { kind: NodeKind },
}

impl std::fmt::Display for LazyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LazyError::Exhausted(e) => write!(f, "{e}"),
            LazyError::KindMismatch { cached, requested } => {
                write!(f, "Token cache mismatch: {cached:?} != {requested:?}")
            }
            LazyError::ReparsedParent { kind } => {
                write!(f, "Cannot create token from reparsed node of kind {kind:?}")
            }
        }
    }
}

#[derive(Default)]
struct LazyState {
    pages: Vec<Arc<Page>>,
    /// Slots published so far; slot indices `1..=published` are readable.
    published: u32,
    /// JSDoc parsed on first request, keyed by the owning node.
    jsdoc: HashMap<NodeId, Vec<NodeId>>,
    /// Tokens created on demand, keyed by `(parent, range)`.
    tokens: HashMap<TokenKey, NodeId>,
    allocations: AllocationGuard,
}

impl LazyState {
    fn page_for(slot: Slot) -> (usize, usize) {
        (slot.index() / PAGE_SIZE, slot.index() % PAGE_SIZE)
    }
}

/// The lazy arena of one file owner.
pub struct LazyArena {
    id: ArenaId,
    slots: SlotCounter,
    state: RwLock<LazyState>,
}

/// A published lazy node, tied to the page that holds it.
///
/// The page reference keeps the node's address valid; the caller separately
/// retains the file owner, because an id never keeps storage alive.
#[derive(Clone)]
pub struct LazyRef {
    id: NodeId,
    page: Arc<Page>,
    index: usize,
}

impl LazyRef {
    pub fn id(&self) -> NodeId {
        self.id
    }

    /// The node's data. Published slots are initialized before publication, so
    /// this cannot observe a partially built node.
    pub fn node(&self) -> &NodeData {
        self.page.slots[self.index]
            .get()
            .expect("a published slot is initialized before it is published")
    }
}

impl std::fmt::Debug for LazyRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LazyRef({:?})", self.id)
    }
}

impl LazyArena {
    /// # Errors
    /// [`Exhausted`] when the process-global arena counter is spent.
    pub fn new() -> Result<Self, Exhausted> {
        Ok(Self::with_ids(allocate_arena_id()?, SlotCounter::new()))
    }

    pub fn with_ids(id: ArenaId, slots: SlotCounter) -> Self {
        Self {
            id,
            slots,
            state: RwLock::new(LazyState::default()),
        }
    }

    pub fn id(&self) -> ArenaId {
        self.id
    }

    pub fn published(&self) -> u32 {
        self.state.read().expect("lazy state").published
    }

    pub fn pages(&self) -> usize {
        self.state.read().expect("lazy state").pages.len()
    }

    pub fn live_nodes(&self) -> i64 {
        self.state.read().expect("lazy state").allocations.live()
    }

    /// Resolve a published slot of this arena.
    pub fn resolve(&self, id: NodeId) -> Option<LazyRef> {
        if id.arena() != self.id {
            return None;
        }
        let state = self.state.read().expect("lazy state");
        Self::resolve_locked(&state, id)
    }

    fn resolve_locked(state: &LazyState, id: NodeId) -> Option<LazyRef> {
        if id.slot().get() > state.published {
            return None;
        }
        let (page, index) = LazyState::page_for(id.slot());
        Some(LazyRef {
            id,
            page: Arc::clone(state.pages.get(page)?),
            index,
        })
    }

    /// Allocate and publish one node under an already-held write guard.
    fn publish(&self, state: &mut LazyState, node: NodeData) -> Result<LazyRef, LazyError> {
        let slot = self.slots.allocate().map_err(LazyError::Exhausted)?;
        if slot.index() != state.published as usize {
            return Err(LazyError::Exhausted(Exhausted { what: "slot" }));
        }
        let (page_index, index) = LazyState::page_for(slot);
        while state.pages.len() <= page_index {
            // A new page is separately allocated, so growing the directory never
            // moves an already published node.
            state.pages.push(Arc::new(Page::new()));
        }
        let page = Arc::clone(&state.pages[page_index]);
        // Initialize the complete node first, then publish the slot. A slot that
        // was already set would mean the counter had handed one out twice; the
        // write is refused rather than overwriting a node a reader may hold.
        page.slots[index]
            .set(node)
            .map_err(|_| LazyError::Exhausted(Exhausted { what: "slot" }))?;
        state.published = slot.get();
        state.allocations.record(1);
        Ok(LazyRef {
            id: NodeId::new(self.id, slot),
            page,
            index,
        })
    }

    /// `SourceFile.GetOrCreateToken`: the cached token for `(parent, range)`, or
    /// a newly created and published one.
    ///
    /// # Panics
    /// When a cached token's kind disagrees with the requested kind, when the
    /// supplied parent is a reparsed node, and when the arena's slots are spent,
    /// exactly where upstream panics. Use
    /// [`LazyArena::try_get_or_create_token`] to observe the refusal instead.
    // port: tsc/internal/ast/ast.go:SourceFile.GetOrCreateToken
    // port: tsc/internal/ast/ast.go:createToken
    pub fn get_or_create_token(
        &self,
        key: TokenKey,
        kind: NodeKind,
        parent_flags: NodeFlags,
        text: Option<ts_jsstring::JsString>,
    ) -> LazyRef {
        match self.try_get_or_create_token(key, kind, parent_flags, text) {
            Ok(token) => token,
            Err(error) => panic!("{error}"),
        }
    }

    /// [`LazyArena::get_or_create_token`] with the refusal returned rather than
    /// raised, so no guard is held across a contract violation.
    ///
    /// # Errors
    /// [`LazyError`] for a cached kind mismatch, a reparsed parent or spent
    /// slots.
    pub fn try_get_or_create_token(
        &self,
        key: TokenKey,
        kind: NodeKind,
        parent_flags: NodeFlags,
        text: Option<ts_jsstring::JsString>,
    ) -> Result<LazyRef, LazyError> {
        // Fast path: the cache under the read guard.
        {
            let state = self.state.read().expect("lazy state");
            if let Some(id) = state.tokens.get(&key).copied() {
                let existing = Self::resolve_locked(&state, id).expect("cached token is published");
                let cached = existing.node().kind;
                return if cached == kind {
                    Ok(existing)
                } else {
                    Err(LazyError::KindMismatch {
                        cached,
                        requested: kind,
                    })
                };
            }
        }
        // Slow path: the same lock's write guard, then recheck.
        let mut state = self.state.write().expect("lazy state");
        if let Some(id) = state.tokens.get(&key).copied() {
            let existing = Self::resolve_locked(&state, id).expect("cached token is published");
            let cached = existing.node().kind;
            return if cached == kind {
                Ok(existing)
            } else {
                Err(LazyError::KindMismatch {
                    cached,
                    requested: kind,
                })
            };
        }
        if parent_flags.contains(NodeFlags::REPARSED) {
            return Err(LazyError::ReparsedParent { kind });
        }
        let mut node = NodeData::new(kind, key.pos, key.end).with_parent(key.parent);
        node.text = text;
        // The complete node is published, then the cache entry, both before the
        // write guard is released.
        let token = self.publish(&mut state, node)?;
        state.tokens.insert(key, token.id());
        Ok(token)
    }

    /// `SourceFile.resolveJSDoc`: the file's JSDoc for one node, parsed and
    /// published on first request.
    ///
    /// # Errors
    /// [`LazyError::Exhausted`] when the arena's slots are spent.
    // port: tsc/internal/ast/ast.go:SourceFile.resolveJSDoc
    pub fn resolve_jsdoc(
        &self,
        node: NodeId,
        parse: &dyn Fn() -> Vec<NodeData>,
    ) -> Result<Vec<LazyRef>, LazyError> {
        // Fast path: the cache under the read guard.
        {
            let state = self.state.read().expect("lazy state");
            if let Some(ids) = state.jsdoc.get(&node) {
                return Ok(ids
                    .iter()
                    .map(|id| Self::resolve_locked(&state, *id).expect("published"))
                    .collect());
            }
        }
        // Slow path: the same lock's write guard, then recheck.
        let mut state = self.state.write().expect("lazy state");
        if let Some(ids) = state.jsdoc.get(&node) {
            return Ok(ids
                .iter()
                .map(|id| Self::resolve_locked(&state, *id).expect("published"))
                .collect());
        }
        let parsed = parse();
        let mut refs = Vec::with_capacity(parsed.len());
        for data in parsed {
            refs.push(self.publish(&mut state, data)?);
        }
        state
            .jsdoc
            .insert(node, refs.iter().map(LazyRef::id).collect());
        Ok(refs)
    }
}

impl std::fmt::Debug for LazyArena {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LazyArena({:?})", self.id)
    }
}
