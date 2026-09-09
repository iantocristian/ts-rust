use crate::compact::{CompactContext, FieldKey, StoredNode};
use crate::{
    AstStorageData, FactoryHooks, FileInfo, JSDocRoots, JsString, Node, NodeData, NodeId, NodeList,
    NodeListId, NodeListRead, NodeRead, NodeSlice, NodeSliceRead, TextSlice, TextSliceRead,
};
use crate::{NodeDataRead, NodeMut};
use std::sync::Arc;
use ts_arena::{
    AuxId, Counters, Error, StorageBuilder, StorageHandle, StorageRead, StorageTransaction,
    StorageView,
};
use ts_core::TextRange;
use ts_jsstring::SourceText;

/// Exclusive syntax construction. Hooks exist only during this exclusive phase.
pub struct AstBuilder {
    pub(crate) storage: StorageBuilder<StoredNode>,
    pub(crate) hooks: Option<Arc<dyn FactoryHooks>>,
    frame: AuxId,
}
impl std::fmt::Debug for AstBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.storage.fmt(f)
    }
}
impl AstBuilder {
    pub fn new(source: SourceText, counters: &Counters) -> Self {
        Self::from_hooks(source, counters, None)
    }
    pub fn with_hooks(
        source: SourceText,
        counters: &Counters,
        hooks: Arc<dyn FactoryHooks>,
    ) -> Self {
        Self::from_hooks(source, counters, Some(hooks))
    }
    fn from_hooks(
        source: SourceText,
        counters: &Counters,
        hooks: Option<Arc<dyn FactoryHooks>>,
    ) -> Self {
        let mut storage = StorageBuilder::from_source_text(source, counters);
        let frame = storage.push_aux(AstStorageData::File(FileInfo::default()));
        storage
            .set_metadata(frame)
            .expect("new file frame belongs to core storage");
        Self {
            storage,
            hooks,
            frame,
        }
    }
    pub fn view(&self) -> AstView<'_> {
        AstView(self.storage.view(), None)
    }
    pub fn id(&self) -> ts_arena::FileId {
        self.storage.id()
    }
    /// Retain a published dependency before storing any of its identities.
    /// Importing a mapped member retains its complete bundle and dependencies.
    pub fn retain_file(&mut self, file: AstFile) {
        self.storage.retain_file(file.0);
    }
    pub fn node_mut(&mut self, id: NodeId) -> Result<NodeMut<'_>, Error> {
        let header = self.storage.core_node(id)?;
        let value = NodeRead::resolved(id, &StorageRead::borrowed(header), self.storage.view())
            .to_owned_preserving_identity();
        let auxiliary = self.storage.view().auxiliary_arena();
        let (header, store, source) = self.storage.node_store_and_source_mut(id)?;
        Ok(NodeMut::core(value, header, store, source, id, auxiliary))
    }
    pub(crate) fn push_node(&mut self, node: Node) -> NodeId {
        let nodes = self.id().arena();
        let auxiliary = self.storage.view().auxiliary_arena();
        let (store, source) = self.storage.store_and_source_mut();
        let (payloads, mut context) = store.packing_parts(nodes, auxiliary, source, node.end());
        let (shape, ordinal) = payloads.insert(node.data, &mut context);
        let id = self.storage.push(StoredNode {
            kind: node.kind,
            flags: node.flags,
            pos: node.pos,
            end: node.end,
            parent: 0,
            shape,
            ordinal,
        });
        if node.parent.is_some() {
            self.write_parent(id, node.parent).expect("new core header");
        }
        id
    }
    pub(crate) fn write_parent(&mut self, id: NodeId, parent: Option<NodeId>) -> Result<(), Error> {
        let auxiliary = self.storage.view().auxiliary_arena();
        let (header, store, source) = self.storage.node_store_and_source_mut(id)?;
        let (_, mut context) = store.packing_parts(id.arena(), auxiliary, source, header.end);
        header.parent = context.encode_node(FieldKey::parent(id.slot()), parent);
        Ok(())
    }
    pub(crate) fn finish_header(
        &mut self,
        id: NodeId,
        range: TextRange,
        flags: u32,
        replace_flags: bool,
    ) -> Result<(), Error> {
        let auxiliary = self.storage.view().auxiliary_arena();
        let (header, store, source) = self.storage.node_store_and_source_mut(id)?;
        let new_end = range.end() as i32;
        if header.end != new_end {
            let (payloads, mut context) =
                store.packing_parts(id.arena(), auxiliary, source, new_end);
            payloads.change_text_end(
                header.actual_shape(),
                header.ordinal,
                header.end,
                new_end,
                &mut context,
            );
        }
        header.pos = range.pos() as i32;
        header.end = new_end;
        header.flags = if replace_flags {
            flags
        } else {
            header.flags | flags
        };
        Ok(())
    }
    pub(crate) fn frame_mut(&mut self) -> &mut FileInfo {
        match self.storage.aux_mut(self.frame).expect("core file frame") {
            AstStorageData::File(info) => info,
            _ => unreachable!("file frame record kind"),
        }
    }
    pub fn node_count(&self) -> i64 {
        self.view().file_info().node_count
    }
    pub fn text_count(&self) -> i64 {
        self.view().file_info().text_count
    }
    #[allow(clippy::needless_pass_by_value)] // Construction transfers and releases its temporary vector.
    pub fn node_slice(&mut self, nodes: Vec<Option<NodeId>>) -> Result<NodeSlice, Error> {
        for &id in nodes.iter().flatten() {
            self.view().node(id)?;
        }
        let len = checked_len(nodes.len())?;
        let owner = self.id().arena();
        let compact = self.storage.store_mut().edges.append(owner, &nodes)?;
        let backing = self.storage.push_aux(AstStorageData::CompactNodes(compact));
        Ok(NodeSlice {
            backing: Some(backing),
            start: 0,
            len,
        })
    }
    pub fn text_slice(&mut self, text: Vec<JsString>) -> Result<TextSlice, Error> {
        let len = checked_len(text.len())?;
        let backing = self
            .storage
            .push_aux(AstStorageData::Text(text.into_boxed_slice()));
        Ok(TextSlice {
            backing: Some(backing),
            start: 0,
            len,
        })
    }
    pub fn new_list(&mut self, loc: TextRange, nodes: NodeSlice) -> Result<NodeListId, Error> {
        self.view().node_slice(nodes)?;
        Ok(NodeListId(
            self.storage
                .push_aux(AstStorageData::List(NodeList::new(loc, nodes))),
        ))
    }
    pub fn clone_list(&mut self, list: NodeListId) -> Result<NodeListId, Error> {
        let header = self.view().list(list)?.clone();
        Ok(NodeListId(
            self.storage.push_aux(AstStorageData::List(header)),
        ))
    }
    pub fn list_mut(&mut self, id: NodeListId) -> Result<&mut NodeList, Error> {
        match self.storage.aux_mut(id.0)? {
            AstStorageData::List(list) => Ok(list),
            _ => Err(Error::InvalidGraph),
        }
    }
    pub fn set_list_nodes(&mut self, id: NodeListId, nodes: NodeSlice) -> Result<(), Error> {
        self.view().node_slice(nodes)?;
        self.list_mut(id)?.set_nodes(nodes);
        Ok(())
    }
    pub fn mark_list_missing(&mut self, id: NodeListId) -> Result<(), Error> {
        self.list_mut(id)?.set_missing(true);
        Ok(())
    }
    pub fn seed_jsdoc(&mut self, parent: NodeId, roots: Vec<NodeId>) -> Result<(), Error> {
        self.storage.seed_jsdoc(parent, roots)
    }
    pub fn seed_source_jsdoc(
        &mut self,
        source: NodeId,
        parent: NodeId,
        roots: Vec<NodeId>,
    ) -> Result<(), Error> {
        self.view().source_file(source)?;
        self.storage.seed_source_jsdoc(source, parent, roots)
    }
    /// Complete a source-file or fragment parse without freezing binder mutation.
    pub fn complete(mut self, root: NodeId) -> Result<ParsedFile, Error> {
        self.view().node(root)?;
        self.frame_mut().root = Some(root);
        self.view().validate_core()?;
        Ok(ParsedFile {
            builder: self,
            validated: true,
        })
    }
}

