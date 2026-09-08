//! Binding scope transitions and allocation-free syntax-child traversal.
use crate::{need, Binder, ContainerFlags as C};
use std::ops::ControlFlow;
use ts_arena::Error;
use ts_ast::{
    flow_flags as F, modifier_flags, node_flags as N, symbol_flags as S, utilities as u, AstView,
    ChildVisitor, FlowData, FlowId, JsString, Node, NodeData, NodeId, NodeListId, NodeSlice,
    SyntaxKind as K,
};

// port: tsc/internal/binder/binder.go:GetContainerFlags
pub fn get_container_flags(view: AstView<'_>, id: NodeId) -> Result<C, Error> {
    let node = view.node(id)?;
    let function =
        C::IS_CONTAINER | C::IS_CONTROL_FLOW_CONTAINER | C::HAS_LOCALS | C::IS_FUNCTION_LIKE;
    Ok(C(match node.kind().known() {
        Some(
            K::ClassExpression
            | K::ClassDeclaration
            | K::EnumDeclaration
            | K::ObjectLiteralExpression
            | K::TypeLiteral
            | K::JsxAttributes,
        ) => C::IS_CONTAINER,
        Some(K::InterfaceDeclaration) => C::IS_CONTAINER | C::IS_INTERFACE,
        Some(
            K::ModuleDeclaration
            | K::TypeAliasDeclaration
            | K::JSTypeAliasDeclaration
            | K::MappedType
            | K::IndexSignature,
        ) => C::IS_CONTAINER | C::HAS_LOCALS,
        Some(K::SourceFile) => C::IS_CONTAINER | C::IS_CONTROL_FLOW_CONTAINER | C::HAS_LOCALS,
        Some(K::GetAccessor | K::SetAccessor | K::MethodDeclaration) => {
            function
                | C::IS_THIS_CONTAINER
                | if u::is_object_literal_or_class_expression_method_or_accessor(view, id)? {
                    C::IS_OBJECT_LITERAL_OR_CLASS_EXPRESSION_METHOD_OR_ACCESSOR
                } else {
                    0
                }
        }
        Some(K::Constructor | K::FunctionDeclaration | K::ClassStaticBlockDeclaration) => {
            function | C::IS_THIS_CONTAINER
        }
        Some(
            K::MethodSignature
            | K::CallSignature
            | K::FunctionType
            | K::ConstructSignature
            | K::ConstructorType,
        ) => function | C::PROPAGATES_THIS_KEYWORD,
        Some(K::FunctionExpression) => function | C::IS_FUNCTION_EXPRESSION | C::IS_THIS_CONTAINER,
        Some(K::ArrowFunction) => function | C::IS_FUNCTION_EXPRESSION | C::PROPAGATES_THIS_KEYWORD,
        Some(K::ModuleBlock) => C::IS_CONTROL_FLOW_CONTAINER,
        Some(K::PropertyDeclaration) if node.initializer().is_some() => {
            C::IS_CONTROL_FLOW_CONTAINER | C::IS_THIS_CONTAINER
        }
        Some(
            K::CatchClause | K::ForStatement | K::ForInStatement | K::ForOfStatement | K::CaseBlock,
        ) => C::IS_BLOCK_SCOPED_CONTAINER | C::HAS_LOCALS,
        Some(K::Block) => {
            let parent = view.node(need(node.parent()))?;
            if u::is_function_like(Some(&parent)) || parent.kind() == K::ClassStaticBlockDeclaration
            {
                0
            } else {
                C::IS_BLOCK_SCOPED_CONTAINER | C::HAS_LOCALS
            }
        }
        _ => C::NONE,
    }))
}

