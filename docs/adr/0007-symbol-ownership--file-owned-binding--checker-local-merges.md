# ADR 0007: Symbol ownership: file-owned binding, checker-local merges, checker-owned types

Status: Proposed (2026-09-05)
Plan: section 6, decision 2

## Context

Corsa binds files in the parse cache before any program exists and shares the bound files, symbols included, across programs and snapshots by reference count. Declaration merging is checker-local: `Checker.mergedSymbols`, `cloneSymbol` and `mergeSymbol` allocate and mutate checker-owned symbols, so two parallel checkers merge independently. Types and signatures are created per checker and must never be mixed across checkers.

## Decision

The binder allocates a file's symbols in that file owner's arena at bind time. Merged symbols, their mapping and all transient symbols remain checker-local. A symbol id distinguishes a file owner from a checker owner and validates its generation and slot. Types and signatures index the checker's arenas and carry checked checker provenance; every pointer-keyed map and `LinkStore` becomes a side table keyed by these identities. Symbol tables remain unordered hash maps keyed by `JsString`, with ordering produced at the same observation points as Go (ADR 0010).

ADR 0006's validated ownership-scope contract governs any release check elision. Cache results, cross-checker/arena imports, stored `Arc<[TypeId]>` lists and callback/reentrant paths either preserve the same non-forgeable owner/generation proof or validate before use. An `Arc` retains its list, not automatically the checker's arena. Ordinary typed IDs and lifetime parameters do not prevent an in-range ID from another checker being used; the design must provide fresh invariant brands, an encapsulated validated handle API, or runtime owner/generation checks. Live storage and an active generation are distinct, including while panic-retired leases drain.

## Consequences

Dropping a checker frees its type universe; dropping a program never touches a shared file. Draft 2's program-level overlay for merged symbols is withdrawn because it would break isolation between parallel checkers. Required E3 verification covers two programs sharing one bound file; disposal of one while the other keeps checking; an edit while an old snapshot answers requests; two checkers merging declarations over shared files; and wrong-checker, stale, recycled and retired-generation lookups failing explicitly in release mode, including through caches and callback reentry. Proven check-elision paths retain debug assertions; sanitizers supplement the identity assertions. This ADR remains Proposed until the owner has reviewed the design note.

## Evidence

`tsc/internal/project/parsecache.go` (binding on cache entry), `tsc/internal/checker/checker.go` (`cloneSymbol`, `getMergedSymbol`, `mergedSymbols`), `tsc/internal/compiler/checkerpool.go` (per-checker universes), `tsc/internal/api/session.go` (symbol registry keyed by global symbol ids).

## Amendments

Draft 3.2 moved merged symbols from a program overlay to checker-local ownership after the third review.

The tracking-scaffold review applied ADR 0006's conditional check-elision and active-generation requirements to symbol, type and signature handles.
