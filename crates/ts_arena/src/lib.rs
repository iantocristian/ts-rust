//! Checked, append-only arena identities and owner-retaining resolution scopes.

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;
use std::num::{NonZeroU32, NonZeroU64};
use std::ops::Deref;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};

pub const LAZY_PAGE_SIZE: usize = 256;

static GLOBAL_ARENA_IDS: ArenaIdAllocator = ArenaIdAllocator::new();
static LIVE_OWNERS: AtomicUsize = AtomicUsize::new(0);
static LIVE_ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

pub fn live_owner_count() -> usize {
    LIVE_OWNERS.load(Ordering::Acquire)
}

pub fn live_allocation_count() -> usize {
    LIVE_ALLOCATIONS.load(Ordering::Acquire)
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ArenaId(NonZeroU32);

impl ArenaId {
    pub fn get(self) -> u32 {
        self.0.get()
    }
}

#[derive(Debug)]
pub struct ArenaIdAllocator {
    next: AtomicU64,
}

impl ArenaIdAllocator {
    pub const fn new() -> Self {
        Self {
            next: AtomicU64::new(1),
        }
    }

    fn allocate(&self) -> Result<ArenaId, Exhausted> {
        let mut current = self.next.load(Ordering::Acquire);
        loop {
            if current == 0 || current > u64::from(u32::MAX) {
                return Err(Exhausted::ArenaIds);
            }
            match self.next.compare_exchange_weak(
                current,
                current + 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    let value = u32::try_from(current).map_err(|_| Exhausted::ArenaIds)?;
                    return Ok(ArenaId(NonZeroU32::new(value).ok_or(Exhausted::ArenaIds)?));
                }
                Err(observed) => current = observed,
            }
        }
    }

    #[cfg(feature = "test-support")]
    pub const fn with_next_for_test(next: u64) -> Self {
        Self {
            next: AtomicU64::new(next),
        }
    }
}

impl Default for ArenaIdAllocator {
    fn default() -> Self {
        Self::new()
    }
}

mod private {
    pub trait Sealed {}
}

pub trait TypedId:
    private::Sealed + Clone + Copy + fmt::Debug + Eq + Hash + Ord + Send + Sync + 'static
{
    #[doc(hidden)]
    fn from_parts(token: IdConstructionToken, arena: ArenaId, slot: NonZeroU32) -> Self;
    fn arena(self) -> ArenaId;
    fn slot(self) -> NonZeroU32;
    fn get(self) -> NonZeroU64;
}

#[doc(hidden)]
pub struct IdConstructionToken {
    private: bool,
}

macro_rules! typed_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
        #[repr(transparent)]
        pub struct $name(NonZeroU64);

        impl $name {
            pub fn arena(self) -> ArenaId {
                <Self as TypedId>::arena(self)
            }

            pub fn slot(self) -> NonZeroU32 {
                <Self as TypedId>::slot(self)
            }

            pub fn get(self) -> NonZeroU64 {
                self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($name))
                    .field("arena", &self.arena().get())
                    .field("slot", &self.slot().get())
                    .finish()
            }
        }

        impl private::Sealed for $name {}

        impl TypedId for $name {
            fn from_parts(token: IdConstructionToken, arena: ArenaId, slot: NonZeroU32) -> Self {
                debug_assert!(!token.private);
                let bits = (u64::from(arena.get()) << 32) | u64::from(slot.get());
                Self(NonZeroU64::new(bits).expect("a nonzero arena makes the id nonzero"))
            }

            fn arena(self) -> ArenaId {
                let arena = u32::try_from(self.0.get() >> 32).expect("high word fits u32");
                ArenaId(NonZeroU32::new(arena).expect("constructors reserve arena zero"))
            }

            fn slot(self) -> NonZeroU32 {
                let slot =
                    u32::try_from(self.0.get() & u64::from(u32::MAX)).expect("low word fits u32");
                NonZeroU32::new(slot).expect("constructors reserve slot zero")
            }

            fn get(self) -> NonZeroU64 {
                self.0
            }
        }
    };
}

