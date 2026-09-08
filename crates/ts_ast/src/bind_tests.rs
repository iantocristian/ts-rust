use crate::*;
use std::{
    panic::{catch_unwind, panic_any, AssertUnwindSafe},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Barrier,
    },
    thread,
};
use ts_arena::{Counters, Error};
use ts_core::TextRange;
use ts_jsstring::SourceText;

fn make_parsed(counters: &Counters, name: &[u8]) -> (ParsedFile, NodeId, NodeId) {
    let text = SourceText::from_loaded_bytes(&b"x"[..]);
    let mut builder = AstBuilder::new(text.clone(), counters);
    let child = builder.new_identifier(JsString::from_bytes(&b"x"[..]));
    let statement = builder.new_expression_statement(Some(child));
    let nodes = builder.node_slice(vec![Some(statement)]).unwrap();
    let list = builder.new_list(TextRange::new(0, 1), nodes).unwrap();
    let source = builder.new_source_file(
        SourceFileParseOptions {
            file_name: JsString::from_bytes(name),
            ..SourceFileParseOptions::default()
        },
        text,
        Some(list),
        None,
    );
    builder.node_mut(child).unwrap().set_parent(Some(statement));
    builder
        .node_mut(statement)
        .unwrap()
        .set_parent(Some(source));
    (builder.complete(source).unwrap(), source, child)
}
fn declare(builder: &mut BindBuilder<'_>, child: NodeId) -> Result<ts_arena::SymbolId, Error> {
    let declarations = builder.declarations_mut().alloc(vec![Some(child)])?;
    let mut symbol = Symbol::new(
        symbol_flags::FUNCTION_SCOPED_VARIABLE,
        JsString::from_bytes(&b"x"[..]),
    );
    symbol.declarations = declarations;
    symbol.value_declaration = Some(child);
    let symbol = builder.symbols_mut().push(symbol);
    builder.binding_mut(child)?.symbol = Some(symbol);
    builder.node_mut(child)?.set_flags(node_flags::UNREACHABLE);
    Ok(symbol)
}

#[test]
fn flow_only_storage_promotes_without_losing_links_and_rejects_foreign_flows() {
    let counters = Counters::new();
    let (parsed, source, child) = make_parsed(&counters, b"/flow-promotion.ts");
    let file = parsed.publish_unbound();
    let mut foreign = FlowNodes::new(&counters);
    let foreign_flow = foreign.push(FlowNode::new(flow_flags::START));
    let bound = file
        .bind_with(source, |builder| {
            let flow = builder.flows_mut().push(FlowNode::new(flow_flags::START));
            assert_eq!(
                builder.set_node_flow(child, Some(foreign_flow)),
                Err(Error::WrongOwner)
            );
            assert!(builder.binding(child)?.is_none());
            builder.set_node_flow(child, Some(flow))?;
            assert_eq!(builder.binding(child)?.unwrap().flow_node, Some(flow));
            assert_eq!(builder.result().bindings().count(), 1);
            builder.set_node_flow(child, None)?;
            assert!(builder.binding(child)?.is_none());
            builder.set_node_flow(child, Some(flow))?;
            let symbol = declare(builder, child)?;
            let binding = builder.binding(child)?.unwrap();
            assert_eq!(binding.flow_node, Some(flow));
            assert_eq!(binding.symbol, Some(symbol));
            assert_eq!(builder.result().bindings().count(), 1);
            builder.set_node_flow(child, None)?;
            assert_eq!(builder.binding(child)?.unwrap().symbol, Some(symbol));
            assert_eq!(builder.binding(child)?.unwrap().flow_node, None);
            Ok(())
        })
        .unwrap();
    assert_eq!(bound.node_binding(child).unwrap().unwrap().flow_node, None);
    assert_eq!(bound.result().bindings().count(), 1);
}

