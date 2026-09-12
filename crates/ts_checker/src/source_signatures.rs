//! Source signatures and their lazy instantiations. Instantiating a signature
//! leaves its return type and predicate unresolved, because reading either can
//! fix type inferences. This ordering is observable (`checker.go`).

use crate::{
    signature_flags as sg, CheckerState, Error, MapperId, SignatureId, TypeId, TypeSystemEntity,
    TypeSystemPropertyName,
};
use ts_arena::{NodeId, SymbolId};
use ts_ast::{modifier_flags as mf, node_flags as nf, SyntaxKind as K};

impl CheckerState {
    // port: tsc/internal/checker/relater.go:Checker.getTypePredicateOfSignature
    pub(crate) fn type_predicate_of_signature(
        &mut self,
        signature: SignatureId,
    ) -> Result<Option<crate::TypePredicateId>, Error> {
        let sig = self.signatures.get(signature)?;
        if let Some(predicate) = sig.resolved_type_predicate {
            return Ok((predicate != self.builtins.no_type_predicate).then_some(predicate));
        }
        let target = sig.target;
        let composite = sig.composite.clone();
        let mapper = sig.mapper;
        let declaration = sig.declaration;
        let predicate = if let Some(target) = target {
            match self.type_predicate_of_signature(target)? {
                Some(predicate) => {
                    let mut data = self.signatures.predicate(predicate)?.clone();
                    let ty = data
                        .t
                        .map(|ty| self.instantiate_type(ty, mapper))
                        .transpose()?;
                    Some(if ty == data.t {
                        predicate
                    } else {
                        data.t = ty;
                        self.signatures.new_type_predicate(data)?
                    })
                }
                None => None,
            }
        } else if let Some(composite) = composite {
            self.compound_type_predicate(&composite.signatures, composite.is_union)?
        } else if let Some(declaration) = declaration {
            let read = self.ast(declaration)?.node(declaration)?;
            match read.type_node() {
                Some(node) if self.ast(node)?.node(node)?.kind() == K::TypePredicate => {
                    Some(self.predicate_from_node(node, signature)?)
                }
                None if read.body().is_some() => {
                    return Err(Error::Unsupported("getTypePredicateFromBody"))
                }
                Some(_) | None => None,
            }
        } else {
            None
        };
        self.signatures.get_mut(signature)?.resolved_type_predicate =
            Some(predicate.unwrap_or(self.builtins.no_type_predicate));
        Ok(predicate)
    }

    // port: tsc/internal/checker/relater.go:Checker.createTypePredicateFromTypePredicateNode
    fn predicate_from_node(
        &mut self,
        node: NodeId,
        signature: SignatureId,
    ) -> Result<crate::TypePredicateId, Error> {
        use crate::TypePredicateKind as Kind;
        let read = self.ast(node)?.node(node)?;
        let data = read
            .data_source()
            .as_type_predicate_node()
            .ok_or(Error::MissingLink("type predicate"))?;
        let name = data
            .parameter_name()
            .ok_or(Error::MissingLink("predicate name"))?;
        let this = self.ast(name)?.node(name)?.kind() == K::ThisType;
        let asserts = data.asserts_modifier().is_some();
        let annotation = data.r#type();
        let text = if this {
            ts_ast::JsString::default()
        } else {
            self.ast(name)?.node_text(name)?.into_js_string()
        };
        let mut parameter_index = if this { 0 } else { -1 };
        if !this {
            for (index, &parameter) in self
                .signatures
                .get(signature)?
                .parameters
                .as_deref()
                .unwrap_or_default()
                .iter()
                .enumerate()
            {
                if self.symbol(parameter)?.name_bytes() == text.as_bytes() {
                    parameter_index = index as i32;
                    break;
                }
            }
        }
        let kind = match (this, asserts) {
            (true, true) => Kind::AssertsThis,
            (true, false) => Kind::This,
            (false, true) => Kind::AssertsIdentifier,
            (false, false) => Kind::Identifier,
        };
        let ty = annotation
            .map(|node| self.get_type_from_type_node(node))
            .transpose()?;
        self.signatures
            .new_type_predicate(crate::signatures::TypePredicate {
                kind,
                parameter_index,
                parameter_name: text,
                t: ty,
            })
    }

