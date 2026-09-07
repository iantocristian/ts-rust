use crate::NodeRecord;
use crate::{
    arena::Arena, ArenaId, Error, NodeId, RecordRef, RetainedRecord, StorageHandle,
    StorageSymbolRef, SymbolId,
};
use std::{collections::BTreeMap, marker::PhantomData};

/// An owning membership table. Raw imports check owner and published slot in release.
pub struct StorageScope<N: NodeRecord, S = ()> {
    nodes: BTreeMap<ArenaId, StorageHandle<N, S>>,
    symbols: BTreeMap<ArenaId, StorageHandle<N, S>>,
}

impl<N: NodeRecord, S> Default for StorageScope<N, S> {
    fn default() -> Self {
        Self {
            nodes: BTreeMap::new(),
            symbols: BTreeMap::new(),
        }
    }
}

impl<N: NodeRecord, S> StorageScope<N, S> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Retaining a mapped member admits every sibling to the resolution set.
    /// This table does not define a program's observable source-file list.
    pub fn insert(&mut self, file: StorageHandle<N, S>) {
        for member in file.into_members() {
            self.nodes.insert(member.core.id, member.clone());
            self.nodes.insert(member.lazy_arena(), member.clone());
            self.symbols.insert(member.symbol_arena(), member);
        }
    }

    pub fn import(&self, id: NodeId) -> Result<RecordRef<'_, N, S>, Error> {
        self.nodes
            .get(&id.arena())
            .ok_or(Error::WrongOwner)?
            .node(id)
    }

    pub fn import_retained(&self, id: NodeId) -> Result<RetainedRecord<N, S>, Error> {
        self.import(id).map(RecordRef::retain)
    }

    pub fn import_symbol(&self, id: SymbolId) -> Result<StorageSymbolRef<'_, N, S>, Error> {
        self.symbols
            .get(&id.arena())
            .ok_or(Error::WrongOwner)?
            .symbol(id)
    }

    /// Open one immutable core arena with a fresh invariant brand. Checks mint
    /// local handles that cannot escape or be used with a different arena.
    ///
    /// ```compile_fail
    /// use ts_arena::{Counters, StorageBuilder, Node, StorageScope};
    /// let counters = Counters::new();
    /// let mut builder = StorageBuilder::<Node<u32>>::new(std::sync::Arc::from(&b"x"[..]), &counters);
    /// let id = builder.push(Node::new(1, 7));
    /// let mut scope = StorageScope::new();
    /// scope.insert(builder.finish());
    /// let escaped = scope.with_core_arena(id.arena(), |local| local.check(id).unwrap());
    /// ```
    ///
    /// ```compile_fail
    /// use ts_arena::{Counters, StorageBuilder, Node, StorageScope};
    /// let counters = Counters::new();
    /// let mut first = StorageBuilder::<Node<u32>>::new(std::sync::Arc::from(&b"x"[..]), &counters);
    /// let mut second = StorageBuilder::<Node<u32>>::new(std::sync::Arc::from(&b"y"[..]), &counters);
    /// let first_id = first.push(Node::new(1, 7));
    /// let second_id = second.push(Node::new(1, 8));
    /// let mut scope = StorageScope::new();
    /// scope.insert(first.finish()); scope.insert(second.finish());
    /// scope.with_core_arena(first_id.arena(), |first| {
    ///     let local = first.check(first_id).unwrap();
    ///     scope.with_core_arena(second_id.arena(), |second| second.get(local));
    /// });
    /// ```
    pub fn with_core_arena<R>(
        &self,
        arena: ArenaId,
        operation: impl for<'brand> FnOnce(StorageLocalArena<'brand, '_, N>) -> R,
    ) -> Result<R, Error> {
        let file = self.nodes.get(&arena).ok_or(Error::WrongOwner)?;
        if file.core.id != arena {
            return Err(Error::WrongOwner);
        }
        Ok(operation(StorageLocalArena {
            arena: &file.core,
            brand: PhantomData,
        }))
    }
}

type Brand<'brand> = PhantomData<fn(&'brand ()) -> &'brand ()>;

/// One immutable core arena, with an invariant brand private to this callback.
pub struct StorageLocalArena<'brand, 'owner, N: NodeRecord> {
    arena: &'owner Arena<N>,
    brand: Brand<'brand>,
}

/// Proof of an owner-and-bounds check within one fresh arena scope.
#[derive(Clone, Copy, Debug)]
pub struct LocalNode<'brand> {
    slot: u32,
    brand: Brand<'brand>,
}

impl<'brand, 'owner, N: NodeRecord> StorageLocalArena<'brand, 'owner, N> {
    pub fn check(&self, id: NodeId) -> Result<LocalNode<'brand>, Error> {
        self.arena.get(id.arena(), id.slot())?;
        Ok(LocalNode {
            slot: id.slot(),
            brand: PhantomData,
        })
    }

    /// Repeated reads elide owner validation and retain safe bounds checks.
    pub fn get(&self, local: LocalNode<'brand>) -> &'owner N {
        self.arena
            .get_slot(local.slot)
            .expect("branded slot was validated")
    }

    pub fn id(&self, local: LocalNode<'brand>) -> NodeId {
        NodeId::new(self.arena.id, local.slot)
    }
}
