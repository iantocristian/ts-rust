use super::LocalBind;
use crate::{
    AstBuilder, EnumMemberData, Factory, FactoryMethods, JsString, NodeId, NodeKind,
    ParenthesizedExpressionData, SourceFileParseOptions, SyntaxKind as K,
};
use std::panic::{catch_unwind, AssertUnwindSafe};
use ts_arena::Counters;
use ts_jsstring::SourceText;

fn outcome(operation: impl FnOnce() -> bool) -> Result<bool, String> {
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
