use crate::{ensure, Error, Result};
use serde_json::{Map, Value};
use std::path::PathBuf;

pub fn object(value: &Value) -> Result<&Map<String, Value>> {
    value
        .as_object()
        .ok_or_else(|| Error::new("expected JSON object"))
}
pub fn integer(value: &Value) -> Result<u64> {
    value
        .as_u64()
        .ok_or_else(|| Error::new("expected unsigned integer"))
}
fn text(value: &Value) -> Result<&str> {
    value.as_str().ok_or_else(|| Error::new("expected string"))
}
fn get<'a>(values: &'a Map<String, Value>, name: &str) -> Result<&'a Value> {
    values
        .get(name)
        .ok_or_else(|| Error::new("missing registry field"))
}
fn field_index(value: &Value) -> Result<usize> {
    ["a", "b", "c", "d"]
        .iter()
        .position(|name| Some(*name) == value.as_str())
        .ok_or_else(|| Error::new("unknown field index"))
}

pub struct Field {
    maximum: u64,
    minimum: u64,
    exact: Option<u64>,
    node: bool,
    optional: bool,
}
impl Field {
    pub fn check(&self, value: u64) -> Result<()> {
        ensure(
            value <= self.maximum && value >= self.minimum,
            "field outside registry bounds",
        )?;
        ensure(
            self.exact.is_none_or(|expected| value == expected),
            "field differs from exact value",
        )?;
        if self.node && !(self.optional && value == 0) {
            node_id(value)?;
        }
        Ok(())
    }
}
pub fn node_id(value: u64) -> Result<()> {
    ensure(
        value >> 32 != 0 && value & u64::from(u32::MAX) != 0,
        "node ID requires a nonzero arena and slot",
    )
}

pub struct Event {
    pub domain: u32,
    pub sites: Option<Vec<bool>>,
    pub checks: Vec<(usize, Field)>,
    pub blob: bool,
    pub category: usize,
    pub constraints: Vec<(usize, usize)>,
}

pub fn load(paths: &[PathBuf]) -> Result<Vec<Option<Event>>> {
    let mut result: Vec<Option<Event>> = (0..150).map(|_| None).collect();
    for path in paths {
        let document = crate::json::read(path)?;
        let document = object(&document)?;
        ensure(
            integer(get(document, "version")?)? == 1,
            "unknown registry version",
        )?;
        let events = get(document, "events")?
            .as_array()
            .ok_or_else(|| Error::new("missing event array"))?;
        for value in events {
            let event = object(value)?;
            let op = integer(get(event, "id")?)?;
            let domain = match op {
                1..=9 => 0,
                10..=79 => 1,
                100..=149 => 2,
                _ => return Err(Error::new("event outside registry ranges")),
            };
            ensure(
                integer(get(event, "domain")?)? == u64::from(domain),
                "event domain differs from registry range",
            )?;
            ensure(
                !text(get(event, "name")?)?.is_empty(),
                "event name is empty",
            )?;
            let op = usize::try_from(op).map_err(|_| Error::new("event index overflow"))?;
            ensure(result[op].is_none(), "duplicate event ID")?;
            let sites = match event.get("sites").filter(|v| !v.is_null()) {
                None => {
                    ensure(
                        !text(get(event, "site_semantics")?)?.is_empty(),
                        "wildcard sites need semantics",
                    )?;
                    None
                }
                Some(value) => {
                    let sites = value
                        .as_array()
                        .ok_or_else(|| Error::new("invalid sites array"))?;
                    ensure(!sites.is_empty(), "empty sites array")?;
                    let mut present = Vec::new();
                    for value in sites {
                        let id = integer(get(object(value)?, "id")?)?;
                        ensure(id <= u64::from(u16::MAX), "site ID overflow")?;
                        let id =
                            usize::try_from(id).map_err(|_| Error::new("site index overflow"))?;
                        present.resize(present.len().max(id + 1), false);
                        ensure(!present[id], "duplicate site ID")?;
                        present[id] = true;
                    }
                    Some(present)
                }
            };
            let fields = object(get(event, "fields")?)?;
            ensure(
                fields.len() == 4
                    && ["a", "b", "c", "d"]
                        .iter()
                        .all(|name| fields.contains_key(*name)),
                "registry requires four named fields",
            )?;
            let mut checks = Vec::new();
            for (index, name) in ["a", "b", "c", "d"].into_iter().enumerate() {
                let field = &fields[name];
                if field.is_string() {
                    continue;
                }
                let field = object(field)?;
                text(get(field, "name")?)?;
                let kind = text(get(field, "type")?)?;
                let type_maximum = match kind {
                    "u64" | "node_id" | "optional_node_id" => u64::MAX,
                    "u32" => u64::from(u32::MAX),
                    "u16" => u64::from(u16::MAX),
                    "bool" => 1,
                    "zero" => 0,
                    _ => return Err(Error::new("unsupported field type")),
                };
                let minimum = field.get("min").map(integer).transpose()?.unwrap_or(0);
                let maximum = field
                    .get("max")
                    .map(integer)
                    .transpose()?
                    .unwrap_or(u64::MAX)
                    .min(type_maximum);
                let exact = field.get("exact_value").map(integer).transpose()?;
                if kind != "u64" || minimum != 0 || maximum != u64::MAX || exact.is_some() {
                    checks.push((
                        index,
                        Field {
                            maximum,
                            minimum,
                            exact,
                            node: matches!(kind, "node_id" | "optional_node_id"),
                            optional: kind == "optional_node_id",
                        },
                    ));
                }
            }
            let blob = match event.get("blob") {
                None | Some(Value::Bool(false)) => false,
                Some(Value::Bool(true)) => true,
                Some(Value::String(value)) if value == "none" => false,
                Some(Value::String(value)) if value == "bytes" => true,
                _ => return Err(Error::new("unsupported blob contract")),
            };
            let category = match event
                .get("category")
                .map(text)
                .transpose()?
                .unwrap_or("other")
            {
                "lookup" => 0,
                "read" => 1,
                "write" => 2,
                "other" | "driver" | "state" => 3,
                _ => return Err(Error::new("unknown category")),
            };
            ensure(
                category == 3 || domain == 2,
                "nonbinder category counted as binder",
            )?;
            let mut constraints = Vec::new();
            if let Some(value) = event.get("constraints") {
                for value in value
                    .as_array()
                    .ok_or_else(|| Error::new("invalid constraints array"))?
                {
                    let value = object(value)?;
                    ensure(
                        value.len() == 3
                            && text(get(value, "rule")?)? == "relative_index_less_than_packed_len",
                        "unknown constraint",
                    )?;
                    constraints.push((
                        field_index(get(value, "index_field")?)?,
                        field_index(get(value, "descriptor_field")?)?,
                    ));
                }
            }
            result[op] = Some(Event {
                domain,
                sites,
                checks,
                blob,
                category,
                constraints,
            });
        }
    }
    ensure(
        result[1..10].iter().all(Option::is_some),
        "driver registry incomplete",
    )?;
    Ok(result)
}
