# S08 data

Frozen inputs and Go observations for the checker sprint. Files here are
regenerated from the pinned upstream commit, never edited by hand; each records
its derivation.

| File | Contents | Derivation |
| --- | --- | --- |
| `checker-flag-observations.json` | Every constant of `TypeFlags`, `ObjectFlags`, `SignatureFlags`, `TypeFormatFlags`, `SymbolFormatFlags`, `VarianceFlags`, `AccessFlags`, `NodeCheckFlags`, `ContextFlags`, `ParseFlags`, `ExternalEmitHelpers`, `Ternary` and `TypeSystemPropertyName` in `internal/checker`, `Flags` and `InternalFlags` in `internal/nodebuilder`, the type-display constants, and `unsafe.Sizeof` of the checker's type, signature and symbol records | Two test files added through `go test -overlay` (the checkout is not modified) read the values out of the compiled pinned packages and write them as JSON; the Rust flag ports assert equality in `crates/ts_checker/src/flag_tests.rs` and `crates/ts_nodebuilder/src/tests.rs` |

The struct sizes are the Go layout on the recording host (darwin/arm64, the
pinned `go1.27.1`). They are the baseline for the per-type footprint census in
plan §6.1: upstream's `Type` header is embedded in every payload struct, so one
Go type costs its payload struct plus its alias record and owned lists.

Planned additions, each frozen at P0 before any Rust result exists (plan §§5–6):
`type-footprint.json` (census membership and sharing rules), `checker-workload.json`
(the fixed query schedule for `checkerbench`), `relater-fixtures.json`, the
baseline request manifest with per-variant missing-output semantics, and
`ownership-cases.json` for the checker-merge E3 scenarios.
