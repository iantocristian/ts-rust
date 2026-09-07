//! Source-level Node accessors retain the pinned kind/payload dispatch boundary.
use crate::{AstView, Node, NodeData, NodeId, NodeListId, NodeSlice, SyntaxKind as K};
use ts_arena::Error;
macro_rules! field {
    ($node:expr, $variant:ident, $field:ident) => {
        match $node.data() {
            NodeData::$variant(data) => data.$field,
            _ => panic!(
                "interface conversion: ast.nodeData is *ast.{}, not *ast.{}",
                $node.data().name(),
                stringify!($variant)
            ),
        }
    };
}
impl Node {
    /// port: tsc/internal/ast/ast.go:Node.Name
    pub fn name(&self) -> Option<NodeId> {
        self.data().declaration_name_generated()
    }

    /// port: tsc/internal/ast/ast.go:Node.Modifiers
    pub fn modifiers(&self) -> Option<NodeListId> {
        match self.data() {
            NodeData::VariableStatement(data) => data.modifiers,
            NodeData::ParameterDeclaration(data) => data.modifiers,
            NodeData::MissingDeclaration(data) => data.modifiers,
            NodeData::FunctionDeclaration(data) => data.modifiers,
            NodeData::ClassDeclaration(data) => data.modifiers,
            NodeData::ClassExpression(data) => data.modifiers,
            NodeData::InterfaceDeclaration(data) => data.modifiers,
            NodeData::TypeAliasDeclaration(data) => data.modifiers,
            NodeData::EnumMember(data) => data.modifiers,
            NodeData::EnumDeclaration(data) => data.modifiers,
            NodeData::ImportDeclaration(data) => data.modifiers,
            NodeData::ExportAssignment(data) => data.modifiers,
            NodeData::NamespaceExportDeclaration(data) => data.modifiers,
            NodeData::ConstructorDeclaration(data) => data.modifiers,
            NodeData::GetAccessorDeclaration(data) => data.modifiers,
            NodeData::SetAccessorDeclaration(data) => data.modifiers,
            NodeData::IndexSignatureDeclaration(data) => data.modifiers,
            NodeData::MethodSignatureDeclaration(data) => data.modifiers,
            NodeData::MethodDeclaration(data) => data.modifiers,
            NodeData::PropertySignatureDeclaration(data) => data.modifiers,
            NodeData::PropertyDeclaration(data) => data.modifiers,
            NodeData::ClassStaticBlockDeclaration(data) => data.modifiers,
            NodeData::BinaryExpression(data) => data.modifiers,
            NodeData::ArrowFunction(data) => data.modifiers,
            NodeData::FunctionExpression(data) => data.modifiers,
            NodeData::PropertyAssignment(data) => data.modifiers,
            NodeData::ShorthandPropertyAssignment(data) => data.modifiers,
            NodeData::FunctionTypeNode(data) => data.modifiers,
            NodeData::ConstructorTypeNode(data) => data.modifiers,
            NodeData::ModuleDeclaration(data) => data.modifiers,
            NodeData::ImportEqualsDeclaration(data) => data.modifiers,
            NodeData::ExportDeclaration(data) => data.modifiers,
            NodeData::TypeParameterDeclaration(data) => data.modifiers,
            _ => None,
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.Body
    pub fn body(&self) -> Option<NodeId> {
        match self.data() {
            NodeData::FunctionDeclaration(data) => data.body,
            NodeData::ConstructorDeclaration(data) => data.body,
            NodeData::GetAccessorDeclaration(data) => data.body,
            NodeData::SetAccessorDeclaration(data) => data.body,
            NodeData::MethodDeclaration(data) => data.body,
            NodeData::ClassStaticBlockDeclaration(data) => data.body,
            NodeData::ArrowFunction(data) => data.body,
            NodeData::FunctionExpression(data) => data.body,
            NodeData::ModuleDeclaration(data) => data.body,
            _ => None,
        }
    }
    fn function_fields(&self) -> Option<(Option<NodeListId>, Option<NodeListId>, Option<NodeId>)> {
        match self.data() {
            NodeData::FunctionDeclaration(data) => {
                Some((data.type_parameters, data.parameters, data.r#type))
            }
            NodeData::CallSignatureDeclaration(data) => {
                Some((data.type_parameters, data.parameters, data.r#type))
            }
            NodeData::ConstructSignatureDeclaration(data) => {
                Some((data.type_parameters, data.parameters, data.r#type))
            }
            NodeData::ConstructorDeclaration(data) => {
                Some((data.type_parameters, data.parameters, data.r#type))
            }
            NodeData::GetAccessorDeclaration(data) => {
                Some((data.type_parameters, data.parameters, data.r#type))
            }
            NodeData::SetAccessorDeclaration(data) => {
                Some((data.type_parameters, data.parameters, data.r#type))
            }
            NodeData::IndexSignatureDeclaration(data) => {
                Some((data.type_parameters, data.parameters, data.r#type))
            }
            NodeData::MethodSignatureDeclaration(data) => {
                Some((data.type_parameters, data.parameters, data.r#type))
            }
            NodeData::MethodDeclaration(data) => {
                Some((data.type_parameters, data.parameters, data.r#type))
            }
            NodeData::ArrowFunction(data) => {
                Some((data.type_parameters, data.parameters, data.r#type))
            }
            NodeData::FunctionExpression(data) => {
                Some((data.type_parameters, data.parameters, data.r#type))
            }
            NodeData::FunctionTypeNode(data) => {
                Some((data.type_parameters, data.parameters, data.r#type))
            }
            NodeData::ConstructorTypeNode(data) => {
                Some((data.type_parameters, data.parameters, data.r#type))
            }
            NodeData::JSDocSignature(data) => {
                Some((data.type_parameters, data.parameters, data.r#type))
            }
            _ => None,
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.ParameterList
    pub fn parameter_list(&self) -> Option<NodeListId> {
        self.function_fields()
            .expect("runtime error: invalid memory address or nil pointer dereference")
            .1
    }
    /// port: tsc/internal/ast/ast.go:Node.Expression
    pub fn expression(&self) -> Option<NodeId> {
        match self.kind().known() {
            Some(K::PropertyAccessExpression) => field!(self, PropertyAccessExpression, expression),
            Some(K::ElementAccessExpression) => field!(self, ElementAccessExpression, expression),
            Some(K::ParenthesizedExpression) => field!(self, ParenthesizedExpression, expression),
            Some(K::CallExpression) => field!(self, CallExpression, expression),
            Some(K::NewExpression) => field!(self, NewExpression, expression),
            Some(K::ExpressionWithTypeArguments) => {
                field!(self, ExpressionWithTypeArguments, expression)
            }
            Some(K::ComputedPropertyName) => field!(self, ComputedPropertyName, expression),
            Some(K::NonNullExpression) => field!(self, NonNullExpression, expression),
            Some(K::TypeAssertionExpression) => field!(self, TypeAssertion, expression),
            Some(K::AsExpression) => field!(self, AsExpression, expression),
            Some(K::SatisfiesExpression) => field!(self, SatisfiesExpression, expression),
            Some(K::TypeOfExpression) => field!(self, TypeOfExpression, expression),
            Some(K::SpreadAssignment) => field!(self, SpreadAssignment, expression),
            Some(K::SpreadElement) => field!(self, SpreadElement, expression),
            Some(K::TemplateSpan) => field!(self, TemplateSpan, expression),
            Some(K::DeleteExpression) => field!(self, DeleteExpression, expression),
            Some(K::VoidExpression) => field!(self, VoidExpression, expression),
            Some(K::AwaitExpression) => field!(self, AwaitExpression, expression),
            Some(K::YieldExpression) => field!(self, YieldExpression, expression),
            Some(K::PartiallyEmittedExpression) => {
                field!(self, PartiallyEmittedExpression, expression)
            }
            Some(K::IfStatement) => field!(self, IfStatement, expression),
            Some(K::DoStatement) => field!(self, DoStatement, expression),
            Some(K::WhileStatement) => field!(self, WhileStatement, expression),
            Some(K::WithStatement) => field!(self, WithStatement, expression),
            Some(K::ForInStatement | K::ForOfStatement) => {
                field!(self, ForInOrOfStatement, expression)
            }
            Some(K::SwitchStatement) => field!(self, SwitchStatement, expression),
            Some(K::CaseClause) => field!(self, CaseOrDefaultClause, expression),
            Some(K::ExpressionStatement) => field!(self, ExpressionStatement, expression),
            Some(K::ReturnStatement) => field!(self, ReturnStatement, expression),
            Some(K::ThrowStatement) => field!(self, ThrowStatement, expression),
            Some(K::ExternalModuleReference) => field!(self, ExternalModuleReference, expression),
            Some(K::ExportAssignment) => field!(self, ExportAssignment, expression),
            Some(K::Decorator) => field!(self, Decorator, expression),
            Some(K::JsxExpression) => field!(self, JsxExpression, expression),
            Some(K::JsxSpreadAttribute) => field!(self, JsxSpreadAttribute, expression),
            _ => panic!("Unhandled case in Node.Expression: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.Type
    pub fn type_node(&self) -> Option<NodeId> {
        match self.kind().known() {
            Some(K::VariableDeclaration) => field!(self, VariableDeclaration, r#type),
            Some(K::Parameter) => field!(self, ParameterDeclaration, r#type),
            Some(K::PropertySignature) => field!(self, PropertySignatureDeclaration, r#type),
            Some(K::PropertyDeclaration) => field!(self, PropertyDeclaration, r#type),
            Some(K::PropertyAssignment) => field!(self, PropertyAssignment, r#type),
            Some(K::ShorthandPropertyAssignment) => {
                field!(self, ShorthandPropertyAssignment, r#type)
            }
            Some(K::TypePredicate) => field!(self, TypePredicateNode, r#type),
            Some(K::ParenthesizedType) => field!(self, ParenthesizedTypeNode, r#type),
            Some(K::TypeOperator) => field!(self, TypeOperatorNode, r#type),
            Some(K::MappedType) => field!(self, MappedTypeNode, r#type),
            Some(K::TypeAssertionExpression) => field!(self, TypeAssertion, r#type),
            Some(K::AsExpression) => field!(self, AsExpression, r#type),
            Some(K::SatisfiesExpression) => field!(self, SatisfiesExpression, r#type),
            Some(K::TypeAliasDeclaration | K::JSTypeAliasDeclaration) => {
                field!(self, TypeAliasDeclaration, r#type)
            }
            Some(K::NamedTupleMember) => field!(self, NamedTupleMember, r#type),
            Some(K::OptionalType) => field!(self, OptionalTypeNode, r#type),
            Some(K::RestType) => field!(self, RestTypeNode, r#type),
            Some(K::TemplateLiteralTypeSpan) => field!(self, TemplateLiteralTypeSpan, r#type),
            Some(K::JSDocTypeExpression) => field!(self, JSDocTypeExpression, r#type),
            Some(K::JSDocParameterTag | K::JSDocPropertyTag) => {
                field!(self, JSDocParameterOrPropertyTag, type_expression)
            }
            Some(K::JSDocNullableType) => field!(self, JSDocNullableType, r#type),
            Some(K::JSDocNonNullableType) => field!(self, JSDocNonNullableType, r#type),
            Some(K::JSDocOptionalType) => field!(self, JSDocOptionalType, r#type),
            Some(K::ExportAssignment) => field!(self, ExportAssignment, r#type),
            Some(K::BinaryExpression) => field!(self, BinaryExpression, r#type),
            _ => self.function_fields().and_then(|fields| fields.2),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.Initializer
    pub fn initializer(&self) -> Option<NodeId> {
        match self.kind().known() {
            Some(K::VariableDeclaration) => field!(self, VariableDeclaration, initializer),
            Some(K::Parameter) => field!(self, ParameterDeclaration, initializer),
            Some(K::BindingElement) => field!(self, BindingElement, initializer),
            Some(K::PropertyDeclaration) => field!(self, PropertyDeclaration, initializer),
            Some(K::PropertySignature) => field!(self, PropertySignatureDeclaration, initializer),
            Some(K::PropertyAssignment) => field!(self, PropertyAssignment, initializer),
            Some(K::EnumMember) => field!(self, EnumMember, initializer),
            Some(K::ForStatement) => field!(self, ForStatement, initializer),
            Some(K::ForInStatement | K::ForOfStatement) => {
                field!(self, ForInOrOfStatement, initializer)
            }
            Some(K::JsxAttribute) => field!(self, JsxAttribute, initializer),
            _ => panic!("Unhandled case in Node.Initializer"),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.TagName
    pub fn tag_name(&self) -> Option<NodeId> {
        match self.kind().known() {
            Some(K::JsxOpeningElement) => field!(self, JsxOpeningElement, tag_name),
            Some(K::JsxClosingElement) => field!(self, JsxClosingElement, tag_name),
            Some(K::JsxSelfClosingElement) => field!(self, JsxSelfClosingElement, tag_name),
            Some(K::JSDocUnknownTag) => field!(self, JSDocUnknownTag, tag_name),
            Some(K::JSDocAugmentsTag) => field!(self, JSDocAugmentsTag, tag_name),
            Some(K::JSDocImplementsTag) => field!(self, JSDocImplementsTag, tag_name),
            Some(K::JSDocDeprecatedTag) => field!(self, JSDocDeprecatedTag, tag_name),
            Some(K::JSDocPublicTag) => field!(self, JSDocPublicTag, tag_name),
            Some(K::JSDocPrivateTag) => field!(self, JSDocPrivateTag, tag_name),
            Some(K::JSDocProtectedTag) => field!(self, JSDocProtectedTag, tag_name),
            Some(K::JSDocReadonlyTag) => field!(self, JSDocReadonlyTag, tag_name),
            Some(K::JSDocOverrideTag) => field!(self, JSDocOverrideTag, tag_name),
            Some(K::JSDocCallbackTag) => field!(self, JSDocCallbackTag, tag_name),
            Some(K::JSDocOverloadTag) => field!(self, JSDocOverloadTag, tag_name),
            Some(K::JSDocParameterTag | K::JSDocPropertyTag) => {
                field!(self, JSDocParameterOrPropertyTag, tag_name)
            }
            Some(K::JSDocReturnTag) => field!(self, JSDocReturnTag, tag_name),
            Some(K::JSDocThisTag) => field!(self, JSDocThisTag, tag_name),
            Some(K::JSDocTypeTag) => field!(self, JSDocTypeTag, tag_name),
            Some(K::JSDocTemplateTag) => field!(self, JSDocTemplateTag, tag_name),
            Some(K::JSDocTypedefTag) => field!(self, JSDocTypedefTag, tag_name),
            Some(K::JSDocSeeTag) => field!(self, JSDocSeeTag, tag_name),
            Some(K::JSDocSatisfiesTag) => field!(self, JSDocSatisfiesTag, tag_name),
            Some(K::JSDocThrowsTag) => field!(self, JSDocThrowsTag, tag_name),
            Some(K::JSDocImportTag) => field!(self, JSDocImportTag, tag_name),
            _ => panic!("Unhandled case in Node.TagName: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.PropertyName
    pub fn property_name(&self) -> Option<NodeId> {
        match self.kind().known() {
            Some(K::ImportSpecifier) => field!(self, ImportSpecifier, property_name),
            Some(K::ExportSpecifier) => field!(self, ExportSpecifier, property_name),
            Some(K::BindingElement) => field!(self, BindingElement, property_name),
            _ => None,
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.Label
    pub fn label(&self) -> Option<NodeId> {
        match self.kind().known() {
            Some(K::LabeledStatement) => field!(self, LabeledStatement, label),
            Some(K::BreakStatement) => field!(self, BreakStatement, label),
            Some(K::ContinueStatement) => field!(self, ContinueStatement, label),
            _ => panic!("Unhandled case in Node.Label: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.Attributes
    pub fn attributes(&self) -> Option<NodeId> {
        match self.kind().known() {
            Some(K::JsxOpeningElement) => field!(self, JsxOpeningElement, attributes),
            Some(K::JsxSelfClosingElement) => field!(self, JsxSelfClosingElement, attributes),
            Some(K::ModuleDeclaration) => field!(self, ModuleDeclaration, attributes),
            _ => panic!("Unhandled case in Node.Attributes: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.ModuleSpecifier
    pub fn module_specifier(&self) -> Option<NodeId> {
        match self.kind().known() {
            Some(K::ImportDeclaration | K::JSImportDeclaration) => {
                field!(self, ImportDeclaration, module_specifier)
            }
            Some(K::ExportDeclaration) => field!(self, ExportDeclaration, module_specifier),
            Some(K::JSDocImportTag) => field!(self, JSDocImportTag, module_specifier),
            _ => panic!("Unhandled case in Node.ModuleSpecifier: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.ImportClause
    pub fn import_clause(&self) -> Option<NodeId> {
        match self.kind().known() {
            Some(K::ImportDeclaration | K::JSImportDeclaration) => {
                field!(self, ImportDeclaration, import_clause)
            }
            Some(K::JSDocImportTag) => field!(self, JSDocImportTag, import_clause),
            _ => panic!("Unhandled case in Node.ImportClause: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.Statement
    pub fn statement(&self) -> Option<NodeId> {
        match self.kind().known() {
            Some(K::DoStatement) => field!(self, DoStatement, statement),
            Some(K::WhileStatement) => field!(self, WhileStatement, statement),
            Some(K::ForStatement) => field!(self, ForStatement, statement),
            Some(K::ForInStatement | K::ForOfStatement) => {
                field!(self, ForInOrOfStatement, statement)
            }
            Some(K::WithStatement) => field!(self, WithStatement, statement),
            Some(K::LabeledStatement) => field!(self, LabeledStatement, statement),
            _ => panic!("Unhandled case in Node.Statement: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.PostfixToken
    pub fn postfix_token(&self) -> Option<NodeId> {
        match self.kind().known() {
            Some(K::MethodDeclaration) => field!(self, MethodDeclaration, postfix_token),
            Some(K::ShorthandPropertyAssignment) => {
                field!(self, ShorthandPropertyAssignment, postfix_token)
            }
            Some(K::MethodSignature) => field!(self, MethodSignatureDeclaration, postfix_token),
            Some(K::PropertySignature) => field!(self, PropertySignatureDeclaration, postfix_token),
            Some(K::PropertyAssignment) => field!(self, PropertyAssignment, postfix_token),
            Some(K::PropertyDeclaration) => field!(self, PropertyDeclaration, postfix_token),
            Some(K::EnumMember) => field!(self, EnumMember, postfix_token),
            Some(K::GetAccessor) => field!(self, GetAccessorDeclaration, postfix_token),
            Some(K::SetAccessor) => field!(self, SetAccessorDeclaration, postfix_token),
            _ => None,
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.QuestionDotToken
    pub fn question_dot_token(&self) -> Option<NodeId> {
        match self.kind().known() {
            Some(K::ElementAccessExpression) => {
                field!(self, ElementAccessExpression, question_dot_token)
            }
            Some(K::PropertyAccessExpression) => {
                field!(self, PropertyAccessExpression, question_dot_token)
            }
            Some(K::CallExpression) => field!(self, CallExpression, question_dot_token),
            Some(K::TaggedTemplateExpression) => {
                field!(self, TaggedTemplateExpression, question_dot_token)
            }
            _ => panic!("Unhandled case in Node.QuestionDotToken: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.TypeExpression
    pub fn type_expression(&self) -> Option<NodeId> {
        match self.kind().known() {
            Some(K::JSDocParameterTag | K::JSDocPropertyTag) => {
                field!(self, JSDocParameterOrPropertyTag, type_expression)
            }
            Some(K::JSDocReturnTag) => field!(self, JSDocReturnTag, type_expression),
            Some(K::JSDocTypeTag) => field!(self, JSDocTypeTag, type_expression),
            Some(K::JSDocTypedefTag) => field!(self, JSDocTypedefTag, type_expression),
            Some(K::JSDocCallbackTag) => field!(self, JSDocCallbackTag, type_expression),
            Some(K::JSDocSatisfiesTag) => field!(self, JSDocSatisfiesTag, type_expression),
            Some(K::JSDocThrowsTag) => field!(self, JSDocThrowsTag, type_expression),
            _ => panic!("Unhandled case in Node.TypeExpression: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.ClassName
    pub fn class_name(&self) -> Option<NodeId> {
        match self.kind().known() {
            Some(K::JSDocAugmentsTag) => field!(self, JSDocAugmentsTag, class_name),
            Some(K::JSDocImplementsTag) => field!(self, JSDocImplementsTag, class_name),
            _ => panic!("Unhandled case in Node.ClassName: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.ArgumentList
    pub fn argument_list(&self) -> Option<NodeListId> {
        match self.kind().known() {
            Some(K::CallExpression) => field!(self, CallExpression, arguments),
            Some(K::NewExpression) => field!(self, NewExpression, arguments),
            _ => panic!("Unhandled case in Node.Arguments: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.TypeArgumentList
    pub fn type_argument_list(&self) -> Option<NodeListId> {
        match self.kind().known() {
            Some(K::CallExpression) => field!(self, CallExpression, type_arguments),
            Some(K::NewExpression) => field!(self, NewExpression, type_arguments),
            Some(K::TaggedTemplateExpression) => {
                field!(self, TaggedTemplateExpression, type_arguments)
            }
            Some(K::TypeReference) => field!(self, TypeReferenceNode, type_arguments),
            Some(K::ExpressionWithTypeArguments) => {
                field!(self, ExpressionWithTypeArguments, type_arguments)
            }
            Some(K::ImportType) => field!(self, ImportTypeNode, type_arguments),
            Some(K::TypeQuery) => field!(self, TypeQueryNode, type_arguments),
            Some(K::JsxOpeningElement) => field!(self, JsxOpeningElement, type_arguments),
            Some(K::JsxSelfClosingElement) => field!(self, JsxSelfClosingElement, type_arguments),
            _ => panic!("Unhandled case in Node.TypeArguments"),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.TypeParameterList
    pub fn type_parameter_list(&self) -> Option<NodeListId> {
        match self.kind().known() {
            Some(K::ClassDeclaration) => field!(self, ClassDeclaration, type_parameters),
            Some(K::ClassExpression) => field!(self, ClassExpression, type_parameters),
            Some(K::InterfaceDeclaration) => field!(self, InterfaceDeclaration, type_parameters),
            Some(K::TypeAliasDeclaration | K::JSTypeAliasDeclaration) => {
                field!(self, TypeAliasDeclaration, type_parameters)
            }
            Some(K::JSDocTemplateTag) => field!(self, JSDocTemplateTag, type_parameters),
            _ => {
                self.function_fields()
                    .expect("Unhandled case in Node.TypeParameterList")
                    .0
            }
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.MemberList
    pub fn member_list(&self) -> Option<NodeListId> {
        match self.kind().known() {
            Some(K::ClassDeclaration) => field!(self, ClassDeclaration, members),
            Some(K::ClassExpression) => field!(self, ClassExpression, members),
            Some(K::InterfaceDeclaration) => field!(self, InterfaceDeclaration, members),
            Some(K::EnumDeclaration) => field!(self, EnumDeclaration, members),
            Some(K::TypeLiteral) => field!(self, TypeLiteralNode, members),
            Some(K::MappedType) => field!(self, MappedTypeNode, members),
            _ => panic!("Unhandled case in Node.MemberList: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.StatementList
    pub fn statement_list(&self) -> Option<NodeListId> {
        match self.kind().known() {
            Some(K::SourceFile) => field!(self, SourceFile, statements),
            Some(K::Block) => field!(self, Block, statements),
            Some(K::ModuleBlock) => field!(self, ModuleBlock, statements),
            Some(K::CaseClause | K::DefaultClause) => {
                field!(self, CaseOrDefaultClause, statements)
            }
            _ => panic!("Unhandled case in Node.StatementList: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.CommentList
    pub fn comment_list(&self) -> Option<NodeListId> {
        match self.kind().known() {
            Some(K::JSDoc) => field!(self, JSDoc, comment),
            Some(K::JSDocUnknownTag) => field!(self, JSDocUnknownTag, comment),
            Some(K::JSDocAugmentsTag) => field!(self, JSDocAugmentsTag, comment),
            Some(K::JSDocImplementsTag) => field!(self, JSDocImplementsTag, comment),
            Some(K::JSDocDeprecatedTag) => field!(self, JSDocDeprecatedTag, comment),
            Some(K::JSDocPublicTag) => field!(self, JSDocPublicTag, comment),
            Some(K::JSDocPrivateTag) => field!(self, JSDocPrivateTag, comment),
            Some(K::JSDocProtectedTag) => field!(self, JSDocProtectedTag, comment),
            Some(K::JSDocReadonlyTag) => field!(self, JSDocReadonlyTag, comment),
            Some(K::JSDocOverrideTag) => field!(self, JSDocOverrideTag, comment),
            Some(K::JSDocCallbackTag) => field!(self, JSDocCallbackTag, comment),
            Some(K::JSDocOverloadTag) => field!(self, JSDocOverloadTag, comment),
            Some(K::JSDocParameterTag | K::JSDocPropertyTag) => {
                field!(self, JSDocParameterOrPropertyTag, comment)
            }
            Some(K::JSDocReturnTag) => field!(self, JSDocReturnTag, comment),
            Some(K::JSDocThisTag) => field!(self, JSDocThisTag, comment),
            Some(K::JSDocTypeTag) => field!(self, JSDocTypeTag, comment),
            Some(K::JSDocTemplateTag) => field!(self, JSDocTemplateTag, comment),
            Some(K::JSDocTypedefTag) => field!(self, JSDocTypedefTag, comment),
            Some(K::JSDocSeeTag) => field!(self, JSDocSeeTag, comment),
            Some(K::JSDocSatisfiesTag) => field!(self, JSDocSatisfiesTag, comment),
            Some(K::JSDocThrowsTag) => field!(self, JSDocThrowsTag, comment),
            Some(K::JSDocImportTag) => field!(self, JSDocImportTag, comment),
            _ => panic!("Unhandled case in Node.CommentList: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.Children
    pub fn children_list(&self) -> Option<NodeListId> {
        match self.kind().known() {
            Some(K::JsxElement) => field!(self, JsxElement, children),
            Some(K::JsxFragment) => field!(self, JsxFragment, children),
            _ => panic!("Unhandled case in Node.Children: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.PropertyList
    pub fn property_list(&self) -> Option<NodeListId> {
        match self.kind().known() {
            Some(K::ObjectLiteralExpression) => field!(self, ObjectLiteralExpression, properties),
            Some(K::JsxAttributes) => field!(self, JsxAttributes, properties),
            _ => panic!("Unhandled case in Node.PropertyList: {}", self.kind()),
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.ElementList
    pub fn element_list(&self) -> Option<NodeListId> {
        match self.kind().known() {
            Some(K::NamedImports) => field!(self, NamedImports, elements),
            Some(K::NamedExports) => field!(self, NamedExports, elements),
            Some(K::ObjectBindingPattern | K::ArrayBindingPattern) => {
                field!(self, BindingPattern, elements)
            }
            Some(K::ArrayLiteralExpression) => field!(self, ArrayLiteralExpression, elements),
            Some(K::TupleType) => field!(self, TupleTypeNode, elements),
            _ => panic!("Unhandled case in Node.ElementList: {}", self.kind()),
        }
    }
}

impl Node {
    /// port: tsc/internal/ast/ast.go:Node.KindString
    pub fn kind_string(&self) -> String {
        self.kind().to_string()
    }
    /// port: tsc/internal/ast/ast.go:Node.KindValue
    pub fn kind_value(&self) -> i16 {
        self.kind().raw()
    }
    /// port: tsc/internal/ast/ast.go:Node.PropertyNameOrName
    pub fn property_name_or_name(&self) -> Option<NodeId> {
        self.property_name().or_else(|| self.name())
    }
    /// port: tsc/internal/ast/ast.go:Node.CanHaveStatements
    pub fn can_have_statements(&self) -> bool {
        matches!(
            self.kind().known(),
            Some(K::SourceFile | K::Block | K::ModuleBlock | K::CaseClause | K::DefaultClause)
        )
    }
    /// port: tsc/internal/ast/ast.go:Node.IsTypeOnly
    pub fn is_type_only(&self) -> bool {
        match self.kind().known() {
            Some(K::ImportEqualsDeclaration) => field!(self, ImportEqualsDeclaration, is_type_only),
            Some(K::ImportSpecifier) => field!(self, ImportSpecifier, is_type_only),
            Some(K::ImportClause) => field!(self, ImportClause, phase_modifier) == K::TypeKeyword,
            Some(K::ExportDeclaration) => field!(self, ExportDeclaration, is_type_only),
            Some(K::ExportSpecifier) => field!(self, ExportSpecifier, is_type_only),
            _ => false,
        }
    }
    /// port: tsc/internal/ast/ast.go:Node.ModifierFlags
    pub fn modifier_flags(&self, view: AstView<'_>) -> Result<u32, Error> {
        self.modifiers()
            .map_or(Ok(0), |id| Ok(view.list(id)?.modifier_flags()))
    }
    /// port: tsc/internal/ast/ast.go:Node.QuestionToken
    pub fn question_token(&self, view: AstView<'_>) -> Result<Option<NodeId>, Error> {
        match self.kind().known() {
            Some(K::Parameter) => return Ok(field!(self, ParameterDeclaration, question_token)),
            Some(K::ConditionalExpression) => {
                return Ok(field!(self, ConditionalExpression, question_token));
            }
            Some(K::MappedType) => return Ok(field!(self, MappedTypeNode, question_token)),
            Some(K::NamedTupleMember) => return Ok(field!(self, NamedTupleMember, question_token)),
            _ => {}
        }
        let token = self.postfix_token();
        Ok(match token {
            Some(token) if view.node(token)?.kind() == K::QuestionToken => Some(token),
            _ => None,
        })
    }
    /// port: tsc/internal/ast/ast.go:Node.Parameters
    pub fn parameters(&self, view: AstView<'_>) -> Result<NodeSlice, Error> {
        Ok(view
            .list(
                self.parameter_list()
                    .expect("runtime error: invalid memory address or nil pointer dereference"),
            )?
            .nodes())
    }
    /// port: tsc/internal/ast/ast.go:Node.Arguments
    pub fn arguments(&self, view: AstView<'_>) -> Result<NodeSlice, Error> {
        self.argument_list()
            .map_or(Ok(NodeSlice::empty()), |id| Ok(view.list(id)?.nodes()))
    }
    /// port: tsc/internal/ast/ast.go:Node.TypeArguments
    pub fn type_arguments(&self, view: AstView<'_>) -> Result<NodeSlice, Error> {
        self.type_argument_list()
            .map_or(Ok(NodeSlice::empty()), |id| Ok(view.list(id)?.nodes()))
    }
    /// port: tsc/internal/ast/ast.go:Node.TypeParameters
    pub fn type_parameters(&self, view: AstView<'_>) -> Result<NodeSlice, Error> {
        self.type_parameter_list()
            .map_or(Ok(NodeSlice::empty()), |id| Ok(view.list(id)?.nodes()))
    }
    /// port: tsc/internal/ast/ast.go:Node.Members
    pub fn members(&self, view: AstView<'_>) -> Result<NodeSlice, Error> {
        self.member_list()
            .map_or(Ok(NodeSlice::empty()), |id| Ok(view.list(id)?.nodes()))
    }
    /// port: tsc/internal/ast/ast.go:Node.Statements
    pub fn statements(&self, view: AstView<'_>) -> Result<NodeSlice, Error> {
        self.statement_list()
            .map_or(Ok(NodeSlice::empty()), |id| Ok(view.list(id)?.nodes()))
    }
    /// port: tsc/internal/ast/ast.go:Node.ModifierNodes
    pub fn modifier_nodes(&self, view: AstView<'_>) -> Result<NodeSlice, Error> {
        self.modifiers()
            .map_or(Ok(NodeSlice::empty()), |id| Ok(view.list(id)?.nodes()))
    }
    /// port: tsc/internal/ast/ast.go:Node.Comments
    pub fn comments(&self, view: AstView<'_>) -> Result<NodeSlice, Error> {
        self.comment_list()
            .map_or(Ok(NodeSlice::empty()), |id| Ok(view.list(id)?.nodes()))
    }
    /// port: tsc/internal/ast/ast.go:Node.Properties
    pub fn properties(&self, view: AstView<'_>) -> Result<NodeSlice, Error> {
        self.property_list()
            .map_or(Ok(NodeSlice::empty()), |id| Ok(view.list(id)?.nodes()))
    }
    /// port: tsc/internal/ast/ast.go:Node.Elements
    pub fn elements(&self, view: AstView<'_>) -> Result<NodeSlice, Error> {
        self.element_list()
            .map_or(Ok(NodeSlice::empty()), |id| Ok(view.list(id)?.nodes()))
    }
}

macro_rules! set_field {
    ($node:expr, $variant:ident, $field:ident, $value:expr) => {{
        let actual = $node.data().name();
        match $node.data_mut() {
            NodeData::$variant(data) => data.$field = $value,
            _ => panic!(
                "interface conversion: ast.nodeData is *ast.{actual}, not *ast.{}",
                stringify!($variant)
            ),
        }
    }};
}
impl Node {
    /// port: tsc/internal/ast/ast.go:MutableNode.SetExpression
    pub fn set_expression(&mut self, value: Option<NodeId>) {
        match self.kind().known() {
            Some(K::PropertyAccessExpression) => {
                set_field!(self, PropertyAccessExpression, expression, value);
            }
            Some(K::ElementAccessExpression) => {
                set_field!(self, ElementAccessExpression, expression, value);
            }
            Some(K::ParenthesizedExpression) => {
                set_field!(self, ParenthesizedExpression, expression, value);
            }
            Some(K::CallExpression) => set_field!(self, CallExpression, expression, value),
            Some(K::NewExpression) => set_field!(self, NewExpression, expression, value),
            Some(K::ExpressionWithTypeArguments) => {
                set_field!(self, ExpressionWithTypeArguments, expression, value);
            }
            Some(K::ComputedPropertyName) => {
                set_field!(self, ComputedPropertyName, expression, value);
            }
            Some(K::NonNullExpression) => set_field!(self, NonNullExpression, expression, value),
            Some(K::TypeAssertionExpression) => set_field!(self, TypeAssertion, expression, value),
            Some(K::AsExpression) => set_field!(self, AsExpression, expression, value),
            Some(K::SatisfiesExpression) => {
                set_field!(self, SatisfiesExpression, expression, value);
            }
            Some(K::TypeOfExpression) => set_field!(self, TypeOfExpression, expression, value),
            Some(K::SpreadAssignment) => set_field!(self, SpreadAssignment, expression, value),
            Some(K::SpreadElement) => set_field!(self, SpreadElement, expression, value),
            Some(K::TemplateSpan) => set_field!(self, TemplateSpan, expression, value),
            Some(K::DeleteExpression) => set_field!(self, DeleteExpression, expression, value),
            Some(K::VoidExpression) => set_field!(self, VoidExpression, expression, value),
            Some(K::AwaitExpression) => set_field!(self, AwaitExpression, expression, value),
            Some(K::YieldExpression) => set_field!(self, YieldExpression, expression, value),
            Some(K::PartiallyEmittedExpression) => {
                set_field!(self, PartiallyEmittedExpression, expression, value);
            }
            Some(K::IfStatement) => set_field!(self, IfStatement, expression, value),
            Some(K::DoStatement) => set_field!(self, DoStatement, expression, value),
            Some(K::WhileStatement) => set_field!(self, WhileStatement, expression, value),
            Some(K::WithStatement) => set_field!(self, WithStatement, expression, value),
            Some(K::ForInStatement | K::ForOfStatement) => {
                set_field!(self, ForInOrOfStatement, expression, value);
            }
            Some(K::SwitchStatement) => set_field!(self, SwitchStatement, expression, value),
            Some(K::CaseClause) => set_field!(self, CaseOrDefaultClause, expression, value),
            Some(K::ExpressionStatement) => {
                set_field!(self, ExpressionStatement, expression, value);
            }
            Some(K::ReturnStatement) => set_field!(self, ReturnStatement, expression, value),
            Some(K::ThrowStatement) => set_field!(self, ThrowStatement, expression, value),
            Some(K::ExternalModuleReference) => {
                set_field!(self, ExternalModuleReference, expression, value);
            }
            Some(K::ExportAssignment) => set_field!(self, ExportAssignment, expression, value),
            Some(K::Decorator) => set_field!(self, Decorator, expression, value),
            Some(K::JsxExpression) => set_field!(self, JsxExpression, expression, value),
            Some(K::JsxSpreadAttribute) => set_field!(self, JsxSpreadAttribute, expression, value),
            _ => panic!(
                "Unhandled case in mutableNode.SetExpression: {}",
                self.kind()
            ),
        }
    }
    /// port: tsc/internal/ast/ast.go:MutableNode.SetType
    pub fn set_type_node(&mut self, value: Option<NodeId>) {
        match self.kind().known() {
            Some(K::VariableDeclaration) => set_field!(self, VariableDeclaration, r#type, value),
            Some(K::Parameter) => set_field!(self, ParameterDeclaration, r#type, value),
            Some(K::PropertySignature) => {
                set_field!(self, PropertySignatureDeclaration, r#type, value);
            }
            Some(K::PropertyDeclaration) => set_field!(self, PropertyDeclaration, r#type, value),
            Some(K::PropertyAssignment) => set_field!(self, PropertyAssignment, r#type, value),
            Some(K::ShorthandPropertyAssignment) => {
                set_field!(self, ShorthandPropertyAssignment, r#type, value);
            }
            Some(K::TypePredicate) => set_field!(self, TypePredicateNode, r#type, value),
            Some(K::ParenthesizedType) => set_field!(self, ParenthesizedTypeNode, r#type, value),
            Some(K::TypeOperator) => set_field!(self, TypeOperatorNode, r#type, value),
            Some(K::MappedType) => set_field!(self, MappedTypeNode, r#type, value),
            Some(K::TypeAssertionExpression) => set_field!(self, TypeAssertion, r#type, value),
            Some(K::AsExpression) => set_field!(self, AsExpression, r#type, value),
            Some(K::SatisfiesExpression) => set_field!(self, SatisfiesExpression, r#type, value),
            Some(K::TypeAliasDeclaration | K::JSTypeAliasDeclaration) => {
                set_field!(self, TypeAliasDeclaration, r#type, value);
            }
            Some(K::NamedTupleMember) => set_field!(self, NamedTupleMember, r#type, value),
            Some(K::OptionalType) => set_field!(self, OptionalTypeNode, r#type, value),
            Some(K::RestType) => set_field!(self, RestTypeNode, r#type, value),
            Some(K::TemplateLiteralTypeSpan) => {
                set_field!(self, TemplateLiteralTypeSpan, r#type, value);
            }
            Some(K::JSDocTypeExpression) => set_field!(self, JSDocTypeExpression, r#type, value),
            Some(K::JSDocParameterTag | K::JSDocPropertyTag) => {
                set_field!(self, JSDocParameterOrPropertyTag, type_expression, value);
            }
            Some(K::JSDocNullableType) => set_field!(self, JSDocNullableType, r#type, value),
            Some(K::JSDocNonNullableType) => set_field!(self, JSDocNonNullableType, r#type, value),
            Some(K::JSDocOptionalType) => set_field!(self, JSDocOptionalType, r#type, value),
            Some(K::ExportAssignment) => set_field!(self, ExportAssignment, r#type, value),
            Some(K::BinaryExpression) => set_field!(self, BinaryExpression, r#type, value),
            _ => match self.data_mut() {
                NodeData::FunctionDeclaration(data) => data.r#type = value,
                NodeData::CallSignatureDeclaration(data) => data.r#type = value,
                NodeData::ConstructSignatureDeclaration(data) => data.r#type = value,
                NodeData::ConstructorDeclaration(data) => data.r#type = value,
                NodeData::GetAccessorDeclaration(data) => data.r#type = value,
                NodeData::SetAccessorDeclaration(data) => data.r#type = value,
                NodeData::IndexSignatureDeclaration(data) => data.r#type = value,
                NodeData::MethodSignatureDeclaration(data) => data.r#type = value,
                NodeData::MethodDeclaration(data) => data.r#type = value,
                NodeData::ArrowFunction(data) => data.r#type = value,
                NodeData::FunctionExpression(data) => data.r#type = value,
                NodeData::FunctionTypeNode(data) => data.r#type = value,
                NodeData::ConstructorTypeNode(data) => data.r#type = value,
                NodeData::JSDocSignature(data) => data.r#type = value,
                _ => panic!("Unhandled case in mutableNode.SetType: {}", self.kind()),
            },
        }
    }
    /// port: tsc/internal/ast/ast.go:MutableNode.SetInitializer
    pub fn set_initializer(&mut self, value: Option<NodeId>) {
        match self.kind().known() {
            Some(K::VariableDeclaration) => {
                set_field!(self, VariableDeclaration, initializer, value);
            }
            Some(K::Parameter) => set_field!(self, ParameterDeclaration, initializer, value),
            Some(K::BindingElement) => set_field!(self, BindingElement, initializer, value),
            Some(K::PropertyDeclaration) => {
                set_field!(self, PropertyDeclaration, initializer, value);
            }
            Some(K::PropertySignature) => {
                set_field!(self, PropertySignatureDeclaration, initializer, value);
            }
            Some(K::PropertyAssignment) => set_field!(self, PropertyAssignment, initializer, value),
            Some(K::EnumMember) => set_field!(self, EnumMember, initializer, value),
            Some(K::ForStatement) => set_field!(self, ForStatement, initializer, value),
            Some(K::ForInStatement | K::ForOfStatement) => {
                set_field!(self, ForInOrOfStatement, initializer, value);
            }
            Some(K::JsxAttribute) => set_field!(self, JsxAttribute, initializer, value),
            _ => panic!("Unhandled case in mutableNode.SetInitializer"),
        }
    }
    /// port: tsc/internal/ast/ast.go:MutableNode.SetModifiers
    pub fn set_modifiers(&mut self, value: Option<NodeListId>) {
        match self.data_mut() {
            NodeData::VariableStatement(data) => data.modifiers = value,
            NodeData::ParameterDeclaration(data) => data.modifiers = value,
            NodeData::MissingDeclaration(data) => data.modifiers = value,
            NodeData::FunctionDeclaration(data) => data.modifiers = value,
            NodeData::ClassDeclaration(data) => data.modifiers = value,
            NodeData::ClassExpression(data) => data.modifiers = value,
            NodeData::InterfaceDeclaration(data) => data.modifiers = value,
            NodeData::TypeAliasDeclaration(data) => data.modifiers = value,
            NodeData::EnumMember(data) => data.modifiers = value,
            NodeData::EnumDeclaration(data) => data.modifiers = value,
            NodeData::ImportDeclaration(data) => data.modifiers = value,
            NodeData::ExportAssignment(data) => data.modifiers = value,
            NodeData::NamespaceExportDeclaration(data) => data.modifiers = value,
            NodeData::ConstructorDeclaration(data) => data.modifiers = value,
            NodeData::GetAccessorDeclaration(data) => data.modifiers = value,
            NodeData::SetAccessorDeclaration(data) => data.modifiers = value,
            NodeData::IndexSignatureDeclaration(data) => data.modifiers = value,
            NodeData::MethodSignatureDeclaration(data) => data.modifiers = value,
            NodeData::MethodDeclaration(data) => data.modifiers = value,
            NodeData::PropertySignatureDeclaration(data) => data.modifiers = value,
            NodeData::PropertyDeclaration(data) => data.modifiers = value,
            NodeData::ClassStaticBlockDeclaration(data) => data.modifiers = value,
            NodeData::BinaryExpression(data) => data.modifiers = value,
            NodeData::ArrowFunction(data) => data.modifiers = value,
            NodeData::FunctionExpression(data) => data.modifiers = value,
            NodeData::PropertyAssignment(data) => data.modifiers = value,
            NodeData::ShorthandPropertyAssignment(data) => data.modifiers = value,
            NodeData::FunctionTypeNode(data) => data.modifiers = value,
            NodeData::ConstructorTypeNode(data) => data.modifiers = value,
            NodeData::ModuleDeclaration(data) => data.modifiers = value,
            NodeData::ImportEqualsDeclaration(data) => data.modifiers = value,
            NodeData::ExportDeclaration(data) => data.modifiers = value,
            NodeData::TypeParameterDeclaration(data) => data.modifiers = value,
            _ => {}
        }
    }
}
