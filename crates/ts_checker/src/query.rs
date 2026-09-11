//! Source declarations and the first type queries, over the retained bound graph.
//! Unsupported branches are failures of the port, not language diagnostics.

use crate::{
    object_flags as of, type_flags as tf, CheckerState, Error, LinkStore, TypeAlias, TypeId,
    TypeSystemEntity, TypeSystemPropertyName,
};
use std::sync::Arc;
use ts_arena::{NodeId, SymbolId};
use ts_ast::{check_flags, node_flags as nf, symbol_flags as sf, SymbolFlags, SyntaxKind as K};
use ts_jsstring::JsString;

#[derive(Default)]
pub(crate) struct QueryState {
    pub declared_types: LinkStore<SymbolId, Option<TypeId>>,
    pub type_nodes: LinkStore<NodeId, Option<TypeId>>,
    pub global_types: crate::types::Map<&'static str, TypeId>,
    pub references: LinkStore<SymbolId, SymbolFlags>,
    pub scope_changes: LinkStore<NodeId, ts_core::Tristate>,
}

fn required<T>(value: Option<T>, name: &'static str) -> Result<T, Error> {
    value.ok_or(Error::MissingLink(name))
}

impl CheckerState {
    // port: tsc/internal/ast/utilities.go:IsTypeDeclaration
    pub(crate) fn is_type_declaration(&self, node: NodeId) -> Result<bool, Error> {
        let read = self.ast(node)?.node(node)?;
        Ok(match read.kind().known() {
            Some(
                K::TypeParameter
                | K::ClassDeclaration
                | K::InterfaceDeclaration
                | K::TypeAliasDeclaration
                | K::JSTypeAliasDeclaration
                | K::EnumDeclaration,
            ) => true,
            Some(K::ImportClause) => read.is_type_only(),
            Some(K::ImportSpecifier | K::ExportSpecifier) => {
                let parent = required(read.parent(), "specifier parent")?;
                let grandparent = required(
                    self.ast(parent)?.node(parent)?.parent(),
                    "specifier grandparent",
                )?;
                self.ast(grandparent)?.node(grandparent)?.is_type_only()
            }
            _ => false,
        })
    }

    // port: tsc/internal/checker/checker.go:Checker.getSymbolOfDeclaration
    pub(crate) fn get_symbol_of_declaration(
        &self,
        node: NodeId,
    ) -> Result<Option<SymbolId>, Error> {
        let binding = self.program()?.bound(node)?.node_binding(node)?;
        let Some(symbol) = binding.and_then(|binding| binding.symbol) else {
            return Ok(None);
        };
        let value = self.symbol(symbol)?;
        if value.flags() & sf::CLASS_MEMBER != 0
            && value.name_bytes() == ts_ast::internal_symbol_names::COMPUTED
        {
            return Err(Error::Unsupported("getLateBoundSymbol"));
        }
        Ok(Some(self.get_merged_symbol(symbol)))
    }

    // port: tsc/internal/checker/checker.go:Checker.getSymbolAtLocation
    pub(crate) fn get_symbol_at_location(
        &mut self,
        node: NodeId,
    ) -> Result<Option<SymbolId>, Error> {
        let read = self.ast(node)?.node(node)?;
        if read.flags() & nf::IN_WITH_STATEMENT != 0 {
            return Ok(None);
        }
        if read.kind() == K::SourceFile {
            let source = self.ast(node)?.source_file(node)?;
            return if ts_ast::utilities::is_external_or_common_js_module(&source) {
                self.get_symbol_of_declaration(node)
            } else {
                Ok(None)
            };
        }
        if let Some(parent) = read.parent() {
            let parent_read = self.ast(parent)?.node(parent)?;
            if parent_read.kind() == K::ComputedPropertyName {
                return Err(Error::Unsupported(
                    "getSymbolAtLocation: computed declaration name",
                ));
            }
            if parent_read.name() == Some(node) && ts_ast::is_declaration(&parent_read) {
                if matches!(
                    parent_read.kind().known(),
                    Some(K::ImportSpecifier | K::ExportSpecifier)
                ) && parent_read.property_name() == Some(node)
                {
                    return Err(Error::Unsupported("getImmediateAliasedSymbol"));
                }
                return self.get_symbol_of_declaration(parent);
            }
        }
        if read.kind() == K::Identifier {
            let name = JsString::from_bytes(self.ast(node)?.node_text(node)?.as_bytes());
            let parent = required(read.parent(), "identifier parent")?;
            let parent = self.ast(parent)?.node(parent)?;
            let type_reference = parent.kind() == K::TypeReference;
            if !type_reference
                && !matches!(
                    parent.kind().known(),
                    Some(
                        K::BinaryExpression
                            | K::VariableDeclaration
                            | K::PropertyAssignment
                            | K::ParenthesizedExpression
                            | K::ExpressionStatement
                    )
                )
            {
                return Err(Error::Unsupported(
                    "getSymbolOfNameOrPropertyAccessExpression: identifier context",
                ));
            }
            let result = self.resolve_name(
                Some(node),
                name.as_bytes(),
                if type_reference { sf::TYPE } else { sf::VALUE },
                None,
                false,
            )?;
            if type_reference && result.is_none() {
                return Err(Error::Unsupported("getUnresolvedSymbolForEntityName"));
            }
            return Ok(result);
        }
        match read.kind().known() {
            Some(K::StringLiteral | K::NoSubstitutionTemplateLiteral | K::NumericLiteral) => {
                if let Some(parent) = read.parent() {
                    let parent = self.ast(parent)?.node(parent)?;
                    if matches!(
                        parent.kind().known(),
                        Some(
                            K::CallExpression
                                | K::ImportDeclaration
                                | K::JSImportDeclaration
                                | K::ExportDeclaration
                                | K::ExternalModuleReference
                                | K::ElementAccessExpression
                        )
                    ) {
                        return Err(Error::Unsupported(
                            "getSymbolAtLocation: module/property lookup",
                        ));
                    }
                    if parent.kind() == K::LiteralType {
                        if let Some(grandparent) = parent.parent() {
                            if matches!(
                                self.ast(grandparent)?.node(grandparent)?.kind().known(),
                                Some(K::ImportType | K::IndexedAccessType)
                            ) {
                                return Err(Error::Unsupported(
                                    "getSymbolAtLocation: import/indexed access type",
                                ));
                            }
                        }
                    }
                }
                Ok(None)
            }
            Some(
                K::PrivateIdentifier
                | K::PropertyAccessExpression
                | K::QualifiedName
                | K::ThisKeyword
                | K::ThisType
                | K::SuperKeyword
                | K::ConstructorKeyword
                | K::DefaultKeyword
                | K::FunctionKeyword
                | K::EqualsGreaterThanToken
                | K::ClassKeyword
                | K::ImportType
                | K::ExportKeyword
                | K::ImportKeyword
                | K::NewKeyword
                | K::InstanceOfKeyword
                | K::MetaProperty
                | K::JsxNamespacedName,
            ) => Err(Error::Unsupported("getSymbolAtLocation: node context")),
            _ => Ok(None),
        }
    }