typed_id!(NodeId);
typed_id!(SymbolId);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum OwnerKind {
    FileCore,
    FileLazy,
    Transform,
    Checker,
    Builder,
    Scratch,
    Symbol,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Exhausted {
    ArenaIds,
    Slots { arena: ArenaId },
}

impl fmt::Display for Exhausted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ArenaIds => write!(f, "arena id space exhausted before wrap or reuse"),
            Self::Slots { arena } => write!(
                f,
                "slot space for arena {} exhausted before wrap or reuse",
                arena.get()
            ),
        }
    }
}

impl std::error::Error for Exhausted {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolveError {
    WrongArena { expected: ArenaId, actual: ArenaId },
    UnknownArena(ArenaId),
    WrongOwnerKind { arena: ArenaId, kind: OwnerKind },
    Unpublished { arena: ArenaId, slot: u32 },
    Exhausted(Exhausted),
    Poisoned,
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongArena { expected, actual } => write!(
                f,
                "id belongs to arena {}, not arena {}",
                actual.get(),
                expected.get()
            ),
            Self::UnknownArena(arena) => {
                write!(f, "arena {} is not retained by this scope", arena.get())
            }
            Self::WrongOwnerKind { arena, kind } => {
                write!(
                    f,
                    "arena {} has disallowed owner kind {kind:?}",
                    arena.get()
                )
            }
            Self::Unpublished { arena, slot } => {
                write!(f, "slot {slot} is not published in arena {}", arena.get())
            }
            Self::Exhausted(error) => error.fmt(f),
            Self::Poisoned => write!(f, "arena lock is poisoned"),
        }
    }
}

impl std::error::Error for ResolveError {}

pub struct Allocation<T> {
    value: T,
}

impl<T> Allocation<T> {
    fn new(value: T) -> Self {
        LIVE_ALLOCATIONS.fetch_add(1, Ordering::AcqRel);
        Self { value }
    }
}

impl<T> Deref for Allocation<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: fmt::Debug> fmt::Debug for Allocation<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<T> Drop for Allocation<T> {
    fn drop(&mut self) {
        LIVE_ALLOCATIONS.fetch_sub(1, Ordering::AcqRel);
    }
}

pub struct ArenaRef<T>(Arc<Allocation<T>>);

impl<T> ArenaRef<T> {
    fn new(value: T) -> Self {
        Self(Arc::new(Allocation::new(value)))
    }
}

impl<T> Clone for ArenaRef<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T> Deref for ArenaRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0.value
    }
}

impl<T: fmt::Debug> fmt::Debug for ArenaRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.value.fmt(f)
    }
}

impl<T: PartialEq> PartialEq for ArenaRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.value == other.0.value
    }
}

impl<T: Eq> Eq for ArenaRef<T> {}

struct ArenaState<T> {
    first_slot: u64,
    next_slot: u64,
    slots: Vec<ArenaRef<T>>,
}

pub struct Arena<I: TypedId, T> {
    id: ArenaId,
    kind: OwnerKind,
    state: Mutex<ArenaState<T>>,
    marker: PhantomData<fn() -> I>,
}

impl<I: TypedId, T> Arena<I, T> {
    pub fn new(kind: OwnerKind) -> Result<Arc<Self>, Exhausted> {
        Self::new_in(&GLOBAL_ARENA_IDS, kind)
    }

    pub fn new_in(allocator: &ArenaIdAllocator, kind: OwnerKind) -> Result<Arc<Self>, Exhausted> {
        Self::new_with_first_slot(allocator, kind, 1)
    }

