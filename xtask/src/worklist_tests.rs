use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct OutputDir(PathBuf);
impl OutputDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "tracker-worklist-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn json(&self, path: &str) -> serde_json::Value {
        serde_json::from_slice(&fs::read(self.0.join(path)).unwrap()).unwrap()
    }
}
impl Drop for OutputDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn report() -> Report {
    Report {
        generated: "2026-09-06".into(),
        metrics: BTreeMap::from([
            ("functions.total".into(), Metric::Num(4.0)),
            ("functions.ported".into(), Metric::Num(1.0)),
        ]),
        ledger_pin: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
        files_in_scope: Vec::new(),
        unported_by_package: BTreeMap::from([
            (
                "scanner".into(),
                vec!["tsc/internal/scanner/scanner.go:Scanner.Scan".into()],
            ),
            (
                "checker".into(),
                vec![
                    "tsc/internal/checker/checker.go:Checker.Zeta".into(),
                    "tsc/internal/checker/checker.go:Checker.Alpha".into(),
                ],
            ),
        ]),
        unknown_markers: Vec::new(),
        experiments: Vec::new(),
        sprints: Vec::new(),
        adrs: Vec::new(),
        context: None,
        evidence_states: BTreeMap::new(),
        evidence_artifacts: BTreeMap::new(),
        errors: Vec::new(),
    }
}

#[test]
fn summary_references_a_worklist_whose_counts_and_checksum_match() {
    let output = OutputDir::new();
    let report = report();
    write_status_json(&output.0, &report);

    let summary = output.json("status/status.json");
    assert!(summary.get("unmapped_functions").is_none());
    assert_eq!(
        summary["unmapped_functions_file"],
        "status/unmapped-functions.json"
    );
    let worklist_path = summary["unmapped_functions_file"].as_str().unwrap();
    let bytes = fs::read(output.0.join(worklist_path)).unwrap();
    assert_eq!(summary["unmapped_functions_sha256"], evidence::hash(&bytes));

    let worklist = output.json(worklist_path);
    assert_eq!(worklist["schema_version"], 1);
    assert_eq!(worklist["pin"], report.ledger_pin);
    let mut total = 0;
    for (package, functions) in worklist["unmapped_functions"].as_object().unwrap() {
        let functions = functions.as_array().unwrap();
        total += functions.len();
        assert_eq!(
            summary["unported_functions_by_package"][package],
            functions.len()
        );
        for function in functions {
            assert!(!summary.to_string().contains(function.as_str().unwrap()));
        }
    }
    assert_eq!(
        total as f64,
        num(&report.metrics, "functions.total") - num(&report.metrics, "functions.ported")
    );
}

#[test]
fn worklist_bytes_are_stable_across_input_order_and_repeated_writes() {
    let output = OutputDir::new();
    let mut report = report();
    write_status_json(&output.0, &report);
    let first = fs::read(output.0.join("status/unmapped-functions.json")).unwrap();
    report
        .unported_by_package
        .get_mut("checker")
        .unwrap()
        .reverse();
    write_status_json(&output.0, &report);
    assert_eq!(
        first,
        fs::read(output.0.join("status/unmapped-functions.json")).unwrap()
    );
    let worklist = output.json("status/unmapped-functions.json");
    assert_eq!(
        worklist["unmapped_functions"]["checker"][0],
        "tsc/internal/checker/checker.go:Checker.Alpha"
    );
}

#[test]
fn empty_worklist_is_emitted_and_both_reports_link_to_it() {
    let output = OutputDir::new();
    let mut report = report();
    report.unported_by_package.clear();
    report
        .metrics
        .insert("functions.ported".into(), Metric::Num(4.0));
    write_status_json(&output.0, &report);
    write_status_md(&output.0, &report);
    write_dashboard(&output.0, &report);

    let worklist = output.json("status/unmapped-functions.json");
    assert_eq!(worklist["unmapped_functions"], serde_json::json!({}));
    assert_eq!(
        output.json("status/status.json")["unported_functions_by_package"],
        serde_json::json!({})
    );
    assert!(fs::read_to_string(output.0.join("STATUS.md"))
        .unwrap()
        .contains("](status/unmapped-functions.json)"));
    assert!(fs::read_to_string(output.0.join("docs/status.html"))
        .unwrap()
        .contains("href=\"../status/unmapped-functions.json\""));
}
