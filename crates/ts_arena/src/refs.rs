use crate::{lazy::LazyNode, Error, FileHandle, LazyTransaction, Node, NodeId, SymbolId, TokenKey};
use std::{fmt, ops::Deref, sync::Arc};

enum Location<N> {
    Core(u32),
    Lazy(LazyNode<N>),
}

impl<N> Clone for Location<N> {
    fn clone(&self) -> Self {
        match self {
            Self::Core(slot) => Self::Core(*slot),
            Self::Lazy(node) => Self::Lazy(node.clone()),
        }
    }
}

impl<N> Location<N> {
    fn get<'a, S>(&'a self, owner: &'a FileHandle<N, S>) -> &'a Node<N> {
        match self {
            Self::Core(slot) => owner.core.get_slot(*slot).expect("validated core slot"),
            Self::Lazy(node) => node.get(),
        }
    }
}

/// A checked borrow of a node. Core lookup does not clone an Arc or acquire a lock.
pub struct NodeRef<'a, N, S = ()> {
    owner: &'a FileHandle<N, S>,
    id: NodeId,
    location: Location<N>,
}

impl<N, S> NodeRef<'_, N, S> {
    pub fn id(&self) -> NodeId {
        self.id
    }
    pub fn owner(&self) -> &FileHandle<N, S> {
        self.owner
    }

    /// Explicitly retain the enclosing file or mapped bundle for an escaped result.
    pub fn retain(self) -> RetainedNode<N, S> {
        RetainedNode {
            owner: self.owner.clone(),
            id: self.id,
            location: self.location,
        }
    }
}

impl<N, S> Deref for NodeRef<'_, N, S> {
    type Target = Node<N>;
    fn deref(&self) -> &Self::Target {
        self.location.get(self.owner)
    }
}

impl<N: fmt::Debug, S> fmt::Debug for NodeRef<'_, N, S> {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(output)
    }
}

/// An escaped node plus its complete retention root, including mapped siblings.
pub struct RetainedNode<N, S = ()> {
    owner: FileHandle<N, S>,
    id: NodeId,
    location: Location<N>,
}

impl<N: fmt::Debug, S> fmt::Debug for RetainedNode<N, S> {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(output)
    }
}

impl<N, S> RetainedNode<N, S> {
    pub fn id(&self) -> NodeId {
        self.id
    }
    pub fn owner(&self) -> &FileHandle<N, S> {
        &self.owner
    }
}

impl<N, S> Clone for RetainedNode<N, S> {
    fn clone(&self) -> Self {
        Self {
            owner: self.owner.clone(),
            id: self.id,
            location: self.location.clone(),
        }
    }
}

impl<N, S> Deref for RetainedNode<N, S> {
    type Target = Node<N>;
    fn deref(&self) -> &Self::Target {
        self.location.get(&self.owner)
    }
}

/// Checked symbol access borrows the scope; `retain` opts into an escaped owner root.
pub struct SymbolRef<'a, N, S> {
    owner: &'a FileHandle<N, S>,
    id: SymbolId,
}

impl<N, S> fmt::Debug for SymbolRef<'_, N, S> {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output.debug_tuple("SymbolRef").field(&self.id).finish()
    }
}

impl<N, S> SymbolRef<'_, N, S> {
    pub fn id(&self) -> SymbolId {
        self.id
    }
    pub fn retain(self) -> RetainedSymbol<N, S> {
        RetainedSymbol {
            owner: self.owner.clone(),
            id: self.id,
        }
    }
}

impl<N, S> Deref for SymbolRef<'_, N, S> {
    type Target = S;
    fn deref(&self) -> &S {
        self.owner
            .symbols
            .get_slot(self.id.slot())
            .expect("validated symbol slot")
    }
}

/// An escaped symbol retaining its file or mapped bundle.
pub struct RetainedSymbol<N, S> {
    owner: FileHandle<N, S>,
    id: SymbolId,
}

