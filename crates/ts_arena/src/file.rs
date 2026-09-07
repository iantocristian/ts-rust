use std::sync::{Arc, OnceLock};
use ts_jsstring::{PositionMap, SourceText};

use crate::{
    arena::Arena, bundle::Root, counters::Track, lazy::LazyArena, ArenaId, Counters, Error,
    FileHandle, FileId, Node, NodeId, SymbolId,
};

/// Mutable parser/binder construction. Finishing consumes all mutable access.
pub struct FileBuilder<N, S = ()> {
    pub(crate) core: Arena<Node<N>>,
    symbols: Arena<S>,
    source: SourceText,
    pub(crate) counters: Counters,
}

impl<N, S> std::fmt::Debug for FileBuilder<N, S> {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        output
            .debug_struct("FileBuilder")
            .field("id", &self.id())
            .finish_non_exhaustive()
    }
}

impl<N, S> FileBuilder<N, S> {
    pub fn new(bytes: Arc<[u8]>, counters: &Counters) -> Self {
        Self::from_source_text(SourceText::from_bytes(bytes), counters)
    }

    pub fn from_source_text(source: SourceText, counters: &Counters) -> Self {
        Self {
            core: Arena::new(counters),
            symbols: Arena::new(counters),
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
        self.core.get_mut(id.arena(), id.slot())
    }

    pub fn finish(self) -> FileHandle<N, S> {
        FileHandle {
            root: Root::File(Arc::new(self.publish(None, Vec::new()))),
        }
    }

    pub(crate) fn publish(
        self,
        canonical: Option<FileId>,
        supplemental: Vec<FileId>,
    ) -> FileOwner<N, S> {
        FileOwner {
            core: self.core,
            symbols: self.symbols,
            position_map: OnceLock::new(),
            source: self.source,
            canonical,
            supplemental,
            lazy: LazyArena::new(&self.counters),
            _owner: self.counters.owner(),
        }
    }
}

/// Immutable core storage, symbols, source text and independently published lazy nodes.
/// A mapped file can be retained only through its bundle-preserving `FileHandle`.
pub struct FileOwner<N, S = ()> {
    pub(crate) core: Arena<Node<N>>,
    pub(crate) symbols: Arena<S>,
    pub(crate) lazy: LazyArena<N>,
    source: SourceText,
    position_map: OnceLock<PositionMap>,
    canonical: Option<FileId>,
    supplemental: Vec<FileId>,
    _owner: Track,
}

impl<N, S> std::fmt::Debug for FileOwner<N, S> {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        output
            .debug_struct("FileOwner")
            .field("id", &self.id())
            .field("lazy_arena", &self.lazy_arena())
            .field("canonical", &self.canonical)
            .field("supplemental", &self.supplemental)
            .finish_non_exhaustive()
    }
}

impl<N, S> FileOwner<N, S> {
    pub fn id(&self) -> FileId {
        FileId(self.core.id)
    }
    pub fn symbol_arena(&self) -> ArenaId {
        self.symbols.id
    }
    pub fn lazy_arena(&self) -> ArenaId {
        self.lazy.id()
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Barrier;

    #[test]
    fn position_map_is_initialized_on_demand_and_shared_between_threads() {
        let file =
            FileBuilder::<()>::new(Arc::from("\u{1f600}x".as_bytes()), &Counters::new()).finish();
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
