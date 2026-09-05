# ADR 0015: Generated code from the pinned schemas, with upstream's extractors authoritative

Status: Accepted (2026-09-05)
Plan: section 6, decision 10; section 13, items 12 and 13

## Context

Upstream generates its AST, encoder, LSP types, diagnostics and API protocol from schemas and extractors it owns: `ast.json` through the resolver in `tools/scripts/tsc/schema.ts`, the LSP metamodel, `diagnosticMessages.json`, and `tools/gen-proto`, which derives the API contract from dispatch code, annotations, serialization tags and special mappings. A second interpreter of any of these would disagree silently.

## Decision

The pinned resolvers and extractors remain authoritative throughout the port. Local adapters run the pinned AST and LSP resolvers and export normalized data for Rust emitters; diagnostics come from the pinned messages; a carried patch adds JSON export to the pinned `gen-proto`, and `api.json` is a generated artifact refreshed on every pin bump, never an edited source. Rust DTOs, method identifiers, dispatch metadata and ordinary field serialization are generated; special codecs keep explicit adapters inventoried beside the export and covered by wire fixtures. Generated TypeScript is produced into a separate directory and must be byte-identical to the untouched pinned client's file before an exporter change or a pin bump is accepted; the original pinned client is what is built and tested against Rust. Libs are embedded with `include_str!` behind a `noembed` feature; locale files stay gzipped.

## Consequences

Patches live in a disposable tooling worktree; the oracle is built from the unmodified pin. Any transfer of schema ownership to this repository is a separate decision.

## Evidence

`tools/scripts/tsc/schema.ts` (`SchemaAPI`), `tools/scripts/tsc/generate.ts`, `tsc/internal/lsp/lsproto/_generate/generate.mts`, `tools/gen-proto/main.go` (`discoverSessionMethods`, tag handling), `tsc/internal/diagnostics/generate.go`.

## Amendments

Draft 3 captured `api.json` once and maintained it locally; draft 3.1 derived it from the extractor; draft 3.2 made the pinned extractors authoritative for the whole port.
