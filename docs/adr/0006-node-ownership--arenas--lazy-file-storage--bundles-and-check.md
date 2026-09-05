# ADR 0006: Node ownership: arenas, lazy file storage, bundles and checked identities

Status: Proposed (2026-09-05)
Plan: section 6, decision 1

## Context

Corsa's AST is a pointer graph with parent links and late mutation. Nodes have several owners: source files, transforms (with original-node side maps), the checker's node builder (with a release protocol), the API session, and scratch factories. Binding does not end file-owned allocation: `SourceFile.resolveJSDoc` and `GetOrCreateToken` allocate and cache nodes on demand under locks, and content mappers link a canonical file and its supplemental files in both directions. Rust needs these rules stated.

## Decision

A file owner retains the immutable parsed and bound core plus synchronized append-only storage for lazy JSDoc and language-service tokens; lazy nodes are published only after initialization, keep stable ids, and live until the file owner is released. A content-mapper bundle owns its canonical and supplemental files together, with cyclic links inside the bundle as non-owning ids. Transform, checker-builder and API-session owners retain their synthetic arenas; options and formatter scratch factories use the same scoped ownership. A node id identifies an arena generation and slot, resolved through a live owner; the compact representation is 64 bits, with packing, overflow and registry protocol fixed in the design note before generation. Logical links (parent, original, bundle) may be cyclic; owning references between owners must be acyclic, with mutually dependent files grouped under one bundle owner. Transform and builder owners retain the file or bundle owners they reference; a builder arena is released only after every cache entry and returned handle retaining it is released. Generated accessors hide the node layout, which is chosen by measurement (section 13, item 6).

## Consequences

Ids do not retain storage; snapshots, bundles, caches and returned handles retain owners explicitly. Draft 1's single immutable arena per file and its newer-to-older reference rule are withdrawn. Generation and provenance checks are boundary checks (registries, snapshots, public handles); internal accesses within an owner on ids it issued should not pay a per-access generation check in release builds, which the design note must state explicitly. This ADR becomes Accepted when the owner has reviewed the design note with the Go evidence attached.

## Evidence

`tsc/internal/ast/ast.go` (`resolveJSDoc`, `GetOrCreateToken`), `tsc/internal/printer/emitcontext.go` (original-node map), `tsc/internal/checker/nodebuilder.go` (`ReleaseArenas`), `tsc/internal/contentmapper/transform.go` (bundle links), `tsc/internal/project/parsecache.go` (`ContentMappedParseCache`), `tsc/internal/api/encoder/encoder.go` (protocol 8 layout).

## Amendments

Draft 2 introduced four arena kinds; draft 3.2 added lazy file storage, bundle ownership and generation-checked identities after the third review.