    fn new_with_first_slot(
        allocator: &ArenaIdAllocator,
        kind: OwnerKind,
        first_slot: u64,
    ) -> Result<Arc<Self>, Exhausted> {
        let id = allocator.allocate()?;
        LIVE_OWNERS.fetch_add(1, Ordering::AcqRel);
        Ok(Arc::new(Self {
            id,
            kind,
            state: Mutex::new(ArenaState {
                first_slot,
                next_slot: first_slot,
                slots: Vec::new(),
            }),
            marker: PhantomData,
        }))
    }

    #[cfg(feature = "test-support")]
    pub fn with_next_slot_for_test(
        allocator: &ArenaIdAllocator,
        kind: OwnerKind,
        next_slot: u64,
    ) -> Result<Arc<Self>, Exhausted> {
        Self::new_with_first_slot(allocator, kind, next_slot)
    }

    pub fn id(&self) -> ArenaId {
        self.id
    }

    pub fn kind(&self) -> OwnerKind {
        self.kind
    }

    pub fn allocate(&self, value: T) -> Result<I, Exhausted> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.next_slot == 0 || state.next_slot > u64::from(u32::MAX) {
            return Err(Exhausted::Slots { arena: self.id });
        }
        let slot =
            u32::try_from(state.next_slot).map_err(|_| Exhausted::Slots { arena: self.id })?;
        let slot = NonZeroU32::new(slot).ok_or(Exhausted::Slots { arena: self.id })?;
        state.next_slot += 1;
        state.slots.push(ArenaRef::new(value));
        Ok(I::from_parts(
            IdConstructionToken { private: false },
            self.id,
            slot,
        ))
    }

    pub fn resolve(&self, id: I) -> Result<ArenaRef<T>, ResolveError> {
        if id.arena() != self.id {
            return Err(ResolveError::WrongArena {
                expected: self.id,
                actual: id.arena(),
            });
        }
        self.resolve_slot(id.slot())
    }

    fn resolve_slot(&self, slot: NonZeroU32) -> Result<ArenaRef<T>, ResolveError> {
        let state = self.state.lock().map_err(|_| ResolveError::Poisoned)?;
        let slot = u64::from(slot.get());
        let Some(offset) = slot.checked_sub(state.first_slot) else {
            return Err(ResolveError::Unpublished {
                arena: self.id,
                slot: u32::try_from(slot).expect("slot originated as u32"),
            });
        };
        state
            .slots
            .get(
                usize::try_from(offset).map_err(|_| ResolveError::Unpublished {
                    arena: self.id,
                    slot: u32::try_from(slot).expect("slot originated as u32"),
                })?,
            )
            .cloned()
            .ok_or(ResolveError::Unpublished {
                arena: self.id,
                slot: u32::try_from(slot).expect("slot originated as u32"),
            })
    }

    pub fn published_len(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .slots
            .len()
    }

    pub fn with_scope<R>(
        &self,
        operation: impl for<'scope> FnOnce(ArenaScope<'scope, I, T>) -> R,
    ) -> R {
        operation(ArenaScope {
            arena: self,
            invariant: PhantomData,
        })
    }
}

impl<I: TypedId, T> Drop for Arena<I, T> {
    fn drop(&mut self) {
        LIVE_OWNERS.fetch_sub(1, Ordering::AcqRel);
    }
}

impl<I: TypedId, T> fmt::Debug for Arena<I, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Arena")
            .field("id", &self.id)
            .field("kind", &self.kind)
            .field("published_len", &self.published_len())
            .finish_non_exhaustive()
    }
}

pub type NodeArena<T> = Arena<NodeId, T>;
pub type SymbolArena<T> = Arena<SymbolId, T>;
type ScopeInvariant<'scope, I> = PhantomData<(&'scope mut &'scope (), I)>;

pub struct ArenaScope<'scope, I: TypedId, T> {
    arena: &'scope Arena<I, T>,
    invariant: ScopeInvariant<'scope, I>,
}

#[derive(Clone, Copy)]
pub struct LocalHandle<'scope, I: TypedId> {
    slot: NonZeroU32,
    invariant: ScopeInvariant<'scope, I>,
}

