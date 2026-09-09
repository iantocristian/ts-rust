use super::*;
use crate::{AstBuilder, Factory, FactoryMethods, NodeData, SyntaxKind, TokenData};
use ts_arena::Counters;
use ts_core::TextRange;

#[test]
fn physical_header_is_24_bytes_and_typed_pages_keep_addresses_during_growth() {
    assert_eq!(std::mem::size_of::<StoredNode>(), 24);
    let mut rows = RowPages::<u32>::default();
    rows.push(41);
    let first = std::ptr::from_ref(rows.get(0).unwrap());
    for value in 1..1027 {
        rows.push(value);
    }
    assert_eq!(std::ptr::from_ref(rows.get(0).unwrap()), first);
    assert_eq!(rows.get(0), Some(&41));
    assert_eq!(rows.get(1026), Some(&1026));
    assert!(rows.get(1027).is_none());
}

#[test]
fn compact_reads_and_unrestricted_edits_keep_identity_text_and_independent_shape() {
    let mut ast = AstBuilder::new(
        SourceText::from_loaded_bytes(b"alpha beta".as_slice()),
        &Counters::new(),
    );
    let name = ast.new_identifier(JsString::from_bytes(b"alpha".as_slice()));
    ast.finish_node(name, TextRange::new(0, 5), 0);
    let runtime = crate::runtime_node_id(&ast.node(name));
    ast.set_node_range(name, TextRange::new(6, 10));
    assert_eq!(ast.node(name).as_identifier().unwrap().text(), b"alpha");
    {
        let mut node = ast.node_mut(name).unwrap();
        assert_eq!(crate::runtime_node_id(&*node), runtime);
        node.data_mut().clone_from(&NodeData::Token(TokenData {}));
    }
    let node = ast.node(name);
    assert_eq!(node.kind(), SyntaxKind::Identifier);
    assert!(node.as_token().is_some());
    assert!(node.as_identifier().is_none());
    assert_eq!(crate::runtime_node_id(&node), runtime);
    drop(node);
    let second = ast.new_identifier(JsString::from_bytes(b"beta".as_slice()));
    let second_runtime = {
        let node = ast.node_mut(second).unwrap();
        crate::runtime_node_id(&*node)
    };
    assert_ne!(second_runtime, 0);
    assert_ne!(runtime, second_runtime);
    assert_eq!(crate::runtime_node_id(&ast.node(second)), second_runtime);
    let file = ast.complete(name).unwrap().publish_unbound();
    assert_eq!(
        crate::runtime_node_id(&file.view().node(second).unwrap()),
        second_runtime
    );
}

#[test]
fn compact_reference_words_preserve_full_slot_and_foreign_namespaces() {
    let owner = AstBuilder::new(SourceText::default(), &Counters::new());
    let foreign = AstBuilder::new(SourceText::default(), &Counters::new());
    let nodes = owner.id().arena();
    let auxiliary = owner.view().0.auxiliary_arena();
    let mut store = CoreStore::default();
    let source = SourceText::default();
    let ids = [
        None,
        Some(NodeId::from_parts(nodes, 7).unwrap()),
        Some(NodeId::from_parts(nodes, u32::MAX).unwrap()),
        Some(NodeId::from_parts(foreign.id().arena(), 7).unwrap()),
    ];
    let mut words = Vec::new();
    {
        let (_, mut context) = store.packing_parts(nodes, auxiliary, &source, -1);
        for (row, id) in ids.iter().enumerate() {
            words.push(context.encode_node(FieldKey::new(3, row as u32, 0), *id));
        }
    }
    assert_eq!(words, [0, 7, u32::MAX, u32::MAX]);
    let context = CompactContext {
        nodes,
        auxiliary,
        source: &source,
        store: &store,
    };
    for (row, word) in words.into_iter().enumerate() {
        assert_eq!(
            context.decode_node(FieldKey::new(3, row as u32, 0), word),
            ids[row]
        );
    }
}

#[test]
fn escape_overwrites_clear_only_the_changed_field_for_every_reference_namespace() {
    let counters = Counters::new();
    let owner = AstBuilder::new(SourceText::default(), &counters);
    let foreign = AstBuilder::new(SourceText::default(), &counters);
    let nodes = owner.id().arena();
    let auxiliary = owner.view().0.auxiliary_arena();
    let symbols = ts_arena::SymbolArena::<crate::Symbol>::new(&counters);
    let tables = crate::SymbolTables::new(&counters);
    let flows = crate::FlowNodes::new(&counters);
    let mut store = CoreStore::default();
    store.initialize_binding(binding::BindingArenas {
        symbols: symbols.id(),
        tables: tables.id(),
        flows: flows.id(),
    });
    let source = SourceText::default();
    let retained_key = FieldKey::parent(99);
    let retained_id = NodeId::from_parts(foreign.id().arena(), 99).unwrap();
    {
        let (_, mut context) = store.packing_parts(nodes, auxiliary, &source, -1);
        assert_eq!(
            context.encode_node(retained_key, Some(retained_id)),
            u32::MAX
        );
    }

    macro_rules! check_overwrite {
        ($encode:ident, $decode:ident, $id:ty, $arena:expr) => {{
            let key = FieldKey::new(3, 7, 0);
            let local = <$id>::from_parts($arena, 7).unwrap();
            for escaped in [
                <$id>::from_parts($arena, u32::MAX).unwrap(),
                <$id>::from_parts(foreign.id().arena(), 7).unwrap(),
            ] {
                for replacement in [Some(local), None] {
                    let word = {
                        let (_, mut context) = store.packing_parts(nodes, auxiliary, &source, -1);
                        assert_eq!(context.$encode(key, Some(escaped)), u32::MAX);
                        context.$encode(key, replacement)
                    };
                    let context = CompactContext {
                        nodes,
                        auxiliary,
                        source: &source,
                        store: &store,
                    };
                    assert_eq!(context.$decode(key, word), replacement);
                    assert_eq!(
                        context.decode_node(retained_key, u32::MAX),
                        Some(retained_id)
                    );
                    assert!(!store.links.contains_key(&key));
                    assert_eq!(store.links.len(), 1);
                }
            }
        }};
    }
    check_overwrite!(encode_node, decode_node, NodeId, nodes);
    check_overwrite!(encode_aux, decode_aux, AuxId, auxiliary);
    check_overwrite!(encode_symbol, decode_symbol, crate::SymbolId, symbols.id());
    check_overwrite!(
        encode_symbol_table,
        decode_symbol_table,
        crate::SymbolTableId,
        tables.id()
    );
    check_overwrite!(encode_flow, decode_flow, crate::FlowId, flows.id());

    {
        let (_, mut context) = store.packing_parts(nodes, auxiliary, &source, -1);
        assert_eq!(context.encode_node(retained_key, None), 0);
    }
    assert!(store.links.is_empty());
    assert_ne!(store.links.capacity(), 0);
    // Reuse an allocated but empty escape directory with ordinary values.
    let (_, mut context) = store.packing_parts(nodes, auxiliary, &source, -1);
    assert_eq!(
        context.encode_node(retained_key, Some(NodeId::from_parts(nodes, 7).unwrap())),
        7
    );
    assert_eq!(context.encode_node(retained_key, None), 0);
}
