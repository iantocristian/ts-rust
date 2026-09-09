use crate::{ast as a, checked, need, Binder};
use std::sync::Arc;
use ts_ast::{
    internal_symbol_names as names, modifier_flags as mf, symbol_flags as sf, JsString, NodeId,
    Symbol, SymbolId, SymbolTable, SymbolTableId, SyntaxKind as K,
};
use ts_diagnostics as d;

impl Binder<'_, '_, '_> {
    // port: tsc/internal/binder/binder.go:Binder.newSymbol
    pub fn new_symbol(&mut self, flags: u32, name: JsString) -> SymbolId {
        self.symbol_count = self.symbol_count.wrapping_add(1);
        self.builder.symbols_mut().push(Symbol::new(flags, name))
    }
    pub fn s(&self, id: SymbolId) -> a::SymbolRead<'_> {
        checked(self.builder.symbols().get(id))
    }
    pub fn symbol_flags_mut(&mut self, id: SymbolId) -> &mut u32 {
        checked(self.builder.symbols_mut().flags_mut(id))
    }
    pub fn set_symbol_declarations(&mut self, id: SymbolId, value: a::DeclarationSlice) {
        checked(self.builder.symbols_mut().set_declarations(id, value));
    }
    pub fn set_symbol_value_declaration(&mut self, id: SymbolId, value: Option<NodeId>) {
        checked(self.builder.symbols_mut().set_value_declaration(id, value));
    }
    pub fn set_symbol_members(&mut self, id: SymbolId, value: Option<SymbolTableId>) {
        checked(self.builder.symbols_mut().set_members(id, value));
    }
    pub fn set_symbol_exports(&mut self, id: SymbolId, value: Option<SymbolTableId>) {
        checked(self.builder.symbols_mut().set_exports(id, value));
    }
    pub fn set_symbol_parent(&mut self, id: SymbolId, value: Option<SymbolId>) {
        checked(self.builder.symbols_mut().set_parent(id, value));
    }
    pub fn set_symbol_export_symbol(&mut self, id: SymbolId, value: Option<SymbolId>) {
        checked(self.builder.symbols_mut().set_export_symbol(id, value));
    }
    pub fn ensure_exports(&mut self, symbol: SymbolId) -> SymbolTableId {
        if let Some(table) = self.s(symbol).exports() {
            return table;
        }
        let table = self.builder.alloc_table(SymbolTable::new());
        self.set_symbol_exports(symbol, Some(table));
        table
    }
    pub fn ensure_members(&mut self, symbol: SymbolId) -> SymbolTableId {
        if let Some(table) = self.s(symbol).members() {
            return table;
        }
        let table = self.builder.alloc_table(SymbolTable::new());
        self.set_symbol_members(symbol, Some(table));
        table
    }
    // port: tsc/internal/binder/binder.go:Binder.declareSymbol
    pub fn declare_symbol(
        &mut self,
        table: SymbolTableId,
        parent: Option<SymbolId>,
        node: NodeId,
        includes: u32,
        excludes: u32,
    ) -> SymbolId {
        self.declare_symbol_ex(table, parent, node, includes, excludes, false, false)
    }
    // port: tsc/internal/binder/binder.go:Binder.declareSymbolEx
    #[allow(
        clippy::too_many_arguments,
        reason = "preserves the pinned declaration conflict operation and independent computed/replaceable flags"
    )]
    pub fn declare_symbol_ex(
        &mut self,
        table: SymbolTableId,
        parent: Option<SymbolId>,
        node: NodeId,
        includes: u32,
        excludes: u32,
        replaceable: bool,
        computed: bool,
    ) -> SymbolId {
        assert!(computed || !checked(a::has_dynamic_name(self.view(), Some(node))));
        let default_export = checked(a::has_syntactic_modifier(self.view(), node, mf::DEFAULT))
            || self.n(node).kind() == K::ExportSpecifier
                && checked(a::module_export_name_is_default(
                    self.view(),
                    need(self.n(node).name()),
                ));
        let name = if computed {
            JsString::from_bytes(names::COMPUTED)
        } else if default_export && parent.is_some() {
            JsString::from_bytes(names::DEFAULT)
        } else {
            self.get_declaration_name(node)
        };
        let mut symbol;
        if name.as_bytes() == names::MISSING {
            symbol = self.new_symbol(sf::NONE, JsString::from_bytes(names::MISSING));
        } else if let Some(existing) = self.table(table).get(name.as_bytes()).flatten() {
            symbol = existing;
            let flags = self.s(symbol).flags();
            if replaceable && flags & sf::REPLACEABLE_BY_METHOD == 0 {
                return symbol;
            }
            if flags & excludes != 0 {
                if flags & sf::REPLACEABLE_BY_METHOD != 0 {
                    symbol = self.new_symbol(sf::NONE, name.clone());
                    self.table_mut(table).insert(name, Some(symbol));
                } else if !(includes & sf::VARIABLE != 0 && flags & sf::ASSIGNMENT != 0
                    || includes & sf::ASSIGNMENT != 0 && flags & sf::VARIABLE != 0)
                {
                    let mut message = if flags & sf::BLOCK_SCOPED_VARIABLE != 0 {
                        d::Cannot_redeclare_block_scoped_variable_0
                    } else {
                        d::Duplicate_identifier_0
                    };
                    let mut needs_name = true;
                    if (flags | includes) & sf::ENUM != 0 {
                        message = d::Enum_declarations_can_only_merge_with_namespace_or_other_enum_declarations;
                        needs_name = false;
                    }
                    let declarations = checked(
                        self.builder
                            .declarations()
                            .get(self.s(symbol).declarations()),
                    )
                    .to_vec();
                    let multiple_defaults = !declarations.is_empty()
                        && (default_export
                            || self.n(node).kind() == K::ExportAssignment
                                && !self
                                    .n(node)
                                    .data_source()
                                    .as_export_assignment()
                                    .expect("export assignment payload")
                                    .is_export_equals());
                    if multiple_defaults {
                        message = d::A_module_cannot_have_multiple_default_exports;
                        needs_name = false;
                    }
                    let declaration_name =
                        checked(a::get_name_of_declaration(self.view(), Some(node)))
                            .unwrap_or(node);
                    let args = if needs_name {
                        vec![self.get_display_name(node)]
                    } else {
                        Vec::new()
                    };
                    let mut diagnostic =
                        self.create_diagnostic_for_node(declaration_name, message, args);
                    if self.n(node).kind() == K::TypeAliasDeclaration
                        && self
                            .n(node)
                            .type_node()
                            .map(|id| self.n(id))
                            .is_none_or(|node| a::node_is_missing(Some(&node)))
                        && checked(a::has_syntactic_modifier(self.view(), node, mf::EXPORT))
                        && flags & (sf::ALIAS | sf::TYPE | sf::NAMESPACE) != 0
                    {
                        let text = self.text(need(self.n(node).name()));
                        let suggestion = JsString::from_bytes(
                            [b"export type { ".as_slice(), text.as_bytes(), b" }"].concat(),
                        );
                        diagnostic.related_information.push(Arc::new(
                            self.create_diagnostic_for_node(
                                node,
                                d::Did_you_mean_0,
                                vec![suggestion],
                            ),
                        ));
                    }
                    for (index, declaration) in declarations.into_iter().enumerate() {
                        let declaration = need(declaration);
                        let name_node =
                            checked(a::get_name_of_declaration(self.view(), Some(declaration)))
                                .unwrap_or(declaration);
                        let args = if needs_name {
                            vec![self.get_display_name(declaration)]
                        } else {
                            Vec::new()
                        };
                        let mut previous =
                            self.create_diagnostic_for_node(name_node, message, args);
                        if multiple_defaults {
                            previous.related_information.push(Arc::new(
                                self.create_diagnostic_for_node(
                                    declaration_name,
                                    if index == 0 {
                                        d::Another_export_default_is_here
                                    } else {
                                        d::X_and_here
                                    },
                                    Vec::new(),
                                ),
                            ));
                        }
                        self.add_diagnostic(previous);
                        if multiple_defaults {
                            diagnostic.related_information.push(Arc::new(
                                self.create_diagnostic_for_node(
                                    name_node,
                                    d::The_first_export_default_is_here,
                                    Vec::new(),
                                ),
                            ));
                        }
                    }
                    self.add_diagnostic(diagnostic);
                    if flags & sf::ACCESSOR != 0 && flags & sf::ACCESSOR != includes & sf::ACCESSOR
                    {
                        *self.symbol_flags_mut(symbol) |= sf::ACCESSOR;
                    }
                    symbol = self.new_symbol(sf::NONE, name);
                }
            }
        } else {
            symbol = self.new_symbol(sf::NONE, name.clone());
            self.table_mut(table).insert(name, Some(symbol));
            if replaceable {
                *self.symbol_flags_mut(symbol) |= sf::REPLACEABLE_BY_METHOD;
            }
        }
        self.add_declaration_to_symbol(symbol, node, includes);
        if self.s(symbol).parent().is_none() {
            self.set_symbol_parent(symbol, parent);
        } else {
            assert_eq!(
                self.s(symbol).parent(),
                parent,
                "Existing symbol parent should match new one"
            );
        }
        symbol
    }
    // port: tsc/internal/binder/binder.go:Binder.getDeclarationName
    pub fn get_declaration_name(&self, node: NodeId) -> JsString {
        if self.n(node).kind() == K::ExportAssignment {
            return JsString::from_bytes(
                if self
                    .n(node)
                    .data_source()
                    .as_export_assignment()
                    .expect("export assignment payload")
                    .is_export_equals()
                {
                    names::EXPORT_EQUALS
                } else {
                    names::DEFAULT
                },
            );
        }
        if let Some(name) = checked(a::get_name_of_declaration(self.view(), Some(node))) {
            if checked(a::is_ambient_module(self.view(), node)) {
                let module_name = self.text(name);
                if a::is_global_scope_augmentation(&self.n(node)) {
                    return JsString::from_bytes(names::GLOBAL);
                }
                let pattern = ts_core::pattern::Pattern::parse(module_name.as_bytes());
                if pattern.is_valid() && pattern.star_index >= 0 {
                    if let Some(attributes) = self
                        .n(node)
                        .data_source()
                        .as_module_declaration()
                        .expect("module payload")
                        .attributes()
                    {
                        let id = a::runtime_node_id(&self.n(attributes));
                        return JsString::from_bytes(
                            [
                                b"\xfe\"".as_slice(),
                                module_name.as_bytes(),
                                b"\"pattern@",
                                id.to_string().as_bytes(),
                            ]
                            .concat(),
                        );
                    }
                }
                return JsString::from_bytes(
                    [b"\"".as_slice(), module_name.as_bytes(), b"\""].concat(),
                );
            }
            if self.n(name).kind() == K::PrivateIdentifier {
                let Some(class) = checked(a::get_containing_class(self.view(), node)) else {
                    return JsString::from_bytes(names::MISSING);
                };
                return get_symbol_name_for_private_identifier(
                    &self.s(need(self.symbol(class))),
                    self.text(name).as_bytes(),
                );
            }
            if a::is_property_name_literal(&self.n(name))
                || self.n(name).kind() == K::JsxNamespacedName
            {
                return self.text(name);
            }
            if self.n(name).kind() == K::ComputedPropertyName {
                let expression = need(self.n(name).expression());
                if a::is_string_or_numeric_literal_like(&self.n(expression)) {
                    return self.text(expression);
                }
                if a::is_signed_numeric_literal(self.view(), expression)
                    .expect("computed literal graph")
                {
                    let unary = self
                        .n(expression)
                        .data_source()
                        .as_prefix_unary_expression()
                        .expect("prefix payload")
                        .to_owned();
                    let token =
                        ts_scanner::token_to_string(unary.operator.known().unwrap_or(K::Unknown));
                    return JsString::from_bytes(
                        [token.as_bytes(), self.text(need(unary.operand)).as_bytes()].concat(),
                    );
                }
                panic!("Only computed properties with literal names have declaration names");
            }
            return JsString::from_bytes(names::MISSING);
        }
        JsString::from_bytes(match self.n(node).kind().known() {
            Some(K::Constructor) => names::CONSTRUCTOR,
            Some(K::FunctionType | K::CallSignature) => names::CALL,
            Some(K::ConstructorType | K::ConstructSignature) => names::NEW,
            Some(K::IndexSignature) => names::INDEX,
            Some(K::ExportDeclaration) => names::EXPORT_STAR,
            Some(K::SourceFile | K::BinaryExpression) => names::EXPORT_EQUALS,
            _ => names::MISSING,
        })
    }
    // port: tsc/internal/binder/binder.go:Binder.getDisplayName
    pub fn get_display_name(&self, node: NodeId) -> JsString {
        if let Some(name) = self.n(node).name() {
            return checked(ts_scanner::declaration_name_to_string(
                self.view(),
                Some(name),
            ));
        }
        let name = self.get_declaration_name(node);
        if name.as_bytes() == names::MISSING {
            JsString::from_bytes(&b"(Missing)"[..])
        } else {
            name
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.addDeclarationToSymbol
    pub fn add_declaration_to_symbol(&mut self, symbol: SymbolId, node: NodeId, flags: u32) {
        *self.symbol_flags_mut(symbol) |= flags;
        self.set_node_symbol(node, Some(symbol));
        let declarations = self.s(symbol).declarations();
        let declarations = if declarations.is_nil() {
            self.new_single_declaration(Some(node))
        } else {
            checked(
                self.builder
                    .declarations_mut()
                    .append_if_unique(declarations, Some(node)),
            )
        };
        self.set_symbol_declarations(symbol, declarations);
        let existing = self.s(symbol).flags();
        if existing & sf::CONST_ENUM_ONLY_MODULE != 0
            && existing & (sf::FUNCTION | sf::CLASS | sf::REGULAR_ENUM) != 0
        {
            *self.symbol_flags_mut(symbol) &= !sf::CONST_ENUM_ONLY_MODULE;
            self.not_const_enum_only_modules.insert(symbol);
        }
        if flags & sf::VALUE != 0 {
            self.set_value_declaration(symbol, node);
        }
    }
    // port: tsc/internal/binder/binder.go:Binder.newSingleDeclaration
    pub fn new_single_declaration(&mut self, node: Option<NodeId>) -> a::DeclarationSlice {
        checked(self.builder.declarations_mut().alloc_one(node))
    }
    // port: tsc/internal/binder/binder.go:SetValueDeclaration
    pub fn set_value_declaration(&mut self, symbol: SymbolId, node: NodeId) {
        let previous = self.s(symbol).value_declaration();
        if previous.is_none_or(|previous| {
            is_assignment_declaration(&self.n(previous))
                && !is_assignment_declaration(&self.n(node))
                || self.n(previous).kind() != self.n(node).kind()
                    && is_effective_module_declaration(&self.n(previous))
        }) {
            self.set_symbol_value_declaration(symbol, Some(node));
        }
    }
}
// port: tsc/internal/binder/binder.go:GetSymbolNameForPrivateIdentifier
pub fn get_symbol_name_for_private_identifier(
    symbol: &(impl a::SymbolAccess + ?Sized),
    description: &[u8],
) -> JsString {
    let id = a::runtime_symbol_id(symbol) as isize;
    JsString::from_bytes(
        [
            b"\xfe#".as_slice(),
            id.to_string().as_bytes(),
            b"@",
            description,
        ]
        .concat(),
    )
}
// port: tsc/internal/binder/binder.go:isAssignmentDeclaration
pub(crate) fn is_assignment_declaration(node: &(impl a::NodeAccess + ?Sized)) -> bool {
    matches!(
        node.kind().known(),
        Some(
            K::BinaryExpression
                | K::PropertyAccessExpression
                | K::ElementAccessExpression
                | K::Identifier
                | K::CallExpression
        )
    )
}
// port: tsc/internal/binder/binder.go:isEffectiveModuleDeclaration
pub(crate) fn is_effective_module_declaration(node: &(impl a::NodeAccess + ?Sized)) -> bool {
    matches!(
        node.kind().known(),
        Some(K::ModuleDeclaration | K::Identifier)
    )
}

impl Binder<'_, '_, '_> {
    // port: tsc/internal/binder/binder.go:Binder.declareModuleMember
    pub fn declare_module_member(&mut self, node: NodeId, flags: u32, excludes: u32) -> SymbolId {
        let container = need(self.container);
        let exported = checked(a::get_combined_modifier_flags(self.view(), node)) & mf::EXPORT != 0
            || checked(a::is_implicitly_exported_js_doc_declaration(
                self.view(),
                node,
            ));
        if flags & sf::ALIAS != 0 {
            if self.n(node).kind() == K::ExportSpecifier
                || self.n(node).kind() == K::ImportEqualsDeclaration && exported
            {
                let parent = need(self.symbol(container));
                let table = self.ensure_exports(parent);
                return self.declare_symbol(table, Some(parent), node, flags, excludes);
            }
            let table = self.ensure_locals(container);
            return self.declare_symbol(table, None, node, flags, excludes);
        }
        if !checked(a::is_ambient_module(self.view(), node))
            && (exported || self.n(container).flags() & a::node_flags::EXPORT_CONTEXT != 0)
        {
            let parent = need(self.symbol(container));
            let exports = self.ensure_exports(parent);
            if !a::is_locals_container(&self.n(container))
                || checked(a::has_syntactic_modifier(self.view(), node, mf::DEFAULT))
                    && self.get_declaration_name(node).as_bytes() == names::MISSING
            {
                return self.declare_symbol(exports, Some(parent), node, flags, excludes);
            }
            let export_kind = if flags & sf::VALUE != 0 {
                sf::EXPORT_VALUE
            } else {
                sf::NONE
            };
            let table = self.ensure_locals(container);
            let local = self.declare_symbol(table, None, node, export_kind, excludes);
            let exported = self.declare_symbol(exports, Some(parent), node, flags, excludes);
            self.set_symbol_export_symbol(local, Some(exported));
            self.set_node_local_symbol(node, Some(local));
            return local;
        }
        let table = self.ensure_locals(container);
        self.declare_symbol(table, None, node, flags, excludes)
    }
    // port: tsc/internal/binder/binder.go:Binder.declareClassMember
    pub fn declare_class_member(&mut self, node: NodeId, flags: u32, excludes: u32) -> SymbolId {
        let parent = need(self.symbol(need(self.container)));
        let table = if checked(a::is_static(self.view(), node)) {
            self.ensure_exports(parent)
        } else {
            self.ensure_members(parent)
        };
        self.declare_symbol(table, Some(parent), node, flags, excludes)
    }
    // port: tsc/internal/binder/binder.go:Binder.declareSourceFileMember
    pub fn declare_source_file_member(
        &mut self,
        node: NodeId,
        flags: u32,
        excludes: u32,
    ) -> SymbolId {
        if checked(self.view().source_file(self.file))
            .external_module_indicator
            .is_some()
        {
            return self.declare_module_member(node, flags, excludes);
        }
        let table = self.ensure_locals(self.file);
        self.declare_symbol(table, None, node, flags, excludes)
    }
    // port: tsc/internal/binder/binder.go:Binder.declareSymbolAndAddToSymbolTable
    pub fn declare_symbol_and_add_to_symbol_table(
        &mut self,
        node: NodeId,
        flags: u32,
        excludes: u32,
    ) -> SymbolId {
        let container = need(self.container);
        match self.n(container).kind().known() {
            Some(K::ModuleDeclaration) => self.declare_module_member(node, flags, excludes),
            Some(K::SourceFile) => self.declare_source_file_member(node, flags, excludes),
            Some(K::ClassExpression | K::ClassDeclaration) => {
                self.declare_class_member(node, flags, excludes)
            }
            Some(K::EnumDeclaration) => {
                let parent = need(self.symbol(container));
                let table = self.ensure_exports(parent);
                self.declare_symbol(table, Some(parent), node, flags, excludes)
            }
            Some(
                K::TypeLiteral
                | K::ObjectLiteralExpression
                | K::InterfaceDeclaration
                | K::JsxAttributes,
            ) => {
                let parent = need(self.symbol(container));
                let table = self.ensure_members(parent);
                self.declare_symbol(table, Some(parent), node, flags, excludes)
            }
            Some(
                K::FunctionType
                | K::ConstructorType
                | K::CallSignature
                | K::ConstructSignature
                | K::IndexSignature
                | K::MethodDeclaration
                | K::MethodSignature
                | K::Constructor
                | K::GetAccessor
                | K::SetAccessor
                | K::FunctionDeclaration
                | K::FunctionExpression
                | K::ArrowFunction
                | K::ClassStaticBlockDeclaration
                | K::TypeAliasDeclaration
                | K::JSTypeAliasDeclaration
                | K::MappedType,
            ) => {
                let table = self.ensure_locals(container);
                self.declare_symbol(table, None, node, flags, excludes)
            }
            _ => panic!("Unhandled case in declareSymbolAndAddToSymbolTable"),
        }
    }
}
