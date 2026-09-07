use crate::{JsString, NodeId};
use std::{
    ops::{Deref, Range},
    sync::Arc,
};
use ts_arena::{AuxId, Error, StorageRead};
use ts_core::TextRange;

/// An immutable list identity. The referenced header remains mutable only while
/// its file or lazy graph is under exclusive construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeListId(pub(crate) AuxId);
impl NodeListId {
    pub fn bits(self) -> u64 {
        self.0.bits()
    }
}

macro_rules! slice_handle {
    ($name:ident) => {
        /// A non-owning slice of one owner-qualified backing allocation. Empty
        /// slices compare equal under `same`, including nil and allocated-empty.
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
        pub struct $name {
            pub(crate) backing: Option<AuxId>,
            pub(crate) start: u32,
            pub(crate) len: u32,
        }
        impl $name {
            pub const fn empty() -> Self {
                Self {
                    backing: None,
                    start: 0,
                    len: 0,
                }
            }
            pub fn len(self) -> usize {
                self.len as usize
            }
            pub fn is_empty(self) -> bool {
                self.len == 0
            }
            pub fn is_nil(self) -> bool {
                self.backing.is_none() && self.start == 0
            }
            pub fn same(self, other: Self) -> bool {
                self.len == other.len
                    && (self.len == 0 || self.backing == other.backing && self.start == other.start)
            }
            pub fn slice(self, range: Range<usize>) -> Result<Self, Error> {
                if range.start > range.end || range.end > self.len() {
                    return Err(Error::InvalidSlot);
                }
                let start = self
                    .start
                    .checked_add(u32::try_from(range.start).map_err(|_| Error::InvalidSlot)?)
                    .ok_or(Error::InvalidSlot)?;
                Ok(Self {
                    backing: self.backing,
                    start,
                    len: u32::try_from(range.len()).map_err(|_| Error::InvalidSlot)?,
                })
            }
        }
    };
}
slice_handle!(NodeSlice);
slice_handle!(TextSlice);

impl NodeSlice {
    // Go's process-wide missingListNodes has an empty, nonnil sentinel backing.
    // A reserved empty descriptor carries that identity without fabricating an
    // owner or allocating a per-file substitute. It still compares Same to any
    // other empty slice, as the pinned core.Same does.
    pub(crate) const fn missing() -> Self {
        Self {
            backing: None,
            start: 1,
            len: 0,
        }
    }
    pub(crate) fn is_missing(self) -> bool {
        self.backing.is_none() && self.start == 1 && self.len == 0
    }
}

/// NodeList/ModifierList header identity is separate from edge backing identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeList {
    loc: TextRange,
    nodes: NodeSlice,
    modifier_flags: u32,
}
impl NodeList {
    pub fn new(loc: TextRange, nodes: NodeSlice) -> Self {
        Self {
            loc,
            nodes,
            modifier_flags: 0,
        }
    }
    pub fn loc(&self) -> TextRange {
        self.loc
    }
    pub fn set_loc(&mut self, loc: TextRange) {
        self.loc = loc;
    }
    pub fn nodes(&self) -> NodeSlice {
        self.nodes
    }
    pub fn modifier_flags(&self) -> u32 {
        self.modifier_flags
    }
    pub fn set_modifier_flags(&mut self, flags: u32) {
        self.modifier_flags = flags;
    }
    pub fn is_missing(&self) -> bool {
        self.nodes.is_missing()
    }
    pub(crate) fn set_missing(&mut self, missing: bool) {
        if missing {
            self.nodes = NodeSlice::missing();
        } else if self.nodes.is_missing() {
            self.nodes = NodeSlice::empty();
        }
    }
    pub(crate) fn set_nodes(&mut self, nodes: NodeSlice) {
        self.nodes = nodes;
    }
}

/// The file frame is owned with records, so an escaped retained node preserves it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FileInfo {
    pub root: Option<NodeId>,
    pub node_count: i64,
    pub text_count: i64,
    pub(crate) source_files: Option<AuxId>,
}

/// Typed auxiliary records share the node owner's lifetime and publication lock.
/// This enum is storage plumbing; checked AST APIs distinguish its record kinds.
#[derive(Debug)]
pub enum AstStorageData {
    List(NodeList),
    Nodes(Box<[Option<NodeId>]>),
    Text(Box<[JsString]>),
    File(FileInfo),
    SourceMetadata(crate::SourceMetadataData),
    SourceFiles(std::collections::BTreeMap<NodeId, crate::SourceFileState>),
}

pub struct NodeListRead<'a>(pub(crate) StorageRead<'a, AstStorageData>);
impl Deref for NodeListRead<'_> {
    type Target = NodeList;
    fn deref(&self) -> &NodeList {
        match &*self.0 {
            AstStorageData::List(list) => list,
            _ => unreachable!("validated list record"),
        }
    }
}
impl std::fmt::Debug for NodeListRead<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}

pub struct NodeSliceRead<'a> {
    pub(crate) record: Option<StorageRead<'a, AstStorageData>>,
    pub(crate) start: usize,
    pub(crate) len: usize,
}
impl Deref for NodeSliceRead<'_> {
    type Target = [Option<NodeId>];
    fn deref(&self) -> &Self::Target {
        match &self.record {
            None => &[],
            Some(record) => match &**record {
                AstStorageData::Nodes(nodes) => &nodes[self.start..self.start + self.len],
                _ => unreachable!("validated node slice record"),
            },
        }
    }
}
pub struct TextSliceRead<'a> {
    pub(crate) record: Option<StorageRead<'a, AstStorageData>>,
    pub(crate) start: usize,
    pub(crate) len: usize,
}
impl Deref for TextSliceRead<'_> {
    type Target = [JsString];
    fn deref(&self) -> &Self::Target {
        match &self.record {
            None => &[],
            Some(record) => match &**record {
                AstStorageData::Text(text) => &text[self.start..self.start + self.len],
                _ => unreachable!("validated text slice record"),
            },
        }
    }
}

/// Cached root IDs do not retain a file. Empty results need no allocation or
/// reference-count operation; cache presence is tracked separately by the owner.
#[derive(Clone, Debug, Default)]
pub struct JSDocRoots(Option<Arc<[NodeId]>>);
impl Deref for JSDocRoots {
    type Target = [NodeId];
    fn deref(&self) -> &[NodeId] {
        self.0.as_deref().unwrap_or(&[])
    }
}

impl JSDocRoots {
    /// An empty result without allocating or retaining shared storage.
    pub const fn empty() -> Self {
        Self(None)
    }
    pub(crate) fn from_shared(roots: Arc<[NodeId]>) -> Self {
        if roots.is_empty() {
            Self::empty()
        } else {
            Self(Some(roots))
        }
    }
}