    // port: tsc/internal/checker/checker.go:Checker.GetTypeAtLocation
    // port: tsc/internal/checker/checker.go:Checker.getTypeOfNode
    pub(crate) fn get_type_at_location(&mut self, node: NodeId) -> Result<TypeId, Error> {
        stacker::maybe_grow(128 * 1024, 2 * 1024 * 1024, || {
            let read = self.ast(node)?.node(node)?;
            if read.flags() & nf::IN_WITH_STATEMENT != 0 {
                return Ok(self.builtins.error_type);
            }
            let kind = read.kind();
            if let Some(parent) = read.parent() {
                let parent_read = self.ast(parent)?.node(parent)?;
                if parent_read.name() == Some(node) && ts_ast::is_declaration(&parent_read) {
                    let symbol = required(
                        self.get_symbol_of_declaration(parent)?,
                        "declaration symbol",
                    )?;
                    return if self.is_type_declaration(parent)? {
                        self.get_declared_type_of_symbol(symbol)
                    } else {
                        self.get_type_of_symbol(symbol)
                    };
                }
                if matches!(
                    parent_read.kind().known(),
                    Some(K::TypeReference | K::QualifiedName)
                ) {
                    return self.get_type_from_type_node(parent);
                }
            }
            if ts_ast::utilities::is_type_node(&read) {
                return self.get_type_from_type_node(node);
            }
            match kind.known() {
                Some(
                    K::TypeAliasDeclaration
                    | K::InterfaceDeclaration
                    | K::ClassDeclaration
                    | K::TypeParameter
                    | K::EnumDeclaration,
                ) => {
                    let symbol =
                        required(self.get_symbol_of_declaration(node)?, "declaration symbol")?;
                    self.get_declared_type_of_symbol(symbol)
                }
                Some(
                    K::VariableDeclaration
                    | K::PropertySignature
                    | K::PropertyDeclaration
                    | K::Parameter,
                ) => {
                    let symbol =
                        required(self.get_symbol_of_declaration(node)?, "declaration symbol")?;
                    self.get_type_of_symbol(symbol)
                }
                Some(K::SourceFile) => {
                    if ts_ast::utilities::is_external_or_common_js_module(
                        &self.ast(node)?.source_file(node)?,
                    ) {
                        Err(Error::Unsupported("getTypeOfNode: external module"))
                    } else {
                        Ok(self.builtins.error_type)
                    }
                }
                Some(
                    K::StringLiteral
                    | K::NoSubstitutionTemplateLiteral
                    | K::NumericLiteral
                    | K::BigIntLiteral
                    | K::TrueKeyword
                    | K::FalseKeyword
                    | K::NullKeyword
                    | K::PrefixUnaryExpression
                    | K::ParenthesizedExpression
                    | K::ObjectLiteralExpression,
                ) => {
                    let ty = self.check_expression(node)?;
                    self.get_regular_type_of_literal_type(ty)
                }
                _ => Err(Error::Unsupported("getTypeOfNode: expression/context")),
            }
        })
    }

    // port: tsc/internal/checker/checker.go:Checker.getDeclaredTypeOfSymbol
    pub(crate) fn get_declared_type_of_symbol(
        &mut self,
        symbol: SymbolId,
    ) -> Result<TypeId, Error> {
        let flags = self.symbol(symbol)?.flags();
        if flags & sf::CLASS != 0 {
            return Err(Error::Unsupported(
                "getDeclaredTypeOfClassOrInterface: class",
            ));
        }
        if flags & sf::INTERFACE != 0 {
            return self.get_declared_type_of_interface(symbol);
        }
        if flags & sf::TYPE_PARAMETER != 0 {
            return Err(Error::Unsupported("getDeclaredTypeOfTypeParameter"));
        }
        if flags & sf::TYPE_ALIAS != 0 {
            return self.get_declared_type_of_type_alias(symbol);
        }
        if flags & sf::ENUM != 0 {
            return Err(Error::Unsupported("getDeclaredTypeOfEnum"));
        }
        if flags & sf::ENUM_MEMBER != 0 {
            return Err(Error::Unsupported("getDeclaredTypeOfEnumMember"));
        }
        if flags & sf::ALIAS != 0 {
            return Err(Error::Unsupported("getDeclaredTypeOfAlias"));
        }
        Ok(self.builtins.error_type)
    }

