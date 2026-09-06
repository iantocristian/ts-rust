//! Live owner and allocation counters.
//!
//! E3 requires that after every scenario's final drop the live owner and live
//! allocation counts equal the pre-scenario baseline, so both are maintained by
//! the production owners and arenas rather than by the harness.

use std::sync::atomic::{AtomicI64, Ordering};

static LIVE_OWNERS: AtomicI64 = AtomicI64::new(0);
static LIVE_ALLOCATIONS: AtomicI64 = AtomicI64::new(0);

/// Owners alive right now: file, bundle, checker, transform, builder and
/// scratch contexts.
pub fn live_owners() -> i64 {
    LIVE_OWNERS.load(Ordering::SeqCst)
}

/// Nodes alive right now across every arena.
pub fn live_allocations() -> i64 {
    LIVE_ALLOCATIONS.load(Ordering::SeqCst)
}

/// Increments the live owner count for as long as it exists.
#[derive(Debug)]
pub struct OwnerGuard(());

impl OwnerGuard {
    pub fn new() -> Self {
        LIVE_OWNERS.fetch_add(1, Ordering::SeqCst);
        Self(())
    }
}

impl Default for OwnerGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for OwnerGuard {
    fn drop(&mut self) {
        LIVE_OWNERS.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Accounts for the nodes of one arena; the arena releases them all at once,
/// because there is no per-node free.
#[derive(Debug)]
pub struct AllocationGuard {
    live: i64,
}

impl AllocationGuard {
    pub fn new() -> Self {
        Self { live: 0 }
    }

    pub fn record(&mut self, nodes: i64) {
        self.live += nodes;
        LIVE_ALLOCATIONS.fetch_add(nodes, Ordering::SeqCst);
    }

    pub fn live(&self) -> i64 {
        self.live
    }
}

impl Default for AllocationGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AllocationGuard {
    fn drop(&mut self) {
        LIVE_ALLOCATIONS.fetch_sub(self.live, Ordering::SeqCst);
    }
}
