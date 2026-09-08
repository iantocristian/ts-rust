use crate::{
    AstBuilder, AstTransaction, Node, NodeData, NodeId, NodeKind, NodeRead, SourceFileRead,
    SourceFileState,
};
use ts_arena::Error;

/// Hooks receive identities and an exclusive factory context, with no outstanding
/// node borrow. An immutable callback can reenter construction or mutate the
/// original core node. A present hook is retained explicitly for each invocation;
/// ordinary construction with no hook performs no hook reference-count operation.
pub trait FactoryHooks: Send + Sync {
    fn on_create(&self, _factory: &mut dyn Factory, _node: NodeId) {}
    fn on_update(&self, _factory: &mut dyn Factory, _node: NodeId, _original: NodeId) {}
    fn on_clone(&self, _factory: &mut dyn Factory, _node: NodeId, _original: NodeId) {}
}

/// Storage operations consumed by the authoritative generated constructors.
/// IDs are checked at this Rust ownership boundary even in release builds.
pub trait Factory {
    fn node(&self, id: NodeId) -> NodeRead<'_>;
    fn node_count(&self) -> i64;
    fn text_count(&self) -> i64;
    fn node_mut(&mut self, id: NodeId) -> &mut Node;
    /// Rich SourceFile metadata belongs to a complete owner. A lazy JSDoc
    /// transaction has no authority to create or mutate another source frame.
    fn read_source_file(&self, _id: NodeId) -> Result<SourceFileRead<'_>, Error> {
        Err(Error::InvalidGraph)
    }
    fn mut_source_file(&mut self, _id: NodeId) -> Result<&mut SourceFileState, Error> {
        Err(Error::InvalidGraph)
    }
    fn new_node(&mut self, kind: NodeKind, data: NodeData) -> NodeId;
    // port: tsc/internal/ast/ast.go:NodeFactory.NewModifier
    fn new_modifier(&mut self, kind: NodeKind) -> NodeId {
        self.new_node(kind, crate::TokenData {}.into())
    }
    fn increment_text_count(&mut self);
    fn set_node_flags(&mut self, id: NodeId, flags: u32);
    fn finish_update(&mut self, updated: NodeId, original: NodeId) -> NodeId;
    fn finish_clone(&mut self, updated: NodeId, original: NodeId) -> NodeId;
}
impl AstBuilder {
    pub(crate) fn new_node_before_hook(&mut self, kind: NodeKind, data: NodeData) -> NodeId {
        self.view()
            .validate_data(&data)
            .expect("factory edges belong to retained storage");
        let frame = self.frame_mut();
        frame.node_count = frame.node_count.wrapping_add(1);
        self.storage.push(Node::from_factory_parts(kind, data))
    }