    fn push_source_resolution(
        &mut self,
        symbol: SymbolId,
        property: TypeSystemPropertyName,
    ) -> bool {
        let declared = &self.query.declared_types;
        let values = &self.value_symbol_links;
        self.resolution
            .push(TypeSystemEntity::Symbol(symbol), property, |entry| {
                let TypeSystemEntity::Symbol(symbol) = entry.target else {
                    return false;
                };
                match entry.property_name {
                    TypeSystemPropertyName::DeclaredType => {
                        declared.try_get(symbol).is_some_and(Option::is_some)
                    }
                    TypeSystemPropertyName::Type => values
                        .try_get(symbol)
                        .is_some_and(|links| links.resolved_type.is_some()),
                    _ => false,
                }
            })
    }

    // port: tsc/internal/checker/checker.go:Checker.getDeclaredTypeOfTypeAlias
    fn get_declared_type_of_type_alias(&mut self, symbol: SymbolId) -> Result<TypeId, Error> {
        if let Some(Some(ty)) = self.query.declared_types.try_get(symbol) {
            return Ok(*ty);
        }
        let declarations = self.symbol_declarations(symbol)?.to_vec();
        let mut declaration = None;
        for node in declarations.into_iter().flatten() {
            if self.ast(node)?.node(node)?.kind() == K::TypeAliasDeclaration {
                declaration = Some(node);
                break;
            }
        }
        let declaration = required(declaration, "type alias declaration")?;
        let read = self.ast(declaration)?.node(declaration)?;
        if let Some(list) = read.type_parameter_list() {
            let view = self.ast(declaration)?;
            if !view.node_slice(view.list(list)?.nodes())?.is_empty() {
                return Err(Error::Unsupported("getDeclaredTypeOfTypeAlias: generic"));
            }
        }
        let type_node = required(read.type_node(), "type alias annotation")?;
        if !self.push_source_resolution(symbol, TypeSystemPropertyName::DeclaredType) {
            return Ok(self.builtins.error_type);
        }
        let result = self.get_type_from_type_node(type_node);
        let complete = self.resolution.pop();
        let mut ty = result?;
        if !complete {
            let error_node = self
                .ast(declaration)?
                .node(declaration)?
                .name()
                .unwrap_or(declaration);
            let name = self.symbol_to_string(symbol)?;
            self.error_at(
                Some(error_node),
                ts_diagnostics::Type_alias_0_circularly_references_itself,
                vec![name],
            )?;
            ty = self.builtins.error_type;
        } else if ty == self.builtins.intrinsic_marker_type {
            return Err(Error::Unsupported("getBuiltinIteratorReturnType"));
        }
        Ok(*self
            .query
            .declared_types
            .get_or_default(symbol)
            .get_or_insert(ty))
    }

    // port: tsc/internal/checker/checker.go:Checker.getDeclaredTypeOfClassOrInterface
    fn get_declared_type_of_interface(&mut self, symbol: SymbolId) -> Result<TypeId, Error> {
        if let Some(Some(ty)) = self.query.declared_types.try_get(symbol) {
            return Ok(*ty);
        }
        // Validate the unported branches before publishing a provisional type:
        // later queries must not turn an Unsupported result into a cache hit.
        for declaration in self.symbol_declarations(symbol)?.iter().flatten() {
            let view = self.ast(declaration)?;
            let node = view.node(declaration)?;
            if node.kind() != K::InterfaceDeclaration {
                return Err(Error::Unsupported("interface declaration kind"));
            }
            let mut ancestor = node.parent();
            while let Some(parent) = ancestor {
                let view = self.ast(parent)?;
                let read = view.node(parent)?;
                let parameters = if matches!(
                    read.kind().known(),
                    Some(
                        K::ClassDeclaration
                            | K::ClassExpression
                            | K::InterfaceDeclaration
                            | K::TypeAliasDeclaration
                            | K::JSTypeAliasDeclaration
                            | K::JSDocTemplateTag
                    )
                ) || ts_ast::utilities::is_function_like(Some(&read))
                {
                    read.type_parameter_list()
                } else {
                    None
                };
                if let Some(list) = parameters {
                    if !view.node_slice(view.list(list)?.nodes())?.is_empty() {
                        return Err(Error::Unsupported(
                            "getOuterTypeParametersOfClassOrInterface",
                        ));
                    }
                }
                if read.flags() & nf::HAS_JS_DOC != 0 {
                    return Err(Error::Unsupported(
                        "getOuterTypeParametersOfClassOrInterface: JSDoc template",
                    ));
                }
                ancestor = read.parent();
            }
            if node.flags() & nf::CONTAINS_THIS != 0 {
                return Err(Error::Unsupported("isThislessInterface: this type"));
            }
            if let Some(list) = node.type_parameter_list() {
                if !view.node_slice(view.list(list)?.nodes())?.is_empty() {
                    return Err(Error::Unsupported("interface type parameters"));
                }
            }
            if node
                .data_source()
                .as_interface_declaration()
                .ok_or(ts_arena::Error::InvalidGraph)?
                .heritage_clauses()
                .is_some()
            {
                return Err(Error::Unsupported("isThislessInterface: base types"));
            }
        }
        let ty = self.new_object_type(of::INTERFACE, Some(symbol))?;
        *self.query.declared_types.get_or_default(symbol) = Some(ty);
        Ok(ty)
    }

