use crate::{
    counters::Track,
    ids::{allocate_slot, next_arena},
    ArenaId, Counters, Error, Node, NodeId,
};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    panic::{catch_unwind, resume_unwind, AssertUnwindSafe},
    sync::{Arc, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

const PAGE_SIZE: usize = 256;

thread_local! {
    // File identities are process-unique, including across different payload types.
    static ACTIVE_INITIALIZERS: RefCell<Vec<ArenaId>> = const { RefCell::new(Vec::new()) };
}

fn assert_not_initializing(id: ArenaId) {
    ACTIVE_INITIALIZERS.with(|active| {
        assert!(
            !active.borrow().contains(&id),
            "ts_arena: lazy initializer reentered its file's lazy storage"
        );
    });
}

/// Cleared before the surrounding catch boundary unlocks or resumes a panic.
struct InitializerGuard(ArenaId);

impl InitializerGuard {
    fn enter(id: ArenaId) -> Self {
        assert_not_initializing(id);
        ACTIVE_INITIALIZERS.with(|active| active.borrow_mut().push(id));
        Self(id)
    }
}

impl Drop for InitializerGuard {
    fn drop(&mut self) {
        ACTIVE_INITIALIZERS.with(|active| {
            let popped = active.borrow_mut().pop();
            debug_assert_eq!(popped, Some(self.0));
        });
    }
}

struct Page<N> {
    slots: [OnceLock<Node<N>>; PAGE_SIZE],
    _allocation: Track,
}

/// Page ownership alone stays private; exported references also retain or borrow a file.
pub(crate) struct LazyNode<N> {
    page: Arc<Page<N>>,
    offset: usize,
}

impl<N> Clone for LazyNode<N> {
    fn clone(&self) -> Self {
        Self {
            page: self.page.clone(),
            offset: self.offset,
        }
    }
}

impl<N> LazyNode<N> {
    pub(crate) fn get(&self) -> &Node<N> {
        self.page.slots[self.offset]
            .get()
            .expect("published lazy node")
    }
}

struct Pages<N> {
    id: ArenaId,
    reserved: usize,
    directory: Vec<Arc<Page<N>>>,
    counters: Counters,
}

impl<N> Pages<N> {
    fn reserve(&mut self) -> u32 {
        let slot = allocate_slot(self.reserved);
        if self.reserved / PAGE_SIZE == self.directory.len() {
            self.directory.push(Arc::new(Page {
                slots: std::array::from_fn(|_| OnceLock::new()),
                _allocation: self.counters.allocation(),
            }));
        }
        self.reserved += 1;
        slot
    }

    fn initialize(&self, slot: u32, node: Node<N>) {
        let index = slot as usize - 1;
        // Only the unused slot is initialized: published readers never overlap
        // a mutable borrow of either their node or its entire page.
        assert!(
            self.directory[index / PAGE_SIZE].slots[index % PAGE_SIZE]
                .set(node)
                .is_ok(),
            "lazy slots are initialized once"
        );
    }

    fn get(&self, slot: u32) -> Result<LazyNode<N>, Error> {
        let index = slot.checked_sub(1).ok_or(Error::InvalidSlot)? as usize;
        if index >= self.reserved {
            return Err(Error::InvalidSlot);
        }
        let page = &self.directory[index / PAGE_SIZE];
        if page.slots[index % PAGE_SIZE].get().is_none() {
            return Err(Error::InvalidSlot);
        }
        Ok(LazyNode {
            page: page.clone(),
            offset: index % PAGE_SIZE,
        })
    }
}

/// Private staging for one graph. Minted ids are burned even on error or unwind.
/// Metadata/payload edges are ids; callers construct their graph through those ids.
pub struct LazyTransaction<'a, N> {
    pages: &'a mut Pages<N>,
    base: usize,
    pending: Vec<Node<N>>,
}

impl<N> LazyTransaction<'_, N> {
    pub fn push(&mut self, node: Node<N>) -> NodeId {
        let slot = self.pages.reserve();
        self.pending.push(node);
        NodeId::new(self.pages.id, slot)
    }

    pub fn node_mut(&mut self, id: NodeId) -> Result<&mut Node<N>, Error> {
        if id.arena() != self.pages.id {
            return Err(Error::WrongOwner);
        }
        let index = (id.slot() as usize)
            .checked_sub(self.base + 1)
            .ok_or(Error::InvalidSlot)?;
        self.pending.get_mut(index).ok_or(Error::InvalidSlot)
    }

    fn publish(self, roots: &[NodeId]) -> Result<(), Error> {
        if roots.iter().any(|id| {
            id.arena() != self.pages.id
                || id.slot() as usize <= self.base
                || id.slot() as usize > self.base + self.pending.len()
        }) {
            return Err(Error::InvalidGraph);
        }
        for (index, node) in self.pending.into_iter().enumerate() {
            self.pages.initialize((self.base + index + 1) as u32, node);
        }
        Ok(())
    }
}

