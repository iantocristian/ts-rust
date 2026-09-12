//! Source checking over bound declarations. Each unsupported semantic branch
//! fails explicitly; a failed file check never becomes a successful cache hit.

use crate::{type_flags as tf, CheckerState, Error, TypeId};
use ts_arena::NodeId;
use ts_ast::{node_flags as nf, symbol_flags as sf, SyntaxKind as K};
use ts_core::Tristate;
use ts_diagnostics as messages;
use ts_jsstring::JsString;

#[derive(Clone, Copy)]
pub(crate) enum SourceCheckStatus {
    Checking,
    Complete,
    Failed(Error),
}

fn required<T>(value: Option<T>, name: &'static str) -> Result<T, Error> {
    value.ok_or(Error::MissingLink(name))
}

impl CheckerState {
    // port: tsc/internal/checker/checker.go:Checker.checkSourceFile
    pub(crate) fn check_source_file(&mut self, source: NodeId) -> Result<(), Error> {
        match self.source_checks.get(&source).copied() {
            Some(SourceCheckStatus::Complete) => return Ok(()),
            Some(SourceCheckStatus::Failed(error)) => return Err(error),
            Some(SourceCheckStatus::Checking) => {
                return Err(Error::Unsupported("recursive checkSourceFile"))
            }
            None => {}
        }
        self.source_checks
            .insert(source, SourceCheckStatus::Checking);
        let result = self.check_source_file_worker(source);
        self.source_checks.insert(
            source,
            match result {
                Ok(()) => SourceCheckStatus::Complete,
                Err(error) => SourceCheckStatus::Failed(error),
            },
        );
        result
    }

    fn check_source_file_worker(&mut self, source: NodeId) -> Result<(), Error> {
        let view = self.ast(source)?;
        let file = view.source_file(source)?;
        if !file.diagnostics().is_empty() {
            return Err(Error::Unsupported(
                "checkSourceFile: recovered parse errors",
            ));
        }
        if file.is_declaration_file || view.node(source)?.flags() & nf::AMBIENT != 0 {
            return Err(Error::Unsupported(
                "checkGrammarSourceFile: ambient declarations",
            ));
        }
        if file.script_kind != ts_core::ScriptKind::TS
            || ts_ast::utilities::is_external_or_common_js_module(&file)
        {
            return Err(Error::Unsupported(
                "checkSourceFile: JavaScript or external module",
            ));
        }
        if !view.source_comments(file.comment_directives)?.is_empty()
            || !file.diagnostic_directives()?.is_empty()
        {
            return Err(Error::Unsupported("checkSourceFile: diagnostic directives"));
        }
        let options = self.program()?.host.options();
        if options.no_check == Tristate::TRUE
            || options.no_unused_locals == Tristate::TRUE
            || options.no_unused_parameters == Tristate::TRUE
            || options.isolated_declarations == Tristate::TRUE
        {
            return Err(Error::Unsupported(
                "checkSourceFile: noCheck/unused/isolated declaration options",
            ));
        }
        let statements: Vec<_> = view
            .node_slice(view.node(source)?.statements(view)?)?
            .iter()
            .collect();
        for statement in statements.into_iter().flatten() {
            self.check_source_element(statement)?;
        }
        Ok(())
    }

    // port: tsc/internal/checker/checker.go:Checker.checkSourceElementWorker
    pub(crate) fn check_source_element(&mut self, node: NodeId) -> Result<(), Error> {
        let previous = self.current_node.replace(node);
        self.instantiation.count = 0;
        let result = stacker::maybe_grow(128 * 1024, 2 * 1024 * 1024, || {
            self.check_source_element_worker(node)
        });
        self.current_node = previous;
        result
    }