    // port: tsc/internal/checker/checker.go:Checker.getTypeFromTypeNode
    pub(crate) fn get_type_from_type_node(&mut self, node: NodeId) -> Result<TypeId, Error> {
        stacker::maybe_grow(128 * 1024, 2 * 1024 * 1024, || {
            self.get_type_from_type_node_worker(node)
        })
    }

    // port: tsc/internal/checker/checker.go:Checker.getTypeFromTypeNodeWorker
    fn get_type_from_type_node_worker(&mut self, node: NodeId) -> Result<TypeId, Error> {
        let read = self.ast(node)?.node(node)?;
        if read.kind() == K::TypeReference && read.flags() & nf::JS_DOC != 0 {
            return Err(Error::Unsupported("getIntendedTypeFromJSDocTypeReference"));
        }
        let builtin = match read.kind().known() {
            Some(K::AnyKeyword) => Some(self.builtins.any_type),
            Some(K::UnknownKeyword) => Some(self.builtins.unknown_type),
            Some(K::StringKeyword) => Some(self.builtins.string_type),
            Some(K::NumberKeyword) => Some(self.builtins.number_type),
            Some(K::BigIntKeyword) => Some(self.builtins.bigint_type),
            Some(K::BooleanKeyword) => Some(self.builtins.boolean_type),
            Some(K::SymbolKeyword) => Some(self.builtins.es_symbol_type),
            Some(K::VoidKeyword) => Some(self.builtins.void_type),
            Some(K::UndefinedKeyword) => Some(self.builtins.undefined_type),
            Some(K::NullKeyword) => Some(self.builtins.null_type),
            Some(K::NeverKeyword) => Some(self.builtins.never_type),
            Some(K::ObjectKeyword) => Some(self.builtins.non_primitive_type),
            Some(K::IntrinsicKeyword) => Some(self.builtins.intrinsic_marker_type),
            _ => None,
        };
        if let Some(ty) = builtin {
            return Ok(ty);
        }
        if let Some(Some(ty)) = self.query.type_nodes.try_get(node) {
            return Ok(*ty);
        }
        let ty = match read.kind().known() {
            Some(K::ParenthesizedType) => {
                return self
                    .get_type_from_type_node(required(read.type_node(), "parenthesized type")?)
            }
            Some(K::LiteralType) => {
                let literal = required(
                    read.data_source()
                        .as_literal_type_node()
                        .ok_or(ts_arena::Error::InvalidGraph)?
                        .literal(),
                    "literal type",
                )?;
                if self.ast(literal)?.node(literal)?.kind() == K::NullKeyword {
                    return Ok(self.builtins.null_type);
                }
                let ty = self.check_expression(literal)?;
                self.get_regular_type_of_literal_type(ty)?
            }
            Some(K::TypeLiteral) => {
                let symbol = self.get_symbol_of_declaration(node)?;
                let alias = self.alias_for_type_node(node)?;
                let members = symbol
                    .map(|s| self.symbol(s).map(ts_ast::SymbolRef::members))
                    .transpose()?
                    .flatten();
                let no_members = match members {
                    Some(table) => self.table(table)?.is_empty(),
                    None => true,
                };
                if symbol.is_none() || no_members && alias.is_none() {
                    self.builtins.empty_type_literal_type
                } else {
                    let ty = self.new_object_type(of::ANONYMOUS, symbol)?;
                    if let Some(alias) = alias {
                        self.types.get_mut(ty)?.alias = Some(self.types.push_alias(alias)?);
                    }
                    ty
                }
            }
            Some(K::TypeReference) => {
                if let Some(list) = read.type_argument_list() {
                    if !self
                        .ast(node)?
                        .node_slice(self.ast(node)?.list(list)?.nodes())?
                        .is_empty()
                    {
                        return Err(Error::Unsupported(
                            "getTypeFromTypeReference: type arguments",
                        ));
                    }
                }
                let name = required(
                    read.data_source()
                        .as_type_reference_node()
                        .ok_or(ts_arena::Error::InvalidGraph)?
                        .type_name(),
                    "type reference name",
                )?;
                if self.ast(name)?.node(name)?.kind() != K::Identifier {
                    return Err(Error::Unsupported("resolveEntityName: qualified name"));
                }
                let text = JsString::from_bytes(self.ast(name)?.node_text(name)?.as_bytes());
                match self.resolve_name(
                    Some(name),
                    text.as_bytes(),
                    sf::TYPE,
                    Some(ts_diagnostics::Cannot_find_name_0),
                    true,
                )? {
                    Some(symbol) => self.get_declared_type_of_symbol(symbol)?,
                    None => self.builtins.error_type,
                }
            }
            _ => return Err(Error::Unsupported("getTypeFromTypeNodeWorker: type family")),
        };
        *self.query.type_nodes.get_or_default(node) = Some(ty);
        Ok(ty)
    }

