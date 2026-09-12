//! Array, tuple and signature syntax emitted by the type node builder.

use super::{
    check_flags, emit_flags, nf, sf, Error, Factory, FactoryMethods, JsString, NodeBuilder, NodeId,
    SymbolId, TypeId, K,
};
use crate::{element_flags as ef, signature_flags as sg, IndexInfoId, SignatureId};

impl NodeBuilder<'_> {
    // port: tsc/internal/checker/nodebuilderimpl.go:NodeBuilderImpl.typeReferenceToTypeNode
    pub(super) fn array_or_tuple_node(&mut self, ty: TypeId) -> Result<NodeId, Error> {
        let target = self.checker.types.target(ty)?;
        let arguments = self.checker.element_types(ty)?;
        if self.checker.is_array_type(ty)? {
            let readonly = target == self.checker.array_target(true)?
                && target != self.checker.array_target(false)?;
            let element = self.type_node(arguments[0])?;
            if self.flags & nf::WRITE_ARRAY_AS_GENERIC_TYPE != 0 {
                let name = self.ast.new_identifier(JsString::from_bytes(if readonly {
                    b"ReadonlyArray".as_slice()
                } else {
                    b"Array".as_slice()
                }));
                let arguments = self.list(vec![element])?;
                return Ok(self
                    .ast
                    .new_type_reference_node(Some(name), Some(arguments)));
            }
            let node = self.ast.new_array_type_node(Some(element));
            return Ok(if readonly {
                self.ast
                    .new_type_operator_node(K::ReadonlyKeyword.into(), Some(node))
            } else {
                node
            });
        }
        let data = self.checker.types.tuple(target)?;
        let readonly = data.readonly;
        let infos = data.element_infos.clone();
        let mut nodes = Vec::new();
        for (&argument, info) in arguments.iter().zip(infos.iter()) {
            let argument = self.without_missing(argument, info.flags & ef::OPTIONAL != 0)?;
            let mut node = self.type_node(argument)?;
            if info.flags & ef::REST != 0 {
                node = self.ast.new_array_type_node(Some(node));
            }
            let question = (info.flags & ef::OPTIONAL != 0)
                .then(|| self.ast.new_token(K::QuestionToken.into()));
            let rest = (info.flags & ef::VARIABLE != 0)
                .then(|| self.ast.new_token(K::DotDotDotToken.into()));
            if let Some(declaration) = info.labeled_declaration {
                let view = self.checker.ast(declaration)?;
                let name = view
                    .node(declaration)?
                    .name()
                    .ok_or(Error::MissingLink("tuple label"))?;
                if view.node(name)?.kind() != K::Identifier {
                    return Err(Error::Unsupported("getTupleElementLabel: binding pattern"));
                }
                let name = self
                    .ast
                    .new_identifier(view.node_text(name)?.into_js_string());
                node = self
                    .ast
                    .new_named_tuple_member(rest, Some(name), question, Some(node));
            } else if rest.is_some() {
                node = self.ast.new_rest_type_node(Some(node));
            } else if question.is_some() {
                node = self.ast.new_optional_type_node(Some(node));
            }
            nodes.push(node);
        }
        let elements = self.list(nodes)?;
        let tuple = self.ast.new_tuple_type_node(Some(elements));
        self.emit.set_emit_flags(tuple, emit_flags::SINGLE_LINE);
        Ok(if readonly {
            self.ast
                .new_type_operator_node(K::ReadonlyKeyword.into(), Some(tuple))
        } else {
            tuple
        })
    }

    fn without_missing(&mut self, ty: TypeId, optional: bool) -> Result<TypeId, Error> {
        if !optional || !self.checker.options.exact_optional_property_types {
            return Ok(ty);
        }
        let missing = self.checker.builtins.missing_type;
        Ok(self
            .checker
            .map_type(ty, &mut |_, ty| Ok((ty != missing).then_some(ty)))?
            .unwrap_or(self.checker.builtins.never_type))
    }

    // port: tsc/internal/checker/nodebuilderimpl.go:NodeBuilderImpl.typeParameterToDeclarationWithConstraint
    pub(super) fn type_parameter_node(&mut self, ty: TypeId) -> Result<NodeId, Error> {
        let constraint = self
            .checker
            .constraint_of_type_parameter(ty)?
            .map(|ty| self.type_node(ty))
            .transpose()?;
        self.type_parameter_node_with_constraint(ty, constraint)
    }

    pub(super) fn type_parameter_node_with_constraint(
        &mut self,
        ty: TypeId,
        constraint: Option<NodeId>,
    ) -> Result<NodeId, Error> {
        let symbol = self.checker.types.get(ty)?.symbol;
        let name = if let Some(symbol) = symbol {
            self.symbol_node(symbol)?
        } else {
            self.ast
                .new_identifier(JsString::from_bytes(b"?".as_slice()))
        };
        let default = self.checker.resolved_type_parameter_default(ty)?;
        let default = if default == self.checker.builtins.no_constraint_type
            || default == self.checker.builtins.circular_constraint_type
        {
            None
        } else {
            Some(self.type_node(default)?)
        };
        if let Some(symbol) = symbol {
            for declaration in self.checker.symbol_declarations(symbol)?.iter().flatten() {
                if self
                    .checker
                    .ast(declaration)?
                    .node(declaration)?
                    .modifiers()
                    .is_some()
                {
                    return Err(Error::Unsupported(
                        "typeParameterToDeclaration: variance/const modifiers",
                    ));
                }
            }
        }
        Ok(self
            .ast
            .new_type_parameter_declaration(None, Some(name), constraint, None, default))
    }

    // port: tsc/internal/checker/nodebuilderimpl.go:NodeBuilderImpl.symbolToParameterDeclaration
    fn parameter_node(&mut self, symbol: SymbolId) -> Result<NodeId, Error> {
        let value = self.checker.symbol(symbol)?;
        let text = value.name_to_owned();
        let mut source_name = None;
        let mut rest = value.check_flags() & check_flags::REST_PARAMETER != 0;
        let mut optional = value.check_flags() & check_flags::OPTIONAL_PARAMETER != 0;
        if let Some(node) = value.value_declaration() {
            let read = self.checker.ast(node)?.node(node)?;
            if let Some(name) = read.name() {
                source_name = Some(name);
            }
            if let Some(parameter) = read.data_source().as_parameter_declaration() {
                rest |= parameter.dot_dot_dot_token().is_some();
            }
            optional |= read.question_token(self.checker.ast(node)?)?.is_some()
                || read.initializer().is_some();
        }
        let ty = self.checker.get_type_of_symbol(symbol)?;
        let annotation = self.type_node(ty)?;
        let name = match source_name {
            Some(name) => self.clone_binding_name(name)?,
            None => self.ast.new_identifier(text.clone()),
        };
        let rest = rest.then(|| self.ast.new_token(K::DotDotDotToken.into()));
        let question = optional.then(|| self.ast.new_token(K::QuestionToken.into()));
        self.approximate_length += text.len() + 3;
        Ok(self.ast.new_parameter_declaration(
            None,
            rest,
            Some(name),
            question,
            Some(annotation),
            None,
        ))
    }

    // port: tsc/internal/checker/nodebuilderimpl.go:NodeBuilderImpl.cloneBindingName
    fn clone_binding_name(&mut self, source: NodeId) -> Result<NodeId, Error> {
        stacker::maybe_grow(128 * 1024, 2 * 1024 * 1024, || {
            self.clone_binding_name_worker(source)
        })
    }

    fn clone_binding_name_worker(&mut self, source: NodeId) -> Result<NodeId, Error> {
        let read = self.checker.ast(source)?.node(source)?;
        let kind = read.kind();
        let result = match kind.known() {
            Some(K::Identifier) => self.ast.new_identifier(
                self.checker
                    .ast(source)?
                    .node_text(source)?
                    .into_js_string(),
            ),
            Some(K::QualifiedName) => {
                let right = read
                    .data_source()
                    .as_qualified_name()
                    .and_then(|data| data.right())
                    .ok_or(Error::MissingLink("qualified parameter name"))?;
                return self.clone_binding_name(right);
            }
            Some(K::StringLiteral) => {
                let flags = read
                    .data_source()
                    .as_string_literal()
                    .ok_or(Error::MissingLink("binding string name"))?
                    .token_flags();
                self.ast.new_string_literal(
                    self.checker
                        .ast(source)?
                        .node_text(source)?
                        .into_js_string(),
                    flags,
                )
            }
            Some(K::NumericLiteral) => self.ast.new_numeric_literal(
                self.checker
                    .ast(source)?
                    .node_text(source)?
                    .into_js_string(),
                0,
            ),
            Some(K::ArrayBindingPattern | K::ObjectBindingPattern) => {
                let source_elements = self.checker.source_list(source, read.element_list())?;
                let mut elements = Vec::with_capacity(source_elements.len());
                for element in source_elements {
                    elements.push(self.clone_binding_name(element)?);
                }
                let elements = self.list(elements)?;
                self.ast.new_binding_pattern(kind, Some(elements))
            }
            Some(K::BindingElement) => {
                let data = read
                    .data_source()
                    .as_binding_element()
                    .ok_or(Error::MissingLink("binding element"))?;
                let (rest, property, name) = (
                    data.dot_dot_dot_token().is_some(),
                    data.property_name(),
                    data.name(),
                );
                let rest = rest.then(|| self.ast.new_token(K::DotDotDotToken.into()));
                let property = property
                    .map(|node| self.clone_binding_name(node))
                    .transpose()?;
                let name = name.map(|node| self.clone_binding_name(node)).transpose()?;
                self.ast.new_binding_element(rest, property, name, None)
            }
            _ => {
                return Err(Error::Unsupported(
                    "cloneBindingName: computed name tracking/expression",
                ))
            }
        };
        self.emit.set_emit_flags(
            result,
            emit_flags::SINGLE_LINE | emit_flags::NO_ASCII_ESCAPING,
        );
        Ok(result)
    }

    // port: tsc/internal/checker/nodebuilderimpl.go:NodeBuilderImpl.signatureToSignatureDeclarationHelper
    pub(crate) fn signature_node(
        &mut self,
        signature: SignatureId,
        kind: K,
        name: Option<NodeId>,
        question: Option<NodeId>,
    ) -> Result<NodeId, Error> {
        let sig = self.checker.signatures.get(signature)?.clone();
        let expanded = self.checker.expanded_signature_parameters(signature)?;
        self.approximate_length += 3;
        let mut type_parameters = Vec::new();
        for &ty in sig.type_parameters.as_deref().unwrap_or_default() {
            type_parameters.push(self.type_parameter_node(ty)?);
        }
        let type_parameters = if type_parameters.is_empty() {
            None
        } else {
            Some(self.list(type_parameters)?)
        };
        let mut parameters = Vec::new();
        if self.flags & nf::OMIT_THIS_PARAMETER == 0 {
            if let Some(this) = sig.this_parameter {
                parameters.push(self.parameter_node(this)?);
            }
        }
        for parameter in expanded {
            parameters.push(self.parameter_node(parameter)?);
        }
        let parameters = self.list(parameters)?;
        let return_type = self.checker.return_type_of_signature(signature)?;
        let return_type = match self.checker.type_predicate_of_signature(signature)? {
            Some(predicate) => self.predicate_node(predicate)?,
            None => self.type_node(return_type)?,
        };
        let modifiers = if kind == K::ConstructorType && sig.flags & sg::ABSTRACT != 0 {
            let abstract_modifier = self.ast.new_modifier(K::AbstractKeyword.into());
            Some(self.list(vec![abstract_modifier])?)
        } else {
            None
        };
        Ok(match kind {
            K::FunctionType => self.ast.new_function_type_node(
                type_parameters,
                Some(parameters),
                Some(return_type),
            ),
            K::ConstructorType => self.ast.new_constructor_type_node(
                modifiers,
                type_parameters,
                Some(parameters),
                Some(return_type),
            ),
            K::CallSignature => self.ast.new_call_signature_declaration(
                type_parameters,
                Some(parameters),
                Some(return_type),
            ),
            K::ConstructSignature => self.ast.new_construct_signature_declaration(
                type_parameters,
                Some(parameters),
                Some(return_type),
            ),
            K::MethodSignature => self.ast.new_method_signature_declaration(
                None,
                name,
                question,
                type_parameters,
                Some(parameters),
                Some(return_type),
            ),
            _ => {
                return Err(Error::Unsupported(
                    "signatureToSignatureDeclarationHelper: syntax kind",
                ))
            }
        })
    }

    pub(super) fn index_signature_node(&mut self, index: IndexInfoId) -> Result<NodeId, Error> {
        self.index_signature_node_with_type(index, None)
    }

    pub(super) fn index_signature_node_with_type(
        &mut self,
        index: IndexInfoId,
        value_node: Option<NodeId>,
    ) -> Result<NodeId, Error> {
        let info = self.checker.signatures.index_info(index)?.clone();
        let name = if let Some(declaration) = info.declaration {
            let view = self.checker.ast(declaration)?;
            let parameters = self
                .checker
                .source_list(declaration, view.node(declaration)?.parameter_list())?;
            let name = view
                .node(parameters[0])?
                .name()
                .ok_or(Error::MissingLink("index parameter name"))?;
            view.node_text(name)?.into_js_string()
        } else {
            JsString::from_bytes(b"x".as_slice())
        };
        let name = self.ast.new_identifier(name);
        let key = self.type_node(info.key_type)?;
        let value = match value_node {
            Some(node) => node,
            None => self.type_node(info.value_type)?,
        };
        let parameter =
            self.ast
                .new_parameter_declaration(None, None, Some(name), None, Some(key), None);
        let parameters = self.list(vec![parameter])?;
        let modifiers = if info.is_readonly {
            let token = self.ast.new_modifier(K::ReadonlyKeyword.into());
            Some(self.list(vec![token])?)
        } else {
            None
        };
        Ok(self
            .ast
            .new_index_signature_declaration(modifiers, Some(parameters), Some(value)))
    }

    // port: tsc/internal/checker/nodebuilderimpl.go:NodeBuilderImpl.typePredicateToTypePredicateNodeHelper
    pub(crate) fn predicate_node(
        &mut self,
        predicate: crate::TypePredicateId,
    ) -> Result<NodeId, Error> {
        use crate::TypePredicateKind as Kind;
        let data = self.checker.signatures.predicate(predicate)?.clone();
        let asserts = matches!(data.kind, Kind::AssertsThis | Kind::AssertsIdentifier)
            .then(|| self.ast.new_token(K::AssertsKeyword.into()));
        let name = if matches!(data.kind, Kind::Identifier | Kind::AssertsIdentifier) {
            let node = self.ast.new_identifier(data.parameter_name);
            self.emit
                .set_emit_flags(node, emit_flags::NO_ASCII_ESCAPING);
            node
        } else {
            self.ast.new_this_type_node()
        };
        let annotation = data.t.map(|ty| self.type_node(ty)).transpose()?;
        Ok(self
            .ast
            .new_type_predicate_node(asserts, Some(name), annotation))
    }

    // port: tsc/internal/checker/nodebuilderimpl.go:NodeBuilderImpl.addPropertyToElementList
    pub(super) fn property_elements(&mut self, symbol: SymbolId) -> Result<Vec<NodeId>, Error> {
        let value = self.checker.symbol(symbol)?;
        if value.flags() & (sf::METHOD | sf::FUNCTION) == 0 {
            return self.property(symbol).map(|node| vec![node]);
        }
        let text = self.symbol_name(symbol)?;
        let optional = value.flags() & sf::OPTIONAL != 0;
        let ty = self.checker.get_type_of_symbol(symbol)?;
        let signatures = self.checker.signatures_of_type(ty, false)?;
        let mut result = Vec::new();
        for signature in signatures {
            let name = self.ast.new_identifier(text.clone());
            let question = optional.then(|| self.ast.new_token(K::QuestionToken.into()));
            result.push(self.signature_node(
                signature,
                K::MethodSignature,
                Some(name),
                question,
            )?);
        }
        Ok(result)
    }
}

