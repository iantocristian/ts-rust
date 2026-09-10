# S08 data

Frozen inputs and Go observations for the checker sprint. Generated observations
are read from the pinned upstream commit; named diagnostic inputs are reviewed
separately and never selected from Rust's successful output.

| File | Contents | Derivation |
| --- | --- | --- |
| `checker-flag-observations.json` | Every constant of `TypeFlags`, `ObjectFlags`, `SignatureFlags`, `TypeFormatFlags`, `SymbolFormatFlags`, `VarianceFlags`, `AccessFlags`, `NodeCheckFlags`, `ContextFlags`, `ParseFlags`, `ExternalEmitHelpers`, `Ternary` and `TypeSystemPropertyName` in `internal/checker`, `Flags` and `InternalFlags` in `internal/nodebuilder`, the type-display constants, and `unsafe.Sizeof` of the checker's type, signature and symbol records | Two test files added through `go test -overlay` (the checkout is not modified) read the values out of the compiled pinned packages and write them as JSON; the Rust flag ports assert equality in `crates/ts_checker/src/flag_tests.rs` and `crates/ts_nodebuilder/src/tests.rs` |
| `flag-requests.json` | Names of the 289 constants and 23 fixed records observed by the scaffold | Observation inventory only, with no copied expected numeric values; `scripts/s08_flags.py` compiles the named Go expressions and compares the whole result with the unchanged fixture |
| `baseline-requests.json` | Phase obligations, baseline enablement, reference inventory and provenance for all 10,728 eligible variants | `scripts/s08.py freeze`; frozen S07 variants joined by exact ID to compiled Go `GetEmitDeclarations()` observations, with input/source hashes. This is not a node-level query schedule or an executed baseline result |
| `storage-pilot.json` | 1,307 intrinsic/string/fresh constructor actions | Two intrinsics; 261 unique strings (empty, ASCII, surrogate WTF-8, malformed byte, emoji and 256 numbered names); duplicate interning, repeated fresh lookup and fresh-of-fresh for each string. This crosses growth boundaries and exercises identity/byte preservation; it is not a representative subset workload |

The struct sizes are the Go layout on the recording host (darwin/arm64, the
pinned `go1.27.1`). They are fixed-record inputs to the future per-type census,
not that census: owned lists, strings, maps, shared backing and capacity must also
be measured. The initial intrinsic/string diagnostic does not satisfy the
whole-checker or per-type footprint gate.

Reproduce from the initialized source pin and the repository's pinned local Go
and Rust toolchains, using a new output directory for every invocation:

```sh
python3 scripts/s08_flags.py --output target/s08/flags-review
python3 scripts/s08.py check --output target/s08/phases-review
python3 scripts/s08_storage.py --output target/s08/storage-review
python3 -m unittest discover -s scripts/tests -p test_s08.py
```

`s08.py prepare` writes a candidate for review; explicit `freeze` replaces the
frozen manifest only after regenerating Go policy. Missing references do not
prove `NoContent`, and `not_implemented` cannot pass. These commands do not
register an E2/E5 producer or modify evidence thresholds.

Checkpoint results, including the unusable first Go retained-memory endpoint,
are documented in `docs/S08-P0-P1.md` and archived under `tools/s08/results/`.

Planned additions, each frozen at P0 before any Rust result exists (plan §§5–6):
`type-footprint.json` (census membership and sharing rules), `checker-workload.json`
(the fixed query schedule for `checkerbench`), `relater-fixtures.json`, the
node-level baseline/query schedule beyond the phase manifest above, and
`ownership-cases.json` for the checker-merge E3 scenarios.
