# ADR 0007: Symbol ownership: file-owned binding, checker-local merges, checker-owned types

Status: Proposed (2026-09-05)
Plan: section 6, decision 2

## Context

Corsa binds files in the parse cache before any program exists and shares the bound files, symbols included, across programs and snapshots by reference count. Declaration merging is checker-local: `Checker.mergedSymbols`, `cloneSymbol` and `mergeSymbol` allocate and mutate checker-owned symbols, so two parallel checkers merge independently. Types and signatures are created per checker and must never be mixed across checkers.

## Decision

The binder allocates a file's symbols in that file owner's arena at bind time. Merged symbols, their mapping and all transient symbols remain checker-local. A symbol id distinguishes a file owner from a checker owner and validates its generation and slot. Types and signatures index the checker's arenas and carry checked checker provenance at boundaries; every pointer-keyed map and `LinkStore` becomes a side table keyed by these identities. Symbol tables remain unordered hash maps keyed by `JsString`, with ordering produced at the same observation points as Go (ADR 0010).

## Consequences

Dropping a checker frees its type universe; dropping a program never touches a shared file. Draft 2's program-level overlay for merged symbols is withdrawn because it would break isolation between parallel checkers. Verified by E3: two programs sharing one bound file; disposal of one while the other keeps checking; an edit while an old snapshot answers requests; two checkers merging declarations over shared files; wrong-checker, stale and recycled-id lookups failing explicitly. This ADR becomes Accepted when the owner has reviewed the design note.

## Evidence

`tsc/internal/project/parsecache.go` (binding on cache entry), `tsc/internal/checker/checker.go` (`cloneSymbol`, `getMergedSymbol`, `mergedSymbols`), `tsc/internal/compiler/checkerpool.go` (per-checker universes), `tsc/internal/api/session.go` (symbol registry keyed by global symbol ids).

## Amendments

Draft 3.2 moved merged symbols from a program overlay to checker-local ownership after the third review.
