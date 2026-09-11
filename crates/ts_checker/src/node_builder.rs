//! The executable type-display slice of checker/nodebuilderimpl.go. Each builder
//! owns its synthetic syntax and emit flags until printing finishes.

use crate::{object_flags as of, type_flags as tf, CheckerState, Error, LiteralValue, TypeId};
use ts_arena::SymbolId;
use ts_ast::{
    check_flags, modifier_flags, symbol_flags as sf, token_flags, AstBuilder, Factory,
    FactoryMethods, JsString, NodeId, NodeListId, SyntaxKind as K,
};
use ts_core::{LanguageVariant, TextRange};
use ts_jsstring::SourceText;
use ts_nodebuilder::flags as nf;
use ts_printer::{emit_flags, EmitContext};

pub(crate) struct NodeBuilder<'a> {
    pub(crate) checker: &'a mut CheckerState,
    pub(crate) ast: AstBuilder,
    pub(crate) emit: EmitContext,
    pub(crate) flags: ts_nodebuilder::Flags,
    pub(crate) approximate_length: usize,
    truncating: bool,
    visited: Vec<TypeId>,
}

impl<'a> NodeBuilder<'a> {
    pub(crate) fn new(checker: &'a mut CheckerState, flags: ts_nodebuilder::Flags) -> Self {
        let ast = AstBuilder::with_hooks(
            SourceText::from_bytes(b"".as_slice()),
            &checker.counters,
            EmitContext::factory_hooks(),
        );
        Self {
            checker,
            ast,
            emit: EmitContext::new(),
            flags,
            approximate_length: 0,
            truncating: false,
            visited: Vec::new(),
        }
    }

    // port: tsc/internal/checker/nodebuilderimpl.go:NodeBuilderImpl.getNameOfSymbolAsWritten
    fn symbol_name(&self, symbol: SymbolId) -> Result<JsString, Error> {
        let read = self.checker.symbol(symbol)?;
        if read.is_external_module() {
            return Err(Error::Unsupported("getSpecifierForModuleSymbol"));
        }
        let declarations = self.checker.symbol_declarations(symbol)?;
        if read.name_bytes() == ts_ast::internal_symbol_names::DEFAULT && declarations.is_empty() {
            return Ok(JsString::from_bytes(b"default".as_slice()));
        }
        for declaration in declarations.iter().flatten() {
            let view = self.checker.ast(declaration)?;
            if let Some(name) = view.node(declaration)?.name() {
                if view.node(name)?.kind() == K::ComputedPropertyName {
                    return Err(Error::Unsupported(
                        "getNameOfSymbolAsWritten: computed name",
                    ));
                }
                return Ok(ts_scanner::declaration_name_to_string(view, Some(name))?);
            }
        }
        if let Some(declaration) = declarations.first().flatten() {
            let view = self.checker.ast(declaration)?;
            let node = view.node(declaration)?;
            if let Some(parent) = node.parent() {
                if view.node(parent)?.kind() == K::VariableDeclaration {
                    return Ok(ts_scanner::declaration_name_to_string(
                        view,
                        view.node(parent)?.name(),
                    )?);
                }
            }
            match node.kind().known() {
                Some(K::ClassExpression) => {
                    return Ok(JsString::from_bytes(b"(Anonymous class)".as_slice()))
                }
                Some(K::FunctionExpression | K::ArrowFunction) => {
                    return Ok(JsString::from_bytes(b"(Anonymous function)".as_slice()))
                }
                _ => {}
            }
        }
        if self
            .checker
            .value_symbol_links
            .try_get(symbol)
            .is_some_and(|links| links.name_type.is_some())
        {
            return Err(Error::Unsupported("getNameOfSymbolFromNameType"));
        }
        Ok(JsString::from_bytes(
            ts_ast::escape_internal_symbol_name(read.name_bytes()).into_owned(),
        ))
    }