// FlowNodeData is a payload interface. Open SyntaxKind values can disagree with
// the payload, so this follows the generated Go embedding graph, not kind ranges.
pub(crate) fn has_flow_node_data(node: &Node) -> bool {
    matches!(
        node.data(),
        NodeData::Identifier(_)
            | NodeData::QualifiedName(_)
            | NodeData::EmptyStatement(_)
            | NodeData::IfStatement(_)
            | NodeData::DoStatement(_)
            | NodeData::WhileStatement(_)
            | NodeData::ForStatement(_)
            | NodeData::ForInOrOfStatement(_)
            | NodeData::BreakStatement(_)
            | NodeData::ContinueStatement(_)
            | NodeData::ReturnStatement(_)
            | NodeData::WithStatement(_)
            | NodeData::SwitchStatement(_)
            | NodeData::ThrowStatement(_)
            | NodeData::TryStatement(_)
            | NodeData::DebuggerStatement(_)
            | NodeData::LabeledStatement(_)
            | NodeData::ExpressionStatement(_)
            | NodeData::Block(_)
            | NodeData::VariableStatement(_)
            | NodeData::BindingElement(_)
            | NodeData::MissingDeclaration(_)
            | NodeData::FunctionDeclaration(_)
            | NodeData::ClassDeclaration(_)
            | NodeData::InterfaceDeclaration(_)
            | NodeData::TypeAliasDeclaration(_)
            | NodeData::EnumDeclaration(_)
            | NodeData::ModuleBlock(_)
            | NodeData::NotEmittedStatement(_)
            | NodeData::ImportDeclaration(_)
            | NodeData::ExportAssignment(_)
            | NodeData::NamespaceExportDeclaration(_)
            | NodeData::GetAccessorDeclaration(_)
            | NodeData::SetAccessorDeclaration(_)
            | NodeData::MethodDeclaration(_)
            | NodeData::KeywordExpression(_)
            | NodeData::ArrowFunction(_)
            | NodeData::FunctionExpression(_)
            | NodeData::PropertyAccessExpression(_)
            | NodeData::ElementAccessExpression(_)
            | NodeData::MetaProperty(_)
            | NodeData::ModuleDeclaration(_)
            | NodeData::ImportEqualsDeclaration(_)
            | NodeData::ExportDeclaration(_)
    )
}
fn has_body_data(node: &Node) -> bool {
    matches!(
        node.data(),
        NodeData::FunctionDeclaration(_)
            | NodeData::ConstructorDeclaration(_)
            | NodeData::GetAccessorDeclaration(_)
            | NodeData::SetAccessorDeclaration(_)
            | NodeData::MethodDeclaration(_)
            | NodeData::ArrowFunction(_)
            | NodeData::FunctionExpression(_)
            | NodeData::ModuleDeclaration(_)
    )
}

