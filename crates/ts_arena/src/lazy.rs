use crate::{
    arena::Arena,
    counters::Track,
    ids::{allocate_slot, next_arena},
    ArenaId, AuxId, Counters, Error, NodeId, NodeRecord,
};
use std::{
    collections::BTreeMap,
    panic::{catch_unwind, resume_unwind, AssertUnwindSafe},
    sync::{Arc, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

const PAGE_SIZE: usize = 256;
fn assert_not_initializing(id: ArenaId) {
    crate::InitializationGuard::assert_inactive(id, crate::InitializationDomain::Lazy, 0);
}
struct InitializerGuard {
    _guard: crate::InitializationGuard,
}
impl InitializerGuard {
    fn enter(id: ArenaId) -> Self {
        Self {
            _guard: crate::InitializationGuard::enter(id, crate::InitializationDomain::Lazy, 0),
        }
    }
}

struct Page<T> {
    slots: [OnceLock<T>; PAGE_SIZE],
    _allocation: Track,
}
/// Kept private: an exported read also borrows or retains its complete owner.
pub(crate) struct LazyRecord<T> {
    page: Arc<Page<T>>,
    offset: usize,
}
impl<T> Clone for LazyRecord<T> {
    fn clone(&self) -> Self {
        Self {
            page: self.page.clone(),
            offset: self.offset,
        }
    }
}
impl<T> LazyRecord<T> {
    pub(crate) fn get(&self) -> &T {
        self.page.slots[self.offset]
            .get()
            .expect("published lazy record")
    }
}
struct Pages<T> {
    id: ArenaId,
    reserved: usize,
    directory: Vec<Arc<Page<T>>>,
    counters: Counters,
}
impl<T> Pages<T> {
    fn new(counters: &Counters) -> Self {
        Self {
            id: next_arena(),
            reserved: 0,
            directory: Vec::new(),
            counters: counters.clone(),
        }
    }
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
    fn initialize(&self, slot: u32, value: T) {
        let index = slot as usize - 1;
        // Initialize only the unused slot; never mutably borrow a published page.
        assert!(
            self.directory[index / PAGE_SIZE].slots[index % PAGE_SIZE]
                .set(value)
                .is_ok(),
            "lazy slots are initialized once"
        );
    }
    fn get_borrowed(&self, slot: u32) -> Result<&T, Error> {
        let index = slot.checked_sub(1).ok_or(Error::InvalidSlot)? as usize;
        if index >= self.reserved {
            return Err(Error::InvalidSlot);
        }
        self.directory[index / PAGE_SIZE].slots[index % PAGE_SIZE]
            .get()
            .ok_or(Error::InvalidSlot)
    }
    fn get(&self, slot: u32) -> Result<LazyRecord<T>, Error> {
        self.get_borrowed(slot)?;
        let index = slot as usize - 1;
        Ok(LazyRecord {
            page: self.directory[index / PAGE_SIZE].clone(),
            offset: index % PAGE_SIZE,
        })
    }
}
struct Staging<'a, T> {
    pages: &'a mut Pages<T>,
    base: usize,
    pending: Vec<T>,
}
impl<'a, T> Staging<'a, T> {
    fn new(pages: &'a mut Pages<T>) -> Self {
        Self {
            base: pages.reserved,
            pages,
            pending: Vec::new(),
        }
    }
    fn push(&mut self, value: T) -> u32 {
        let slot = self.pages.reserve();
        self.pending.push(value);
        slot
    }
    fn index(&self, slot: u32) -> Result<usize, Error> {
        (slot as usize)
            .checked_sub(self.base + 1)
            .filter(|&i| i < self.pending.len())
            .ok_or(Error::InvalidSlot)
    }
    fn get(&self, slot: u32) -> Result<&T, Error> {
        if slot as usize <= self.base {
            self.pages.get_borrowed(slot)
        } else {
            self.pending
                .get(self.index(slot)?)
                .ok_or(Error::InvalidSlot)
        }
    }
    fn get_mut(&mut self, slot: u32) -> Result<&mut T, Error> {
        let index = self.index(slot)?;
        self.pending.get_mut(index).ok_or(Error::InvalidSlot)
    }
    fn publish(self) {
        for (index, value) in self.pending.into_iter().enumerate() {
            self.pages.initialize((self.base + index + 1) as u32, value);
        }
    }
}

/// Node and auxiliary staging share the cache's single publication lock. Reads
/// resolve core, already published and staged records without taking that lock again.
/// Error and unwind burn both sets of identities and discard all pending values.
pub struct StorageTransaction<'a, N: NodeRecord> {
    nodes: Staging<'a, N>,
    auxiliary: Staging<'a, N::Aux>,
    core: &'a Arena<N>,
    core_auxiliary: &'a Arena<N::Aux>,
}
impl<N: NodeRecord> StorageTransaction<'_, N> {
    pub fn staged_nodes(&self) -> impl Iterator<Item = &N> {
        self.nodes.pending.iter()
    }
    pub fn staged_aux(&self) -> impl Iterator<Item = &N::Aux> {
        self.auxiliary.pending.iter()
    }
    pub fn push(&mut self, node: N) -> NodeId {
        NodeId::new(self.nodes.pages.id, self.nodes.push(node))
    }
    pub fn push_aux(&mut self, value: N::Aux) -> AuxId {
        AuxId::new(self.auxiliary.pages.id, self.auxiliary.push(value))
    }
    pub fn node(&self, id: NodeId) -> Result<&N, Error> {
        if id.arena() == self.core.id {
            return self.core.get_slot(id.slot());
        }
        if id.arena() != self.nodes.pages.id {
            return Err(Error::WrongOwner);
        }
        self.nodes.get(id.slot())
    }
    pub fn aux(&self, id: AuxId) -> Result<&N::Aux, Error> {
        if id.arena() == self.core_auxiliary.id {
            return self.core_auxiliary.get_slot(id.slot());
        }
        if id.arena() != self.auxiliary.pages.id {
            return Err(Error::WrongOwner);
        }
        self.auxiliary.get(id.slot())
    }
    pub fn node_mut(&mut self, id: NodeId) -> Result<&mut N, Error> {
        if id.arena() != self.nodes.pages.id {
            return Err(Error::WrongOwner);
        }
        self.nodes.get_mut(id.slot())
    }
    pub fn aux_mut(&mut self, id: AuxId) -> Result<&mut N::Aux, Error> {
        if id.arena() != self.auxiliary.pages.id {
            return Err(Error::WrongOwner);
        }
        self.auxiliary.get_mut(id.slot())
    }
    fn publish(self, roots: &[NodeId]) -> Result<(), Error> {
        for id in roots {
            if id.arena() != self.nodes.pages.id || self.nodes.index(id.slot()).is_err() {
                return Err(Error::InvalidGraph);
            }
        }
        // No user callbacks or fallible operations after this publication point.
        self.auxiliary.publish();
        self.nodes.publish();
        Ok(())
    }
}

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
struct LazyState<N: NodeRecord> {
    pages: Pages<N>,
    auxiliary: Pages<N::Aux>,
    // None is the S04 file cache. Concrete ASTs can distinguish several logical
    // source files housed in the same storage owner without adding another lock.
    jsdoc: BTreeMap<(Option<NodeId>, NodeId), Arc<[NodeId]>>,
    tokens: BTreeMap<TokenKey, CachedToken>,
}
/// Node/auxiliary directories, reservations, publication and caches use one lock.
pub(crate) struct LazyArena<N: NodeRecord> {
    id: ArenaId,
    auxiliary_id: ArenaId,
    state: RwLock<LazyState<N>>,
}
impl<N: NodeRecord> LazyArena<N> {
    pub(crate) fn new(counters: &Counters) -> Self {
        let pages = Pages::new(counters);
        let auxiliary = Pages::new(counters);
        Self {
            id: pages.id,
            auxiliary_id: auxiliary.id,
            state: RwLock::new(LazyState {
                pages,
                auxiliary,
                jsdoc: BTreeMap::new(),
                tokens: BTreeMap::new(),
            }),
        }
    }
    pub(crate) fn id(&self) -> ArenaId {
        self.id
    }
    pub(crate) fn auxiliary_id(&self) -> ArenaId {
        self.auxiliary_id
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
    pub(crate) fn node(&self, id: NodeId) -> Result<LazyRecord<N>, Error> {
        if id.arena() != self.id {
            return Err(Error::WrongOwner);
        }
        self.read().pages.get(id.slot())
    }
    pub(crate) fn aux(&self, id: AuxId) -> Result<LazyRecord<N::Aux>, Error> {
        if id.arena() != self.auxiliary_id {
            return Err(Error::WrongOwner);
        }
        self.read().auxiliary.get(id.slot())
    }
    pub(crate) fn eager_jsdoc(
        &self,
        source: Option<NodeId>,
        parent: NodeId,
    ) -> Option<Arc<[NodeId]>> {
        self.read().jsdoc.get(&(source, parent)).cloned()
    }
    pub(crate) fn seed_jsdoc(
        &mut self,
        source: Option<NodeId>,
        parent: NodeId,
        roots: Arc<[NodeId]>,
    ) -> Result<(), Error> {
        let state = self
            .state
            .get_mut()
            .expect("lazy publication lock is not poisoned");
        if state.jsdoc.contains_key(&(source, parent)) {
            return Err(Error::InvalidGraph);
        }
        state.jsdoc.insert((source, parent), roots);
        Ok(())
    }
    pub(crate) fn jsdoc(
        &self,
        source: Option<NodeId>,
        parent: NodeId,
        core: &Arena<N>,
        core_auxiliary: &Arena<N::Aux>,
        initialize: impl FnOnce(&mut StorageTransaction<'_, N>) -> Result<Vec<NodeId>, Error>,
    ) -> Result<Arc<[NodeId]>, Error> {
        if let Some(ids) = self.read().jsdoc.get(&(source, parent)) {
            return Ok(ids.clone());
        }
        let mut state = self.write();
        if let Some(ids) = state.jsdoc.get(&(source, parent)) {
            return Ok(ids.clone());
        }
        let result = catch_unwind(AssertUnwindSafe(|| {
            let _initializer = InitializerGuard::enter(self.id);
            let LazyState {
                pages, auxiliary, ..
            } = &mut *state;
            let mut transaction = StorageTransaction {
                nodes: Staging::new(pages),
                auxiliary: Staging::new(auxiliary),
                core,
                core_auxiliary,
            };
            let roots = initialize(&mut transaction)?;
            transaction.publish(&roots)?;
            let roots: Arc<[NodeId]> = roots.into();
            state.jsdoc.insert((source, parent), roots.clone());
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
            let mut node = initialize();
            if node.storage_kind() != kind {
                return Err(Error::TokenKindMismatch {
                    cached: node.storage_kind(),
                    requested: kind,
                });
            }
            node.set_storage_parent(Some(key.parent));
            let slot = state.pages.reserve();
            state.pages.initialize(slot, node);
            let id = NodeId::new(self.id, slot);
            state.tokens.insert(key, CachedToken { id, kind });
            Ok(id)
        }));
        drop(state);
        match result {
            Ok(result) => result,
            Err(panic) => resume_unwind(panic),
        }
    }
}
