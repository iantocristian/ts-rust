use crate::{ast as a, checked, need, Binder};
use ts_ast::{node_flags as nf, AstView, Diagnostic, JsString, NodeId, SyntaxKind as K};
use ts_diagnostics::{self as d, Message};

impl Binder<'_, '_> {
    // port: tsc/internal/binder/binder.go:Binder.errorOnNode
    pub fn error_on_node(&mut self, node: NodeId, message: &'static Message, args: Vec<JsString>) {
        let diagnostic = self.create_diagnostic_for_node(node, message, args);
        self.add_diagnostic(diagnostic);
    }
    // port: tsc/internal/binder/binder.go:Binder.errorOnFirstToken
    pub fn error_on_first_token(
        &mut self,
        node: NodeId,
        message: &'static Message,
        args: Vec<JsString>,
    ) {
        let range = checked(ts_scanner::get_range_of_token_at_position(
            self.view(),
            self.file,
            i64::from(self.n(node).pos()),
        ));
        self.add_diagnostic(Diagnostic::new(Some(self.file), range, message, args));
    }
    // port: tsc/internal/binder/binder.go:Binder.createDiagnosticForNode
    pub fn create_diagnostic_for_node(
        &self,
        node: NodeId,
        message: &'static Message,
        args: Vec<JsString>,
    ) -> Diagnostic {
        Diagnostic::new(
            Some(self.file),
            checked(ts_scanner::get_error_range_for_node(
                self.view(),
                self.file,
                node,
            )),
            message,
            args,
        )
    }
    // port: tsc/internal/binder/binder.go:Binder.addDiagnostic
    pub fn add_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.builder.diagnostics_mut().push(diagnostic);
    }
    // port: tsc/internal/binder/binder.go:Binder.checkContextualIdentifier
    pub fn check_contextual_identifier(&mut self, node: NodeId) {
        if checked(self.view().source_file(self.file))
            .diagnostics
            .is_empty()
            && self.n(node).flags() & (nf::AMBIENT | nf::JS_DOC) == 0
            && !checked(a::is_identifier_name(self.view(), node))
        {
            let keyword = ts_scanner::get_identifier_token(self.text(node).as_bytes());
            if keyword == K::Identifier {
                return;
            }
            let message = if keyword >= K::FirstFutureReservedWord
                && keyword <= K::LastFutureReservedWord
            {
                Some(self.get_strict_mode_identifier_message(node))
            } else if keyword == K::AwaitKeyword {
                if checked(self.view().source_file(self.file))
                    .external_module_indicator
                    .is_some()
                    && checked(a::is_in_top_level_context(self.view(), node))
                {
                    Some(d::Identifier_expected_0_is_a_reserved_word_at_the_top_level_of_a_module)
                } else if self.n(node).flags() & nf::AWAIT_CONTEXT != 0 {
                    Some(d::Identifier_expected_0_is_a_reserved_word_that_cannot_be_used_here)
                } else {
                    None
                }
            } else if keyword == K::YieldKeyword && self.n(node).flags() & nf::YIELD_CONTEXT != 0 {
                Some(d::Identifier_expected_0_is_a_reserved_word_that_cannot_be_used_here)
            } else {
                None
            };
            if let Some(message) = message {
                self.error_on_node(
                    node,
                    message,
                    vec![checked(ts_scanner::declaration_name_to_string(
                        self.view(),
                        Some(node),
                    ))],
                );
            }
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.checkPrivateIdentifier
    pub fn check_private_identifier(&mut self, node: NodeId) {
        if self.text(node).as_bytes() == b"#constructor"
            && checked(self.view().source_file(self.file))
                .diagnostics
                .is_empty()
        {
            self.error_on_node(
                node,
                d::X_constructor_is_a_reserved_word,
                vec![checked(ts_scanner::declaration_name_to_string(
                    self.view(),
                    Some(node),
                ))],
            );
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.getStrictModeIdentifierMessage
    pub fn get_strict_mode_identifier_message(&self, node: NodeId) -> &'static Message {
        if checked(a::get_containing_class(self.view(), node)).is_some() {
            d::Identifier_expected_0_is_a_reserved_word_in_strict_mode_Class_definitions_are_automatically_in_strict_mode
        } else if checked(self.view().source_file(self.file))
            .external_module_indicator
            .is_some()
        {
            d::Identifier_expected_0_is_a_reserved_word_in_strict_mode_Modules_are_automatically_in_strict_mode
        } else {
            d::Identifier_expected_0_is_a_reserved_word_in_strict_mode
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.checkStrictModeFunctionName
    pub fn check_strict_mode_function_name(&mut self, node: NodeId) {
        if self.n(node).flags() & nf::AMBIENT == 0 {
            self.check_strict_mode_eval_or_arguments(node, self.n(node).name());
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.checkStrictModeBinaryExpression
    pub fn check_strict_mode_binary_expression(&mut self, node: NodeId) {
        let data = self
            .n(node)
            .data()
            .as_binary_expression()
            .expect("binary payload")
            .clone();
        if checked(a::is_left_hand_side_expression(
            self.view(),
            need(data.left),
        )) && a::is_assignment_operator(self.n(need(data.operator_token)).kind())
        {
            self.check_strict_mode_eval_or_arguments(node, data.left);
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.checkStrictModeCatchClause
    pub fn check_strict_mode_catch_clause(&mut self, node: NodeId) {
        let declaration = self
            .n(node)
            .data()
            .as_catch_clause()
            .expect("catch payload")
            .variable_declaration;
        if let Some(declaration) = declaration {
            self.check_strict_mode_eval_or_arguments(node, self.n(declaration).name());
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.checkStrictModeDeleteExpression
    pub fn check_strict_mode_delete_expression(&mut self, node: NodeId) {
        let expression = need(self.n(node).expression());
        if self.n(expression).kind() == K::Identifier {
            self.error_on_node(
                expression,
                d::X_delete_cannot_be_called_on_an_identifier_in_strict_mode,
                Vec::new(),
            );
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.checkStrictModePostfixUnaryExpression
    pub fn check_strict_mode_postfix_unary_expression(&mut self, node: NodeId) {
        self.check_strict_mode_eval_or_arguments(
            node,
            self.n(node)
                .data()
                .as_postfix_unary_expression()
                .expect("postfix payload")
                .operand,
        );
    }
    // port: tsc/internal/binder/binder.go:Binder.checkStrictModePrefixUnaryExpression
    pub fn check_strict_mode_prefix_unary_expression(&mut self, node: NodeId) {
        let data = self
            .n(node)
            .data()
            .as_prefix_unary_expression()
            .expect("prefix payload")
            .clone();
        if matches!(
            data.operator.known(),
            Some(K::PlusPlusToken | K::MinusMinusToken)
        ) {
            self.check_strict_mode_eval_or_arguments(node, data.operand);
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.checkStrictModeWithStatement
    pub fn check_strict_mode_with_statement(&mut self, node: NodeId) {
        self.error_on_first_token(
            node,
            d::X_with_statements_are_not_allowed_in_strict_mode,
            Vec::new(),
        );
    }
    // port: tsc/internal/binder/binder.go:Binder.checkStrictModeLabeledStatement
    pub fn check_strict_mode_labeled_statement(&mut self, node: NodeId) {
        let data = self
            .n(node)
            .data()
            .as_labeled_statement()
            .expect("label payload")
            .clone();
        let statement = self.n(need(data.statement));
        if a::is_declaration_statement(&statement) || statement.kind() == K::VariableStatement {
            self.error_on_first_token(need(data.label), d::A_label_is_not_allowed_here, Vec::new());
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.checkStrictModeEvalOrArguments
    pub fn check_strict_mode_eval_or_arguments(&mut self, context: NodeId, name: Option<NodeId>) {
        if let Some(name) = name {
            if is_eval_or_arguments_identifier(self.view(), name) {
                self.error_on_node(
                    name,
                    self.get_strict_mode_eval_or_arguments_message(context),
                    vec![self.text(name)],
                );
            }
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.getStrictModeEvalOrArgumentsMessage
    pub fn get_strict_mode_eval_or_arguments_message(&self, node: NodeId) -> &'static Message {
        if checked(a::get_containing_class(self.view(), node)).is_some() {
            d::Code_contained_in_a_class_is_evaluated_in_JavaScript_s_strict_mode_which_does_not_allow_this_use_of_0_For_more_information_see_https_Colon_Slash_Slashdeveloper_mozilla_org_Slashen_US_Slashdocs_SlashWeb_SlashJavaScript_SlashReference_SlashStrict_mode
        } else if checked(self.view().source_file(self.file))
            .external_module_indicator
            .is_some()
        {
            d::Invalid_use_of_0_Modules_are_automatically_in_strict_mode
        } else {
            d::Invalid_use_of_0_in_strict_mode
        }
    }
}
// port: tsc/internal/binder/binder.go:isEvalOrArgumentsIdentifier
pub fn is_eval_or_arguments_identifier(view: AstView<'_>, node: NodeId) -> bool {
    checked(view.node(node)).kind() == K::Identifier
        && matches!(
            checked(view.node_text(node)).as_bytes(),
            b"eval" | b"arguments"
        )
}
// port: tsc/internal/binder/binder.go:isUseStrictPrologueDirective
pub fn is_use_strict_prologue_directive(view: AstView<'_>, source: NodeId, node: NodeId) -> bool {
    matches!(
        checked(ts_scanner::get_source_text_of_node_from_source_file(
            view,
            source,
            checked(view.node(node)).expression(),
            false
        ))
        .as_bytes(),
        b"\"use strict\"" | b"'use strict'"
    )
}
// port: tsc/internal/binder/binder.go:FindUseStrictPrologue
pub fn find_use_strict_prologue(
    view: AstView<'_>,
    source: NodeId,
    statements: &[Option<NodeId>],
) -> Option<NodeId> {
    for &statement in statements {
        let statement = need(statement);
        if !checked(a::is_prologue_directive(view, statement)) {
            return None;
        }
        if is_use_strict_prologue_directive(view, source, statement) {
            return Some(statement);
        }
    }
    None
}
