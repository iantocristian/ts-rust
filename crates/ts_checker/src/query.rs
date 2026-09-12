//! Source declarations and the first type queries, over the retained bound graph.
//! Unsupported branches are failures of the port, not language diagnostics.

use crate::{
    object_flags as of, type_flags as tf, CheckerState, Error, LinkStore, TypeAlias, TypeId,
    TypeSystemEntity, TypeSystemPropertyName,
};
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
    /// The union/intersection slice of deferredSymbolLinks.constituents. The containing
    /// type lives in valueSymbolLinks; no owning references back to the checker.
    pub deferred_property_write_types: LinkStore<SymbolId, Option<crate::TypeList>>,
    pub deferred_property_types: LinkStore<SymbolId, Option<crate::TypeList>>,
    pub type_aliases: LinkStore<SymbolId, crate::type_parameters::TypeAliasLinks>,
    pub outer_type_parameters: LinkStore<NodeId, Option<crate::TypeList>>,
    pub source_signatures: LinkStore<NodeId, Option<crate::SignatureId>>,
    pub apparent_types: crate::types::Map<TypeId, TypeId>,
    pub type_parameters_checked: crate::types::Set<SymbolId>,
    pub index_constraints_checked: crate::types::Set<TypeId>,
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
        &mut self,
        node: NodeId,
    ) -> Result<Option<SymbolId>, Error> {
        let Some(symbol) = self.raw_declaration_symbol(node)? else {
            return Ok(None);
        };
        let symbol = self.late_bound_symbol(symbol)?;
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
                true,
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
                if kind == K::Identifier && parent_read.kind() == K::TypeReference {
                    // At this pin, IsPartOfTypeNode selects the identifier
                    // itself; getTypeFromTypeNodeWorker falls back to errorType.
                    // Querying the complete reference is a different operation.
                    return Ok(self.builtins.error_type);
                }
                if parent_read.kind() == K::QualifiedName {
                    return Err(Error::Unsupported("getTypeOfNode: qualified name context"));
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
            return self.declared_interface_type(symbol);
        }
        if flags & sf::TYPE_PARAMETER != 0 {
            return self.get_declared_type_of_type_parameter(symbol);
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
        let type_node = required(read.type_node(), "type alias annotation")?;
        if !self.push_source_resolution(symbol, TypeSystemPropertyName::DeclaredType) {
            return Ok(self.builtins.error_type);
        }
        let result = self.get_type_from_type_node(type_node);
        let complete = self.resolution.pop();
        let mut ty = result?;
        if complete {
            let parameters = self.get_local_type_parameters(symbol)?;
            if !parameters.is_empty() {
                let key = self.object_instantiation_key(&parameters, None, false)?;
                let links = self.query.type_aliases.get_or_default(symbol);
                links.instantiations.insert(key, ty);
                links.parameters = Some(parameters);
            }
            if ty == self.builtins.intrinsic_marker_type
                && self.symbol(symbol)?.name_bytes() == b"BuiltinIteratorReturn"
            {
                let options = self.program()?.host.options();
                ty = if options.strict_option_value(options.strict_builtin_iterator_return) {
                    self.builtins.undefined_type
                } else {
                    self.builtins.any_type
                };
            }
        } else {
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
        }
        Ok(*self
            .query
            .declared_types
            .get_or_default(symbol)
            .get_or_insert(ty))
    }

    // port: tsc/internal/checker/checker.go:Checker.getTypeFromTypeNode
    pub(crate) fn get_type_from_type_node(&mut self, node: NodeId) -> Result<TypeId, Error> {
        stacker::maybe_grow(128 * 1024, 2 * 1024 * 1024, || {
            let ty = self.get_type_from_type_node_worker(node)?;
            self.conditional_flow_type(ty, node)
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
            Some(K::ConditionalType) => self.source_conditional_type(node)?,
            Some(K::InferType) => self.source_infer_type(node)?,
            Some(K::UnionType | K::IntersectionType) => {
                self.get_type_from_union_or_intersection_type_node(node)?
            }
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
            Some(K::TypeLiteral | K::FunctionType | K::ConstructorType) => {
                let symbol = self.get_symbol_of_declaration(node)?;
                let alias = self.alias_for_type_node(node)?;
                let members = symbol
                    .map(|symbol| self.members_of_symbol(symbol))
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
            Some(K::TypeReference | K::ExpressionWithTypeArguments) => {
                self.source_type_reference(node)?
            }
            Some(K::MappedType) => self.source_mapped_type(node)?,
            Some(K::TemplateLiteralType) => self.source_template_type(node)?,
            Some(K::ThisType) => self.type_from_this_node(node)?,
            Some(K::IndexedAccessType) => self.source_indexed_access_type(node)?,
            Some(K::TypeOperator)
                if read
                    .data_source()
                    .as_type_operator_node()
                    .is_some_and(|data| data.operator() == K::KeyOfKeyword) =>
            {
                let operand = read
                    .type_node()
                    .ok_or(Error::MissingLink("keyof operand"))?;
                let ty = self.get_type_from_type_node(operand)?;
                self.get_index_type(ty, 0)?
            }
            Some(K::TypeOperator)
                if read
                    .data_source()
                    .as_type_operator_node()
                    .is_some_and(|data| data.operator() == K::UniqueKeyword) =>
            {
                let argument = read
                    .type_node()
                    .ok_or(Error::MissingLink("unique type operand"))?;
                let mut declaration = read
                    .parent()
                    .ok_or(Error::MissingLink("unique type parent"))?;
                if self.ast(argument)?.node(argument)?.kind() == K::SymbolKeyword {
                    while self.ast(declaration)?.node(declaration)?.kind() == K::ParenthesizedType {
                        declaration = self
                            .ast(declaration)?
                            .node(declaration)?
                            .parent()
                            .ok_or(Error::MissingLink("parenthesized unique type parent"))?;
                    }
                    self.es_symbol_like_type_for_node(declaration)?
                } else {
                    self.builtins.error_type
                }
            }
            Some(K::TypePredicate) => {
                if read
                    .data_source()
                    .as_type_predicate_node()
                    .is_some_and(|data| data.asserts_modifier().is_some())
                {
                    self.builtins.void_type
                } else {
                    self.builtins.boolean_type
                }
            }
            Some(K::ArrayType | K::TupleType) => self.source_array_or_tuple_type(node)?,
            Some(K::OptionalType | K::RestType | K::NamedTupleMember) => {
                self.source_tuple_element_type(node)?
            }
            Some(K::TypeOperator)
                if read
                    .data_source()
                    .as_type_operator_node()
                    .is_some_and(|data| data.operator() == K::ReadonlyKeyword) =>
            {
                self.get_type_from_type_node(
                    read.type_node()
                        .ok_or(Error::MissingLink("readonly type"))?,
                )?
            }
            _ => return Err(Error::Unsupported("getTypeFromTypeNodeWorker: type family")),
        };
        *self.query.type_nodes.get_or_default(node) = Some(ty);
        Ok(ty)
    }

    // port: tsc/internal/checker/checker.go:Checker.getAliasForTypeNode
    pub(crate) fn alias_for_type_node(&mut self, node: NodeId) -> Result<Option<TypeAlias>, Error> {
        let mut parent = self.ast(node)?.node(node)?.parent();
        while let Some(current) = parent {
            let read = self.ast(current)?.node(current)?;
            if read.kind() == K::ParenthesizedType
                || read
                    .data_source()
                    .as_type_operator_node()
                    .is_some_and(|data| data.operator() == K::ReadonlyKeyword)
            {
                parent = read.parent();
                continue;
            }
            if read.kind() != K::TypeAliasDeclaration {
                return Ok(None);
            }
            let Some(symbol) = self.get_symbol_of_declaration(current)? else {
                return Ok(None);
            };
            let type_arguments = self.get_local_type_parameters(symbol)?;
            return Ok(Some(TypeAlias {
                symbol,
                type_arguments,
            }));
        }
        Ok(None)
    }

    // port: tsc/internal/checker/checker.go:Checker.getTypeFromUnionTypeNode
    // port: tsc/internal/checker/checker.go:Checker.getTypeFromIntersectionTypeNode
    fn get_type_from_union_or_intersection_type_node(
        &mut self,
        node: NodeId,
    ) -> Result<TypeId, Error> {
        let alias = self.alias_for_type_node(node)?;
        let view = self.ast(node)?;
        let read = view.node(node)?;
        let is_union = read.kind() == K::UnionType;
        let list = required(
            if is_union {
                read.data_source()
                    .as_union_type_node()
                    .ok_or(ts_arena::Error::InvalidGraph)?
                    .types()
            } else {
                read.data_source()
                    .as_intersection_type_node()
                    .ok_or(ts_arena::Error::InvalidGraph)?
                    .types()
            },
            "compound types",
        )?;
        let nodes = view
            .node_slice(view.list(list)?.nodes())?
            .iter()
            .collect::<Vec<_>>();
        let mut types = Vec::with_capacity(nodes.len());
        for node in nodes {
            types.push(self.get_type_from_type_node(required(node, "compound constituent")?)?);
        }
        let alias = alias
            .map(|alias| self.types.push_alias(alias))
            .transpose()?;
        if is_union {
            return self.get_union_type_ex(&types, crate::UnionReduction::Literal, alias, None);
        }
        let mut flags = 0;
        if types.len() == 2 {
            if let Some(empty) = types
                .iter()
                .position(|&ty| ty == self.builtins.empty_type_literal_type)
            {
                let other = self.types.flags(types[1 - empty])?;
                if other & tf::TEMPLATE_LITERAL != 0 {
                    return Err(Error::Unsupported(
                        "getTypeFromIntersectionTypeNode: pattern literal",
                    ));
                }
                if other & (tf::STRING | tf::NUMBER | tf::BIG_INT) != 0 {
                    flags = crate::intersection::NO_SUPERTYPE_REDUCTION;
                }
            }
        }
        self.get_intersection_type_ex(&types, flags, alias)
    }

    // port: tsc/internal/checker/checker.go:Checker.checkExpressionCached
    // port: tsc/internal/checker/checker.go:Checker.checkExpressionCachedEx
    pub(crate) fn check_expression_cached(&mut self, node: NodeId) -> Result<TypeId, Error> {
        if let Some(Some(ty)) = self.query.type_nodes.try_get(node) {
            return Ok(*ty);
        }
        // P2 implements normal checking only and rejects flow-dependent
        // expressions. When flow checking is added, a cache fill must save,
        // clear and restore flowLoopStack/flowTypeCache as upstream does.
        let ty = self.check_expression(node)?;
        *self.query.type_nodes.get_or_default(node) = Some(ty);
        Ok(ty)
    }

    // port: tsc/internal/checker/checker.go:Checker.checkExpressionWorker
    pub(crate) fn check_expression(&mut self, node: NodeId) -> Result<TypeId, Error> {
        let previous = self.current_node.replace(node);
        self.instantiation.count = 0;
        let result = stacker::maybe_grow(128 * 1024, 2 * 1024 * 1024, || {
            self.check_expression_worker(node)
        });
        self.current_node = previous;
        result
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
            Some(K::Identifier | K::PropertyAccessExpression) => {
                return self.check_ambient_entity_expression(node)
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
        if read.check_flags() & check_flags::DEFERRED_TYPE != 0 {
            return self.get_type_of_symbol_with_deferred_type(symbol);
        }
        if read.check_flags() & check_flags::INSTANTIATED != 0 {
            return self.get_type_of_instantiated_symbol(symbol);
        }
        if read.check_flags() & check_flags::MAPPED != 0 {
            return self.type_of_mapped_symbol(symbol);
        }
        if read.check_flags() & check_flags::REVERSE_MAPPED != 0 {
            return self.type_of_reverse_mapped_symbol(symbol);
        }
        if let Some(ty) = self
            .value_symbol_links
            .try_get(symbol)
            .and_then(|links| links.resolved_type)
        {
            return Ok(ty);
        }
        if read.flags() & (sf::FUNCTION | sf::METHOD) != 0 {
            let ty = self.new_object_type(of::ANONYMOUS, Some(symbol))?;
            self.value_symbol_links.get_or_default(symbol).resolved_type = Some(ty);
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
            Some(K::VariableDeclaration | K::PropertySignature | K::Parameter)
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
            // checkDeclarationInitializer bypasses the cache for quick literal
            // types. Other supported initializers use the normal-mode cache.
            let kind = self.ast(initializer)?.node(initializer)?.kind();
            let ty = if matches!(
                kind.known(),
                Some(
                    K::StringLiteral
                        | K::NoSubstitutionTemplateLiteral
                        | K::NumericLiteral
                        | K::BigIntLiteral
                        | K::TrueKeyword
                        | K::FalseKeyword
                )
            ) {
                self.check_expression(initializer)?
            } else {
                self.check_expression_cached(initializer)?
            };
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
    pub(crate) fn widen_literal_type(&mut self, ty: TypeId) -> Result<TypeId, Error> {
        if self.is_fresh_literal_type(ty)? {
            return Ok(match self.types.flags(ty)? {
                tf::STRING_LITERAL => self.builtins.string_type,
                tf::NUMBER_LITERAL => self.builtins.number_type,
                tf::BIG_INT_LITERAL => self.builtins.bigint_type,
                tf::BOOLEAN_LITERAL => self.builtins.boolean_type,
                _ => return Err(Error::Unsupported("getWidenedLiteralType: enum")),
            });
        }
        if self.types.flags(ty)? & tf::UNION != 0 {
            return self
                .map_type(ty, &mut |checker, part| {
                    checker.widen_literal_type(part).map(Some)
                })?
                .ok_or(Error::MissingLink("widened literal union"));
        }
        Ok(ty)
    }

    // port: tsc/internal/checker/checker.go:Checker.getApparentType
    pub(crate) fn properties_of_primitive_type(
        &mut self,
        ty: TypeId,
    ) -> Result<Vec<SymbolId>, Error> {
        let Some(apparent) = self.apparent_primitive_type(ty)? else {
            return Ok(Vec::new());
        };
        self.resolve_type_members(apparent)?;
        Ok(self
            .types
            .structured(apparent)?
            .properties
            .as_deref()
            .unwrap_or_default()
            .to_vec())
    }

    pub(crate) fn apparent_primitive_type(&mut self, ty: TypeId) -> Result<Option<TypeId>, Error> {
        let flags = self.types.flags(ty)?;
        let name = if flags & tf::STRING_LIKE != 0 {
            Some("String")
        } else if flags & tf::NUMBER_LIKE != 0 {
            Some("Number")
        } else if flags & tf::BOOLEAN_LIKE != 0 {
            Some("Boolean")
        } else if flags & tf::BIG_INT_LIKE != 0 {
            Some("BigInt")
        } else if flags & tf::ES_SYMBOL_LIKE != 0 {
            Some("Symbol")
        } else if flags & tf::NON_PRIMITIVE != 0 {
            return Ok(Some(self.builtins.empty_object_type));
        } else if flags & (tf::ANY | tf::UNKNOWN | tf::VOID | tf::UNDEFINED | tf::NULL | tf::NEVER)
            != 0
        {
            None
        } else {
            return Err(Error::Unsupported("getPropertiesOfType: apparent type"));
        };
        let Some(name) = name else {
            return Ok(None);
        };
        // These global resolver closures are lazy and do not report an absent
        // library type. Preserve that ordering instead of eagerly loading them
        // during program initialization.
        if matches!(name, "BigInt" | "Symbol") && !self.query.global_types.contains_key(name) {
            let ty = self.get_global_type(name, 0, false)?;
            self.query.global_types.insert(name, ty);
        }
        let apparent = self
            .query
            .global_types
            .get(name)
            .copied()
            .ok_or(Error::Unsupported(
                "getApparentType without program initialization",
            ))?;
        Ok(Some(apparent))
    }
}