/// Parse completion keeps exclusive core mutation available to the later binder.
/// The whole value can move to a worker; none of its views can escape that move.
///
/// ```compile_fail
/// use ts_ast::{AstBuilder, FactoryMethods, NodeRead, SyntaxKind};
/// fn escaped() -> NodeRead<'static> {
///     let mut builder = AstBuilder::new(ts_jsstring::SourceText::from_loaded_bytes(&b""[..]), &ts_arena::Counters::new());
///     let root = builder.new_token(SyntaxKind::Unknown.into());
///     builder.view().node(root).unwrap()
/// }
/// ```
///
/// ```compile_fail
/// use ts_ast::{AstBuilder, FactoryMethods, SyntaxKind};
/// let mut builder = AstBuilder::new(ts_jsstring::SourceText::from_loaded_bytes(&b""[..]), &ts_arena::Counters::new());
/// let root = builder.new_token(SyntaxKind::Unknown.into());
/// let published = builder.complete(root).unwrap().publish_unbound();
/// published.node_mut(root).unwrap().set_flags(1);
/// ```
#[derive(Debug)]
pub struct ParsedFile {
    builder: AstBuilder,
    validated: bool,
}
impl ParsedFile {
    pub(crate) fn initialize_binding_storage(
        &mut self,
        arenas: crate::compact::binding::BindingArenas,
    ) {
        self.builder.storage.store_mut().initialize_binding(arenas);
    }
    pub(crate) fn write_binding_field(
        &mut self,
        id: NodeId,
        write: crate::compact::binding::BindingWrite,
    ) -> Result<bool, Error> {
        let auxiliary = self.builder.storage.view().auxiliary_arena();
        let (header, store, source) = self.builder.storage.node_store_and_source_mut(id)?;
        Ok(store.write_binding(header, id, auxiliary, source, write))
    }
    pub(crate) fn exclusive_core_only(&self) -> bool {
        self.builder.hooks.is_none() && self.builder.storage.is_core_only()
    }
    pub(crate) fn core_node(&self, id: NodeId) -> Result<&StoredNode, Error> {
        self.builder.storage.core_node(id)
    }
    pub(crate) fn core_node_read(&self, id: NodeId) -> Result<NodeRead<'_>, Error> {
        let record = self.builder.storage.core_node(id)?;
        let owner = self.builder.storage.view();
        Ok(NodeRead::core(
            id,
            owner.id(),
            record,
            CompactContext {
                nodes: owner.id().arena(),
                auxiliary: owner.auxiliary_arena(),
                source: owner.source(),
                store: owner.store(),
            },
        ))
    }
    pub(crate) fn set_node_flags(&mut self, id: NodeId, flags: u32) -> Result<(), Error> {
        self.builder.storage.node_mut(id)?.set_flags(flags);
        // Do not restore a proof invalidated by an earlier unrestricted edit.
        Ok(())
    }
    pub fn view(&self) -> AstView<'_> {
        self.builder.view()
    }
    pub fn builder_mut(&mut self) -> &mut AstBuilder {
        self.validated = false;
        &mut self.builder
    }
    pub fn root(&self) -> NodeId {
        self.view().file_info().root.expect("completed parse root")
    }
    /// Parser-tool publication deliberately makes no claim that binding ran.
    pub fn publish_unbound(self) -> AstFile {
        self.try_publish_unbound()
            .expect("published AST references belong to retained storage")
    }
    /// Revalidate every core record and auxiliary edge after exclusive mutations.
    /// A freshly completed parse carries its validation through publication;
    /// requesting its mutable builder invalidates that proof conservatively.
    pub fn try_publish_unbound(self) -> Result<AstFile, Error> {
        if !self.validated {
            self.view().validate_core()?;
        }
        Ok(AstFile(self.builder.storage.finish()))
    }
    /// Publish a mapped group under one retention root, without asserting binding.
    pub fn publish_bundle_unbound(self, supplemental: Vec<Self>) -> AstBundle {
        self.try_publish_bundle_unbound(supplemental)
            .expect("published bundle references belong to retained members")
    }
    pub fn try_publish_bundle_unbound(self, supplemental: Vec<Self>) -> Result<AstBundle, Error> {
        let mut members = vec![&self.builder.storage];
        members.extend(supplemental.iter().map(|file| &file.builder.storage));
        StorageBuilder::with_group_views(&members, |views| {
            for view in views {
                AstView(*view, None).validate_core()?;
            }
            Ok(())
        })?;
        Ok(AstBundle(ts_arena::StorageBundle::new(
            self.builder.storage,
            supplemental
                .into_iter()
                .map(|file| file.builder.storage)
                .collect(),
        )))
    }
}