    pub(crate) fn run_create_hook(&mut self, node: NodeId) {
        if let Some(hooks) = self.hooks.clone() {
            hooks.on_create(self, node);
        }
    }
}
impl Factory for AstBuilder {
    fn read_source_file(&self, id: NodeId) -> Result<SourceFileRead<'_>, Error> {
        self.view().source_file(id)
    }
    fn mut_source_file(&mut self, id: NodeId) -> Result<&mut SourceFileState, Error> {
        self.source_file_mut(id)
    }
    fn node_count(&self) -> i64 {
        AstBuilder::node_count(self)
    }
    fn text_count(&self) -> i64 {
        AstBuilder::text_count(self)
    }
    fn node(&self, id: NodeId) -> NodeRead<'_> {
        self.view()
            .node(id)
            .expect("factory node belongs to retained storage")
    }
    fn node_mut(&mut self, id: NodeId) -> &mut Node {
        self.storage
            .node_mut(id)
            .expect("factory owns mutable core node")
    }
    fn new_node(&mut self, kind: NodeKind, data: NodeData) -> NodeId {
        let node = self.new_node_before_hook(kind, data);
        self.run_create_hook(node);
        node
    }
    fn increment_text_count(&mut self) {
        let frame = self.frame_mut();
        frame.text_count = frame.text_count.wrapping_add(1);
    }
    fn set_node_flags(&mut self, id: NodeId, flags: u32) {
        self.storage
            .node_mut(id)
            .expect("factory owns mutable node")
            .set_flags(flags);
    }
    fn finish_update(&mut self, updated: NodeId, original: NodeId) -> NodeId {
        if updated != original {
            let (flags, range) = {
                let original = self.node(original);
                (original.flags(), original.range())
            };
            let node = self
                .storage
                .node_mut(updated)
                .expect("factory owns updated core node");
            node.set_flags(flags);
            node.set_range(range);
            if let Some(hooks) = self.hooks.clone() {
                hooks.on_update(self, updated, original);
            }
        }
        updated
    }
    fn finish_clone(&mut self, updated: NodeId, original: NodeId) -> NodeId {
        let updated = self.finish_update(updated, original);
        if updated != original {
            if let Some(hooks) = self.hooks.clone() {
                hooks.on_clone(self, updated, original);
            }
        }
        updated
    }
}
impl Factory for AstTransaction<'_, '_> {
    fn node_count(&self) -> i64 {
        self.node_count
    }
    fn text_count(&self) -> i64 {
        self.text_count
    }
    fn node(&self, id: NodeId) -> NodeRead<'_> {
        AstTransaction::node(self, id).expect("transaction node belongs to this file")
    }
    fn node_mut(&mut self, id: NodeId) -> &mut Node {
        self.storage
            .node_mut(id)
            .expect("transaction owns mutable staged node")
    }
    fn new_node(&mut self, kind: NodeKind, data: NodeData) -> NodeId {
        self.validate_data(&data)
            .expect("transaction edges belong to this file");
        self.node_count = self.node_count.wrapping_add(1);
        self.storage.push(Node::from_factory_parts(kind, data))
    }
    fn increment_text_count(&mut self) {
        self.text_count = self.text_count.wrapping_add(1);
    }
    fn set_node_flags(&mut self, id: NodeId, flags: u32) {
        self.storage
            .node_mut(id)
            .expect("transaction owns mutable node")
            .set_flags(flags);
    }
    fn finish_update(&mut self, updated: NodeId, original: NodeId) -> NodeId {
        if updated != original {
            let original = self
                .storage
                .node(original)
                .expect("original resolves in transaction");
            let flags = original.flags();
            let range = original.range();
            let updated = self
                .storage
                .node_mut(updated)
                .expect("updated node is staged");
            updated.set_flags(flags);
            updated.set_range(range);
        }
        updated
    }
    fn finish_clone(&mut self, updated: NodeId, original: NodeId) -> NodeId {
        self.finish_update(updated, original)
    }
}

// Borrowed transaction adapters preserve exclusive construction without moving
// the transaction out of its publication callback.
pub struct BorrowedFactory<'a, T: ?Sized>(pub &'a mut T);

impl<T: Factory + ?Sized> Factory for BorrowedFactory<'_, T> {
    fn read_source_file(&self, id: NodeId) -> Result<SourceFileRead<'_>, Error> {
        self.0.read_source_file(id)
    }
    fn mut_source_file(&mut self, id: NodeId) -> Result<&mut SourceFileState, Error> {
        self.0.mut_source_file(id)
    }
    fn node(&self, id: NodeId) -> NodeRead<'_> {
        self.0.node(id)
    }
    fn node_count(&self) -> i64 {
        self.0.node_count()
    }
    fn text_count(&self) -> i64 {
        self.0.text_count()
    }
    fn node_mut(&mut self, id: NodeId) -> &mut Node {
        self.0.node_mut(id)
    }
    fn new_node(&mut self, kind: NodeKind, data: NodeData) -> NodeId {
        self.0.new_node(kind, data)
    }
    fn new_modifier(&mut self, kind: NodeKind) -> NodeId {
        self.0.new_modifier(kind)
    }
    fn increment_text_count(&mut self) {
        self.0.increment_text_count();
    }
    fn set_node_flags(&mut self, id: NodeId, flags: u32) {
        self.0.set_node_flags(id, flags);
    }
    fn finish_update(&mut self, updated: NodeId, original: NodeId) -> NodeId {
        self.0.finish_update(updated, original)
    }
    fn finish_clone(&mut self, updated: NodeId, original: NodeId) -> NodeId {
        self.0.finish_clone(updated, original)
    }
}
