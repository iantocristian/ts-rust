//! Live owner and allocation counters for the E3 baseline checks.

use std::sync::atomic::{AtomicI64, Ordering};

static LIVE_OWNERS: AtomicI64 = AtomicI64::new(0);
static LIVE_ALLOCATIONS: AtomicI64 = AtomicI64::new(0);

/// A snapshot of both counters.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Counters {
    pub owners: i64,
    pub allocations: i64,
}

pub fn snapshot() -> Counters {
    Counters {
        owners: LIVE_OWNERS.load(Ordering::SeqCst),
        allocations: LIVE_ALLOCATIONS.load(Ordering::SeqCst),
    }
}

pub(crate) struct OwnerGuard;

impl OwnerGuard {
    pub(crate) fn new() -> Self {
        LIVE_OWNERS.fetch_add(1, Ordering::SeqCst);
        Self
    }
}

impl Drop for OwnerGuard {
    fn drop(&mut self) {
        LIVE_OWNERS.fetch_sub(1, Ordering::SeqCst);
    }
}

pub(crate) struct AllocationGuard;

impl AllocationGuard {
    pub(crate) fn new() -> Self {
        LIVE_ALLOCATIONS.fetch_add(1, Ordering::SeqCst);
        Self
    }
}

impl Drop for AllocationGuard {
    fn drop(&mut self) {
        LIVE_ALLOCATIONS.fetch_sub(1, Ordering::SeqCst);
    }
}