#[derive(Clone, Debug)]
pub struct AstBundle(Arc<ts_arena::StorageBundle<StoredNode>>);
impl AstBundle {
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn file(&self, index: usize) -> Option<AstFile> {
        self.0.file(index).map(AstFile)
    }
}

/// An explicitly retained file or mapped bundle. Nodes, list backing and the file
/// frame remain below this storage root, including after a node escapes its scope.
/// Raw arena writers cannot bypass AST validation after publication.
///
/// ```compile_fail
/// fn raw_storage(file: &ts_ast::AstFile) {
///     let _: &ts_arena::StorageHandle<ts_ast::Node> = file.storage();
/// }
/// ```
///
/// Generic arena storage has no AST graph-validation guarantee and cannot be
/// adopted as a published AST file.
///
/// ```compile_fail
/// fn unchecked_adoption(storage: ts_arena::StorageHandle<ts_ast::Node>) {
///     let _ = ts_ast::AstFile::from_storage(storage);
/// }
/// ```
#[derive(Clone, Debug)]
pub struct AstFile(pub(crate) StorageHandle<StoredNode>);
impl AstFile {
    pub fn view(&self) -> AstView<'_> {
        AstView(self.0.view(), None)
    }
    pub fn root(&self) -> Option<NodeId> {
        self.view().file_info().root
    }
    pub fn retain_node(&self, id: NodeId) -> Result<RetainedNode, Error> {
        let record = self.0.resolved_node(id)?.retain();
        let owner = self.0.view().for_arena(id.arena())?;
        let fallback = if id.arena() == owner.id().arena() {
            None
        } else {
            let aux = AuxId::from_parts(owner.lazy_auxiliary_arena(), record.ordinal)?;
            match &*owner.aux(aux)? {
                AstStorageData::FallbackNode(node) => Some(node.clone()),
                _ => return Err(Error::InvalidGraph),
            }
        };
        Ok(RetainedNode { record, fallback })
    }
    /// Follow a mapped-file or imported-file identity within this retention root.
    pub fn file(&self, id: ts_arena::FileId) -> Option<Self> {
        self.0.file(id).map(Self)
    }
}

