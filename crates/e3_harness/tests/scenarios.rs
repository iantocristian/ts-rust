//! The same scenarios as assertions, so `cargo test`, Miri and AddressSanitizer
//! exercise them with the reduced configuration where needed.

use std::sync::Mutex;

use e3_harness::{
    concurrent_lazy_storage, id_exhaustion, mapper_bundle_disposal,
    stale_and_recycled_ids_rejected, wrong_owner_rejected, Config,
};

/// The live-owner counters are process-global, so scenarios that compare them
/// against a baseline must not overlap with other tests in this process.
static SERIAL: Mutex<()> = Mutex::new(());

#[test]
fn id_exhaustion_scenario() {
    let _serial = SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    id_exhaustion().unwrap();
}

#[test]
fn wrong_owner_rejected_scenario() {
    let _serial = SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    wrong_owner_rejected().unwrap();
}

#[test]
fn stale_and_recycled_ids_rejected_scenario() {
    let _serial = SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    stale_and_recycled_ids_rejected().unwrap();
}

#[test]
fn concurrent_lazy_storage_scenario() {
    let _serial = SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    concurrent_lazy_storage(Config::for_tests()).unwrap();
}

#[test]
fn mapper_bundle_disposal_scenario() {
    let _serial = SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    mapper_bundle_disposal().unwrap();
}
