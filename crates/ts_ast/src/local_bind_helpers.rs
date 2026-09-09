//! Source helpers whose traversal stays inside the validated local core.
use super::{BindNode, LocalBind};
use crate::SyntaxKind as K;

impl<'scope> LocalBind<'scope, '_> {
    /// port: tsc/internal/ast/utilities.go:IsEntityNameExpression
    ///
    /// This is the non-JavaScript variant: `this` and element access are not
    /// entity names. As in Node.Name, the name follows the concrete shape;
    /// its kind is checked before the receiver's payload or nil value.
    pub fn is_entity_name_expression(&self, mut node: BindNode<'scope>) -> bool {
        loop {
            let read = self.node(node);
            match read.kind().known() {
                Some(K::Identifier) => return true,
                Some(K::PropertyAccessExpression) => {
                    let name = read.name().expect("nil node in source AST utility");
                    if self.node(name).kind() != K::Identifier {
                        return false;
                    }
                    node = read
                        .as_property_access_expression()
                        .map(|data| data.expression())
                        .unwrap_or_else(|| self.incompatible_helper_expression(node))
                        .expect("nil entity-name expression");
                }
                _ => return false,
            }
        }
    }

    /// port: tsc/internal/ast/utilities.go:IsLeftHandSideExpression
    ///
    /// Only partially emitted wrappers are skipped. The terminal predicate
    /// depends on kind, including for factory-created kind/shape mismatches.
    pub fn is_left_hand_side_expression(&self, mut node: BindNode<'scope>) -> bool {
        loop {
            let read = self.node(node);
            if read.kind() != K::PartiallyEmittedExpression {
                return crate::is_left_hand_side_expression_kind(read.kind());
            }
            node = read
                .as_partially_emitted_expression()
                .map(|data| data.expression())
                .unwrap_or_else(|| self.incompatible_helper_expression(node))
                .expect("nil partially emitted expression");
        }
    }

    // Factory kinds are independent of concrete shape. Retain Node.Expression's
    // interface-conversion panic for the exceptional mismatch, without routing
    // ordinary typed traversal through NodeRead or an owner check.
    #[cold]
    fn incompatible_helper_expression(&self, node: BindNode<'scope>) -> Option<BindNode<'scope>> {
        self.general_node(self.node_id(node))
            .expect("validated local node")
            .expression()
            .map(|node| self.import_node(node).expect("validated local expression"))
    }
}

#[cfg(test)]
#[path = "local_bind_helpers_tests.rs"]
mod tests;
