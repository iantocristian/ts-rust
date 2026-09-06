//! Checked identities and ownership for compiler storage.
//!
//! Payloads are supplied by the AST/binder crates. This crate implements their
//! storage and retention mechanisms, not an alternate AST or semantic checker.
//! All imports are checked in release builds; no unsafe fast path is exposed.
#![forbid(unsafe_code)]

mod counters;
mod ids;
mod lease;
mod owners;
mod storage;

pub use counters::{Counters, Counts};
pub use ids::{ArenaId, FileId, NodeId, SymbolId};
pub use lease::{CheckerIdentity, CheckerLease, Generation};
pub use owners::{
    BundleOwner, FileBuilder, FileHandle, FileOwner, LazyTransaction, NodeListRef, NodeRef, Scope,
    ScratchOwner, SymbolRef, TokenKey,
};
pub use storage::Node;

/// Failure at an ownership, bounds, or publication boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidId,
    WrongOwner,
    InvalidSlot,
    Retired,
    InvalidGraph,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}

#[cfg(any(test, feature = "harness"))]
pub mod scenarios;

#[cfg(test)]
mod tests;