    fn check_source_element_worker(&mut self, node: NodeId) -> Result<(), Error> {
        let view = self.ast(node)?;
        let read = view.node(node)?;
        if read.flags() & nf::HAS_JS_DOC != 0 {
            return Err(Error::Unsupported("checkSourceElement: JSDoc"));
        }
        if let Some(modifiers) = read.modifiers() {
            let nodes = self.source_list(node, Some(modifiers))?;
            let readonly = nodes.len() == 1
                && self.ast(nodes[0])?.node(nodes[0])?.kind() == K::ReadonlyKeyword
                && matches!(
                    read.kind().known(),
                    Some(K::PropertySignature | K::IndexSignature)
                );
            let declare = nodes.len() == 1
                && self.ast(nodes[0])?.node(nodes[0])?.kind() == K::DeclareKeyword
                && read.kind() == K::VariableStatement
                && match read.parent() {
                    Some(parent) => self.ast(parent)?.node(parent)?.kind() == K::SourceFile,
                    None => false,
                };
            if !readonly && !declare {
                return Err(Error::Unsupported("checkGrammarModifiers"));
            }
        }
        match read.kind().known() {
            Some(K::VariableStatement) => {
                let list = required(
                    read.data_source()
                        .as_variable_statement()
                        .ok_or(ts_arena::Error::InvalidGraph)?
                        .declaration_list(),
                    "variable declaration list",
                )?;
                self.check_source_element(list)
            }
            Some(K::VariableDeclarationList) => {
                if read.flags() & nf::USING != 0 {
                    return Err(Error::Unsupported(
                        "checkGrammarVariableDeclarationList: using",
                    ));
                }
                let list = required(
                    read.data_source()
                        .as_variable_declaration_list()
                        .ok_or(ts_arena::Error::InvalidGraph)?
                        .declarations(),
                    "variable declarations",
                )?;
                if view.list_has_trailing_comma(list)? {
                    return Err(Error::Unsupported(
                        "checkGrammarVariableDeclarationList: trailing comma",
                    ));
                }
                let declarations: Vec<_> =
                    view.node_slice(view.list(list)?.nodes())?.iter().collect();
                if declarations.is_empty() {
                    return Err(Error::Unsupported(
                        "checkGrammarVariableDeclarationList: empty",
                    ));
                }
                for declaration in declarations.into_iter().flatten() {
                    self.check_source_element(declaration)?;
                }
                Ok(())
            }
            Some(K::VariableDeclaration | K::PropertySignature) => self.check_variable_like(node),
            Some(K::TypeAliasDeclaration | K::InterfaceDeclaration) => {
                self.check_type_declaration(node)
            }
            Some(K::TypeLiteral) => {
                self.check_object_type_members(node)?;
                let ty = self.get_type_from_type_node(node)?;
                self.resolve_type_members(ty)?;
                self.check_source_index_constraints(ty, node)
            }
            Some(K::ParenthesizedType) => {
                self.check_source_element(required(read.type_node(), "parenthesized type")?)
            }
            Some(K::UnionType | K::IntersectionType) => self.check_union_or_intersection_type(node),
            Some(K::TypeReference | K::ExpressionWithTypeArguments) => {
                self.check_type_reference_node(node)
            }
            Some(K::TypeParameter) => self.check_type_parameter(node),
            Some(K::TypePredicate) => self.check_type_predicate(node),
            Some(K::ConditionalType) => {
                for child in self.source_children(node)? {
                    self.check_source_element(child)?;
                }
                Ok(())
            }
            Some(K::InferType) => self.check_infer_type(node),
            Some(K::MappedType) => self.check_mapped_type(node),
            Some(K::TemplateLiteralType) => self.check_template_type(node),
            Some(K::IndexedAccessType) => self.check_indexed_access_type(node),
            Some(
                K::ArrayType
                | K::TupleType
                | K::OptionalType
                | K::RestType
                | K::NamedTupleMember
                | K::TypeOperator,
            ) => self.check_array_tuple_syntax(node),
            Some(
                K::FunctionType
                | K::ConstructorType
                | K::CallSignature
                | K::ConstructSignature
                | K::MethodSignature
                | K::IndexSignature,
            ) => self.check_signature_syntax(node),
            Some(
                K::ThisType
                | K::LiteralType
                | K::AnyKeyword
                | K::UnknownKeyword
                | K::StringKeyword
                | K::NumberKeyword
                | K::BigIntKeyword
                | K::BooleanKeyword
                | K::SymbolKeyword
                | K::VoidKeyword
                | K::UndefinedKeyword
                | K::NullKeyword
                | K::NeverKeyword
                | K::ObjectKeyword
                | K::IntrinsicKeyword,
            ) => {
                self.get_type_from_type_node(node)?;
                Ok(())
            }
            Some(K::ExpressionStatement) => self
                .check_assignment_expression(required(read.expression(), "expression statement")?),
            Some(K::EmptyStatement) => Ok(()),
            _ => Err(Error::Unsupported(
                "checkSourceElementWorker: statement/type family",
            )),
        }
    }