    // With no enclosing declaration or UseFullyQualifiedType, upstream's
    // lookupSymbolChain returns the symbol itself, regardless of its parent.
    // port: tsc/internal/checker/nodebuilderimpl.go:NodeBuilderImpl.symbolToExpression
    pub(crate) fn symbol_node(&mut self, symbol: SymbolId) -> Result<NodeId, Error> {
        if self.flags & ts_nodebuilder::flags::USE_FULLY_QUALIFIED_TYPE != 0 {
            return Err(Error::Unsupported("getSymbolChain: qualified type display"));
        }
        let name = self.symbol_name(symbol)?;
        self.approximate_length += name.len() + 1;
        let node = self.ast.new_identifier(name);
        self.emit
            .add_emit_flags(node, emit_flags::NO_ASCII_ESCAPING);
        Ok(node)
    }

    fn list(&mut self, nodes: Vec<NodeId>) -> Result<NodeListId, Error> {
        let nodes = self.ast.node_slice(nodes.into_iter().map(Some).collect())?;
        Ok(self.ast.new_list(TextRange::new(-1, -1), nodes)?)
    }

    fn keyword(&mut self, kind: K, length: usize) -> NodeId {
        self.approximate_length += length;
        self.ast.new_keyword_type_node(kind.into())
    }

    fn string_literal(&mut self, text: JsString) -> NodeId {
        let flags = if self.flags & nf::USE_SINGLE_QUOTES_FOR_STRING_LITERAL_TYPE != 0 {
            token_flags::SINGLE_QUOTE
        } else {
            0
        };
        self.ast.new_string_literal(text, flags)
    }

    fn type_reference(&mut self, symbol: SymbolId, arguments: &[TypeId]) -> Result<NodeId, Error> {
        let arguments = if arguments.is_empty() {
            None
        } else {
            Some(self.type_list(arguments, false)?)
        };
        let name = self.symbol_node(symbol)?;
        // createAccessFromSymbolChain charges the first component both when
        // obtaining its written name and when constructing its access node.
        self.approximate_length += self.ast.view().node_text(name)?.as_bytes().len() + 1;
        Ok(self.ast.new_type_reference_node(Some(name), arguments))
    }

    // port: tsc/internal/checker/nodebuilderimpl.go:NodeBuilderImpl.checkTruncationLength
    fn check_truncation(&mut self) -> bool {
        let limit = if self.flags & nf::NO_TRUNCATION != 0 {
            crate::NO_TRUNCATION_MAXIMUM_TRUNCATION_LENGTH
        } else {
            crate::DEFAULT_MAXIMUM_TRUNCATION_LENGTH
        };
        self.truncating |= self.approximate_length > limit;
        self.truncating
    }

    fn elision(&mut self, text: &[u8]) -> Result<NodeId, Error> {
        if self.flags & nf::NO_TRUNCATION != 0 {
            return Err(Error::Unsupported(
                "node builder synthetic elision comments",
            ));
        }
        let name = self.ast.new_identifier(JsString::from_bytes(text));
        Ok(self.ast.new_type_reference_node(Some(name), None))
    }

    // port: tsc/internal/checker/nodebuilderimpl.go:NodeBuilderImpl.mapToTypeNodes
    fn type_list(&mut self, types: &[TypeId], bare: bool) -> Result<NodeListId, Error> {
        if self.check_truncation() {
            if !bare {
                let node = self.elision(b"...")?;
                return self.list(vec![node]);
            }
            if types.len() > 2 {
                let first = self.type_node(types[0])?;
                let last = self.type_node(types[types.len() - 1])?;
                let elision =
                    self.elision(format!("... {} more ...", types.len() - 2).as_bytes())?;
                return self.list(vec![first, elision, last]);
            }
        }
        let mut nodes = Vec::new();
        let mut seen_names: Vec<(JsString, TypeId)> = Vec::new();
        for (index, &ty) in types.iter().enumerate() {
            let display_index = index + 1;
            if self.check_truncation() && display_index + 2 < types.len().saturating_sub(1) {
                nodes.push(
                    self.elision(
                        format!("... {} more ...", types.len() - display_index).as_bytes(),
                    )?,
                );
                nodes.push(self.type_node(types[types.len() - 1])?);
                break;
            }
            self.approximate_length += 2;
            let node = self.type_node(ty)?;
            let view = self.ast.view();
            if let Some(reference) = view.node(node)?.data_source().as_type_reference_node() {
                if let Some(name) = reference.type_name() {
                    if view.node(name)?.kind() == K::Identifier {
                        let name = JsString::from_bytes(view.node_text(name)?.as_bytes());
                        for (_, previous) in seen_names.iter().filter(|(seen, _)| *seen == name) {
                            let current = self.checker.types.get(ty)?;
                            let previous_record = self.checker.types.get(*previous)?;
                            // typesAreSameReference: shared type, symbol, or alias record.
                            if ty != *previous
                                && !(current.symbol.is_some()
                                    && current.symbol == previous_record.symbol)
                                && !(current.alias.is_some()
                                    && current.alias == previous_record.alias)
                            {
                                return Err(Error::Unsupported(
                                    "mapToTypeNodes: colliding names require qualified display",
                                ));
                            }
                        }
                        seen_names.push((name, ty));
                    }
                }
            }
            nodes.push(node);
        }
        self.list(nodes)
    }

