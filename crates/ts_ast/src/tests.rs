use std::ops::ControlFlow;
use std::sync::Arc;

use ts_arena::{Counters, FileBuilder};

use crate::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Edge {
    Node(NodeId),
    List(NodeListRange),
}

#[derive(Default)]
struct Record {
    edges: Vec<Edge>,
    stop_after: Option<usize>,
}
impl Record {
    fn push(&mut self, edge: Edge) -> ControlFlow<()> {
        self.edges.push(edge);
        if self.stop_after == Some(self.edges.len()) {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}
impl ChildVisitor for Record {
    fn visit_node(&mut self, node: NodeId) -> ControlFlow<()> {
        self.push(Edge::Node(node))
    }
    fn visit_list(&mut self, nodes: NodeListRange) -> ControlFlow<()> {
        self.push(Edge::List(nodes))
    }
}

fn ids() -> (FileBuilder<()>, [NodeId; 4]) {
    let mut builder = FileBuilder::new(Arc::from([]), &Counters::default());
    let ids = std::array::from_fn(|_| builder.push(ts_arena::Node::new(0, ())));
    (builder, ids)
}

#[test]
fn syntax_kind_values_round_trip_and_markers_keep_upstream_values() {
    assert_eq!(SyntaxKind::Unknown as u16, 0);
    assert_eq!(SyntaxKind::EndOfFile as u16, 1);
    assert_eq!(SyntaxKind::NumericLiteral as u16, 8);
    assert_eq!(SyntaxKind::FirstToken, SyntaxKind::Unknown);
    assert_eq!(SyntaxKind::LastToken, SyntaxKind::DeferKeyword);
    for kind in SyntaxKind::ALL {
        assert_eq!(SyntaxKind::from_u16(*kind as u16), Some(*kind));
        assert!(!kind.as_str().is_empty());
    }
    assert_eq!(SyntaxKind::from_u16(SyntaxKind::COUNT as u16), None);
    assert_eq!(SyntaxKind::from_u16(u16::MAX), None);
    assert!(SyntaxKind::PlusToken.is_binary_operator());
    assert!(!SyntaxKind::SourceFile.is_binary_operator());
}

#[test]
fn validates_kind_data_pair_without_losing_text_bytes() {
    let data = IdentifierData {
        text: JsString::from_bytes(&b"\xed\xa0\x80\xff"[..]),
    };
    let node = Node::new(SyntaxKind::Identifier, -1, -1, data.clone().into()).unwrap();
    assert_eq!(
        node.data().as_identifier().unwrap().text.as_bytes(),
        b"\xed\xa0\x80\xff"
    );
    assert!(node.data().as_binary_expression().is_none());
    assert!(Node::new(SyntaxKind::StringLiteral, 0, 0, data.into()).is_err());
    assert_eq!((node.pos(), node.end()), (-1, -1));
}

#[test]
fn optional_schema_kind_is_a_go_scalar_and_flags_keep_all_bits() {
    for phase_modifier in [
        SyntaxKind::Unknown,
        SyntaxKind::TypeKeyword,
        SyntaxKind::DeferKeyword,
    ] {
        let mut node = Node::new(
            SyntaxKind::ImportClause,
            0,
            0,
            ImportClauseData {
                phase_modifier,
                name: None,
                named_bindings: None,
            }
            .into(),
        )
        .unwrap();
        assert_eq!(node.flags(), 0);
        node.set_flags(u32::MAX);
        assert_eq!(node.flags(), u32::MAX);
        assert_eq!(
            node.data().as_import_clause().unwrap().phase_modifier,
            phase_modifier
        );
    }
}

#[test]
fn visitor_preserves_list_boundary_order_and_short_circuits() {
    let (builder, [left, operator, right, type_node]) = ids();
    let list = NodeListRange::new(builder.id().arena(), 2, 0).unwrap();
    let binary: NodeData = BinaryExpressionData {
        modifiers: Some(list),
        left: Some(left),
        r#type: Some(type_node),
        operator_token: Some(operator),
        right: Some(right),
    }
    .into();
    let mut record = Record::default();
    assert_eq!(
        binary.for_each_child(&mut record),
        ControlFlow::Continue(())
    );
    assert_eq!(
        record.edges,
        [
            Edge::List(list),
            Edge::Node(left),
            Edge::Node(type_node),
            Edge::Node(operator),
            Edge::Node(right)
        ]
    );
    let mut record = Record {
        stop_after: Some(2),
        ..Record::default()
    };
    assert_eq!(binary.for_each_child(&mut record), ControlFlow::Break(()));
    assert_eq!(record.edges, [Edge::List(list), Edge::Node(left)]);
}

#[test]
fn default_clause_can_have_nil_expression_despite_required_schema_property() {
    let (builder, _) = ids();
    let statements = NodeListRange::new(builder.id().arena(), 0, 0).unwrap();
    let node = Node::new(
        SyntaxKind::DefaultClause,
        0,
        0,
        CaseOrDefaultClauseData {
            expression: None,
            statements: Some(statements),
        }
        .into(),
    )
    .unwrap();
    let mut record = Record::default();
    assert_eq!(node.for_each_child(&mut record), ControlFlow::Continue(()));
    assert_eq!(record.edges, [Edge::List(statements)]);
}

#[test]
fn jsdoc_visit_order_is_runtime_dependent_and_mapping_keeps_factory_order() {
    let (builder, [tag, name, typ, replacement]) = ids();
    let comment = NodeListRange::new(builder.id().arena(), 0, 1).unwrap();
    for is_name_first in [false, true] {
        let data: NodeData = JSDocParameterOrPropertyTagData {
            tag_name: Some(tag),
            comment: Some(comment),
            name: Some(name),
            is_bracketed: false,
            type_expression: Some(typ),
            is_name_first,
        }
        .into();
        let mut record = Record::default();
        assert_eq!(data.for_each_child(&mut record), ControlFlow::Continue(()));
        let (first, second) = if is_name_first {
            (name, typ)
        } else {
            (typ, name)
        };
        assert_eq!(
            record.edges,
            [
                Edge::Node(tag),
                Edge::Node(first),
                Edge::Node(second),
                Edge::List(comment)
            ]
        );
        struct Mapper {
            seen: Vec<NodeId>,
            replacement: NodeId,
        }
        impl ChildMapper for Mapper {
            fn map_node(&mut self, id: NodeId, role: ChildRole) -> NodeId {
                assert_eq!(role, ChildRole::Node);
                self.seen.push(id);
                self.replacement
            }
            fn map_list(&mut self, list: NodeListRange, role: ChildRole) -> NodeListRange {
                assert_eq!(role, ChildRole::Nodes);
                list
            }
        }
        let mut mapper = Mapper {
            seen: Vec::new(),
            replacement,
        };
        let mapped = data.map_children(&mut mapper);
        assert_eq!(mapper.seen, [tag, name, typ]);
        assert_eq!(
            mapped.as_js_doc_parameter_or_property_tag().unwrap().name,
            Some(replacement)
        );
        assert_eq!(
            data.as_js_doc_parameter_or_property_tag().unwrap().name,
            Some(name)
        );
    }
}

#[test]
fn range_checks_arithmetic_without_claiming_owner_validation() {
    let (builder, _) = ids();
    let owner = builder.id().arena();
    assert!(NodeListRange::new(owner, u32::MAX, 1).is_none());
    assert!(NodeListRange::new(owner, u32::MAX, 0).unwrap().is_empty());
    let range = NodeListRange::new(owner, 10, 20).unwrap();
    assert_eq!((range.owner(), range.start(), range.len()), (owner, 10, 20));
}

#[test]
fn mapping_preserves_special_source_file_and_statement_roles() {
    let (builder, [eof, condition, then_statement, else_statement]) = ids();
    let statements = NodeListRange::new(builder.id().arena(), 0, 2).unwrap();
    struct Roles(Vec<ChildRole>);
    impl ChildMapper for Roles {
        fn map_node(&mut self, node: NodeId, role: ChildRole) -> NodeId {
            self.0.push(role);
            node
        }
        fn map_list(&mut self, list: NodeListRange, role: ChildRole) -> NodeListRange {
            self.0.push(role);
            list
        }
    }
    let source: NodeData = SourceFileData {
        statements: Some(statements),
        end_of_file_token: Some(eof),
    }
    .into();
    let mut roles = Roles(Vec::new());
    assert_eq!(source.map_children(&mut roles), source);
    assert_eq!(roles.0, [ChildRole::TopLevelStatements, ChildRole::Token]);

    let statement: NodeData = IfStatementData {
        expression: Some(condition),
        then_statement: Some(then_statement),
        else_statement: Some(else_statement),
    }
    .into();
    let mut roles = Roles(Vec::new());
    assert_eq!(statement.map_children(&mut roles), statement);
    assert_eq!(
        roles.0,
        [
            ChildRole::Node,
            ChildRole::EmbeddedStatement,
            ChildRole::EmbeddedStatement
        ]
    );
    let raw: NodeData = SyntaxListData {
        children: Some(statements),
    }
    .into();
    let mut roles = Roles(Vec::new());
    assert_eq!(raw.map_children(&mut roles), raw);
    assert_eq!(roles.0, [ChildRole::RawNodes]);
}

#[test]
fn large_payloads_do_not_inflate_every_enum_slot() {
    assert!(std::mem::size_of::<NodeData>() <= 72);
    assert!(std::mem::size_of::<Node>() <= 88);
    assert!(std::mem::size_of::<FunctionDeclarationData>() > 64);
    assert!(DEFERRED_FIELDS
        .iter()
        .any(|field| field.node == "SyntheticExpression"
            && field.field == "Type"
            && field.owner == "checker type"));
    assert!(!DEFERRED_FIELDS.iter().any(|field| field.field == "Text"));
}
