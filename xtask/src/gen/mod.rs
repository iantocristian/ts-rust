//! Pinned frontends export facts; these Rust emitters own Rust syntax only.
mod ast;
mod diagnostics;
mod encoder;

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::{json, Value};

const SCHEMAS: &[&str] = &[
    "ast",
    "diagnostics",
    "encoder",
    "api",
    "api-wire-fixtures",
    "client-files",
];
const MANIFEST: &str = "data/s03/generated.json";

fn json_bytes(value: &Value) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn read_json(path: &Path) -> Result<Value, String> {
    serde_json::from_slice(&fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?)
        .map_err(|e| format!("{}: {e}", path.display()))
}

fn format_rust(root: &Path, source: &str) -> Result<Vec<u8>, String> {
    let mut child = Command::new("rustfmt")
        .args(["--edition", "2021", "--config-path"])
        .arg(root.join("rustfmt.toml"))
        .args(["--emit", "stdout"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("start rustfmt: {e}"))?;
    // Write concurrently: generated sources can exceed the stdout pipe capacity.
    let mut stdin = child.stdin.take().ok_or("rustfmt stdin unavailable")?;
    let input = source.as_bytes().to_vec();
    let writer = std::thread::spawn(move || stdin.write_all(&input));
    let output = child
        .wait_with_output()
        .map_err(|e| format!("rustfmt: {e}"))?;
    writer
        .join()
        .map_err(|_| "rustfmt input writer panicked")?
        .map_err(|e| format!("rustfmt stdin: {e}"))?;
    if !output.status.success() {
        return Err(format!("rustfmt failed: {}", output.status));
    }
    Ok(output.stdout)
}

fn outputs(root: &Path, stage: &Path, pin: &str) -> Result<BTreeMap<PathBuf, Vec<u8>>, String> {
    let mut files = BTreeMap::new();
    for name in SCHEMAS {
        let value = read_json(&stage.join(format!("{name}.json")))?;
        let rust = match *name {
            "ast" => ast::emit(&value, pin)?,
            "diagnostics" => diagnostics::emit(&value, pin)?,
            "encoder" => encoder::emit(&value, pin)?,
            _ => BTreeMap::new(),
        };
        for (path, source) in rust {
            if files
                .insert(path.clone(), format_rust(root, &source)?)
                .is_some()
            {
                return Err(format!("duplicate emitted file: {}", path.display()));
            }
        }
        files.insert(
            PathBuf::from(format!("data/s03/schema/{name}.json")),
            json_bytes(&value)?,
        );
    }
    let hashes: BTreeMap<_, _> = files
        .iter()
        .map(|(path, bytes)| {
            (
                path.to_string_lossy().into_owned(),
                super::evidence::hash(bytes),
            )
        })
        .collect();
    files.insert(
        PathBuf::from(MANIFEST),
        json_bytes(&json!({
            "version": 1, "upstreamPin": pin, "files": hashes,
        }))?,
    );
    Ok(files)
}

fn managed(path: &Path) -> bool {
    if path
        .components()
        .any(|part| !matches!(part, std::path::Component::Normal(_)))
    {
        return false;
    }
    let text = path.to_string_lossy();
    (text.starts_with("data/s03/schema/") && text.ends_with(".json"))
        || (["ts_ast", "ts_diagnostics", "ts_encoder"]
            .iter()
            .any(|krate| text.starts_with(&format!("crates/{krate}/src/")))
            && (text.ends_with("/generated.rs") || text.ends_with("_generated.rs")))
}

fn previous_paths(root: &Path) -> Result<Vec<PathBuf>, String> {
    if !root.join(MANIFEST).exists() {
        return Ok(Vec::new());
    }
    let manifest = read_json(&root.join(MANIFEST))?;
    if manifest["version"] != 1 {
        return Err("unsupported S03 generated manifest".into());
    }
    manifest["files"]
        .as_object()
        .ok_or("invalid generated file inventory")?
        .keys()
        .map(|path| {
            let path = PathBuf::from(path);
            if managed(&path) {
                Ok(path)
            } else {
                Err(format!("unsafe managed path: {}", path.display()))
            }
        })
        .collect()
}

fn existing_paths(root: &Path) -> Result<Vec<PathBuf>, String> {
    fn visit(root: &Path, directory: &Path, paths: &mut Vec<PathBuf>) -> Result<(), String> {
        if !directory.exists() {
            return Ok(());
        }
        for entry in fs::read_dir(directory).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let kind = entry.file_type().map_err(|e| e.to_string())?;
            if kind.is_dir() {
                visit(root, &path, paths)?;
            } else {
                let relative = path.strip_prefix(root).map_err(|e| e.to_string())?;
                if managed(relative) {
                    if kind.is_symlink() {
                        return Err(format!(
                            "managed output is a symlink: {}",
                            relative.display()
                        ));
                    }
                    paths.push(relative.to_path_buf());
                }
            }
        }
        Ok(())
    }
    let mut paths = Vec::new();
    for directory in [
        "data/s03/schema",
        "crates/ts_ast/src",
        "crates/ts_diagnostics/src",
        "crates/ts_encoder/src",
    ] {
        visit(root, &root.join(directory), &mut paths)?;
    }
    Ok(paths)
}