/// An escaped AST node retaining its complete file/bundle and dependencies.
/// The private arena handle cannot expose unchecked lazy publication.
///
/// ```compile_fail
/// fn raw_owner(node: &ts_ast::RetainedNode) {
///     let _ = node.owner();
/// }
/// ```
#[derive(Clone, Debug)]
pub struct RetainedNode {
    record: ts_arena::RetainedRecord<StoredNode>,
    fallback: Option<Arc<Node>>,
}
impl RetainedNode {
    pub fn id(&self) -> NodeId {
        self.record.id()
    }
    /// Escaped lazy payloads already retain their stable backing; this read
    /// never reacquires a lazy publication lock or increments a reference count.
    pub fn read(&self) -> NodeRead<'_> {
        let owner = self
            .record
            .owner()
            .view()
            .for_arena(self.id().arena())
            .expect("retained node owner");
        if let Some(node) = &self.fallback {
            NodeRead::owned(self.id(), node, owner.id(), owner.source())
        } else {
            NodeRead::resolved(self.id(), &StorageRead::borrowed(&self.record), owner)
        }
    }
    pub fn file(&self) -> AstFile {
        AstFile(self.record.owner().clone())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AstView<'a>(
    pub(crate) StorageView<'a, StoredNode>,
    pub(crate) Option<&'a crate::BindResult>,
);
impl<'a> AstView<'a> {
    /// Development-only layout inventory. Excluded from normal production builds.
    #[cfg(feature = "layout-profile")]
    pub fn layout_profile(self) -> std::collections::BTreeMap<&'static str, usize> {
        let mut counts = std::collections::BTreeMap::new();
        let context = CompactContext {
            nodes: self.0.id().arena(),
            auxiliary: self.0.auxiliary_arena(),
            source: self.source(),
            store: self.0.store(),
        };
        for header in self.0.core_nodes() {
            let data = context.store.payloads.read(
                header.actual_shape(),
                header.ordinal,
                context,
                header.end,
            );
            *counts.entry(data.name()).or_default() += 1;
        }
        counts
    }
    pub fn for_node_owner(self, node: NodeId) -> Result<Self, Error> {
        self.0.for_node_owner(node).map(|view| Self(view, self.1))
    }
    pub fn source(self) -> &'a SourceText {
        self.0.source()
    }
    pub fn position_map(self) -> &'a ts_jsstring::PositionMap {
        self.0.position_map()
    }
    pub fn node(self, id: NodeId) -> Result<NodeRead<'a>, Error> {
        if let Some(node) = self
            .binding_for_node(id)?
            .and_then(|result| result.overlay(id))
        {
            // The overlay's identity was validated when it was inserted. Select
            // its retained owner without reading the parsed record or locking
            // its lazy directory a second time.
            let owner = self.0.for_arena(id.arena())?;
            return Ok(NodeRead::owned(id, node, owner.id(), owner.source()));
        }
        let (record, owner) = self.0.node_with_owner(id)?;
        Ok(NodeRead::resolved(id, &record, owner))
    }
    pub(crate) fn binding_for_node(
        self,
        id: NodeId,
    ) -> Result<Option<&'a crate::BindResult>, Error> {
        let Some(active) = self.1 else {
            return Ok(None);
        };
        if active.is_single_source_owner(id) {
            return Ok(Some(active));
        }
        let source = match self.owning_source(id) {
            Ok(source) => source,
            // A general AST read may inspect a parentless synthetic node. Only
            // bound retention requires an unambiguous logical source.
            Err(Error::InvalidGraph) => return Ok(None),
            Err(error) => return Err(error),
        };
        if source == active.source() {
            return Ok(Some(active));
        }
        let parsed = AstView(self.0, None);
        Ok(parsed.source_file(source)?.state_ref().binding.result())
    }
    /// Resolve a logical source through the immutable syntax parent chain. Do
    /// not guess an arena's canonical root for orphans or parent cycles.
    pub(crate) fn owning_source(self, id: NodeId) -> Result<NodeId, Error> {
        let mut slow = Some(id);
        let mut fast = Some(id);
        loop {
            let current = slow.ok_or(Error::InvalidGraph)?;
            let node = AstView(self.0, None).node(current)?;
            if node.kind() == crate::SyntaxKind::SourceFile {
                AstView(self.0, None).source_file(current)?;
                return Ok(current);
            }
            slow = node.parent();
            for _ in 0..2 {
                fast = match fast {
                    Some(id) => {
                        let node = AstView(self.0, None).node(id)?;
                        if node.kind() == crate::SyntaxKind::SourceFile {
                            None
                        } else {
                            node.parent()
                        }
                    }
                    None => None,
                };
            }
            if slow.is_some() && slow == fast {
                return Err(Error::InvalidGraph);
            }
        }
    }
    pub fn file_info(self) -> FileInfo {
        let id = self.0.metadata().expect("AST storage has a file frame");
        match &*self.0.aux(id).expect("core file frame") {
            AstStorageData::File(info) => *info,
            _ => unreachable!("file frame record kind"),
        }
    }
    pub fn list(self, id: NodeListId) -> Result<NodeListRead<'a>, Error> {
        list_read(self.0.aux(id.0)?)
    }
    // port: tsc/internal/ast/ast.go:NodeList.HasTrailingComma
    pub fn list_has_trailing_comma(self, id: NodeListId) -> Result<bool, Error> {
        let list = self.list(id)?;
        let nodes = self.node_slice(list.nodes())?;
        let Some(last) = nodes.last() else {
            return Ok(false);
        };
        let last = last.expect("nil last node in NodeList.HasTrailingComma");
        Ok(i64::from(self.node(last)?.end()) < list.loc().end())
    }
    pub fn node_slice(self, nodes: NodeSlice) -> Result<NodeSliceRead<'a>, Error> {
        let (record, compact) = match nodes.backing {
            Some(id) => {
                let (record, owner) = self.0.aux_with_owner(id)?;
                (
                    Some(record),
                    Some((&owner.store().edges, owner.id().arena())),
                )
            }
            None => (None, None),
        };
        node_slice_read(nodes, record, compact)
    }
    pub fn text_slice(self, text: TextSlice) -> Result<TextSliceRead<'a>, Error> {
        text_slice_read(text, text.backing.map(|id| self.0.aux(id)).transpose()?)
    }
    pub fn eager_jsdoc(self, parent: NodeId) -> Result<Option<JSDocRoots>, Error> {
        Ok(self.0.eager_jsdoc(parent)?.map(JSDocRoots::from_shared))
    }
    pub fn source_eager_jsdoc(
        self,
        source: NodeId,
        parent: NodeId,
    ) -> Result<Option<JSDocRoots>, Error> {
        self.source_file(source)?;
        Ok(self
            .0
            .source_eager_jsdoc(source, parent)?
            .map(JSDocRoots::from_shared))
    }
    pub fn source_jsdoc(
        self,
        source: NodeId,
        parent: NodeId,
        initialize: impl FnOnce(&mut AstTransaction<'_, '_>) -> Result<Vec<NodeId>, Error>,
    ) -> Result<JSDocRoots, Error> {
        self.source_file(source)?;
        self.0
            .source_jsdoc(source, parent, |storage| {
                let mut transaction = AstTransaction {
                    storage,
                    node_count: 0,
                    text_count: 0,
                };
                let roots = initialize(&mut transaction)?;
                transaction.validate_staged()?;
                Ok(roots)
            })
            .map(JSDocRoots::from_shared)
    }
    /// Call this on the chosen execution thread before any lazy lock is acquired.
    /// The initializer may use only its transaction to read this file's lazy data.
    pub fn jsdoc(
        self,
        parent: NodeId,
        initialize: impl FnOnce(&mut AstTransaction<'_, '_>) -> Result<Vec<NodeId>, Error>,
    ) -> Result<JSDocRoots, Error> {
        self.0
            .jsdoc(parent, |storage| {
                let mut transaction = AstTransaction {
                    storage,
                    node_count: 0,
                    text_count: 0,
                };
                let roots = initialize(&mut transaction)?;
                transaction.validate_staged()?;
                Ok(roots)
            })
            .map(JSDocRoots::from_shared)
    }
    pub(crate) fn validate_data(self, data: &NodeData) -> Result<(), Error> {
        validate_data(
            data,
            |id| self.node(id).map(|_| ()),
            |id| self.list(id).map(|_| ()),
            |slice| self.node_slice(slice).map(|_| ()),
            |slice| self.text_slice(slice).map(|_| ()),
        )
    }
    fn validate_read_data(self, data: NodeDataRead<'_>) -> Result<(), Error> {
        data.validate_references(
            |id| self.node(id).map(|_| ()),
            |id| self.list(id).map(|_| ()),
            |slice| self.node_slice(slice).map(|_| ()),
            |slice| self.text_slice(slice).map(|_| ()),
        )
    }
    fn validate_core(self) -> Result<(), Error> {
        for (index, header) in self.0.core_nodes().enumerate() {
            let id = NodeId::from_parts(
                self.0.id().arena(),
                u32::try_from(index + 1).map_err(|_| Error::InvalidSlot)?,
            )?;
            let node = NodeRead::resolved(id, &StorageRead::borrowed(header), self.0);
            if let Some(parent) = node.parent() {
                self.node(parent)?;
            }
            self.validate_read_data(node.data())?;
        }
        for value in self.0.core_auxiliary() {
            match value {
                AstStorageData::List(list) => {
                    self.node_slice(list.nodes())?;
                }
                AstStorageData::Nodes(nodes) => {
                    for &node in nodes.iter().flatten() {
                        self.node(node)?;
                    }
                }
                AstStorageData::CompactNodes(backing) => {
                    let edges = &self.0.store().edges;
                    let range = backing.start
                        ..backing
                            .start
                            .checked_add(backing.len as usize)
                            .ok_or(Error::InvalidGraph)?;
                    if !edges.valid_range(range.clone()) {
                        return Err(Error::InvalidGraph);
                    }
                    for node in edges.iter(self.0.id().arena(), range).flatten() {
                        self.node(node)?;
                    }
                }
                AstStorageData::FallbackNode(_) => return Err(Error::InvalidGraph),
                AstStorageData::SourceMetadata(data) => data.validate(self)?,
                AstStorageData::Text(_) => {}
                AstStorageData::File(info) => {
                    if let Some(root) = info.root {
                        self.node(root)?;
                    }
                    if let Some(map) = info.source_files {
                        if !matches!(&*self.0.aux(map)?, AstStorageData::SourceFiles(_)) {
                            return Err(Error::InvalidGraph);
                        }
                    }
                }
                AstStorageData::SourceFiles(files) => {
                    for (&node, source) in files {
                        self.node(node)?;
                        source.validate_references(self)?;
                    }
                }
            }
        }
        Ok(())
    }
}