    // port: tsc/internal/checker/checker.go:Checker.getAliasForTypeNode
    fn alias_for_type_node(&self, node: NodeId) -> Result<Option<TypeAlias>, Error> {
        let mut parent = self.ast(node)?.node(node)?.parent();
        while let Some(current) = parent {
            let read = self.ast(current)?.node(current)?;
            if read.kind() == K::ParenthesizedType {
                parent = read.parent();
                continue;
            }
            if read.kind() != K::TypeAliasDeclaration {
                return Ok(None);
            }
            if read.type_parameter_list().is_some() {
                return Err(Error::Unsupported("getTypeArgumentsForAliasSymbol"));
            }
            return Ok(self
                .get_symbol_of_declaration(current)?
                .map(|symbol| TypeAlias {
                    symbol,
                    type_arguments: Arc::from([]),
                }));
        }
        Ok(None)
    }

    // port: tsc/internal/checker/checker.go:Checker.checkExpressionWorker
    pub(crate) fn check_expression(&mut self, node: NodeId) -> Result<TypeId, Error> {
        stacker::maybe_grow(128 * 1024, 2 * 1024 * 1024, || {
            self.check_expression_worker(node)
        })
    }

    fn check_expression_worker(&mut self, node: NodeId) -> Result<TypeId, Error> {
        let read = self.ast(node)?.node(node)?;
        let ty = match read.kind().known() {
            Some(K::StringLiteral | K::NoSubstitutionTemplateLiteral) => {
                let text = self.ast(node)?.node_text(node)?;
                self.get_string_literal_type(JsString::from_bytes(text.as_bytes()))?
            }
            Some(K::NumericLiteral) => {
                self.check_grammar_numeric_literal(node)?;
                let text = self.ast(node)?.node_text(node)?;
                self.get_number_literal_type(ts_jsnum::from_string(text.as_bytes()))?
            }
            Some(K::BigIntLiteral) => {
                self.check_grammar_big_int_literal(node)?;
                let text = self.ast(node)?.node_text(node)?;
                self.get_big_int_literal_type(ts_jsnum::PseudoBigInt::new(
                    &ts_jsnum::parse_pseudo_big_int(text.as_bytes()),
                    false,
                ))?
            }
            Some(K::TrueKeyword) => return Ok(self.builtins.true_type),
            Some(K::FalseKeyword) => return Ok(self.builtins.false_type),
            Some(K::NullKeyword) => return Ok(self.builtins.null_widening_type),
            Some(K::ParenthesizedExpression) => {
                return self
                    .check_expression(required(read.expression(), "parenthesized expression")?)
            }
            Some(K::ObjectLiteralExpression) => return self.check_object_literal(node),
            Some(K::PrefixUnaryExpression) => {
                let data = read
                    .data_source()
                    .as_prefix_unary_expression()
                    .ok_or(ts_arena::Error::InvalidGraph)?;
                let operand = required(data.operand(), "prefix operand")?;
                let operator = data.operator();
                if self.ast(operand)?.node(operand)?.kind() != K::NumericLiteral
                    || !matches!(operator.known(), Some(K::PlusToken | K::MinusToken))
                {
                    return Err(Error::Unsupported("checkPrefixUnaryExpression"));
                }
                // Operand checking owns grammar diagnostics even when the
                // literal fast path determines the unary expression's type.
                self.check_expression(operand)?;
                let number =
                    ts_jsnum::from_string(self.ast(operand)?.node_text(operand)?.as_bytes())
                        .value();
                self.get_number_literal_type(ts_jsnum::Number::new(if operator == K::MinusToken {
                    -number
                } else {
                    number
                }))?
            }
            _ => return Err(Error::Unsupported("checkExpressionWorker")),
        };
        self.get_fresh_type_of_literal_type(ty)
    }

    // port: tsc/internal/checker/grammarchecks.go:Checker.checkGrammarNumericLiteral
    fn check_grammar_numeric_literal(&mut self, node: NodeId) -> Result<(), Error> {
        let view = self.ast(node)?;
        let read = view.node(node)?;
        let literal = read
            .data_source()
            .as_numeric_literal()
            .ok_or(ts_arena::Error::InvalidGraph)?;
        // The scanner's normalized text can itself acquire a decimal point.
        // Fractional spelling is determined from the original source bytes.
        if literal.token_flags() & ts_ast::token_flags::SCIENTIFIC != 0
            || ts_scanner::get_text_of_node(view, node)?
                .as_bytes()
                .contains(&b'.')
            || ts_jsnum::from_string(view.node_text(node)?.as_bytes()).value()
                <= 9_007_199_254_740_991.0
        {
            return Ok(());
        }
        let diagnostic = self.diagnostic_for_node(Some(node), ts_diagnostics::Numeric_literals_with_absolute_values_equal_to_2_53_or_greater_are_too_large_to_be_represented_accurately_as_integers, Vec::new())?;
        self.add_suggestion_diagnostic(diagnostic)?;
        Ok(())
    }