impl<N, S> fmt::Debug for RetainedSymbol<N, S> {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output
            .debug_tuple("RetainedSymbol")
            .field(&self.id)
            .finish()
    }
}

impl<N, S> RetainedSymbol<N, S> {
    pub fn id(&self) -> SymbolId {
        self.id
    }
}

impl<N, S> Deref for RetainedSymbol<N, S> {
    type Target = S;
    fn deref(&self) -> &S {
        self.owner
            .symbols
            .get_slot(self.id.slot())
            .expect("validated symbol slot")
    }
}

/// Cached ids are shared without copying the list. Only returned lists retain a file.
pub struct NodeListRef<N, S = ()> {
    owner: FileHandle<N, S>,
    ids: Arc<[NodeId]>,
}

impl<N, S> fmt::Debug for NodeListRef<N, S> {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output.debug_tuple("NodeListRef").field(&self.ids).finish()
    }
}

impl<N, S> NodeListRef<N, S> {
    pub fn ids(&self) -> &[NodeId] {
        &self.ids
    }
    pub fn node(&self, index: usize) -> Result<NodeRef<'_, N, S>, Error> {
        self.owner
            .node(*self.ids.get(index).ok_or(Error::InvalidSlot)?)
    }
}

impl<N, S> FileHandle<N, S> {
    pub fn node(&self, id: NodeId) -> Result<NodeRef<'_, N, S>, Error> {
        let location = if id.arena() == self.core.id {
            self.core.get_slot(id.slot())?;
            Location::Core(id.slot())
        } else if id.arena() == self.lazy_arena() {
            Location::Lazy(self.lazy.node(id)?)
        } else {
            return Err(Error::WrongOwner);
        };
        Ok(NodeRef {
            owner: self,
            id,
            location,
        })
    }

    pub fn retain_node(&self, id: NodeId) -> Result<RetainedNode<N, S>, Error> {
        self.node(id).map(NodeRef::retain)
    }

    pub fn symbol(&self, id: SymbolId) -> Result<SymbolRef<'_, N, S>, Error> {
        self.symbols.get(id.arena(), id.slot())?;
        Ok(SymbolRef { owner: self, id })
    }

    /// Runs the winning initializer under this file's lazy write lock. The callback
    /// panics on same-thread reentry to this file's lazy APIs. The callback must not
    /// wait for another thread to use those APIs. Failure burns every provisional id.
    pub fn jsdoc(
        &self,
        parent: NodeId,
        initialize: impl FnOnce(&mut LazyTransaction<'_, N>) -> Result<Vec<NodeId>, Error>,
    ) -> Result<NodeListRef<N, S>, Error> {
        self.node(parent)?;
        let ids = self.lazy.jsdoc(parent, initialize)?;
        Ok(NodeListRef {
            owner: self.clone(),
            ids,
        })
    }

    /// Source-token access with upstream invariant panics. The initializer must
    /// not wait for another thread to use this file's lazy APIs; same-thread reentry
    /// panics before locking. A cache hit never calls the initializer.
    pub fn token(
        &self,
        key: TokenKey,
        kind: u32,
        initialize: impl FnOnce() -> N,
    ) -> Result<NodeRef<'_, N, S>, Error> {
        match self.try_token(key, kind, initialize) {
            Err(
                error @ (Error::TokenKindMismatch { .. }
                | Error::ReparsedParent
                | Error::InvalidTokenRange),
            ) => panic!("{error}"),
            result => result,
        }
    }

    /// Fallible invariant checks; the parent flags always come from retained storage.
    pub fn try_token(
        &self,
        key: TokenKey,
        kind: u32,
        initialize: impl FnOnce() -> N,
    ) -> Result<NodeRef<'_, N, S>, Error> {
        let parent = self.node(key.parent)?;
        let id = self
            .lazy
            .token(key, kind, parent.reparsed, self.source().len(), initialize)?;
        self.node(id)
    }
}