    // port: tsc/internal/checker/nodebuilderimpl.go:NodeBuilderImpl.typeToTypeNode
    pub(crate) fn type_node(&mut self, ty: TypeId) -> Result<NodeId, Error> {
        let in_alias = self.flags & nf::IN_TYPE_ALIAS != 0;
        self.flags &= !nf::IN_TYPE_ALIAS;
        let record = *self.checker.types.get(ty)?;
        if record.flags & tf::ANY != 0 {
            if let Some(alias) = self.checker.types.alias_of(ty)?.cloned() {
                // TypeAlias.ToTypeReferenceNode uses raw entity-name symbols,
                // separately from the ordinary alias accessibility path below.
                let symbol = self.checker.symbol(alias.symbol)?;
                if symbol.parent().is_some() {
                    return Err(Error::Unsupported(
                        "TypeAlias.ToTypeReferenceNode: parent chain",
                    ));
                }
                let name = self.ast.new_identifier(symbol.name_to_owned());
                let arguments = if alias.type_arguments.is_empty() {
                    None
                } else {
                    Some(self.type_list(&alias.type_arguments, false)?)
                };
                return Ok(self.ast.new_type_reference_node(Some(name), arguments));
            }
            if ty == self.checker.builtins.unresolved_type {
                return Err(Error::Unsupported("unresolved type synthetic comment"));
            }
            return Ok(self.keyword(
                if ty == self.checker.builtins.intrinsic_marker_type {
                    K::IntrinsicKeyword
                } else {
                    K::AnyKeyword
                },
                3,
            ));
        }
        for (flag, kind, length) in [
            (tf::UNKNOWN, K::UnknownKeyword, 0),
            (tf::STRING, K::StringKeyword, 6),
            (tf::NUMBER, K::NumberKeyword, 6),
            (tf::BIG_INT, K::BigIntKeyword, 6),
        ] {
            if record.flags & flag != 0 {
                return Ok(self.keyword(kind, length));
            }
        }
        if record.flags & tf::BOOLEAN != 0 && record.alias.is_none() {
            return Ok(self.keyword(K::BooleanKeyword, 7));
        }
        if record.flags & tf::ENUM_LIKE != 0 {
            return Err(Error::Unsupported("typeToTypeNode: enum display"));
        }
        if record.flags & tf::LITERAL != 0 {
            let value = self.checker.types.literal(ty)?.value.clone();
            let literal = match value {
                LiteralValue::String(text) => {
                    self.approximate_length += text.len() + 2;
                    let literal = self.string_literal(text);
                    self.emit
                        .add_emit_flags(literal, emit_flags::NO_ASCII_ESCAPING);
                    literal
                }
                LiteralValue::Number(value) => {
                    let text = value.to_string();
                    self.approximate_length += text.len();
                    if value.value() < 0.0 {
                        let operand = self
                            .ast
                            .new_numeric_literal(JsString::from_bytes(&text.as_bytes()[1..]), 0);
                        self.ast
                            .new_prefix_unary_expression(K::MinusToken.into(), Some(operand))
                    } else {
                        self.ast
                            .new_numeric_literal(JsString::from_bytes(text.into_bytes()), 0)
                    }
                }
                LiteralValue::BigInt(value) => {
                    let mut text = value.to_text();
                    text.push(b'n');
                    self.approximate_length += text.len();
                    self.ast.new_big_int_literal(JsString::from_bytes(text), 0)
                }
                LiteralValue::Boolean(value) => {
                    self.approximate_length += if value { 4 } else { 5 };
                    self.ast.new_keyword_expression(
                        if value {
                            K::TrueKeyword
                        } else {
                            K::FalseKeyword
                        }
                        .into(),
                    )
                }
                LiteralValue::ComputedEnum => {
                    return Err(Error::Unsupported("typeToTypeNode: computed enum"))
                }
            };
            return Ok(self.ast.new_literal_type_node(Some(literal)));
        }
        if record.flags & tf::NULL != 0 {
            self.approximate_length += 4;
            let literal = self.ast.new_keyword_expression(K::NullKeyword.into());
            return Ok(self.ast.new_literal_type_node(Some(literal)));
        }
        for (flag, kind, length) in [
            (tf::VOID, K::VoidKeyword, 4),
            (tf::UNDEFINED, K::UndefinedKeyword, 9),
            (tf::NEVER, K::NeverKeyword, 5),
            (tf::ES_SYMBOL, K::SymbolKeyword, 6),
            (tf::NON_PRIMITIVE, K::ObjectKeyword, 6),
        ] {
            if record.flags & flag != 0 {
                return Ok(self.keyword(kind, length));
            }
        }
        if !in_alias {
            let alias = self.checker.types.alias_of(ty)?.cloned();
            if let Some(symbol) = crate::type_display::alias_symbol(alias.as_ref()) {
                return self.type_reference(
                    symbol,
                    crate::type_display::alias_type_arguments(alias.as_ref()),
                );
            }
        }
        if record.object_flags & of::CLASS_OR_INTERFACE != 0 {
            if !self
                .checker
                .types
                .interface(ty)?
                .type_parameters()
                .is_empty()
            {
                return Err(Error::Unsupported(
                    "typeReferenceToTypeNode: generic interface",
                ));
            }
            return self.type_reference(
                record
                    .symbol
                    .ok_or(Error::MissingLink("interface type symbol"))?,
                &[],
            );
        }
        if record.flags & tf::UNION != 0 {
            let union = self.checker.types.union(ty)?;
            let types = if let Some(origin) = union.origin {
                if self.checker.types.flags(origin)? & tf::UNION == 0 {
                    return Err(Error::Unsupported("typeToTypeNode: non-union origin"));
                }
                self.checker.types.union(origin)?.types.clone()
            } else {
                union.types.clone()
            };
            let types = self.format_union(&types)?;
            if types.len() == 1 {
                return self.type_node(types[0]);
            }
            let nodes = self.type_list(&types, true)?;
            return Ok(self.ast.new_union_type_node(Some(nodes)));
        }
        if record.object_flags & of::ANONYMOUS != 0 {
            return self.object_type(ty);
        }
        Err(Error::Unsupported(
            "typeToTypeNode: unsupported type family",
        ))
    }

