//! Generated algorithms checked against pinned Go factory/dispatch witnesses.

use std::ops::ControlFlow;
use std::sync::{Arc, Mutex};

use ts_arena::Counters;
use ts_ast::*;
use ts_core::TextRange;
use ts_jsstring::SourceText;

#[derive(Default)]
struct Hooks(Mutex<Vec<[i64; 7]>>);

impl Hooks {
    fn record(&self, stage: i64, factory: &dyn Factory, id: NodeId) {
        let node = factory.node(id);
        self.0.lock().unwrap().push([
            stage,
            i64::from(node.kind().raw()),
            i64::from(node.flags()),
            i64::from(node.pos()),
            i64::from(node.end()),
            factory.node_count(),
            factory.text_count(),
        ]);
    }
}

impl FactoryHooks for Hooks {
    fn on_create(&self, factory: &mut dyn Factory, id: NodeId) {
        self.record(0, factory, id);
        factory.set_node_flags(id, 0x8000_0000);
    }
    fn on_update(&self, factory: &mut dyn Factory, id: NodeId, _: NodeId) {
        self.record(1, factory, id);
    }
    fn on_clone(&self, factory: &mut dyn Factory, id: NodeId, _: NodeId) {
        self.record(2, factory, id);
    }
}

fn builder() -> AstBuilder {
    AstBuilder::new(
        SourceText::from_loaded_bytes(&b""[..]),
        &Counters::default(),
    )
}

#[test]
fn factory_masks_counts_and_hooks_follow_the_pinned_observation_order() {
    let hooks = Arc::new(Hooks::default());
    let mut factory = AstBuilder::with_hooks(
        SourceText::from_loaded_bytes(&b""[..]),
        &Counters::default(),
        hooks.clone(),
    );
    let name = factory.new_identifier(JsString::from_bytes(&b"id"[..]));
    let property = factory.new_property_access_expression(Some(name), None, Some(name), u32::MAX);
    assert_eq!(factory.node(property).flags(), 2_147_483_680);
    factory.node_mut(property).unwrap().set_flags(0xdead_beef);
    factory
        .node_mut(property)
        .unwrap()
        .set_range(TextRange::new(17, 29));
    let unchanged = factory.update_property_access_expression(
        property,
        Some(name),
        None,
        Some(name),
        0xdead_beef,
    );
    let changed =
        factory.update_property_access_expression(property, Some(name), None, None, 0xdead_beef);
    let cloned = factory.clone_property_access_expression(property);
    assert_eq!(unchanged, property);
    assert_ne!(changed, property);
    assert_ne!(cloned, property);
    // Values recorded by clean Go at pin 1f70213, including pre-mask OnCreate.
    assert_eq!(
        *hooks.0.lock().unwrap(),
        [
            [0, 79, 0, -1, -1, 1, 1],
            [0, 212, 0, -1, -1, 2, 1],
            [0, 212, 0, -1, -1, 3, 1],
            [1, 212, 3_735_928_559, 17, 29, 3, 1],
            [0, 212, 0, -1, -1, 4, 1],
            [1, 212, 3_735_928_559, 17, 29, 4, 1],
            [2, 212, 3_735_928_559, 17, 29, 4, 1],
        ]
    );
}

#[test]
fn raw_slice_updates_use_backing_identity_and_all_empty_slices_compare_same() {
    let mut factory = builder();
    let first = factory
        .text_slice(vec![JsString::from_bytes(&b"same"[..])])
        .unwrap();
    let second = factory
        .text_slice(vec![JsString::from_bytes(&b"same"[..])])
        .unwrap();
    let link = factory.new_js_doc_link(None, first);
    assert_eq!(factory.update_js_doc_link(link, None, first), link);
    let updated = factory.update_js_doc_link(link, None, second);
    assert_ne!(updated, link);
    let empty = factory.text_slice(vec![]).unwrap();
    let empty_link = factory.new_js_doc_link(None, TextSlice::empty());
    assert_eq!(
        factory.update_js_doc_link(empty_link, None, empty),
        empty_link
    );
    let cloned = factory.clone_js_doc_link(link);
    assert!(factory
        .node(cloned)
        .data()
        .as_js_doc_link()
        .unwrap()
        .text()
        .same(first));

    let missing = factory.new_case_or_default_clause(SyntaxKind::DefaultClause.into(), None, None);
    assert_eq!(
        factory
            .node(missing)
            .data()
            .as_case_or_default_clause()
            .unwrap()
            .expression(),
        None
    );
    let raw = factory.node_slice(vec![None, Some(missing)]).unwrap();
    let list = factory.new_syntax_list(raw);
    assert_eq!(
        factory
            .view()
            .node_slice(
                factory
                    .node(list)
                    .data()
                    .as_syntax_list()
                    .unwrap()
                    .children()
            )
            .unwrap()
            .iter()
            .collect::<Vec<_>>(),
        &[None, Some(missing)]
    );
}

