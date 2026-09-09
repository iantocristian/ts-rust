//! Shared logical reads for owned factory nodes and contextual compact reads.
use crate::{
    AstView, Node, NodeDataRead, NodeDataSource, NodeId, NodeKind, NodeListId, NodeSlice,
    SyntaxKind as K,
};
use ts_arena::Error;
use ts_core::TextRange;

macro_rules! field {
    ($node:expr, $variant:ident, $accessor:ident, $field:ident) => {
        match $node.data_source().$accessor() {
            Some(data) => data.$field(),
            None => panic!(
                "interface conversion: ast.nodeData is *ast.{}, not *ast.{}",
                $node.data_source().name(),
                stringify!($variant)
            ),
        }
    };
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for crate::Node {}
    impl Sealed for crate::NodeRead<'_> {}
}

/// A borrowed semantic node interface; physical payload storage stays private.
/// Implementations are limited to owned construction nodes and checked reads.
pub trait NodeAccess: sealed::Sealed {
    fn kind(&self) -> NodeKind;
    fn parent(&self) -> Option<NodeId>;
    fn flags(&self) -> u32;
    fn pos(&self) -> i32;
    fn end(&self) -> i32;
    fn range(&self) -> TextRange;
    fn data(&self) -> NodeDataRead<'_>;
    fn data_source(&self) -> NodeDataSource<'_>;
    fn cached_subtree_facts(&self) -> u32;
    fn store_subtree_facts(&self, facts: u32);
    fn existing_runtime_id(&self) -> u64;
    fn runtime_id(&self) -> u64;

    /// Visit children in the pinned node-kind order.
    fn for_each_child(&self, visitor: &mut impl crate::ChildVisitor) -> std::ops::ControlFlow<()>
    where
        Self: Sized,
    {
        crate::runtime_generated::for_each_child_generated(self, visitor)
    }

    /// port: tsc/internal/ast/ast.go:Node.RawText
    fn raw_text(&self) -> &[u8] {
        match self.kind().known() {
            Some(K::TemplateHead) => field!(self, TemplateHead, as_template_head, raw_text),
            Some(K::TemplateMiddle) => field!(self, TemplateMiddle, as_template_middle, raw_text),
            Some(K::TemplateTail) => field!(self, TemplateTail, as_template_tail, raw_text),
            _ => panic!("Unhandled case in Node.RawText: {}", self.kind()),
        }
    }

    /// port: tsc/internal/ast/ast.go:Node.Name
    fn name(&self) -> Option<NodeId> {
        self.data().declaration_name_generated()
    }

