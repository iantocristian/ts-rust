//! Source-program regressions for the bounded production checker. The two
//! assignment diagnostics below are the pinned Go P2 observations, including
//! scanner ranges and diagnostic payload, not a second expression evaluator.

use std::sync::Arc;
use ts_arena::{CheckerIdentity, Counters, Generation, NodeId};
use ts_checker::{CheckerOwner, Error};
use ts_compiler::{FileCache, Program, ProgramCheckerHost, ProgramOptions};
use ts_core::{CompilerOptions, ModuleKind, ScriptTarget, Tristate};
use ts_jsstring::JsString;

fn options() -> CompilerOptions {
    CompilerOptions {
        target: ScriptTarget::ESNEXT,
        module: ModuleKind::ESNEXT,
        strict: Tristate::TRUE,
        no_lib: Tristate::TRUE,
        ..Default::default()
    }
}

fn checker(text: &[u8], options: CompilerOptions) -> (Arc<CheckerOwner>, NodeId) {
    let (checker, program, _) = fixture(text, options);
    (checker, program.file(b"/main.ts").unwrap().source())
}

fn fixture(text: &[u8], options: CompilerOptions) -> (Arc<CheckerOwner>, Arc<Program>, Counters) {
    let counters = Counters::new();
    let generation = Generation::new(&counters);
    let mut fs = ts_vfs::MemoryBuilder::new(b"/", true);
    fs.insert_loaded(b"/main.ts", text);
    let program = Arc::new(
        Program::load(
            ProgramOptions {
                config: ts_tsoptions::ParsedCommandLine::new(
                    options,
                    vec![JsString::from_bytes(b"/main.ts".as_slice())],
                ),
                host: Arc::new(fs.finish()),
                current_directory: JsString::from_bytes(b"/".as_slice()),
                default_library_path: JsString::from_bytes(b"/no-default-lib".as_slice()),
                skip_module_resolution: false,
            },
            &mut FileCache::new(),
            &counters,
        )
        .unwrap(),
    );
    let checker = CheckerOwner::for_program(
        CheckerIdentity::new(generation, &counters),
        &counters,
        Arc::new(ProgramCheckerHost::new(program.clone())),
    )
    .unwrap();
    (Arc::new(checker), program, counters)
}

