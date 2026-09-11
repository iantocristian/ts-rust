mod baseline;
mod diagnostics;
mod graph;
mod merges;
mod queries;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use ts_arena::{CheckerIdentity, Counters, Generation, NodeId};
use ts_ast::{AstView, CompletedFile};
use ts_checker::CheckerOwner;
use ts_compiler::{FileCache, Program, ProgramCheckerHost, ProgramOptions};
use ts_core::{CompilerOptions, ModuleKind, ScriptTarget, Tristate};
use ts_jsstring::JsString;

#[derive(Debug)]
pub enum Error {
    Checker(ts_checker::Error),
    Compiler(ts_compiler::Error),
    Ast(ts_arena::Error),
    Protocol(String),
    Unsupported(&'static str),
}
impl From<ts_checker::Error> for Error {
    fn from(v: ts_checker::Error) -> Self {
        Self::Checker(v)
    }
}
impl From<ts_compiler::Error> for Error {
    fn from(v: ts_compiler::Error) -> Self {
        Self::Compiler(v)
    }
}
impl From<ts_arena::Error> for Error {
    fn from(v: ts_arena::Error) -> Self {
        Self::Ast(v)
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Checker(error) => write!(f, "checker: {error}"),
            Self::Compiler(error) => write!(f, "compiler: {error}"),
            Self::Ast(error) => write!(f, "AST: {error}"),
            Self::Protocol(message) => write!(f, "protocol: {message}"),
            Self::Unsupported(operation) => write!(f, "unsupported: {operation}"),
        }
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;

pub fn text(value: &Value) -> Result<&str> {
    value
        .as_str()
        .ok_or_else(|| Error::Protocol("expected string".into()))
}
pub fn array(value: &Value) -> Result<&[Value]> {
    value
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| Error::Protocol("expected array".into()))
}
pub fn utf8(value: &[u8]) -> Result<&str> {
    std::str::from_utf8(value)
        .map_err(|_| Error::Unsupported("P2 diagnostic JSON argument requires valid UTF-8"))
}
pub fn hex(value: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(value.len() * 2);
    for &byte in value {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 15)]));
    }
    output
}
pub fn panic_error(payload: &(dyn std::any::Any + Send)) -> Error {
    let message = payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&str>().copied())
        .unwrap_or("non-string panic payload");
    Error::Protocol(format!("panic: {message}"))
}
pub fn failure(error: &Error, operation: &str) -> Value {
    let unsupported = match error {
        Error::Checker(ts_checker::Error::Unsupported(name))
        | Error::Compiler(ts_compiler::Error::Unsupported(name))
        | Error::Unsupported(name) => Some(*name),
        _ => None,
    };
    match unsupported {
        Some(reason) => json!({"state":"unsupported","operation":operation,"reason":reason}),
        None => json!({"state":"failed","operation":operation,"reason":error.to_string()}),
    }
}
pub fn source(program: &Program, node: NodeId) -> Result<&CompletedFile> {
    program
        .files()
        .iter()
        .find(|f| f.source().arena() == node.arena())
        .map(|f| f.bound())
        .ok_or(Error::Ast(ts_arena::Error::WrongOwner))
}
pub fn view(program: &Program, node: NodeId) -> Result<AstView<'_>> {
    Ok(source(program, node)?.view().ast())
}
pub fn file_name(program: &Program, node: NodeId) -> Result<&[u8]> {
    Ok(source(program, node)?
        .view()
        .source_file()?
        .parse_options()
        .file_name
        .as_bytes())
}
pub fn node_json(program: &Program, node: Option<NodeId>) -> Result<Value> {
    let Some(id) = node else {
        return Ok(Value::Null);
    };
    let n = view(program, id)?.node(id)?;
    Ok(
        json!({"file":utf8(file_name(program,id)?)?,"kind":n.kind().raw(),"pos":n.pos(),"end":n.end()}),
    )
}
pub fn load(
    files: &Value,
    roots: &Value,
    cache: &mut FileCache,
    counters: &Counters,
) -> Result<Arc<Program>> {
    let mut fs = ts_vfs::MemoryBuilder::new(b"/", true);
    for (name, source) in files
        .as_object()
        .ok_or_else(|| Error::Protocol("expected files".into()))?
    {
        fs.insert_loaded(name.as_bytes(), text(source)?.as_bytes());
    }
    let options = CompilerOptions {
        target: ScriptTarget::ESNEXT,
        module: ModuleKind::ESNEXT,
        strict: Tristate::TRUE,
        no_lib: Tristate::TRUE,
        ..Default::default()
    };
    let config = ts_tsoptions::ParsedCommandLine::new(
        options,
        array(roots)?
            .iter()
            .map(|name| Ok(JsString::from_bytes(text(name)?.as_bytes())))
            .collect::<Result<Vec<_>>>()?,
    );
    Ok(Arc::new(Program::load(
        ProgramOptions {
            config,
            host: Arc::new(fs.finish()),
            current_directory: JsString::from_bytes(b"/".as_slice()),
            default_library_path: JsString::from_bytes(b"/no-default-lib".as_slice()),
            skip_module_resolution: false,
        },
        cache,
        counters,
    )?))
}
pub fn owner(
    program: Arc<Program>,
    generation: &Generation,
    counters: &Counters,
) -> Result<Arc<CheckerOwner>> {
    Ok(Arc::new(CheckerOwner::for_program(
        CheckerIdentity::new(generation.clone(), counters),
        counters,
        Arc::new(ProgramCheckerHost::new(program)),
    )?))
}

