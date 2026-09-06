//! Checked owner and arena ids, lazy storage, bundle ownership and validated
//! scopes: the contract of ADR 0006, docs/design/ownership.md and the id rules
//! of docs/design/symbols.md.

pub mod arena;
pub mod counters;
pub mod id;
pub mod lazy;
pub mod owner;
pub mod scope;

pub use arena::Arena;
pub use counters::{snapshot, Counters};
pub use id::{ArenaCounter, ArenaExhausted, ArenaId, NodeId, SlotExhausted, SymbolId};
pub use lazy::{LazyAlloc, LazyArena, LazyRef, PAGE_SIZE};
pub use owner::{BundleLinks, BundleOwner, FileId, FileOwner, ScratchOwner};
pub use scope::{LocalArena, NodeRef, Scope, Slot, StaleId};
