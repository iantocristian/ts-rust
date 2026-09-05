# ADR 0008: Checker mutation model

Status: Accepted (2026-09-05)
Plan: section 6, decision 3; section 13, item 7

## Context

The Go checker mutates anything through `*Checker` while holding pointers into its own graph. Rust cannot hold a borrow into an arena across a recursive `&mut self` call.

## Decision

The checker is a single-threaded state machine that takes `&mut self` and never holds a borrow into its arenas across a call: accessors copy small values out, scalar lazily-resolved fields use `Cell`, collections are addressed by id. Type, parameter and type-argument lists are built mutably and published as independently owned immutable handles (`Arc<[TypeId]>`, since the checker is `Send`). Lazy resolution keeps Corsa's `pushTypeResolution` and `popTypeResolution` reentrancy guard. `isRelatedTo`, instantiation, inference and control flow keep Corsa's structure, and keep its order of side effects wherever ADR 0010 lists the output as order-sensitive.

## Consequences

The port ledger depends on Go and Rust having the same shape so upstream diffs transplant. The arena-references-with-interior-mutability alternative (rustc's pattern) is prototyped on the relater core in the spike so that it is a measured fallback, not an assumed one. Rejected: `RefCell` on every table; splitting the checker into pure passes.

## Evidence

`tsc/internal/checker/checker.go` (`getTypeArguments` and the resolution stack), `tsc/internal/checker/relater.go`.
