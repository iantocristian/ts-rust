use crate::{
    counters::Track,
    storage::{Page, Pages, Slab},
    ArenaId, Counters, Error, FileId, Node, NodeId, SymbolId,
};
use parking_lot::RwLock;
use std::{collections::BTreeMap, ops::Deref, sync::Arc};

/// Parser/binder construction state. Moving it into a file or bundle publishes
/// the immutable core and symbols; no mutable core accessor exists afterward.
pub struct FileBuilder<N, S = ()> {
    core: Slab<Node<N>>,
    symbols: Slab<S>,
    source: Arc<[u8]>,
    counters: Counters,
}
impl<N, S> FileBuilder<N, S> {
    pub fn new(source: Arc<[u8]>, counters: &Counters) -> Self {
        Self {
            core: Slab::new(counters),
            symbols: Slab::new(counters),
            source,
            counters: counters.clone(),
        }
    }
    pub fn id(&self) -> FileId {
        FileId(self.core.id)
    }
    pub fn push(&mut self, node: Node<N>) -> NodeId {
        NodeId::new(self.core.id, self.core.push(node))
    }
    pub fn push_symbol(&mut self, symbol: S) -> SymbolId {
        SymbolId::new(self.symbols.id, self.symbols.push(symbol))
    }
    pub fn node_mut(&mut self, id: NodeId) -> Result<&mut Node<N>, Error> {
        if id.arena() != self.core.id {
            return Err(Error::WrongOwner);
        }
        self.core
            .values
            .get_mut(id.slot() as usize - 1)
            .ok_or(Error::InvalidSlot)
    }
    pub fn finish(self) -> FileHandle<N, S> {
        FileHandle {
            root: Root::File(Arc::new(self.publish(None, Vec::new()))),
        }
    }
    fn publish(self, canonical: Option<FileId>, supplemental: Vec<FileId>) -> FileOwner<N, S> {
        let pages = Pages::new(&self.counters);
        FileOwner {
            lazy_id: pages.id,
            core: self.core,
            symbols: self.symbols,
            source: self.source,
            canonical,
            supplemental,
            lazy: RwLock::new(LazyState {
                pages,
                jsdoc: BTreeMap::new(),
                tokens: BTreeMap::new(),
            }),
            _owner: self.counters.owner(),
        }
    }
}

/// File storage is accessed through a FileHandle, which also retains any mapped
/// siblings. Node payloads and symbol links must use non-owning ids.
pub struct FileOwner<N, S = ()> {
    core: Slab<Node<N>>,
    symbols: Slab<S>,
    source: Arc<[u8]>,
    canonical: Option<FileId>,
    supplemental: Vec<FileId>,
    lazy_id: ArenaId,
    lazy: RwLock<LazyState<N>>,
    _owner: Track,
}
impl<N, S> FileOwner<N, S> {
    pub fn id(&self) -> FileId {
        FileId(self.core.id)
    }
    pub fn symbol_arena(&self) -> ArenaId {
        self.symbols.id
    }
    pub fn lazy_arena(&self) -> ArenaId {
        self.lazy_id
    }
    pub fn source(&self) -> &[u8] {
        &self.source
    }
    pub fn canonical(&self) -> Option<FileId> {
        self.canonical
    }
    pub fn supplemental(&self) -> &[FileId] {
        &self.supplemental
    }
}

/// One outer owner holds all files in a content-mapped result. Files never own
/// each other, and a file handle obtained here always keeps this bundle alive.
pub struct BundleOwner<N, S = ()> {
    files: Vec<FileOwner<N, S>>,
    _owner: Track,
}
impl<N, S> BundleOwner<N, S> {
    pub fn new(canonical: FileBuilder<N, S>, supplemental: Vec<FileBuilder<N, S>>) -> Arc<Self> {
        let counters = canonical.counters.clone();
        let canonical_id = canonical.id();
        let ids = supplemental.iter().map(FileBuilder::id).collect();
        let mut files = vec![canonical.publish(None, ids)];
        files.extend(
            supplemental
                .into_iter()
                .map(|file| file.publish(Some(canonical_id), Vec::new())),
        );
        Arc::new(Self {
            files,
            _owner: counters.owner(),
        })
    }
    pub fn len(&self) -> usize {
        self.files.len()
    }
    pub fn is_empty(&self) -> bool {
        false
    }
    pub fn file(self: &Arc<Self>, index: usize) -> Option<FileHandle<N, S>> {
        (index < self.files.len()).then(|| FileHandle {
            root: Root::Bundle(self.clone(), index),
        })
    }
}

