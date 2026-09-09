//! Counterexamples at the boundary between local and checked binder access.
use crate::Binder;
use std::{
    ops::ControlFlow,
    panic::{catch_unwind, AssertUnwindSafe},
};
use ts_arena::Counters;
use ts_ast::{
    AstBuilder, AstView, ChildVisitor, Diagnostic, FactoryMethods, JsString, NodeId, NodeListId,
    NodeSlice, ParsedFile, SourceFileParseOptions, SyntaxKind,
};
use ts_core::{ScriptKind, TextRange};
use ts_jsstring::SourceText;

fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .unwrap_or_else(|| {
            payload
                .downcast_ref::<&str>()
                .expect("text contract panic")
                .to_string()
        })
}

fn malformed_receiver(kind: SyntaxKind) -> (ParsedFile, NodeId) {
    let source_text = SourceText::default();
    let mut build = AstBuilder::new(source_text.clone(), &Counters::new());
    let node = build.new_token(kind.into());
    let nodes = build.node_slice(vec![Some(node)]).unwrap();
    let list = build.new_list(TextRange::new(0, 0), nodes).unwrap();
    let source = build.new_source_file(
        SourceFileParseOptions {
            file_name: JsString::from_bytes(b"/local-receiver.ts".as_slice()),
            ..Default::default()
        },
        source_text,
        Some(list),
        None,
    );
    build.node_mut(node).unwrap().set_parent(Some(source));
    (build.complete(source).unwrap(), node)
}

#[test]
fn narrowable_receivers_preserve_checked_interface_conversion_panics() {
    for (kind, payload) in [
        (
            SyntaxKind::PropertyAccessExpression,
            "PropertyAccessExpression",
        ),
        (
            SyntaxKind::ParenthesizedExpression,
            "ParenthesizedExpression",
        ),
        (SyntaxKind::NonNullExpression, "NonNullExpression"),
    ] {
        let expected =
            format!("interface conversion: ast.nodeData is *ast.Token, not *ast.{payload}");
        let (parsed, node) = malformed_receiver(kind);
        parsed
            .bind_and_publish(|builder| {
                // Explicit Checked selection exercises the retained implementation,
                // even though the source also admits the new local capability.
                let checked_failure = {
                    let binder = Binder::new(builder);
                    catch_unwind(AssertUnwindSafe(|| binder.is_narrowable_reference(node)))
                        .unwrap_err()
                };
                assert_eq!(panic_message(&*checked_failure), expected);
                builder
                    .with_local_scope(|local| {
                        let node = local.import_node(node).unwrap();
                        let local_failure = catch_unwind(AssertUnwindSafe(|| {
                            crate::local::is_narrowable_reference(&local, node)
                        }))
                        .unwrap_err();
                        assert_eq!(panic_message(&*local_failure), expected);
                    })
                    .expect("constructed receiver admits local access");
                Ok(())
            })
            .unwrap();
    }
}

#[test]
fn local_block_dispatch_preserves_checked_payload_failures() {
    for kind in [SyntaxKind::Block, SyntaxKind::ModuleBlock] {
        let (parsed, node) = malformed_receiver(kind);
        parsed
            .bind_and_publish(|builder| {
                let checked_failure = {
                    let mut binder = Binder::new(builder);
                    binder.current_flow = Some(binder.new_flow_node(ts_ast::flow_flags::START));
                    catch_unwind(AssertUnwindSafe(|| binder.bind(Some(node)))).unwrap_err()
                };
                builder
                    .with_local_scope(|local| {
                        let mut binder =
                            Binder::from_backend(crate::backend::Backend::Local(local));
                        binder.current_flow = Some(binder.new_flow_node(ts_ast::flow_flags::START));
                        let local_failure =
                            catch_unwind(AssertUnwindSafe(|| binder.bind(Some(node)))).unwrap_err();
                        assert_eq!(
                            panic_message(&*local_failure),
                            panic_message(&*checked_failure)
                        );
                    })
                    .expect("constructed block admits local access");
                Ok(())
            })
            .unwrap();
    }
}

fn parse(bytes: &[u8]) -> ParsedFile {
    ts_parser::parse_source_file_with_counters(
        SourceText::from_loaded_bytes(bytes),
        ScriptKind::TS,
        SourceFileParseOptions {
            file_name: JsString::from_bytes(b"/contextual.ts".as_slice()),
            ..Default::default()
        },
        &Counters::new(),
    )
}