#[test]
fn binding_publishes_staged_headers_symbols_and_source_metadata_without_changing_parsed_views() {
    let counters = Counters::new();
    let (parsed, source, child) = make_parsed(&counters, b"/one.ts");
    let file = parsed.publish_unbound();
    let before = file.view();
    let id = runtime_node_id(&before.node(child).unwrap());
    let bound = file
        .bind_with(source, |builder| {
            declare(builder, child)?;
            builder.set_common_js_module_indicator(Some(child));
            assert!(utilities::is_external_or_common_js_module(
                &builder.view().source_file(source)?
            ));
            assert_eq!(builder.view().node(child)?.flags(), node_flags::UNREACHABLE);
            Ok(())
        })
        .unwrap();
    assert_eq!(before.node(child).unwrap().flags(), 0);
    assert_eq!(bound.node(child).unwrap().flags(), node_flags::UNREACHABLE);
    assert_eq!(runtime_node_id(&bound.node(child).unwrap()), id);
    assert_eq!(
        before
            .source_file(source)
            .unwrap()
            .common_js_module_indicator(),
        None
    );
    assert_eq!(
        bound.source_file().unwrap().common_js_module_indicator(),
        Some(child)
    );
    assert!(utilities::is_external_or_common_js_module(
        &bound.source_file().unwrap()
    ));
    let symbol = bound.result().node_binding(child).unwrap().symbol.unwrap();
    assert_eq!(bound.symbol(symbol).unwrap().value_declaration, Some(child));
    assert_eq!(
        file.bind_with(source, |_| panic!("must not replay binding"))
            .unwrap()
            .result()
            .node_binding(child)
            .unwrap()
            .symbol,
        Some(symbol)
    );
}

