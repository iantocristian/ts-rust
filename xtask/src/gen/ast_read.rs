//! Contextual payload borrows keep semantic field access independent of storage.

use serde_json::Value;
use std::collections::BTreeSet;

use super::ast::{fields, header, nullable, rust_type, snake, string};

pub(super) fn owned_method(field: &str) -> String {
    format!("{}_owned", field.strip_prefix("r#").unwrap_or(field))
}

pub(super) fn emit(nodes: &[Value], pin: &str) -> Result<String, String> {
    let mut code = header(pin, "ast_generated.go");
    code.push_str("#[allow(clippy::wildcard_imports)] // Generated views consume the complete schema API.\nuse crate::*;\n\n");
    let mut accessors = String::from("impl<'owner> NodeRead<'owner> {\n");
    for node in nodes {
        let name = string(node, "name")?;
        let view = format!("{name}PayloadRead");
        let accessor = format!("as_{}", snake(name));
        let fields = fields(node)?;
        let mut names = BTreeSet::from(["node".to_owned()]);
        for field in &fields {
            let field_name = snake(string(field, "name")?);
            if !names.insert(field_name.clone()) {
                return Err(format!(
                    "AST read: duplicate or reserved method {name}.{field_name}"
                ));
            }
            if rust_type(&field["type"])? == "JsString" {
                let owned = owned_method(&field_name);
                if !names.insert(owned.clone()) {
                    return Err(format!("AST read: duplicate method {name}.{owned}"));
                }
            }
        }
        code.push_str(&format!(
            "/// Borrowed {name} payload selected by its concrete storage shape.\n/// The view cannot outlive its originating node read.\npub struct {view}<'read, 'owner> {{\n    node: &'read NodeRead<'owner>,\n"
        ));
        if !fields.is_empty() {
            code.push_str(&format!("    data: &'read {name}Data,\n"));
        }
        code.push_str(&format!(
            "}}\nimpl<'read, 'owner> {view}<'read, 'owner> {{\n    /// Identity and physical owner/source context for this payload.\n    pub fn node(&self) -> &'read NodeRead<'owner> {{ self.node }}\n"
        ));
        for field in &fields {
            let field_name = snake(string(field, "name")?);
            let typ = rust_type(&field["type"])?;
            if typ == "JsString" {
                let owned = owned_method(&field_name);
                code.push_str(&format!(
                    "    /// Borrow the selected bytes without retaining or copying their backing.\n    pub fn {field_name}(&self) -> &'read [u8] {{ self.data.{field_name}.as_bytes() }}\n    /// Explicitly retain independently usable text.\n    pub fn {owned}(&self) -> JsString {{ self.data.{field_name}.clone() }}\n"
                ));
            } else {
                let typ = if nullable(field)? {
                    format!("Option<{typ}>")
                } else {
                    typ
                };
                code.push_str(&format!(
                    "    pub fn {field_name}(&self) -> {typ} {{ self.data.{field_name} }}\n"
                ));
            }
        }
        code.push_str("}\n\n");
        accessors.push_str(&format!(
            "    pub fn {accessor}<'read>(&'read self) -> Option<{view}<'read, 'owner>> {{\n"
        ));
        if fields.is_empty() {
            accessors.push_str(&format!(
                "        self.data().{accessor}()?;\n        Some({view} {{ node: self }})\n"
            ));
        } else {
            accessors.push_str(&format!(
                "        let data = self.data().{accessor}()?;\n        Some({view} {{ node: self, data }})\n"
            ));
        }
        accessors.push_str("    }\n");
    }
    accessors.push_str("}\n");
    code.push_str(&accessors);
    Ok(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn field(name: &str, typ: &Value) -> Value {
        json!({"name": name, "type": typ, "goOnly": false})
    }

    #[test]
    fn field_names_cannot_shadow_context_or_owned_text_methods() {
        let text = json!({"kind": "primitive", "name": "string"});
        for members in [
            vec![field("Node", &text)],
            vec![field("Text", &text), field("TextOwned", &text)],
            vec![field("TextOwned", &text), field("Text", &text)],
            vec![field("Type", &text), field("TypeOwned", &text)],
        ] {
            assert!(emit(&[json!({"name": "Fixture", "fields": members})], "pin").is_err());
        }
    }
}
