//! Binder helpers whose child links stay in the exclusive owner's namespace.
use crate::need;
use crate::{backend::Backend, Binder};
use std::ops::ControlFlow;
use ts_ast::local_bind::{BindList, BindSlice, LocalChildVisitor};
use ts_ast::{
    local_bind::{BindNode, LocalBind},
    SyntaxKind as K,
};

#[derive(Clone, Copy)]
enum Child<'scope> {
    Node(BindNode<'scope>),
    List(BindList<'scope>),
    Slice(BindSlice<'scope>),
}
struct Children<'scope> {
    entries: [Option<Child<'scope>>; 9],
    len: usize,
}
impl<'scope> Children<'scope> {
    fn push(&mut self, child: Child<'scope>) -> ControlFlow<()> {
        self.entries[self.len] = Some(child);
        self.len += 1;
        ControlFlow::Continue(())
    }
}
impl<'scope> LocalChildVisitor<'scope> for Children<'scope> {
    fn visit_node(&mut self, node: BindNode<'scope>) -> ControlFlow<()> {
        self.push(Child::Node(node))
    }
    fn visit_list(&mut self, list: BindList<'scope>) -> ControlFlow<()> {
        self.push(Child::List(list))
    }
    fn visit_node_slice(&mut self, slice: BindSlice<'scope>) -> ControlFlow<()> {
        self.push(Child::Slice(slice))
    }
}

impl<'scope> Binder<'_, 'scope, '_> {
    pub(crate) fn bind_local_entry(&mut self, node: BindNode<'scope>) -> bool {
        crate::recursion::guarded(|| {
            let Backend::Local(local) = &self.builder else {
                unreachable!("local binder scope");
            };
            let read = local.node(node);
            let flags = read.flags();
            if read.kind() == K::Identifier && read.as_identifier().is_some() {
                let Backend::Local(local) = &mut self.builder else {
                    unreachable!("local binder scope");
                };
                if let Ok(flow) = self
                    .current_flow
                    .map(|flow| local.import_flow(flow))
                    .transpose()
                {
                    assert!(local.set_flow(node, flow), "Identifier flow field");
                } else {
                    let id = local.node_id(node);
                    local
                        .general_set_node_flow(id, self.current_flow)
                        .expect("binder flow and target belong to result");
                }
                self.check_local_contextual_identifier(node);
                if flags & ts_ast::node_flags::THIS_NODE_HAS_ERROR != 0 {
                    let Backend::Local(local) = &mut self.builder else {
                        unreachable!("local binder scope");
                    };
                    local.set_flags(
                        node,
                        flags | ts_ast::node_flags::THIS_NODE_OR_ANY_SUB_NODES_HAS_ERROR,
                    );
                    self.seen_parse_error = true;
                }
                return false;
            }
            // Remaining dispatch migration is explicit. This is one shared
            // algorithm, and no second traversal precedes it.
            self.bind_worker(Some(local.node_id(node)))
        })
    }

    pub(crate) fn try_bind_local_children(&mut self, node: ts_ast::NodeId) -> bool {
        let Backend::Local(local) = &self.builder else {
            return false;
        };
        let Ok(node) = local.import_node(node) else {
            return false;
        };
        let mut children = Children {
            entries: [None; 9],
            len: 0,
        };
        let _ = local.node(node).for_each_child(&mut children);
        for child in children.entries[..children.len].iter().flatten() {
            if let Child::Node(node) = *child {
                if self.bind_local_entry(node) {
                    break;
                }
                continue;
            }
            let Backend::Local(local) = &self.builder else {
                unreachable!("local binder scope");
            };
            let slice = match *child {
                Child::List(list) => local.list(list),
                Child::Slice(slice) => slice,
                Child::Node(_) => unreachable!(),
            };
            let edges = local.edges(slice);
            for index in 0..edges.len() {
                let Backend::Local(local) = &self.builder else {
                    unreachable!("local binder scope");
                };
                if let Some(node) = local.edge(edges, index) {
                    if self.bind_local_entry(node) {
                        return true;
                    }
                }
            }
        }
        true
    }
}

// port: tsc/internal/binder/binder.go:isNarrowableReference
pub(crate) fn is_narrowable_reference<'scope>(
    local: &LocalBind<'scope, '_>,
    node: BindNode<'scope>,
) -> bool {
    let read = local.node(node);
    match read.kind().known() {
        Some(K::Identifier | K::ThisKeyword | K::SuperKeyword | K::MetaProperty) => true,
        Some(K::PropertyAccessExpression) => is_narrowable_reference(
            local,
            need(
                read.as_property_access_expression()
                    .map(|data| data.expression())
                    .unwrap_or_else(|| incompatible_expression(local, node)),
            ),
        ),
        Some(K::ParenthesizedExpression) => is_narrowable_reference(
            local,
            need(
                read.as_parenthesized_expression()
                    .map(|data| data.expression())
                    .unwrap_or_else(|| incompatible_expression(local, node)),
            ),
        ),
        Some(K::NonNullExpression) => is_narrowable_reference(
            local,
            need(
                read.as_non_null_expression()
                    .map(|data| data.expression())
                    .unwrap_or_else(|| incompatible_expression(local, node)),
            ),
        ),
        Some(K::ElementAccessExpression) => {
            let access = read
                .as_element_access_expression()
                .expect("binder syntax payload");
            let argument = need(access.argument_expression());
            let kind = local.node(argument).kind();
            // These general helper boundaries are temporary migration work;
            // recursive receiver reads above and below remain local.
            matches!(
                kind.known(),
                Some(K::StringLiteral | K::NumericLiteral | K::NoSubstitutionTemplateLiteral)
            ) || ts_ast::is_entity_name_expression(local.view(), local.node_id(argument))
                .expect("retained element name")
                && is_narrowable_reference(local, need(access.expression()))
        }
        Some(K::BinaryExpression) => {
            let binary = read.as_binary_expression().expect("binder syntax payload");
            let operator = local.node(need(binary.operator_token())).kind();
            operator == K::CommaToken && is_narrowable_reference(local, need(binary.right()))
                || ts_ast::is_assignment_operator(operator)
                    && ts_ast::is_left_hand_side_expression(
                        local.view(),
                        local.node_id(need(binary.left())),
                    )
                    .expect("retained assignment target")
        }
        _ => false,
    }
}

// Preserve the public interface-conversion failure for a constructed kind/shape
// mismatch. This checked boundary is absent from matching typed receiver reads.
fn incompatible_expression<'scope>(
    local: &LocalBind<'scope, '_>,
    node: BindNode<'scope>,
) -> Option<BindNode<'scope>> {
    local
        .general_node(local.node_id(node))
        .expect("binder node is retained")
        .expression()
        .map(|node| local.import_node(node).expect("validated local expression"))
}
