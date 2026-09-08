//! Explicit continuations for ordinary binary-expression evaluation. Logical and
//! destructuring forms retain their own branch semantics under the bind guard.
use crate::{expressions::payload, need, Binder};
use ts_ast::{flow_flags as F, node_flags as N, NodeId, SyntaxKind as K};

struct Exit {
    node: NodeId,
    parse_error: bool,
    saved_parse_error: bool,
    saved_assignment_pattern: bool,
}
enum Step {
    Visit(Option<NodeId>),
    AfterLeft(NodeId),
    AfterRight(NodeId),
    Exit(Exit),
}
impl Binder<'_, '_> {
    pub(crate) fn bind_binary_expression_trampoline(&mut self, node: NodeId) {
        let expression = payload!(self, node, as_binary_expression);
        let left = expression.left;
        // Ordinary leaves need no continuation allocation. Long binary chains
        // share one stack for every directly nested ordinary binary operand.
        if [left, expression.right]
            .into_iter()
            .flatten()
            .all(|child| self.n(child).kind() != K::BinaryExpression)
        {
            self.bind(left);
            self.bind_binary_middle(node);
            self.bind(expression.right);
            self.finish_binary_flow(node);
            return;
        }
        let mut steps = vec![Step::AfterLeft(node), Step::Visit(left)];
        #[cfg(test)]
        crate::recursion::binary_frame(steps.len());
        while let Some(step) = steps.pop() {
            match step {
                Step::Visit(Some(child)) if self.can_continue_binary(child) => {
                    self.bind_node_head(child);
                    let exit = Exit {
                        node: child,
                        parse_error: self.n(child).flags() & N::THIS_NODE_HAS_ERROR != 0,
                        saved_parse_error: self.seen_parse_error,
                        saved_assignment_pattern: self.in_assignment_pattern,
                    };
                    self.seen_parse_error = false;
                    self.in_assignment_pattern = false;
                    steps.push(Step::Exit(exit));
                    steps.push(Step::AfterLeft(child));
                    steps.push(Step::Visit(
                        payload!(self, child, as_binary_expression).left,
                    ));
                    #[cfg(test)]
                    crate::recursion::binary_frame(steps.len());
                }
                Step::Visit(node) => {
                    self.bind(node);
                }
                Step::AfterLeft(node) => {
                    let expression = payload!(self, node, as_binary_expression);
                    self.bind_binary_middle(node);
                    steps.push(Step::AfterRight(node));
                    steps.push(Step::Visit(expression.right));
                }
                Step::AfterRight(node) => self.finish_binary_flow(node),
                Step::Exit(exit) => {
                    self.in_assignment_pattern = exit.saved_assignment_pattern;
                    let error = exit.parse_error || self.seen_parse_error;
                    self.seen_parse_error = exit.saved_parse_error;
                    self.bind_node_error(exit.node, error);
                }
            }
        }
    }
    fn can_continue_binary(&self, node: NodeId) -> bool {
        if self.n(node).kind() != K::BinaryExpression || self.current_flow == self.unreachable_flow
        {
            return false;
        }
        if ts_ast::is_destructuring_assignment(self.view(), node)
            .expect("retained destructuring expression")
        {
            return false;
        }
        let operator = self
            .n(need(
                payload!(self, node, as_binary_expression).operator_token,
            ))
            .kind();
        !ts_ast::utilities::is_logical_or_coalescing_binary_operator(operator)
            && !ts_ast::is_logical_or_coalescing_assignment_operator(operator)
    }
    fn bind_binary_middle(&mut self, node: NodeId) {
        let expression = payload!(self, node, as_binary_expression);
        self.bind(expression.r#type);
        if self.n(need(expression.operator_token)).kind() == K::CommaToken {
            self.maybe_bind_expression_flow_if_call(need(expression.left));
        }
        self.bind(expression.operator_token);
    }
    fn finish_binary_flow(&mut self, node: NodeId) {
        let expression = payload!(self, node, as_binary_expression);
        let operator = self.n(need(expression.operator_token)).kind();
        if operator == K::CommaToken {
            self.maybe_bind_expression_flow_if_call(need(expression.right));
        }
        if ts_ast::is_assignment_operator(operator)
            && !ts_ast::is_assignment_target(self.view(), node).expect("retained assignment")
        {
            self.bind_assignment_target_flow(need(expression.left));
            if operator == K::EqualsToken
                && self.n(need(expression.left)).kind() == K::ElementAccessExpression
            {
                let target = need(self.n(need(expression.left)).expression());
                if self.is_narrowable_operand(target) {
                    self.current_flow = Some(self.create_flow_mutation(
                        F::ARRAY_MUTATION,
                        need(self.current_flow),
                        node,
                    ));
                }
            }
        }
    }
}
