use crate::NodeId;

/// The storage operations that inspect syntax metadata use this single record
/// interface. Concrete AST records own their header; storage adds no wrapper.
pub trait NodeRecord {
    type Aux;
    fn storage_kind(&self) -> u32;
    fn storage_parent(&self) -> Option<NodeId>;
    fn set_storage_parent(&mut self, parent: Option<NodeId>);
    fn storage_reparsed(&self) -> bool;
}

/// Storage metadata around an AST-defined payload. Graph edges are non-owning ids.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node<T> {
    pub kind: u32,
    pub parent: Option<NodeId>,
    pub reparsed: bool,
    pub data: T,
}

impl<T> Node<T> {
    pub fn new(kind: u32, data: T) -> Self {
        Self {
            kind,
            parent: None,
            reparsed: false,
            data,
        }
    }
}

impl<T> NodeRecord for Node<T> {
    type Aux = ();
    fn storage_kind(&self) -> u32 {
        self.kind
    }
    fn storage_parent(&self) -> Option<NodeId> {
        self.parent
    }
    fn set_storage_parent(&mut self, parent: Option<NodeId>) {
        self.parent = parent;
    }
    fn storage_reparsed(&self) -> bool {
        self.reparsed
    }
}
