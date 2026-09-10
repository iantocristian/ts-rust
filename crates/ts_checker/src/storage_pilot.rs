//! First P1 vertical slice: actual intrinsic/string construction, interning and
//! fresh/regular links. This is not the subset's per-type footprint measurement.

use crate::{Error, TypeId, TypeStore};
use serde_json::{json, Value};
use ts_ast::JsString;

enum Action {
    Intrinsic(u32, JsString),
    String(JsString),
    Fresh(usize),
}

/// Input and text backing are prepared before the measured constructor phase.
pub struct Prepared {
    actions: Vec<Action>,
}

/// All returned roots and type storage stay alive at the allocation checkpoint.
pub struct Live {
    store: TypeStore,
    roots: Vec<TypeId>,
}

fn bytes(text: &str) -> Result<Vec<u8>, String> {
    if !text.len().is_multiple_of(2) || !text.is_ascii() {
        return Err("invalid byte hex".into());
    }
    text.as_bytes()
        .chunks_exact(2)
        .map(|chunk| {
            let digit = |byte: u8| char::from(byte).to_digit(16).ok_or("invalid hex digit");
            Ok((digit(chunk[0])? * 16 + digit(chunk[1])?) as u8)
        })
        .collect()
}

pub(crate) fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("write to string");
    }
    output
}

impl Prepared {
    pub fn new(requests: &Value) -> Result<Self, String> {
        let rows = requests.as_array().ok_or("expected action array")?;
        if rows.is_empty() {
            return Err("empty constructor trace".into());
        }
        let mut actions = Vec::with_capacity(rows.len());
        for (index, row) in rows.iter().enumerate() {
            let row = row.as_object().ok_or("action must be an object")?;
            let op = row.get("op").and_then(Value::as_str).ok_or("missing op")?;
            let expected: &[&str] = match op {
                "intrinsic" => &["op", "flags", "text_hex"],
                "string" => &["op", "text_hex"],
                "fresh" => &["op", "root"],
                _ => return Err("unknown constructor action".into()),
            };
            if row.len() != expected.len() || expected.iter().any(|key| !row.contains_key(*key)) {
                return Err("missing or unexpected action field".into());
            }
            actions.push(if op == "fresh" {
                let root = row["root"]
                    .as_u64()
                    .and_then(|value| usize::try_from(value).ok())
                    .ok_or("invalid root")?;
                if root >= index {
                    return Err("fresh root is not yet available".into());
                }
                Action::Fresh(root)
            } else {
                let value =
                    JsString::from_bytes(bytes(row["text_hex"].as_str().ok_or("invalid text")?)?);
                if op == "string" {
                    Action::String(value)
                } else {
                    let flags = row["flags"]
                        .as_u64()
                        .and_then(|value| u32::try_from(value).ok())
                        .ok_or("invalid flags")?;
                    Action::Intrinsic(flags, value)
                }
            });
        }
        Ok(Self { actions })
    }

    pub fn execute(&self) -> Result<Live, Error> {
        let mut store = TypeStore::new();
        let mut roots = Vec::with_capacity(self.actions.len());
        for action in &self.actions {
            let id = match action {
                Action::Intrinsic(flags, name) => {
                    store.new_intrinsic_type(*flags, name.clone(), 0)?
                }
                Action::String(value) => store.string_literal_type(value.clone())?,
                Action::Fresh(root) => store.fresh_string_literal_type(roots[*root])?,
            };
            roots.push(id);
        }
        Ok(Live { store, roots })
    }
}

impl Live {
    /// Called after allocator counters are sampled so JSON does not enter the interval.
    pub fn observation(&self) -> Value {
        json!({"roots": self.roots.iter().map(|id| self.store.pilot_observation(*id)).collect::<Vec<_>>(),
            "census": self.store.pilot_census()})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_or_forward_referencing_requests_cannot_produce_observations() {
        for request in [
            json!([]),
            json!([{"op":"string", "text_hex":"f"}]),
            json!([{"op":"string", "text_hex":"gg"}]),
            json!([{"op":"string", "text_hex":"00", "unexpected":true}]),
            json!([{"op":"fresh", "root":0}]),
            json!([{"op":"intrinsic", "text_hex":"", "flags":true}]),
            json!([{"op":"intrinsic", "text_hex":"", "flags":4_294_967_296_u64}]),
        ] {
            assert!(Prepared::new(&request).is_err(), "accepted {request}");
        }
        let non_string = Prepared::new(&json!([
            {"op":"intrinsic", "text_hex":"616e79", "flags":1}, {"op":"fresh", "root":0}
        ]))
        .unwrap();
        assert!(matches!(non_string.execute(), Err(Error::Unsupported(_))));
    }
}