    // port: tsc/internal/checker/checker.go:Checker.checkUnionOrIntersectionType
    fn check_union_or_intersection_type(&mut self, node: NodeId) -> Result<(), Error> {
        let view = self.ast(node)?;
        let read = view.node(node)?;
        let list = required(
            if read.kind() == K::UnionType {
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
        let constituents: Vec<_> = view.node_slice(view.list(list)?.nodes())?.iter().collect();
        // Check the source children before construction can reduce or reorder
        // the compound type. A cached type is not a completed declaration check.
        for constituent in constituents {
            self.check_source_element(required(constituent, "compound constituent")?)?;
        }
        self.get_type_from_type_node(node)?;
        Ok(())
    }

    // port: tsc/internal/checker/checker.go:Checker.checkTypeAliasDeclaration
    // port: tsc/internal/checker/checker.go:Checker.checkInterfaceDeclaration
    fn check_type_declaration(&mut self, node: NodeId) -> Result<(), Error> {
        self.check_type_parameters(node)?;
        let read = self.ast(node)?.node(node)?;
        let interface = read.kind() == K::InterfaceDeclaration;
        let name = required(read.name(), "type declaration name")?;
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
                if interface {
                    messages::Interface_name_cannot_be_0
                } else {
                    messages::Type_alias_name_cannot_be_0
                },
                vec![text],
            )?;
        }
        let symbol = required(
            self.get_symbol_of_declaration(node)?,
            "type declaration symbol",
        )?;
        if self.symbol(symbol)?.export_symbol().is_some() {
            return Err(Error::Unsupported("checkExportsOnMergedDeclarations"));
        }
        if interface {
            let ty = self.get_declared_type_of_symbol(symbol)?;
            self.check_object_type_members(node)?;
            self.resolve_type_members(ty)?;
            self.check_source_index_constraints(ty, node)
        } else {
            let annotation = required(
                self.ast(node)?.node(node)?.type_node(),
                "type alias annotation",
            )?;
            self.check_source_element(annotation)
        }
    }

    // port: tsc/internal/checker/checker.go:Checker.checkTypeLiteral
    fn check_object_type_members(&mut self, node: NodeId) -> Result<(), Error> {
        let view = self.ast(node)?;
        let members: Vec<_> = view
            .node_slice(view.node(node)?.members(view)?)?
            .iter()
            .collect();
        for member in members.into_iter().flatten() {
            let symbol = required(
                self.get_symbol_of_declaration(member)?,
                "type member symbol",
            )?;
            if self.symbol_declarations(symbol)?.len() != 1
                && !matches!(
                    self.ast(member)?.node(member)?.kind().known(),
                    Some(
                        K::CallSignature
                            | K::ConstructSignature
                            | K::MethodSignature
                            | K::IndexSignature
                    )
                )
            {
                return Err(Error::Unsupported(
                    "checkObjectTypeForDuplicateDeclarations/subsequent property declarations",
                ));
            }
            self.check_source_element(member)?;
        }
        Ok(())
    }

