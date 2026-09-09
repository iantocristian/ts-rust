//! Scoped wrappers around the same semantic algorithms as the checked AST API.
use super::{BindNode, LocalBind};

impl<'scope> LocalBind<'scope, '_> {
    pub fn is_push_or_unshift_identifier(&self, node: BindNode<'scope>) -> bool {
        let read = self.node(node);
        if read.kind() == crate::SyntaxKind::Identifier {
            if let Some(identifier) = read.as_identifier() {
                return crate::binder_helpers::is_push_or_unshift_text(identifier.text());
            }
        }
        crate::is_push_or_unshift_identifier(self.view(), self.node_id(node))
            .expect("validated local text")
    }

    pub fn is_left_hand_side_expression(&self, node: BindNode<'scope>) -> bool {
        crate::syntax_helpers::is_left_hand_side_expression(self, node)
            .unwrap_or_else(|never| match never {})
    }
    pub fn is_dotted_name(&self, node: BindNode<'scope>) -> bool {
        crate::syntax_helpers::is_dotted_name(self, node).unwrap_or_else(|never| match never {})
    }
    pub fn is_outermost_optional_chain(&self, node: BindNode<'scope>) -> bool {
        crate::syntax_helpers::is_outermost_optional_chain(self, node)
            .unwrap_or_else(|never| match never {})
    }
    pub fn is_expression_of_optional_chain_root(&self, node: BindNode<'scope>) -> bool {
        crate::syntax_helpers::is_expression_of_optional_chain_root(self, node)
            .unwrap_or_else(|never| match never {})
    }
    pub fn is_logical_expression(&self, node: BindNode<'scope>) -> bool {
        crate::syntax_helpers::is_logical_expression(self, node)
            .unwrap_or_else(|never| match never {})
    }
    pub fn is_logical_or_coalescing_binary_expression(&self, node: BindNode<'scope>) -> bool {
        crate::syntax_helpers::is_logical_or_coalescing_binary_expression(self, node)
            .unwrap_or_else(|never| match never {})
    }
    pub fn is_logical_or_coalescing_assignment_expression(&self, node: BindNode<'scope>) -> bool {
        crate::syntax_helpers::is_logical_or_coalescing_assignment_expression(self, node)
            .unwrap_or_else(|never| match never {})
    }
    pub fn is_nullish_coalesce(&self, node: BindNode<'scope>) -> bool {
        crate::syntax_helpers::is_nullish_coalesce(self, node)
            .unwrap_or_else(|never| match never {})
    }
    pub fn is_entity_name_expression(&self, node: BindNode<'scope>) -> bool {
        crate::syntax_helpers::is_entity_name_expression(self, node, false)
            .unwrap_or_else(|never| match never {})
    }
    pub fn skip_parentheses(&self, node: BindNode<'scope>) -> BindNode<'scope> {
        crate::syntax_helpers::skip_parentheses(self, node).unwrap_or_else(|never| match never {})
    }
    pub fn is_optional_chain(&self, node: BindNode<'scope>) -> bool {
        crate::syntax_helpers::is_optional_chain(&self.node(node))
    }
    pub fn is_optional_chain_root(&self, node: BindNode<'scope>) -> bool {
        crate::syntax_helpers::is_optional_chain_root(&self.node(node))
    }
}

#[cfg(test)]
#[path = "local_bind_helpers_tests.rs"]
mod tests;
