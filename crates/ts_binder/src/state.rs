use std::collections::HashSet;
use ts_ast::{
    AstView, BindBuilder, FlowId, JsString, NodeBinding, NodeId, NodeRead, SymbolId, SymbolTable,
    SymbolTableId,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct ContainerFlags(pub i32);
impl ContainerFlags {
    pub const NONE: i32 = 0;
    pub const IS_CONTAINER: i32 = 1 << 0;
    pub const IS_BLOCK_SCOPED_CONTAINER: i32 = 1 << 1;
    pub const IS_CONTROL_FLOW_CONTAINER: i32 = 1 << 2;
    pub const IS_FUNCTION_LIKE: i32 = 1 << 3;
    pub const IS_FUNCTION_EXPRESSION: i32 = 1 << 4;
    pub const HAS_LOCALS: i32 = 1 << 5;
    pub const IS_INTERFACE: i32 = 1 << 6;
    pub const IS_OBJECT_LITERAL_OR_CLASS_EXPRESSION_METHOD_OR_ACCESSOR: i32 = 1 << 7;
    pub const IS_THIS_CONTAINER: i32 = 1 << 8;
    pub const PROPAGATES_THIS_KEYWORD: i32 = 1 << 9;
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct ExpandoAssignmentInfo {
    pub node: NodeId,
    pub container: Option<NodeId>,
    pub block_scope_container: Option<NodeId>,
}
#[derive(Debug)]
pub(crate) struct ActiveLabel {
    pub next: Option<usize>,
    pub break_target: Option<FlowId>,
    pub continue_target: Option<FlowId>,
    pub name: JsString,
    pub referenced: bool,
}

pub(crate) struct Binder<'build, 'ast> {
    pub builder: &'build mut BindBuilder<'ast>,
    pub file: NodeId,
    pub unreachable_flow: Option<FlowId>,
    pub container: Option<NodeId>,
    pub this_container: Option<NodeId>,
    pub block_scope_container: Option<NodeId>,
    pub last_container: Option<NodeId>,
    pub current_flow: Option<FlowId>,
    pub current_break_target: Option<FlowId>,
    pub current_continue_target: Option<FlowId>,
    pub current_return_target: Option<FlowId>,
    pub current_true_target: Option<FlowId>,
    pub current_false_target: Option<FlowId>,
    pub current_exception_target: Option<FlowId>,
    pub pre_switch_case_flow: Option<FlowId>,
    pub active_label_list: Option<usize>,
    pub labels: Vec<ActiveLabel>,
    pub emit_flags: u32,
    pub seen_this_keyword: bool,
    pub has_explicit_return: bool,
    pub has_flow_effects: bool,
    pub in_assignment_pattern: bool,
    pub seen_parse_error: bool,
    pub symbol_count: isize,
    pub not_const_enum_only_modules: HashSet<SymbolId>,
    pub expando_assignments: Vec<ExpandoAssignmentInfo>,
}
impl<'build, 'ast> Binder<'build, 'ast> {
    pub fn new(builder: &'build mut BindBuilder<'ast>) -> Self {
        let file = builder.source();
        Self {
            builder,
            file,
            unreachable_flow: None,
            container: None,
            this_container: None,
            block_scope_container: None,
            last_container: None,
            current_flow: None,
            current_break_target: None,
            current_continue_target: None,
            current_return_target: None,
            current_true_target: None,
            current_false_target: None,
            current_exception_target: None,
            pre_switch_case_flow: None,
            active_label_list: None,
            labels: Vec::new(),
            emit_flags: 0,
            seen_this_keyword: false,
            has_explicit_return: false,
            has_flow_effects: false,
            in_assignment_pattern: false,
            seen_parse_error: false,
            symbol_count: 0,
            not_const_enum_only_modules: HashSet::new(),
            expando_assignments: Vec::new(),
        }
    }
    pub fn parsed_view(&self) -> AstView<'ast> {
        self.builder.parsed_view()
    }
    pub fn view(&self) -> AstView<'_> {
        self.builder.view()
    }
    pub fn n(&self, id: NodeId) -> NodeRead<'_> {
        self.builder.node(id).expect("binder node is retained")
    }
    pub fn binding_mut(&mut self, id: NodeId) -> &mut NodeBinding {
        self.builder
            .binding_mut(id)
            .expect("binder writes its own file")
    }
    pub fn symbol(&self, node: NodeId) -> Option<SymbolId> {
        self.builder
            .node_symbol(node)
            .expect("binder node is retained")
    }
    pub fn locals(&self, node: NodeId) -> Option<SymbolTableId> {
        self.builder
            .node_locals(node)
            .expect("binder node is retained")
    }
    pub fn table(&self, table: SymbolTableId) -> &SymbolTable {
        self.builder
            .tables()
            .get(table)
            .expect("binder symbol table belongs to result")
    }
    pub fn table_mut(&mut self, table: SymbolTableId) -> &mut SymbolTable {
        self.builder
            .tables_mut()
            .get_mut(table)
            .expect("binder symbol table belongs to result")
    }
    // port: tsc/internal/ast/utilities.go:GetLocals
    pub fn ensure_locals(&mut self, node: NodeId) -> SymbolTableId {
        if let Some(table) = self.locals(node) {
            return table;
        }
        assert!(
            ts_ast::is_locals_container(&self.n(node)),
            "locals-container payload required"
        );
        let table = self.builder.tables_mut().alloc(SymbolTable::new());
        self.binding_mut(node).locals = Some(table);
        table
    }
    pub fn set_flags(&mut self, node: NodeId, flags: u32) {
        if self.n(node).flags() != flags {
            self.builder
                .node_mut(node)
                .expect("binder writes its own file")
                .set_flags(flags);
        }
    }
    pub fn text(&self, node: NodeId) -> JsString {
        self.view()
            .node_text(node)
            .expect("text payload required")
            .into_js_string()
    }
}
