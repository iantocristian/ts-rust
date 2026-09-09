//! AST reads keep their physical namespace and source next to the borrowed record.
//! Payload storage can therefore move out of `Node` without making callers
//! rediscover the owner or retain it during ordinary reads.

use crate::{Node, NodeId};
use std::ops::Deref;
use ts_arena::{FileId, StorageRead, StorageView};
use ts_jsstring::SourceText;

/// A checked AST record and its physical owner context. The node ID may name
/// the owner's core or lazy arena; `owner_id` always identifies the core file.
///
/// Core and already retained records borrow directly. Resolving a lazy ID through
/// a view instead acquires a page guard, whose payload references stay bounded
/// by a borrow of this read. Neither path retains an additional file owner.
/// Use `RetainedNode` to escape.
///
/// Contextual payload bytes cannot outlive a directory-resolved lazy read guard,
/// even when its enclosing file lives longer:
///
/// ```compile_fail
/// use ts_ast::{AstFile, NodeId};
/// fn escaped_text<'owner>(file: &'owner AstFile, lazy: NodeId) -> &'owner [u8] {
///     let read = file.view().node(lazy).unwrap();
///     read.as_identifier().unwrap().text()
/// }
/// ```
pub struct NodeRead<'a> {
    record: StorageRead<'a, Node>,
    id: NodeId,
    owner_id: FileId,
    source: &'a SourceText,
}

impl std::fmt::Debug for NodeRead<'_> {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.debug_struct("NodeRead")
            .field("id", &self.id)
            .field("owner_id", &self.owner_id)
            .field("record", &self.record)
            .finish_non_exhaustive()
    }
}

impl<'a> NodeRead<'a> {
    pub(crate) fn resolved(
        id: NodeId,
        record: StorageRead<'a, Node>,
        owner: StorageView<'a, Node>,
    ) -> Self {
        Self::new(id, record, owner.id(), owner.source())
    }

    pub(crate) fn new(
        id: NodeId,
        record: StorageRead<'a, Node>,
        owner_id: FileId,
        source: &'a SourceText,
    ) -> Self {
        Self {
            record,
            id,
            owner_id,
            source,
        }
    }

    /// Full identity, including the core or lazy arena that contains this node.
    pub fn id(&self) -> NodeId {
        self.id
    }

    /// The physical file whose source and local namespaces interpret this node.
    /// This can differ from both its lazy arena and the importing caller's file.
    pub fn owner_id(&self) -> FileId {
        self.owner_id
    }

    pub fn source(&self) -> &'a SourceText {
        self.source
    }

    /// Return the original borrow for a core or already retained record. A read
    /// that acquired its own lazy page guard returns `None`, since that payload
    /// cannot outlive this guard even when the enclosing file lives longer.
    pub fn as_borrowed(&self) -> Option<&'a Node> {
        self.record.as_borrowed()
    }
}

// Transitional physical access while generated and handwritten consumers move
// to contextual payload views. This does not reconstruct or copy a Node.
impl Deref for NodeRead<'_> {
    type Target = Node;

    fn deref(&self) -> &Node {
        &self.record
    }
}

#[cfg(test)]
mod tests;
