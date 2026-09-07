//! Schema-generated syntax shapes. Runtime parsing and list storage arrive in S06.
//!
//! Child identities do not retain their owner. The generated visitors enumerate
//! node and list edges; an owning tree supplies list resolution and recursion.

mod accessors_generated;
mod data_generated;
mod kinds_generated;
mod visitors_generated;

pub use data_generated::*;
pub use kinds_generated::SyntaxKind;
pub use ts_arena::{ArenaId, NodeId};
pub use ts_jsstring::JsString;

use std::ops::ControlFlow;

/// A descriptor into one owner's child-id table, not an ownership proof.
///
/// Construction checks arithmetic only. The future list owner must additionally
/// validate its identity and published bounds before resolving the range.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeListRange {
    owner: ArenaId,
    start: u32,
    len: u32,
}

impl NodeListRange {
    pub fn new(owner: ArenaId, start: u32, len: u32) -> Option<Self> {
        start.checked_add(len)?;
        Some(Self { owner, start, len })
    }
    pub fn owner(self) -> ArenaId {
        self.owner
    }
    pub fn start(self) -> u32 {
        self.start
    }
    pub fn len(self) -> u32 {
        self.len
    }
    pub fn is_empty(self) -> bool {
        self.len == 0
    }
}

/// Source positions are signed byte offsets, including upstream's synthetic -1.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    kind: SyntaxKind,
    flags: u32,
    pos: i32,
    end: i32,
    data: NodeData,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidNodeKind {
    pub kind: SyntaxKind,
    pub data_name: &'static str,
}

impl Node {
    /// Validate the schema's admissible kinds for this payload, not a complete
    /// parsed-tree or encoding invariant. Upstream's generic Token shape admits
    /// identifier/literal kinds too; children, concrete encoding payloads and
    /// owner/list validity require validation by the consuming runtime.
    pub fn new(
        kind: SyntaxKind,
        pos: i32,
        end: i32,
        data: NodeData,
    ) -> Result<Self, InvalidNodeKind> {
        if !data.supports_kind(kind) {
            return Err(InvalidNodeKind {
                kind,
                data_name: data.name(),
            });
        }
        Ok(Self {
            kind,
            flags: 0,
            pos,
            end,
            data,
        })
    }
    pub fn kind(&self) -> SyntaxKind {
        self.kind
    }
    pub fn flags(&self) -> u32 {
        self.flags
    }
    pub fn set_flags(&mut self, flags: u32) {
        self.flags = flags;
    }
    pub fn pos(&self) -> i32 {
        self.pos
    }
    pub fn end(&self) -> i32 {
        self.end
    }
    pub fn data(&self) -> &NodeData {
        &self.data
    }
    pub fn for_each_child(&self, visitor: &mut impl ChildVisitor) -> ControlFlow<()> {
        self.data.for_each_child(visitor)
    }
}

/// List callbacks preserve list boundaries and empty-vs-absent distinctions.
pub trait ChildVisitor {
    fn visit_node(&mut self, node: NodeId) -> ControlFlow<()>;
    fn visit_list(&mut self, nodes: NodeListRange) -> ControlFlow<()>;
}

/// Maps immediate child edges into a new payload without mutating the input.
/// Allocation, recursion and retaining replacement owners are caller concerns.
pub trait ChildMapper {
    fn map_node(&mut self, node: NodeId, role: ChildRole) -> NodeId;
    fn map_list(&mut self, nodes: NodeListRange, role: ChildRole) -> NodeListRange;
}

/// Pinned factory-visitor dispatch. These roles select different transformation
/// hooks upstream even when the fields share a Rust storage representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChildRole {
    Node,
    Nodes,
    RawNodes,
    Token,
    Modifiers,
    EmbeddedStatement,
    IterationBody,
    Parameters,
    FunctionBody,
    TopLevelStatements,
}

/// Schema storage deliberately deferred to the owning runtime phase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeferredField {
    pub node: &'static str,
    pub field: &'static str,
    pub upstream_type: &'static str,
    pub owner: &'static str,
}

#[cfg(test)]
mod tests;