    /// port: tsc/internal/ast/ast.go:Node.Modifiers
    fn modifiers(&self) -> Option<NodeListId> {
        match self.data() {
            NodeDataRead::VariableStatement(data) => data.modifiers(),
            NodeDataRead::ParameterDeclaration(data) => data.modifiers(),
            NodeDataRead::MissingDeclaration(data) => data.modifiers(),
            NodeDataRead::FunctionDeclaration(data) => data.modifiers(),
            NodeDataRead::ClassDeclaration(data) => data.modifiers(),
            NodeDataRead::ClassExpression(data) => data.modifiers(),
            NodeDataRead::InterfaceDeclaration(data) => data.modifiers(),
            NodeDataRead::TypeAliasDeclaration(data) => data.modifiers(),
            NodeDataRead::EnumMember(data) => data.modifiers(),
            NodeDataRead::EnumDeclaration(data) => data.modifiers(),
            NodeDataRead::ImportDeclaration(data) => data.modifiers(),
            NodeDataRead::ExportAssignment(data) => data.modifiers(),
            NodeDataRead::NamespaceExportDeclaration(data) => data.modifiers(),
            NodeDataRead::ConstructorDeclaration(data) => data.modifiers(),
            NodeDataRead::GetAccessorDeclaration(data) => data.modifiers(),
            NodeDataRead::SetAccessorDeclaration(data) => data.modifiers(),
            NodeDataRead::IndexSignatureDeclaration(data) => data.modifiers(),
            NodeDataRead::MethodSignatureDeclaration(data) => data.modifiers(),
            NodeDataRead::MethodDeclaration(data) => data.modifiers(),
            NodeDataRead::PropertySignatureDeclaration(data) => data.modifiers(),
            NodeDataRead::PropertyDeclaration(data) => data.modifiers(),
            NodeDataRead::ClassStaticBlockDeclaration(data) => data.modifiers(),
            NodeDataRead::BinaryExpression(data) => data.modifiers(),
            NodeDataRead::ArrowFunction(data) => data.modifiers(),
            NodeDataRead::FunctionExpression(data) => data.modifiers(),
            NodeDataRead::PropertyAssignment(data) => data.modifiers(),
            NodeDataRead::ShorthandPropertyAssignment(data) => data.modifiers(),
            NodeDataRead::FunctionTypeNode(data) => data.modifiers(),
            NodeDataRead::ConstructorTypeNode(data) => data.modifiers(),
            NodeDataRead::ModuleDeclaration(data) => data.modifiers(),
            NodeDataRead::ImportEqualsDeclaration(data) => data.modifiers(),
            NodeDataRead::ExportDeclaration(data) => data.modifiers(),
            NodeDataRead::TypeParameterDeclaration(data) => data.modifiers(),
            _ => None,
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.Body
    fn body(&self) -> Option<NodeId> {
        match self.data() {
            NodeDataRead::FunctionDeclaration(data) => data.body(),
            NodeDataRead::ConstructorDeclaration(data) => data.body(),
            NodeDataRead::GetAccessorDeclaration(data) => data.body(),
            NodeDataRead::SetAccessorDeclaration(data) => data.body(),
            NodeDataRead::MethodDeclaration(data) => data.body(),
            NodeDataRead::ClassStaticBlockDeclaration(data) => data.body(),
            NodeDataRead::ArrowFunction(data) => data.body(),
            NodeDataRead::FunctionExpression(data) => data.body(),
            NodeDataRead::ModuleDeclaration(data) => data.body(),
            _ => None,
        }
    }
    fn function_fields(&self) -> Option<(Option<NodeListId>, Option<NodeListId>, Option<NodeId>)> {
        match self.data() {
            NodeDataRead::FunctionDeclaration(data) => {
                Some((data.type_parameters(), data.parameters(), data.r#type()))
            }
            NodeDataRead::CallSignatureDeclaration(data) => {
                Some((data.type_parameters(), data.parameters(), data.r#type()))
            }
            NodeDataRead::ConstructSignatureDeclaration(data) => {
                Some((data.type_parameters(), data.parameters(), data.r#type()))
            }
            NodeDataRead::ConstructorDeclaration(data) => {
                Some((data.type_parameters(), data.parameters(), data.r#type()))
            }
            NodeDataRead::GetAccessorDeclaration(data) => {
                Some((data.type_parameters(), data.parameters(), data.r#type()))
            }
            NodeDataRead::SetAccessorDeclaration(data) => {
                Some((data.type_parameters(), data.parameters(), data.r#type()))
            }
            NodeDataRead::IndexSignatureDeclaration(data) => {
                Some((data.type_parameters(), data.parameters(), data.r#type()))
            }
            NodeDataRead::MethodSignatureDeclaration(data) => {
                Some((data.type_parameters(), data.parameters(), data.r#type()))
            }
            NodeDataRead::MethodDeclaration(data) => {
                Some((data.type_parameters(), data.parameters(), data.r#type()))
            }
            NodeDataRead::ArrowFunction(data) => {
                Some((data.type_parameters(), data.parameters(), data.r#type()))
            }
            NodeDataRead::FunctionExpression(data) => {
                Some((data.type_parameters(), data.parameters(), data.r#type()))
            }
            NodeDataRead::FunctionTypeNode(data) => {
                Some((data.type_parameters(), data.parameters(), data.r#type()))
            }
            NodeDataRead::ConstructorTypeNode(data) => {
                Some((data.type_parameters(), data.parameters(), data.r#type()))
            }
            NodeDataRead::JSDocSignature(data) => {
                Some((data.type_parameters(), data.parameters(), data.r#type()))
            }
            _ => None,
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.ParameterList
    fn parameter_list(&self) -> Option<NodeListId> {
        self.function_fields()
            .expect("runtime error: invalid memory address or nil pointer dereference")
            .1
    }
    /// port: tsc/internal/ast/ast.go:Node.Expression
    fn expression(&self) -> Option<NodeId> {
        match self.kind().known() {
            Some(K::PropertyAccessExpression) => field!(
                self,
                PropertyAccessExpression,
                as_property_access_expression,
                expression
            ),
            Some(K::ElementAccessExpression) => field!(
                self,
                ElementAccessExpression,
                as_element_access_expression,
                expression
            ),
            Some(K::ParenthesizedExpression) => field!(
                self,
                ParenthesizedExpression,
                as_parenthesized_expression,
                expression
            ),
            Some(K::CallExpression) => field!(self, CallExpression, as_call_expression, expression),
            Some(K::NewExpression) => field!(self, NewExpression, as_new_expression, expression),
            Some(K::ExpressionWithTypeArguments) => {
                field!(
                    self,
                    ExpressionWithTypeArguments,
                    as_expression_with_type_arguments,
                    expression
                )
            }
            Some(K::ComputedPropertyName) => field!(
                self,
                ComputedPropertyName,
                as_computed_property_name,
                expression
            ),
            Some(K::NonNullExpression) => {
                field!(self, NonNullExpression, as_non_null_expression, expression)
            }
            Some(K::TypeAssertionExpression) => {
                field!(self, TypeAssertion, as_type_assertion, expression)
            }
            Some(K::AsExpression) => field!(self, AsExpression, as_as_expression, expression),
            Some(K::SatisfiesExpression) => field!(
                self,
                SatisfiesExpression,
                as_satisfies_expression,
                expression
            ),
            Some(K::TypeOfExpression) => {
                field!(self, TypeOfExpression, as_type_of_expression, expression)
            }
            Some(K::SpreadAssignment) => {
                field!(self, SpreadAssignment, as_spread_assignment, expression)
            }
            Some(K::SpreadElement) => field!(self, SpreadElement, as_spread_element, expression),
            Some(K::TemplateSpan) => field!(self, TemplateSpan, as_template_span, expression),
            Some(K::DeleteExpression) => {
                field!(self, DeleteExpression, as_delete_expression, expression)
            }
            Some(K::VoidExpression) => field!(self, VoidExpression, as_void_expression, expression),
            Some(K::AwaitExpression) => {
                field!(self, AwaitExpression, as_await_expression, expression)
            }
            Some(K::YieldExpression) => {
                field!(self, YieldExpression, as_yield_expression, expression)
            }
            Some(K::PartiallyEmittedExpression) => {
                field!(
                    self,
                    PartiallyEmittedExpression,
                    as_partially_emitted_expression,
                    expression
                )
            }
            Some(K::IfStatement) => field!(self, IfStatement, as_if_statement, expression),
            Some(K::DoStatement) => field!(self, DoStatement, as_do_statement, expression),
            Some(K::WhileStatement) => field!(self, WhileStatement, as_while_statement, expression),
            Some(K::WithStatement) => field!(self, WithStatement, as_with_statement, expression),
            Some(K::ForInStatement | K::ForOfStatement) => {
                field!(
                    self,
                    ForInOrOfStatement,
                    as_for_in_or_of_statement,
                    expression
                )
            }
            Some(K::SwitchStatement) => {
                field!(self, SwitchStatement, as_switch_statement, expression)
            }
            Some(K::CaseClause) => field!(
                self,
                CaseOrDefaultClause,
                as_case_or_default_clause,
                expression
            ),
            Some(K::ExpressionStatement) => field!(
                self,
                ExpressionStatement,
                as_expression_statement,
                expression
            ),
            Some(K::ReturnStatement) => {
                field!(self, ReturnStatement, as_return_statement, expression)
            }
            Some(K::ThrowStatement) => field!(self, ThrowStatement, as_throw_statement, expression),
            Some(K::ExternalModuleReference) => field!(
                self,
                ExternalModuleReference,
                as_external_module_reference,
                expression
            ),
            Some(K::ExportAssignment) => {
                field!(self, ExportAssignment, as_export_assignment, expression)
            }
            Some(K::Decorator) => field!(self, Decorator, as_decorator, expression),
            Some(K::JsxExpression) => field!(self, JsxExpression, as_jsx_expression, expression),
            Some(K::JsxSpreadAttribute) => field!(
                self,
                JsxSpreadAttribute,
                as_jsx_spread_attribute,
                expression
            ),
            _ => panic!("Unhandled case in Node.Expression: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.Type
    fn type_node(&self) -> Option<NodeId> {
        match self.kind().known() {
            Some(K::VariableDeclaration) => {
                field!(self, VariableDeclaration, as_variable_declaration, r#type)
            }
            Some(K::Parameter) => {
                field!(self, ParameterDeclaration, as_parameter_declaration, r#type)
            }
            Some(K::PropertySignature) => field!(
                self,
                PropertySignatureDeclaration,
                as_property_signature_declaration,
                r#type
            ),
            Some(K::PropertyDeclaration) => {
                field!(self, PropertyDeclaration, as_property_declaration, r#type)
            }
            Some(K::PropertyAssignment) => {
                field!(self, PropertyAssignment, as_property_assignment, r#type)
            }
            Some(K::ShorthandPropertyAssignment) => {
                field!(
                    self,
                    ShorthandPropertyAssignment,
                    as_shorthand_property_assignment,
                    r#type
                )
            }
            Some(K::TypePredicate) => {
                field!(self, TypePredicateNode, as_type_predicate_node, r#type)
            }
            Some(K::ParenthesizedType) => field!(
                self,
                ParenthesizedTypeNode,
                as_parenthesized_type_node,
                r#type
            ),
            Some(K::TypeOperator) => field!(self, TypeOperatorNode, as_type_operator_node, r#type),
            Some(K::MappedType) => field!(self, MappedTypeNode, as_mapped_type_node, r#type),
            Some(K::TypeAssertionExpression) => {
                field!(self, TypeAssertion, as_type_assertion, r#type)
            }
            Some(K::AsExpression) => field!(self, AsExpression, as_as_expression, r#type),
            Some(K::SatisfiesExpression) => {
                field!(self, SatisfiesExpression, as_satisfies_expression, r#type)
            }
            Some(K::TypeAliasDeclaration | K::JSTypeAliasDeclaration) => {
                field!(
                    self,
                    TypeAliasDeclaration,
                    as_type_alias_declaration,
                    r#type
                )
            }
            Some(K::NamedTupleMember) => {
                field!(self, NamedTupleMember, as_named_tuple_member, r#type)
            }
            Some(K::OptionalType) => field!(self, OptionalTypeNode, as_optional_type_node, r#type),
            Some(K::RestType) => field!(self, RestTypeNode, as_rest_type_node, r#type),
            Some(K::TemplateLiteralTypeSpan) => field!(
                self,
                TemplateLiteralTypeSpan,
                as_template_literal_type_span,
                r#type
            ),
            Some(K::JSDocTypeExpression) => {
                field!(self, JSDocTypeExpression, as_js_doc_type_expression, r#type)
            }
            Some(K::JSDocParameterTag | K::JSDocPropertyTag) => {
                field!(
                    self,
                    JSDocParameterOrPropertyTag,
                    as_js_doc_parameter_or_property_tag,
                    type_expression
                )
            }
            Some(K::JSDocNullableType) => {
                field!(self, JSDocNullableType, as_js_doc_nullable_type, r#type)
            }
            Some(K::JSDocNonNullableType) => field!(
                self,
                JSDocNonNullableType,
                as_js_doc_non_nullable_type,
                r#type
            ),
            Some(K::JSDocOptionalType) => {
                field!(self, JSDocOptionalType, as_js_doc_optional_type, r#type)
            }
            Some(K::ExportAssignment) => {
                field!(self, ExportAssignment, as_export_assignment, r#type)
            }
            Some(K::BinaryExpression) => {
                field!(self, BinaryExpression, as_binary_expression, r#type)
            }
            _ => self.function_fields().and_then(|fields| fields.2),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.Initializer
    fn initializer(&self) -> Option<NodeId> {
        match self.kind().known() {
            Some(K::VariableDeclaration) => field!(
                self,
                VariableDeclaration,
                as_variable_declaration,
                initializer
            ),
            Some(K::Parameter) => field!(
                self,
                ParameterDeclaration,
                as_parameter_declaration,
                initializer
            ),
            Some(K::BindingElement) => {
                field!(self, BindingElement, as_binding_element, initializer)
            }
            Some(K::PropertyDeclaration) => field!(
                self,
                PropertyDeclaration,
                as_property_declaration,
                initializer
            ),
            Some(K::PropertySignature) => field!(
                self,
                PropertySignatureDeclaration,
                as_property_signature_declaration,
                initializer
            ),
            Some(K::PropertyAssignment) => field!(
                self,
                PropertyAssignment,
                as_property_assignment,
                initializer
            ),
            Some(K::EnumMember) => field!(self, EnumMember, as_enum_member, initializer),
            Some(K::ForStatement) => field!(self, ForStatement, as_for_statement, initializer),
            Some(K::ForInStatement | K::ForOfStatement) => {
                field!(
                    self,
                    ForInOrOfStatement,
                    as_for_in_or_of_statement,
                    initializer
                )
            }
            Some(K::JsxAttribute) => field!(self, JsxAttribute, as_jsx_attribute, initializer),
            _ => panic!("Unhandled case in Node.Initializer"),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.TagName
    fn tag_name(&self) -> Option<NodeId> {
        match self.kind().known() {
            Some(K::JsxOpeningElement) => {
                field!(self, JsxOpeningElement, as_jsx_opening_element, tag_name)
            }
            Some(K::JsxClosingElement) => {
                field!(self, JsxClosingElement, as_jsx_closing_element, tag_name)
            }
            Some(K::JsxSelfClosingElement) => field!(
                self,
                JsxSelfClosingElement,
                as_jsx_self_closing_element,
                tag_name
            ),
            Some(K::JSDocUnknownTag) => {
                field!(self, JSDocUnknownTag, as_js_doc_unknown_tag, tag_name)
            }
            Some(K::JSDocAugmentsTag) => {
                field!(self, JSDocAugmentsTag, as_js_doc_augments_tag, tag_name)
            }
            Some(K::JSDocImplementsTag) => {
                field!(self, JSDocImplementsTag, as_js_doc_implements_tag, tag_name)
            }
            Some(K::JSDocDeprecatedTag) => {
                field!(self, JSDocDeprecatedTag, as_js_doc_deprecated_tag, tag_name)
            }
            Some(K::JSDocPublicTag) => field!(self, JSDocPublicTag, as_js_doc_public_tag, tag_name),
            Some(K::JSDocPrivateTag) => {
                field!(self, JSDocPrivateTag, as_js_doc_private_tag, tag_name)
            }
            Some(K::JSDocProtectedTag) => {
                field!(self, JSDocProtectedTag, as_js_doc_protected_tag, tag_name)
            }
            Some(K::JSDocReadonlyTag) => {
                field!(self, JSDocReadonlyTag, as_js_doc_readonly_tag, tag_name)
            }
            Some(K::JSDocOverrideTag) => {
                field!(self, JSDocOverrideTag, as_js_doc_override_tag, tag_name)
            }
            Some(K::JSDocCallbackTag) => {
                field!(self, JSDocCallbackTag, as_js_doc_callback_tag, tag_name)
            }
            Some(K::JSDocOverloadTag) => {
                field!(self, JSDocOverloadTag, as_js_doc_overload_tag, tag_name)
            }
            Some(K::JSDocParameterTag | K::JSDocPropertyTag) => {
                field!(
                    self,
                    JSDocParameterOrPropertyTag,
                    as_js_doc_parameter_or_property_tag,
                    tag_name
                )
            }
            Some(K::JSDocReturnTag) => field!(self, JSDocReturnTag, as_js_doc_return_tag, tag_name),
            Some(K::JSDocThisTag) => field!(self, JSDocThisTag, as_js_doc_this_tag, tag_name),
            Some(K::JSDocTypeTag) => field!(self, JSDocTypeTag, as_js_doc_type_tag, tag_name),
            Some(K::JSDocTemplateTag) => {
                field!(self, JSDocTemplateTag, as_js_doc_template_tag, tag_name)
            }
            Some(K::JSDocTypedefTag) => {
                field!(self, JSDocTypedefTag, as_js_doc_typedef_tag, tag_name)
            }
            Some(K::JSDocSeeTag) => field!(self, JSDocSeeTag, as_js_doc_see_tag, tag_name),
            Some(K::JSDocSatisfiesTag) => {
                field!(self, JSDocSatisfiesTag, as_js_doc_satisfies_tag, tag_name)
            }
            Some(K::JSDocThrowsTag) => field!(self, JSDocThrowsTag, as_js_doc_throws_tag, tag_name),
            Some(K::JSDocImportTag) => field!(self, JSDocImportTag, as_js_doc_import_tag, tag_name),
            _ => panic!("Unhandled case in Node.TagName: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.PropertyName
    fn property_name(&self) -> Option<NodeId> {
        match self.kind().known() {
            Some(K::ImportSpecifier) => {
                field!(self, ImportSpecifier, as_import_specifier, property_name)
            }
            Some(K::ExportSpecifier) => {
                field!(self, ExportSpecifier, as_export_specifier, property_name)
            }
            Some(K::BindingElement) => {
                field!(self, BindingElement, as_binding_element, property_name)
            }
            _ => None,
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.Label
    fn label(&self) -> Option<NodeId> {
        match self.kind().known() {
            Some(K::LabeledStatement) => {
                field!(self, LabeledStatement, as_labeled_statement, label)
            }
            Some(K::BreakStatement) => field!(self, BreakStatement, as_break_statement, label),
            Some(K::ContinueStatement) => {
                field!(self, ContinueStatement, as_continue_statement, label)
            }
            _ => panic!("Unhandled case in Node.Label: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.Attributes
    fn attributes(&self) -> Option<NodeId> {
        match self.kind().known() {
            Some(K::JsxOpeningElement) => {
                field!(self, JsxOpeningElement, as_jsx_opening_element, attributes)
            }
            Some(K::JsxSelfClosingElement) => field!(
                self,
                JsxSelfClosingElement,
                as_jsx_self_closing_element,
                attributes
            ),
            Some(K::ModuleDeclaration) => {
                field!(self, ModuleDeclaration, as_module_declaration, attributes)
            }
            _ => panic!("Unhandled case in Node.Attributes: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.ModuleSpecifier
    fn module_specifier(&self) -> Option<NodeId> {
        match self.kind().known() {
            Some(K::ImportDeclaration | K::JSImportDeclaration) => {
                field!(
                    self,
                    ImportDeclaration,
                    as_import_declaration,
                    module_specifier
                )
            }
            Some(K::ExportDeclaration) => field!(
                self,
                ExportDeclaration,
                as_export_declaration,
                module_specifier
            ),
            Some(K::JSDocImportTag) => {
                field!(self, JSDocImportTag, as_js_doc_import_tag, module_specifier)
            }
            _ => panic!("Unhandled case in Node.ModuleSpecifier: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.ImportClause
    fn import_clause(&self) -> Option<NodeId> {
        match self.kind().known() {
            Some(K::ImportDeclaration | K::JSImportDeclaration) => {
                field!(
                    self,
                    ImportDeclaration,
                    as_import_declaration,
                    import_clause
                )
            }
            Some(K::JSDocImportTag) => {
                field!(self, JSDocImportTag, as_js_doc_import_tag, import_clause)
            }
            _ => panic!("Unhandled case in Node.ImportClause: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.Statement
    fn statement(&self) -> Option<NodeId> {
        match self.kind().known() {
            Some(K::DoStatement) => field!(self, DoStatement, as_do_statement, statement),
            Some(K::WhileStatement) => field!(self, WhileStatement, as_while_statement, statement),
            Some(K::ForStatement) => field!(self, ForStatement, as_for_statement, statement),
            Some(K::ForInStatement | K::ForOfStatement) => {
                field!(
                    self,
                    ForInOrOfStatement,
                    as_for_in_or_of_statement,
                    statement
                )
            }
            Some(K::WithStatement) => field!(self, WithStatement, as_with_statement, statement),
            Some(K::LabeledStatement) => {
                field!(self, LabeledStatement, as_labeled_statement, statement)
            }
            _ => panic!("Unhandled case in Node.Statement: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.PostfixToken
    fn postfix_token(&self) -> Option<NodeId> {
        match self.kind().known() {
            Some(K::MethodDeclaration) => field!(
                self,
                MethodDeclaration,
                as_method_declaration,
                postfix_token
            ),
            Some(K::ShorthandPropertyAssignment) => {
                field!(
                    self,
                    ShorthandPropertyAssignment,
                    as_shorthand_property_assignment,
                    postfix_token
                )
            }
            Some(K::MethodSignature) => field!(
                self,
                MethodSignatureDeclaration,
                as_method_signature_declaration,
                postfix_token
            ),
            Some(K::PropertySignature) => field!(
                self,
                PropertySignatureDeclaration,
                as_property_signature_declaration,
                postfix_token
            ),
            Some(K::PropertyAssignment) => field!(
                self,
                PropertyAssignment,
                as_property_assignment,
                postfix_token
            ),
            Some(K::PropertyDeclaration) => field!(
                self,
                PropertyDeclaration,
                as_property_declaration,
                postfix_token
            ),
            Some(K::EnumMember) => field!(self, EnumMember, as_enum_member, postfix_token),
            Some(K::GetAccessor) => field!(
                self,
                GetAccessorDeclaration,
                as_get_accessor_declaration,
                postfix_token
            ),
            Some(K::SetAccessor) => field!(
                self,
                SetAccessorDeclaration,
                as_set_accessor_declaration,
                postfix_token
            ),
            _ => None,
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.QuestionDotToken
    fn question_dot_token(&self) -> Option<NodeId> {
        match self.kind().known() {
            Some(K::ElementAccessExpression) => {
                field!(
                    self,
                    ElementAccessExpression,
                    as_element_access_expression,
                    question_dot_token
                )
            }
            Some(K::PropertyAccessExpression) => {
                field!(
                    self,
                    PropertyAccessExpression,
                    as_property_access_expression,
                    question_dot_token
                )
            }
            Some(K::CallExpression) => {
                field!(self, CallExpression, as_call_expression, question_dot_token)
            }
            Some(K::TaggedTemplateExpression) => {
                field!(
                    self,
                    TaggedTemplateExpression,
                    as_tagged_template_expression,
                    question_dot_token
                )
            }
            _ => panic!("Unhandled case in Node.QuestionDotToken: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.TypeExpression
    fn type_expression(&self) -> Option<NodeId> {
        match self.kind().known() {
            Some(K::JSDocParameterTag | K::JSDocPropertyTag) => {
                field!(
                    self,
                    JSDocParameterOrPropertyTag,
                    as_js_doc_parameter_or_property_tag,
                    type_expression
                )
            }
            Some(K::JSDocReturnTag) => {
                field!(self, JSDocReturnTag, as_js_doc_return_tag, type_expression)
            }
            Some(K::JSDocTypeTag) => {
                field!(self, JSDocTypeTag, as_js_doc_type_tag, type_expression)
            }
            Some(K::JSDocTypedefTag) => field!(
                self,
                JSDocTypedefTag,
                as_js_doc_typedef_tag,
                type_expression
            ),
            Some(K::JSDocCallbackTag) => field!(
                self,
                JSDocCallbackTag,
                as_js_doc_callback_tag,
                type_expression
            ),
            Some(K::JSDocSatisfiesTag) => field!(
                self,
                JSDocSatisfiesTag,
                as_js_doc_satisfies_tag,
                type_expression
            ),
            Some(K::JSDocThrowsTag) => {
                field!(self, JSDocThrowsTag, as_js_doc_throws_tag, type_expression)
            }
            _ => panic!("Unhandled case in Node.TypeExpression: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.ClassName
    fn class_name(&self) -> Option<NodeId> {
        match self.kind().known() {
            Some(K::JSDocAugmentsTag) => {
                field!(self, JSDocAugmentsTag, as_js_doc_augments_tag, class_name)
            }
            Some(K::JSDocImplementsTag) => field!(
                self,
                JSDocImplementsTag,
                as_js_doc_implements_tag,
                class_name
            ),
            _ => panic!("Unhandled case in Node.ClassName: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.ArgumentList
    fn argument_list(&self) -> Option<NodeListId> {
        match self.kind().known() {
            Some(K::CallExpression) => field!(self, CallExpression, as_call_expression, arguments),
            Some(K::NewExpression) => field!(self, NewExpression, as_new_expression, arguments),
            _ => panic!("Unhandled case in Node.Arguments: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.TypeArgumentList
    fn type_argument_list(&self) -> Option<NodeListId> {
        match self.kind().known() {
            Some(K::CallExpression) => {
                field!(self, CallExpression, as_call_expression, type_arguments)
            }
            Some(K::NewExpression) => {
                field!(self, NewExpression, as_new_expression, type_arguments)
            }
            Some(K::TaggedTemplateExpression) => {
                field!(
                    self,
                    TaggedTemplateExpression,
                    as_tagged_template_expression,
                    type_arguments
                )
            }
            Some(K::TypeReference) => field!(
                self,
                TypeReferenceNode,
                as_type_reference_node,
                type_arguments
            ),
            Some(K::ExpressionWithTypeArguments) => {
                field!(
                    self,
                    ExpressionWithTypeArguments,
                    as_expression_with_type_arguments,
                    type_arguments
                )
            }
            Some(K::ImportType) => {
                field!(self, ImportTypeNode, as_import_type_node, type_arguments)
            }
            Some(K::TypeQuery) => field!(self, TypeQueryNode, as_type_query_node, type_arguments),
            Some(K::JsxOpeningElement) => field!(
                self,
                JsxOpeningElement,
                as_jsx_opening_element,
                type_arguments
            ),
            Some(K::JsxSelfClosingElement) => field!(
                self,
                JsxSelfClosingElement,
                as_jsx_self_closing_element,
                type_arguments
            ),
            _ => panic!("Unhandled case in Node.TypeArguments"),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.TypeParameterList
    fn type_parameter_list(&self) -> Option<NodeListId> {
        match self.kind().known() {
            Some(K::ClassDeclaration) => field!(
                self,
                ClassDeclaration,
                as_class_declaration,
                type_parameters
            ),
            Some(K::ClassExpression) => {
                field!(self, ClassExpression, as_class_expression, type_parameters)
            }
            Some(K::InterfaceDeclaration) => field!(
                self,
                InterfaceDeclaration,
                as_interface_declaration,
                type_parameters
            ),
            Some(K::TypeAliasDeclaration | K::JSTypeAliasDeclaration) => {
                field!(
                    self,
                    TypeAliasDeclaration,
                    as_type_alias_declaration,
                    type_parameters
                )
            }
            Some(K::JSDocTemplateTag) => field!(
                self,
                JSDocTemplateTag,
                as_js_doc_template_tag,
                type_parameters
            ),
            _ => {
                self.function_fields()
                    .expect("Unhandled case in Node.TypeParameterList")
                    .0
            }
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.MemberList
    fn member_list(&self) -> Option<NodeListId> {
        match self.kind().known() {
            Some(K::ClassDeclaration) => {
                field!(self, ClassDeclaration, as_class_declaration, members)
            }
            Some(K::ClassExpression) => field!(self, ClassExpression, as_class_expression, members),
            Some(K::InterfaceDeclaration) => field!(
                self,
                InterfaceDeclaration,
                as_interface_declaration,
                members
            ),
            Some(K::EnumDeclaration) => field!(self, EnumDeclaration, as_enum_declaration, members),
            Some(K::TypeLiteral) => field!(self, TypeLiteralNode, as_type_literal_node, members),
            Some(K::MappedType) => field!(self, MappedTypeNode, as_mapped_type_node, members),
            _ => panic!("Unhandled case in Node.MemberList: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.StatementList
    fn statement_list(&self) -> Option<NodeListId> {
        match self.kind().known() {
            Some(K::SourceFile) => field!(self, SourceFile, as_source_file, statements),
            Some(K::Block) => field!(self, Block, as_block, statements),
            Some(K::ModuleBlock) => field!(self, ModuleBlock, as_module_block, statements),
            Some(K::CaseClause | K::DefaultClause) => {
                field!(
                    self,
                    CaseOrDefaultClause,
                    as_case_or_default_clause,
                    statements
                )
            }
            _ => panic!("Unhandled case in Node.StatementList: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.CommentList
    fn comment_list(&self) -> Option<NodeListId> {
        match self.kind().known() {
            Some(K::JSDoc) => field!(self, JSDoc, as_js_doc, comment),
            Some(K::JSDocUnknownTag) => {
                field!(self, JSDocUnknownTag, as_js_doc_unknown_tag, comment)
            }
            Some(K::JSDocAugmentsTag) => {
                field!(self, JSDocAugmentsTag, as_js_doc_augments_tag, comment)
            }
            Some(K::JSDocImplementsTag) => {
                field!(self, JSDocImplementsTag, as_js_doc_implements_tag, comment)
            }
            Some(K::JSDocDeprecatedTag) => {
                field!(self, JSDocDeprecatedTag, as_js_doc_deprecated_tag, comment)
            }
            Some(K::JSDocPublicTag) => field!(self, JSDocPublicTag, as_js_doc_public_tag, comment),
            Some(K::JSDocPrivateTag) => {
                field!(self, JSDocPrivateTag, as_js_doc_private_tag, comment)
            }
            Some(K::JSDocProtectedTag) => {
                field!(self, JSDocProtectedTag, as_js_doc_protected_tag, comment)
            }
            Some(K::JSDocReadonlyTag) => {
                field!(self, JSDocReadonlyTag, as_js_doc_readonly_tag, comment)
            }
            Some(K::JSDocOverrideTag) => {
                field!(self, JSDocOverrideTag, as_js_doc_override_tag, comment)
            }
            Some(K::JSDocCallbackTag) => {
                field!(self, JSDocCallbackTag, as_js_doc_callback_tag, comment)
            }
            Some(K::JSDocOverloadTag) => {
                field!(self, JSDocOverloadTag, as_js_doc_overload_tag, comment)
            }
            Some(K::JSDocParameterTag | K::JSDocPropertyTag) => {
                field!(
                    self,
                    JSDocParameterOrPropertyTag,
                    as_js_doc_parameter_or_property_tag,
                    comment
                )
            }
            Some(K::JSDocReturnTag) => field!(self, JSDocReturnTag, as_js_doc_return_tag, comment),
            Some(K::JSDocThisTag) => field!(self, JSDocThisTag, as_js_doc_this_tag, comment),
            Some(K::JSDocTypeTag) => field!(self, JSDocTypeTag, as_js_doc_type_tag, comment),
            Some(K::JSDocTemplateTag) => {
                field!(self, JSDocTemplateTag, as_js_doc_template_tag, comment)
            }
            Some(K::JSDocTypedefTag) => {
                field!(self, JSDocTypedefTag, as_js_doc_typedef_tag, comment)
            }
            Some(K::JSDocSeeTag) => field!(self, JSDocSeeTag, as_js_doc_see_tag, comment),
            Some(K::JSDocSatisfiesTag) => {
                field!(self, JSDocSatisfiesTag, as_js_doc_satisfies_tag, comment)
            }
            Some(K::JSDocThrowsTag) => field!(self, JSDocThrowsTag, as_js_doc_throws_tag, comment),
            Some(K::JSDocImportTag) => field!(self, JSDocImportTag, as_js_doc_import_tag, comment),
            _ => panic!("Unhandled case in Node.CommentList: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.Children
    fn children_list(&self) -> Option<NodeListId> {
        match self.kind().known() {
            Some(K::JsxElement) => field!(self, JsxElement, as_jsx_element, children),
            Some(K::JsxFragment) => field!(self, JsxFragment, as_jsx_fragment, children),
            _ => panic!("Unhandled case in Node.Children: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.PropertyList
    fn property_list(&self) -> Option<NodeListId> {
        match self.kind().known() {
            Some(K::ObjectLiteralExpression) => field!(
                self,
                ObjectLiteralExpression,
                as_object_literal_expression,
                properties
            ),
            Some(K::JsxAttributes) => field!(self, JsxAttributes, as_jsx_attributes, properties),
            _ => panic!("Unhandled case in Node.PropertyList: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.ElementList
    fn element_list(&self) -> Option<NodeListId> {
        match self.kind().known() {
            Some(K::NamedImports) => field!(self, NamedImports, as_named_imports, elements),
            Some(K::NamedExports) => field!(self, NamedExports, as_named_exports, elements),
            Some(K::ObjectBindingPattern | K::ArrayBindingPattern) => {
                field!(self, BindingPattern, as_binding_pattern, elements)
            }
            Some(K::ArrayLiteralExpression) => field!(
                self,
                ArrayLiteralExpression,
                as_array_literal_expression,
                elements
            ),
            Some(K::TupleType) => field!(self, TupleTypeNode, as_tuple_type_node, elements),
            _ => panic!("Unhandled case in Node.ElementList: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.KindString
    fn kind_string(&self) -> String {
        self.kind().to_string()
    }
    /// port: tsc/internal/ast/ast.go:Node.KindValue
    fn kind_value(&self) -> i16 {
        self.kind().raw()
    }
    /// port: tsc/internal/ast/ast.go:Node.PropertyNameOrName
    fn property_name_or_name(&self) -> Option<NodeId> {
        self.property_name().or_else(|| self.name())
    }
    /// port: tsc/internal/ast/ast.go:Node.CanHaveStatements
    fn can_have_statements(&self) -> bool {
        matches!(
            self.kind().known(),
            Some(K::SourceFile | K::Block | K::ModuleBlock | K::CaseClause | K::DefaultClause)
        )
    }
    /// port: tsc/internal/ast/ast.go:Node.IsTypeOnly
    fn is_type_only(&self) -> bool {
        match self.kind().known() {
            Some(K::ImportEqualsDeclaration) => field!(
                self,
                ImportEqualsDeclaration,
                as_import_equals_declaration,
                is_type_only
            ),
            Some(K::ImportSpecifier) => {
                field!(self, ImportSpecifier, as_import_specifier, is_type_only)
            }
            Some(K::ImportClause) => {
                field!(self, ImportClause, as_import_clause, phase_modifier) == K::TypeKeyword
            }
            Some(K::ExportDeclaration) => {
                field!(self, ExportDeclaration, as_export_declaration, is_type_only)
            }
            Some(K::ExportSpecifier) => {
                field!(self, ExportSpecifier, as_export_specifier, is_type_only)
            }
            _ => false,
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.ModifierFlags
    fn modifier_flags(&self, view: AstView<'_>) -> Result<u32, Error> {
        self.modifiers()
            .map_or(Ok(0), |id| Ok(view.list(id)?.modifier_flags()))
    }
    /// port: tsc/internal/ast/ast.go:Node.QuestionToken
    fn question_token(&self, view: AstView<'_>) -> Result<Option<NodeId>, Error> {
        match self.kind().known() {
            Some(K::Parameter) => {
                return Ok(field!(
                    self,
                    ParameterDeclaration,
                    as_parameter_declaration,
                    question_token
                ))
            }
            Some(K::ConditionalExpression) => {
                return Ok(field!(
                    self,
                    ConditionalExpression,
                    as_conditional_expression,
                    question_token
                ));
            }
            Some(K::MappedType) => {
                return Ok(field!(
                    self,
                    MappedTypeNode,
                    as_mapped_type_node,
                    question_token
                ))
            }
            Some(K::NamedTupleMember) => {
                return Ok(field!(
                    self,
                    NamedTupleMember,
                    as_named_tuple_member,
                    question_token
                ))
            }
            _ => {}
        }
        let token = self.postfix_token();
        Ok(match token {
            Some(token) if view.node(token)?.kind() == K::QuestionToken => Some(token),
            _ => None,
        })
    }
    /// port: tsc/internal/ast/ast.go:Node.Parameters
    fn parameters(&self, view: AstView<'_>) -> Result<NodeSlice, Error> {
        Ok(view
            .list(
                self.parameter_list()
                    .expect("runtime error: invalid memory address or nil pointer dereference"),
            )?
            .nodes())
    }
    /// port: tsc/internal/ast/ast.go:Node.Arguments
    fn arguments(&self, view: AstView<'_>) -> Result<NodeSlice, Error> {
        self.argument_list()
            .map_or(Ok(NodeSlice::empty()), |id| Ok(view.list(id)?.nodes()))
    }
    /// port: tsc/internal/ast/ast.go:Node.TypeArguments
    fn type_arguments(&self, view: AstView<'_>) -> Result<NodeSlice, Error> {
        self.type_argument_list()
            .map_or(Ok(NodeSlice::empty()), |id| Ok(view.list(id)?.nodes()))
    }
    /// port: tsc/internal/ast/ast.go:Node.TypeParameters
    fn type_parameters(&self, view: AstView<'_>) -> Result<NodeSlice, Error> {
        self.type_parameter_list()
            .map_or(Ok(NodeSlice::empty()), |id| Ok(view.list(id)?.nodes()))
    }
    /// port: tsc/internal/ast/ast.go:Node.Members
    fn members(&self, view: AstView<'_>) -> Result<NodeSlice, Error> {
        self.member_list()
            .map_or(Ok(NodeSlice::empty()), |id| Ok(view.list(id)?.nodes()))
    }
    /// port: tsc/internal/ast/ast.go:Node.Statements
    fn statements(&self, view: AstView<'_>) -> Result<NodeSlice, Error> {
        self.statement_list()
            .map_or(Ok(NodeSlice::empty()), |id| Ok(view.list(id)?.nodes()))
    }
    /// port: tsc/internal/ast/ast.go:Node.ModifierNodes
    fn modifier_nodes(&self, view: AstView<'_>) -> Result<NodeSlice, Error> {
        self.modifiers()
            .map_or(Ok(NodeSlice::empty()), |id| Ok(view.list(id)?.nodes()))
    }
    /// port: tsc/internal/ast/ast.go:Node.Comments
    fn comments(&self, view: AstView<'_>) -> Result<NodeSlice, Error> {
        self.comment_list()
            .map_or(Ok(NodeSlice::empty()), |id| Ok(view.list(id)?.nodes()))
    }
    /// port: tsc/internal/ast/ast.go:Node.Properties
    fn properties(&self, view: AstView<'_>) -> Result<NodeSlice, Error> {
        self.property_list()
            .map_or(Ok(NodeSlice::empty()), |id| Ok(view.list(id)?.nodes()))
    }
    /// port: tsc/internal/ast/ast.go:Node.Elements
    fn elements(&self, view: AstView<'_>) -> Result<NodeSlice, Error> {
        self.element_list()
            .map_or(Ok(NodeSlice::empty()), |id| Ok(view.list(id)?.nodes()))
    }
}

impl Node {
    pub fn data_source(&self) -> NodeDataSource<'_> {
        NodeDataSource::from_owned(self.data())
    }
}

impl NodeAccess for Node {
    fn kind(&self) -> NodeKind {
        Node::kind(self)
    }
    fn parent(&self) -> Option<NodeId> {
        Node::parent(self)
    }
    fn flags(&self) -> u32 {
        Node::flags(self)
    }
    fn pos(&self) -> i32 {
        Node::pos(self)
    }
    fn end(&self) -> i32 {
        Node::end(self)
    }
    fn range(&self) -> TextRange {
        Node::range(self)
    }
    fn data(&self) -> NodeDataRead<'_> {
        NodeDataRead::from_owned(Node::data(self))
    }
    fn data_source(&self) -> NodeDataSource<'_> {
        NodeDataSource::from_owned(Node::data(self))
    }
    fn cached_subtree_facts(&self) -> u32 {
        Node::cached_subtree_facts(self)
    }
    fn store_subtree_facts(&self, facts: u32) {
        Node::store_subtree_facts(self, facts);
    }
    fn existing_runtime_id(&self) -> u64 {
        crate::runtime_id::owned_existing_runtime_node_id(self)
    }
    fn runtime_id(&self) -> u64 {
        crate::runtime_id::owned_runtime_node_id(self)
    }
}
