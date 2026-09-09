//! Checked semantic reads borrow their physical owner and its compact payloads.
use crate::compact::{CompactContext, FieldKey, StoredNode};
use crate::{AstStorageData, Node, NodeDataRead, NodeDataSource, NodeId};
use ts_arena::{AuxId, FileId, StorageRead, StorageView};
use ts_jsstring::SourceText;

/// A node read borrows its physical owner. Lazy payload references cannot outlive
/// the read's publication guard. Escaping requires `RetainedNode`.
///
/// ```compile_fail
/// use ts_ast::{AstFile, NodeId};
/// fn escaped_text<'owner>(file: &'owner AstFile, lazy: NodeId) -> &'owner [u8] {
///     let read = file.view().node(lazy).unwrap();
///     read.as_identifier().unwrap().text()
/// }
/// ```
///
/// ```compile_fail
/// use ts_ast::{AstFile, NodeId};
/// fn escaped_payload<'owner>(file: &'owner AstFile, lazy: NodeId) -> &'owner [u8] {
///     let read = file.view().node(lazy).unwrap();
///     read.data_source().as_identifier().unwrap().text()
/// }
/// ```
pub struct NodeRead<'a> {
    record: ReadRecord<'a>,
    id: NodeId,
    owner_id: FileId,
}
enum ReadRecord<'a> {
    Core {
        header: &'a StoredNode,
        context: CompactContext<'a>,
    },
    Owned {
        node: &'a Node,
        source: &'a SourceText,
    },
    Lazy {
        record: StorageRead<'a, AstStorageData>,
        source: &'a SourceText,
    },
}
impl std::fmt::Debug for NodeRead<'_> {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.debug_struct("NodeRead")
            .field("id", &self.id)
            .field("owner_id", &self.owner_id)
            .field("kind", &self.kind())
            .field("shape", &self.data().name())
            .finish_non_exhaustive()
    }
}
impl<'a> NodeRead<'a> {
    #[inline]
    pub(crate) fn resolved(
        id: NodeId,
        record: &StorageRead<'a, StoredNode>,
        owner: StorageView<'a, StoredNode>,
    ) -> Self {
        if id.arena() == owner.id().arena() {
            Self::core(
                id,
                owner.id(),
                record
                    .as_borrowed()
                    .expect("core headers borrow their owner"),
                CompactContext {
                    nodes: owner.id().arena(),
                    auxiliary: owner.auxiliary_arena(),
                    source: owner.source(),
                    store: owner.store(),
                },
            )
        } else {
            Self::resolved_lazy(id, record, owner)
        }
    }
    #[cold]
    #[inline(never)]
    fn resolved_lazy(
        id: NodeId,
        record: &StorageRead<'a, StoredNode>,
        owner: StorageView<'a, StoredNode>,
    ) -> Self {
        let aux = AuxId::from_parts(owner.lazy_auxiliary_arena(), record.ordinal)
            .expect("lazy payload slot");
        Self {
            record: ReadRecord::Lazy {
                record: owner.aux(aux).expect("published lazy payload"),
                source: owner.source(),
            },
            id,
            owner_id: owner.id(),
        }
    }
    #[inline]
    pub(crate) fn core(
        id: NodeId,
        owner_id: FileId,
        header: &'a StoredNode,
        context: CompactContext<'a>,
    ) -> Self {
        Self {
            record: ReadRecord::Core { header, context },
            id,
            owner_id,
        }
    }
    pub(crate) fn owned(
        id: NodeId,
        node: &'a Node,
        owner_id: FileId,
        source: &'a SourceText,
    ) -> Self {
        Self {
            record: ReadRecord::Owned { node, source },
            id,
            owner_id,
        }
    }
    pub fn id(&self) -> NodeId {
        self.id
    }
    pub fn owner_id(&self) -> FileId {
        self.owner_id
    }
    pub fn source(&self) -> &'a SourceText {
        match self.record {
            ReadRecord::Core { context, .. } => context.source,
            ReadRecord::Owned { source, .. } | ReadRecord::Lazy { source, .. } => source,
        }
    }
    /// A directly borrowed read can retain its owner's lifetime. A directory
    /// resolved lazy read must stay bounded by its publication guard instead.
    pub fn as_borrowed(&self) -> Option<Self> {
        let record = match self.record {
            ReadRecord::Core { header, context } => ReadRecord::Core { header, context },
            ReadRecord::Owned { node, source } => ReadRecord::Owned { node, source },
            ReadRecord::Lazy { .. } => return None,
        };
        Some(Self {
            record,
            id: self.id,
            owner_id: self.owner_id,
        })
    }
    fn owned_record(&self) -> Option<&Node> {
        match &self.record {
            ReadRecord::Owned { node, .. } => Some(node),
            ReadRecord::Lazy { record, .. } => match &**record {
                AstStorageData::FallbackNode(node) => Some(node),
                _ => unreachable!("lazy header names its owned payload"),
            },
            ReadRecord::Core { .. } => None,
        }
    }
    /// Explicit construction copy used by unrestricted exclusive edits and cold
    /// published overlays; ordinary payload reads never reconstruct a node.
    pub(crate) fn to_owned_preserving_identity(&self) -> Node {
        let facts = match self.record {
            ReadRecord::Core { context, .. } => context
                .store
                .parked_facts(self.id.slot())
                .unwrap_or_else(|| self.cached_subtree_facts()),
            _ => self.cached_subtree_facts(),
        };
        Node {
            kind: self.kind(),
            parent: self.parent(),
            flags: self.flags(),
            pos: self.pos(),
            end: self.end(),
            data: self.data().to_owned(),
            subtree_facts: std::sync::atomic::AtomicU32::new(facts),
            runtime_id: std::sync::atomic::AtomicU64::new(crate::existing_runtime_node_id(self)),
        }
    }
    pub(crate) fn copy_for_binding(&self) -> Node {
        let mut node = self.to_owned_preserving_identity();
        node.runtime_id = std::sync::atomic::AtomicU64::new(crate::runtime_node_id(self));
        node
    }
    pub fn kind(&self) -> crate::NodeKind {
        match self.record {
            ReadRecord::Core { header, .. } => header.kind,
            _ => self.owned_record().unwrap().kind(),
        }
    }
    pub fn parent(&self) -> Option<NodeId> {
        match self.record {
            ReadRecord::Core { header, context } => {
                context.decode_node(FieldKey::parent(self.id.slot()), header.parent)
            }
            _ => self.owned_record().unwrap().parent(),
        }
    }
    pub fn flags(&self) -> u32 {
        match self.record {
            ReadRecord::Core { header, .. } => header.flags,
            _ => self.owned_record().unwrap().flags(),
        }
    }
    pub fn pos(&self) -> i32 {
        match self.record {
            ReadRecord::Core { header, .. } => header.pos,
            _ => self.owned_record().unwrap().pos(),
        }
    }
    pub fn end(&self) -> i32 {
        match self.record {
            ReadRecord::Core { header, .. } => header.end,
            _ => self.owned_record().unwrap().end(),
        }
    }
    pub fn range(&self) -> ts_core::TextRange {
        ts_core::TextRange::new(i64::from(self.pos()), i64::from(self.end()))
    }
    pub fn data(&self) -> NodeDataRead<'_> {
        match self.record {
            ReadRecord::Core { header, context } => context.store.payloads.read(
                header.actual_shape(),
                header.ordinal,
                context,
                header.end,
            ),
            _ => NodeDataRead::from_owned(self.owned_record().unwrap().data()),
        }
    }
    pub fn data_source(&self) -> NodeDataSource<'_> {
        match &self.record {
            ReadRecord::Core { header, context } => NodeDataSource::from_stored(header, context),
            _ => NodeDataSource::from_owned(self.owned_record().unwrap().data()),
        }
    }
    pub(crate) fn inline_binding(&self) -> Option<crate::NodeBinding> {
        match self.record {
            ReadRecord::Core { header, context } => {
                context.store.node_binding(header, self.id.slot(), context)
            }
            _ => None,
        }
    }
    pub(crate) fn inline_symbol(&self) -> Option<crate::SymbolId> {
        match self.record {
            ReadRecord::Core { header, context } => {
                context.store.node_symbol(header, self.id.slot(), context)
            }
            _ => None,
        }
    }
    pub(crate) fn inline_locals(&self) -> Option<crate::SymbolTableId> {
        match self.record {
            ReadRecord::Core { header, context } => {
                context.store.node_locals(header, self.id.slot(), context)
            }
            _ => None,
        }
    }
    pub(crate) fn inline_flow(&self) -> Option<crate::FlowId> {
        match self.record {
            ReadRecord::Core { header, context } => {
                context.store.node_flow(header, self.id.slot(), context)
            }
            _ => None,
        }
    }
    pub fn for_each_child(
        &self,
        visitor: &mut impl crate::ChildVisitor,
    ) -> std::ops::ControlFlow<()> {
        self.for_each_child_generated(visitor)
    }
    pub(crate) fn cached_subtree_facts(&self) -> u32 {
        match self.record {
            ReadRecord::Core { header, context } => context
                .store
                .payloads
                .cached_subtree_facts(header.actual_shape(), header.ordinal),
            _ => self.owned_record().unwrap().cached_subtree_facts(),
        }
    }
    pub(crate) fn store_subtree_facts(&self, facts: u32) {
        match self.record {
            ReadRecord::Core { header, context } => context.store.payloads.store_subtree_facts(
                header.actual_shape(),
                header.ordinal,
                facts,
            ),
            _ => self.owned_record().unwrap().store_subtree_facts(facts),
        }
    }
}
impl crate::NodeAccess for NodeRead<'_> {
    fn kind(&self) -> crate::NodeKind {
        self.kind()
    }
    fn parent(&self) -> Option<NodeId> {
        self.parent()
    }
    fn flags(&self) -> u32 {
        self.flags()
    }
    fn pos(&self) -> i32 {
        self.pos()
    }
    fn end(&self) -> i32 {
        self.end()
    }
    fn range(&self) -> ts_core::TextRange {
        self.range()
    }
    fn data(&self) -> NodeDataRead<'_> {
        self.data()
    }
    fn data_source(&self) -> NodeDataSource<'_> {
        self.data_source()
    }
    fn cached_subtree_facts(&self) -> u32 {
        self.cached_subtree_facts()
    }
    fn store_subtree_facts(&self, facts: u32) {
        self.store_subtree_facts(facts);
    }
    fn existing_runtime_id(&self) -> u64 {
        match self.record {
            ReadRecord::Core { context, .. } => context.store.existing_runtime_id(self.id.slot()),
            _ => crate::runtime_id::owned_existing_runtime_node_id(self.owned_record().unwrap()),
        }
    }
    fn runtime_id(&self) -> u64 {
        match self.record {
            ReadRecord::Core { context, .. } => context.store.runtime_id(self.id.slot()),
            _ => crate::runtime_id::owned_runtime_node_id(self.owned_record().unwrap()),
        }
    }
}
#[cfg(test)]
mod tests;
