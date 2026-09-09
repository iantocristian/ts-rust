//! Local syntax identities stay branded through dispatch and recursive walks.
//! Unmigrated helpers cross the explicit raw-ID boundary below.
use crate::{backend::Backend, Binder};
use ts_ast::{local_bind::BindNode, NodeId, NodeKind};

#[derive(Clone, Copy)]
pub(crate) enum BindingNode<'scope> {
    Local(BindNode<'scope>),
    Checked(NodeId),
}

impl<'scope> Binder<'_, 'scope, '_> {
    pub(crate) fn binding_node(&self, id: NodeId) -> BindingNode<'scope> {
        if let Backend::Local(local) = &self.builder {
            if let Ok(node) = local.import_node(id) {
                return BindingNode::Local(node);
            }
        }
        BindingNode::Checked(id)
    }

    /// An explicit compatibility boundary; local dispatch does not re-import it.
    pub(crate) fn node_id(&self, node: BindingNode<'scope>) -> NodeId {
        match node {
            BindingNode::Local(node) => {
                let Backend::Local(local) = &self.builder else {
                    unreachable!("local binder scope");
                };
                local.node_id(node)
            }
            BindingNode::Checked(id) => id,
        }
    }

    pub(crate) fn node_kind(&self, node: BindingNode<'scope>) -> NodeKind {
        match node {
            BindingNode::Local(node) => {
                let Backend::Local(local) = &self.builder else {
                    unreachable!("local binder scope");
                };
                local.node(node).kind()
            }
            BindingNode::Checked(id) => self.n(id).kind(),
        }
    }

    pub(crate) fn node_flags(&self, node: BindingNode<'scope>) -> u32 {
        match node {
            BindingNode::Local(node) => {
                let Backend::Local(local) = &self.builder else {
                    unreachable!("local binder scope");
                };
                local.node(node).flags()
            }
            BindingNode::Checked(id) => self.n(id).flags(),
        }
    }

    pub(crate) fn set_binding_flags(&mut self, node: BindingNode<'scope>, flags: u32) {
        match node {
            BindingNode::Local(node) => {
                let Backend::Local(local) = &mut self.builder else {
                    unreachable!("local binder scope");
                };
                if local.node(node).flags() != flags {
                    local.set_flags(node, flags);
                }
            }
            BindingNode::Checked(id) => self.set_flags(id, flags),
        }
    }

    pub(crate) fn container_flags(&self, node: BindingNode<'scope>) -> crate::ContainerFlags {
        match node {
            BindingNode::Local(node) => {
                let Backend::Local(local) = &self.builder else {
                    unreachable!("local binder scope");
                };
                crate::container_classification::local_container_flags(local, node)
            }
            BindingNode::Checked(id) => crate::checked(crate::get_container_flags(self.view(), id)),
        }
    }
}
