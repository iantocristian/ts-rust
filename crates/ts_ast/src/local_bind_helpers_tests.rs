use super::LocalBind;
use crate::{
    AstBuilder, EnumMemberData, Factory, FactoryMethods, JsString, NodeId, NodeKind,
    ParenthesizedExpressionData, SourceFileParseOptions, SyntaxKind as K,
};
use std::panic::{catch_unwind, AssertUnwindSafe};
use ts_arena::Counters;
use ts_jsstring::SourceText;

fn outcome<T>(operation: impl FnOnce() -> T) -> Result<T, String> {
    catch_unwind(AssertUnwindSafe(operation)).map_err(|error| {
        error
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| {
                error
                    .downcast_ref::<&str>()
                    .map(|message| (*message).to_owned())
            })
            .expect("source helper panics have string payloads")
    })
}

#[test]
fn semantic_getters_share_kind_shape_and_nil_contracts() {
    let mut build = AstBuilder::new(SourceText::default(), &Counters::new());
    let name = build.new_identifier(JsString::from_bytes(b"name".as_slice()));
    let annotation = build.new_token(K::NumberKeyword.into());
    let initializer = build.new_token(K::NullKeyword.into());
    let backing = build.node_slice(vec![Some(name), None]).unwrap();
    let list = build
        .new_list(ts_core::TextRange::new(0, 0), backing)
        .unwrap();
    let function = build.new_function_declaration_data(
        K::Unknown.into(),
        crate::FunctionDeclarationData {
            modifiers: Some(list),
            type_parameters: None,
            parameters: Some(list),
            r#type: Some(annotation),
            full_signature: None,
            asterisk_token: None,
            body: Some(name),
            name: Some(name),
        },
    );
    let parameter = build.new_parameter_declaration_data(
        K::Parameter.into(),
        crate::ParameterDeclarationData {
            modifiers: None,
            dot_dot_dot_token: None,
            name: Some(name),
            question_token: None,
            r#type: Some(annotation),
            initializer: Some(initializer),
        },
    );
    let mut nodes = vec![function, parameter, name];
    for kind in [
        K::Unknown,
        K::CallExpression,
        K::PropertyAccessExpression,
        K::PropertyDeclaration,
        K::Parameter,
        K::FunctionDeclaration,
        K::Block,
        K::DefaultClause,
        K::CaseClause,
        K::ForOfStatement,
        K::BinaryExpression,
        K::JSDocParameterTag,
    ] {
        nodes.push(build.new_token(kind.into()));
    }
    in_local_scope(build, |local| {
        for node in nodes {
            let read = local.node(local.import_node(node).unwrap());
            let public = local.general_node(node).unwrap();
            macro_rules! node_fields {
                ($($getter:ident),*) => {$(
                    assert_eq!(outcome(|| read.$getter().map(|node| local.node_id(node))),
                               outcome(|| public.$getter()), stringify!($getter));
                )*};
            }
            macro_rules! list_fields {
                ($($getter:ident),*) => {$(
                    assert_eq!(outcome(|| read.$getter()),
                               outcome(|| public.$getter().map(|list| local.import_list(list).unwrap())),
                               stringify!($getter));
                )*};
            }
            node_fields!(
                name,
                body,
                expression,
                type_node,
                initializer,
                property_name,
                label,
                attributes,
                statement,
                postfix_token,
                question_dot_token,
                module_specifier,
                import_clause,
                tag_name,
                type_expression,
                class_name
            );
            list_fields!(
                modifiers,
                parameter_list,
                argument_list,
                type_argument_list,
                type_parameter_list,
                member_list,
                statement_list,
                comment_list,
                children_list,
                property_list,
                element_list
            );
        }
        let read = local.node(local.import_node(function).unwrap());
        assert_eq!(read.body().map(|node| local.node_id(node)), Some(name));
        assert_eq!(
            read.type_node().map(|node| local.node_id(node)),
            Some(annotation)
        );
        assert_eq!(
            read.parameter_list(),
            Some(local.import_list(list).unwrap())
        );
        assert_eq!(
            local
                .node(local.import_node(parameter).unwrap())
                .initializer()
                .map(|node| local.node_id(node)),
            Some(initializer)
        );
    });
}