    // port: tsc/internal/checker/grammarchecks.go:Checker.checkGrammarBigIntLiteral
    fn check_grammar_big_int_literal(&mut self, node: NodeId) -> Result<(), Error> {
        let view = self.ast(node)?;
        let read = view.node(node)?;
        let parent = read.parent().map(|parent| view.node(parent)).transpose()?;
        let literal_type = match parent {
            Some(parent) if parent.kind() == K::LiteralType => true,
            Some(parent) if parent.kind() == K::PrefixUnaryExpression => parent
                .parent()
                .map(|grandparent| {
                    view.node(grandparent)
                        .map(|node| node.kind() == K::LiteralType)
                })
                .transpose()?
                .unwrap_or(false),
            _ => false,
        };
        if !literal_type
            && read.flags() & nf::AMBIENT == 0
            && self.program()?.host.options().emit_script_target() < ts_core::ScriptTarget::ES2020
        {
            let source = required(
                ts_ast::utilities::get_source_file_of_node(view, Some(node))?,
                "bigint source",
            )?;
            if view.source_file(source)?.diagnostics().is_empty() {
                self.error_at(Some(node), ts_diagnostics::BigInt_literals_are_not_available_when_targeting_lower_than_ES2020, Vec::new())?;
            }
        }
        Ok(())
    }

    // port: tsc/internal/checker/checker.go:Checker.checkObjectLiteral
    fn check_object_literal(&mut self, node: NodeId) -> Result<TypeId, Error> {
        let view = self.ast(node)?;
        let read = view.node(node)?;
        if read.flags() & (nf::JAVA_SCRIPT_FILE | nf::JSON_FILE) != 0 {
            return Err(Error::Unsupported(
                "checkObjectLiteral: JavaScript/JSON context",
            ));
        }
        let parent = required(read.parent(), "object literal parent")?;
        let parent_read = self.ast(parent)?.node(parent)?;
        let contextual = if parent_read.kind() == K::VariableDeclaration
            && parent_read.initializer() == Some(node)
        {
            parent_read
                .type_node()
                .map(|annotation| self.get_type_from_type_node(annotation))
                .transpose()?
        } else if parent_read.kind() == K::PropertyAssignment {
            self.contextual_property_type(parent)?
        } else {
            return Err(Error::Unsupported(
                "getContextualType: object literal context",
            ));
        };
        let view = self.ast(node)?;
        let properties = match view.node(node)?.property_list() {
            Some(list) => view
                .node_slice(view.list(list)?.nodes())?
                .iter()
                .collect::<Vec<_>>(),
            None => Vec::new(),
        };
        let mut members = ts_ast::SymbolTable::new();
        let mut flags = of::FRESH_LITERAL;
        for property in properties.into_iter().flatten() {
            let read = self.ast(property)?.node(property)?;
            if read.kind() != K::PropertyAssignment {
                return Err(Error::Unsupported(
                    "checkObjectLiteral: spread, shorthand or method",
                ));
            }
            let name = required(read.name(), "property assignment name")?;
            if !ts_ast::utilities::is_property_name_literal(&self.ast(name)?.node(name)?) {
                return Err(Error::Unsupported("checkComputedPropertyName"));
            }
            let original = required(self.get_symbol_of_declaration(property)?, "property symbol")?;
            let text = self.symbol(original)?.name_to_owned();
            if members.contains_key(text.as_bytes()) {
                return Err(Error::Unsupported(
                    "checkGrammarObjectLiteralExpression: duplicate property",
                ));
            }
            let contextual_property = match contextual {
                Some(contextual) => self.property_type(contextual, text.as_bytes())?,
                None => None,
            };
            let ty = self.check_property_assignment(property, contextual_property)?;
            flags |= self.types.get(ty)?.object_flags & of::PROPAGATING_FLAGS;
            let original_read = self.symbol(original)?;
            let symbol_flags = sf::PROPERTY | original_read.flags();
            let declarations = original_read.declarations();
            let parent = original_read.parent();
            let value_declaration = original_read.value_declaration();
            let prop = self.new_symbol_ex(symbol_flags, text.clone(), check_flags::NONE)?;
            let stored = self.symbol_mut(prop)?;
            stored.declarations = declarations;
            stored.parent = parent;
            stored.value_declaration = value_declaration;
            let links = self.value_symbol_links.get_or_default(prop);
            links.resolved_type = Some(ty);
            links.target = Some(original);
            members.insert(text, Some(prop));
        }
        let symbol = self.get_symbol_of_declaration(node)?;
        let table = self.alloc_symbol_table(members);
        let ty = self.new_anonymous_type(symbol, Some(table), &[], &[], &[])?;
        self.types.get_mut(ty)?.object_flags |=
            flags | of::OBJECT_LITERAL | of::CONTAINS_OBJECT_OR_ARRAY_LITERAL;
        Ok(ty)
    }

    pub(crate) fn property_type(
        &mut self,
        ty: TypeId,
        name: &[u8],
    ) -> Result<Option<TypeId>, Error> {
        if self.types.flags(ty)? & tf::ANY != 0 {
            return Ok(Some(self.builtins.any_type));
        }
        self.resolve_type_members(ty)?;
        let table = self.types.structured(ty)?.members;
        let symbol = table
            .map(|table| self.table(table).map(|table| table.get(name).flatten()))
            .transpose()?
            .flatten();
        symbol
            .map(|symbol| self.get_type_of_symbol(symbol))
            .transpose()
    }