struct NoChildren;
impl ChildVisitor for NoChildren {
    fn visit_node(&mut self, _: NodeId) -> ControlFlow<()> {
        panic!("unexpected child");
    }
    fn visit_list(&mut self, _: NodeListId) -> ControlFlow<()> {
        panic!("unexpected list");
    }
    fn visit_node_slice(&mut self, _: NodeSlice) -> ControlFlow<()> {
        panic!("unexpected raw slice");
    }
}

struct IdentityVisitor<'a> {
    factory: &'a mut AstBuilder,
    calls: Vec<ChildRole>,
}
impl Factory for IdentityVisitor<'_> {
    fn node(&self, id: NodeId) -> NodeRead<'_> {
        self.factory.node(id)
    }
    fn node_mut(&mut self, id: NodeId) -> NodeMut<'_> {
        Factory::node_mut(self.factory, id)
    }
    fn node_count(&self) -> i64 {
        self.factory.node_count()
    }
    fn text_count(&self) -> i64 {
        self.factory.text_count()
    }
    fn new_node(&mut self, kind: NodeKind, data: NodeData) -> NodeId {
        self.factory.new_node(kind, data)
    }
    fn increment_text_count(&mut self) {
        self.factory.increment_text_count();
    }
    fn set_node_flags(&mut self, id: NodeId, flags: u32) {
        self.factory.set_node_flags(id, flags);
    }
    fn finish_update(&mut self, updated: NodeId, original: NodeId) -> NodeId {
        self.factory.finish_update(updated, original)
    }
    fn finish_clone(&mut self, updated: NodeId, original: NodeId) -> NodeId {
        self.factory.finish_clone(updated, original)
    }
}
impl VisitContext for IdentityVisitor<'_> {
    fn visit_node(&mut self, node: Option<NodeId>, role: ChildRole) -> Option<NodeId> {
        self.calls.push(role);
        node
    }
    fn visit_list(&mut self, list: Option<NodeListId>, role: ChildRole) -> Option<NodeListId> {
        self.calls.push(role);
        list
    }
    fn map_raw_nodes(&mut self, nodes: NodeSlice) -> NodeSlice {
        self.calls.push(ChildRole::RawNodes);
        nodes
    }
    fn visit_each_child_source_file(&mut self, _: NodeId) -> NodeId {
        panic!("Token payload must not dispatch SourceFile visitor");
    }
}

#[test]
fn enumeration_uses_kind_but_clone_and_transformation_use_payload() {
    let mut factory = builder();
    let mismatch = factory.new_token(SyntaxKind::SourceFile.into());
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| factory
            .node(mismatch)
            .for_each_child(&mut NoChildren)))
        .is_err()
    );
    let unknown = factory.new_token(NodeKind::from_raw(-1));
    assert_eq!(
        factory.node(unknown).for_each_child(&mut NoChildren),
        ControlFlow::Continue(())
    );
    let cloned = factory.clone_node_generated(unknown).unwrap();
    assert_eq!(factory.node(cloned).kind().raw(), -1);
    let unknown_block = factory.new_node(
        NodeKind::from_raw(-1),
        BlockData {
            statements: None,
            multi_line: false,
        }
        .into(),
    );
    let mut visitor = IdentityVisitor {
        factory: &mut factory,
        calls: vec![],
    };
    assert_eq!(visitor.visit_each_child_generated(mismatch), mismatch);
    assert!(visitor.calls.is_empty());
    assert_eq!(
        visitor.visit_each_child_generated(unknown_block),
        unknown_block
    );
    assert_eq!(visitor.calls, [ChildRole::Nodes]);
}

#[test]
fn owner_validation_includes_references_omitted_from_child_enumeration() {
    let mut foreign = builder();
    let foreign_id = foreign.new_identifier(JsString::from_bytes(&b"foreign"[..]));
    let mut destination = builder();
    let data: NodeData = CallSignatureDeclarationData {
        type_parameters: None,
        parameters: None,
        r#type: None,
        full_signature: Some(foreign_id),
    }
    .into();
    // This is deliberately not a source visitor edge, but still needs an owner.
    assert_eq!(
        data.for_each_child(&mut NoChildren),
        ControlFlow::Continue(())
    );
    let result = data.validate_references(
        |id| destination.view().node(id).map(|_| ()),
        |id| destination.view().list(id).map(|_| ()),
        |nodes| destination.view().node_slice(nodes).map(|_| ()),
        |text| destination.view().text_slice(text).map(|_| ()),
    );
    assert_eq!(result, Err(ts_arena::Error::WrongOwner));
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        destination.new_node(SyntaxKind::CallSignature.into(), data)
    }))
    .is_err());
    assert_eq!(destination.node_count(), 0);
}
