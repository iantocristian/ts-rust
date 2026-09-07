use crate::{
    AstStorageData, FactoryHooks, FileInfo, JSDocRoots, JsString, Node, NodeData, NodeId, NodeList,
    NodeListId, NodeListRead, NodeSlice, NodeSliceRead, TextSlice, TextSliceRead,
};
use std::sync::Arc;
use ts_arena::{
    AuxId, Counters, Error, StorageBuilder, StorageHandle, StorageRead, StorageTransaction,
    StorageView,
};
use ts_core::TextRange;
use ts_jsstring::SourceText;

pub type NodeRead<'a> = StorageRead<'a, Node>;

/// Exclusive syntax construction. Hooks exist only during this exclusive phase.
pub struct AstBuilder {
    pub(crate) storage: StorageBuilder<Node>,
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
        AstView(self.storage.view())
    }
    pub fn id(&self) -> ts_arena::FileId {
        self.storage.id()
    }
    /// Retain a published dependency before storing any of its identities.
    /// Importing a mapped member retains its complete bundle and dependencies.
    pub fn retain_file(&mut self, file: AstFile) {
        self.storage.retain_file(file.0);
    }
    pub fn node_mut(&mut self, id: NodeId) -> Result<&mut Node, Error> {
        self.storage.node_mut(id)
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
    pub fn node_slice(&mut self, nodes: Vec<Option<NodeId>>) -> Result<NodeSlice, Error> {
        for &id in nodes.iter().flatten() {
            self.view().node(id)?;
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
        Ok(ParsedFile { builder: self })
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
}
impl ParsedFile {
    pub fn view(&self) -> AstView<'_> {
        self.builder.view()
    }
    pub fn builder_mut(&mut self) -> &mut AstBuilder {
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
    /// This is an O(nodes + auxiliary values + stored edges) publication pass.
    pub fn try_publish_unbound(self) -> Result<AstFile, Error> {
        self.view().validate_core()?;
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
                AstView(*view).validate_core()?;
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
pub struct AstBundle(Arc<ts_arena::StorageBundle<Node>>);
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
pub struct AstFile(pub(crate) StorageHandle<Node>);
impl AstFile {
    pub fn view(&self) -> AstView<'_> {
        AstView(self.0.view())
    }
    pub fn root(&self) -> Option<NodeId> {
        self.view().file_info().root
    }
    pub fn retain_node(&self, id: NodeId) -> Result<RetainedNode, Error> {
        self.0
            .resolved_node(id)
            .map(|node| RetainedNode(node.retain()))
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
pub struct RetainedNode(ts_arena::RetainedRecord<Node>);
impl RetainedNode {
    pub fn id(&self) -> NodeId {
        self.0.id()
    }
    /// Explicitly retain the file through which this node was resolved, including
    /// its mapped siblings and imported dependencies.
    pub fn file(&self) -> AstFile {
        AstFile(self.0.owner().clone())
    }
}
impl std::ops::Deref for RetainedNode {
    type Target = Node;
    fn deref(&self) -> &Node {
        &self.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AstView<'a>(pub(crate) StorageView<'a, Node>);
impl<'a> AstView<'a> {
    pub fn for_node_owner(self, node: NodeId) -> Result<Self, Error> {
        self.0.for_node_owner(node).map(Self)
    }
    pub fn source(self) -> &'a SourceText {
        self.0.source()
    }
    pub fn position_map(self) -> &'a ts_jsstring::PositionMap {
        self.0.position_map()
    }
    pub fn node(self, id: NodeId) -> Result<NodeRead<'a>, Error> {
        self.0.node(id)
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
        node_slice_read(nodes, nodes.backing.map(|id| self.0.aux(id)).transpose()?)
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
    fn validate_core(self) -> Result<(), Error> {
        for node in self.0.core_nodes() {
            if let Some(parent) = node.parent() {
                self.node(parent)?;
            }
            self.validate_data(node.data())?;
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
    pub(crate) storage: &'a mut StorageTransaction<'storage, Node>,
    pub(crate) node_count: i64,
    pub(crate) text_count: i64,
}
impl AstTransaction<'_, '_> {
    pub fn node(&self, id: NodeId) -> Result<NodeRead<'_>, Error> {
        self.storage.node(id).map(StorageRead::borrowed)
    }
    pub fn node_mut(&mut self, id: NodeId) -> Result<&mut Node, Error> {
        self.storage.node_mut(id)
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
        for node in self.storage.staged_nodes() {
            if let Some(parent) = node.parent() {
                self.storage.node(parent)?;
            }
            self.validate_data(node.data())?;
        }
        for value in self.storage.staged_aux() {
            match value {
                AstStorageData::List(list) => {
                    self.node_slice_read(list.nodes())?;
                }
                AstStorageData::Nodes(nodes) => {
                    for &node in nodes.iter().flatten() {
                        self.storage.node(node)?;
                    }
                }
                AstStorageData::Text(_) => {}
                AstStorageData::File(_)
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
fn node_slice_read(
    nodes: NodeSlice,
    record: Option<StorageRead<'_, AstStorageData>>,
) -> Result<NodeSliceRead<'_>, Error> {
    let start = nodes.start as usize;
    let len = nodes.len();
    match &record {
        None if len == 0 && (start == 0 || nodes.is_missing()) => {}
        Some(record) => match &**record {
            AstStorageData::Nodes(values)
                if start
                    .checked_add(len)
                    .is_some_and(|end| end <= values.len()) => {}
            _ => return Err(Error::InvalidGraph),
        },
        None => return Err(Error::InvalidGraph),
    }
    Ok(NodeSliceRead { record, start, len })
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
