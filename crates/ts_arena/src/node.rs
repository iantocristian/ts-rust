use crate::NodeId;

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