/// The supplied parent and source range identify a token; kind is checked separately.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TokenKey {
    pub parent: NodeId,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Copy)]
struct CachedToken {
    id: NodeId,
    kind: u32,
}

struct LazyState<N> {
    pages: Pages<N>,
    jsdoc: BTreeMap<NodeId, Arc<[NodeId]>>,
    tokens: BTreeMap<TokenKey, CachedToken>,
}

/// Directory, reservations, publication and both caches share exactly one lock.
pub(crate) struct LazyArena<N> {
    id: ArenaId,
    state: RwLock<LazyState<N>>,
}

impl<N> LazyArena<N> {
    pub(crate) fn new(counters: &Counters) -> Self {
        let id = next_arena();
        Self {
            id,
            state: RwLock::new(LazyState {
                pages: Pages {
                    id,
                    reserved: 0,
                    directory: Vec::new(),
                    counters: counters.clone(),
                },
                jsdoc: BTreeMap::new(),
                tokens: BTreeMap::new(),
            }),
        }
    }

    pub(crate) fn id(&self) -> ArenaId {
        self.id
    }

    fn read(&self) -> RwLockReadGuard<'_, LazyState<N>> {
        assert_not_initializing(self.id);
        self.state
            .read()
            .expect("lazy publication lock is not poisoned")
    }

    fn write(&self) -> RwLockWriteGuard<'_, LazyState<N>> {
        assert_not_initializing(self.id);
        self.state
            .write()
            .expect("lazy publication lock is not poisoned")
    }

    pub(crate) fn node(&self, id: NodeId) -> Result<LazyNode<N>, Error> {
        if id.arena() != self.id {
            return Err(Error::WrongOwner);
        }
        self.read().pages.get(id.slot())
    }

    pub(crate) fn jsdoc(
        &self,
        parent: NodeId,
        initialize: impl FnOnce(&mut LazyTransaction<'_, N>) -> Result<Vec<NodeId>, Error>,
    ) -> Result<Arc<[NodeId]>, Error> {
        if let Some(ids) = self.read().jsdoc.get(&parent) {
            return Ok(ids.clone());
        }
        let mut state = self.write();
        if let Some(ids) = state.jsdoc.get(&parent) {
            return Ok(ids.clone());
        }
        // Catch before dropping the guard so a caller's panic cannot poison
        // valid existing storage. The transaction drops pending payloads and
        // preserves reserved tombstones; the panic resumes only after unlock.
        let result = catch_unwind(AssertUnwindSafe(|| {
            let _initializer = InitializerGuard::enter(self.id);
            let mut transaction = LazyTransaction {
                base: state.pages.reserved,
                pages: &mut state.pages,
                pending: Vec::new(),
            };
            let roots = initialize(&mut transaction)?;
            transaction.publish(&roots)?;
            let roots: Arc<[NodeId]> = roots.into();
            state.jsdoc.insert(parent, roots.clone());
            Ok(roots)
        }));
        drop(state);
        match result {
            Ok(result) => result,
            Err(panic) => resume_unwind(panic),
        }
    }

    fn cached_token(
        state: &LazyState<N>,
        key: TokenKey,
        kind: u32,
    ) -> Result<Option<NodeId>, Error> {
        state
            .tokens
            .get(&key)
            .map(|cached| {
                if cached.kind == kind {
                    Ok(cached.id)
                } else {
                    Err(Error::TokenKindMismatch {
                        cached: cached.kind,
                        requested: kind,
                    })
                }
            })
            .transpose()
    }

    pub(crate) fn token(
        &self,
        key: TokenKey,
        kind: u32,
        reparsed: bool,
        source_len: usize,
        initialize: impl FnOnce() -> N,
    ) -> Result<NodeId, Error> {
        if let Some(id) = Self::cached_token(&self.read(), key, kind)? {
            return Ok(id);
        }
        let mut state = self.write();
        if let Some(id) = Self::cached_token(&state, key, kind)? {
            return Ok(id);
        }
        if reparsed {
            return Err(Error::ReparsedParent);
        }
        if key.start > key.end || key.end > source_len {
            return Err(Error::InvalidTokenRange);
        }
        let result = catch_unwind(AssertUnwindSafe(|| {
            let _initializer = InitializerGuard::enter(self.id);
            let mut node = Node::new(kind, initialize());
            node.parent = Some(key.parent);
            let slot = state.pages.reserve();
            state.pages.initialize(slot, node);
            let id = NodeId::new(self.id, slot);
            state.tokens.insert(key, CachedToken { id, kind });
            id
        }));
        drop(state);
        match result {
            Ok(id) => Ok(id),
            Err(panic) => resume_unwind(panic),
        }
    }
}
