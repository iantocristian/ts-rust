//! Scoped typed-row borrows leave graph references in their local namespaces.

use serde_json::Value;
use std::collections::BTreeSet;

use super::ast::{fields, header, rust_type, snake, string};

pub(super) fn emit(nodes: &[Value], pin: &str) -> Result<String, String> {
    let mut code = header(pin, "ast_generated.go");
    code.push_str("use crate::compact::FieldKey;\n#[allow(clippy::wildcard_imports)] // Every concrete typed row is selected below.\nuse crate::compact_generated::*;\nuse crate::local_bind::{BindContext, BindList, BindNode, BindRead, BindSlice, BindTextSlice, LocalChildVisitor};\nuse crate::{NodeKind, SyntaxKind};\nuse std::marker::PhantomData;\nuse std::ops::ControlFlow;\n\n");
    let mut accessors = String::from("impl<'scope, 'read> BindRead<'scope, 'read> {\n");
    code.push_str("#[allow(non_upper_case_globals)] // Match concrete schema names in shared shape selectors.\npub(crate) mod shapes {\n");
    for (shape, node) in nodes.iter().enumerate() {
        code.push_str(&format!(
            "    pub(crate) const {}: u16 = {shape};\n",
            string(node, "name")?
        ));
    }
    code.push_str("}\n\n");
    accessors.push_str("    pub(crate) fn payload_name(&self) -> &'static str {\n        match self.header.actual_shape() {\n");
    for node in nodes {
        let name = string(node, "name")?;
        accessors.push_str(&format!("            shapes::{name} => \"{name}\",\n"));
    }
    accessors.push_str(
        "            _ => unreachable!(\"validated local payload shape\"),\n        }\n    }\n",
    );
    let mut flow_shapes = Vec::new();
    for (shape, node) in nodes.iter().enumerate() {
        let name = string(node, "name")?;
        let member = snake(name);
        let read = format!("Local{name}Read");
        let fields = fields(node)?;
        let types = fields
            .iter()
            .map(|field| rust_type(&field["type"]))
            .collect::<Result<Vec<_>, _>>()?;
        let uses_context = types.iter().any(|typ| {
            matches!(
                typ.as_str(),
                "NodeId" | "NodeListId" | "NodeSlice" | "TextSlice" | "JsString"
            )
        });
        let has_text = types.iter().any(|typ| typ == "JsString");
        let row_member = if fields.is_empty() { "_row" } else { "row" };
        code.push_str(&format!("/// A typed payload borrow within one validated binding scope.\n#[derive(Clone, Copy)]\npub struct {read}<'scope, 'read> {{\n    {row_member}: &'read {name}Row,\n"));
        if uses_context {
            code.push_str("    context: BindContext<'scope, 'read>,\n");
        } else {
            code.push_str("    scope: PhantomData<fn(&'scope ()) -> &'scope ()>,\n");
        }
        if has_text {
            code.push_str("    ordinal: u32,\n    end: i32,\n");
        }
        if has_text {
            code.push_str(&format!(
                "}}\nimpl<'scope, 'read> {read}<'scope, 'read> {{\n"
            ));
        } else {
            code.push_str(&format!("}}\nimpl<'scope> {read}<'scope, '_> {{\n"));
        }
        let mut names = BTreeSet::from(["for_each_child".to_owned()]);
        for (index, (field, typ)) in fields.iter().zip(&types).enumerate() {
            let field_name = snake(string(field, "name")?);
            if !names.insert(field_name.clone()) {
                return Err(format!(
                    "AST local read: duplicate or reserved method {name}.{field_name}"
                ));
            }
            let (output, value) = match typ.as_str() {
                "NodeId" => (
                    "Option<BindNode<'scope>>".to_owned(),
                    format!("self.context.node(self.row.{field_name})"),
                ),
                "NodeListId" => (
                    "Option<BindList<'scope>>".to_owned(),
                    format!("self.context.list(self.row.{field_name})"),
                ),
                "NodeSlice" => (
                    "BindSlice<'scope>".to_owned(),
                    format!("self.context.node_slice(self.row.{field_name})"),
                ),
                "TextSlice" => (
                    "BindTextSlice<'scope>".to_owned(),
                    format!("self.context.text_slice(self.row.{field_name})"),
                ),
                "JsString" => (
                    "&'read [u8]".to_owned(),
                    format!(
                        "self.context.text({}, self.row.{field_name}, self.end)",
                        super::ast_compact::field_key(shape, index)
                    ),
                ),
                _ => (typ.clone(), format!("self.row.{field_name}")),
            };
            code.push_str(&format!(
                "    #[inline]\n    pub fn {field_name}(&self) -> {output} {{\n"
            ));
            if typ == "JsString" {
                code.push_str("        let ordinal = self.ordinal;\n");
            }
            code.push_str(&format!("        {value}\n    }}\n"));
        }
        super::ast_read::emit_children(&mut code, node, "LocalChildVisitor<'scope>")?;
        code.push_str("}\n\n");
        accessors.push_str(&format!("    #[inline]\n    pub fn as_{member}(&self) -> Option<{read}<'scope, 'read>> {{\n        if self.header.actual_shape() != {shape} {{ return None; }}\n        Some({read} {{\n            {row_member}: self.context.store.payloads.local_{member}_row(self.header.ordinal),\n"));
        if uses_context {
            accessors.push_str("            context: self.context,\n");
        } else {
            accessors.push_str("            scope: PhantomData,\n");
        }
        if has_text {
            accessors.push_str(
                "            ordinal: self.header.ordinal,\n            end: self.header.end,\n",
            );
        }
        accessors.push_str("        })\n    }\n");
        if super::ast_compact::has_flow_node(node)? {
            flow_shapes.push(shape.to_string());
        }
    }
    accessors.push_str("    /// Visit public children in kind order, preserving kind/shape mismatch errors.\n    pub fn for_each_child(&self, visitor: &mut impl LocalChildVisitor<'scope>) -> ControlFlow<()> {\n        match self.header.kind.known() {\n");
    for node in nodes {
        let name = string(node, "name")?;
        let Some(pattern) = super::ast_runtime::child_kind_pattern(node)? else {
            continue;
        };
        accessors.push_str(&format!(
            "            {pattern} => self.as_{}().expect(\"{name} kind requires {name} payload\").for_each_child(visitor),\n",
            snake(name)
        ));
    }
    accessors.push_str("            _ => ControlFlow::Continue(()),\n        }\n    }\n");
    accessors.push_str("    /// Declaration names follow the same pinned schema predicate as Node.Name.\n    pub fn name(&self) -> Option<BindNode<'scope>> {\n        match self.header.actual_shape() {\n");
    for (shape, node) in nodes.iter().enumerate() {
        if super::ast::has_declaration_name(node)? {
            let member = snake(string(node, "name")?);
            accessors.push_str(&format!("            {shape} => self.as_{member}().expect(\"matched local declaration shape\").name(),\n"));
        }
    }
    accessors.push_str("            _ => None,\n        }\n    }\n");
    accessors.push_str(&format!(
        "    /// Field capability follows the pinned Go base embeddings, not syntax kind.\n    #[inline]\n    pub fn has_flow_node(&self) -> bool {{\n        matches!(self.header.actual_shape(), {})\n    }}\n}}\n",
        flow_shapes.join(" | ")
    ));
    code.push_str(&accessors);
    Ok(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schema() -> Value {
        serde_json::from_str(include_str!("../../../data/s03/schema/ast.json")).unwrap()
    }

    #[test]
    fn unexpected_schema_method_collision_is_rejected() {
        let mut schema = schema();
        let nodes = schema["nodes"].as_array_mut().unwrap();
        let identifier = nodes
            .iter_mut()
            .find(|node| node["name"] == "Identifier")
            .unwrap();
        let text = identifier["fields"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|field| field["name"] == "Text")
            .unwrap();
        text["name"] = Value::String("ForEachChild".into());
        assert!(emit(nodes, "test")
            .unwrap_err()
            .contains("reserved method Identifier.for_each_child"));
    }

    #[test]
    fn unknown_dynamic_visitor_fails_instead_of_losing_child_order() {
        let mut schema = schema();
        let nodes = schema["nodes"].as_array_mut().unwrap();
        let node = nodes
            .iter_mut()
            .find(|node| node["name"] == "PropertyAccessExpression")
            .unwrap();
        node["handWrittenVisitor"] = Value::Bool(true);
        assert!(emit(nodes, "test")
            .unwrap_err()
            .contains("unknown dynamic visitor"));
    }
}
