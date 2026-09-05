use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "tracker-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&dir).unwrap();
        for p in ["data", "status", "sprints", "crates/demo", "docs/adr"] {
            fs::create_dir_all(dir.join(p)).unwrap();
        }
        let f = Self(dir);
        f.write(
            "PORTS.toml",
            &format!(
                r#"pin = "{PIN}"
[[file]]
go = "tsc/demo.go"
package = "demo"
crate = "demo"
phase = 0
kind = "source"
status = "ported"
rust = ["crates/demo/lib.rs"]
verify = ["run.proof.ok == true"]
pin = "{PIN}"
source_hash = "{}"
loc = 10
"#,
                "b".repeat(64)
            ),
        );
        f.write(
            "crates/demo/lib.rs",
            "// port: tsc/demo.go:A.Map\npub fn map() {}\n",
        );
        f.write("data/go-functions.tsv", &format!("# upstream {PIN}\nfile\tpackage\treceiver\tname\tstart\tend\tid\ntsc/demo.go\tdemo\t*A\tMap\t1\t2\ttsc/demo.go:A.Map\ntsc/demo.go\tdemo\t*B\tMap\t3\t4\ttsc/demo.go:B.Map\n"));
        f.write(
            "status/experiments.toml",
            r#"[E7]
title = "WASM"
nature = "fixture"
[[E7.criteria]]
id = "size"
metric = "run.proof.size"
op = "<="
threshold = 0.25
[[E7.criteria]]
id = "checker"
metric = "run.proof.checker"
op = "=="
threshold = true
"#,
        );
        f.write(
            "status/runs.toml",
            r#"[proof]
command = ["python3", "producer.py"]
inputs = ["producer.py"]
target = "host"
config = "test"
"#,
        );
        f.write("producer.py", "import json\nprint(json.dumps({'metrics': {'ok': True, 'size': 0.2, 'checker': False}}))\n");
        f.write(
            "sprints/S01.toml",
            r#"id = "S01"
title = "fixture"
exit = ["ledger.files_total > 0"]
[[item]]
id = "required"
title = "Required missing result"
done_when = ["run.proof.missing == true"]
"#,
        );
        f.manifest();
        f.git(&["init", "--quiet"]);
        f.git(&["add", "."]);
        f.git(&[
            "-c",
            "user.name=Tracker Test",
            "-c",
            "user.email=test@example.invalid",
            "commit",
            "--quiet",
            "-m",
            "fixture",
        ]);
        f
    }
    fn write(&self, path: &str, text: &str) {
        fs::write(self.0.join(path), text).unwrap();
    }
    fn replace(&self, path: &str, from: &str, to: &str) {
        self.write(
            path,
            &fs::read_to_string(self.0.join(path))
                .unwrap()
                .replace(from, to),
        );
    }
    fn git(&self, args: &[&str]) {
        assert!(std::process::Command::new("git")
            .arg("-C")
            .arg(&self.0)
            .args(args)
            .output()
            .unwrap()
            .status
            .success());
    }
    fn manifest(&self) {
        let value = serde_json::json!({"schema_version":2,"pin":PIN,"ledger_generated_sha256":evidence::ledger_generated_hash(&self.0).unwrap(),"inventory_sha256":evidence::hash(&fs::read(self.0.join("data/go-functions.tsv")).unwrap())});
        self.write("data/upstream.json", &value.to_string());
    }
    fn report(&self) -> Report {
        build_report(&self.0)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
const PIN: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

#[test]
fn receiver_ids_count_distinct_mappings_and_reach_full_coverage() {
    let f = Fixture::new();
    let report = f.report();
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    assert_eq!(num(&report.metrics, "functions.total"), 2.0);
    assert_eq!(num(&report.metrics, "functions.ratio"), 0.5);
    f.write(
        "crates/demo/lib.rs",
        "// port: tsc/demo.go:A.Map\n// port: tsc/demo.go:B.Map\n",
    );
    assert_eq!(num(&f.report().metrics, "functions.ratio"), 1.0);
}
#[test]
fn manual_verified_and_nonexistent_rust_cannot_count_as_verified() {
    let f = Fixture::new();
    f.replace("PORTS.toml", "status = \"ported\"", "status = \"verified\"");
    f.manifest();
    let r = f.report();
    assert_eq!(num(&r.metrics, "ledger.files_verified"), 0.0);
    assert!(!r.errors.is_empty());
    f.replace("PORTS.toml", "status = \"verified\"", "status = \"ported\"");
    fs::remove_file(f.0.join("crates/demo/lib.rs")).unwrap();
    f.manifest();
    let r = f.report();
    assert_eq!(num(&r.metrics, "ledger.files_verified"), 0.0);
    assert!(!r.errors.is_empty());
}
#[test]
fn complete_evidence_is_required_for_verification_and_experiments() {
    let f = Fixture::new();
    assert_eq!(num(&f.report().metrics, "ledger.files_verified"), 0.0);
    assert!(evidence::run(&f.0, "proof", PIN).unwrap());
    let r = f.report();
    assert_eq!(num(&r.metrics, "ledger.files_verified"), 1.0);
    assert_eq!(num(&r.metrics, "exp.E7.size.pass"), 1.0);
    assert_eq!(num(&r.metrics, "exp.E7.checker.pass"), 0.0);
    assert_eq!(num(&r.metrics, "exp.E7.pass"), 0.0);
    assert!(!r.sprints[0].done);
    f.replace("producer.py", "'checker': False", "'checker': True");
    assert_eq!(num(&f.report().metrics, "ledger.files_verified"), 0.0);
    assert!(evidence::run(&f.0, "proof", PIN).unwrap());
    assert_eq!(num(&f.report().metrics, "exp.E7.pass"), 1.0);
}
#[test]
fn stale_sync_pin_and_missing_verification_checks_do_not_pass() {
    let f = Fixture::new();
    f.replace(
        "PORTS.toml",
        &format!("pin = \"{PIN}\"\nsource_hash"),
        "pin = \"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\"\nsource_hash",
    );
    f.manifest();
    assert!(evidence::run(&f.0, "proof", PIN).unwrap());
    let r = f.report();
    assert_eq!(num(&r.metrics, "ledger.files_stale"), 1.0);
    assert_eq!(num(&r.metrics, "ledger.files_verified"), 0.0);
}
#[test]
fn required_unchecked_or_failing_items_block_but_optional_items_do_not() {
    let f = Fixture::new();
    assert!(!f.report().sprints[0].done);
    f.replace(
        "sprints/S01.toml",
        "done_when = [\"run.proof.missing == true\"]",
        "",
    );
    assert!(!f.report().sprints[0].done);
    f.replace(
        "sprints/S01.toml",
        "title = \"Required missing result\"",
        "title = \"Optional research\"\nrequired = false",
    );
    assert!(f.report().sprints[0].done);
}
#[test]
fn unknown_markers_and_duplicate_sprint_ids_block_check() {
    let f = Fixture::new();
    f.replace(
        "sprints/S01.toml",
        "done_when = [\"run.proof.missing == true\"]",
        "required = false",
    );
    f.write("crates/demo/lib.rs", "// port: tsc/demo.go:Missing\n");
    assert!(!f.report().sprints[0].done);
    f.write("crates/demo/lib.rs", "");
    fs::copy(f.0.join("sprints/S01.toml"), f.0.join("sprints/S02.toml")).unwrap();
    let r = f.report();
    assert!(r.errors.iter().any(|e| e.contains("duplicate sprint ID")));
    assert!(r.sprints.iter().all(|s| !s.done));
}
#[test]
fn schema_rejects_misspelled_items_and_legacy_scalar_measurements() {
    assert!(toml::from_str::<Sprint>("id='S'\ntitle='x'\n[[items]]\nid='x'\n").is_err());
    assert!(toml::from_str::<Experiment>(
        "title='x'\nnature='x'\nmeasured=1\nop='=='\nthreshold=1\n"
    )
    .is_err());
}
#[test]
fn duplicate_history_publication_does_not_create_new_measurements() {
    let f = Fixture::new();
    let r = f.report();
    record_history(&f.0, &r);
    record_history(&f.0, &r);
    assert_eq!(
        fs::read_to_string(f.0.join("status/history.jsonl"))
            .unwrap()
            .lines()
            .count(),
        1
    );
}
#[test]
fn metadata_only_commit_does_not_invalidate_identical_tested_content() {
    let f = Fixture::new();
    assert!(evidence::run(&f.0, "proof", PIN).unwrap());
    record_history(&f.0, &f.report());
    f.git(&["add", "status/evidence"]);
    f.git(&[
        "-c",
        "user.name=Tracker Test",
        "-c",
        "user.email=test@example.invalid",
        "commit",
        "--quiet",
        "-m",
        "record evidence",
    ]);
    assert_eq!(num(&f.report().metrics, "ledger.files_verified"), 1.0);
    record_history(&f.0, &f.report());
    assert_eq!(
        fs::read_to_string(f.0.join("status/history.jsonl"))
            .unwrap()
            .lines()
            .count(),
        1
    );
}

#[test]
fn complete_parity_requires_exact_equality() {
    let metrics = BTreeMap::from([("run.corpus.parity".into(), Metric::Num(1.0 - 1e-10))]);
    assert_eq!(eval_check(&metrics, "run.corpus.parity == 1"), Some(false));
}

#[test]
fn editable_ledger_state_and_policy_changes_reuse_current_evidence() {
    let f = Fixture::new();
    f.replace("PORTS.toml", "status = \"ported\"", "status = \"planned\"");
    assert!(evidence::provenance(&f.0, PIN).is_ok());
    assert!(evidence::run(&f.0, "proof", PIN).unwrap());
    let artifact = f.report().evidence_artifacts["proof"].clone();
    f.replace("PORTS.toml", "status = \"planned\"", "status = \"ported\"");
    assert!(evidence::provenance(&f.0, PIN).is_ok());
    let r = f.report();
    assert_eq!(r.evidence_artifacts["proof"], artifact);
    assert_eq!(num(&r.metrics, "ledger.files_verified"), 1.0);
    f.replace(
        "PORTS.toml",
        "run.proof.ok == true",
        "run.proof.checker == true",
    );
    assert!(evidence::provenance(&f.0, PIN).is_ok());
    let r = f.report();
    assert_eq!(r.evidence_states["proof"], "current");
    assert_eq!(num(&r.metrics, "ledger.files_verified"), 0.0);
    f.replace("PORTS.toml", "loc = 10", "loc = 11");
    assert!(evidence::provenance(&f.0, PIN).is_err());
}

#[test]
fn generated_projection_has_the_same_canonical_hash_as_python() {
    let f = Fixture::new();
    f.write(
        "PORTS.toml",
        &format!(
            r#"pin = "{PIN}"
[[file]]
go = "tsc/internal/é.go"
package = "internal"
crate = "ts_core"
phase = 0
kind = "source"
pin = "{PIN}"
source_hash = "{}"
loc = 2
status = "planned"
rust = []
verify = []
"#,
            "b".repeat(64)
        ),
    );
    assert_eq!(
        evidence::ledger_generated_hash(&f.0).unwrap(),
        "e6b84925e58249fe2524f0db7428309a0bb5433d123d5549ab1b302f67ea39ee"
    );
}

#[test]
fn history_records_policy_state_changes_without_relabeling_old_measurements() {
    let f = Fixture::new();
    assert!(evidence::run(&f.0, "proof", PIN).unwrap());
    let first = f.report();
    record_history(&f.0, &first);
    f.replace(
        "PORTS.toml",
        "status = \"ported\"",
        "status = \"in-progress\"",
    );
    let second = f.report();
    assert_eq!(first.evidence_artifacts, second.evidence_artifacts);
    assert_eq!(second.evidence_states["proof"], "current");
    record_history(&f.0, &second);
    let entries: Vec<serde_json::Value> = fs::read_to_string(f.0.join("status/history.jsonl"))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(entries.len(), 2);
    assert_eq!(
        entries[0]["evidence_artifacts"],
        entries[1]["evidence_artifacts"]
    );
    assert_ne!(entries[0]["files_verified"], entries[1]["files_verified"]);
}