    // port: tsc/internal/checker/checker.go:Checker.checkVariableLikeDeclaration
    fn check_variable_like(&mut self, node: NodeId) -> Result<(), Error> {
        let read = self.ast(node)?.node(node)?;
        let name = required(read.name(), "variable/property name")?;
        let name_kind = self.ast(name)?.node(name)?.kind();
        let property = read.kind() == K::PropertySignature;
        if !property && name_kind != K::Identifier
            || property
                && !ts_ast::utilities::is_property_name_literal(&self.ast(name)?.node(name)?)
        {
            return Err(Error::Unsupported(
                "checkVariableLikeDeclaration: binding/computed/private name",
            ));
        }
        if !property && read.question_token(self.ast(node)?)?.is_some() {
            return Err(Error::Unsupported(
                "checkVariableLikeDeclaration: optional declaration",
            ));
        }
        if !property
            && read
                .data_source()
                .as_variable_declaration()
                .ok_or(ts_arena::Error::InvalidGraph)?
                .exclamation_token()
                .is_some()
        {
            return Err(Error::Unsupported(
                "checkGrammarVariableDeclaration: definite assignment assertion",
            ));
        }
        let initializer = read.initializer();
        let annotation = read.type_node();
        let ambient = read.flags() & nf::AMBIENT != 0;
        if ambient && initializer.is_some() {
            return Err(Error::Unsupported(
                "checkGrammarInitializer: ambient declaration initializer",
            ));
        }
        if property && initializer.is_some() {
            return Err(Error::Unsupported(
                "checkGrammarProperty: signature initializer",
            ));
        }
        if !property {
            let parent = required(read.parent(), "variable declaration parent")?;
            let flags = self.ast(parent)?.node(parent)?.flags();
            if flags & nf::CONSTANT != 0 && initializer.is_none() && !ambient {
                self.error_at(
                    Some(node),
                    messages::X_0_declarations_must_be_initialized,
                    vec![JsString::from_bytes(b"const".as_slice())],
                )?;
            } else if flags & nf::BLOCK_SCOPED != 0
                && self.ast(name)?.node_text(name)?.as_bytes() == b"let"
            {
                self.error_at(Some(name), messages::X_let_is_not_allowed_to_be_used_as_a_name_in_let_or_const_declarations, vec![])?;
            }
        }
        if let Some(annotation) = annotation {
            self.check_source_element(annotation)?;
        }
        let symbol = required(
            self.get_symbol_of_declaration(node)?,
            "variable/property symbol",
        )?;
        if self.symbol(symbol)?.value_declaration() != Some(node) {
            return Err(Error::Unsupported(
                "checkVariableLikeDeclaration: subsequent declaration",
            ));
        }
        let target = self.get_type_of_symbol(symbol)?;
        if let Some(initializer) = initializer {
            let source = self.check_expression_cached(initializer)?;
            self.check_assignable_at(source, target, node)?;
        }
        Ok(())
    }

