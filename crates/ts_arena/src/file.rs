use crate::{
    arena::Arena, bundle::Root, counters::Track, lazy::LazyArena, ArenaId, AuxId, Counters, Error,
    FileId, NodeId, NodeRecord, StorageHandle, StorageRead, StorageTransaction, SymbolId,
};
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};
use ts_jsstring::{PositionMap, SourceText};

/// Exclusive construction owns the same storage later transferred into a file.
/// Read views cannot retain this unpublished owner; mutation requires `&mut self`.
pub struct StorageBuilder<N: NodeRecord, S = ()> {
    owner: StorageOwner<N, S>,
    pub(crate) counters: Counters,
}

// Only the imported capability requires shared payloads. Keeping that bound on
// this private object preserves Send-only exclusive builders with no imports.
trait RetainedImport<N: NodeRecord, S>: Send + Sync {
    fn owner(&self) -> &StorageOwner<N, S>;
    fn handle(&self) -> StorageHandle<N, S>;
    fn borrowed_handle(&self) -> &StorageHandle<N, S>;
}
impl<N, S> RetainedImport<N, S> for StorageHandle<N, S>
where
    N: NodeRecord + Send + Sync,
    N::Aux: Send + Sync,
    S: Send + Sync,
{
    fn owner(&self) -> &StorageOwner<N, S> {
        self
    }
    fn handle(&self) -> StorageHandle<N, S> {
        self.clone()
    }
    fn borrowed_handle(&self) -> &StorageHandle<N, S> {
        self
    }
}
impl<N: NodeRecord, S> std::fmt::Debug for StorageBuilder<N, S> {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.debug_struct("StorageBuilder")
            .field("id", &self.id())
            .finish_non_exhaustive()
    }
}
impl<N: NodeRecord, S> StorageBuilder<N, S> {
    pub fn new(bytes: Arc<[u8]>, counters: &Counters) -> Self {
        Self::from_source_text(SourceText::from_bytes(bytes), counters)
    }
    pub fn from_source_text(source: SourceText, counters: &Counters) -> Self {
        Self {
            owner: StorageOwner {
                core: Arena::new(counters),
                symbols: Arena::new(counters),
                auxiliary: Arena::new(counters),
                lazy: LazyArena::new(counters),
                source,
                position_map: OnceLock::new(),
                canonical: None,
                supplemental: Vec::new(),
                metadata: None,
                imports: Vec::new(),
                imported_arenas: HashMap::new(),
                _owner: counters.owner(),
            },
            counters: counters.clone(),
        }
    }
    pub fn id(&self) -> FileId {
        self.owner.id()
    }
    pub fn view(&self) -> StorageView<'_, N, S> {
        self.owner.view()
    }
    /// Retain a previously published file and its complete mapped bundle.
    /// Imports are flattened and deduplicated at this explicit ownership boundary;
    /// ordinary lookup only borrows the selected owner. Published-only inputs
    /// make dependencies acyclic by publication order.
    pub fn retain_file(&mut self, file: StorageHandle<N, S>)
    where
        N: Send + Sync + 'static,
        N::Aux: Send + Sync + 'static,
        S: Send + Sync + 'static,
    {
        if self.owner.imported_arenas.contains_key(&file.core.id) {
            return;
        }
        for member in file.into_members() {
            for dependency in &member.imports {
                self.owner.retain_import(dependency.handle());
            }
            self.owner.retain_import(member);
        }
    }
    /// Borrow all prospective bundle owners while validating their cross-file
    /// identities. The operation cannot return a view borrowing this group table.
    ///
    /// ```compile_fail
    /// use ts_arena::{Counters, Node, StorageBuilder};
    /// let builder = StorageBuilder::<Node<()>>::new(std::sync::Arc::from(&b""[..]), &Counters::new());
    /// let escaped = StorageBuilder::with_group_views(&[&builder], |views| views[0]);
    /// println!("{:?}", escaped.id());
    /// ```
    pub fn with_group_views<T>(
        builders: &[&Self],
        operation: impl FnOnce(&[StorageView<'_, N, S>]) -> T,
    ) -> T {
        let owners: Vec<_> = builders.iter().map(|builder| &builder.owner).collect();
        let views: Vec<_> = owners
            .iter()
            .map(|&owner| StorageView {
                owner,
                retention: ViewRetention::Group(&owners),
            })
            .collect();
        operation(&views)
    }
    pub fn push(&mut self, node: N) -> NodeId {
        NodeId::new(self.owner.core.id, self.owner.core.push(node))
    }
    pub fn push_aux(&mut self, value: N::Aux) -> AuxId {
        AuxId::new(self.owner.auxiliary.id, self.owner.auxiliary.push(value))
    }
    pub fn push_symbol(&mut self, symbol: S) -> SymbolId {
        SymbolId::new(self.owner.symbols.id, self.owner.symbols.push(symbol))
    }
    pub fn node_mut(&mut self, id: NodeId) -> Result<&mut N, Error> {
        self.owner.core.get_mut(id.arena(), id.slot())
    }
    pub fn aux_mut(&mut self, id: AuxId) -> Result<&mut N::Aux, Error> {
        self.owner.auxiliary.get_mut(id.arena(), id.slot())
    }
    pub fn symbol_mut(&mut self, id: SymbolId) -> Result<&mut S, Error> {
        self.owner.symbols.get_mut(id.arena(), id.slot())
    }

    /// Select a core auxiliary record containing runtime-defined file metadata.
    pub fn set_metadata(&mut self, id: AuxId) -> Result<(), Error> {
        self.owner.auxiliary.get(id.arena(), id.slot())?;
        self.owner.metadata = Some(id);
        Ok(())
    }
    /// Seed eager roots only from validated core storage. A populated cache is not overwritten.
    pub fn seed_jsdoc(&mut self, parent: NodeId, roots: Vec<NodeId>) -> Result<(), Error> {
        self.owner.core.get(parent.arena(), parent.slot())?;
        for id in &roots {
            self.owner.core.get(id.arena(), id.slot())?;
        }
        self.owner.lazy.seed_jsdoc(None, parent, roots.into())
    }
    /// Seed a logical source's cache, independently of other sources sharing
    /// its storage owner. Only validated core references can be seeded.
    pub fn seed_source_jsdoc(
        &mut self,
        source: NodeId,
        parent: NodeId,
        roots: Vec<NodeId>,
    ) -> Result<(), Error> {
        self.owner.core.get(source.arena(), source.slot())?;
        self.owner.core.get(parent.arena(), parent.slot())?;
        for id in &roots {
            self.owner.core.get(id.arena(), id.slot())?;
        }
        self.owner
            .lazy
            .seed_jsdoc(Some(source), parent, roots.into())
    }
    pub fn finish(self) -> StorageHandle<N, S> {
        StorageHandle {
            root: Root::File(Arc::new(self.publish(None, Vec::new()))),
        }
    }
    pub(crate) fn publish(
        mut self,
        canonical: Option<FileId>,
        supplemental: Vec<FileId>,
    ) -> StorageOwner<N, S> {
        self.owner.canonical = canonical;
        self.owner.supplemental = supplemental;
        self.owner
    }
}

/// Owns core and lazy records, auxiliary data, symbols and source bytes together.
/// Shared access has no mutable core path; the exclusive builder contains this
/// same owner before binding and publication.
pub struct StorageOwner<N: NodeRecord, S = ()> {
    pub(crate) core: Arena<N>,
    pub(crate) symbols: Arena<S>,
    pub(crate) auxiliary: Arena<N::Aux>,
    pub(crate) lazy: LazyArena<N>,
    source: SourceText,
    position_map: OnceLock<PositionMap>,
    canonical: Option<FileId>,
    supplemental: Vec<FileId>,
    metadata: Option<AuxId>,
    imports: Vec<Box<dyn RetainedImport<N, S>>>,
    imported_arenas: HashMap<ArenaId, usize>,
    _owner: Track,
}
impl<N: NodeRecord, S> std::fmt::Debug for StorageOwner<N, S> {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.debug_struct("StorageOwner")
            .field("id", &self.id())
            .field("lazy_arena", &self.lazy_arena())
            .field("canonical", &self.canonical)
            .field("supplemental", &self.supplemental)
            .finish_non_exhaustive()
    }
}
impl<N: NodeRecord, S> StorageOwner<N, S> {
    pub(crate) fn imported_view(&self, arena: ArenaId) -> Option<StorageView<'_, N, S>> {
        let import = self.imports.get(*self.imported_arenas.get(&arena)?)?;
        import.borrowed_handle().view().for_arena(arena).ok()
    }
    fn retain_import(&mut self, file: StorageHandle<N, S>)
    where
        N: Send + Sync + 'static,
        N::Aux: Send + Sync + 'static,
        S: Send + Sync + 'static,
    {
        if self.imported_arenas.contains_key(&file.core.id) {
            return;
        }
        let index = self.imports.len();
        for arena in [
            file.core.id,
            file.lazy_arena(),
            file.auxiliary_arena(),
            file.lazy_auxiliary_arena(),
        ] {
            self.imported_arenas.insert(arena, index);
        }
        self.imports.push(Box::new(file));
    }
    pub(crate) fn imported_owner(&self, arena: ArenaId) -> Option<&StorageOwner<N, S>> {
        self.imported_arenas
            .get(&arena)
            .map(|&index| self.imports[index].owner())
    }
    pub(crate) fn imported_file(&self, id: FileId) -> Option<StorageHandle<N, S>> {
        self.imported_arenas
            .get(&id.0)
            .map(|&index| self.imports[index].handle())
    }
    pub fn id(&self) -> FileId {
        FileId(self.core.id)
    }
    pub fn symbol_arena(&self) -> ArenaId {
        self.symbols.id
    }
    pub fn auxiliary_arena(&self) -> ArenaId {
        self.auxiliary.id
    }
    pub fn lazy_arena(&self) -> ArenaId {
        self.lazy.id()
    }
    pub fn lazy_auxiliary_arena(&self) -> ArenaId {
        self.lazy.auxiliary_id()
    }
    pub fn source(&self) -> &[u8] {
        self.source.as_bytes()
    }
    pub fn source_text(&self) -> &SourceText {
        &self.source
    }
    pub fn position_map(&self) -> &PositionMap {
        self.position_map
            .get_or_init(|| PositionMap::new(self.source.as_bytes()))
    }
    pub fn canonical(&self) -> Option<FileId> {
        self.canonical
    }
    pub fn supplemental(&self) -> &[FileId] {
        &self.supplemental
    }
    pub fn metadata(&self) -> Option<AuxId> {
        self.metadata
    }
    pub fn view(&self) -> StorageView<'_, N, S> {
        StorageView {
            owner: self,
            retention: ViewRetention::Owner(self),
        }
    }
}

