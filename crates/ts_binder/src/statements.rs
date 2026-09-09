//! Statement control-flow and label handling from the pinned binder.
use crate::{expressions::payload, need, ActiveLabel, Binder};
use ts_ast::{flow_flags as F, node_flags, FlowId, JsString, NodeId, SyntaxKind as K};

impl Binder<'_, '_> {
    // port: tsc/internal/binder/binder.go:Binder.setContinueTarget
    pub(crate) fn set_continue_target(&mut self, mut node: NodeId, target: FlowId) -> FlowId {
        let mut label = self.active_label_list;
        while let Some(index) = label {
            let parent = need(self.n(node).parent());
            if self.n(parent).kind() != K::LabeledStatement {
                break;
            }
            self.labels[index].continue_target = Some(target);
            label = self.labels[index].next;
            node = parent;
        }
        target
    }
    // port: tsc/internal/binder/binder.go:Binder.doWithConditionalBranches
    pub(crate) fn do_with_conditional_branches(
        &mut self,
        action: fn(&mut Self, Option<NodeId>) -> bool,
        value: Option<NodeId>,
        true_target: FlowId,
        false_target: FlowId,
    ) {
        let saved_true = self.current_true_target;
        let saved_false = self.current_false_target;
        self.current_true_target = Some(true_target);
        self.current_false_target = Some(false_target);
        action(self, value);
        self.current_true_target = saved_true;
        self.current_false_target = saved_false;
    }
    // port: tsc/internal/binder/binder.go:Binder.bindCondition
    pub(crate) fn bind_condition(
        &mut self,
        node: Option<NodeId>,
        true_target: FlowId,
        false_target: FlowId,
    ) {
        self.do_with_conditional_branches(Self::bind, node, true_target, false_target);
        if node.is_none_or(|node| {
            !(self.is_logical_assignment_expression(node)
                || ts_ast::utilities::is_logical_expression(self.view(), node)
                    .expect("retained condition")
                || ts_ast::utilities::is_optional_chain(&self.n(node))
                    && ts_ast::utilities::is_outermost_optional_chain(self.view(), node)
                        .expect("retained condition"))
        }) {
            let yes = self.create_flow_condition(F::TRUE_CONDITION, need(self.current_flow), node);
            self.add_antecedent(true_target, yes);
            let no = self.create_flow_condition(F::FALSE_CONDITION, need(self.current_flow), node);
            self.add_antecedent(false_target, no);
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.bindIterativeStatement
    pub(crate) fn bind_iterative_statement(
        &mut self,
        node: Option<NodeId>,
        break_target: FlowId,
        continue_target: FlowId,
    ) {
        let saved_break = self.current_break_target;
        let saved_continue = self.current_continue_target;
        self.current_break_target = Some(break_target);
        self.current_continue_target = Some(continue_target);
        self.bind(node);
        self.current_break_target = saved_break;
        self.current_continue_target = saved_continue;
    }
    // port: tsc/internal/binder/binder.go:Binder.bindWhileStatement
    pub(crate) fn bind_while_statement(&mut self, node: NodeId) {
        let statement = payload!(self, node, as_while_statement);
        let pre = self.create_loop_label();
        let pre = self.set_continue_target(node, pre);
        let body = self.create_branch_label();
        let post = self.create_branch_label();
        self.add_antecedent(pre, need(self.current_flow));
        self.current_flow = Some(pre);
        self.bind_condition(statement.expression, body, post);
        self.current_flow = Some(self.finish_flow_label(body));
        self.bind_iterative_statement(statement.statement, post, pre);
        self.add_antecedent(pre, need(self.current_flow));
        self.current_flow = Some(self.finish_flow_label(post));
    }
    // port: tsc/internal/binder/binder.go:Binder.bindDoStatement
    pub(crate) fn bind_do_statement(&mut self, node: NodeId) {
        let statement = payload!(self, node, as_do_statement);
        let pre = self.create_loop_label();
        let condition = self.create_branch_label();
        let condition = self.set_continue_target(node, condition);
        let post = self.create_branch_label();
        self.add_antecedent(pre, need(self.current_flow));
        self.current_flow = Some(pre);
        self.bind_iterative_statement(statement.statement, post, condition);
        self.add_antecedent(condition, need(self.current_flow));
        self.current_flow = Some(self.finish_flow_label(condition));
        self.bind_condition(statement.expression, pre, post);
        self.current_flow = Some(self.finish_flow_label(post));
    }
    // port: tsc/internal/binder/binder.go:Binder.bindForStatement
    pub(crate) fn bind_for_statement(&mut self, node: NodeId) {
        let statement = payload!(self, node, as_for_statement);
        self.bind(statement.initializer);
        // Preserve the source's early exit: an unreachable initializer must not
        // leave a loop graph with only its cyclic incrementor antecedent.
        if self.current_flow == self.unreachable_flow {
            self.bind(statement.condition);
            self.bind(statement.statement);
            self.bind(statement.incrementor);
            return;
        }
        let pre = self.create_loop_label();
        let pre = self.set_continue_target(node, pre);
        let body = self.create_branch_label();
        let incrementor = self.create_branch_label();
        let post = self.create_branch_label();
        self.add_antecedent(pre, need(self.current_flow));
        self.current_flow = Some(pre);
        self.bind_condition(statement.condition, body, post);
        self.current_flow = Some(self.finish_flow_label(body));
        self.bind_iterative_statement(statement.statement, post, incrementor);
        self.add_antecedent(incrementor, need(self.current_flow));
        self.current_flow = Some(self.finish_flow_label(incrementor));
        self.bind(statement.incrementor);
        self.add_antecedent(pre, need(self.current_flow));
        self.current_flow = Some(self.finish_flow_label(post));
    }
    // port: tsc/internal/binder/binder.go:Binder.bindForInOrForOfStatement
    pub(crate) fn bind_for_in_or_for_of_statement(&mut self, node: NodeId) {
        let statement = payload!(self, node, as_for_in_or_of_statement);
        self.bind(statement.expression);
        if self.current_flow == self.unreachable_flow {
            self.bind(statement.initializer);
            self.bind(statement.statement);
            return;
        }
        let pre = self.create_loop_label();
        let pre = self.set_continue_target(node, pre);
        let post = self.create_branch_label();
        self.add_antecedent(pre, need(self.current_flow));
        self.current_flow = Some(pre);
        if self.n(node).kind() == K::ForOfStatement {
            self.bind(statement.await_modifier);
        }
        self.add_antecedent(post, need(self.current_flow));
        self.bind(statement.initializer);
        if self.n(need(statement.initializer)).kind() != K::VariableDeclarationList {
            self.bind_assignment_target_flow(need(statement.initializer));
        }
        self.bind_iterative_statement(statement.statement, post, pre);
        self.add_antecedent(pre, need(self.current_flow));
        self.current_flow = Some(self.finish_flow_label(post));
    }
    // port: tsc/internal/binder/binder.go:Binder.bindIfStatement
    pub(crate) fn bind_if_statement(&mut self, node: NodeId) {
        let statement = payload!(self, node, as_if_statement);
        let yes = self.create_branch_label();
        let no = self.create_branch_label();
        let post = self.create_branch_label();
        self.bind_condition(statement.expression, yes, no);
        self.current_flow = Some(self.finish_flow_label(yes));
        self.bind(statement.then_statement);
        self.add_antecedent(post, need(self.current_flow));
        self.current_flow = Some(self.finish_flow_label(no));
        self.bind(statement.else_statement);
        self.add_antecedent(post, need(self.current_flow));
        self.current_flow = Some(self.finish_flow_label(post));
    }
    // port: tsc/internal/binder/binder.go:Binder.bindReturnStatement
    pub(crate) fn bind_return_statement(&mut self, node: NodeId) {
        self.bind(self.n(node).expression());
        if let Some(target) = self.current_return_target {
            self.add_antecedent(target, need(self.current_flow));
        }
        self.current_flow = self.unreachable_flow;
        self.has_explicit_return = true;
        self.has_flow_effects = true;
    }
    // port: tsc/internal/binder/binder.go:Binder.bindThrowStatement
    pub(crate) fn bind_throw_statement(&mut self, node: NodeId) {
        self.bind(self.n(node).expression());
        self.current_flow = self.unreachable_flow;
        self.has_flow_effects = true;
    }
    // port: tsc/internal/binder/binder.go:Binder.bindBreakStatement
    pub(crate) fn bind_break_statement(&mut self, node: NodeId) {
        self.bind_break_or_continue_statement(
            self.n(node).label(),
            self.current_break_target,
            |label| label.break_target,
        );
    }
    // port: tsc/internal/binder/binder.go:Binder.bindContinueStatement
    pub(crate) fn bind_continue_statement(&mut self, node: NodeId) {
        self.bind_break_or_continue_statement(
            self.n(node).label(),
            self.current_continue_target,
            |label| label.continue_target,
        );
    }
    // port: tsc/internal/binder/binder.go:Binder.bindBreakOrContinueStatement
    pub(crate) fn bind_break_or_continue_statement(
        &mut self,
        label: Option<NodeId>,
        current: Option<FlowId>,
        target: fn(&ActiveLabel) -> Option<FlowId>,
    ) {
        self.bind(label);
        if let Some(label) = label {
            if let Some(index) = self.find_active_label(&self.text(label)) {
                self.labels[index].referenced = true;
                self.bind_break_or_continue_flow(target(&self.labels[index]));
            }
        } else {
            self.bind_break_or_continue_flow(current);
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.findActiveLabel
    pub(crate) fn find_active_label(&self, name: &JsString) -> Option<usize> {
        let mut label = self.active_label_list;
        while let Some(index) = label {
            if self.labels[index].name == *name {
                return Some(index);
            }
            label = self.labels[index].next;
        }
        None
    }
    // port: tsc/internal/binder/binder.go:Binder.bindBreakOrContinueFlow
    pub(crate) fn bind_break_or_continue_flow(&mut self, target: Option<FlowId>) {
        if let Some(target) = target {
            self.add_antecedent(target, need(self.current_flow));
            self.current_flow = self.unreachable_flow;
            self.has_flow_effects = true;
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.bindTryStatement
    pub(crate) fn bind_try_statement(&mut self, node: NodeId) {
        let statement = payload!(self, node, as_try_statement);
        let saved_return = self.current_return_target;
        let saved_exception = self.current_exception_target;
        let normal = self.create_branch_label();
        let returned = self.create_branch_label();
        let mut exception = self.create_branch_label();
        if statement.finally_block.is_some() {
            self.current_return_target = Some(returned);
        }
        self.add_antecedent(exception, need(self.current_flow));
        self.current_exception_target = Some(exception);
        self.bind(statement.try_block);
        self.add_antecedent(normal, need(self.current_flow));
        if statement.catch_clause.is_some() {
            self.current_flow = Some(self.finish_flow_label(exception));
            exception = self.create_branch_label();
            self.add_antecedent(exception, need(self.current_flow));
            self.current_exception_target = Some(exception);
            self.bind(statement.catch_clause);
            self.add_antecedent(normal, need(self.current_flow));
        }
        self.current_return_target = saved_return;
        self.current_exception_target = saved_exception;
        if statement.finally_block.is_some() {
            let finally = self.create_branch_label();
            let exceptions_and_returns = self.combine_flow_lists(
                self.flow(exception).antecedents(),
                self.flow(returned).antecedents(),
            );
            let all =
                self.combine_flow_lists(self.flow(normal).antecedents(), exceptions_and_returns);
            self.set_flow_antecedents(finally, all);
            self.current_flow = Some(finally);
            self.bind(statement.finally_block);
            if self.flow(need(self.current_flow)).flags() & F::UNREACHABLE != 0 {
                self.current_flow = self.unreachable_flow;
            } else {
                if let Some(target) = self
                    .current_return_target
                    .filter(|_| self.flow(returned).antecedents().is_some())
                {
                    let reduced = self.create_reduce_label(
                        finally,
                        self.flow(returned).antecedents(),
                        need(self.current_flow),
                    );
                    self.add_antecedent(target, reduced);
                }
                if let Some(target) = self
                    .current_exception_target
                    .filter(|_| self.flow(exception).antecedents().is_some())
                {
                    let reduced = self.create_reduce_label(
                        finally,
                        self.flow(exception).antecedents(),
                        need(self.current_flow),
                    );
                    self.add_antecedent(target, reduced);
                }
                self.current_flow = if self.flow(normal).antecedents().is_some() {
                    Some(self.create_reduce_label(
                        finally,
                        self.flow(normal).antecedents(),
                        need(self.current_flow),
                    ))
                } else {
                    self.unreachable_flow
                };
            }
        } else {
            self.current_flow = Some(self.finish_flow_label(normal));
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.bindSwitchStatement
    pub(crate) fn bind_switch_statement(&mut self, node: NodeId) {
        let statement = payload!(self, node, as_switch_statement);
        let post = self.create_branch_label();
        self.bind(statement.expression);
        let saved_break = self.current_break_target;
        let saved_pre = self.pre_switch_case_flow;
        self.current_break_target = Some(post);
        self.pre_switch_case_flow = self.current_flow;
        self.bind(statement.case_block);
        self.add_antecedent(post, need(self.current_flow));
        let case_block = payload!(self, need(statement.case_block), as_case_block);
        let has_default = self
            .syntax_nodes(Some(need(case_block.clauses)))
            .iter()
            .any(|clause| self.n(need(clause)).kind() == K::DefaultClause);
        if !has_default {
            let clause =
                self.create_flow_switch_clause(need(self.pre_switch_case_flow), node, 0, 0);
            self.add_antecedent(post, clause);
        }
        self.current_break_target = saved_break;
        self.pre_switch_case_flow = saved_pre;
        self.current_flow = Some(self.finish_flow_label(post));
    }
    // port: tsc/internal/binder/binder.go:Binder.bindCaseBlock
    pub(crate) fn bind_case_block(&mut self, node: NodeId) {
        let statement = need(self.n(node).parent());
        let expression = need(self.n(statement).expression());
        let clauses = self.syntax_slice(Some(need(payload!(self, node, as_case_block).clauses)));
        let narrowing =
            self.n(expression).kind() == K::TrueKeyword || self.is_narrowing_expression(expression);
        let mut fallthrough = self.unreachable_flow;
        let mut index = 0;
        while index < clauses.len() {
            let start = index;
            while self
                .syntax_nodes(
                    self.n(need(self.syntax_node(clauses, index)))
                        .statement_list(),
                )
                .is_empty()
                && index + 1 < clauses.len()
            {
                if fallthrough == self.unreachable_flow {
                    self.current_flow = self.pre_switch_case_flow;
                }
                self.bind(self.syntax_node(clauses, index));
                index += 1;
            }
            let pre_case = self.create_branch_label();
            let pre_flow = if narrowing {
                self.create_flow_switch_clause(
                    need(self.pre_switch_case_flow),
                    statement,
                    start as i64,
                    (index + 1) as i64,
                )
            } else {
                need(self.pre_switch_case_flow)
            };
            self.add_antecedent(pre_case, pre_flow);
            self.add_antecedent(pre_case, need(fallthrough));
            self.current_flow = Some(self.finish_flow_label(pre_case));
            let clause = need(self.syntax_node(clauses, index));
            self.bind(Some(clause));
            fallthrough = self.current_flow;
            if self.flow(need(self.current_flow)).flags() & F::UNREACHABLE == 0
                && index != clauses.len() - 1
            {
                self.set_node_fallthrough_flow(clause, self.current_flow);
            }
            index += 1;
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.bindCaseOrDefaultClause
    pub(crate) fn bind_case_or_default_clause(&mut self, node: NodeId) {
        let clause = payload!(self, node, as_case_or_default_clause);
        if clause.expression.is_some() {
            let saved = self.current_flow;
            self.current_flow = self.pre_switch_case_flow;
            self.bind(clause.expression);
            self.current_flow = saved;
        }
        self.bind_node_list(Some(need(clause.statements)));
    }
    // port: tsc/internal/binder/binder.go:Binder.bindExpressionStatement
    pub(crate) fn bind_expression_statement(&mut self, node: NodeId) {
        let expression = self.n(node).expression();
        self.bind(expression);
        self.maybe_bind_expression_flow_if_call(need(expression));
    }
    // port: tsc/internal/binder/binder.go:Binder.bindLabeledStatement
    pub(crate) fn bind_labeled_statement(&mut self, node: NodeId) {
        let statement = payload!(self, node, as_labeled_statement);
        let post = self.create_branch_label();
        let index = self.labels.len();
        self.labels.push(ActiveLabel {
            next: self.active_label_list,
            name: self.text(need(statement.label)),
            break_target: Some(post),
            continue_target: None,
            referenced: false,
        });
        self.active_label_list = Some(index);
        self.bind(statement.label);
        self.bind(statement.statement);
        let active = need(self.active_label_list);
        if !self.labels[active].referenced {
            let label = need(statement.label);
            self.set_flags(label, self.n(label).flags() | node_flags::UNREACHABLE);
        }
        self.active_label_list = self.labels[active].next;
        self.add_antecedent(post, need(self.current_flow));
        self.current_flow = Some(self.finish_flow_label(post));
    }
}
