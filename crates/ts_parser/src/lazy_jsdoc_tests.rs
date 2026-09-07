use crate::ParserJsDocProvider;
use std::sync::Arc;
use ts_ast::{node_flags, JsDocProvider, JsString, NodeId, ParsedFile, SourceFileParseOptions};
use ts_core::ScriptKind;
use ts_jsstring::SourceText;

fn parse(name: &[u8], text: &[u8], kind: ScriptKind) -> ParsedFile {
    crate::parse_source_file(
        SourceText::from_loaded_bytes(text),
        kind,
        SourceFileParseOptions {
            file_name: JsString::from_bytes(name),
            ..SourceFileParseOptions::default()
        },
    )
}
fn first_statement(file: &ParsedFile) -> NodeId {
    let view = file.view();
    let list = view
        .node(file.root())
        .unwrap()
        .data()
        .as_source_file()
        .unwrap()
        .statements
        .unwrap();
    view.node_slice(view.list(list).unwrap().nodes()).unwrap()[0].unwrap()
}

#[test]
fn lazy_jsdoc_publishes_once_without_changing_source_parse_counts_or_diagnostics() {
    let file = parse(b"/lazy.ts", b"/** Hello */ const x = 1;", ScriptKind::TS);
    let parent = first_statement(&file);
    let root = file.root();
    let view = file.view();
    assert_ne!(
        view.node(parent).unwrap().flags() & node_flags::HAS_JS_DOC,
        0
    );
    assert!(view.source_eager_jsdoc(root, parent).unwrap().is_none());
    let before = view.source_file(root).unwrap();
    let counts = (
        before.node_count,
        before.text_count,
        before.identifier_count,
    );
    let diagnostic_counts = (before.diagnostics.len(), before.jsdoc_diagnostics.len());
    let mut provider = ParserJsDocProvider::default();
    let roots = provider.jsdoc(view, root, parent).unwrap();
    assert_eq!(roots.len(), 1);
    assert_eq!(view.node(roots[0]).unwrap().parent(), Some(parent));
    assert_eq!(view.node(roots[0]).unwrap().range().pos(), 0);
    assert_eq!(
        provider.jsdoc(view, root, parent).unwrap().as_ref(),
        roots.as_ref()
    );
    assert_eq!(
        view.source_eager_jsdoc(root, parent)
            .unwrap()
            .unwrap()
            .as_ref(),
        roots.as_ref()
    );
    let after = view.source_file(root).unwrap();
    assert_eq!(
        (after.node_count, after.text_count, after.identifier_count),
        counts
    );
    assert_eq!(
        (after.diagnostics.len(), after.jsdoc_diagnostics.len()),
        diagnostic_counts
    );
}

#[test]
fn ordinary_nodes_and_empty_flagged_comments_have_distinct_cache_paths() {
    let mut file = parse(b"/empty.ts", b"const x = 1;", ScriptKind::TS);
    let parent = first_statement(&file);
    let root = file.root();
    let mut provider = ParserJsDocProvider::default();
    assert!(provider
        .jsdoc(file.view(), root, parent)
        .unwrap()
        .is_empty());
    assert!(file
        .view()
        .source_eager_jsdoc(root, parent)
        .unwrap()
        .is_none());
    let node = file.builder_mut().node_mut(parent).unwrap();
    node.set_flags(node.flags() | node_flags::HAS_JS_DOC);
    assert!(provider
        .jsdoc(file.view(), root, parent)
        .unwrap()
        .is_empty());
    assert!(file
        .view()
        .source_eager_jsdoc(root, parent)
        .unwrap()
        .unwrap()
        .is_empty());
}

#[test]
fn published_lazy_jsdoc_races_return_the_same_published_ids() {
    let file = parse(
        b"/racing.ts",
        b"/** @param {number} x */ function f(x) {}",
        ScriptKind::TS,
    );
    let parent = first_statement(&file);
    let root = file.root();
    let file = Arc::new(file.publish_unbound());
    let results = std::thread::scope(|scope| {
        let mut workers = Vec::new();
        for _ in 0..4 {
            let file = file.clone();
            workers.push(scope.spawn(move || {
                let mut provider = ParserJsDocProvider::default();
                provider.jsdoc(file.view(), root, parent).unwrap()
            }));
        }
        workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert_eq!(results[0].len(), 1);
    for roots in &results[1..] {
        assert_eq!(roots.as_ref(), results[0].as_ref());
    }
}

#[test]
fn eager_javascript_docs_are_reused_by_the_same_provider() {
    let file = parse(b"/eager.js", b"/** Hello */ const x = 1;", ScriptKind::JS);
    let parent = first_statement(&file);
    let root = file.root();
    let cached = file
        .view()
        .source_eager_jsdoc(root, parent)
        .unwrap()
        .unwrap();
    let mut provider = ParserJsDocProvider::default();
    assert_eq!(
        provider.jsdoc(file.view(), root, parent).unwrap().as_ref(),
        cached.as_ref()
    );
}