/// A non-retaining view shared by exclusive and published file storage. Core
/// reads borrow directly; lazy reads lock the directory and retain a stable page.
pub struct StorageView<'a, N: NodeRecord, S = ()> {
    owner: &'a StorageOwner<N, S>,
    retention: ViewRetention<'a, N, S>,
}
enum ViewRetention<'a, N: NodeRecord, S> {
    Owner(&'a StorageOwner<N, S>),
    Handle(&'a StorageHandle<N, S>),
    Group(&'a [&'a StorageOwner<N, S>]),
}
impl<N: NodeRecord, S> Copy for ViewRetention<'_, N, S> {}
impl<N: NodeRecord, S> Clone for ViewRetention<'_, N, S> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<N: NodeRecord, S> Copy for StorageView<'_, N, S> {}
impl<N: NodeRecord, S> Clone for StorageView<'_, N, S> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<N: NodeRecord, S> std::fmt::Debug for StorageView<'_, N, S> {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.owner.fmt(out)
    }
}
impl<'a, N: NodeRecord, S> StorageView<'a, N, S> {
    pub(crate) fn retained(handle: &'a StorageHandle<N, S>) -> Self {
        Self {
            owner: handle,
            retention: ViewRetention::Handle(handle),
        }
    }
    pub(crate) fn for_arena(self, arena: ArenaId) -> Result<Self, Error> {
        let contains = |owner: &&StorageOwner<N, S>| {
            owner.core.id == arena
                || owner.lazy_arena() == arena
                || owner.auxiliary_arena() == arena
                || owner.lazy_auxiliary_arena() == arena
        };
        if contains(&self.owner) {
            return Ok(self);
        }
        let owner = match self.retention {
            ViewRetention::Owner(owner) => Some(owner)
                .filter(contains)
                .or_else(|| owner.imported_owner(arena)),
            ViewRetention::Handle(handle) => handle.retained_owner(arena),
            ViewRetention::Group(owners) => owners
                .iter()
                .copied()
                .find(contains)
                .or_else(|| owners.iter().find_map(|owner| owner.imported_owner(arena))),
        }
        .ok_or(Error::WrongOwner)?;
        Ok(Self {
            owner,
            retention: self.retention,
        })
    }
    pub fn for_node_owner(self, node: NodeId) -> Result<Self, Error> {
        let view = self.for_arena(node.arena())?;
        view.node_here(node)?;
        Ok(view)
    }
    /// Narrow a selected owner's service to the storage that owner will retain
    /// independently. Ordinary traversal keeps its caller's broader context;
    /// persistent caches cannot borrow an importing owner's extra capabilities.
    pub fn owner_retention(self) -> Result<Self, Error> {
        let arena = self.owner.core.id;
        match self.retention {
            ViewRetention::Owner(owner) => {
                if owner.core.id == arena {
                    Ok(self)
                } else {
                    owner.imported_view(arena).ok_or(Error::WrongOwner)
                }
            }
            ViewRetention::Handle(handle) => handle.owner_retention(arena).ok_or(Error::WrongOwner),
            ViewRetention::Group(owners) => {
                if owners.iter().any(|owner| owner.core.id == arena) {
                    Ok(self)
                } else {
                    owners
                        .iter()
                        .find_map(|owner| owner.imported_view(arena))
                        .ok_or(Error::WrongOwner)
                }
            }
        }
    }
    pub fn core_nodes(self) -> impl Iterator<Item = &'a N> {
        self.owner.core.values()
    }
    pub fn core_auxiliary(self) -> impl Iterator<Item = &'a N::Aux> {
        self.owner.auxiliary.values()
    }
    pub fn id(self) -> FileId {
        self.owner.id()
    }
    pub fn metadata(self) -> Option<AuxId> {
        self.owner.metadata
    }
    pub fn source(self) -> &'a SourceText {
        self.owner.source_text()
    }
    pub fn position_map(self) -> &'a PositionMap {
        self.owner.position_map()
    }
    pub fn lazy_arena(self) -> ArenaId {
        self.owner.lazy_arena()
    }
    pub fn lazy_auxiliary_arena(self) -> ArenaId {
        self.owner.lazy_auxiliary_arena()
    }
    pub fn node(self, id: NodeId) -> Result<StorageRead<'a, N>, Error> {
        self.for_arena(id.arena())?.node_here(id)
    }
    fn node_here(self, id: NodeId) -> Result<StorageRead<'a, N>, Error> {
        if id.arena() == self.owner.core.id {
            Ok(StorageRead::borrowed(self.owner.core.get_slot(id.slot())?))
        } else {
            Ok(StorageRead::lazy(self.owner.lazy.node(id)?))
        }
    }
    pub fn aux(self, id: AuxId) -> Result<StorageRead<'a, N::Aux>, Error> {
        self.for_arena(id.arena())?.aux_here(id)
    }
    fn aux_here(self, id: AuxId) -> Result<StorageRead<'a, N::Aux>, Error> {
        if id.arena() == self.owner.auxiliary.id {
            Ok(StorageRead::borrowed(
                self.owner.auxiliary.get_slot(id.slot())?,
            ))
        } else {
            Ok(StorageRead::lazy(self.owner.lazy.aux(id)?))
        }
    }
    /// Cache-only lookup. It never invokes a parser or allocates nodes.
    pub fn eager_jsdoc(self, parent: NodeId) -> Result<Option<Arc<[NodeId]>>, Error> {
        let selected = self.for_node_owner(parent)?;
        Ok(selected.owner.lazy.eager_jsdoc(None, parent))
    }
    /// Cache lookup for one logical source, including imported source owners.
    pub fn source_eager_jsdoc(
        self,
        source: NodeId,
        parent: NodeId,
    ) -> Result<Option<Arc<[NodeId]>>, Error> {
        let selected = self.for_node_owner(source)?;
        selected.node_here(parent)?;
        Ok(selected.owner.lazy.eager_jsdoc(Some(source), parent))
    }
    /// The caller must retain this borrow for the operation. Dispatch the whole
    /// operation to any worker before entering this lock, never the initializer.
    pub fn jsdoc(
        self,
        parent: NodeId,
        initialize: impl FnOnce(&mut StorageTransaction<'_, N>) -> Result<Vec<NodeId>, Error>,
    ) -> Result<Arc<[NodeId]>, Error> {
        let selected = self.for_node_owner(parent)?;
        selected.owner.lazy.jsdoc(
            None,
            parent,
            &selected.owner.core,
            &selected.owner.auxiliary,
            initialize,
        )
    }
    /// Initialize in the selected source's storage on the calling thread. The
    /// parent must belong to that owner; the source key is independent of it.
    pub fn source_jsdoc(
        self,
        source: NodeId,
        parent: NodeId,
        initialize: impl FnOnce(&mut StorageTransaction<'_, N>) -> Result<Vec<NodeId>, Error>,
    ) -> Result<Arc<[NodeId]>, Error> {
        let selected = self.for_node_owner(source)?;
        selected.node_here(parent)?;
        selected.owner.lazy.jsdoc(
            Some(source),
            parent,
            &selected.owner.core,
            &selected.owner.auxiliary,
            initialize,
        )
    }
    pub fn try_token_record(
        self,
        key: crate::TokenKey,
        kind: u32,
        initialize: impl FnOnce() -> N,
    ) -> Result<StorageRead<'a, N>, Error> {
        let selected = self.for_node_owner(key.parent)?;
        let parent = selected.node(key.parent)?;
        let id = selected.owner.lazy.token(
            key,
            kind,
            parent.storage_reparsed(),
            selected.owner.source().len(),
            initialize,
        )?;
        self.node(id)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Node;
    use std::sync::Barrier;

    #[test]
    fn position_map_is_initialized_on_demand_and_shared_between_threads() {
        let file =
            StorageBuilder::<Node<()>>::new(Arc::from("\u{1f600}x".as_bytes()), &Counters::new())
                .finish();
        assert!(file.position_map.get().is_none());
        assert_eq!(file.source(), "\u{1f600}x".as_bytes());
        assert!(file.position_map.get().is_none());
        let barrier = Barrier::new(4);
        std::thread::scope(|scope| {
            let handles: Vec<_> = (0..4)
                .map(|_| {
                    scope.spawn(|| {
                        barrier.wait();
                        file.position_map()
                    })
                })
                .collect();
            let maps: Vec<_> = handles
                .into_iter()
                .map(|thread| thread.join().unwrap())
                .collect();
            assert!(maps.windows(2).all(|pair| std::ptr::eq(pair[0], pair[1])));
            assert_eq!(maps[0].utf8_to_utf16(4), 2);
        });
        assert!(std::ptr::eq(
            file.position_map.get().unwrap(),
            file.position_map()
        ));
    }
}

#[cfg(test)]
mod construction_tests {
    use super::*;
    use crate::Node;

    #[test]
    fn construction_transfers_requested_map_and_lazy_arena_without_reinitializing() {
        let builder = StorageBuilder::<Node<()>>::new(Arc::from(&b"x\ny"[..]), &Counters::new());
        let lazy_id = builder.view().lazy_arena();
        let auxiliary_id = builder.view().lazy_auxiliary_arena();
        assert!(builder.owner.position_map.get().is_none());
        assert_eq!(builder.view().position_map().utf8_to_utf16(3), 3);
        assert!(builder.owner.position_map.get().is_some());
        let file = builder.finish();
        assert!(file.position_map.get().is_some());
        assert_eq!(file.lazy_arena(), lazy_id);
        assert_eq!(file.lazy_auxiliary_arena(), auxiliary_id);
    }

    #[test]
    fn exclusive_worker_transfer_does_not_require_sync_symbol_payloads() {
        let mut builder = StorageBuilder::<Node<()>, std::cell::Cell<u32>>::new(
            Arc::from(&b"x"[..]),
            &Counters::new(),
        );
        let id = builder.push_symbol(std::cell::Cell::new(1));
        let mut builder = std::thread::spawn(move || {
            builder.symbol_mut(id).unwrap().set(2);
            builder
        })
        .join()
        .unwrap();
        assert_eq!(builder.symbol_mut(id).unwrap().get(), 2);
    }
}
