//! Node and symbol ownership: arenas, checked ids, lazy storage, bundles and
//! leases (ADRs 0006 and 0007).
//!
//! This is the second Phase 0 contract leaf. It fixes identity and ownership,
//! not node layout: the header and payload are left to measurement, and
//! [`node::NodeData`] is the placeholder the generated AST replaces.
//!
//! The invariants this crate exists to hold:
//!
//! * arena ids come from one process-global counter and are **never reused**, so
//!   a stale id cannot alias a newer arena and there is no ABA window;
//! * slots are never reused within an arena, and arenas are append-only;
//! * ids never keep storage alive, so anything that stores an id must retain the
//!   owner too;
//! * resolution goes through a [`scope::Scope`] that holds the owners, and an id
//!   from an arena it does not hold is rejected rather than aliased;
//! * exhaustion is refused before any counter can wrap or publish a reused id.
//!
//! The design note is docs/design/ownership.md. The pieces this crate leaves for
//! later sprints are named there: transform and builder owners, the generation
//! gate and the retirement-versus-publication ordering are S08 and S09 work, and
//! nothing here pretends to implement them.
//!
//! No `unsafe` is used. The lazy arena's stable node addresses come from
//! separately allocated, reference-counted pages whose slots are `OnceLock`s,
//! not from raw pointers, which is what lets a published node be read after its
//! guard is dropped without any aliasing argument.

pub mod checker;
pub mod core_arena;
pub mod counters;
pub mod ids;
pub mod lazy;
pub mod node;
pub mod owner;
pub mod scope;

pub use checker::{CheckerId, CheckerOwner, CheckerPool, Lease, PoolGeneration, StaleId};
pub use core_arena::CoreArena;
pub use counters::{live_allocations, live_owners};
pub use ids::{allocate_arena_id, ArenaId, Exhausted, NodeId, Slot, SlotCounter, SymbolId, MAX_ID};
pub use lazy::{LazyArena, LazyError, LazyRef, TokenKey, PAGE_SIZE};
pub use node::{NodeData, NodeFlags, NodeKind};
pub use owner::{
    BundleOwner, ContentMapperInfo, FileId, FileOwner, FileOwnerBuilder, ScratchOwner,
};
pub use scope::{with_scoped_arena, LocalNode, NodeRef, OwnerKind, Scope, ScopedArena};

#[cfg(test)]
mod tests;