    // port: tsc/internal/checker/printer.go:Checker.formatUnionTypes
    fn format_union(&self, types: &[TypeId]) -> Result<Vec<TypeId>, Error> {
        let mut result = Vec::new();
        let mut nullable = 0;
        let mut index = 0;
        while index < types.len() {
            let ty = types[index];
            let flags = self.checker.types.flags(ty)?;
            nullable |= flags & tf::NULLABLE;
            if flags & tf::NULLABLE == 0 {
                if flags & tf::BOOLEAN_LITERAL != 0
                    && index + 1 < types.len()
                    && self.checker.types.flags(types[index + 1])? & tf::BOOLEAN_LITERAL != 0
                    && self.checker.types.literal(types[index + 1])?.regular
                        == self.checker.builtins.regular_true_type
                {
                    result.push(self.checker.builtins.boolean_type);
                    index += 2;
                    continue;
                }
                if flags & tf::ENUM_LIKE != 0 {
                    return Err(Error::Unsupported("formatUnionTypes: enum"));
                }
                result.push(ty);
            }
            index += 1;
        }
        if nullable & tf::NULL != 0 {
            result.push(self.checker.builtins.null_type);
        }
        if nullable & tf::UNDEFINED != 0 {
            result.push(self.checker.builtins.undefined_type);
        }
        Ok(result)
    }