    fn contextual_property_type(&mut self, property: NodeId) -> Result<Option<TypeId>, Error> {
        let read = self.ast(property)?.node(property)?;
        let object = required(read.parent(), "property parent")?;
        let parent = required(self.ast(object)?.node(object)?.parent(), "object parent")?;
        let parent_read = self.ast(parent)?.node(parent)?;
        let contextual = if parent_read.kind() == K::VariableDeclaration {
            parent_read
                .type_node()
                .map(|node| self.get_type_from_type_node(node))
                .transpose()?
        } else if parent_read.kind() == K::PropertyAssignment {
            self.contextual_property_type(parent)?
        } else {
            return Err(Error::Unsupported(
                "getContextualTypeForObjectLiteralElement",
            ));
        };
        let Some(contextual) = contextual else {
            return Ok(None);
        };
        let name = required(self.ast(property)?.node(property)?.name(), "property name")?;
        let name = self.ast(name)?.node_text(name)?.into_js_string();
        self.property_type(contextual, name.as_bytes())
    }

    // port: tsc/internal/checker/checker.go:Checker.checkPropertyAssignment
    fn check_property_assignment(
        &mut self,
        node: NodeId,
        contextual: Option<TypeId>,
    ) -> Result<TypeId, Error> {
        let read = self.ast(node)?.node(node)?;
        if read.type_node().is_some() {
            return Err(Error::Unsupported(
                "checkPropertyAssignment: annotation compatibility",
            ));
        }
        let initializer = required(read.initializer(), "property initializer")?;
        let mut ty = self.check_expression(initializer)?;
        if !self.literal_of_context(ty, contextual)? {
            ty = self.widen_literal_type(ty)?;
        }
        self.get_regular_type_of_literal_type(ty)
    }

    // port: tsc/internal/checker/checker.go:Checker.isLiteralOfContextualType
    fn literal_of_context(
        &self,
        candidate: TypeId,
        contextual: Option<TypeId>,
    ) -> Result<bool, Error> {
        let Some(contextual) = contextual else {
            return Ok(false);
        };
        let flags = self.types.flags(contextual)?;
        if flags & tf::UNION != 0 {
            for &ty in self.types.union(contextual)?.types.iter() {
                if self.literal_of_context(candidate, Some(ty))? {
                    return Ok(true);
                }
            }
            return Ok(false);
        }
        if flags & (tf::INTERSECTION | tf::INSTANTIABLE_NON_PRIMITIVE) != 0 {
            return Err(Error::Unsupported(
                "isLiteralOfContextualType: constraint/intersection",
            ));
        }
        let candidate = self.types.flags(candidate)?;
        Ok(
            flags & (tf::STRING_LITERAL | tf::INDEX | tf::TEMPLATE_LITERAL | tf::STRING_MAPPING)
                != 0
                && candidate & tf::STRING_LITERAL != 0
                || flags
                    & candidate
                    & (tf::NUMBER_LITERAL
                        | tf::BIG_INT_LITERAL
                        | tf::BOOLEAN_LITERAL
                        | tf::UNIQUE_ES_SYMBOL)
                    != 0,
        )
    }

    // port: tsc/internal/checker/checker.go:Checker.getTypeOfSymbol
    pub(crate) fn get_type_of_symbol(&mut self, symbol: SymbolId) -> Result<TypeId, Error> {
        let read = self.symbol(symbol)?;
        if read.check_flags()
            & (check_flags::DEFERRED_TYPE
                | check_flags::INSTANTIATED
                | check_flags::MAPPED
                | check_flags::REVERSE_MAPPED)
            != 0
        {
            return Err(Error::Unsupported("getTypeOfSymbol: transformed symbol"));
        }
        if let Some(ty) = self
            .value_symbol_links
            .try_get(symbol)
            .and_then(|links| links.resolved_type)
        {
            return Ok(ty);
        }
        if read.flags() & (sf::VARIABLE | sf::PROPERTY) == 0 {
            return Err(Error::Unsupported("getTypeOfSymbol: value family"));
        }
        let declaration = required(read.value_declaration(), "value declaration")?;
        if !self.push_source_resolution(symbol, TypeSystemPropertyName::Type) {
            return Err(Error::Unsupported("reportCircularityError"));
        }
        let result = (|| {
            if self.ast(declaration)?.node(declaration)?.kind() == K::PropertyAssignment {
                let context = self.contextual_property_type(declaration)?;
                self.check_property_assignment(declaration, context)
            } else {
                self.type_of_variable_like(declaration)
            }
        })();
        let complete = self.resolution.pop();
        let ty = result?;
        if !complete {
            return Err(Error::Unsupported("reportCircularityError"));
        }
        self.value_symbol_links.get_or_default(symbol).resolved_type = Some(ty);
        Ok(ty)
    }

