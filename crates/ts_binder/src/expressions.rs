//! Expression evaluation order and narrowing predicates from the pinned binder.
use crate::{need, Binder};
use ts_ast::{flow_flags as F, node_flags, utilities as u, FlowId, NodeId, SyntaxKind as K};

// These payloads contain only non-owning identities and scalar fields. Snapshot
// their fields before mutating the binding overlay, without retaining AST owners.
macro_rules! payload {
    ($b:expr, $node:expr, $accessor:ident) => {
        $b.n($node)
            .data()
            .$accessor()
            .expect("binder syntax payload")
            .clone()
    };
}
pub(crate) use payload;

impl Binder<'_, '_> {
    // port: tsc/internal/binder/binder.go:Binder.bindAssignmentTargetFlow
    pub(crate) fn bind_assignment_target_flow(&mut self, node: NodeId) {
        match self.n(node).kind().known() {
            Some(K::ArrayLiteralExpression) => {
                let elements = self.n(node).element_list();
                for &element in &*self.syntax_nodes(elements) {
                    let element = need(element);
                    if self.n(element).kind() == K::SpreadElement {
                        self.bind_assignment_target_flow(need(self.n(element).expression()));
                    } else {
                        self.bind_destructuring_target_flow(element);
                    }
                }
            }
            Some(K::ObjectLiteralExpression) => {
                for &property in &*self.syntax_nodes(self.n(node).property_list()) {
                    let property = need(property);
                    match self.n(property).kind().known() {
                        Some(K::PropertyAssignment) => self
                            .bind_destructuring_target_flow(need(self.n(property).initializer())),
                        Some(K::ShorthandPropertyAssignment) => {
                            self.bind_assignment_target_flow(need(self.n(property).name()));
                        }
                        Some(K::SpreadAssignment) => {
                            self.bind_assignment_target_flow(need(self.n(property).expression()));
                        }
                        _ => {}
                    }
                }
            }
            _ => {
                if self.is_narrowable_reference(node) {
                    self.current_flow = Some(self.create_flow_mutation(
                        F::ASSIGNMENT,
                        need(self.current_flow),
                        node,
                    ));
                }
            }
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.bindDestructuringTargetFlow
    pub(crate) fn bind_destructuring_target_flow(&mut self, node: NodeId) {
        if self.n(node).kind() == K::BinaryExpression {
            let binary = payload!(self, node, as_binary_expression);
            if self.n(need(binary.operator_token)).kind() == K::EqualsToken {
                self.bind_assignment_target_flow(need(binary.left));
                return;
            }
        }
        self.bind_assignment_target_flow(node);
    }
    // port: tsc/internal/binder/binder.go:Binder.maybeBindExpressionFlowIfCall
    pub(crate) fn maybe_bind_expression_flow_if_call(&mut self, node: NodeId) {
        if self.n(node).kind() == K::CallExpression {
            let expression = need(self.n(node).expression());
            if self.n(expression).kind() != K::SuperKeyword
                && ts_ast::is_dotted_name(self.view(), expression).expect("retained dotted name")
            {
                self.current_flow = Some(self.create_flow_call(need(self.current_flow), node));
            }
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.bindPrefixUnaryExpressionFlow
    pub(crate) fn bind_prefix_unary_expression_flow(&mut self, node: NodeId) {
        let expression = payload!(self, node, as_prefix_unary_expression);
        if expression.operator == K::ExclamationToken {
            let saved_true = self.current_true_target;
            std::mem::swap(
                &mut self.current_true_target,
                &mut self.current_false_target,
            );
            self.bind_each_child(node);
            self.current_false_target = self.current_true_target;
            self.current_true_target = saved_true;
        } else {
            self.bind_each_child(node);
            if matches!(
                expression.operator.known(),
                Some(K::PlusPlusToken | K::MinusMinusToken)
            ) {
                self.bind_assignment_target_flow(need(expression.operand));
            }
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.bindPostfixUnaryExpressionFlow
    pub(crate) fn bind_postfix_unary_expression_flow(&mut self, node: NodeId) {
        let expression = payload!(self, node, as_postfix_unary_expression);
        self.bind_each_child(node);
        if matches!(
            expression.operator.known(),
            Some(K::PlusPlusToken | K::MinusMinusToken)
        ) {
            self.bind_assignment_target_flow(need(expression.operand));
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.bindDestructuringAssignmentFlow
    pub(crate) fn bind_destructuring_assignment_flow(&mut self, node: NodeId) {
        let expression = payload!(self, node, as_binary_expression);
        if self.in_assignment_pattern {
            self.in_assignment_pattern = false;
            self.bind(expression.operator_token);
            self.bind(expression.right);
            self.in_assignment_pattern = true;
            self.bind(expression.left);
            self.bind(expression.r#type);
        } else {
            self.in_assignment_pattern = true;
            self.bind(expression.left);
            self.bind(expression.r#type);
            self.in_assignment_pattern = false;
            self.bind(expression.operator_token);
            self.bind(expression.right);
        }
        self.bind_assignment_target_flow(need(expression.left));
    }
    // port: tsc/internal/binder/binder.go:Binder.bindBinaryExpressionFlow
    pub(crate) fn bind_binary_expression_flow(&mut self, node: NodeId) {
        let expression = payload!(self, node, as_binary_expression);
        let operator = self.n(need(expression.operator_token)).kind();
        if u::is_logical_or_coalescing_binary_operator(operator)
            || ts_ast::is_logical_or_coalescing_assignment_operator(operator)
        {
            if self.is_top_level_logical_expression(node) {
                let post = self.create_branch_label();
                let saved_flow = self.current_flow;
                let saved_effects = self.has_flow_effects;
                self.has_flow_effects = false;
                self.bind_logical_like_expression(node, post, post);
                self.current_flow = if self.has_flow_effects {
                    Some(self.finish_flow_label(post))
                } else {
                    saved_flow
                };
                self.has_flow_effects |= saved_effects;
            } else {
                self.bind_logical_like_expression(
                    node,
                    need(self.current_true_target),
                    need(self.current_false_target),
                );
            }
        } else {
            self.bind_binary_expression_trampoline(node);
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.bindLogicalLikeExpression
    pub(crate) fn bind_logical_like_expression(
        &mut self,
        node: NodeId,
        true_target: FlowId,
        false_target: FlowId,
    ) {
        let expression = payload!(self, node, as_binary_expression);
        let operator = self.n(need(expression.operator_token)).kind();
        let pre_right = self.create_branch_label();
        if matches!(
            operator.known(),
            Some(K::AmpersandAmpersandToken | K::AmpersandAmpersandEqualsToken)
        ) {
            self.bind_condition(expression.left, pre_right, false_target);
        } else {
            self.bind_condition(expression.left, true_target, pre_right);
        }
        self.current_flow = Some(self.finish_flow_label(pre_right));
        self.bind(expression.operator_token);
        if ts_ast::is_logical_or_coalescing_assignment_operator(operator) {
            self.do_with_conditional_branches(
                Self::bind,
                expression.right,
                true_target,
                false_target,
            );
            self.bind_assignment_target_flow(need(expression.left));
            let true_flow =
                self.create_flow_condition(F::TRUE_CONDITION, need(self.current_flow), Some(node));
            self.add_antecedent(true_target, true_flow);
            let false_flow =
                self.create_flow_condition(F::FALSE_CONDITION, need(self.current_flow), Some(node));
            self.add_antecedent(false_target, false_flow);
        } else {
            self.bind_condition(expression.right, true_target, false_target);
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.bindDeleteExpressionFlow
    pub(crate) fn bind_delete_expression_flow(&mut self, node: NodeId) {
        let expression = need(self.n(node).expression());
        self.bind_each_child(node);
        if self.n(expression).kind() == K::PropertyAccessExpression {
            self.bind_assignment_target_flow(expression);
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.bindConditionalExpressionFlow
    pub(crate) fn bind_conditional_expression_flow(&mut self, node: NodeId) {
        let expression = payload!(self, node, as_conditional_expression);
        let true_label = self.create_branch_label();
        let false_label = self.create_branch_label();
        let post = self.create_branch_label();
        let saved_flow = self.current_flow;
        let saved_effects = self.has_flow_effects;
        self.has_flow_effects = false;
        self.bind_condition(expression.condition, true_label, false_label);
        self.current_flow = Some(self.finish_flow_label(true_label));
        self.bind(expression.question_token);
        self.bind(expression.when_true);
        self.add_antecedent(post, need(self.current_flow));
        self.current_flow = Some(self.finish_flow_label(false_label));
        self.bind(expression.colon_token);
        self.bind(expression.when_false);
        self.add_antecedent(post, need(self.current_flow));
        self.current_flow = if self.has_flow_effects {
            Some(self.finish_flow_label(post))
        } else {
            saved_flow
        };
        self.has_flow_effects |= saved_effects;
    }
    // port: tsc/internal/binder/binder.go:Binder.bindVariableDeclarationFlow
    pub(crate) fn bind_variable_declaration_flow(&mut self, node: NodeId) {
        self.bind_each_child(node);
        if self.n(node).initializer().is_some()
            || u::is_for_in_or_of_statement(
                self.n(need(self.n(node).parent()))
                    .parent()
                    .map(|id| self.n(id))
                    .as_deref(),
            )
        {
            self.bind_initialized_variable_flow(node);
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.bindInitializedVariableFlow
    pub(crate) fn bind_initialized_variable_flow(&mut self, node: NodeId) {
        let name = if matches!(
            self.n(node).kind().known(),
            Some(K::VariableDeclaration | K::BindingElement)
        ) {
            self.n(node).name()
        } else {
            None
        };
        if let Some(name) = name.filter(|&name| u::is_binding_pattern(&self.n(name))) {
            for &child in &*self.syntax_nodes(self.n(name).element_list()) {
                self.bind_initialized_variable_flow(need(child));
            }
        } else {
            self.current_flow =
                Some(self.create_flow_mutation(F::ASSIGNMENT, need(self.current_flow), node));
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.bindAccessExpressionFlow
    pub(crate) fn bind_access_expression_flow(&mut self, node: NodeId) {
        if u::is_optional_chain(&self.n(node)) {
            self.bind_optional_chain_flow(node);
        } else {
            self.bind_each_child(node);
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.bindOptionalChainFlow
    pub(crate) fn bind_optional_chain_flow(&mut self, node: NodeId) {
        if self.is_top_level_logical_expression(node) {
            let post = self.create_branch_label();
            let saved_flow = self.current_flow;
            let saved_effects = self.has_flow_effects;
            // The source deliberately does not clear hasFlowEffects on this path.
            self.bind_optional_chain(node, post, post);
            self.current_flow = if self.has_flow_effects {
                Some(self.finish_flow_label(post))
            } else {
                saved_flow
            };
            self.has_flow_effects |= saved_effects;
        } else {
            self.bind_optional_chain(
                node,
                need(self.current_true_target),
                need(self.current_false_target),
            );
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.bindOptionalChain
    pub(crate) fn bind_optional_chain(
        &mut self,
        node: NodeId,
        true_target: FlowId,
        false_target: FlowId,
    ) {
        let pre_chain = if u::is_optional_chain_root(&self.n(node)) {
            Some(self.create_branch_label())
        } else {
            None
        };
        self.bind_optional_expression(
            need(self.n(node).expression()),
            pre_chain.unwrap_or(true_target),
            false_target,
        );
        if let Some(pre_chain) = pre_chain {
            self.current_flow = Some(self.finish_flow_label(pre_chain));
        }
        self.do_with_conditional_branches(
            Self::bind_optional_chain_rest,
            Some(node),
            true_target,
            false_target,
        );
        if u::is_outermost_optional_chain(self.view(), node).expect("retained optional chain") {
            let yes =
                self.create_flow_condition(F::TRUE_CONDITION, need(self.current_flow), Some(node));
            self.add_antecedent(true_target, yes);
            let no =
                self.create_flow_condition(F::FALSE_CONDITION, need(self.current_flow), Some(node));
            self.add_antecedent(false_target, no);
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.bindOptionalExpression
    pub(crate) fn bind_optional_expression(
        &mut self,
        node: NodeId,
        true_target: FlowId,
        false_target: FlowId,
    ) {
        self.do_with_conditional_branches(Self::bind, Some(node), true_target, false_target);
        if !u::is_optional_chain(&self.n(node))
            || u::is_outermost_optional_chain(self.view(), node).expect("retained optional chain")
        {
            let yes =
                self.create_flow_condition(F::TRUE_CONDITION, need(self.current_flow), Some(node));
            self.add_antecedent(true_target, yes);
            let no =
                self.create_flow_condition(F::FALSE_CONDITION, need(self.current_flow), Some(node));
            self.add_antecedent(false_target, no);
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.bindOptionalChainRest
    pub(crate) fn bind_optional_chain_rest(&mut self, node: Option<NodeId>) -> bool {
        let node = need(node);
        match self.n(node).kind().known() {
            Some(K::PropertyAccessExpression) => {
                self.bind(self.n(node).question_dot_token());
                self.bind(self.n(node).name());
            }
            Some(K::ElementAccessExpression) => {
                let data = payload!(self, node, as_element_access_expression);
                self.bind(data.question_dot_token);
                self.bind(data.argument_expression);
            }
            Some(K::CallExpression) => {
                let data = payload!(self, node, as_call_expression);
                self.bind(data.question_dot_token);
                self.bind_node_list(data.type_arguments);
                self.bind_node_list(data.arguments);
            }
            _ => {}
        }
        false
    }
    // port: tsc/internal/binder/binder.go:Binder.bindCallExpressionFlow
    pub(crate) fn bind_call_expression_flow(&mut self, node: NodeId) {
        let call = payload!(self, node, as_call_expression);
        let expression = need(call.expression);
        if u::is_optional_chain(&self.n(node)) {
            self.bind_optional_chain_flow(node);
        } else {
            let target = ts_ast::skip_parentheses(self.view(), expression)
                .expect("retained call expression");
            if matches!(
                self.n(target).kind().known(),
                Some(K::FunctionExpression | K::ArrowFunction)
            ) {
                self.bind_node_list(call.type_arguments);
                self.bind_node_list(Some(need(call.arguments)));
                self.bind(call.expression);
            } else {
                self.bind_each_child(node);
                if self.n(expression).kind() == K::SuperKeyword {
                    self.current_flow = Some(self.create_flow_call(need(self.current_flow), node));
                }
            }
        }
        if self.n(expression).kind() == K::PropertyAccessExpression {
            let access = payload!(self, expression, as_property_access_expression);
            if self.n(need(access.name)).kind() == K::Identifier
                && self.is_narrowable_operand(need(access.expression))
                && ts_ast::is_push_or_unshift_identifier(self.view(), need(access.name))
                    .expect("retained method name")
            {
                self.current_flow = Some(self.create_flow_mutation(
                    F::ARRAY_MUTATION,
                    need(self.current_flow),
                    node,
                ));
            }
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.bindNonNullExpressionFlow
    pub(crate) fn bind_non_null_expression_flow(&mut self, node: NodeId) {
        self.bind_access_expression_flow(node);
    }
    // port: tsc/internal/binder/binder.go:Binder.bindBindingElementFlow
    pub(crate) fn bind_binding_element_flow(&mut self, node: NodeId) {
        let element = payload!(self, node, as_binding_element);
        self.bind(element.dot_dot_dot_token);
        self.bind(element.property_name);
        self.bind_initializer(element.initializer);
        self.bind(element.name);
    }
    // port: tsc/internal/binder/binder.go:Binder.bindParameterFlow
    pub(crate) fn bind_parameter_flow(&mut self, node: NodeId) {
        let parameter = payload!(self, node, as_parameter_declaration);
        self.bind_modifiers(parameter.modifiers);
        self.bind(parameter.dot_dot_dot_token);
        self.bind(parameter.question_token);
        self.bind(parameter.r#type);
        self.bind_initializer(parameter.initializer);
        self.bind(parameter.name);
    }
    // port: tsc/internal/binder/binder.go:Binder.bindInitializer
    pub(crate) fn bind_initializer(&mut self, node: Option<NodeId>) {
        if node.is_none() {
            return;
        }
        let entry = self.current_flow;
        self.bind(node);
        if entry == self.unreachable_flow || entry == self.current_flow {
            return;
        }
        let exit = self.create_branch_label();
        self.add_antecedent(exit, need(entry));
        self.add_antecedent(exit, need(self.current_flow));
        self.current_flow = Some(self.finish_flow_label(exit));
    }
    // port: tsc/internal/binder/binder.go:isNarrowingExpression
    pub(crate) fn is_narrowing_expression(&self, node: NodeId) -> bool {
        match self.n(node).kind().known() {
            Some(K::Identifier | K::ThisKeyword) => true,
            Some(K::PropertyAccessExpression | K::ElementAccessExpression) => {
                self.contains_narrowable_reference(node)
            }
            Some(K::CallExpression) => self.has_narrowable_argument(node),
            Some(K::ParenthesizedExpression | K::NonNullExpression | K::TypeOfExpression) => {
                self.is_narrowing_expression(need(self.n(node).expression()))
            }
            Some(K::BinaryExpression) => self.is_narrowing_binary_expression(node),
            Some(K::PrefixUnaryExpression) => {
                let p = payload!(self, node, as_prefix_unary_expression);
                p.operator == K::ExclamationToken && self.is_narrowing_expression(need(p.operand))
            }
            _ => false,
        }
    }
    // port: tsc/internal/binder/binder.go:containsNarrowableReference
    pub(crate) fn contains_narrowable_reference(&self, node: NodeId) -> bool {
        if self.is_narrowable_reference(node) {
            return true;
        }
        self.n(node).flags() & node_flags::OPTIONAL_CHAIN != 0
            && matches!(
                self.n(node).kind().known(),
                Some(
                    K::PropertyAccessExpression
                        | K::ElementAccessExpression
                        | K::CallExpression
                        | K::NonNullExpression
                )
            )
            && self.contains_narrowable_reference(need(self.n(node).expression()))
    }
    // port: tsc/internal/binder/binder.go:isNarrowableReference
    pub(crate) fn is_narrowable_reference(&self, node: NodeId) -> bool {
        match self.n(node).kind().known() {
            Some(K::Identifier | K::ThisKeyword | K::SuperKeyword | K::MetaProperty) => true,
            Some(
                K::PropertyAccessExpression | K::ParenthesizedExpression | K::NonNullExpression,
            ) => self.is_narrowable_reference(need(self.n(node).expression())),
            Some(K::ElementAccessExpression) => {
                let access = payload!(self, node, as_element_access_expression);
                u::is_string_or_numeric_literal_like(&self.n(need(access.argument_expression)))
                    || ts_ast::is_entity_name_expression(
                        self.view(),
                        need(access.argument_expression),
                    )
                    .expect("retained element name")
                        && self.is_narrowable_reference(need(access.expression))
            }
            Some(K::BinaryExpression) => {
                let binary = payload!(self, node, as_binary_expression);
                let operator = self.n(need(binary.operator_token)).kind();
                operator == K::CommaToken && self.is_narrowable_reference(need(binary.right))
                    || ts_ast::is_assignment_operator(operator)
                        && ts_ast::is_left_hand_side_expression(self.view(), need(binary.left))
                            .expect("retained assignment target")
            }
            _ => false,
        }
    }
    // port: tsc/internal/binder/binder.go:hasNarrowableArgument
    pub(crate) fn has_narrowable_argument(&self, node: NodeId) -> bool {
        let call = payload!(self, node, as_call_expression);
        for &argument in &*self.syntax_nodes(Some(need(call.arguments))) {
            if self.contains_narrowable_reference(need(argument)) {
                return true;
            }
        }
        self.n(need(call.expression)).kind() == K::PropertyAccessExpression
            && self.contains_narrowable_reference(need(self.n(need(call.expression)).expression()))
    }
    // port: tsc/internal/binder/binder.go:isNarrowingBinaryExpression
    pub(crate) fn is_narrowing_binary_expression(&self, node: NodeId) -> bool {
        let binary = payload!(self, node, as_binary_expression);
        let left = need(binary.left);
        let right = need(binary.right);
        match self.n(need(binary.operator_token)).kind().known() {
            Some(
                K::EqualsToken
                | K::BarBarEqualsToken
                | K::AmpersandAmpersandEqualsToken
                | K::QuestionQuestionEqualsToken,
            ) => self.contains_narrowable_reference(left),
            Some(
                K::EqualsEqualsToken
                | K::ExclamationEqualsToken
                | K::EqualsEqualsEqualsToken
                | K::ExclamationEqualsEqualsToken,
            ) => {
                let left =
                    ts_ast::skip_parentheses(self.view(), left).expect("retained equality operand");
                let right = ts_ast::skip_parentheses(self.view(), right)
                    .expect("retained equality operand");
                self.is_narrowable_operand(left)
                    || self.is_narrowable_operand(right)
                    || self.is_narrowing_type_of_operands(right, left)
                    || self.is_narrowing_type_of_operands(left, right)
                    || u::is_boolean_literal(&self.n(right)) && self.is_narrowing_expression(left)
                    || u::is_boolean_literal(&self.n(left)) && self.is_narrowing_expression(right)
            }
            Some(K::InstanceOfKeyword) => self.is_narrowable_operand(left),
            Some(K::InKeyword | K::CommaToken) => self.is_narrowing_expression(right),
            _ => false,
        }
    }
    // port: tsc/internal/binder/binder.go:isNarrowableOperand
    pub(crate) fn is_narrowable_operand(&self, node: NodeId) -> bool {
        match self.n(node).kind().known() {
            Some(K::ParenthesizedExpression) => {
                return self.is_narrowable_operand(need(self.n(node).expression()))
            }
            Some(K::BinaryExpression) => {
                let binary = payload!(self, node, as_binary_expression);
                match self.n(need(binary.operator_token)).kind().known() {
                    Some(K::EqualsToken) => return self.is_narrowable_operand(need(binary.left)),
                    Some(K::CommaToken) => return self.is_narrowable_operand(need(binary.right)),
                    _ => {}
                }
            }
            _ => {}
        }
        self.contains_narrowable_reference(node)
    }
    // port: tsc/internal/binder/binder.go:isNarrowingTypeOfOperands
    pub(crate) fn is_narrowing_type_of_operands(&self, first: NodeId, second: NodeId) -> bool {
        self.n(first).kind() == K::TypeOfExpression
            && self.is_narrowable_operand(need(self.n(first).expression()))
            && u::is_string_literal_like(&self.n(second))
    }
    // port: tsc/internal/binder/binder.go:isStatementCondition
    pub(crate) fn is_statement_condition(&self, node: NodeId) -> bool {
        let parent = need(self.n(node).parent());
        match self.n(parent).kind().known() {
            Some(K::IfStatement | K::WhileStatement | K::DoStatement) => {
                self.n(parent).expression() == Some(node)
            }
            Some(K::ForStatement) => {
                payload!(self, parent, as_for_statement).condition == Some(node)
            }
            Some(K::ConditionalExpression) => {
                payload!(self, parent, as_conditional_expression).condition == Some(node)
            }
            _ => false,
        }
    }
    // port: tsc/internal/binder/binder.go:isTopLevelLogicalExpression
    pub(crate) fn is_top_level_logical_expression(&self, mut node: NodeId) -> bool {
        loop {
            let parent = need(self.n(node).parent());
            if self.n(parent).kind() == K::ParenthesizedExpression
                || self.n(parent).kind() == K::PrefixUnaryExpression
                    && payload!(self, parent, as_prefix_unary_expression).operator
                        == K::ExclamationToken
            {
                node = parent;
            } else {
                break;
            }
        }
        let parent = need(self.n(node).parent());
        !(self.is_statement_condition(node)
            || u::is_logical_expression(self.view(), parent).expect("retained logical parent")
            || u::is_optional_chain(&self.n(parent)) && self.n(parent).expression() == Some(node))
    }
    // port: tsc/internal/binder/binder.go:isLogicalAssignmentExpression
    pub(crate) fn is_logical_assignment_expression(&self, node: NodeId) -> bool {
        u::is_logical_or_coalescing_assignment_expression(
            self.view(),
            ts_ast::skip_parentheses(self.view(), node).expect("retained logical assignment"),
        )
        .expect("retained logical assignment")
    }
}