    // port: tsc/internal/checker/nodebuilderimpl.go:NodeBuilderImpl.createTypeNodeFromObjectType
    fn object_type(&mut self, ty: TypeId) -> Result<NodeId, Error> {
        if let Some(symbol) = self.checker.types.get(ty)?.symbol {
            if self.checker.symbol(symbol)?.flags()
                & (sf::CLASS | sf::ENUM | sf::VALUE_MODULE | sf::FUNCTION | sf::METHOD)
                != 0
            {
                return Err(Error::Unsupported("createAnonymousTypeNode: typeof symbol"));
            }
        }
        if self.visited.contains(&ty) {
            return Err(Error::Unsupported(
                "createAnonymousTypeNode: circularity recovery",
            ));
        }
        self.checker.resolve_type_members(ty)?;
        let members = self.checker.types.structured(ty)?;
        if members
            .signatures
            .as_ref()
            .is_some_and(|items| !items.is_empty())
            || members
                .index_infos
                .as_ref()
                .is_some_and(|items| !items.is_empty())
        {
            return Err(Error::Unsupported(
                "createTypeNodesFromResolvedType: signatures/index infos",
            ));
        }
        let properties = members.properties.clone().unwrap_or_default();
        self.visited.push(ty);
        let result = self.object_members(&properties);
        self.visited.pop();
        let members = self.list(result?)?;
        let node = self.ast.new_type_literal_node(Some(members));
        self.approximate_length += 2;
        if properties.is_empty() || self.flags & nf::MULTILINE_OBJECT_LITERALS == 0 {
            self.emit.set_emit_flags(node, emit_flags::SINGLE_LINE);
        }
        Ok(node)
    }

    fn elided_property(&mut self, text: &[u8]) -> Result<NodeId, Error> {
        if self.flags & nf::NO_TRUNCATION != 0 {
            return Err(Error::Unsupported(
                "node builder synthetic property elision comments",
            ));
        }
        let name = self.ast.new_identifier(JsString::from_bytes(text));
        Ok(self
            .ast
            .new_property_signature_declaration(None, Some(name), None, None, None))
    }

    // port: tsc/internal/checker/nodebuilderimpl.go:NodeBuilderImpl.createTypeNodesFromResolvedType
    fn object_members(&mut self, properties: &[SymbolId]) -> Result<Vec<NodeId>, Error> {
        if !properties.is_empty() && self.check_truncation() {
            return Ok(vec![self.elided_property(b"...")?]);
        }
        let mut members = Vec::new();
        for (index, &property) in properties.iter().enumerate() {
            let display_index = index + 1;
            if self.check_truncation() && display_index + 2 < properties.len().saturating_sub(1) {
                members.push(self.elided_property(
                    format!("... {} more ...", properties.len() - display_index).as_bytes(),
                )?);
                members.push(self.property(properties[properties.len() - 1])?);
                break;
            }
            members.push(self.property(property)?);
        }
        Ok(members)
    }