enum Root<N, S> {
    File(Arc<FileOwner<N, S>>),
    Bundle(Arc<BundleOwner<N, S>>, usize),
}
/// An owning file reference. Cloning a mapped reference retains the entire bundle.
pub struct FileHandle<N, S = ()> {
    root: Root<N, S>,
}
impl<N, S> Clone for FileHandle<N, S> {
    fn clone(&self) -> Self {
        Self {
            root: match &self.root {
                Root::File(file) => Root::File(file.clone()),
                Root::Bundle(bundle, index) => Root::Bundle(bundle.clone(), *index),
            },
        }
    }
}
impl<N, S> Deref for FileHandle<N, S> {
    type Target = FileOwner<N, S>;
    fn deref(&self) -> &Self::Target {
        match &self.root {
            Root::File(file) => file,
            Root::Bundle(bundle, index) => &bundle.files[*index],
        }
    }
}
impl<N, S> FileHandle<N, S> {
    /// Resolve a member through its retention group without changing any
    /// program's observable source-file ordering or membership.
    pub fn file(&self, id: FileId) -> Option<Self> {
        match &self.root {
            Root::File(file) => (file.id() == id).then(|| self.clone()),
            Root::Bundle(bundle, _) => bundle
                .files
                .iter()
                .position(|file| file.id() == id)
                .and_then(|index| bundle.file(index)),
        }
    }
    pub fn node(&self, id: NodeId) -> Result<NodeRef<N, S>, Error> {
        let location = if id.arena() == self.core.id {
            self.core.get(id.arena(), id.slot())?;
            Location::Core(id.slot() as usize - 1)
        } else if id.arena() == self.lazy_id {
            let state = self.lazy.read();
            let (page, offset) = state.pages.get(id.slot())?;
            Location::Lazy(page, offset)
        } else {
            return Err(Error::WrongOwner);
        };
        Ok(NodeRef {
            owner: self.clone(),
            id,
            location,
        })
    }
    pub fn symbol(&self, id: SymbolId) -> Result<SymbolRef<N, S>, Error> {
        self.symbols.get(id.arena(), id.slot())?;
        Ok(SymbolRef {
            owner: self.clone(),
            id,
        })
    }
    /// Initialize a JSDoc graph exactly once for a parent. The initializer runs
    /// under this file's write lock and must not reenter its lazy APIs. It builds
    /// privately; the whole graph and cache entry become visible together.
    pub fn jsdoc<F>(&self, parent: NodeId, initialize: F) -> Result<NodeListRef<N, S>, Error>
    where
        F: FnOnce(&mut LazyTransaction<'_, N>) -> Result<Vec<NodeId>, Error>,
    {
        self.node(parent)?;
        {
            let state = self.lazy.read();
            if let Some(ids) = state.jsdoc.get(&parent) {
                return Ok(NodeListRef {
                    owner: self.clone(),
                    ids: ids.clone(),
                });
            }
        }
        let mut state = self.lazy.write();
        if let Some(ids) = state.jsdoc.get(&parent) {
            return Ok(NodeListRef {
                owner: self.clone(),
                ids: ids.clone(),
            });
        }
        let base = state.pages.len;
        let mut transaction = LazyTransaction {
            pages: &mut state.pages,
            base,
            pending: Vec::new(),
        };
        let roots = initialize(&mut transaction)?;
        if roots.iter().any(|id| {
            id.arena() != self.lazy_id
                || id.slot() as usize <= transaction.base
                || id.slot() as usize > transaction.base + transaction.pending.len()
        }) {
            return Err(Error::InvalidGraph);
        }
        for (index, node) in transaction.pending.into_iter().enumerate() {
            transaction
                .pages
                .initialize((base + index + 1) as u32, node);
        }
        let roots: Arc<[NodeId]> = roots.into();
        state.jsdoc.insert(parent, roots.clone());
        Ok(NodeListRef {
            owner: self.clone(),
            ids: roots,
        })
    }
    /// Create a source token, preserving upstream's key, supplied parent and
    /// kind/reparsed-parent invariant panics. Payload construction runs only on
    /// the winning cache miss and must not reenter this file's lazy APIs.
    pub fn token<F>(&self, key: TokenKey, kind: u32, initialize: F) -> Result<NodeRef<N, S>, Error>
    where
        F: FnOnce() -> N,
    {
        let parent = self.node(key.parent)?;
        assert!(
            !parent.reparsed,
            "cannot create token for a reparsed parent"
        );
        assert!(
            key.start <= key.end && key.end <= self.source.len(),
            "invalid source token range"
        );
        {
            let state = self.lazy.read();
            if let Some(token) = state.tokens.get(&key) {
                assert_eq!(token.kind, kind, "token cache kind mismatch");
                let id = token.id;
                drop(state);
                return self.node(id);
            }
        }
        let mut state = self.lazy.write();
        let id = if let Some(token) = state.tokens.get(&key) {
            assert_eq!(token.kind, kind, "token cache kind mismatch");
            token.id
        } else {
            let mut node = Node::new(kind, initialize());
            node.parent = Some(key.parent);
            let slot = state.pages.push(node);
            let id = NodeId::new(self.lazy_id, slot);
            state.tokens.insert(key, CachedToken { id, kind });
            id
        };
        drop(state);
        self.node(id)
    }
}

/// Private staging area for one lazy graph. Its ids are not resolvable until the
/// enclosing publication succeeds. Every minted slot is permanently reserved;
/// failed initialization leaves an unresolvable tombstone, never a reusable id.
pub struct LazyTransaction<'a, N> {
    pages: &'a mut Pages<Node<N>>,
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TokenKey {
    pub parent: NodeId,
    pub start: usize,
    pub end: usize,
}
struct CachedToken {
    id: NodeId,
    kind: u32,
}
struct LazyState<N> {
    pages: Pages<Node<N>>,
    jsdoc: BTreeMap<NodeId, Arc<[NodeId]>>,
    tokens: BTreeMap<TokenKey, CachedToken>,
}
enum Location<N> {
    Core(usize),
    Lazy(Arc<Page<Node<N>>>, usize),
}

/// A retained immutable node. Lazy references own both the file/bundle and the
/// stable page; obtaining a reference never borrows a growing directory.
pub struct NodeRef<N, S = ()> {
    owner: FileHandle<N, S>,
    id: NodeId,
    location: Location<N>,
}
impl<N, S> NodeRef<N, S> {
    pub fn id(&self) -> NodeId {
        self.id
    }
    pub fn owner(&self) -> &FileHandle<N, S> {
        &self.owner
    }
}
impl<N, S> Deref for NodeRef<N, S> {
    type Target = Node<N>;
    fn deref(&self) -> &Self::Target {
        match &self.location {
            Location::Core(index) => &self.owner.core.values[*index],
            Location::Lazy(page, offset) => page.get(*offset),
        }
    }
}

pub struct SymbolRef<N, S> {
    owner: FileHandle<N, S>,
    id: SymbolId,
}
impl<N, S> SymbolRef<N, S> {
    pub fn id(&self) -> SymbolId {
        self.id
    }
}
impl<N, S> Deref for SymbolRef<N, S> {
    type Target = S;
    fn deref(&self) -> &S {
        &self.owner.symbols.values[self.id.slot() as usize - 1]
    }
}

/// An immutable published list and its file/bundle retention root. Internal cache
/// entries store just the ids; only escaped lists retain the enclosing owner.
pub struct NodeListRef<N, S = ()> {
    owner: FileHandle<N, S>,
    ids: Arc<[NodeId]>,
}
impl<N, S> NodeListRef<N, S> {
    pub fn ids(&self) -> &[NodeId] {
        &self.ids
    }
    pub fn node(&self, index: usize) -> Result<NodeRef<N, S>, Error> {
        self.owner
            .node(*self.ids.get(index).ok_or(Error::InvalidSlot)?)
    }
}

/// Owning membership table used by programs and operations. Import checks the
/// arena and published bounds on every call, including in optimized builds.
pub struct Scope<N, S = ()> {
    nodes: BTreeMap<ArenaId, FileHandle<N, S>>,
    symbols: BTreeMap<ArenaId, FileHandle<N, S>>,
}
impl<N, S> Default for Scope<N, S> {
    fn default() -> Self {
        Self {
            nodes: BTreeMap::new(),
            symbols: BTreeMap::new(),
        }
    }
}
impl<N, S> Scope<N, S> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn insert(&mut self, file: FileHandle<N, S>) {
        let members = match file.root {
            Root::File(owner) => vec![FileHandle {
                root: Root::File(owner),
            }],
            Root::Bundle(bundle, _) => (0..bundle.len())
                .map(|index| bundle.file(index).expect("bundle member"))
                .collect(),
        };
        for file in members {
            self.nodes.insert(file.core.id, file.clone());
            self.nodes.insert(file.lazy_id, file.clone());
            self.symbols.insert(file.symbols.id, file);
        }
    }
    pub fn import(&self, id: NodeId) -> Result<NodeRef<N, S>, Error> {
        self.nodes
            .get(&id.arena())
            .ok_or(Error::WrongOwner)?
            .node(id)
    }
    pub fn import_symbol(&self, id: SymbolId) -> Result<SymbolRef<N, S>, Error> {
        self.symbols
            .get(&id.arena())
            .ok_or(Error::WrongOwner)?
            .symbol(id)
    }
}

/// Request-local synthetic storage. Borrowed nodes cannot escape its lifetime;
/// a fresh scratch owner always has a new arena identity, even at the same slot.
pub struct ScratchOwner<N> {
    nodes: Slab<Node<N>>,
    _owner: Track,
}
impl<N> ScratchOwner<N> {
    pub fn new(counters: &Counters) -> Self {
        Self {
            nodes: Slab::new(counters),
            _owner: counters.owner(),
        }
    }
    pub fn push(&mut self, node: Node<N>) -> NodeId {
        NodeId::new(self.nodes.id, self.nodes.push(node))
    }
    pub fn node(&self, id: NodeId) -> Result<&Node<N>, Error> {
        self.nodes.get(id.arena(), id.slot())
    }
}
