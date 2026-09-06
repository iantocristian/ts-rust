//! The node record the arenas store.
//!
//! The generated AST is S03 and S06 work; ADR 0006's note deliberately leaves
//! the header and payload layout to measurement (plan section 13, item 6) and
//! fixes only the id and its semantics. This record therefore carries just what
//! the ownership contract itself constrains: a kind, a range, a parent link that
//! is an id rather than a reference, and an optional string payload. The
//! generated node data replaces it without changing any invariant here.

use ts_jsstring::JsString;

use crate::ids::NodeId;

/// A syntax kind. The generated `ts_ast` enumeration replaces this newtype.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct NodeKind(pub u16);

/// `ast.NodeFlags`, restricted to the flags the ownership contract reads.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct NodeFlags(pub u32);

impl NodeFlags {
    pub const NONE: Self = Self(0);
    /// `ast.NodeFlagsReparsed`: a JSDoc reparse clone, which belongs to the core
    /// arena and can never be a lazily created token's parent.
    pub const REPARSED: Self = Self(1 << 0);

    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}

/// One node's data. Parent and other cross-links are ids, never owning
/// references, so they may be cyclic.
#[derive(Clone, Debug)]
pub struct NodeData {
    pub kind: NodeKind,
    pub pos: u32,
    pub end: u32,
    pub flags: NodeFlags,
    pub parent: Option<NodeId>,
    /// The token or literal text, when the node carries one.
    pub text: Option<JsString>,
}

impl NodeData {
    pub fn new(kind: NodeKind, pos: u32, end: u32) -> Self {
        Self {
            kind,
            pos,
            end,
            flags: NodeFlags::NONE,
            parent: None,
            text: None,
        }
    }

    #[must_use]
    pub fn with_parent(mut self, parent: NodeId) -> Self {
        self.parent = Some(parent);
        self
    }

    #[must_use]
    pub fn with_flags(mut self, flags: NodeFlags) -> Self {
        self.flags = flags;
        self
    }

    #[must_use]
    pub fn with_text(mut self, text: JsString) -> Self {
        self.text = Some(text);
        self
    }
}