#[test]
fn locals_container_predicate_includes_source_roles_without_inline_locals_storage() {
    let mut build = AstBuilder::new(SourceText::default(), &Counters::new());
    let switch = build.new_switch_statement(None, None);
    let attempt = build.new_try_statement(None, None, None);
    let block = build.new_block(None, false);
    let token_block = build.new_token(K::Block.into());
    in_local_scope(build, |local| {
        for (node, expected) in [
            (switch, true),
            (attempt, true),
            (block, true),
            (token_block, false),
        ] {
            assert_eq!(
                local
                    .node(local.import_node(node).unwrap())
                    .is_locals_container(),
                expected
            );
            assert_eq!(
                crate::is_locals_container(&local.general_node(node).unwrap()),
                expected
            );
        }
    });
}

#[test]
fn shared_optional_logical_and_text_helpers_preserve_selected_reads() {
    use crate::{node_flags as nf, utilities as u};
    let mut build = AstBuilder::new(SourceText::default(), &Counters::new());
    let identifier = build.new_identifier(JsString::from_bytes(b"push".as_slice()));
    let keyword = build.new_token(K::ThisKeyword.into());
    let operator = build.new_token(K::QuestionQuestionToken.into());
    let logical =
        build.new_binary_expression(None, Some(identifier), None, Some(operator), Some(keyword));
    let wrapper = build.new_parenthesized_expression(Some(logical));
    let negated = build.new_prefix_unary_expression(K::ExclamationToken.into(), Some(wrapper));
    let unselected_nil = build.new_prefix_unary_expression(K::PlusToken.into(), None);
    let nil_binary = build.new_binary_expression(None, None, None, None, None);
    let nil_parenthesized = build.new_parenthesized_expression(None);
    let access = build.new_property_access_expression(
        Some(identifier),
        None,
        Some(identifier),
        nf::OPTIONAL_CHAIN,
    );
    let root = build.new_property_access_expression(
        Some(access),
        Some(operator),
        Some(identifier),
        nf::OPTIONAL_CHAIN,
    );
    build.node_mut(identifier).unwrap().set_parent(Some(access));
    build.node_mut(access).unwrap().set_parent(Some(root));
    let wrong_shape = build.new_token(K::PropertyAccessExpression.into());
    let unselected_shape = build.new_token(K::NonNullExpression.into());
    for node in [wrong_shape, unselected_shape] {
        build.node_mut(node).unwrap().set_flags(nf::OPTIONAL_CHAIN);
    }
    let nodes = [
        identifier,
        keyword,
        logical,
        wrapper,
        negated,
        unselected_nil,
        nil_binary,
        nil_parenthesized,
        access,
        root,
        wrong_shape,
        unselected_shape,
    ];
    in_local_scope(build, |local| {
        for node in nodes {
            let id = local.import_node(node).unwrap();
            macro_rules! view_helpers {
                ($($helper:ident),*) => {$(
                    assert_eq!(outcome(|| local.$helper(id)), outcome(|| u::$helper(local.view(), node).unwrap()), stringify!($helper));
                )*};
            }
            view_helpers!(
                is_outermost_optional_chain,
                is_expression_of_optional_chain_root,
                is_logical_expression,
                is_logical_or_coalescing_assignment_expression,
                is_nullish_coalesce
            );
            assert_eq!(
                outcome(|| local.is_optional_chain(id)),
                outcome(|| u::is_optional_chain(&local.general_node(node).unwrap()))
            );
            assert_eq!(
                outcome(|| local.is_optional_chain_root(id)),
                outcome(|| u::is_optional_chain_root(&local.general_node(node).unwrap()))
            );
            assert_eq!(
                outcome(|| local.is_dotted_name(id)),
                outcome(|| crate::is_dotted_name(local.view(), node).unwrap())
            );
            assert_eq!(
                outcome(|| local.node_id(local.skip_parentheses(id))),
                outcome(|| crate::skip_parentheses(local.view(), node).unwrap())
            );
            assert_eq!(
                outcome(|| local.is_push_or_unshift_identifier(id)),
                outcome(|| crate::is_push_or_unshift_identifier(local.view(), node).unwrap())
            );
        }
        assert!(local.is_logical_expression(local.import_node(negated).unwrap()));
        assert!(!local.is_logical_expression(local.import_node(unselected_nil).unwrap()));
        assert!(!local.is_optional_chain_root(local.import_node(unselected_shape).unwrap()));
        assert!(local.is_push_or_unshift_identifier(local.import_node(identifier).unwrap()));
        assert_eq!(
            outcome(|| local.is_logical_expression(local.import_node(nil_binary).unwrap())),
            Err("nil node in source AST utility".to_owned())
        );
        assert_eq!(outcome(|| local.is_optional_chain_root(local.import_node(wrong_shape).unwrap())), Err("interface conversion: ast.nodeData is *ast.Token, not *ast.PropertyAccessExpression".to_owned()));
    });
}