    // port: tsc/internal/checker/checker.go:Checker.getSignaturesOfSymbol
    pub(crate) fn signatures_of_symbol(
        &mut self,
        symbol: Option<SymbolId>,
    ) -> Result<Vec<SignatureId>, Error> {
        let Some(symbol) = symbol else {
            return Ok(Vec::new());
        };
        let declarations = self.symbol_declarations(symbol)?.to_vec();
        let mut signatures = Vec::new();
        for (index, &declaration) in declarations.iter().enumerate() {
            let Some(declaration) = declaration else {
                continue;
            };
            let read = self.ast(declaration)?.node(declaration)?;
            if !ts_ast::utilities::is_function_like(Some(&read)) {
                continue;
            }
            if index > 0 && read.body().is_some() {
                if let Some(previous) = declarations[index - 1] {
                    let previous = self.ast(previous)?.node(previous)?;
                    if read.parent() == previous.parent()
                        && read.kind() == previous.kind()
                        && (read.pos() == previous.end() || previous.flags() & nf::REPARSED != 0)
                    {
                        continue;
                    }
                }
            }
            signatures.push(self.signature_from_declaration(declaration)?);
        }
        Ok(signatures)
    }

    // port: tsc/internal/checker/checker.go:Checker.getSignatureFromDeclaration
    pub(crate) fn signature_from_declaration(
        &mut self,
        node: NodeId,
    ) -> Result<SignatureId, Error> {
        if let Some(Some(signature)) = self.query.source_signatures.try_get(node) {
            return Ok(*signature);
        }
        let read = self.ast(node)?.node(node)?;
        if read.flags() & nf::JAVA_SCRIPT_FILE != 0 {
            return Err(Error::Unsupported(
                "getSignatureFromDeclaration: JavaScript contextual signature",
            ));
        }
        let construct = matches!(
            read.kind().known(),
            Some(K::Constructor | K::ConstructorType | K::ConstructSignature)
        );
        let mut flags = if construct { sg::CONSTRUCT } else { 0 };
        if construct && read.modifier_flags(self.ast(node)?)? & mf::ABSTRACT != 0 {
            flags |= sg::ABSTRACT;
        }
        let nodes = self.source_list(node, read.parameter_list())?;
        let mut parameters = Vec::new();
        let mut this_parameter = None;
        let mut minimum = 0;
        for (index, parameter) in nodes.iter().copied().enumerate() {
            let symbol = self
                .get_symbol_of_declaration(parameter)?
                .ok_or(Error::MissingLink("parameter symbol"))?;
            let read = self.ast(parameter)?.node(parameter)?;
            if index == 0 && self.symbol(symbol)?.name_bytes() == b"this" {
                this_parameter = Some(symbol);
            } else {
                parameters.push(symbol);
            }
            if let Some(annotation) = read.type_node() {
                if self.ast(annotation)?.node(annotation)?.kind() == K::LiteralType {
                    flags |= sg::HAS_LITERAL_TYPES;
                }
            }
            let rest = read
                .data_source()
                .as_parameter_declaration()
                .ok_or(Error::MissingLink("parameter declaration"))?
                .dot_dot_dot_token()
                .is_some();
            if index + 1 == nodes.len() && rest {
                flags |= sg::HAS_REST_PARAMETER;
            }
            if read.question_token(self.ast(parameter)?)?.is_none()
                && read.initializer().is_none()
                && !rest
            {
                minimum = parameters.len();
            }
        }
        let type_parameters = self.type_parameters_from_declaration(node)?;
        let signature = self.signatures.new_signature(
            flags,
            Some(node),
            (!type_parameters.is_empty()).then_some(type_parameters),
            this_parameter,
            (!parameters.is_empty()).then(|| parameters.into()),
            None,
            None,
            minimum as i32,
        )?;
        *self.query.source_signatures.get_or_default(node) = Some(signature);
        Ok(signature)
    }