impl<'scope, I: TypedId, T> ArenaScope<'scope, I, T> {
    pub fn validate(&self, id: I) -> Result<LocalHandle<'scope, I>, ResolveError> {
        self.arena.resolve(id)?;
        Ok(LocalHandle {
            slot: id.slot(),
            invariant: PhantomData,
        })
    }

    pub fn resolve(&self, handle: LocalHandle<'scope, I>) -> Result<ArenaRef<T>, ResolveError> {
        self.arena.resolve_slot(handle.slot)
    }
}

pub struct ValidatedScope<I: TypedId, T> {
    arenas: BTreeMap<ArenaId, Arc<Arena<I, T>>>,
    allowed_kinds: Option<Vec<OwnerKind>>,
}

impl<I: TypedId, T> ValidatedScope<I, T> {
    pub fn new() -> Self {
        Self {
            arenas: BTreeMap::new(),
            allowed_kinds: None,
        }
    }

    pub fn allowing(kinds: impl IntoIterator<Item = OwnerKind>) -> Self {
        Self {
            arenas: BTreeMap::new(),
            allowed_kinds: Some(kinds.into_iter().collect()),
        }
    }

    pub fn retain(&mut self, arena: Arc<Arena<I, T>>) -> Result<(), ResolveError> {
        if self
            .allowed_kinds
            .as_ref()
            .is_some_and(|kinds| !kinds.contains(&arena.kind()))
        {
            return Err(ResolveError::WrongOwnerKind {
                arena: arena.id(),
                kind: arena.kind(),
            });
        }
        self.arenas.insert(arena.id(), arena);
        Ok(())
    }

    pub fn import(&self, id: I) -> Result<ArenaRef<T>, ResolveError> {
        self.arenas
            .get(&id.arena())
            .ok_or(ResolveError::UnknownArena(id.arena()))?
            .resolve(id)
    }

    pub fn contains(&self, arena: ArenaId) -> bool {
        self.arenas.contains_key(&arena)
    }
}

impl<I: TypedId, T> Default for ValidatedScope<I, T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ArenaLease<I: TypedId, T> {
    arena: Arc<Arena<I, T>>,
}

impl<I: TypedId, T> ArenaLease<I, T> {
    pub fn new(arena: Arc<Arena<I, T>>) -> Self {
        Self { arena }
    }

    pub fn arena_id(&self) -> ArenaId {
        self.arena.id()
    }

    pub fn resolve(&self, id: I) -> Result<ArenaRef<T>, ResolveError> {
        self.arena.resolve(id)
    }
}

impl<I: TypedId, T> Clone for ArenaLease<I, T> {
    fn clone(&self) -> Self {
        Self {
            arena: Arc::clone(&self.arena),
        }
    }
}

type LazyPage<T> = Box<[Option<ArenaRef<T>>; LAZY_PAGE_SIZE]>;

struct LazyState<K, T> {
    cache: HashMap<K, NodeId>,
    pages: Vec<LazyPage<T>>,
    first_slot: u64,
    next_slot: u64,
}

pub struct LazyArena<K, T> {
    id: ArenaId,
    state: RwLock<LazyState<K, T>>,
}