fn in_local_scope(build: AstBuilder, operation: impl for<'scope> FnOnce(LocalBind<'scope, '_>)) {
    let mut build = build;
    let source = build.new_source_file(
        SourceFileParseOptions {
            file_name: JsString::from_bytes(b"/local-helpers.ts".as_slice()),
            ..Default::default()
        },
        SourceText::from_loaded_bytes(b"".as_slice()),
        None,
        None,
    );
    let parsed = build.complete(source).unwrap();
    parsed
        .bind_and_publish(|builder| {
            builder
                .with_local_scope(operation)
                .expect("validated factory source enters local binding");
            Ok(())
        })
        .unwrap();
}

fn compare_helpers(local: &LocalBind<'_, '_>, node: NodeId) {
    let id = local.import_node(node).unwrap();
    assert_eq!(
        outcome(|| local.is_entity_name_expression(id)),
        outcome(|| crate::is_entity_name_expression(local.view(), node).unwrap()),
        "entity-name kind {:?}",
        local.node(id).kind()
    );
    assert_eq!(
        outcome(|| local.is_left_hand_side_expression(id)),
        outcome(|| crate::is_left_hand_side_expression(local.view(), node).unwrap()),
        "left-hand-side kind {:?}",
        local.node(id).kind()
    );
}

#[test]
fn kind_predicates_preserve_open_kinds_and_token_payloads() {
    let mut build = AstBuilder::new(SourceText::default(), &Counters::new());
    let nodes: Vec<_> = K::ALL
        .iter()
        .copied()
        .map(NodeKind::from)
        .chain([i16::MIN, -1, 351, 352, i16::MAX].map(NodeKind::from_raw))
        .map(|kind| build.new_token(kind))
        .collect();
    in_local_scope(build, |local| {
        for node in nodes {
            compare_helpers(&local, node);
        }
    });
}

