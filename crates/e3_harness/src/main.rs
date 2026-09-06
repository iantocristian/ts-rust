//! The e3 producer's Rust half: runs the S04 ownership scenarios in this build
//! configuration and prints their metrics as one JSON object. The Python
//! producer adds the Miri and AddressSanitizer results.

use e3_harness::{run_all, Config};

fn main() {
    let report = run_all(Config::release());
    let mut metrics = serde_json::Map::new();
    let mut failures = Vec::new();
    for (name, outcome) in [
        ("id_exhaustion", &report.id_exhaustion),
        ("wrong_owner_rejected", &report.wrong_owner_rejected),
        (
            "stale_and_recycled_ids_rejected",
            &report.stale_and_recycled_ids_rejected,
        ),
        ("concurrent_lazy_storage", &report.concurrent_lazy_storage),
        ("mapper_bundle_disposal", &report.mapper_bundle_disposal),
    ] {
        metrics.insert(name.to_string(), serde_json::Value::Bool(outcome.is_ok()));
        if let Err(why) = outcome {
            failures.push(format!("{name}: {why}"));
        }
    }
    metrics.insert(
        "live_owner_delta".into(),
        serde_json::Value::from(report.live_owner_delta),
    );
    metrics.insert(
        "live_allocation_delta".into(),
        serde_json::Value::from(report.live_allocation_delta),
    );
    for f in &failures {
        eprintln!("e3: {f}");
    }
    println!("{}", serde_json::json!({ "metrics": metrics }));
}
