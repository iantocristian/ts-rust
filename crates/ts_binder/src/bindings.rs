use crate::{ast as a, checked, need, Binder};
use ts_ast::{
    internal_symbol_names as names, node_flags as nf, symbol_flags as sf, JsString, NodeId,
    SymbolTable, SyntaxKind as K,
};
use ts_diagnostics as d;

impl Binder<'_, '_> {
    // port: tsc/internal/binder/binder.go:Binder.bindPropertyWorker
    pub fn bind_property_worker(&mut self, node: NodeId) {
        let accessor = checked(a::is_auto_accessor_property_declaration(self.view(), node));
        self.bind_property_or_method_or_accessor(
            node,
            if accessor { sf::ACCESSOR } else { sf::PROPERTY } | self.optional_symbol_flag(node),
            if accessor {
                sf::ACCESSOR_EXCLUDES
            } else {
                sf::PROPERTY_EXCLUDES
            },
        );
    }
    // port: tsc/internal/binder/binder.go:Binder.bindPropertyOrMethodOrAccessor
    pub fn bind_property_or_method_or_accessor(&mut self, node: NodeId, flags: u32, excludes: u32) {
        self.record_async_function(node);
        if self.current_flow.is_some()
            && checked(a::is_object_literal_or_class_expression_method_or_accessor(
                self.view(),
                node,
            ))
        {
            self.set_flow_node(node, self.current_flow);
        }
        if checked(a::has_dynamic_name(self.view(), Some(node))) {
            self.bind_anonymous_declaration(node, flags, JsString::from_bytes(names::COMPUTED));
        } else {
            self.declare_symbol_and_add_to_symbol_table(node, flags, excludes);
        }
    }
    fn record_async_function(&mut self, node: NodeId) {
        if !checked(self.view().source_file(self.file)).is_declaration_file
            && self.n(node).flags() & nf::AMBIENT == 0
            && checked(a::is_async_function(self.view(), node))
        {
            self.emit_flags |= nf::HAS_ASYNC_FUNCTIONS;
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.bindFunctionExpression
    pub fn bind_function_expression(&mut self, node: NodeId) {
        self.record_async_function(node);
        self.set_flow_node(node, self.current_flow);
        let name = if self.n(node).kind() == K::FunctionExpression && self.n(node).name().is_some()
        {
            self.check_strict_mode_function_name(node);
            self.text(need(self.n(node).name()))
        } else {
            JsString::from_bytes(names::FUNCTION)
        };
        self.bind_anonymous_declaration(node, sf::FUNCTION, name);
    }
    // port: tsc/internal/binder/binder.go:Binder.bindClassLikeDeclaration
    pub fn bind_class_like_declaration(&mut self, node: NodeId) {
        if self.n(node).kind() == K::ClassDeclaration {
            self.bind_block_scoped_declaration(node, sf::CLASS, sf::CLASS_EXCLUDES);
        } else if self.n(node).kind() == K::ClassExpression {
            let name = self.n(node).name().map_or_else(
                || JsString::from_bytes(names::CLASS),
                |name| self.text(name),
            );
            self.bind_anonymous_declaration(node, sf::CLASS, name);
        }
        let symbol = need(self.symbol(node));
        let prototype = self.new_symbol(
            sf::PROPERTY | sf::PROTOTYPE,
            JsString::from_bytes(&b"prototype"[..]),
        );
        let table = self.ensure_exports(symbol);
        if let Some(previous) = self.table(table).get(b"prototype".as_slice()).flatten() {
            let declaration = need(
                checked(
                    self.builder
                        .declarations()
                        .get(self.s(previous).declarations()),
                )
                .at(0),
            );
            self.error_on_node(
                declaration,
                d::Duplicate_identifier_0,
                vec![self.s(prototype).name_to_owned()],
            );
        }
        self.table_mut(table)
            .insert(JsString::from_bytes(&b"prototype"[..]), Some(prototype));
        self.set_symbol_parent(prototype, Some(symbol));
    }
    // port: tsc/internal/binder/binder.go:Binder.bindFunctionOrConstructorType
    pub fn bind_function_or_constructor_type(&mut self, node: NodeId) {
        let symbol = self.new_symbol(sf::SIGNATURE, self.get_declaration_name(node));
        self.add_declaration_to_symbol(symbol, node, sf::SIGNATURE);
        let type_literal = self.new_symbol(sf::TYPE_LITERAL, JsString::from_bytes(names::TYPE));
        self.add_declaration_to_symbol(type_literal, node, sf::TYPE_LITERAL);
        let name = self.s(symbol).name_to_owned();
        let table = self
            .builder
            .tables_mut()
            .alloc(SymbolTable::from([(name, Some(symbol))]));
        self.set_symbol_members(type_literal, Some(table));
    }
    // port: tsc/internal/binder/binder.go:Binder.bindEnumDeclaration
    pub fn bind_enum_declaration(&mut self, node: NodeId) {
        if checked(a::is_enum_const(self.view(), node)) {
            self.bind_block_scoped_declaration(node, sf::CONST_ENUM, sf::CONST_ENUM_EXCLUDES);
        } else {
            self.bind_block_scoped_declaration(node, sf::REGULAR_ENUM, sf::REGULAR_ENUM_EXCLUDES);
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.bindVariableDeclarationOrBindingElement
    pub fn bind_variable_declaration_or_binding_element(&mut self, node: NodeId) {
        self.check_strict_mode_eval_or_arguments(node, self.n(node).name());
        if self
            .n(node)
            .name()
            .is_some_and(|name| !a::is_binding_pattern(&self.n(name)))
        {
            if checked(a::is_variable_declaration_initialized_to_require(
                self.view(),
                node,
            )) {
                self.declare_symbol_and_add_to_symbol_table(node, sf::ALIAS, sf::ALIAS_EXCLUDES);
            } else if checked(a::is_block_or_catch_scoped(self.view(), node)) {
                self.bind_block_scoped_declaration(
                    node,
                    sf::BLOCK_SCOPED_VARIABLE,
                    sf::BLOCK_SCOPED_VARIABLE_EXCLUDES,
                );
            } else if checked(a::is_part_of_parameter_declaration(self.view(), node)) {
                self.declare_symbol_and_add_to_symbol_table(
                    node,
                    sf::FUNCTION_SCOPED_VARIABLE,
                    sf::PARAMETER_EXCLUDES,
                );
            } else {
                self.declare_symbol_and_add_to_symbol_table(
                    node,
                    sf::FUNCTION_SCOPED_VARIABLE,
                    sf::FUNCTION_SCOPED_VARIABLE_EXCLUDES,
                );
            }
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.bindParameter
    pub fn bind_parameter(&mut self, node: NodeId) {
        if self.n(node).flags() & nf::AMBIENT == 0 {
            self.check_strict_mode_eval_or_arguments(node, self.n(node).name());
        }
        let parent = need(self.n(node).parent());
        if a::is_binding_pattern(&self.n(need(self.n(node).name()))) {
            let index = checked(
                self.view()
                    .node_slice(checked(self.n(parent).parameters(self.view()))),
            )
            .iter()
            .position(|id| id == Some(node))
            .map_or(-1, |index| index as isize);
            self.bind_anonymous_declaration(
                node,
                sf::FUNCTION_SCOPED_VARIABLE,
                JsString::from_bytes(format!("__{index}").into_bytes()),
            );
        } else {
            self.declare_symbol_and_add_to_symbol_table(
                node,
                sf::FUNCTION_SCOPED_VARIABLE,
                sf::PARAMETER_EXCLUDES,
            );
        }
        if checked(a::is_parameter_property_declaration(
            self.view(),
            node,
            parent,
        )) {
            let class = need(self.n(parent).parent());
            let symbol = need(self.symbol(class));
            let flags = sf::PROPERTY
                | if self
                    .n(node)
                    .data_source()
                    .as_parameter_declaration()
                    .expect("parameter payload")
                    .question_token()
                    .is_some()
                {
                    sf::OPTIONAL
                } else {
                    0
                };
            let table = self.ensure_members(symbol);
            self.declare_symbol(table, Some(symbol), node, flags, sf::PROPERTY_EXCLUDES);
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.bindFunctionDeclaration
    pub fn bind_function_declaration(&mut self, node: NodeId) {
        self.record_async_function(node);
        self.check_strict_mode_function_name(node);
        self.bind_block_scoped_declaration(node, sf::FUNCTION, sf::FUNCTION_EXCLUDES);
    }
    // port: tsc/internal/binder/binder.go:Binder.getInferTypeContainer
    pub fn get_infer_type_container(&self, node: NodeId) -> Option<NodeId> {
        let mut current = Some(node);
        while let Some(node) = current {
            let parent = self.n(node).parent();
            if parent.is_some_and(|parent| {
                self.n(parent).kind() == K::ConditionalType
                    && self
                        .n(parent)
                        .data_source()
                        .as_conditional_type_node()
                        .expect("conditional type payload")
                        .extends_type()
                        == Some(node)
            }) {
                return parent;
            }
            current = parent;
        }
        None
    }
    // port: tsc/internal/binder/binder.go:Binder.bindAnonymousDeclaration
    pub fn bind_anonymous_declaration(&mut self, node: NodeId, flags: u32, name: JsString) {
        let symbol = self.new_symbol(flags, name);
        if flags & (sf::ENUM_MEMBER | sf::CLASS_MEMBER) != 0 {
            self.set_symbol_parent(symbol, self.symbol(need(self.container)));
        }
        self.add_declaration_to_symbol(symbol, node, flags);
    }
    // port: tsc/internal/binder/binder.go:Binder.bindBlockScopedDeclaration
    pub fn bind_block_scoped_declaration(&mut self, node: NodeId, flags: u32, excludes: u32) {
        let block = need(self.block_scope_container);
        if self.n(block).kind() == K::ModuleDeclaration
            || self.n(block).kind() == K::SourceFile
                && a::is_external_or_common_js_module(&checked(
                    self.view().source_file(need(self.container)),
                ))
        {
            self.declare_module_member(node, flags, excludes);
        } else {
            let table = self.ensure_locals(block);
            self.declare_symbol(table, None, node, flags, excludes);
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.bindTypeParameter
    pub fn bind_type_parameter(&mut self, node: NodeId) {
        let parent = need(self.n(node).parent());
        if self.n(parent).kind() == K::InferType {
            if let Some(container) = self.get_infer_type_container(parent) {
                let table = self.ensure_locals(container);
                self.declare_symbol(
                    table,
                    None,
                    node,
                    sf::TYPE_PARAMETER,
                    sf::TYPE_PARAMETER_EXCLUDES,
                );
            } else {
                self.bind_anonymous_declaration(
                    node,
                    sf::TYPE_PARAMETER,
                    self.get_declaration_name(node),
                );
            }
        } else {
            self.declare_symbol_and_add_to_symbol_table(
                node,
                sf::TYPE_PARAMETER,
                sf::TYPE_PARAMETER_EXCLUDES,
            );
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.bindJsxAttributes
    pub fn bind_jsx_attributes(&mut self, node: NodeId) {
        self.bind_anonymous_declaration(
            node,
            sf::OBJECT_LITERAL,
            JsString::from_bytes(names::JSX_ATTRIBUTES),
        );
    }
    // port: tsc/internal/binder/binder.go:Binder.bindJsxAttribute
    pub fn bind_jsx_attribute(&mut self, node: NodeId, flags: u32, excludes: u32) {
        self.declare_symbol_and_add_to_symbol_table(node, flags, excludes);
    }
    // port: tsc/internal/binder/binder.go:getOptionalSymbolFlagForNode
    pub fn optional_symbol_flag(&self, node: NodeId) -> u32 {
        if self
            .n(node)
            .postfix_token()
            .is_some_and(|token| self.n(token).kind() == K::QuestionToken)
        {
            sf::OPTIONAL
        } else {
            sf::NONE
        }
    }
}