#[test]
fn binding_concurrent_first_use_publishes_once_and_keeps_stable_symbol_ids() {
    let counters = Counters::new();
    let (parsed, source, child) = make_parsed(&counters, b"/race.ts");
    let file = parsed.publish_unbound();
    let barrier = Barrier::new(4);
    let calls = AtomicUsize::new(0);
    let ids = thread::scope(|scope| {
        let handles: Vec<_> = (0..4)
            .map(|_| {
                scope.spawn(|| {
                    barrier.wait();
                    let bound = file
                        .bind_with(source, |builder| {
                            calls.fetch_add(1, Ordering::SeqCst);
                            declare(builder, child)?;
                            Ok(())
                        })
                        .unwrap();
                    bound.result().node_binding(child).unwrap().symbol.unwrap()
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(ids.iter().all(|&id| id == ids[0]));
}

#[test]
fn binding_panic_is_terminal_and_preserves_the_initiators_payload() {
    #[derive(Debug, PartialEq)]
    struct Payload(u32);
    let counters = Counters::new();
    let (parsed, source, child) = make_parsed(&counters, b"/panic.ts");
    let file = parsed.publish_unbound();
    let before = counters.snapshot();
    let entered = Barrier::new(2);
    let release = Barrier::new(2);
    thread::scope(|scope| {
        let winner = scope.spawn(|| {
            catch_unwind(AssertUnwindSafe(|| {
                file.bind_with(source, |builder| {
                    declare(builder, child)?;
                    entered.wait();
                    release.wait();
                    panic_any(Payload(17));
                })
            }))
        });
        entered.wait();
        let waiter =
            scope.spawn(|| file.bind_with(source, |_| panic!("waiter must not initialize")));
        release.wait();
        assert_eq!(
            *winner
                .join()
                .unwrap()
                .unwrap_err()
                .downcast::<Payload>()
                .unwrap(),
            Payload(17)
        );
        assert!(matches!(waiter.join().unwrap(), Err(BindError::Failed)));
    });
    assert_eq!(counters.snapshot(), before);
    assert!(!file.is_bound(source).unwrap());
    assert!(matches!(
        file.bind_with(source, |_| panic!("terminal failure must not retry")),
        Err(BindError::Failed)
    ));
    assert_eq!(file.view().node(child).unwrap().flags(), 0);
}

#[test]
fn binding_reentry_is_diagnosed_and_does_not_block_ordinary_lazy_nesting() {
    let counters = Counters::new();
    let (parsed, source, child) = make_parsed(&counters, b"/reentry.ts");
    let file = parsed.publish_unbound();
    let panic = catch_unwind(AssertUnwindSafe(|| {
        file.bind_with(source, |_| {
            file.bind_with(source, |_| Ok(())).unwrap();
            Ok(())
        })
    }))
    .unwrap_err();
    let reason = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap();
    assert!(reason.contains("reentrant binding"), "{reason}");
    assert!(!file.is_bound(source).unwrap());
    assert!(file.view().node(child).is_ok());
    let (parsed, source, child) = make_parsed(&counters, b"/lazy.ts");
    let file = parsed.publish_unbound();
    file.bind_with(source, |builder| {
        let roots = builder.view().jsdoc(child, |_| Ok(Vec::new()))?;
        assert!(roots.is_empty());
        Ok(())
    })
    .unwrap();
    assert!(file.is_bound(source).unwrap());
}

#[test]
fn binding_mapped_members_initialize_independently_and_retained_symbols_keep_the_bundle() {
    let counters = Counters::new();
    let before = counters.snapshot();
    let retained = {
        let (first, a, child_a) = make_parsed(&counters, b"/a.ts");
        let (second, b, _) = make_parsed(&counters, b"/b.ts");
        let (third, c, child_c) = make_parsed(&counters, b"/c.ts");
        let bundle = first.publish_bundle_unbound(vec![second, third]);
        let first = bundle.file(0).unwrap();
        let second = bundle.file(1).unwrap();
        let third = bundle.file(2).unwrap();
        let bound = first
            .bind_with(a, |builder| {
                declare(builder, child_a)?;
                Ok(())
            })
            .unwrap();
        let id = bound
            .result()
            .node_binding(child_a)
            .unwrap()
            .symbol
            .unwrap();
        assert!(!second.is_bound(b).unwrap());
        assert!(!third.is_bound(c).unwrap());
        assert!(catch_unwind(AssertUnwindSafe(
            || second.bind_with(b, |_| panic!("sibling failure"))
        ))
        .is_err());
        assert_eq!(
            first
                .bound_view(a)
                .unwrap()
                .unwrap()
                .symbol(id)
                .unwrap()
                .value_declaration,
            Some(child_a)
        );
        third
            .bind_with(c, |builder| {
                declare(builder, child_c)?;
                Ok(())
            })
            .unwrap();
        let file = first.retain_bound(a).unwrap();
        let node = file.retain_node(child_a).unwrap();
        let symbol = file.retain_symbol(id).unwrap();
        (node, symbol, c, child_c)
    };
    assert_ne!(counters.snapshot(), before);
    assert_eq!(retained.0.node().flags(), node_flags::UNREACHABLE);
    assert_eq!(retained.1.value_declaration, Some(retained.0.id()));
    let sibling = retained.1.file().parsed_file();
    assert!(sibling.is_bound(retained.2).unwrap());
    assert!(sibling
        .bound_view(retained.2)
        .unwrap()
        .unwrap()
        .result()
        .node_binding(retained.3)
        .unwrap()
        .symbol
        .is_some());
    drop(retained);
    assert_eq!(counters.snapshot(), before);
}

#[test]
fn binding_rejects_foreign_symbol_flow_and_ast_edges_before_publication() {
    let counters = Counters::new();
    let (foreign, _, foreign_node) = make_parsed(&counters, b"/foreign.ts");
    let _foreign = foreign.publish_unbound();
    let mut symbols = ts_arena::SymbolArena::new(&counters);
    let foreign_symbol = symbols.push(Symbol::default());
    let mut flows = FlowNodes::new(&counters);
    let foreign_flow = flows.push(FlowNode::default());
    for case in 0..6 {
        let (parsed, source, child) = make_parsed(&counters, b"/owner.ts");
        let file = parsed.publish_unbound();
        let result = file.bind_with(source, |builder| {
            match case {
                0 => builder.node_mut(child)?.set_parent(Some(foreign_node)),
                1 => builder.binding_mut(child)?.symbol = Some(foreign_symbol),
                2 => builder.binding_mut(child)?.flow_node = Some(foreign_flow),
                3 => {
                    let mut symbol = Symbol::default();
                    symbol.value_declaration = Some(foreign_node);
                    builder.symbols_mut().push(symbol);
                }
                4 => {
                    builder.flows_mut().push(FlowNode::new_ex(
                        flow_flags::START,
                        Some(FlowData::Ast(foreign_node)),
                        None,
                    ));
                }
                _ => builder.set_common_js_module_indicator(Some(foreign_node)),
            }
            Ok(())
        });
        assert!(
            matches!(result, Err(BindError::Storage(Error::WrongOwner))),
            "case {case}: {result:?}"
        );
        assert!(!file.is_bound(source).unwrap());
        assert!(matches!(
            file.bind_with(source, |_| Ok(())),
            Err(BindError::Failed)
        ));
    }
}

#[test]
fn binding_imported_initializer_cannot_borrow_the_importers_wider_retention() {
    let counters = Counters::new();
    let (first, source, child) = make_parsed(&counters, b"/retained.ts");
    let first = first.publish_unbound();
    let (foreign, _, foreign_child) = make_parsed(&counters, b"/extra.ts");
    let foreign = foreign.publish_unbound();
    let (mut caller, root, _) = make_parsed(&counters, b"/caller.ts");
    caller.builder_mut().retain_file(first);
    caller.builder_mut().retain_file(foreign);
    let caller = caller.publish_unbound();
    assert!(caller.view().node(foreign_child).is_ok());
    assert!(matches!(
        caller.bind_with(source, |builder| {
            builder.node_mut(child)?.set_parent(Some(foreign_child));
            Ok(())
        }),
        Err(BindError::Storage(Error::WrongOwner))
    ));
    assert!(!caller.is_bound(source).unwrap());
    assert!(!caller.is_bound(root).unwrap());
}

#[test]
fn bound_mapped_reads_and_retention_select_the_target_sources_completed_overlay() {
    let counters = Counters::new();
    let (first, a, child_a) = make_parsed(&counters, b"/a.ts");
    let (second, b, child_b) = make_parsed(&counters, b"/b.ts");
    let bundle = first.publish_bundle_unbound(vec![second]);
    let first = bundle.file(0).unwrap();
    let second = bundle.file(1).unwrap();
    first
        .bind_with(a, |builder| {
            declare(builder, child_a)?;
            Ok(())
        })
        .unwrap();
    let retained_a = first.retain_bound(a).unwrap();
    let before_b = retained_a.view();
    assert_eq!(before_b.node(child_b).unwrap().flags(), 0);
    assert!(matches!(
        retained_a.retain_node(child_b),
        Err(Error::InvalidGraph)
    ));
    assert!(!second.is_bound(b).unwrap());
    second
        .bind_with(b, |builder| {
            declare(builder, child_b)?;
            builder.node_mut(child_b)?.set_flags(node_flags::AMBIENT);
            builder.set_common_js_module_indicator(Some(child_b));
            builder.diagnostics_mut().push(Diagnostic::new(
                Some(b),
                TextRange::new(0, 1),
                ts_diagnostics::Identifier_expected,
                vec![],
            ));
            Ok(())
        })
        .unwrap();
    assert_eq!(before_b.node(child_b).unwrap().flags(), node_flags::AMBIENT);
    assert!(before_b
        .node_binding(child_b)
        .unwrap()
        .unwrap()
        .symbol
        .is_some());
    let foreign_source = before_b.ast().source_file(b).unwrap();
    assert_eq!(foreign_source.common_js_module_indicator(), Some(child_b));
    assert_eq!(foreign_source.bind_diagnostics().len(), 1);
    assert!(before_b
        .source_file()
        .unwrap()
        .bind_diagnostics()
        .is_empty());
    let retained_b = retained_a.retain_node(child_b).unwrap();
    assert_eq!(retained_b.file().source(), b);
    drop((retained_a, first, second, bundle));
    assert_eq!(retained_b.node().flags(), node_flags::AMBIENT);
    assert_eq!(
        retained_b
            .file()
            .view()
            .source_file()
            .unwrap()
            .common_js_module_indicator(),
        Some(child_b)
    );
}

fn make_shared_arena(
    counters: &Counters,
) -> (ParsedFile, NodeId, NodeId, NodeId, NodeId, NodeId, NodeId) {
    let (mut parsed, a, child_a) = make_parsed(counters, b"/a.ts");
    let builder = parsed.builder_mut();
    let child_b = builder.new_identifier(JsString::from_bytes(b"b".as_slice()));
    let statement_b = builder.new_expression_statement(Some(child_b));
    let nodes = builder.node_slice(vec![Some(statement_b)]).unwrap();
    let statements = builder.new_list(TextRange::new(0, 1), nodes).unwrap();
    let b = builder.new_source_file(
        SourceFileParseOptions {
            file_name: JsString::from_bytes(b"/b.ts".as_slice()),
            ..SourceFileParseOptions::default()
        },
        SourceText::from_loaded_bytes(b"b".as_slice()),
        Some(statements),
        None,
    );
    builder
        .node_mut(child_b)
        .unwrap()
        .set_parent(Some(statement_b));
    builder.node_mut(statement_b).unwrap().set_parent(Some(b));
    let orphan = builder.new_identifier(JsString::from_bytes(b"orphan".as_slice()));
    let cycle = builder.new_identifier(JsString::from_bytes(b"cycle".as_slice()));
    builder.node_mut(cycle).unwrap().set_parent(Some(cycle));
    (parsed, a, child_a, b, child_b, orphan, cycle)
}

#[test]
fn bound_same_arena_sources_route_independently_and_reject_ambiguous_retention() {
    let counters = Counters::new();
    let (parsed, a, child_a, b, child_b, orphan, cycle) = make_shared_arena(&counters);
    let file = parsed.publish_unbound();
    file.bind_with(a, |builder| {
        declare(builder, child_a)?;
        Ok(())
    })
    .unwrap();
    let retained = file.retain_bound(a).unwrap();
    assert!(matches!(
        retained.retain_node(child_b),
        Err(Error::InvalidGraph)
    ));
    assert!(matches!(
        retained.retain_node(orphan),
        Err(Error::InvalidGraph)
    ));
    assert!(matches!(
        retained.retain_node(cycle),
        Err(Error::InvalidGraph)
    ));
    file.bind_with(b, |builder| {
        declare(builder, child_b)?;
        builder.node_mut(child_b)?.set_flags(node_flags::AMBIENT);
        builder.set_common_js_module_indicator(Some(child_b));
        Ok(())
    })
    .unwrap();
    let a_view = retained.view();
    assert_eq!(
        a_view.node(child_a).unwrap().flags(),
        node_flags::UNREACHABLE
    );
    assert_eq!(a_view.node(child_b).unwrap().flags(), node_flags::AMBIENT);
    assert_eq!(
        a_view
            .ast()
            .source_file(b)
            .unwrap()
            .common_js_module_indicator(),
        Some(child_b)
    );
    let b_retained = retained.retain_node(child_b).unwrap();
    assert_eq!(b_retained.file().source(), b);
    assert_eq!(b_retained.node().flags(), node_flags::AMBIENT);
    assert_eq!(file.view().node(child_b).unwrap().flags(), 0);
}

#[test]
fn binding_one_logical_source_cannot_stage_a_siblings_headers_in_the_same_arena() {
    let counters = Counters::new();
    let (parsed, a, _, b, child_b, _, _) = make_shared_arena(&counters);
    let file = parsed.publish_unbound();
    assert!(matches!(
        file.bind_with(a, |builder| {
            builder.node_mut(child_b)?.set_flags(node_flags::AMBIENT);
            Ok(())
        }),
        Err(BindError::Storage(Error::WrongOwner))
    ));
    assert!(!file.is_bound(a).unwrap());
    assert!(!file.is_bound(b).unwrap());
    file.bind_with(b, |builder| {
        declare(builder, child_b)?;
        Ok(())
    })
    .unwrap();
    assert_eq!(
        file.bound_view(b)
            .unwrap()
            .unwrap()
            .node(child_b)
            .unwrap()
            .flags(),
        node_flags::UNREACHABLE
    );
}

#[test]
fn factory_copies_select_each_retained_logical_sources_completed_binding() {
    let mut original = AstBuilder::new(SourceText::default(), &Counters::new());
    let mut sources = Vec::new();
    for name in [b"/one.js".as_slice(), b"/two.js", b"/unbound.js"] {
        let child = original.new_identifier(JsString::from_bytes(b"exports".as_slice()));
        let root = original.new_source_file(
            SourceFileParseOptions {
                file_name: JsString::from_bytes(name),
                ..Default::default()
            },
            SourceText::default(),
            None,
            None,
        );
        original.node_mut(child).unwrap().set_parent(Some(root));
        sources.push((root, child));
    }
    let file = original.complete(sources[0].0).unwrap().publish_unbound();
    for (index, &(root, child)) in sources[..2].iter().enumerate() {
        file.bind_with(root, |binding| {
            binding.set_common_js_module_indicator(Some(child));
            binding.node_mut(root)?.set_flags(1 << (index + 10));
            binding.node_mut(child)?.set_flags(1 << (index + 20));
            Ok(())
        })
        .unwrap();
    }
    let mut copies = AstBuilder::new(SourceText::default(), &Counters::new());
    copies.retain_file(file.clone());
    let mut roots = Vec::new();
    for (index, &(root, child)) in sources.iter().enumerate() {
        // Parsed access through either owner stays parsed after publication.
        assert_eq!(file.view().node(root).unwrap().flags(), 0);
        assert_eq!(copies.view().node(root).unwrap().flags(), 0);
        assert_eq!(copies.view().node(child).unwrap().flags(), 0);
        assert!(copies
            .view()
            .source_file(root)
            .unwrap()
            .common_js_module_indicator()
            .is_none());
        let clone = copies.clone_source_file(root);
        let eof = copies.new_token(SyntaxKind::EndOfFile.into());
        let update = copies.update_source_file(root, None, Some(eof));
        let child_clone = copies.clone_identifier(child);
        let bound = index < 2;
        for copy in [clone, update] {
            let state = copies.view().source_file(copy).unwrap();
            assert_eq!(state.common_js_module_indicator(), bound.then_some(child));
            assert_eq!(
                copies.view().node(copy).unwrap().flags(),
                if bound { 1 << (index + 10) } else { 0 }
            );
        }
        assert_eq!(
            copies.view().node(child_clone).unwrap().flags(),
            if bound { 1 << (index + 20) } else { 0 }
        );
        roots.extend([clone, update]);
    }
    let copies = copies.complete(roots[0]).unwrap().publish_unbound();
    drop(file);
    for &root in &roots {
        assert!(!copies.is_bound(root).unwrap());
        if let Some(indicator) = copies
            .view()
            .source_file(root)
            .unwrap()
            .common_js_module_indicator()
        {
            assert_eq!(
                copies.view().node(indicator).unwrap().kind(),
                SyntaxKind::Identifier
            );
        }
    }
    assert!(!copies.is_bound(sources[2].0).unwrap());
}
