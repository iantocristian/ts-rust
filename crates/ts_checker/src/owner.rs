//! One checker owner, one operation at a time (plan §4.1; ADR 0007, ADR 0012).
//!
//! `CheckerOwner` holds the exact checker identity and every piece of mutable
//! checker state behind it. An [`Operation`] is the only way to reach that
//! state: it holds the identity's exclusive permit and the state lock together,
//! and drops both together. Inside an operation the checker is a plain
//! single-threaded `&mut` state machine; no lock is taken per node, type,
//! symbol or relation edge.
//!
//! Reentry on the same thread is a bug in the caller, not contention, and is
//! reported before waiting: the thread-local set of active identity leases answers it
//! without touching the lock. Contention from another thread waits normally.
//! A panic inside an operation retires the shared generation before the permit
//! is released (`CheckerLease`'s drop does this); the poisoned state lock is a
//! backstop, never the recovery mechanism.
//!
//! Two lock acquisitions per operation is a known cost of reusing the `ts_arena`
//! permit unchanged. Operations are per query, not per node; P1 measures it and
//! folds the state into the permit if it shows.

use crate::{Error, ResolutionStack, TypeStore};
use std::sync::{Arc, Mutex, MutexGuard};
use ts_arena::{CheckerIdentity, CheckerLease, Counters, SymbolArena};
use ts_ast::Symbol;

/// Everything a checker mutates. Reachable only through an [`Operation`].
#[allow(dead_code, reason = "private state is completed by the P1/P2 families")]
pub(crate) struct CheckerState {
    /// Checker-owned symbols: merged clones, transient and synthetic symbols.
    /// Its arena identity is the checker identity (`CheckerIdentity::id`).
    symbols: SymbolArena<Symbol>,
    types: TypeStore,
    resolution: ResolutionStack,
}

#[allow(dead_code, reason = "private state is completed by the P1/P2 families")]
impl CheckerState {
    pub fn symbols(&self) -> &SymbolArena<Symbol> {
        &self.symbols
    }
    pub fn symbols_mut(&mut self) -> &mut SymbolArena<Symbol> {
        &mut self.symbols
    }
    pub fn types(&self) -> &TypeStore {
        &self.types
    }
    pub fn types_mut(&mut self) -> &mut TypeStore {
        &mut self.types
    }
    pub fn resolution(&self) -> &ResolutionStack {
        &self.resolution
    }
    pub fn resolution_mut(&mut self) -> &mut ResolutionStack {
        &mut self.resolution
    }
}

pub struct CheckerOwner {
    identity: Arc<CheckerIdentity>,
    state: Mutex<CheckerState>,
}

impl CheckerOwner {
    /// Adopts the identity's reserved arena number as the checker symbol arena.
    /// Each identity can back one owner; a second attempt fails with
    /// `IdentityAdopted` rather than minting a second, inconsistent identity.
    pub fn new(identity: Arc<CheckerIdentity>, counters: &Counters) -> Result<Self, Error> {
        let symbols = identity.adopt_symbol_arena(counters)?;
        Ok(Self {
            identity,
            state: Mutex::new(CheckerState {
                symbols,
                types: TypeStore::new(),
                resolution: ResolutionStack::new(),
            }),
        })
    }

    pub fn identity(&self) -> &Arc<CheckerIdentity> {
        &self.identity
    }

    /// Begins an exclusive operation: validates the generation, takes the permit,
    /// then the state. Fails with [`Error::Reentry`] if this thread already holds
    /// an operation on this owner, and with `Retired` once the generation is gone.
    pub fn operation(&self) -> Result<Operation<'_>, Error> {
        let lease = self.identity.lease()?;
        let state = self.state.lock().map_err(|_| {
            self.identity.generation().retire();
            Error::Arena(ts_arena::Error::Retired)
        })?;
        Ok(Operation { lease, state })
    }
}

/// An exclusive operation scope. Mutable storage never escapes this crate.
///
/// ```compile_fail
/// use ts_checker::Operation;
/// fn swap_checkers(a: &mut Operation<'_>, b: &mut Operation<'_>) {
///     std::mem::swap(a.state_mut(), b.state_mut());
/// }
/// ```
///
/// ```compile_fail
/// use ts_checker::{TypeStore, TypeId};
/// fn foreign_id(store: &TypeStore, id: TypeId) { store.get(id); }
/// ```
///
/// Field order is the drop order: the lease goes
/// first so a panicking unwind retires the generation before the state unlocks.
pub struct Operation<'a> {
    lease: CheckerLease<'a>,
    state: MutexGuard<'a, CheckerState>,
}

impl Operation<'_> {
    pub fn lease(&self) -> &CheckerLease<'_> {
        &self.lease
    }
    #[allow(dead_code, reason = "private state access used by the P1/P2 families")]
    pub(crate) fn state(&self) -> &CheckerState {
        &self.state
    }
    #[allow(dead_code, reason = "private state access used by the P1/P2 families")]
    pub(crate) fn state_mut(&mut self) -> &mut CheckerState {
        &mut self.state
    }
}