impl<K, T> LazyArena<K, T>
where
    K: Clone + Eq + Hash,
{
    pub fn new() -> Result<Arc<Self>, Exhausted> {
        Self::new_in(&GLOBAL_ARENA_IDS)
    }

    pub fn new_in(allocator: &ArenaIdAllocator) -> Result<Arc<Self>, Exhausted> {
        Self::new_with_first_slot(allocator, 1)
    }

    fn new_with_first_slot(
        allocator: &ArenaIdAllocator,
        first_slot: u64,
    ) -> Result<Arc<Self>, Exhausted> {
        let id = allocator.allocate()?;
        LIVE_OWNERS.fetch_add(1, Ordering::AcqRel);
        Ok(Arc::new(Self {
            id,
            state: RwLock::new(LazyState {
                cache: HashMap::new(),
                pages: Vec::new(),
                first_slot,
                next_slot: first_slot,
            }),
        }))
    }

    #[cfg(feature = "test-support")]
    pub fn with_next_slot_for_test(
        allocator: &ArenaIdAllocator,
        next_slot: u64,
    ) -> Result<Arc<Self>, Exhausted> {
        Self::new_with_first_slot(allocator, next_slot)
    }

    pub fn id(&self) -> ArenaId {
        self.id
    }

    pub fn get_or_insert_with(
        &self,
        key: K,
        create: impl FnOnce() -> T,
    ) -> Result<(NodeId, ArenaRef<T>, bool), ResolveError> {
        {
            let state = self.state.read().map_err(|_| ResolveError::Poisoned)?;
            if let Some(id) = state.cache.get(&key).copied() {
                let allocation = resolve_lazy_slot(self.id, &state, id.slot())?;
                return Ok((id, allocation, false));
            }
        }

        let mut state = self.state.write().map_err(|_| ResolveError::Poisoned)?;
        if let Some(id) = state.cache.get(&key).copied() {
            return Ok((id, resolve_lazy_slot(self.id, &state, id.slot())?, false));
        }
        if state.next_slot == 0 || state.next_slot > u64::from(u32::MAX) {
            return Err(ResolveError::Exhausted(Exhausted::Slots { arena: self.id }));
        }
        let slot = u32::try_from(state.next_slot)
            .map_err(|_| ResolveError::Exhausted(Exhausted::Slots { arena: self.id }))?;
        let slot = NonZeroU32::new(slot).ok_or(ResolveError::Unpublished {
            arena: self.id,
            slot,
        })?;
        let zero_based = usize::try_from(state.next_slot - state.first_slot).map_err(|_| {
            ResolveError::Unpublished {
                arena: self.id,
                slot: slot.get(),
            }
        })?;
        let page_index = zero_based / LAZY_PAGE_SIZE;
        let page_slot = zero_based % LAZY_PAGE_SIZE;
        while state.pages.len() <= page_index {
            state.pages.push(Box::new(std::array::from_fn(|_| None)));
        }
        let allocation = ArenaRef::new(create());
        state.pages[page_index][page_slot] = Some(allocation.clone());
        state.next_slot += 1;
        let id = NodeId::from_parts(IdConstructionToken { private: false }, self.id, slot);
        state.cache.insert(key, id);
        Ok((id, allocation, true))
    }

    pub fn resolve(&self, id: NodeId) -> Result<ArenaRef<T>, ResolveError> {
        if id.arena() != self.id {
            return Err(ResolveError::WrongArena {
                expected: self.id,
                actual: id.arena(),
            });
        }
        let state = self.state.read().map_err(|_| ResolveError::Poisoned)?;
        resolve_lazy_slot(self.id, &state, id.slot())
    }

    pub fn published_len(&self) -> usize {
        self.state
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .cache
            .len()
    }

    pub fn page_count(&self) -> usize {
        self.state
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .pages
            .len()
    }
}

fn resolve_lazy_slot<K, T>(
    arena: ArenaId,
    state: &LazyState<K, T>,
    slot: NonZeroU32,
) -> Result<ArenaRef<T>, ResolveError> {
    let Some(offset) = u64::from(slot.get()).checked_sub(state.first_slot) else {
        return Err(ResolveError::Unpublished {
            arena,
            slot: slot.get(),
        });
    };
    let zero_based = usize::try_from(offset).map_err(|_| ResolveError::Unpublished {
        arena,
        slot: slot.get(),
    })?;
    state
        .pages
        .get(zero_based / LAZY_PAGE_SIZE)
        .and_then(|page| page[zero_based % LAZY_PAGE_SIZE].as_ref())
        .cloned()
        .ok_or(ResolveError::Unpublished {
            arena,
            slot: slot.get(),
        })
}