    // port: tsc/internal/checker/checker.go:Checker.getTypeForVariableLikeDeclaration
    fn type_of_variable_like(&mut self, declaration: NodeId) -> Result<TypeId, Error> {
        let read = self.ast(declaration)?.node(declaration)?;
        if !matches!(
            read.kind().known(),
            Some(K::VariableDeclaration | K::PropertySignature)
        ) {
            return Err(Error::Unsupported(
                "getTypeForVariableLikeDeclaration: declaration family",
            ));
        }
        let property = read.kind() == K::PropertySignature;
        if read.flags() & nf::JAVA_SCRIPT_FILE != 0 {
            return Err(Error::Unsupported(
                "getTypeForVariableLikeDeclaration: JavaScript inference",
            ));
        }
        if !property {
            let parent = required(read.parent(), "variable parent")?;
            let parent = self.ast(parent)?.node(parent)?;
            if parent.kind() == K::CatchClause {
                return Err(Error::Unsupported(
                    "getTypeForVariableLikeDeclaration: catch variable",
                ));
            }
            let grandparent = required(parent.parent(), "variable grandparent")?;
            if matches!(
                self.ast(grandparent)?.node(grandparent)?.kind().known(),
                Some(K::ForInStatement | K::ForOfStatement)
            ) {
                return Err(Error::Unsupported(
                    "getTypeForVariableLikeDeclaration: iteration variable",
                ));
            }
        }
        let optional = read.question_token(self.ast(declaration)?)?.is_some();
        let annotation = read.type_node();
        let initializer = read.initializer();
        if let Some(annotation) = annotation {
            let ty = self.get_type_from_type_node(annotation)?;
            if optional && self.options.strict_null_checks {
                let optional = if property {
                    self.builtins.undefined_or_missing_type
                } else {
                    self.builtins.undefined_type
                };
                return self.get_union_type(&[ty, optional]);
            }
            return Ok(ty);
        }
        if let Some(initializer) = initializer {
            let ty = self.check_expression(initializer)?;
            if self.types.flags(ty)? & (tf::NULL | tf::UNDEFINED) != 0
                || self.types.get(ty)?.object_flags & of::REQUIRES_WIDENING != 0
            {
                return Err(Error::Unsupported(
                    "widenTypeForVariableLikeDeclaration: auto/null/object widening",
                ));
            }
            let read = self.ast(declaration)?.node(declaration)?;
            let constant = match read.parent() {
                Some(parent) => self.ast(parent)?.node(parent)?.flags() & nf::CONSTANT != 0,
                None => false,
            };
            if constant {
                return Ok(ty);
            }
            return self.widen_literal_type(ty);
        }
        Err(Error::Unsupported(
            "getTypeForVariableLikeDeclaration: implicit type",
        ))
    }

    // port: tsc/internal/checker/checker.go:Checker.getWidenedLiteralType
    fn widen_literal_type(&mut self, ty: TypeId) -> Result<TypeId, Error> {
        if self.is_fresh_literal_type(ty)? {
            return Ok(match self.types.flags(ty)? {
                tf::STRING_LITERAL => self.builtins.string_type,
                tf::NUMBER_LITERAL => self.builtins.number_type,
                tf::BIG_INT_LITERAL => self.builtins.bigint_type,
                tf::BOOLEAN_LITERAL => self.builtins.boolean_type,
                _ => return Err(Error::Unsupported("getWidenedLiteralType: enum")),
            });
        }
        Ok(ty)
    }

    // port: tsc/internal/checker/checker.go:Checker.resolveAnonymousTypeMembers
    // port: tsc/internal/checker/checker.go:Checker.resolveObjectTypeMembers
    pub(crate) fn resolve_type_members(&mut self, ty: TypeId) -> Result<(), Error> {
        let record = self.types.get(ty)?;
        if record.object_flags & of::MEMBERS_RESOLVED != 0 {
            return Ok(());
        }
        if record.object_flags & (of::ANONYMOUS | of::INTERFACE) == 0 {
            return Err(Error::Unsupported("resolveStructuredTypeMembers"));
        }
        let symbol = self.get_merged_symbol(required(record.symbol, "object type symbol")?);
        let symbol_read = self.symbol(symbol)?;
        if symbol_read.flags() & (sf::TYPE_LITERAL | sf::INTERFACE) == 0 {
            return Err(Error::Unsupported(
                "resolveAnonymousTypeMembers: value object",
            ));
        }
        let members = symbol_read.members();
        if let Some(members) = members {
            for reserved in [
                ts_ast::internal_symbol_names::CALL,
                ts_ast::internal_symbol_names::NEW,
                ts_ast::internal_symbol_names::INDEX,
            ] {
                if self.table(members)?.get(reserved).flatten().is_some() {
                    return Err(Error::Unsupported(
                        "resolveDeclaredMembers: signatures/index infos",
                    ));
                }
            }
        }
        self.set_structured_type_members(ty, members, &[], &[], &[])
    }

    // port: tsc/internal/checker/checker.go:Checker.getApparentType
    pub(crate) fn properties_of_primitive_type(
        &mut self,
        ty: TypeId,
    ) -> Result<Vec<SymbolId>, Error> {
        let flags = self.types.flags(ty)?;
        let name = if flags & tf::STRING_LIKE != 0 {
            Some("String")
        } else if flags & tf::NUMBER_LIKE != 0 {
            Some("Number")
        } else if flags & tf::BOOLEAN_LIKE != 0 {
            Some("Boolean")
        } else if flags & (tf::BIG_INT_LIKE | tf::ES_SYMBOL_LIKE) != 0 {
            return Err(Error::Unsupported("getApparentType: deferred global type"));
        } else if flags & (tf::ANY | tf::UNKNOWN | tf::VOID | tf::UNDEFINED | tf::NULL | tf::NEVER)
            != 0
        {
            None
        } else {
            return Err(Error::Unsupported("getPropertiesOfType: apparent type"));
        };
        let Some(name) = name else {
            return Ok(Vec::new());
        };
        let apparent = self
            .query
            .global_types
            .get(name)
            .copied()
            .ok_or(Error::Unsupported(
                "getApparentType without program initialization",
            ))?;
        self.resolve_type_members(apparent)?;
        Ok(self
            .types
            .structured(apparent)?
            .properties
            .as_deref()
            .unwrap_or_default()
            .to_vec())
    }
}
