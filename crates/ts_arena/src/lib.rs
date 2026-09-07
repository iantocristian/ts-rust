//! Compiler storage: checked identities, immutable files and transactional lazy nodes.
//! Payloads come from the AST and binder; this crate supplies storage and retention.
#![forbid(unsafe_code)]

mod arena;
mod bundle;
mod counters;
mod error;
mod file;
mod ids;
mod lazy;
mod lease;
mod node;
mod refs;
mod scope;
mod scratch;

pub use bundle::{BundleOwner, FileHandle};
pub use counters::{Counters, Counts};
pub use error::Error;
pub use file::{FileBuilder, FileOwner};
pub use ids::{ArenaId, FileId, NodeId, SymbolId};
pub use lazy::{LazyTransaction, TokenKey};
pub use lease::{CheckerIdentity, CheckerLease, Generation};
pub use node::Node;
pub use refs::{NodeListRef, NodeRef, RetainedNode, RetainedSymbol, SymbolRef};
pub use scope::{LocalArena, LocalNode, Scope};
pub use scratch::ScratchOwner;

#[cfg(any(test, feature = "harness"))]
pub mod scenarios;

#[cfg(test)]
mod tests;