    // port: tsc/internal/checker/nodebuilderimpl.go:NodeBuilderImpl.addPropertyToElementList
    fn property(&mut self, symbol: SymbolId) -> Result<NodeId, Error> {
        let read = self.checker.symbol(symbol)?;
        if read.flags() & sf::PROPERTY == 0
            || read.flags() & (sf::METHOD | sf::ACCESSOR | sf::FUNCTION) != 0
            || read.check_flags() & check_flags::REVERSE_MAPPED != 0
        {
            return Err(Error::Unsupported(
                "addPropertyToElementList: non-property/accessor/reverse mapping",
            ));
        }
        if self
            .checker
            .value_symbol_links
            .try_get(symbol)
            .is_some_and(|links| links.name_type.is_some())
        {
            return Err(Error::Unsupported(
                "getPropertyNameNodeForSymbolFromNameType",
            ));
        }
        let name = read.name_to_owned();
        let optional = read.flags() & sf::OPTIONAL != 0;
        let mut readonly = read.check_flags() & check_flags::READONLY != 0;
        if read.check_flags() & check_flags::SYNTHETIC == 0 {
            if let Some(declaration) = read.value_declaration() {
                let view = self.checker.ast(declaration)?;
                readonly |=
                    view.node(declaration)?.modifier_flags(view)? & modifier_flags::READONLY != 0;
            }
        }
        let (string_named, single_quote) = self.property_name_style(symbol)?;
        if name
            .as_bytes()
            .starts_with(ts_ast::INTERNAL_SYMBOL_NAME_PREFIX)
        {
            return Err(Error::Unsupported(
                "getPropertyNameNodeForSymbol: late/private name",
            ));
        }
        let property_name =
            if ts_scanner::is_identifier_text(name.as_bytes(), LanguageVariant::STANDARD) {
                self.ast.new_identifier(name.clone())
            } else if !string_named
                && ts_jsnum::from_string(name.as_bytes())
                    .to_string()
                    .as_bytes()
                    == name.as_bytes()
                && ts_jsnum::from_string(name.as_bytes()).value() >= 0.0
            {
                self.ast.new_numeric_literal(name.clone(), 0)
            } else {
                self.ast.new_string_literal(
                    name.clone(),
                    if single_quote {
                        token_flags::SINGLE_QUOTE
                    } else {
                        0
                    },
                )
            };
        self.approximate_length += name.len() + 1;
        let mut ty = self.checker.get_type_of_symbol(symbol)?;
        // getNonMissingTypeOfSymbol -> removeMissingType. Undefined remains
        // visible when exactOptionalPropertyTypes is disabled.
        if optional && self.checker.options.exact_optional_property_types {
            let missing = self.checker.builtins.missing_type;
            ty = self
                .checker
                .map_type(ty, &mut |_, member| {
                    Ok((member != missing).then_some(member))
                })?
                .unwrap_or(self.checker.builtins.never_type);
        }
        let type_node = self.type_node(ty)?;
        let question = optional.then(|| self.ast.new_token(K::QuestionToken.into()));
        let modifiers = if readonly {
            self.approximate_length += 9;
            let modifier = self.ast.new_modifier(K::ReadonlyKeyword.into());
            Some(self.list(vec![modifier])?)
        } else {
            None
        };
        Ok(self.ast.new_property_signature_declaration(
            modifiers,
            Some(property_name),
            question,
            Some(type_node),
            None,
        ))
    }

    // port: tsc/internal/checker/nodebuilderimpl.go:NodeBuilderImpl.isStringNamed
    // port: tsc/internal/checker/nodebuilderimpl.go:NodeBuilderImpl.isSingleQuotedStringNamed
    fn property_name_style(&self, symbol: SymbolId) -> Result<(bool, bool), Error> {
        let declarations = self.checker.symbol_declarations(symbol)?;
        let mut string_named = !declarations.is_empty();
        let mut single_quote = string_named;
        for declaration in declarations.iter() {
            let declaration = declaration.ok_or(Error::MissingLink("property declaration"))?;
            let view = self.checker.ast(declaration)?;
            if !matches!(
                view.node(declaration)?.kind().known(),
                Some(
                    K::PropertySignature
                        | K::PropertyDeclaration
                        | K::PropertyAssignment
                        | K::ShorthandPropertyAssignment
                        | K::Parameter
                )
            ) {
                return Err(Error::Unsupported(
                    "addPropertyToElementList: unsupported property declaration",
                ));
            }
            let Some(name) = view.node(declaration)?.name() else {
                string_named = false;
                single_quote = false;
                continue;
            };
            let node = view.node(name)?;
            if node.kind() == K::ComputedPropertyName {
                return Err(Error::Unsupported("isStringNamed: computed property"));
            }
            let is_string = node.kind() == K::StringLiteral;
            string_named &= is_string;
            single_quote &= is_string
                && node
                    .data_source()
                    .as_string_literal()
                    .ok_or(ts_arena::Error::InvalidGraph)?
                    .token_flags()
                    & token_flags::SINGLE_QUOTE
                    != 0;
        }
        Ok((string_named, single_quote))
    }
}
