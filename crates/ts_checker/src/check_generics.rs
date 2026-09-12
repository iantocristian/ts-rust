//! Checking source generic parameters and argument lists. Type-node resolution
//! is separate from checking their source children, even when resolution hits a
//! cache or eliminates a constituent.

use crate::{type_flags as tf, CheckerState, Error, TypeId};
use ts_arena::NodeId;
use ts_ast::SyntaxKind as K;

impl CheckerState {
    // port: tsc/internal/checker/checker.go:Checker.checkTypeParameters
    pub(crate) fn check_type_parameters(&mut self, node: NodeId) -> Result<(), Error> {
        let nodes = self.source_list(node, self.ast(node)?.node(node)?.type_parameter_list())?;
        let mut seen_default = false;
        for (index, &parameter) in nodes.iter().enumerate() {
            self.check_type_parameter(parameter)?;
            let read = self.ast(parameter)?.node(parameter)?;
            let default = read
                .data_source()
                .as_type_parameter_declaration()
                .ok_or(Error::MissingLink("type parameter declaration"))?
                .default_type();
            let name = read.name();
            if let Some(default) = default {
                seen_default = true;
                self.check_default_references(default, &nodes[index..])?;
            } else if seen_default {
                self.error_at(Some(parameter), ts_diagnostics::Required_type_parameters_may_not_follow_optional_type_parameters, vec![])?;
            }
            let symbol = self.get_symbol_of_declaration(parameter)?;
            for &previous in &nodes[..index] {
                if self.get_symbol_of_declaration(previous)? == symbol {
                    let text = ts_scanner::declaration_name_to_string(self.ast(parameter)?, name)?;
                    self.error_at(name, ts_diagnostics::Duplicate_identifier_0, vec![text])?;
                }
            }
        }
        Ok(())
    }

    // port: tsc/internal/checker/checker.go:Checker.checkTypeParametersNotReferenced
    fn check_default_references(&mut self, node: NodeId, later: &[NodeId]) -> Result<(), Error> {
        if self.ast(node)?.node(node)?.kind() == K::TypeReference {
            let ty = self.source_type_reference(node)?;
            if self.types.flags(ty)? & tf::TYPE_PARAMETER != 0 {
                for &parameter in later {
                    let parameter_symbol = self.get_symbol_of_declaration(parameter)?;
                    if self.types.get(ty)?.symbol == parameter_symbol {
                        self.error_at(Some(node), ts_diagnostics::Type_parameter_defaults_can_only_reference_previously_declared_type_parameters, vec![])?;
                    }
                }
            }
        }
        for child in self.source_children(node)? {
            self.check_default_references(child, later)?;
        }
        Ok(())
    }

    // port: tsc/internal/checker/checker.go:Checker.checkTypeParameter
    pub(crate) fn check_type_parameter(&mut self, node: NodeId) -> Result<(), Error> {
        let read = self.ast(node)?.node(node)?;
        if read.modifiers().is_some() {
            return Err(Error::Unsupported(
                "checkTypeParameter: variance/const modifiers",
            ));
        }
        let data = read
            .data_source()
            .as_type_parameter_declaration()
            .ok_or(Error::MissingLink("type parameter declaration"))?;
        let constraint = data.constraint();
        let default = data.default_type();
        if data.expression().is_some() {
            return Err(Error::Unsupported(
                "checkTypeParameter: recovered expression",
            ));
        }
        if let Some(constraint) = constraint {
            self.check_source_element(constraint)?;
        }
        if let Some(default) = default {
            self.check_source_element(default)?;
        }
        let symbol = self
            .get_symbol_of_declaration(node)?
            .ok_or(Error::MissingLink("type parameter symbol"))?;
        let ty = self.get_declared_type_of_type_parameter(symbol)?;
        self.base_constraint_of_type(ty)?;
        let default_type = self.resolved_type_parameter_default(ty)?;
        if default_type == self.builtins.circular_constraint_type {
            let name = self.type_to_string(ty, crate::type_format_flags::NONE)?;
            self.error_at(
                default,
                ts_diagnostics::Type_parameter_0_has_a_circular_default,
                vec![name],
            )?;
        }
        if let Some(constraint) = self.constraint_of_type_parameter(ty)? {
            if default_type != self.builtins.circular_constraint_type
                && default_type != self.builtins.no_constraint_type
            {
                let mapper = self.new_type_mapper(&[ty], &[default_type])?;
                let constraint = self.instantiate_type(constraint, Some(mapper))?;
                let constraint =
                    self.get_type_with_this_argument(constraint, default_type, false)?;
                self.check_constraint_assignable(default_type, constraint, default)?;
            }
        }
        let name = self
            .ast(node)?
            .node(node)?
            .name()
            .ok_or(Error::MissingLink("type parameter name"))?;
        let text = self.ast(name)?.node_text(name)?.into_js_string();
        if matches!(
            text.as_bytes(),
            b"any"
                | b"unknown"
                | b"never"
                | b"number"
                | b"bigint"
                | b"boolean"
                | b"string"
                | b"symbol"
                | b"void"
                | b"object"
                | b"undefined"
        ) {
            self.error_at(
                Some(name),
                ts_diagnostics::Type_parameter_name_cannot_be_0,
                vec![text],
            )?;
        }
        Ok(())
    }

    fn check_constraint_assignable(
        &mut self,
        source: TypeId,
        target: TypeId,
        node: Option<NodeId>,
    ) -> Result<bool, Error> {
        let (related, diagnostic) = self.check_type_related_ex(
            source,
            target,
            crate::RelationKind::Assignable,
            node,
            Some(ts_diagnostics::Type_0_does_not_satisfy_the_constraint_1),
        )?;
        if let Some(diagnostic) = diagnostic {
            self.add_diagnostic(diagnostic)?;
        }
        Ok(related)
    }

    // port: tsc/internal/checker/checker.go:Checker.checkTypeReferenceNode
    // port: tsc/internal/checker/checker.go:Checker.checkTypeArgumentConstraints
    pub(crate) fn check_type_reference_node(&mut self, node: NodeId) -> Result<(), Error> {
        let nodes = self.source_list(node, self.ast(node)?.node(node)?.type_argument_list())?;
        for &node in &nodes {
            self.check_source_element(node)?;
        }
        let ty = self.get_type_from_type_node(node)?;
        if ty == self.builtins.error_type || nodes.is_empty() {
            return Ok(());
        }
        let Some(symbol) = self.type_reference_symbol(node, true)? else {
            return Ok(());
        };
        let parameters = self.get_local_type_parameters(symbol)?;
        let mut arguments = None;
        let mut mapper = None;
        let mut valid = true;
        for (index, &parameter) in parameters.iter().enumerate() {
            if let Some(constraint) = self.constraint_of_type_parameter(parameter)? {
                if arguments.is_none() {
                    let effective = self.effective_type_arguments(node, &parameters)?;
                    mapper = Some(self.new_type_mapper(&parameters, &effective)?);
                    arguments = Some(effective);
                }
                if valid {
                    let source = arguments
                        .as_ref()
                        .ok_or(Error::MissingLink("effective arguments"))?[index];
                    let constraint = self.instantiate_type(constraint, mapper)?;
                    valid = self.check_constraint_assignable(
                        source,
                        constraint,
                        nodes.get(index).copied(),
                    )?;
                }
            }
        }
        Ok(())
    }
}