impl<'ast> Binder<'_, 'ast> {
    pub(crate) fn syntax_nodes(&self, list: Option<NodeListId>) -> ts_ast::NodeSliceRead<'ast> {
        let parsed = self.parsed_view();
        let nodes = list.map_or_else(NodeSlice::empty, |list| {
            parsed.list(list).expect("retained syntax list").nodes()
        });
        parsed.node_slice(nodes).expect("retained syntax slice")
    }
    // port: tsc/internal/binder/binder.go:Binder.bindContainer
    pub(crate) fn bind_container(&mut self, node: NodeId, flags: C) {
        let flags = flags.0;
        let saved_container = self.container;
        let saved_this = self.this_container;
        let saved_block = self.block_scope_container;
        if flags & C::IS_CONTAINER != 0 {
            self.container = Some(node);
            self.block_scope_container = Some(node);
            if flags & C::HAS_LOCALS != 0 {
                self.add_to_container_chain(node);
            }
        } else if flags & C::IS_BLOCK_SCOPED_CONTAINER != 0 {
            self.block_scope_container = Some(node);
            self.add_to_container_chain(node);
        }
        if flags & C::IS_THIS_CONTAINER != 0 {
            self.this_container = Some(node);
        }
        if flags & C::IS_CONTROL_FLOW_CONTAINER != 0 {
            let saved_flow = self.current_flow;
            let saved_break = self.current_break_target;
            let saved_continue = self.current_continue_target;
            let saved_return = self.current_return_target;
            let saved_exception = self.current_exception_target;
            let saved_labels = self.active_label_list;
            let saved_explicit_return = self.has_explicit_return;
            let saved_seen_this = self.seen_this_keyword;
            let immediately_invoked = flags & C::IS_FUNCTION_EXPRESSION != 0
                && !u::has_syntactic_modifier(self.view(), node, modifier_flags::ASYNC)
                    .expect("retained function modifiers")
                && !self.is_generator_function_expression(node)
                && ts_ast::get_immediately_invoked_function_expression(self.view(), node)
                    .expect("retained IIFE")
                    .is_some()
                || self.n(node).kind() == K::ClassStaticBlockDeclaration;
            if !immediately_invoked {
                let start = self.new_flow_node(F::START);
                self.current_flow = Some(start);
                if flags
                    & (C::IS_FUNCTION_EXPRESSION
                        | C::IS_OBJECT_LITERAL_OR_CLASS_EXPRESSION_METHOD_OR_ACCESSOR)
                    != 0
                {
                    self.flow_mut(start).node = Some(FlowData::Ast(node));
                }
            }
            self.current_return_target =
                if immediately_invoked || self.n(node).kind() == K::Constructor {
                    Some(self.new_flow_node(F::BRANCH_LABEL))
                } else {
                    None
                };
            self.current_exception_target = None;
            self.current_break_target = None;
            self.current_continue_target = None;
            self.active_label_list = None;
            self.has_explicit_return = false;
            self.seen_this_keyword = false;
            self.bind_children(node);
            let mut node_flags =
                self.n(node).flags() & !(N::REACHABILITY_AND_EMIT_FLAGS | N::CONTAINS_THIS);
            if self.flow(need(self.current_flow)).flags & F::UNREACHABLE == 0
                && flags & C::IS_FUNCTION_LIKE != 0
                && has_body_data(&self.n(node))
            {
                let body = self.n(node).body();
                if body.is_some_and(|body| ts_ast::node_is_present(Some(&self.n(body)))) {
                    node_flags |= N::HAS_IMPLICIT_RETURN;
                    if self.has_explicit_return {
                        node_flags |= N::HAS_EXPLICIT_RETURN;
                    }
                    self.binding_mut(node).end_flow_node = self.current_flow;
                }
            }
            if self.seen_this_keyword {
                node_flags |= N::CONTAINS_THIS;
            }
            if self.n(node).kind() == K::SourceFile {
                node_flags |= self.emit_flags;
            }
            self.set_flags(node, node_flags);
            if let Some(target) = self.current_return_target {
                self.add_antecedent(target, need(self.current_flow));
                self.current_flow = Some(self.finish_flow_label(target));
                if matches!(
                    self.n(node).kind().known(),
                    Some(K::Constructor | K::ClassStaticBlockDeclaration)
                ) {
                    self.set_return_flow_node(node, self.current_flow);
                }
            }
            if !immediately_invoked {
                self.current_flow = saved_flow;
            }
            self.current_break_target = saved_break;
            self.current_continue_target = saved_continue;
            self.current_return_target = saved_return;
            self.current_exception_target = saved_exception;
            self.active_label_list = saved_labels;
            self.has_explicit_return = saved_explicit_return;
            self.seen_this_keyword = if flags & C::PROPAGATES_THIS_KEYWORD != 0 {
                saved_seen_this || self.seen_this_keyword
            } else {
                saved_seen_this
            };
        } else if flags & C::IS_INTERFACE != 0 {
            let saved_seen_this = self.seen_this_keyword;
            self.seen_this_keyword = false;
            self.bind_children(node);
            let flags = if self.seen_this_keyword {
                self.n(node).flags() | N::CONTAINS_THIS
            } else {
                self.n(node).flags() & !N::CONTAINS_THIS
            };
            self.set_flags(node, flags);
            self.seen_this_keyword = saved_seen_this;
        } else {
            self.bind_children(node);
        }
        if self.n(node).kind() == K::SourceFile && u::is_in_js_file(Some(&self.n(node))) {
            let parsed = self.parsed_view();
            let list = parsed
                .list(need(self.n(node).statement_list()))
                .expect("source statement list");
            let statements = parsed
                .node_slice(list.nodes())
                .expect("source statement slice");
            for &statement in &*statements {
                let statement = need(statement);
                if self.n(statement).kind() == K::JSTypeAliasDeclaration {
                    self.bind_block_scoped_declaration(
                        statement,
                        S::TYPE_ALIAS,
                        S::TYPE_ALIAS_EXCLUDES,
                    );
                }
            }
            if self
                .view()
                .source_file(self.file)
                .expect("binder source")
                .common_js_module_indicator()
                .is_some()
            {
                self.declare_common_js_variable(JsString::from_bytes(b"module".as_slice()));
                self.declare_common_js_variable(JsString::from_bytes(b"exports".as_slice()));
            }
        }
        if self.n(node).kind() == K::SourceFile
            && u::is_external_or_common_js_module(
                &self.view().source_file(node).expect("source file"),
            )
            || ts_ast::is_ambient_module(self.view(), node).expect("retained module")
        {
            self.bind_common_js_type_exports(need(self.symbol(node)));
        }
        self.container = saved_container;
        self.this_container = saved_this;
        self.block_scope_container = saved_block;
    }
    // port: tsc/internal/binder/binder.go:Binder.bindChildren
    pub(crate) fn bind_children(&mut self, node: NodeId) {
        let saved_pattern = self.in_assignment_pattern;
        self.in_assignment_pattern = false;
        if self.current_flow == self.unreachable_flow {
            self.set_flow_node(node, None);
            if ts_ast::is_potentially_executable_node(self.view(), node)
                .expect("retained executable node")
            {
                self.set_flags(node, self.n(node).flags() | N::UNREACHABLE);
            }
            self.bind_each_child(node);
            self.in_assignment_pattern = saved_pattern;
            return;
        }
        let kind = self.n(node).kind();
        if kind.raw() >= K::FirstStatement as i16 && kind.raw() <= K::LastStatement as i16 {
            self.set_flow_node(node, self.current_flow);
        }
        match kind.known() {
            Some(K::WhileStatement) => self.bind_while_statement(node),
            Some(K::DoStatement) => self.bind_do_statement(node),
            Some(K::ForStatement) => self.bind_for_statement(node),
            Some(K::ForInStatement | K::ForOfStatement) => {
                self.bind_for_in_or_for_of_statement(node);
            }
            Some(K::IfStatement) => self.bind_if_statement(node),
            Some(K::ReturnStatement) => self.bind_return_statement(node),
            Some(K::ThrowStatement) => self.bind_throw_statement(node),
            Some(K::BreakStatement) => self.bind_break_statement(node),
            Some(K::ContinueStatement) => self.bind_continue_statement(node),
            Some(K::TryStatement) => self.bind_try_statement(node),
            Some(K::SwitchStatement) => self.bind_switch_statement(node),
            Some(K::CaseBlock) => self.bind_case_block(node),
            Some(K::CaseClause | K::DefaultClause) => self.bind_case_or_default_clause(node),
            Some(K::ExpressionStatement) => self.bind_expression_statement(node),
            Some(K::LabeledStatement) => self.bind_labeled_statement(node),
            Some(K::PrefixUnaryExpression) => self.bind_prefix_unary_expression_flow(node),
            Some(K::PostfixUnaryExpression) => self.bind_postfix_unary_expression_flow(node),
            Some(K::BinaryExpression) => {
                if ts_ast::is_destructuring_assignment(self.view(), node)
                    .expect("retained destructuring expression")
                {
                    self.in_assignment_pattern = saved_pattern;
                    self.bind_destructuring_assignment_flow(node);
                    return;
                }
                self.bind_binary_expression_flow(node);
            }
            Some(K::DeleteExpression) => self.bind_delete_expression_flow(node),
            Some(K::ConditionalExpression) => self.bind_conditional_expression_flow(node),
            Some(K::VariableDeclaration) => self.bind_variable_declaration_flow(node),
            Some(K::PropertyAccessExpression | K::ElementAccessExpression) => {
                self.bind_access_expression_flow(node);
            }
            Some(K::CallExpression) => self.bind_call_expression_flow(node),
            Some(K::NonNullExpression) => self.bind_non_null_expression_flow(node),
            Some(K::SourceFile) => {
                let n = self.n(node);
                let source = n.data().as_source_file().expect("SourceFile payload");
                let statements = source.statements;
                let eof = source.end_of_file_token;
                drop(n);
                self.bind_each_statement_functions_first(need(statements));
                self.bind(eof);
            }
            Some(K::Block | K::ModuleBlock) => {
                self.bind_each_statement_functions_first(need(self.n(node).statement_list()));
            }
            Some(K::BindingElement) => self.bind_binding_element_flow(node),
            Some(K::Parameter) => self.bind_parameter_flow(node),
            Some(
                K::ObjectLiteralExpression
                | K::ArrayLiteralExpression
                | K::PropertyAssignment
                | K::SpreadElement,
            ) => {
                self.in_assignment_pattern = saved_pattern;
                self.bind_each_child(node);
            }
            _ => self.bind_each_child(node),
        }
        self.in_assignment_pattern = saved_pattern;
    }
    // port: tsc/internal/binder/binder.go:Binder.bindEachChild
    pub(crate) fn bind_each_child(&mut self, node: NodeId) {
        let parsed = self.parsed_view();
        let node = parsed.node(node).expect("binder syntax is retained");
        let _ = node.for_each_child(self);
    }
    // port: tsc/internal/binder/binder.go:Binder.bindEach
    pub(crate) fn bind_each(&mut self, nodes: &[Option<NodeId>]) {
        for &node in nodes {
            self.bind(node);
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.bindNodeList
    pub(crate) fn bind_node_list(&mut self, list: Option<NodeListId>) {
        if let Some(list) = list {
            let parsed = self.parsed_view();
            let list = parsed.list(list).expect("retained syntax list");
            let nodes = parsed
                .node_slice(list.nodes())
                .expect("retained syntax slice");
            self.bind_each(&nodes);
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.bindModifiers
    pub(crate) fn bind_modifiers(&mut self, list: Option<NodeListId>) {
        self.bind_node_list(list);
    }
    // port: tsc/internal/binder/binder.go:Binder.bindEachStatementFunctionsFirst
    pub(crate) fn bind_each_statement_functions_first(&mut self, statements: NodeListId) {
        let parsed = self.parsed_view();
        let list = parsed.list(statements).expect("retained statement list");
        let nodes = parsed
            .node_slice(list.nodes())
            .expect("retained statement slice");
        for &node in &*nodes {
            if self.n(need(node)).kind() == K::FunctionDeclaration {
                self.bind(node);
            }
        }
        for &node in &*nodes {
            if self.n(need(node)).kind() != K::FunctionDeclaration {
                self.bind(node);
            }
        }
    }
    // port: tsc/internal/binder/binder.go:setFlowNode
    pub(crate) fn set_flow_node(&mut self, node: NodeId, flow: Option<FlowId>) {
        if has_flow_node_data(&self.n(node)) {
            self.builder
                .set_node_flow(node, flow)
                .expect("binder flow and target belong to result");
        }
    }
    // port: tsc/internal/binder/binder.go:setReturnFlowNode
    pub(crate) fn set_return_flow_node(&mut self, node: NodeId, flow: Option<FlowId>) {
        match self.n(node).kind().known() {
            Some(K::Constructor) => {
                self.n(node)
                    .data()
                    .as_constructor_declaration()
                    .expect("Constructor payload");
            }
            Some(K::FunctionDeclaration) => {
                self.n(node)
                    .data()
                    .as_function_declaration()
                    .expect("FunctionDeclaration payload");
            }
            Some(K::FunctionExpression) => {
                self.n(node)
                    .data()
                    .as_function_expression()
                    .expect("FunctionExpression payload");
            }
            Some(K::ClassStaticBlockDeclaration) => {
                self.n(node)
                    .data()
                    .as_class_static_block_declaration()
                    .expect("ClassStaticBlockDeclaration payload");
            }
            _ => return,
        }
        self.binding_mut(node).return_flow_node = flow;
    }
    // port: tsc/internal/binder/binder.go:isGeneratorFunctionExpression
    pub(crate) fn is_generator_function_expression(&self, node: NodeId) -> bool {
        self.n(node).kind() == K::FunctionExpression
            && self
                .n(node)
                .data()
                .as_function_expression()
                .expect("FunctionExpression payload")
                .asterisk_token
                .is_some()
    }
    // port: tsc/internal/binder/binder.go:Binder.addToContainerChain
    pub(crate) fn add_to_container_chain(&mut self, next: NodeId) {
        if let Some(last) = self.last_container {
            assert!(
                ts_ast::is_locals_container(&self.n(last)),
                "locals-container payload required"
            );
            self.binding_mut(last).next_container = Some(next);
        }
        self.last_container = Some(next);
    }
}
impl ChildVisitor for Binder<'_, '_> {
    fn visit_node(&mut self, node: NodeId) -> ControlFlow<()> {
        if self.bind(Some(node)) {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
    fn visit_list(&mut self, list: NodeListId) -> ControlFlow<()> {
        let parsed = self.parsed_view();
        let list = parsed.list(list).expect("retained child list");
        self.visit_node_slice(list.nodes())
    }
    fn visit_node_slice(&mut self, nodes: NodeSlice) -> ControlFlow<()> {
        let parsed = self.parsed_view();
        let nodes = parsed.node_slice(nodes).expect("retained child slice");
        for &node in &*nodes {
            if self.bind(node) {
                return ControlFlow::Break(());
            }
        }
        ControlFlow::Continue(())
    }
}