    // port: tsc/internal/checker/checker.go:Checker.checkAssignmentOperator
    fn check_assignment_expression(&mut self, expression: NodeId) -> Result<(), Error> {
        let read = self.ast(expression)?.node(expression)?;
        if read.kind() == K::ParenthesizedExpression {
            return self.check_assignment_expression(required(
                read.expression(),
                "parenthesized assignment",
            )?);
        }
        if read.kind() != K::BinaryExpression {
            self.check_expression(expression)?;
            return Ok(());
        }
        let data = read
            .data_source()
            .as_binary_expression()
            .ok_or(ts_arena::Error::InvalidGraph)?;
        let operator = required(data.operator_token(), "assignment operator")?;
        if self.ast(operator)?.node(operator)?.kind() != K::EqualsToken {
            return Err(Error::Unsupported(
                "checkAssignmentOperator: compound/non-assignment",
            ));
        }
        let mut left = required(data.left(), "assignment left")?;
        let right = required(data.right(), "assignment right")?;
        while self.ast(left)?.node(left)?.kind() == K::ParenthesizedExpression {
            left = required(
                self.ast(left)?.node(left)?.expression(),
                "parenthesized reference",
            )?;
        }
        if self.ast(left)?.node(left)?.kind() != K::Identifier {
            return Err(Error::Unsupported(
                "checkReferenceExpression: non-identifier assignment",
            ));
        }
        let text = self.ast(left)?.node_text(left)?.into_js_string();
        let symbol = self.resolve_name(
            Some(left),
            text.as_bytes(),
            sf::VALUE,
            Some(messages::Cannot_find_name_0),
            true,
        )?;
        let right_type = self.check_expression(right)?;
        let Some(symbol) = symbol else { return Ok(()) };
        if self.symbol(symbol)?.flags() & sf::VARIABLE == 0 {
            return Err(Error::Unsupported(
                "checkAssignmentOperator: non-variable symbol",
            ));
        }
        let declaration = required(
            self.symbol(symbol)?.value_declaration(),
            "assignment value declaration",
        )?;
        let parent = required(
            self.ast(declaration)?.node(declaration)?.parent(),
            "assignment declaration parent",
        )?;
        if self.ast(parent)?.node(parent)?.flags() & nf::CONSTANT != 0 {
            self.error_at(
                Some(left),
                messages::Cannot_assign_to_0_because_it_is_a_constant,
                vec![text],
            )?;
            // checkIdentifier returns errorType for a readonly assignment.
            // The right side has already been checked; do not also relate it
            // to the constant's literal type.
            return Ok(());
        }
        let left_type = self.get_type_of_symbol(symbol)?;
        self.check_assignable_at(right_type, left_type, left)
    }

    // The previous slice used a separate limited relation. Every caller now
    // enters the production assignability cache; recursive work stays in the
    // relater that owns its assumption stack.
    pub(crate) fn source_type_assignable(
        &mut self,
        source: TypeId,
        target: TypeId,
        _active: &mut Vec<(TypeId, TypeId)>,
    ) -> Result<bool, Error> {
        self.is_type_related_to(source, target, crate::RelationKind::Assignable)
    }

    // port: tsc/internal/checker/relater.go:Checker.typeCouldHaveTopLevelSingletonTypes
    pub(crate) fn type_could_have_top_level_singletons(
        &mut self,
        ty: TypeId,
    ) -> Result<bool, Error> {
        let flags = self.types.flags(ty)?;
        if flags & tf::BOOLEAN != 0 {
            return Ok(false);
        }
        if flags & tf::UNION_OR_INTERSECTION != 0 {
            for &ty in self.types.compound_types(ty)?.clone().iter() {
                if self.type_could_have_top_level_singletons(ty)? {
                    return Ok(true);
                }
            }
            return Ok(false);
        }
        if flags & tf::INSTANTIABLE != 0 {
            if let Some(constraint) = self.constraint_of_type(ty)? {
                if constraint != ty {
                    return self.type_could_have_top_level_singletons(constraint);
                }
            }
        }
        Ok(flags & (tf::UNIT | tf::TEMPLATE_LITERAL | tf::STRING_MAPPING) != 0)
    }

    // port: tsc/internal/checker/relater.go:Checker.checkTypeAssignableTo
    // port: tsc/internal/checker/relater.go:Relater.reportRelationError
    pub(crate) fn check_assignable_at(
        &mut self,
        source: TypeId,
        target: TypeId,
        node: NodeId,
    ) -> Result<(), Error> {
        let (_, diagnostic) = self.check_type_related_ex(
            source,
            target,
            crate::RelationKind::Assignable,
            Some(node),
            None,
        )?;
        if let Some(diagnostic) = diagnostic {
            self.add_diagnostic(diagnostic)?;
        }
        Ok(())
    }
}