/// A borrowed transaction is never sent to a worker or used to reacquire its lock.
pub struct AstTransaction<'a, 'storage> {
    pub(crate) storage: &'a mut StorageTransaction<'storage, StoredNode>,
    pub(crate) node_count: i64,
    pub(crate) text_count: i64,
}
impl AstTransaction<'_, '_> {
    pub fn node(&self, id: NodeId) -> Result<NodeRead<'_>, Error> {
        let header = self.storage.node(id)?;
        if id.arena() == self.storage.owner_id().arena() {
            Ok(NodeRead::core(
                id,
                self.storage.owner_id(),
                header,
                CompactContext {
                    nodes: self.storage.owner_id().arena(),
                    auxiliary: self.storage.core_auxiliary_arena(),
                    source: self.storage.source(),
                    store: self.storage.store(),
                },
            ))
        } else {
            let aux = AuxId::from_parts(self.storage.lazy_auxiliary_arena(), header.ordinal)?;
            match self.storage.aux(aux)? {
                AstStorageData::FallbackNode(node) => Ok(NodeRead::owned(
                    id,
                    node,
                    self.storage.owner_id(),
                    self.storage.source(),
                )),
                _ => Err(Error::InvalidGraph),
            }
        }
    }
    pub fn node_mut(&mut self, id: NodeId) -> Result<NodeMut<'_>, Error> {
        let ordinal = self.storage.node_mut(id)?.ordinal;
        let aux = AuxId::from_parts(self.storage.lazy_auxiliary_arena(), ordinal)?;
        let (header, payload) = self.storage.node_and_aux_mut(id, aux)?;
        match payload {
            AstStorageData::FallbackNode(node) => Ok(NodeMut::lazy(
                Arc::get_mut(node).expect("unpublished payload is exclusive"),
                header,
            )),
            _ => Err(Error::InvalidGraph),
        }
    }
    pub(crate) fn push_node(&mut self, node: Node) -> NodeId {
        let header = Self::stage_node(self.storage, node);
        self.storage.push(header)
    }
    fn stage_node(storage: &mut StorageTransaction<'_, StoredNode>, node: Node) -> StoredNode {
        let mut header = StoredNode::fallback(&node, 0);
        let aux = storage.push_aux(AstStorageData::FallbackNode(Arc::new(node)));
        header.ordinal = aux.slot();
        header
    }
    pub fn list(&self, id: NodeListId) -> Result<NodeListRead<'_>, Error> {
        list_read(StorageRead::borrowed(self.storage.aux(id.0)?))
    }
    pub fn node_slice_read(&self, nodes: NodeSlice) -> Result<NodeSliceRead<'_>, Error> {
        node_slice_read(
            nodes,
            nodes
                .backing
                .map(|id| self.storage.aux(id).map(StorageRead::borrowed))
                .transpose()?,
            Some((&self.storage.store().edges, self.storage.owner_id().arena())),
        )
    }
    pub fn text_slice_read(&self, text: TextSlice) -> Result<TextSliceRead<'_>, Error> {
        text_slice_read(
            text,
            text.backing
                .map(|id| self.storage.aux(id).map(StorageRead::borrowed))
                .transpose()?,
        )
    }
    pub fn node_slice(&mut self, nodes: Vec<Option<NodeId>>) -> Result<NodeSlice, Error> {
        for &id in nodes.iter().flatten() {
            self.storage.node(id)?;
        }
        let len = checked_len(nodes.len())?;
        let backing = self
            .storage
            .push_aux(AstStorageData::Nodes(nodes.into_boxed_slice()));
        Ok(NodeSlice {
            backing: Some(backing),
            start: 0,
            len,
        })
    }
    pub fn text_slice(&mut self, text: Vec<JsString>) -> Result<TextSlice, Error> {
        let len = checked_len(text.len())?;
        let backing = self
            .storage
            .push_aux(AstStorageData::Text(text.into_boxed_slice()));
        Ok(TextSlice {
            backing: Some(backing),
            start: 0,
            len,
        })
    }
    pub fn new_list(&mut self, loc: TextRange, nodes: NodeSlice) -> Result<NodeListId, Error> {
        self.node_slice_read(nodes)?;
        Ok(NodeListId(
            self.storage
                .push_aux(AstStorageData::List(NodeList::new(loc, nodes))),
        ))
    }
    pub fn list_mut(&mut self, id: NodeListId) -> Result<&mut NodeList, Error> {
        match self.storage.aux_mut(id.0)? {
            AstStorageData::List(list) => Ok(list),
            _ => Err(Error::InvalidGraph),
        }
    }
    pub fn set_list_nodes(&mut self, id: NodeListId, nodes: NodeSlice) -> Result<(), Error> {
        self.node_slice_read(nodes)?;
        self.list_mut(id)?.set_nodes(nodes);
        Ok(())
    }
    pub fn mark_list_missing(&mut self, id: NodeListId) -> Result<(), Error> {
        self.list_mut(id)?.set_missing(true);
        Ok(())
    }
    pub fn node_count(&self) -> i64 {
        self.node_count
    }
    pub fn text_count(&self) -> i64 {
        self.text_count
    }
    pub(crate) fn validate_data(&self, data: &NodeData) -> Result<(), Error> {
        validate_data(
            data,
            |id| self.node(id).map(|_| ()),
            |id| self.list(id).map(|_| ()),
            |slice| self.node_slice_read(slice).map(|_| ()),
            |slice| self.text_slice_read(slice).map(|_| ()),
        )
    }
    fn validate_staged(&self) -> Result<(), Error> {
        for header in self.storage.staged_nodes() {
            let aux = AuxId::from_parts(self.storage.lazy_auxiliary_arena(), header.ordinal)?;
            if !matches!(self.storage.aux(aux)?, AstStorageData::FallbackNode(_)) {
                return Err(Error::InvalidGraph);
            }
        }
        for value in self.storage.staged_aux() {
            match value {
                AstStorageData::FallbackNode(node) => {
                    if let Some(parent) = node.parent() {
                        self.storage.node(parent)?;
                    }
                    self.validate_data(node.data())?;
                }
                AstStorageData::List(list) => {
                    self.node_slice_read(list.nodes())?;
                }
                AstStorageData::Nodes(nodes) => {
                    for &node in nodes.iter().flatten() {
                        self.storage.node(node)?;
                    }
                }
                AstStorageData::Text(_) => {}
                AstStorageData::CompactNodes(_)
                | AstStorageData::File(_)
                | AstStorageData::SourceFiles(_)
                | AstStorageData::SourceMetadata(_) => {
                    return Err(Error::InvalidGraph);
                }
            }
        }
        Ok(())
    }
}