fn differences(root: &Path, files: &BTreeMap<PathBuf, Vec<u8>>) -> Result<Vec<PathBuf>, String> {
    let mut changed = Vec::new();
    for (path, bytes) in files {
        match fs::read(root.join(path)) {
            Ok(existing) if existing == *bytes => {}
            Ok(_) => changed.push(path.clone()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => changed.push(path.clone()),
            Err(e) => return Err(format!("{}: {e}", path.display())),
        }
    }
    for path in previous_paths(root)?
        .into_iter()
        .chain(existing_paths(root)?)
    {
        if !files.contains_key(&path) {
            changed.push(path);
        }
    }
    changed.sort();
    changed.dedup();
    Ok(changed)
}

fn write_outputs(root: &Path, files: &BTreeMap<PathBuf, Vec<u8>>) -> Result<(), String> {
    let previous = previous_paths(root)?;
    for path in existing_paths(root)? {
        if !files.contains_key(&path) && !previous.contains(&path) {
            return Err(format!(
                "unrecognized generated output {}; review/remove it before regenerating",
                path.display()
            ));
        }
    }
    for path in previous {
        if !files.contains_key(&path) && root.join(&path).exists() {
            fs::remove_file(root.join(path)).map_err(|e| e.to_string())?;
        }
    }
    for (path, bytes) in files {
        let destination = root.join(path);
        fs::create_dir_all(destination.parent().ok_or("output lacks parent")?)
            .map_err(|e| e.to_string())?;
        fs::write(&destination, bytes).map_err(|e| format!("{}: {e}", destination.display()))?;
    }
    Ok(())
}

pub(super) fn run(root: &Path, args: &[String], pin: &str) -> Result<bool, String> {
    let mode = match args {
        [] => "write",
        [option] if option == "--check" || option == "--verify" => option,
        _ => return Err("usage: cargo xtask gen [--check | --verify]".into()),
    };
    // Retain the most recent staging output for inspecting failures; never touch upstream/.
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let stage = root.join(format!("target/s03-stage-{}-{nonce}", std::process::id()));
    fs::create_dir_all(root.join("target")).map_err(|e| e.to_string())?;
    fs::create_dir(&stage).map_err(|e| e.to_string())?;
    let output = Command::new("python3")
        .arg(root.join("scripts/s03.py"))
        .arg("prepare")
        .arg("--output")
        .arg(&stage)
        .arg("--pin")
        .arg(pin)
        .current_dir(root)
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| format!("S03 frontend: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "S03 frontend failed ({}); stage: {}",
            output.status,
            stage.display()
        ));
    }
    let summary: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("invalid frontend summary: {e}"))?;
    let patches = summary["patches_apply"]
        .as_bool()
        .ok_or("missing patches_apply")?;
    let client = summary["client_identical"]
        .as_bool()
        .ok_or("missing client_identical")?;
    let files = outputs(root, &stage, pin)?;
    let changed = differences(root, &files)?;
    for path in &changed {
        eprintln!("S03 drift: {}", path.display());
    }
    let mut metrics = summary
        .as_object()
        .ok_or("frontend summary must be an object")?
        .clone();
    metrics.insert(
        "ast_schema".into(),
        json!(files.contains_key(Path::new("crates/ts_ast/src/data_generated.rs"))),
    );
    metrics.insert("drift".into(), json!(!changed.is_empty()));
    metrics.insert("generated_files".into(), json!(files.len()));
    let passed = patches && client && changed.is_empty();
    if mode == "--verify" {
        println!("{}", json!({"metrics": metrics}));
        return Ok(true); // Successful measurement; consumers enforce individual metrics.
    }
    if mode == "write" && patches && client {
        write_outputs(root, &files)?;
        eprintln!("generated {} files from {pin}", files.len());
        return Ok(true);
    }
    if passed {
        eprintln!(
            "S03: all {} generated files and pinned client bytes match",
            files.len()
        );
    }
    Ok(passed)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn managed_paths_cannot_escape_or_delete_handwritten_code() {
        for path in [
            "../Cargo.toml",
            "/tmp/generated.rs",
            "data/s03/schema/../../Cargo.toml",
            "crates/ts_ast/src/lib.rs",
            "Cargo.toml",
        ] {
            assert!(!managed(Path::new(path)), "{path}");
        }
        for path in [
            "data/s03/schema/ast.json",
            "crates/ts_ast/src/data_generated.rs",
            "crates/ts_encoder/src/generated.rs",
        ] {
            assert!(managed(Path::new(path)), "{path}");
        }
    }

    #[test]
    fn drift_catches_missing_changed_and_removed_outputs_without_writing() {
        let root = std::env::temp_dir().join(format!("ts-rust-gen-test-{}", std::process::id()));
        fs::create_dir_all(root.join("data/s03/schema")).unwrap();
        fs::write(
            root.join(MANIFEST),
            br#"{"version":1,"files":{"data/s03/schema/old.json":"hash"}}"#,
        )
        .unwrap();
        fs::write(root.join("data/s03/schema/changed.json"), b"old").unwrap();
        fs::write(root.join("data/s03/schema/untracked.json"), b"extra").unwrap();
        let files = BTreeMap::from([
            (
                PathBuf::from("data/s03/schema/changed.json"),
                b"new".to_vec(),
            ),
            (
                PathBuf::from("data/s03/schema/missing.json"),
                b"new".to_vec(),
            ),
        ]);
        assert_eq!(differences(&root, &files).unwrap().len(), 4);
        assert!(write_outputs(&root, &files)
            .unwrap_err()
            .contains("unrecognized"));
        assert_eq!(
            fs::read(root.join("data/s03/schema/changed.json")).unwrap(),
            b"old"
        );
        assert!(!root.join("data/s03/schema/missing.json").exists());
        fs::remove_dir_all(root).unwrap();
    }
}
