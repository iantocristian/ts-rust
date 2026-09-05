# ADR 0007: Symbol ownership: file-owned binding, checker-local merges, checker-owned types

Status: Accepted (2026-09-05)
Design note: [docs/design/symbols.md](../design/symbols.md), reviewed and accepted by the owner on 2026-09-05
Plan: section 6, decision 2

## Context

Corsa binds files in the parse cache before any program exists and shares the bound files, symbols included, across programs and snapshots by reference count. Declaration merging is checker-local: `Checker.mergedSymbols`, `cloneSymbol` and `mergeSymbol` allocate and mutate checker-owned symbols, so two parallel checkers merge independently. Types and signatures are created per checker and must never be mixed across checkers.

## Decision

The binder allocates a file's symbols in that file owner's arena at bind time. Merged symbols, their mapping and all transient symbols remain checker-local. A symbol id distinguishes a file owner from a checker owner and validates its generation and slot. Types and signatures index the checker's arenas and carry checked checker provenance; every pointer-keyed map and `LinkStore` becomes a side table keyed by these identities. Symbol tables remain unordered hash maps keyed by `JsString`, with ordering produced at the same observation points as Go (ADR 0010).

ADR 0006's validated ownership-scope contract governs any release check elision. Cache results, cross-checker/arena imports, stored `Arc<[TypeId]>` lists and callback/reentrant paths either preserve the same non-forgeable owner/generation proof or validate before use. An `Arc` retains its list, not automatically the checker's arena. Ordinary typed IDs and lifetime parameters do not prevent an in-range ID from another checker being used; the design must provide fresh invariant brands, an encapsulated validated handle API, or runtime owner/generation checks. Live storage and an active generation are distinct, including while panic-retired leases drain.

## Consequences

The final checker-owner reference releases its type universe, including synthetic AST storage; retained results can outlive a pool lease. Dropping a program does not dispose a file still retained elsewhere. Draft 2's program-level overlay for merged symbols is withdrawn because it would break isolation between parallel checkers. Required E3 verification covers two programs sharing one bound file; disposal of one while the other keeps checking; an edit while an old snapshot answers requests; two checkers merging declarations over shared files; and wrong-checker, stale, recycled and retired-generation lookups failing explicitly in release mode, including through caches and callback reentry. Proven check-elision paths retain debug assertions; sanitizers supplement the identity assertions. Accepted on 2026-09-05 after the owner reviewed the design note and its Go evidence; the E3 and E4 assertions remain required verification, not completed verification.

## Evidence

`tsc/internal/project/parsecache.go` (binding on cache entry), `tsc/internal/checker/checker.go` (`cloneSymbol`, `getMergedSymbol`, `mergedSymbols`), `tsc/internal/compiler/checkerpool.go` (per-checker universes), `tsc/internal/api/session.go` (symbol registry keyed by global symbol ids).

## Amendments

Draft 3.2 moved merged symbols from a program overlay to checker-local ownership after the third review.

The tracking-scaffold review applied ADR 0006's conditional check-elision and active-generation requirements to symbol, type and signature handles.

The design-note correction pass defines retained results carrying exact checker identity, pool generation and an owning root, with release-build validation before entering an operation. Private local slots cannot escape into service results or external caches alone. Checker identity is distinct from a pool generation; symbol owner kind comes from arena metadata so the full 32-bit arena counter fits. Checker-created AST nodes share the checker's storage lifetime, and internal caches do not retain their own enclosing checker through an owning cycle.
