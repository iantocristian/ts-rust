//! The E3 harness: ownership scenarios plus the live owner and allocation
//! counters, reported as the metrics `status/experiments.toml` names.
//!
//! One JSON object goes to stdout; every explanation goes to stderr. Only the
//! criteria whose scenarios exist are emitted, so the rest stay pending rather
//! than being reported as passing. The producer contract is documented in
//! `docs/harnesses.md`.
//!
//! `--scenarios` runs the same scenarios and exits non-zero on failure without
//! emitting metrics; that is the entry point the Miri and AddressSanitizer runs
//! use, because those measure the same code under a different runtime rather
//! than producing a second set of results.

mod scenarios;

use std::collections::BTreeMap;
use std::process::ExitCode;

use scenarios::Outcome;

/// The criterion each scenario settles, in `status/experiments.toml`.
const CRITERIA: &[(&str, &str)] = &[
    ("id_exhaustion", "id_exhaustion"),
    ("wrong_owner_rejected", "wrong_owner_rejected"),
    (
        "stale_and_recycled_ids_rejected",
        "stale_and_recycled_ids_rejected",
    ),
    ("concurrent_lazy_storage", "concurrent_lazy_storage"),
    ("mapper_bundle_disposal", "mapper_bundle_disposal"),
];

fn report(outcomes: &[Outcome]) {
    for outcome in outcomes {
        let failures: Vec<&str> = outcome
            .checks
            .iter()
            .filter(|(_, ok)| !*ok)
            .map(|(what, _)| what.as_str())
            .collect();
        eprintln!(
            "{}: {} ({} checks, owner delta {}, allocation delta {})",
            outcome.name,
            if outcome.passed { "pass" } else { "FAIL" },
            outcome.checks.len(),
            outcome.owner_delta,
            outcome.allocation_delta,
        );
        for failure in failures {
            eprintln!("    failed: {failure}");
        }
    }
}

fn main() -> ExitCode {
    let scenarios_only = std::env::args().any(|arg| arg == "--scenarios");
    let quiet = scenarios::QuietPanics::install();
    let outcomes = scenarios::all();
    drop(quiet);
    if scenarios::reduced_scale() {
        eprintln!("TS_E3_SCALE=reduced: the concurrent scenario runs its smaller workload");
    }
    report(&outcomes);

    let passed = outcomes.iter().all(|outcome| outcome.passed);
    let owner_delta = outcomes
        .iter()
        .map(|outcome| outcome.owner_delta.abs())
        .max()
        .unwrap_or(0);
    let allocation_delta = outcomes
        .iter()
        .map(|outcome| outcome.allocation_delta.abs())
        .max()
        .unwrap_or(0);
    let counters_clean = owner_delta == 0 && allocation_delta == 0;

    if scenarios_only {
        eprintln!(
            "scenarios {}; max owner delta {owner_delta}, max allocation delta {allocation_delta}",
            if passed { "passed" } else { "FAILED" }
        );
        return if passed && counters_clean {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        };
    }

    let by_name: BTreeMap<&str, &Outcome> = outcomes
        .iter()
        .map(|outcome| (outcome.name, outcome))
        .collect();
    let mut metrics = Vec::new();
    for (scenario, criterion) in CRITERIA {
        let outcome = by_name
            .get(scenario)
            .unwrap_or_else(|| panic!("scenario {scenario} is registered but did not run"));
        metrics.push(format!("\"{criterion}\":{}", outcome.passed));
    }
    // The counter criteria are the maximum absolute deviation over every
    // scenario, so one leak anywhere fails them.
    metrics.push(format!("\"live_owner_delta\":{owner_delta}"));
    metrics.push(format!("\"live_allocation_delta\":{allocation_delta}"));
    println!("{{\"metrics\":{{{}}}}}", metrics.join(","));
    ExitCode::SUCCESS
}