#[test]
fn entity_names_preserve_name_first_failures_and_shape_independence() {
    let mut build = AstBuilder::new(SourceText::default(), &Counters::new());
    let identifier = build.new_token(K::Identifier.into());
    let private = build.new_private_identifier(JsString::from_bytes(b"#x".as_slice()));
    let this = build.new_token(K::ThisKeyword.into());
    let property =
        build.new_property_access_expression(Some(identifier), None, Some(identifier), 0);
    let chain = build.new_property_access_expression(Some(property), None, Some(identifier), 0);
    let parenthesized = build.new_parenthesized_expression(Some(identifier));
    let this_property = build.new_property_access_expression(Some(this), None, Some(identifier), 0);
    let element = build.new_element_access_expression(Some(identifier), None, Some(identifier), 0);
    let nil_name = build.new_property_access_expression(None, None, None, 0);
    let nil_expression = build.new_property_access_expression(None, None, Some(identifier), 0);
    let private_name = build.new_property_access_expression(None, None, Some(private), 0);
    let named_wrong_shape = build.new_enum_member_data(
        K::PropertyAccessExpression.into(),
        EnumMemberData {
            name: Some(identifier),
            initializer: None,
            modifiers: None,
            postfix_token: None,
        },
    );
    let private_wrong_shape = build.new_enum_member_data(
        K::PropertyAccessExpression.into(),
        EnumMemberData {
            name: Some(private),
            initializer: None,
            modifiers: None,
            postfix_token: None,
        },
    );
    let nil_wrong_shape = build.new_enum_member_data(
        K::PropertyAccessExpression.into(),
        EnumMemberData {
            name: None,
            initializer: None,
            modifiers: None,
            postfix_token: None,
        },
    );
    let nameless_wrong_shape = build.new_parenthesized_expression_data(
        K::PropertyAccessExpression.into(),
        ParenthesizedExpressionData {
            expression: Some(identifier),
        },
    );
    in_local_scope(build, |local| {
        for (node, expected) in [
            (identifier, Ok(true)),
            (property, Ok(true)),
            (chain, Ok(true)),
            (private, Ok(false)),
            (parenthesized, Ok(false)),
            (this, Ok(false)),
            (this_property, Ok(false)),
            (element, Ok(false)),
            (private_name, Ok(false)),
            (private_wrong_shape, Ok(false)),
            (nil_name, Err("nil node in source AST utility")),
            (nil_wrong_shape, Err("nil node in source AST utility")),
            (nameless_wrong_shape, Err("nil node in source AST utility")),
            (nil_expression, Err("nil entity-name expression")),
            (
                named_wrong_shape,
                Err("interface conversion: ast.nodeData is *ast.EnumMember, not *ast.PropertyAccessExpression"),
            ),
        ] {
            compare_helpers(&local, node);
            assert_eq!(
                outcome(|| local.is_entity_name_expression(local.import_node(node).unwrap())),
                expected.map_err(str::to_owned)
            );
        }
    });
}

#[test]
fn left_hand_side_skips_only_partially_emitted_wrappers() {
    let mut build = AstBuilder::new(SourceText::default(), &Counters::new());
    let identifier = build.new_token(K::Identifier.into());
    let binary = build.new_token(K::BinaryExpression.into());
    let parenthesized = build.new_parenthesized_expression(Some(binary));
    let nil = build.new_partially_emitted_expression(None);
    let wrong_shape = build.new_parenthesized_expression_data(
        K::PartiallyEmittedExpression.into(),
        ParenthesizedExpressionData {
            expression: Some(identifier),
        },
    );
    let mut deep = identifier;
    for _ in 0..4_096 {
        deep = build.new_partially_emitted_expression(Some(deep));
    }
    let wrapped_binary = build.new_partially_emitted_expression(Some(binary));
    let wrapped_parenthesized = build.new_partially_emitted_expression(Some(parenthesized));
    let missing = build.new_token(K::MissingDeclaration.into());
    let import = build.new_token(K::ImportKeyword.into());
    in_local_scope(build, |local| {
        for (node, expected) in [
            (deep, Ok(true)),
            (wrapped_binary, Ok(false)),
            (wrapped_parenthesized, Ok(true)),
            (missing, Ok(true)),
            (import, Ok(true)),
            (nil, Err("nil partially emitted expression")),
            (
                wrong_shape,
                Err("interface conversion: ast.nodeData is *ast.ParenthesizedExpression, not *ast.PartiallyEmittedExpression"),
            ),
        ] {
            compare_helpers(&local, node);
            assert_eq!(
                outcome(|| local.is_left_hand_side_expression(local.import_node(node).unwrap())),
                expected.map_err(str::to_owned)
            );
        }
    });
}
