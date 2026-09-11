//! Source checking over bound declarations. Each unsupported semantic branch
//! fails explicitly; a failed file check never becomes a successful cache hit.

use crate::{object_flags as of, type_flags as tf, type_format_flags, CheckerState, Error, TypeId};
use ts_arena::{NodeId, SymbolId};
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
    fn check_source_element(&mut self, node: NodeId) -> Result<(), Error> {
        stacker::maybe_grow(128 * 1024, 2 * 1024 * 1024, || {
            self.check_source_element_worker(node)
        })
    }

    fn check_source_element_worker(&mut self, node: NodeId) -> Result<(), Error> {
        let view = self.ast(node)?;
        let read = view.node(node)?;
        if read.flags() & nf::HAS_JS_DOC != 0 {
            return Err(Error::Unsupported("checkSourceElement: JSDoc"));
        }
        if read.modifiers().is_some() {
            return Err(Error::Unsupported("checkGrammarModifiers"));
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
                self.resolve_type_members(ty)
            }
            Some(K::ParenthesizedType) => {
                self.check_source_element(required(read.type_node(), "parenthesized type")?)
            }
            Some(
                K::UnionType
                | K::TypeReference
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
                | K::ObjectKeyword,
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

    // port: tsc/internal/checker/checker.go:Checker.checkTypeAliasDeclaration
    // port: tsc/internal/checker/checker.go:Checker.checkInterfaceDeclaration
    fn check_type_declaration(&mut self, node: NodeId) -> Result<(), Error> {
        let read = self.ast(node)?.node(node)?;
        if read.type_parameter_list().is_some() {
            return Err(Error::Unsupported("checkTypeParameters"));
        }
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
            self.resolve_type_members(ty)
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
            if self.symbol_declarations(symbol)?.len() != 1 {
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
        if property && initializer.is_some() {
            return Err(Error::Unsupported(
                "checkGrammarProperty: signature initializer",
            ));
        }
        if !property {
            let parent = required(read.parent(), "variable declaration parent")?;
            let flags = self.ast(parent)?.node(parent)?.flags();
            if flags & nf::CONSTANT != 0 && initializer.is_none() {
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

    // port: tsc/internal/checker/relater.go:Checker.isTypeRelatedTo
    // port: tsc/internal/checker/relater.go:Checker.isSimpleTypeRelatedTo
    fn source_type_assignable(
        &mut self,
        source: TypeId,
        target: TypeId,
        active: &mut Vec<(TypeId, TypeId)>,
    ) -> Result<bool, Error> {
        let source = self.get_regular_type_of_literal_type(source)?;
        let target = self.get_regular_type_of_literal_type(target)?;
        let s = self.types.flags(source)?;
        let t = self.types.flags(target)?;
        if source == target
            || t & tf::ANY != 0
            || s & tf::NEVER != 0
            || source == self.builtins.wildcard_type
            || t & tf::UNKNOWN != 0
        {
            return Ok(true);
        }
        if t & tf::NEVER != 0 {
            return Ok(false);
        }
        if s & tf::STRING_LIKE != 0 && t & tf::STRING != 0
            || s & tf::NUMBER_LIKE != 0 && t & tf::NUMBER != 0
            || s & tf::BIG_INT_LIKE != 0 && t & tf::BIG_INT != 0
            || s & tf::BOOLEAN_LIKE != 0 && t & tf::BOOLEAN != 0
            || s & tf::ES_SYMBOL_LIKE != 0 && t & tf::ES_SYMBOL != 0
        {
            return Ok(true);
        }
        if (s | t) & tf::ENUM_LIKE != 0 {
            return Err(Error::Unsupported("isSimpleTypeRelatedTo: enum relation"));
        }
        if s & tf::UNDEFINED != 0
            && (!self.options.strict_null_checks && t & tf::UNION_OR_INTERSECTION == 0
                || t & (tf::UNDEFINED | tf::VOID) != 0)
            || s & tf::NULL != 0
                && (!self.options.strict_null_checks && t & tf::UNION_OR_INTERSECTION == 0
                    || t & tf::NULL != 0)
            || s & tf::OBJECT != 0 && t & tf::NON_PRIMITIVE != 0
            || s & tf::ANY != 0
        {
            return Ok(true);
        }
        if !self.options.strict_null_checks
            && s & tf::NULLABLE != 0
            && target == self.builtins.boolean_type
        {
            // The native union relation accepts null/undefined into both
            // constituents of the canonical boolean type in non-strict mode.
            return Ok(true);
        }
        if (s | t) & tf::UNION != 0
            && self.is_primitive_union(source)?
            && self.is_primitive_union(target)?
        {
            if s & tf::UNION != 0 {
                let types = self.types.union(source)?.types.clone();
                for &ty in types.iter() {
                    if !self.source_type_assignable(ty, target, active)? {
                        return Ok(false);
                    }
                }
                return Ok(true);
            }
            let types = self.types.union(target)?.types.clone();
            for &ty in types.iter() {
                if self.source_type_assignable(source, ty, active)? {
                    return Ok(true);
                }
            }
            return Ok(false);
        }
        if s & tf::OBJECT != 0 && t & tf::OBJECT != 0 {
            if active.contains(&(source, target)) {
                return Err(Error::Unsupported("recursive structured type relation"));
            }
            active.push((source, target));
            let result = self.plain_properties_assignable(source, target, active);
            active.pop();
            return result;
        }
        // The canonical boolean union has the primitive boolean meaning. Other
        // unions and structured/instantiable types require the full relater.
        let complex = |ty, flags| {
            ty != self.builtins.boolean_type && flags & tf::STRUCTURED_OR_INSTANTIABLE != 0
        };
        if complex(source, s) || complex(target, t) {
            return Err(Error::Unsupported(
                "checkTypeRelatedTo: structured/instantiable relation",
            ));
        }
        Ok(false)
    }

    // The primitive slice can use all-source/any-target constituent relations
    // without structural matching, constraints or recursive assumptions.
    fn is_primitive_union(&self, ty: TypeId) -> Result<bool, Error> {
        let flags = self.types.flags(ty)?;
        if flags & tf::UNION != 0 {
            for &ty in self.types.union(ty)?.types.iter() {
                if !self.is_primitive_union(ty)? {
                    return Ok(false);
                }
            }
            return Ok(true);
        }
        Ok(flags & tf::STRUCTURED_OR_INSTANTIABLE == 0)
    }

    // port: tsc/internal/checker/relater.go:Relater.propertiesRelatedTo
    // port: tsc/internal/checker/relater.go:Relater.propertyRelatedTo
    // The supported branch has required public properties, no signatures or
    // indices, and no excess properties on fresh literals. All other structural
    // paths remain explicit boundaries, including richer mismatch diagnostics.
    fn plain_properties_assignable(
        &mut self,
        source: TypeId,
        target: TypeId,
        active: &mut Vec<(TypeId, TypeId)>,
    ) -> Result<bool, Error> {
        self.resolve_type_members(source)?;
        self.resolve_type_members(target)?;
        for ty in [source, target] {
            let record = self.types.get(ty)?;
            if record.object_flags & (of::ANONYMOUS | of::INTERFACE) == 0 {
                return Err(Error::Unsupported("structuredTypeRelatedTo: object family"));
            }
            let members = self.types.structured(ty)?;
            if members
                .signatures
                .as_ref()
                .is_some_and(|list| !list.is_empty())
                || members
                    .index_infos
                    .as_ref()
                    .is_some_and(|list| !list.is_empty())
            {
                return Err(Error::Unsupported(
                    "structuredTypeRelatedTo: signatures/index signatures",
                ));
            }
        }
        let sources = self
            .types
            .structured(source)?
            .properties
            .as_deref()
            .unwrap_or_default()
            .to_vec();
        let targets = self
            .types
            .structured(target)?
            .properties
            .as_deref()
            .unwrap_or_default()
            .to_vec();
        if self.types.get(source)?.object_flags & of::FRESH_LITERAL != 0 {
            for &property in &sources {
                let name = self.symbol(property)?.name_bytes();
                let table = self.types.structured(target)?.members;
                if !table
                    .map(|table| {
                        self.table(table)
                            .map(|table| table.get(name).flatten().is_some())
                    })
                    .transpose()?
                    .unwrap_or(false)
                {
                    return Err(Error::Unsupported("hasExcessProperties"));
                }
            }
        }
        for target_property in targets {
            self.require_plain_property(target_property)?;
            let name = self.symbol(target_property)?.name_to_owned();
            let table = self.types.structured(source)?.members;
            let source_property = table
                .map(|table| {
                    self.table(table)
                        .map(|table| table.get(name.as_bytes()).flatten())
                })
                .transpose()?
                .flatten()
                .ok_or(Error::Unsupported("reportUnmatchedProperty"))?;
            self.require_plain_property(source_property)?;
            let source_type = self.get_type_of_symbol(source_property)?;
            let target_type = self.get_type_of_symbol(target_property)?;
            if !self.source_type_assignable(source_type, target_type, active)? {
                return Err(Error::Unsupported(
                    "propertyRelatedTo: incompatible property diagnostic",
                ));
            }
        }
        Ok(true)
    }

    fn require_plain_property(&self, symbol: SymbolId) -> Result<(), Error> {
        let read = self.symbol(symbol)?;
        if read.flags() & sf::PROPERTY == 0 || read.flags() & sf::OPTIONAL != 0 {
            return Err(Error::Unsupported(
                "propertyRelatedTo: optional/accessor/method",
            ));
        }
        for node in self.symbol_declarations(symbol)?.iter().flatten() {
            let declaration = self.ast(node)?.node(node)?;
            if !matches!(
                declaration.kind().known(),
                Some(K::PropertySignature | K::PropertyAssignment)
            ) || declaration.modifiers().is_some()
            {
                return Err(Error::Unsupported(
                    "propertyRelatedTo: declaration accessibility",
                ));
            }
        }
        Ok(())
    }

    // port: tsc/internal/checker/relater.go:Checker.typeCouldHaveTopLevelSingletonTypes
    fn type_could_have_top_level_singletons(&self, ty: TypeId) -> Result<bool, Error> {
        let flags = self.types.flags(ty)?;
        if flags & tf::BOOLEAN != 0 {
            return Ok(false);
        }
        if flags & tf::UNION_OR_INTERSECTION != 0 {
            for &ty in self.types.types_of(ty)? {
                if self.type_could_have_top_level_singletons(ty)? {
                    return Ok(true);
                }
            }
            return Ok(false);
        }
        if flags & tf::INSTANTIABLE != 0 {
            return Err(Error::Unsupported(
                "typeCouldHaveTopLevelSingletonTypes: constraint",
            ));
        }
        Ok(flags & (tf::UNIT | tf::TEMPLATE_LITERAL | tf::STRING_MAPPING) != 0)
    }

    // port: tsc/internal/checker/relater.go:Checker.checkTypeAssignableTo
    // port: tsc/internal/checker/relater.go:Relater.reportRelationError
    fn check_assignable_at(
        &mut self,
        source: TypeId,
        target: TypeId,
        node: NodeId,
    ) -> Result<(), Error> {
        if self.source_type_assignable(source, target, &mut Vec::new())? {
            return Ok(());
        }
        let source_flags = self.types.flags(source)?;
        let target_flags = self.types.flags(target)?;
        let source_for_error = if target_flags & tf::NEVER == 0
            && !self.type_could_have_top_level_singletons(target)?
        {
            if source_flags & tf::STRING_LITERAL != 0 {
                self.builtins.string_type
            } else if source_flags & tf::NUMBER_LITERAL != 0 {
                self.builtins.number_type
            } else if source_flags & tf::BIG_INT_LITERAL != 0 {
                self.builtins.bigint_type
            } else if source_flags & tf::BOOLEAN_LITERAL != 0 {
                self.builtins.boolean_type
            } else {
                source
            }
        } else {
            source
        };
        let source_name = self.type_to_string(
            source_for_error,
            type_format_flags::USE_FULLY_QUALIFIED_TYPE,
        )?;
        let target_name =
            self.type_to_string(target, type_format_flags::USE_FULLY_QUALIFIED_TYPE)?;
        if source_name == target_name {
            return Err(Error::Unsupported(
                "reportRelationError: unrelated types with identical names",
            ));
        }
        self.error_at(
            Some(node),
            messages::Type_0_is_not_assignable_to_type_1,
            vec![source_name, target_name],
        )?;
        Ok(())
    }
}
