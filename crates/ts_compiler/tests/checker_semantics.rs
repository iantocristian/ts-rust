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
        b"let before: number = \"wrong\"; type Later = typeof before;",
        options(),
    );
    for _ in 0..3 {
        assert_eq!(
            checker.operation().unwrap().semantic_diagnostics(source),
            Err(Error::Unsupported(
                "checkSourceElementWorker: statement/type family"
            ))
        );
    }
}

#[test]
fn source_check_rejects_unported_grammar_relations_and_options() {
    for (text, expected) in [
        (
            b"interface A { value: number; value: string }".as_slice(),
            "checkObjectTypeForDuplicateDeclarations/subsequent property declarations",
        ),
        (
            b"let value: { field: number } = { field: 1, extra: 2 };",
            "report excess properties: source object expression",
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
fn source_check_reports_structural_assignment_failures() {
    // Pinned Go emits no chain for either shape: the elaborated property
    // mismatch carries a TS6500 note, and the missing property carries TS2728.
    for (text, code, range, note_code, note_range, note_args) in [
        (
            b"let value: { field: number } = { field: \"wrong\" };".as_slice(),
            2322,
            (33, 38),
            6500,
            (13, 18),
            vec!["field".to_string(), "{ field: number; }".to_string()],
        ),
        (
            b"let value: { field: number } = {};".as_slice(),
            2741,
            (4, 9),
            2728,
            (13, 18),
            vec!["field".to_string()],
        ),
    ] {
        let (owner, source) = checker(text, options());
        for _ in 0..2 {
            let diagnostics = owner
                .operation()
                .unwrap()
                .semantic_diagnostics(source)
                .unwrap();
            assert_eq!(diagnostics.len(), 1);
            assert_eq!(diagnostics[0].code, code);
            assert_eq!(
                (diagnostics[0].loc.pos(), diagnostics[0].loc.end()),
                range,
                "diagnostic range for {text:?}"
            );
            assert!(diagnostics[0].message_chain.is_empty());
            assert_eq!(diagnostics[0].related_information.len(), 1);
            let note = &diagnostics[0].related_information[0];
            assert_eq!(note.code, note_code);
            assert_eq!((note.loc.pos(), note.loc.end()), note_range);
            assert_eq!(
                note.message_args
                    .iter()
                    .map(|argument| String::from_utf8_lossy(argument.as_bytes()).into_owned())
                    .collect::<Vec<_>>(),
                note_args
            );
        }
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
    let (owner, program, _) = fixture(b"interface Callable { (value: string): number } interface Generic<T> {} type Failed = typeof value; type Good = number;", options());
    let declarations = declarations(&program);
    for _ in 0..3 {
        let mut op = owner.operation().unwrap();
        let symbol = op
            .get_symbol_at_location(declaration_name(&program, declarations[0]))
            .unwrap()
            .unwrap();
        let callable = op.get_declared_type_of_symbol(symbol).unwrap();
        assert!(op.properties_of_type(callable).unwrap().is_empty());
        let generic_symbol = op
            .get_symbol_at_location(declaration_name(&program, declarations[1]))
            .unwrap()
            .unwrap();
        let generic = op.get_declared_type_of_symbol(generic_symbol).unwrap();
        assert_ne!(
            op.type_object_flags(generic).unwrap() & ts_checker::object_flags::REFERENCE,
            0
        );
        for &declaration in &declarations[2..3] {
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
fn an_outer_generic_interface_retains_outer_parameters_on_repeated_queries() {
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
        let ty = op.get_declared_type_of_symbol(symbol).unwrap();
        assert_ne!(
            op.type_object_flags(ty).unwrap() & ts_checker::object_flags::REFERENCE,
            0
        );
        let property = op.properties_of_type(ty).unwrap()[0];
        let value = op.get_type_of_symbol(property).unwrap();
        assert_eq!(
            op.type_flags(value).unwrap(),
            ts_checker::type_flags::TYPE_PARAMETER
        );
        assert_eq!(op.type_to_string(value, 0).unwrap().as_bytes(), b"T");
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

#[test]
fn readonly_and_const_grammar_errors_do_not_cascade_into_unrelated_checks() {
    for (text, codes) in [
        (b"const a = 1; a = 2;".as_slice(), vec![2588]),
        (b"const let: number;", vec![1155]),
        // An ordinary let declaration still runs its name grammar check.
        (b"let let: number;", vec![2480]),
    ] {
        let (owner, source) = checker(text, options());
        for _ in 0..2 {
            let actual = owner
                .operation()
                .unwrap()
                .semantic_diagnostics(source)
                .unwrap();
            assert_eq!(actual.iter().map(|d| d.code).collect::<Vec<_>>(), codes);
        }
    }
    let (owner, source) = checker(
        b"const a = 1; a = 1n;",
        CompilerOptions {
            target: ScriptTarget::ES2019,
            ..options()
        },
    );
    let diagnostics = owner
        .operation()
        .unwrap()
        .semantic_diagnostics(source)
        .unwrap();
    assert_eq!(
        diagnostics.iter().map(|d| d.code).collect::<Vec<_>>(),
        [2588, 2737],
        "a readonly left side must not suppress right-side grammar errors"
    );
}

#[test]
fn reference_identifier_and_whole_reference_preserve_distinct_native_queries() {
    let (owner, program, _) = fixture(b"type Foo = number; type Alias = Foo;", options());
    let view = program.file(b"/main.ts").unwrap().bound().view().ast();
    let declaration = declarations(&program)[1];
    let reference = view.node(declaration).unwrap().type_node().unwrap();
    let name = view
        .node(reference)
        .unwrap()
        .data_source()
        .as_type_reference_node()
        .unwrap()
        .type_name()
        .unwrap();
    let mut op = owner.operation().unwrap();
    assert_eq!(
        op.get_type_at_location(name).unwrap(),
        op.builtin_type("errorType").unwrap()
    );
    assert_eq!(
        op.get_type_at_location(reference).unwrap(),
        op.builtin_type("numberType").unwrap()
    );
    let symbol = op.get_symbol_at_location(name).unwrap().unwrap();
    assert_eq!(
        op.get_declared_type_of_symbol(symbol).unwrap(),
        op.builtin_type("numberType").unwrap()
    );
}

#[test]
fn unsupported_variable_widening_does_not_rebuild_a_successful_cached_initializer() {
    let (owner, program, _) = fixture(b"let value = { field: 1 };", options());
    let file = program.file(b"/main.ts").unwrap();
    let view = file.bound().view().ast();
    let statement = declarations(&program)[0];
    let list = view
        .node(statement)
        .unwrap()
        .data_source()
        .as_variable_statement()
        .unwrap()
        .declaration_list()
        .unwrap();
    let declarations = view
        .node(list)
        .unwrap()
        .data_source()
        .as_variable_declaration_list()
        .unwrap()
        .declarations()
        .unwrap();
    let declaration = view
        .node_slice(view.list(declarations).unwrap().nodes())
        .unwrap()
        .at(0)
        .unwrap();
    let name = view.node(declaration).unwrap().name().unwrap();
    let mut op = owner.operation().unwrap();
    let symbol = op.get_symbol_at_location(name).unwrap().unwrap();
    let unsupported = Err(Error::Unsupported(
        "widenTypeForVariableLikeDeclaration: auto/null/object widening",
    ));
    assert_eq!(op.get_type_of_symbol(symbol), unsupported);
    let before = (op.type_count(), op.symbol_count());
    for _ in 0..2 {
        assert_eq!(op.get_type_of_symbol(symbol), unsupported);
        assert_eq!(
            (op.type_count(), op.symbol_count()),
            before,
            "retrying widening must reuse the checked initializer's type and property symbols"
        );
    }
    assert!(matches!(
        op.semantic_diagnostics(file.source()),
        Err(Error::Unsupported(_))
    ));
    assert_eq!((op.type_count(), op.symbol_count()), before);
}

#[test]
fn union_property_normalization_is_deferred_and_repeated_identity_is_stable() {
    let (owner, program, _) = fixture(b"type A = { value: 'a' }; type B = { value: 'b' }; type C = { value: 'c' }; type U = A | B | C;", options());
    let declarations = declarations(&program);
    let mut op = owner.operation().unwrap();
    // Resolve the three property types first. Discovering U's property should
    // create only its deferred symbol, with no normalized value union yet.
    for &declaration in &declarations[..3] {
        let symbol = op
            .get_symbol_at_location(declaration_name(&program, declaration))
            .unwrap()
            .unwrap();
        let ty = op.get_declared_type_of_symbol(symbol).unwrap();
        let property = op.properties_of_type(ty).unwrap()[0];
        op.get_type_of_symbol(property).unwrap();
    }
    let symbol = op
        .get_symbol_at_location(declaration_name(&program, declarations[3]))
        .unwrap()
        .unwrap();
    let union = op.get_declared_type_of_symbol(symbol).unwrap();
    let before = op.type_count();
    let properties = op.properties_of_type(union).unwrap();
    assert_eq!(properties.len(), 1);
    assert_eq!(
        op.type_count(),
        before,
        "discovery must defer the property type union"
    );
    assert_ne!(
        op.symbol(properties[0]).unwrap().check_flags() & ts_ast::check_flags::DEFERRED_TYPE,
        0
    );
    let ty = op.get_type_of_symbol(properties[0]).unwrap();
    assert_eq!(op.type_count(), before + 1);
    assert_eq!(
        op.type_to_string(ty, 0).unwrap().as_bytes(),
        b"\"a\" | \"b\" | \"c\""
    );
    let counts = (op.type_count(), op.symbol_count());
    for _ in 0..3 {
        assert_eq!(op.properties_of_type(union).unwrap(), properties);
        assert_eq!(op.get_type_of_symbol(properties[0]).unwrap(), ty);
        assert_eq!((op.type_count(), op.symbol_count()), counts);
    }
}

#[test]
fn union_property_failure_does_not_publish_a_partial_property_list() {
    let (owner, program, _) = fixture(b"type A = { good: string; bad: typeof missingA }; type B = { good: number; bad: typeof missingB }; type U = A | B;", options());
    let name = declaration_name(&program, declarations(&program)[2]);
    let mut op = owner.operation().unwrap();
    let symbol = op.get_symbol_at_location(name).unwrap().unwrap();
    let ty = op.get_declared_type_of_symbol(symbol).unwrap();
    let mut previous_counts = None;
    for _ in 0..3 {
        assert!(matches!(
            op.properties_of_type(ty),
            Err(Error::Unsupported("getTypeFromTypeNodeWorker: type family"))
        ));
        let counts = (op.type_count(), op.symbol_count());
        if let Some(previous) = previous_counts {
            assert_eq!(counts, previous);
        }
        previous_counts = Some(counts);
    }
}

#[test]
fn recursive_union_properties_keep_the_original_type_on_a_small_stack() {
    std::thread::Builder::new().stack_size(512 * 1024).spawn(|| {
        let (owner, program, _) = fixture(b"type A = { next: U; a: string }; type B = { next: U; b: number }; type U = A | B;", options());
        let mut op = owner.operation().unwrap();
        let symbol = op.get_symbol_at_location(declaration_name(&program, declarations(&program)[2])).unwrap().unwrap();
        let union = op.get_declared_type_of_symbol(symbol).unwrap();
        let properties = op.properties_of_type(union).unwrap();
        assert_eq!(properties.len(), 1);
        assert_eq!(op.symbol(properties[0]).unwrap().name_bytes(), b"next");
        assert_eq!(op.get_type_of_symbol(properties[0]).unwrap(), union);
        assert_eq!(op.properties_of_type(union).unwrap(), properties);
        assert!(op.semantic_diagnostics(program.file(b"/main.ts").unwrap().source()).unwrap().is_empty());
    }).unwrap().join().unwrap();
}

#[test]
fn primitive_union_diagnostics_preserve_literal_target_spelling() {
    let (owner, source) = checker(b"type Allowed = string | number; let good: Allowed = 1; let bad: Allowed = true; let wrong: 'a' | 'b' = 'c';", options());
    let diagnostics = owner
        .operation()
        .unwrap()
        .semantic_diagnostics(source)
        .unwrap();
    assert_eq!(
        diagnostics.iter().map(|d| d.code).collect::<Vec<_>>(),
        [2322, 2322]
    );
    assert_eq!(
        diagnostics[0]
            .message_args
            .iter()
            .map(JsString::as_bytes)
            .collect::<Vec<_>>(),
        [b"boolean".as_slice(), b"Allowed"]
    );
    assert_eq!(
        diagnostics[1]
            .message_args
            .iter()
            .map(JsString::as_bytes)
            .collect::<Vec<_>>(),
        [b"\"c\"".as_slice(), b"\"a\" | \"b\""]
    );
    let (owner, source) = checker(
        b"type A = { value?: number }; type B = { value: string }; type U = A | B;",
        options(),
    );
    assert!(owner
        .operation()
        .unwrap()
        .semantic_diagnostics(source)
        .unwrap()
        .is_empty());
}

#[test]
fn compound_constituents_are_checked_even_after_reduction_and_on_retry() {
    for text in [
        "type U = { a: string; a: number } | string;",
        "type U = string | { a: string; a: number };",
        "type U = unknown | ({ a: string; a: number } | string);",
        "type U = { a: Missing } | string;",
        "type U = unknown | { a: Missing };",
        "type U = unknown & { a: string; a: number };",
        "type U = never & { a: Missing };",
    ] {
        let (owner, source) = checker(text.as_bytes(), options());
        let first = owner.operation().unwrap().semantic_diagnostics(source);
        assert!(
            matches!(first, Err(Error::Unsupported(_))),
            "{text}: {first:?}"
        );
        for _ in 0..2 {
            assert_eq!(
                owner.operation().unwrap().semantic_diagnostics(source),
                first
            );
        }
    }
    // Checking follows source order, before construction sorts/reduces types.
    for (text, expected) in [
        (
            "type U = { a: string; a: number } | typeof missing;",
            "checkObjectTypeForDuplicateDeclarations/subsequent property declarations",
        ),
        (
            "type U = typeof missing | { a: string; a: number };",
            "checkSourceElementWorker: statement/type family",
        ),
    ] {
        let (owner, source) = checker(text.as_bytes(), options());
        assert_eq!(
            owner.operation().unwrap().semantic_diagnostics(source),
            Err(Error::Unsupported(expected))
        );
    }
    let (owner, source) = checker(b"type U = { a: string } | number;", options());
    assert!(owner
        .operation()
        .unwrap()
        .semantic_diagnostics(source)
        .unwrap()
        .is_empty());
}

#[test]
fn a_unique_symbol_widens_in_a_mutable_object_literal_location() {
    // Upstream widens a unique symbol outside its const-like declaration, so a
    // later declaration serialization never reaches an inaccessible one.
    let (owner, program, _) = fixture(
        b"declare function Symbol(): symbol; const key = Symbol(); const inner = { key }; type Probe = typeof inner;",
        options(),
    );
    let mut op = owner.operation().unwrap();
    let symbol = op
        .get_symbol_at_location(declaration_name(&program, declarations(&program)[3]))
        .unwrap()
        .unwrap();
    let ty = op.get_declared_type_of_symbol(symbol).unwrap();
    assert_eq!(
        op.type_to_string(ty, 0).unwrap().as_bytes(),
        b"{ key: symbol; }"
    );
}

#[test]
fn an_incompatible_property_reports_the_index_signature_wrapper_chain() {
    // A fresh object literal elaborates to the offending property instead, so
    // the wrapper is only observable through an already-typed source.
    let (owner, source) = checker(
        b"interface Target { [key: string]: string }\ndeclare const source: { type: number };\nconst check: Target = source;",
        options(),
    );
    let diagnostics = owner
        .operation()
        .unwrap()
        .semantic_diagnostics(source)
        .unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, 2322);
    let wrapper = diagnostics[0]
        .message_chain
        .first()
        .expect("index signature wrapper");
    assert_eq!(wrapper.code, 2530);
    assert_eq!(
        wrapper
            .message_args
            .first()
            .map(|argument| argument.as_bytes().to_vec()),
        Some(b"type".to_vec())
    );
    assert_eq!(
        wrapper.message_chain.first().map(|inner| inner.code),
        Some(2322)
    );
}

#[test]
fn intersection_discriminant_reduction_is_lazy_and_raw_display_is_available() {
    use ts_checker::{object_flags as of, type_format_flags as ff};
    let (owner, program, _) = fixture(
        b"type A = { tag: 'a' }; type B = { tag: 'b' }; type I = A & B;",
        options(),
    );
    let mut op = owner.operation().unwrap();
    let symbol = op
        .get_symbol_at_location(declaration_name(&program, declarations(&program)[2]))
        .unwrap()
        .unwrap();
    let ty = op.get_declared_type_of_symbol(symbol).unwrap();
    assert_eq!(
        op.type_object_flags(ty).unwrap() & of::IS_NEVER_INTERSECTION_COMPUTED,
        0
    );
    assert_eq!(
        op.type_to_string(ty, ff::NO_TYPE_REDUCTION | ff::IN_TYPE_ALIAS)
            .unwrap()
            .as_bytes(),
        b"A & B"
    );
    assert_eq!(
        op.type_object_flags(ty).unwrap() & of::IS_NEVER_INTERSECTION_COMPUTED,
        0
    );
    assert_eq!(op.type_to_string(ty, 0).unwrap().as_bytes(), b"never");
    assert_ne!(
        op.type_object_flags(ty).unwrap() & of::IS_NEVER_INTERSECTION,
        0
    );
    assert!(op.properties_of_type(ty).unwrap().is_empty());
    let counts = (op.type_count(), op.symbol_count());
    for _ in 0..3 {
        assert_eq!(op.type_to_string(ty, 0).unwrap().as_bytes(), b"never");
        assert_eq!(
            op.type_to_string(ty, ff::NO_TYPE_REDUCTION | ff::IN_TYPE_ALIAS)
                .unwrap()
                .as_bytes(),
            b"A & B"
        );
        assert_eq!((op.type_count(), op.symbol_count()), counts);
    }
}

#[test]
fn failed_intersection_reduction_clears_its_computed_flag_on_every_retry() {
    let (owner, program, _) = fixture(b"type A = { good: string; bad: typeof missingA }; type B = { good: number; bad: typeof missingB }; type I = A & B;", options());
    let mut op = owner.operation().unwrap();
    let symbol = op
        .get_symbol_at_location(declaration_name(&program, declarations(&program)[2]))
        .unwrap()
        .unwrap();
    let ty = op.get_declared_type_of_symbol(symbol).unwrap();
    let mut counts = None;
    for _ in 0..3 {
        assert!(matches!(
            op.properties_of_type(ty),
            Err(Error::Unsupported("getTypeFromTypeNodeWorker: type family"))
        ));
        assert!(matches!(
            op.type_to_string(ty, 0),
            Err(Error::Unsupported("getTypeFromTypeNodeWorker: type family"))
        ));
        assert_eq!(
            op.type_object_flags(ty).unwrap()
                & ts_checker::object_flags::IS_NEVER_INTERSECTION_COMPUTED,
            0
        );
        let current = (op.type_count(), op.symbol_count());
        if let Some(counts) = counts {
            assert_eq!(current, counts);
        }
        counts = Some(current);
    }
}

#[test]
fn deferred_intersection_property_builds_an_intersection_and_caches_its_identity() {
    let (owner, program, _) = fixture(b"type PA = { a: string }; type PB = { b: number }; type PC = { c: boolean }; type A = { value: PA }; type B = { value: PB }; type C = { value: PC }; type I = A & B & C;", options());
    let declarations = declarations(&program);
    let mut op = owner.operation().unwrap();
    for &declaration in &declarations[3..6] {
        let symbol = op
            .get_symbol_at_location(declaration_name(&program, declaration))
            .unwrap()
            .unwrap();
        let ty = op.get_declared_type_of_symbol(symbol).unwrap();
        let property = op.properties_of_type(ty).unwrap()[0];
        op.get_type_of_symbol(property).unwrap();
    }
    let symbol = op
        .get_symbol_at_location(declaration_name(&program, declarations[6]))
        .unwrap()
        .unwrap();
    let ty = op.get_declared_type_of_symbol(symbol).unwrap();
    let before = op.type_count();
    let properties = op.properties_of_type(ty).unwrap();
    assert_eq!(properties.len(), 1);
    assert_ne!(
        op.symbol(properties[0]).unwrap().check_flags() & ts_ast::check_flags::DEFERRED_TYPE,
        0
    );
    assert_eq!(
        op.type_count(),
        before,
        "property discovery must defer normalization"
    );
    let value = op.get_type_of_symbol(properties[0]).unwrap();
    assert_eq!(op.type_count(), before + 1);
    assert_eq!(
        op.type_flags(value).unwrap(),
        ts_checker::type_flags::INTERSECTION
    );
    assert_eq!(
        op.type_to_string(value, 0).unwrap().as_bytes(),
        b"PA & PB & PC"
    );
    let counts = (op.type_count(), op.symbol_count());
    for _ in 0..3 {
        assert_eq!(op.properties_of_type(ty).unwrap(), properties);
        assert_eq!(op.get_type_of_symbol(properties[0]).unwrap(), value);
        assert_eq!((op.type_count(), op.symbol_count()), counts);
    }
}

#[test]
fn recursive_intersection_properties_preserve_order_and_identity_on_a_small_stack() {
    std::thread::Builder::new().stack_size(512 * 1024).spawn(|| {
        let (owner, program, _) = fixture(b"type A = { next: I; a: string }; type B = { next: I; b: number }; type I = A & B;", options());
        let mut op = owner.operation().unwrap();
        let symbol = op.get_symbol_at_location(declaration_name(&program, declarations(&program)[2])).unwrap().unwrap();
        let ty = op.get_declared_type_of_symbol(symbol).unwrap();
        let properties = op.properties_of_type(ty).unwrap();
        let names: Vec<_> = properties.iter().map(|&prop| op.symbol(prop).unwrap().name_bytes().to_vec()).collect();
        assert_eq!(names, [b"next".to_vec(), b"a".to_vec(), b"b".to_vec()]);
        // Native displays next as A & B, not I: intersection construction
        // flattens I & I and interns an unaliased A & B result.
        let next = op.get_type_of_symbol(properties[0]).unwrap();
        assert_ne!(next, ty);
        assert_eq!(op.type_to_string(next, 0).unwrap().as_bytes(), b"A & B");
        assert_eq!(op.get_type_of_symbol(properties[0]).unwrap(), next);
        assert_eq!(op.properties_of_type(ty).unwrap(), properties);
        assert!(op.semantic_diagnostics(program.file(b"/main.ts").unwrap().source()).unwrap().is_empty());
    }).unwrap().join().unwrap();
}

#[test]
fn delete_operands_must_be_optional_writable_property_references() {
    let (owner, source) = checker(
        b"declare const o: { a?: number; b: number; readonly c?: number };\ndelete o.a;\ndelete o.b;\ndelete o.c;\ndelete 1;\n",
        options(),
    );
    let diagnostics = owner
        .operation()
        .unwrap()
        .semantic_diagnostics(source)
        .unwrap();
    let codes: Vec<i32> = diagnostics.iter().map(|d| d.code).collect();
    assert_eq!(
        codes,
        vec![
            ts_diagnostics::The_operand_of_a_delete_operator_must_be_optional.code,
            ts_diagnostics::The_operand_of_a_delete_operator_cannot_be_a_read_only_property.code,
            ts_diagnostics::The_operand_of_a_delete_operator_must_be_a_property_reference.code,
        ]
    );
}

#[test]
fn meta_properties_need_their_containers_and_module_targets() {
    let (owner, source) = checker(
        b"const outer = new.target;\nfunction f() { return new.target; }\n",
        options(),
    );
    let diagnostics = owner
        .operation()
        .unwrap()
        .semantic_diagnostics(source)
        .unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        ts_diagnostics::Meta_property_0_is_only_allowed_in_the_body_of_a_function_declaration_function_expression_or_constructor.code
    );
    assert_eq!(
        (diagnostics[0].loc.pos(), diagnostics[0].loc.end()),
        (14, 24),
        "the diagnostic spans `new.target` without its leading trivia"
    );
    let mut es2015 = options();
    es2015.module = ModuleKind::ES2015;
    let (owner, source) = checker(b"const m = import.meta;\n", es2015);
    let diagnostics = owner
        .operation()
        .unwrap()
        .semantic_diagnostics(source)
        .unwrap();
    let codes: Vec<i32> = diagnostics.iter().map(|d| d.code).collect();
    assert_eq!(
        codes,
        vec![ts_diagnostics::The_import_meta_meta_property_is_only_allowed_when_the_module_option_is_es2020_es2022_esnext_system_node16_node18_node20_or_nodenext.code]
    );
}

#[test]
fn regular_expression_literals_report_grammar_errors_once() {
    let (owner, source) = checker(b"const a = /x/gg;\nconst b = /y/i;\n", options());
    for _ in 0..2 {
        let diagnostics = owner
            .operation()
            .unwrap()
            .semantic_diagnostics(source)
            .unwrap();
        let codes: Vec<i32> = diagnostics.iter().map(|d| d.code).collect();
        assert_eq!(
            codes,
            vec![ts_diagnostics::Duplicate_regular_expression_flag.code],
            "one flag error for the duplicated `g`, none for `/y/i`"
        );
    }
}

#[test]
fn debugger_statements_in_ambient_blocks_report_once_per_block() {
    let (owner, source) = checker(
        b"declare namespace N { debugger; debugger; }\ndeclare namespace M { debugger; }\n",
        options(),
    );
    let diagnostics = owner
        .operation()
        .unwrap()
        .semantic_diagnostics(source)
        .unwrap();
    let codes: Vec<i32> = diagnostics.iter().map(|d| d.code).collect();
    assert_eq!(
        codes,
        vec![
            ts_diagnostics::Statements_are_not_allowed_in_ambient_contexts.code,
            ts_diagnostics::Statements_are_not_allowed_in_ambient_contexts.code,
        ]
    );
}