    // port: tsc/internal/checker/checker.go:Checker.getReturnTypeOfSignature
    pub(crate) fn return_type_of_signature(
        &mut self,
        signature: SignatureId,
    ) -> Result<TypeId, Error> {
        let sig = self.signatures.get(signature)?;
        if let Some(ty) = sig.resolved_return_type {
            return Ok(ty);
        }
        let declaration = sig.declaration;
        let target = sig.target;
        let composite = sig.composite.clone();
        let mapper = sig.mapper;
        let signatures = &self.signatures;
        if !self.resolution.push(
            TypeSystemEntity::Signature(signature),
            TypeSystemPropertyName::ResolvedReturnType,
            |entry| {
                let TypeSystemEntity::Signature(signature) = entry.target else {
                    return false;
                };
                entry.property_name == TypeSystemPropertyName::ResolvedReturnType
                    && signatures
                        .get(signature)
                        .is_ok_and(|signature| signature.resolved_return_type.is_some())
            },
        ) {
            return Ok(self.builtins.error_type);
        }
        let result = (|| {
            if let Some(target) = target {
                let ty = self.return_type_of_signature(target)?;
                return self.instantiate_type(ty, mapper);
            }
            if let Some(composite) = composite {
                let mut returns = Vec::with_capacity(composite.signatures.len());
                for &signature in composite.signatures.iter() {
                    returns.push(self.return_type_of_signature(signature)?);
                }
                let ty = if composite.is_union {
                    self.get_union_type_ex(&returns, crate::UnionReduction::Subtype, None, None)?
                } else {
                    self.get_intersection_type(&returns)?
                };
                return self.instantiate_type(ty, mapper);
            }
            let node = declaration.ok_or(Error::MissingLink("signature declaration"))?;
            let read = self.ast(node)?.node(node)?;
            if let Some(annotation) = read.type_node() {
                return self.get_type_from_type_node(annotation);
            }
            if read.body().is_some() {
                return Err(Error::Unsupported("getReturnTypeFromBody"));
            }
            Ok(self.builtins.any_type)
        })();
        let complete = self.resolution.pop();
        let mut ty = result?;
        if !complete {
            if let Some(node) = declaration {
                let annotation = self.ast(node)?.node(node)?.type_node();
                if let Some(annotation) = annotation {
                    self.error_at(
                        Some(annotation),
                        ts_diagnostics::Return_type_annotation_circularly_references_itself,
                        vec![],
                    )?;
                } else {
                    return Err(Error::Unsupported(
                        "getReturnTypeOfSignature: circular body diagnostic",
                    ));
                }
            }
            ty = self.builtins.any_type;
        }
        Ok(*self
            .signatures
            .get_mut(signature)?
            .resolved_return_type
            .get_or_insert(ty))
    }

    // port: tsc/internal/checker/checker.go:Checker.instantiateSignature
    pub(crate) fn instantiate_signature(
        &mut self,
        signature: SignatureId,
        mapper: MapperId,
    ) -> Result<SignatureId, Error> {
        self.instantiate_signature_ex(signature, mapper, false)
    }

    // port: tsc/internal/checker/checker.go:Checker.instantiateSignatureEx
    pub(crate) fn instantiate_signature_ex(
        &mut self,
        signature: SignatureId,
        mapper: MapperId,
        erase: bool,
    ) -> Result<SignatureId, Error> {
        let sig = self.signatures.get(signature)?.clone();
        let mut mapper = mapper;
        let parameters = if erase {
            &[]
        } else {
            sig.type_parameters.as_deref().unwrap_or_default()
        };
        let mut fresh = Vec::with_capacity(parameters.len());
        for &parameter in parameters {
            let ty = self.new_type_parameter(self.types.get(parameter)?.symbol)?;
            self.types.type_parameter_mut(ty)?.target = Some(parameter);
            fresh.push(ty);
        }
        if !fresh.is_empty() {
            let fresh_mapper = self.new_type_mapper(parameters, &fresh)?;
            mapper = self.combine_type_mappers(Some(fresh_mapper), mapper)?;
            for &parameter in &fresh {
                self.types.type_parameter_mut(parameter)?.mapper = Some(mapper);
            }
        }
        let this_parameter = sig
            .this_parameter
            .map(|symbol| self.instantiate_symbol(symbol, mapper))
            .transpose()?;
        let mut parameters = Vec::new();
        for &parameter in sig.parameters.as_deref().unwrap_or_default() {
            parameters.push(self.instantiate_symbol(parameter, mapper)?);
        }
        let result = self.signatures.new_signature(
            sig.flags & sg::PROPAGATING_FLAGS,
            sig.declaration,
            (!fresh.is_empty()).then(|| fresh.into()),
            this_parameter,
            (!parameters.is_empty()).then(|| parameters.into()),
            None,
            None,
            sig.min_argument_count,
        )?;
        let instantiated = self.signatures.get_mut(result)?;
        instantiated.target = Some(signature);
        instantiated.mapper = Some(mapper);
        Ok(result)
    }

    // port: tsc/internal/checker/checker.go:Checker.getTypeOfInstantiatedSymbol
    pub(crate) fn get_type_of_instantiated_symbol(
        &mut self,
        symbol: SymbolId,
    ) -> Result<TypeId, Error> {
        let links = self
            .value_symbol_links
            .try_get(symbol)
            .copied()
            .ok_or(Error::MissingLink("instantiated symbol links"))?;
        if let Some(ty) = links.resolved_type {
            return Ok(ty);
        }
        let target = links
            .target
            .ok_or(Error::MissingLink("instantiated symbol target"))?;
        let ty = self.get_type_of_symbol(target)?;
        let ty = self.instantiate_type(ty, links.mapper)?;
        self.value_symbol_links.get_or_default(symbol).resolved_type = Some(ty);
        Ok(ty)
    }
}