pub fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err("usage: p2_checker canonical-native-requests.json observations.json".into());
    }
    let raw = std::fs::read(&args[1])?;
    let spec: Value = serde_json::from_slice(&raw)?;
    let mut canonical = serde_json::to_vec(&spec)?;
    canonical.push(b'\n');
    // A canonical wire request rejects duplicate keys as well as trailing data;
    // the checked Python producer supplies this exact representation.
    if raw != canonical {
        return Err("P2 requires the canonical native request bytes".into());
    }
    if spec["version"] != 1
        || spec["options"]
            != json!({"target":"ESNext","module":"ESNext","strict":true,"noLib":true})
    {
        return Err("unsupported P2 protocol/options".into());
    }
    let counters = Counters::new();
    let generation = Generation::new(&counters);
    let mut programs = Vec::new();
    let mut ownership = Vec::new();
    for request in array(&spec["programs"])? {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            queries::program(request, &generation, &counters)
        }))
        .map_err(|payload| panic_error(payload.as_ref()))
        .and_then(|value| value);
        let mut row = match result {
            Ok((row, lifetime)) => {
                ownership.push(lifetime);
                row
            }
            Err(error) => {
                let mut lifetime = failure(
                    &Error::Unsupported("program operation did not yield a retained type"),
                    "retain_type",
                );
                lifetime["id"] = request["id"].clone();
                ownership.push(lifetime);
                failure(&error, "program")
            }
        };
        row["id"] = request["id"].clone();
        programs.push(row);
    }
    let mut modes = Vec::new();
    for mode in array(&spec["merge"]["modes"])? {
        let name = text(mode)?;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            merges::mode(&spec["merge"], name, &generation, &counters)
        }))
        .map_err(|payload| panic_error(payload.as_ref()))
        .and_then(|value| value);
        let mut row = match result {
            Ok(row) => row,
            Err(error) => failure(&error, "checker_local_merge"),
        };
        row["mode"] = mode.clone();
        modes.push(row);
    }
    let out = json!({"version":1,"request_sha256":format!("{:x}",Sha256::digest(&raw)),"programs":programs,"merges":modes,
        "rust_ownership":{"scope":"Executed Rust-only retained-result lifetime checks; no native ownership equivalence claim","programs":ownership}});
    let mut bytes = serde_json::to_vec(&out)?;
    bytes.push(b'\n');
    std::fs::write(&args[2], bytes)?;
    Ok(())
}