#[test]
fn source_assignment_diagnostics_match_pinned_native_ranges_and_payload_on_repeat() {
    let requests: serde_json::Value =
        serde_json::from_str(include_str!("../../../tools/s08/p2/requests.json")).unwrap();
    let request = requests["programs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|request| request["id"] == "variables-and-failing-assignment")
        .unwrap();
    let text = request["files"]["/variables.ts"].as_str().unwrap();
    let (checker, source) = checker(text.as_bytes(), options());
    let expected_ranges = [(96, 99), (119, 124)];
    let mut previous = None;
    for _ in 0..3 {
        let actual = checker
            .operation()
            .unwrap()
            .semantic_diagnostics(source)
            .unwrap();
        assert_eq!(actual.len(), 2);
        for (diagnostic, (pos, end)) in actual.iter().zip(expected_ranges) {
            assert_eq!(diagnostic.file, Some(source));
            assert_eq!((diagnostic.loc.pos(), diagnostic.loc.end()), (pos, end));
            assert_eq!(diagnostic.code, 2322);
            assert_eq!(diagnostic.category, 1);
            assert_eq!(
                diagnostic.message_key.as_bytes(),
                b"Type_0_is_not_assignable_to_type_1_2322"
            );
            assert_eq!(
                diagnostic
                    .message_args
                    .iter()
                    .map(JsString::as_bytes)
                    .collect::<Vec<_>>(),
                [b"string".as_slice(), b"number"]
            );
            assert!(diagnostic.message_text.is_empty());
            assert!(diagnostic.source.is_empty());
            assert!(diagnostic.message_chain.is_empty());
            assert!(diagnostic.related_information.is_empty());
            assert!(
                !diagnostic.reports_unnecessary
                    && !diagnostic.reports_deprecated
                    && !diagnostic.skipped_on_no_emit
            );
        }
        if let Some(previous) = previous {
            assert_eq!(actual, previous);
        }
        previous = Some(actual);
    }
}

#[test]
fn source_check_failure_after_a_diagnostic_stays_failed_across_operations() {
    let (checker, source) = checker(
        b"let before: number = \"wrong\"; type Later<T> = T;",
        options(),
    );
    for _ in 0..3 {
        assert_eq!(
            checker.operation().unwrap().semantic_diagnostics(source),
            Err(Error::Unsupported("checkTypeParameters"))
        );
    }
}

#[test]
fn source_check_rejects_unported_grammar_relations_and_options() {
    for (text, expected) in [
        (
            b"interface A { value?: number }".as_slice(),
            "checkVariableLikeDeclaration: optional declaration",
        ),
        (
            b"interface A { value: number; value: string }",
            "checkObjectTypeForDuplicateDeclarations/subsequent property declarations",
        ),
        (
            b"let value: { field: number } = { field: \"wrong\" };",
            "propertyRelatedTo: incompatible property diagnostic",
        ),
        (
            b"let value: { field: number } = {};",
            "reportUnmatchedProperty",
        ),
        (
            b"let value: { field: number } = { field: 1, extra: 2 };",
            "hasExcessProperties",
        ),
        (
            b"let value!: number;",
            "checkGrammarVariableDeclaration: definite assignment assertion",
        ),
        (
            b"// @ts-ignore\nlet value: number = \"wrong\";",
            "checkSourceFile: diagnostic directives",
        ),
    ] {
        let (checker, source) = checker(text, options());
        assert_eq!(
            checker.operation().unwrap().semantic_diagnostics(source),
            Err(Error::Unsupported(expected)),
            "source {text:?}"
        );
    }
    for options in [
        CompilerOptions {
            no_check: Tristate::TRUE,
            ..options()
        },
        CompilerOptions {
            no_unused_locals: Tristate::TRUE,
            ..options()
        },
        CompilerOptions {
            isolated_declarations: Tristate::TRUE,
            ..options()
        },
    ] {
        let (checker, source) = checker(b"let value: number = 1;", options);
        assert_eq!(
            checker.operation().unwrap().semantic_diagnostics(source),
            Err(Error::Unsupported(
                "checkSourceFile: noCheck/unused/isolated declaration options"
            ))
        );
    }
}

#[test]
fn source_check_accepts_plain_structural_assignments_and_nonstrict_nullable_boolean() {
    for (text, options) in [
        (b"type Record = { label: string; count: number }; let value: Record = { label: \"ok\", count: 1 };".as_slice(), options()),
        (b"let flag: boolean = null;", CompilerOptions { strict_null_checks: Tristate::FALSE, ..options() }),
    ] {
        let (checker, source) = checker(text, options);
        assert!(checker.operation().unwrap().semantic_diagnostics(source).unwrap().is_empty());
    }
}

fn declarations(program: &Program) -> Vec<NodeId> {
    let file = program.file(b"/main.ts").unwrap();
    let view = file.bound().view().ast();
    view.node_slice(view.node(file.source()).unwrap().statements(view).unwrap())
        .unwrap()
        .iter()
        .flatten()
        .collect()
}

fn declaration_name(program: &Program, declaration: NodeId) -> NodeId {
    program
        .file(b"/main.ts")
        .unwrap()
        .bound()
        .view()
        .node(declaration)
        .unwrap()
        .name()
        .unwrap()
}

#[test]
fn source_symbol_references_are_bound_to_the_exact_checker_even_when_source_is_shared() {
    let (first, program, counters) = fixture(b"interface Shared { field: string }", options());
    let second = Arc::new(
        CheckerOwner::for_program(
            CheckerIdentity::new(Generation::new(&counters), &counters),
            &counters,
            Arc::new(ProgramCheckerHost::new(program.clone())),
        )
        .unwrap(),
    );
    let name = declaration_name(&program, declarations(&program)[0]);
    let mut first_op = first.operation().unwrap();
    let first_symbol = first_op.get_symbol_at_location(name).unwrap().unwrap();
    let ty = first_op.get_declared_type_of_symbol(first_symbol).unwrap();
    let retained = first_op.retain_type(ty).unwrap();
    let mut second_op = second.operation().unwrap();
    let second_symbol = second_op.get_symbol_at_location(name).unwrap().unwrap();
    assert_eq!(
        first_symbol.id(),
        second_symbol.id(),
        "shared bound symbol identity"
    );
    assert_ne!(
        first_symbol, second_symbol,
        "checker association is not source identity"
    );
    assert!(matches!(
        second_op.get_declared_type_of_symbol(first_symbol),
        Err(Error::Arena(ts_arena::Error::WrongOwner))
    ));
    assert!(matches!(
        second_op.properties_of_type(ty),
        Err(Error::Arena(ts_arena::Error::WrongOwner))
    ));
    assert!(second_op.get_declared_type_of_symbol(second_symbol).is_ok());
    let weak = Arc::downgrade(&program);
    drop(first_op);
    drop(second_op);
    drop(second);
    drop(first);
    drop(program);
    assert!(weak.upgrade().is_some());
    drop(retained);
    assert!(weak.upgrade().is_none());
}

#[test]
fn failed_query_caches_and_resolution_stack_cannot_convert_failure_to_success() {
    let (owner, program, _) = fixture(b"interface Callable { (value: string): number } interface Generic<T> {} type Failed = string[]; type Good = number;", options());
    let declarations = declarations(&program);
    for _ in 0..3 {
        let mut op = owner.operation().unwrap();
        let symbol = op
            .get_symbol_at_location(declaration_name(&program, declarations[0]))
            .unwrap()
            .unwrap();
        let callable = op.get_declared_type_of_symbol(symbol).unwrap();
        assert!(matches!(
            op.properties_of_type(callable),
            Err(Error::Unsupported(
                "resolveDeclaredMembers: signatures/index infos"
            ))
        ));
        for &declaration in &declarations[1..3] {
            let symbol = op
                .get_symbol_at_location(declaration_name(&program, declaration))
                .unwrap()
                .unwrap();
            assert!(matches!(
                op.get_declared_type_of_symbol(symbol),
                Err(Error::Unsupported(_))
            ));
        }
        let symbol = op
            .get_symbol_at_location(declaration_name(&program, declarations[3]))
            .unwrap()
            .unwrap();
        let good = op.get_declared_type_of_symbol(symbol).unwrap();
        assert_eq!(op.type_to_string(good, 0).unwrap().as_bytes(), b"number");
    }
}

#[test]
fn literal_grammar_preserves_bigint_spelling_and_keeps_suggestions_out_of_errors() {
    let (owner, program, _) = fixture(b"type Hex = 0xffn; type Decimal = 255n; let large: number = -9007199254740993; let scientific: number = 9e99; let fractional: number = 9007199254740993.1; let value: bigint = 1n;", CompilerOptions { target: ScriptTarget::ES2019, ..options() });
    let source = program.file(b"/main.ts").unwrap().source();
    let decls = declarations(&program);
    let mut op = owner.operation().unwrap();
    let mut types = Vec::new();
    for &declaration in &decls[..2] {
        let symbol = op
            .get_symbol_at_location(declaration_name(&program, declaration))
            .unwrap()
            .unwrap();
        let ty = op.get_declared_type_of_symbol(symbol).unwrap();
        assert_eq!(op.type_to_string(ty, 0).unwrap().as_bytes(), b"255n");
        types.push(ty);
    }
    assert_eq!(types[0], types[1]);
    let errors = op.semantic_diagnostics(source).unwrap();
    assert_eq!(errors.iter().map(|d| d.code).collect::<Vec<_>>(), [2737]);
    let suggestions = op.recorded_suggestions(source).unwrap();
    assert_eq!(
        suggestions
            .iter()
            .map(|d| (d.code, d.category))
            .collect::<Vec<_>>(),
        [(80008, 2)]
    );
}

#[test]
fn an_outer_generic_interface_never_caches_a_plain_interface_type() {
    let (owner, program, _) = fixture(
        b"function outer<T>() { interface Nested { value: T } }",
        options(),
    );
    let view = program.file(b"/main.ts").unwrap().bound().view().ast();
    let function = declarations(&program)[0];
    let body = view.node(function).unwrap().body().unwrap();
    let nested = view
        .node_slice(view.node(body).unwrap().statements(view).unwrap())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let name = declaration_name(&program, nested);
    for _ in 0..2 {
        let mut op = owner.operation().unwrap();
        let symbol = op.get_symbol_at_location(name).unwrap().unwrap();
        assert_eq!(
            op.get_declared_type_of_symbol(symbol),
            Err(Error::Unsupported(
                "getOuterTypeParametersOfClassOrInterface"
            ))
        );
    }
}

#[test]
fn lazy_jsdoc_type_names_do_not_resolve_as_ordinary_wrapper_interfaces() {
    use ts_ast::JsDocProvider;
    let (owner, program, _) = fixture(
        b"interface String { tag: number }\n/** @type {String} */\nlet value: string = \"ok\";",
        options(),
    );
    let file = program.file(b"/main.ts").unwrap();
    let view = file.bound().view().ast();
    let statement = declarations(&program)[1];
    let roots = ts_parser::ParserJsDocProvider::default()
        .jsdoc(view, file.source(), statement)
        .unwrap();
    let doc = view.node(roots[0]).unwrap();
    let tags = doc.data_source().as_js_doc().unwrap().tags().unwrap();
    let tag = view
        .node_slice(view.list(tags).unwrap().nodes())
        .unwrap()
        .at(0)
        .unwrap();
    let expression = view
        .node(tag)
        .unwrap()
        .data_source()
        .as_js_doc_type_tag()
        .unwrap()
        .type_expression()
        .unwrap();
    let reference = view.node(expression).unwrap().type_node().unwrap();
    assert_eq!(
        view.node(reference).unwrap().kind(),
        ts_ast::SyntaxKind::TypeReference
    );
    assert_ne!(
        reference.arena(),
        file.source().arena(),
        "exercises retained lazy owner routing"
    );
    for _ in 0..2 {
        assert_eq!(
            owner.operation().unwrap().get_type_at_location(reference),
            Err(Error::Unsupported("getIntendedTypeFromJSDocTypeReference"))
        );
    }
}