fn checked_len(len: usize) -> Result<u32, Error> {
    u32::try_from(len).map_err(|_| Error::InvalidSlot)
}
fn list_read(record: StorageRead<'_, AstStorageData>) -> Result<NodeListRead<'_>, Error> {
    if !matches!(&*record, AstStorageData::List(_)) {
        return Err(Error::InvalidGraph);
    }
    Ok(NodeListRead(record))
}
fn node_slice_read<'a>(
    nodes: NodeSlice,
    record: Option<StorageRead<'a, AstStorageData>>,
    compact: Option<(&'a crate::compact::lists::EdgePages, ts_arena::ArenaId)>,
) -> Result<NodeSliceRead<'a>, Error> {
    let mut start = nodes.start as usize;
    let len = nodes.len();
    let mut selected_compact = None;
    match &record {
        None if len == 0 && (start == 0 || nodes.is_missing()) => {}
        Some(record) => match &**record {
            AstStorageData::Nodes(values)
                if start
                    .checked_add(len)
                    .is_some_and(|end| end <= values.len()) => {}
            AstStorageData::CompactNodes(backing) => {
                let (edges, owner) = compact.ok_or(Error::InvalidGraph)?;
                if start
                    .checked_add(len)
                    .is_none_or(|end| end > backing.len as usize)
                {
                    return Err(Error::InvalidGraph);
                }
                start = backing
                    .start
                    .checked_add(start)
                    .ok_or(Error::InvalidGraph)?;
                let end = start.checked_add(len).ok_or(Error::InvalidGraph)?;
                if !edges.valid_range(start..end) {
                    return Err(Error::InvalidGraph);
                }
                selected_compact = Some((edges, owner));
            }
            _ => return Err(Error::InvalidGraph),
        },
        None => return Err(Error::InvalidGraph),
    }
    Ok(NodeSliceRead {
        record,
        compact: selected_compact,
        start,
        len,
    })
}
fn text_slice_read(
    text: TextSlice,
    record: Option<StorageRead<'_, AstStorageData>>,
) -> Result<TextSliceRead<'_>, Error> {
    let start = text.start as usize;
    let len = text.len();
    match &record {
        None if len == 0 && start == 0 => {}
        Some(record) => match &**record {
            AstStorageData::Text(values)
                if start
                    .checked_add(len)
                    .is_some_and(|end| end <= values.len()) => {}
            _ => return Err(Error::InvalidGraph),
        },
        None => return Err(Error::InvalidGraph),
    }
    Ok(TextSliceRead { record, start, len })
}