impl NodeBuilder<'_> {
    // port: tsc/internal/checker/nodebuilderimpl.go:NodeBuilderImpl.createMappedTypeNodeFromType
    pub(super) fn mapped_type_node(&mut self, ty: TypeId) -> Result<NodeId, Error> {
        let declaration = self.checker.mapped_declaration(ty)?;
        let read = self.checker.ast(declaration)?.node(declaration)?;
        let data = read
            .data_source()
            .as_mapped_type_node()
            .ok_or(Error::MissingLink("mapped display syntax"))?;
        let readonly = data
            .readonly_token()
            .map(|node| {
                self.checker
                    .ast(node)?
                    .node(node)
                    .map(|read| read.kind())
                    .map_err(Error::from)
            })
            .transpose()?;
        let question = data
            .question_token()
            .map(|node| {
                self.checker
                    .ast(node)?
                    .node(node)
                    .map(|read| read.kind())
                    .map_err(Error::from)
            })
            .transpose()?;
        if self.flags & ts_nodebuilder::flags::GENERATE_NAMES_FOR_SHADOWED_TYPE_PARAMS != 0 {
            return Err(Error::Unsupported(
                "mapped display: modifier preserving wrapper",
            ));
        }
        let template = self.checker.mapped_template(ty)?;
        let parameter = self.checker.mapped_parameter(ty)?;
        let constraint = if self.checker.mapped_keyof_constraint(ty)? {
            let modifiers = self.checker.mapped_modifiers_type(ty)?;
            let operand = self.type_node(modifiers)?;
            self.ast
                .new_type_operator_node(K::KeyOfKeyword.into(), Some(operand))
        } else {
            let constraint = self.checker.mapped_constraint(ty)?;
            self.type_node(constraint)?
        };
        let name = self.symbol_node(
            self.checker
                .types
                .get(parameter)?
                .symbol
                .ok_or(Error::MissingLink("mapped parameter display"))?,
        )?;
        let parameter =
            self.ast
                .new_type_parameter_declaration(None, Some(name), Some(constraint), None, None);
        let name_type = self
            .checker
            .mapped_name(ty)?
            .map(|ty| self.type_node(ty))
            .transpose()?;
        let optional = self.checker.mapped_modifiers(ty)? & crate::mapped::INCLUDE_OPTIONAL != 0;
        let template = self.without_missing(template, optional)?;
        let template = self.type_node(template)?;
        let readonly = readonly.map(|kind| self.ast.new_token(kind));
        let question = question.map(|kind| self.ast.new_token(kind));
        let result = self.ast.new_mapped_type_node(
            readonly,
            Some(parameter),
            name_type,
            question,
            Some(template),
            None,
        );
        self.approximate_length += 10;
        self.emit.add_emit_flags(result, emit_flags::SINGLE_LINE);
        Ok(result)
    }
}