#[derive(Debug, PartialEq, Eq)]
struct Header {
    kind: i16,
    pos: i32,
    end: i32,
    flags: u32,
}

struct SyntaxHeaders<'view> {
    view: AstView<'view>,
    headers: Vec<Header>,
}
impl ChildVisitor for SyntaxHeaders<'_> {
    fn visit_node(&mut self, node: NodeId) -> ControlFlow<()> {
        let read = self.view.node(node).unwrap();
        self.headers.push(Header {
            kind: read.kind().raw(),
            pos: read.pos(),
            end: read.end(),
            flags: read.flags(),
        });
        read.for_each_child(self)
    }
    fn visit_list(&mut self, list: NodeListId) -> ControlFlow<()> {
        self.visit_node_slice(self.view.list(list).unwrap().nodes())
    }
    fn visit_node_slice(&mut self, nodes: NodeSlice) -> ControlFlow<()> {
        for node in self.view.node_slice(nodes).unwrap().iter().flatten() {
            self.visit_node(node)?;
        }
        ControlFlow::Continue(())
    }
}

fn syntax_headers(view: AstView<'_>, source: NodeId) -> Vec<Header> {
    let mut visitor = SyntaxHeaders {
        view,
        headers: Vec::new(),
    };
    assert_eq!(visitor.visit_node(source), ControlFlow::Continue(()));
    visitor.headers
}

fn diagnostics(values: &[Diagnostic], source: NodeId) -> Vec<Diagnostic> {
    values
        .iter()
        .map(|diagnostic| {
            assert_eq!(diagnostic.file, Some(source));
            assert!(diagnostic.message_chain.is_empty());
            assert!(diagnostic.related_information.is_empty());
            let mut diagnostic = diagnostic.clone();
            // Separate parses deliberately allocate distinct source identities.
            // Every other diagnostic field, including argument bytes, stays exact.
            diagnostic.file = None;
            diagnostic
        })
        .collect()
}

#[test]
fn parsed_contextual_keywords_preserve_diagnostics_and_error_flags_across_backends() {
    for (name, bytes) in [
        (
            "cooked reserved word",
            b"'use strict'; let \\u0069mplements = 1; implements;".as_slice(),
        ),
        ("external module", b"export {}; let await = 1; await;"),
        (
            "clean external module",
            b"export {}; let \\u0069mplements = 1; implements;",
        ),
        (
            "generator and async contexts",
            b"function* f() { let yield = 1; } async function g() { let await = 1; }",
        ),
        (
            "property names",
            b"const object = { implements: 1, await: 2 }; object.implements; object.await;",
        ),
        (
            "ambient context",
            b"declare namespace N { const implements: number; }",
        ),
        (
            "parse-error suppression",
            b"let broken = ; let \\u0069mplements = 1; implements;",
        ),
        // Run a clean source after the error/module cases to catch cached source
        // facts accidentally surviving the Binder that owns them.
        ("subsequent clean source", b"let value = 1; value;"),
    ] {
        let parsed = parse(bytes);
        let file = parsed.publish_unbound();
        let source = file.root().unwrap();
        let checked = crate::bind_source_file(&file, source).unwrap();
        let checked_view = checked.view();
        let expected_headers = syntax_headers(checked_view.ast(), source);
        let expected_diagnostics = diagnostics(checked_view.result().diagnostics(), source);

        let completed = crate::bind_parsed_file(parse(bytes)).unwrap();
        // Await reparsing can leave multiple source records before binding.
        // Those fixtures must still match through the existing fallback. Cases
        // below prove that this comparison also exercises the local backend.
        if matches!(
            name,
            "clean external module" | "property names" | "subsequent clean source"
        ) {
            assert!(
                completed.bound_with_local_scope(),
                "{name}: expected local scope"
            );
        }
        let actual = completed.view();
        assert_eq!(
            syntax_headers(actual.ast(), completed.source()),
            expected_headers,
            "{name}: bound flags"
        );
        assert_eq!(
            diagnostics(actual.result().diagnostics(), completed.source()),
            expected_diagnostics,
            "{name}: diagnostics"
        );
    }
}