fn validate_data(
    data: &NodeData,
    node: impl FnMut(NodeId) -> Result<(), Error>,
    list: impl FnMut(NodeListId) -> Result<(), Error>,
    raw: impl FnMut(NodeSlice) -> Result<(), Error>,
    text: impl FnMut(TextSlice) -> Result<(), Error>,
) -> Result<(), Error> {
    data.validate_references(node, list, raw, text)
}

#[cfg(test)]
mod validation_proof_tests {
    use super::*;
    use crate::{node_flags, FactoryMethods, SyntaxKind};

    fn parsed(counters: &Counters) -> (ParsedFile, NodeId) {
        let mut builder = AstBuilder::new(SourceText::default(), counters);
        let root = builder.new_token(SyntaxKind::Unknown.into());
        (builder.complete(root).unwrap(), root)
    }

    #[test]
    fn narrow_flag_writes_preserve_the_completed_core_proof() {
        let counters = Counters::new();
        let baseline = counters.snapshot();
        let (mut parsed, root) = parsed(&counters);
        assert!(parsed.validated);
        parsed
            .set_node_flags(root, node_flags::UNREACHABLE)
            .unwrap();
        assert!(parsed.validated);
        assert_eq!(
            parsed.view().node(root).unwrap().flags(),
            node_flags::UNREACHABLE
        );
        let file = parsed.try_publish_unbound().unwrap();
        assert_eq!(
            file.view().node(root).unwrap().flags(),
            node_flags::UNREACHABLE
        );
        drop(file);
        assert_eq!(counters.snapshot(), baseline);
    }

    #[test]
    fn narrow_flag_writes_do_not_restore_a_dirty_core_proof() {
        let counters = Counters::new();
        let (mut parsed, root) = parsed(&counters);
        // Even a valid unrestricted mutation conservatively invalidates the
        // proof. A later narrow write must never turn that false back to true.
        parsed
            .builder_mut()
            .node_mut(root)
            .unwrap()
            .set_flags(node_flags::AMBIENT);
        assert!(!parsed.validated);
        parsed
            .set_node_flags(root, node_flags::UNREACHABLE)
            .unwrap();
        assert!(!parsed.validated);
        assert!(parsed.try_publish_unbound().is_ok());
    }

    #[test]
    fn narrow_flag_writes_reject_foreign_owners_without_changing_the_proof() {
        let counters = Counters::new();
        let (mut local, root) = parsed(&counters);
        let (foreign, foreign_root) = parsed(&counters);
        assert_eq!(
            local.set_node_flags(foreign_root, node_flags::UNREACHABLE),
            Err(Error::WrongOwner)
        );
        assert!(local.validated);
        assert_eq!(local.view().node(root).unwrap().flags(), 0);
        assert_eq!(foreign.view().node(foreign_root).unwrap().flags(), 0);
    }
}