impl<K, T> Drop for LazyArena<K, T> {
    fn drop(&mut self) {
        LIVE_OWNERS.fetch_sub(1, Ordering::AcqRel);
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FileId(ArenaId);

impl FileId {
    pub fn get(self) -> u32 {
        self.0.get()
    }
}

pub struct FileOwner<C, K, L>
where
    K: Clone + Eq + Hash,
{
    id: FileId,
    core: Arc<NodeArena<C>>,
    lazy: Arc<LazyArena<K, L>>,
}

impl<C, K, L> FileOwner<C, K, L>
where
    K: Clone + Eq + Hash,
{
    pub fn new() -> Result<Arc<Self>, Exhausted> {
        let core = NodeArena::new(OwnerKind::FileCore)?;
        let lazy = LazyArena::new()?;
        LIVE_OWNERS.fetch_add(1, Ordering::AcqRel);
        Ok(Arc::new(Self {
            id: FileId(core.id()),
            core,
            lazy,
        }))
    }

    pub fn id(&self) -> FileId {
        self.id
    }

    pub fn core(&self) -> &Arc<NodeArena<C>> {
        &self.core
    }

    pub fn lazy(&self) -> &Arc<LazyArena<K, L>> {
        &self.lazy
    }
}

impl<C, K, L> Drop for FileOwner<C, K, L>
where
    K: Clone + Eq + Hash,
{
    fn drop(&mut self) {
        LIVE_OWNERS.fetch_sub(1, Ordering::AcqRel);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BundleLink {
    Canonical { supplementals: Vec<FileId> },
    Supplemental { canonical: FileId },
}

struct BundleMember<C, K, L>
where
    K: Clone + Eq + Hash,
{
    file: Arc<FileOwner<C, K, L>>,
    link: BundleLink,
}

pub struct BundleOwner<C, K, L>
where
    K: Clone + Eq + Hash,
{
    members: BTreeMap<FileId, BundleMember<C, K, L>>,
}

impl<C, K, L> BundleOwner<C, K, L>
where
    K: Clone + Eq + Hash,
{
    pub fn new(
        canonical: Arc<FileOwner<C, K, L>>,
        supplementals: Vec<Arc<FileOwner<C, K, L>>>,
    ) -> Arc<Self> {
        let canonical_id = canonical.id();
        let supplemental_ids = supplementals.iter().map(|file| file.id()).collect();
        let mut members = BTreeMap::new();
        members.insert(
            canonical_id,
            BundleMember {
                file: canonical,
                link: BundleLink::Canonical {
                    supplementals: supplemental_ids,
                },
            },
        );
        for file in supplementals {
            members.insert(
                file.id(),
                BundleMember {
                    file,
                    link: BundleLink::Supplemental {
                        canonical: canonical_id,
                    },
                },
            );
        }
        LIVE_OWNERS.fetch_add(1, Ordering::AcqRel);
        Arc::new(Self { members })
    }

    pub fn link(&self, file: FileId) -> Option<&BundleLink> {
        self.members.get(&file).map(|member| &member.link)
    }

    pub fn file(&self, file: FileId) -> Option<Arc<FileOwner<C, K, L>>> {
        self.members
            .get(&file)
            .map(|member| Arc::clone(&member.file))
    }

    pub fn handle(self: &Arc<Self>, file: FileId) -> Option<BundleFileHandle<C, K, L>> {
        self.members.contains_key(&file).then(|| BundleFileHandle {
            bundle: Arc::clone(self),
            file,
        })
    }

    pub fn len(&self) -> usize {
        self.members.len()
    }

    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }
}

impl<C, K, L> Drop for BundleOwner<C, K, L>
where
    K: Clone + Eq + Hash,
{
    fn drop(&mut self) {
        LIVE_OWNERS.fetch_sub(1, Ordering::AcqRel);
    }
}

pub struct BundleFileHandle<C, K, L>
where
    K: Clone + Eq + Hash,
{
    bundle: Arc<BundleOwner<C, K, L>>,
    file: FileId,
}

impl<C, K, L> BundleFileHandle<C, K, L>
where
    K: Clone + Eq + Hash,
{
    pub fn file_id(&self) -> FileId {
        self.file
    }

    pub fn owner(&self) -> Arc<FileOwner<C, K, L>> {
        self.bundle
            .file(self.file)
            .expect("bundle handles are constructed only for members")
    }

    pub fn sibling(&self, file: FileId) -> Option<Self> {
        self.bundle.handle(file)
    }
}

impl<C, K, L> Clone for BundleFileHandle<C, K, L>
where
    K: Clone + Eq + Hash,
{
    fn clone(&self) -> Self {
        Self {
            bundle: Arc::clone(&self.bundle),
            file: self.file,
        }
    }
}

pub struct ScratchOwner<T> {
    arena: Arc<NodeArena<T>>,
}

impl<T> ScratchOwner<T> {
    pub fn new() -> Result<Self, Exhausted> {
        LIVE_OWNERS.fetch_add(1, Ordering::AcqRel);
        match NodeArena::new(OwnerKind::Scratch) {
            Ok(arena) => Ok(Self { arena }),
            Err(error) => {
                LIVE_OWNERS.fetch_sub(1, Ordering::AcqRel);
                Err(error)
            }
        }
    }

    pub fn arena(&self) -> &Arc<NodeArena<T>> {
        &self.arena
    }
}

impl<T> Drop for ScratchOwner<T> {
    fn drop(&mut self) {
        LIVE_OWNERS.fetch_sub(1, Ordering::AcqRel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_nonzero_and_wrong_owners_are_rejected() {
        let first = NodeArena::new(OwnerKind::FileCore).unwrap();
        let second = NodeArena::new(OwnerKind::FileCore).unwrap();
        let first_id = first.allocate("first").unwrap();
        let second_id = second.allocate("second").unwrap();
        assert_ne!(first_id.arena(), second_id.arena());
        assert_eq!(*first.resolve(first_id).unwrap(), "first");
        assert!(matches!(
            first.resolve(second_id),
            Err(ResolveError::WrongArena { .. })
        ));
    }

    #[test]
    fn injected_counters_fail_before_wrap() {
        let allocator = ArenaIdAllocator::with_next_for_test(u64::from(u32::MAX));
        let last = NodeArena::<()>::new_in(&allocator, OwnerKind::FileCore).unwrap();
        assert_eq!(last.id().get(), u32::MAX);
        assert_eq!(
            NodeArena::<()>::new_in(&allocator, OwnerKind::FileCore).unwrap_err(),
            Exhausted::ArenaIds
        );

        let slots = ArenaIdAllocator::with_next_for_test(1);
        let arena =
            NodeArena::with_next_slot_for_test(&slots, OwnerKind::FileCore, u64::from(u32::MAX))
                .unwrap();
        let id = arena.allocate(42).unwrap();
        assert_eq!(id.slot().get(), u32::MAX);
        assert_eq!(*arena.resolve(id).unwrap(), 42);
        assert_eq!(
            arena.allocate(43),
            Err(Exhausted::Slots { arena: arena.id() })
        );
    }

    #[test]
    fn leases_retain_storage_but_ids_do_not() {
        let owner = NodeArena::new(OwnerKind::Scratch).unwrap();
        let id = owner.allocate(7).unwrap();
        let lease = ArenaLease::new(Arc::clone(&owner));
        drop(owner);
        assert_eq!(*lease.resolve(id).unwrap(), 7);
        drop(lease);
        let empty = ValidatedScope::<NodeId, i32>::new();
        assert!(matches!(
            empty.import(id),
            Err(ResolveError::UnknownArena(_))
        ));
    }
}
