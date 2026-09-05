# Tracking

Implementation state is declared in the ledger. Verification is computed from current execution evidence. A sprint closes only when every exit check and every required item passes. The status tool validates provenance and arithmetic; the owner still reviews the adequacy of tests, scope exclusions, thresholds and architectural decisions.

## Repository records

| Record | Purpose |
|---|---|
| `PORTS.toml` | Upstream file inventory, implementation state, Rust paths and required verification checks |
| `data/go-functions.tsv`, `data/upstream.json` | Receiver-qualified function inventory and full upstream pin with generated-field/inventory checksums |
| `status/runs.toml` | Reviewed producer commands, source globs, declared inputs, target and configuration |
| `status/experiments.toml` | Typed acceptance criteria for E1–E8 |
| `sprints/*.toml` | Functional tasks, required evidence and sprint exit criteria |
| `docs/adr/` | Proposed, accepted, amended and superseded architectural decisions |
| `status/evidence/` | Immutable run artifacts and a latest-attempt pointer per run |
| `STATUS.md`, `status/status.json`, `docs/status.html` | Generated views of current evidence |
| `status/unmapped-functions.json` | Complete unmapped-function worklist, kept separate from the small status summary |
| `status/history.jsonl` | Deduplicated snapshots linking the evidence behind each point |

## Ledger and pin provenance

The ledger has one entry per non-test upstream Go file. Files are inventory units; sprint tasks should describe smaller functional slices. One Go file can map to several Rust files, and multiple Go entries can reference the same Rust file.

```toml
[[file]]
go = "tsc/internal/scanner/scanner.go"
package = "internal/scanner"
crate = "ts_scanner"
phase = 0
kind = "source"
status = "ported"
rust = ["crates/ts_scanner/src/lib.rs", "crates/ts_scanner/src/literals.rs"]
verify = ["run.scanner.parity == 1", "run.scanner.tests_skipped == 0"]
pin = "<full upstream SHA>"
source_hash = "<SHA-256 of current upstream file bytes>"
loc = 1
```

The editable fields are `status`, `rust` and `verify`; changing them does not require regenerating upstream provenance. Implementation states are `planned`, `in-progress`, `ported` and `out-of-scope`. `ported` requires existing Rust paths. The tool derives `verified` only when the file is synchronized to the current pin and every nonempty `verify` check passes against current run evidence. Writing `status = "verified"` is an error. Missing paths, missing evidence, stale synchronization and failed checks cannot earn verification. `kind` distinguishes `source`, `generated`, `harness` and `out-of-scope`; generated files need generator/drift evidence as well as the applicable crate parity gate.

`pin` is the last synchronization commit; `source_hash` records the current pinned bytes. On a pin bump the generator advances unchanged, previously synchronized files. Changed files retain their old synchronization pin until their port is reviewed and updated. Repeating generation cannot clear that stale state. Evidence must be rerun for the new input fingerprint even if a source file itself was unchanged.

Generate all provenance records together:

```sh
python3 scripts/ledger-init.py                    # uses this repository's upstream/
python3 scripts/ledger-init.py ../TypeScript --pin 1f70213d49  # explicit bootstrap checkout
```

The script requires a clean checkout, resolves the requested pin to the actual full HEAD, reads tracked Git blobs, generates both files, then publishes their hash manifest last. A wrong pin, dirty checkout, malformed Go file or duplicate function ID fails generation. Manifest version 2 stores `ledger_generated_sha256`, the hash of a canonical projection containing the global pin and each file's generated `go`, `package`, `crate`, `phase`, `kind`, `pin`, `source_hash` and `loc` fields. Its inventory checksum covers the complete function-inventory file. `status`, `rust` and `verify` are excluded from that projection, so ordinary progress and gate edits preserve upstream provenance. Go 1.26 or later must be on `PATH` for the inventory generator, its tests and the oracle producer; where Go is installed through mise, export `~/.local/share/mise/shims` or run `mise activate` first.

`cargo xtask validate` checks the generated-field and inventory hashes plus pin agreement. Changes to generated fields require regeneration; editable fields are validated separately and their verification checks are reevaluated against current evidence. These hashes detect drift and partial generation; they do not replace review of the generator or authenticate an arbitrary hand-edited manifest.

The bootstrap checkout is sufficient for inventory generation. S01's oracle gate separately requires a registered, initialized `upstream/` submodule at that same pin, with the canonical Microsoft URL and a clean worktree.

The canonical submodule is now registered and initialized at `1f70213d4922b434345f639b441681e470c7cfc1`, and the actual oracle build and `--version` smoke test have passing run evidence. ADRs 0006, 0007 and 0013 and their design notes are accepted; S01 passes against the current bootstrap evidence. E1–E8 implementation and verification remain pending. Inspect current status when changing selected inputs: recorded bootstrap success is not a permanent waiver of those checks or proof of Rust compiler parity.

## Function traceability

```rust
/// port: tsc/internal/checker/checker.go:Checker.getTypeOfSymbol
fn get_type_of_symbol(/* ... */) { /* ... */ }
```

Canonical identities include the repository-relative file path, receiver type and function name. Pointer receivers are normalized to their base type. Free functions use `path:Function`; `init` declarations use per-file `path:init#1`, `path:init#2`, etc. The inventory carries its full upstream pin, source line ranges and canonical IDs. Parse errors and duplicate identities fail generation.

Use several markers when one Rust item replaces several Go functions, or repeat one marker across Rust items when a function is split. Each upstream identity counts once. File markers such as `//! port: tsc/internal/scanner/scanner.go` are navigation aids and earn no function credit. Unknown function markers fail validation and sprint checks. Counts cover in-scope source files; generated and excluded functions do not distort that denominator.

`status/status.json` is a small summary containing counts and a link to [status/unmapped-functions.json](../status/unmapped-functions.json). The separate worklist contains every unmapped in-scope source-function identity grouped by package. This keeps routine status reads small without truncating the actionable list.

This measures **mapped functions**, not semantic completeness or remaining effort. A marker can accompany a stub. Report mappings, baseline parity and completed gates separately; correctness comes from the required behavioral evidence.

## Execution evidence

A producer is a reviewed command registered in `status/runs.toml`:

```toml
[scanner]
command = ["python3", "scripts/run-scanner-parity.py"]
sources = ["crates/**", "Cargo.toml", "Cargo.lock", ".cargo/**", "rust-toolchain*", "xtask/**", "scripts/**"]
inputs = ["tests/scanner/options.json", "tests/scanner/baselines.json"]
cases = "tests/scanner/cases.json"
target = "aarch64-apple-darwin"
config = "release; frozen scanner corpus"
```

This is an example; the scanner harness does not exist yet. Only the actual workspace and oracle bootstrap producers are currently registered. Add E1–E8 producers when their implementations exist. Numeric and boolean thresholds are defined now so missing implementations remain pending.

Run a producer with `cargo xtask run scanner`. The command executes directly, without a shell, in the repository root. It must write exactly one JSON object to stdout; logs go to stderr:

```json
{"metrics":{"encoder_errors_match":true},"tests":{"literal-a":"pass","literal-b":"fail"}}
```

If `cases` is declared, that file is a nonempty JSON array of unique test IDs. The runner requires exactly those IDs, rejects duplicate or unknown results, and accepts only `pass`, `fail` and `skip`. It derives `tests_total`, `tests_passed`, `tests_failed`, `tests_skipped` and `parity`; the producer cannot override them. Skips stay in the denominator. Producers without a case manifest may emit numeric or boolean metrics, but cannot emit test results. Future nextest/JUnit adapters must preserve case identities and counts through this contract; no adapter is currently claimed to exist.

The runner records the full Rust revision, the run's source fingerprint, full upstream pin, declared input hashes, command, target/configuration, selected build environment, host, Rust toolchain, exit code and raw stdout/stderr with hashes. `sources` selects tracked and nonignored new files using Git-style glob matching, including `**`. If omitted, it defaults to `crates/**`, `Cargo.toml`, `Cargo.lock`, `.cargo/**`, `rust-toolchain*`, `xtask/**` and `scripts/**`. Include every implementation, shared dependency and producer that can affect the result; reviewing that source set is part of reviewing the producer contract. Build outputs, evidence and generated status views are excluded to avoid circular invalidation. Dirty submodules cannot provide evidence.

The workspace producer declares its own source set, using `Cargo.*` for the root manifest/lockfile alongside the relevant configuration and code. The oracle producer includes the `upstream` gitlink in its source set and `.gitmodules` as a declared input, so its evidence is tied to the actual checkout and submodule registration as well as the pinned provenance.

Documentation, root Markdown files and sprint files are outside the default source set. A producer that consumes them can include them explicitly in `sources`; `inputs` and `cases` are always hashed even when their paths are documentation or otherwise outside the source set. Changes to a run's source set, selected source bytes, command, target/configuration or declared corpus/configuration inputs invalidate that run. The specification of an unrelated run does not invalidate it. The upstream pin and relevant captured build environment must also match. Source-identical commits may reuse results; the original tested revision remains recorded.

Acceptance policy is separate from measurement inputs. Changing experiment thresholds, sprint checks or editable ledger fields reevaluates the recorded metrics without rerunning unaffected producers. If such a file is explicitly consumed through `sources`, `inputs` or `cases`, its change does invalidate that producer. Review policy changes directly; changing a threshold must not manufacture a new measurement timestamp or hide an unmet requirement from the plan.

The top-level `context` in `status/status.json` is a broader repository snapshot used for policy/history tracking. Artifact freshness uses each run's selected-source digest and declared inputs instead. Editing `PLAN.md` can therefore change the status context while every unaffected run remains current; a changed summary context alone does not mean its evidence is stale.

Successful capture requires exit code zero, a valid report, and unchanged source/inputs across execution. A harness may exit zero after successfully measuring imperfect parity; the typed thresholds decide whether that measured result passes. A process failure is not a measurement. A failed or malformed rerun replaces the latest attempt and cannot expose a preceding success. Attempts for one run ID are serialized. An interrupted process can leave `status/evidence/<id>.lock`; remove that empty directory only after confirming no process for that run remains, then rerun.

Artifacts are named by their SHA-256 under `status/evidence/`; `<run-id>.latest` selects the latest attempt. Consumers verify artifact content and compare its inputs with the current repository before exporting `run.<id>.<metric>`. Missing, stale, failed and corrupt records remain visible in the evidence state and supply no passing metrics. Artifact capture proves which producer ran against which inputs, not that the producer's assertions are sufficient; review those contracts with the port.

## Experiments and sprints

Each experiment has multiple typed criteria. **Every criterion must pass.** For example:

```toml
[[E7.criteria]]
id = "checker_parity"
metric = "run.e7.checker_parity"
op = "=="
threshold = 1
unit = "matching checker cases / frozen subset"
```

E7 requires parser size, parser throughput, checker parity and a portable host. E8 requires Node latency, Rust-consumer parity and lifetime checks. E5 includes the per-type memory threshold. Notes and `nature` explain measurement limits but never supply passing values.

The design notes behind ADRs 0006, 0007 and 0013 name the E3/E4 criterion ids each scenario row covers, and every criterion appears in at least one row, so a producer case can be traced to a note row and a criterion to its scenarios.

Sprint checks use `<metric> <op> <value>`, with numeric comparisons, boolean equality and ADR/implementation states. Missing metrics cannot pass; malformed definitions, unknown fields and duplicate sprint/item IDs are errors. Required items default to `required = true`. They need nonempty `done_when` checks and block completion until every check passes. Only explicitly optional items may remain informational.

S01 checks the reviewed contracts, provenance, registered upstream pin, actual workspace build, oracle build and version smoke. Merely having inventory entries cannot complete it. ADR acceptance is a reviewed human decision; changing an `Accepted` label is not a substitute for the required design note and review.

## Commands, publication and history

```sh
cargo xtask validate           # read-only tracking/provenance validation; pending work is allowed
cargo xtask run workspace      # actual locked Cargo workspace build and evidence capture
cargo xtask run oracle         # checked submodule, Go build and --version evidence
cargo xtask status             # regenerate summaries, dashboard and the unmapped-function worklist
cargo xtask check S01          # fails unless every required item and exit criterion passes
cargo xtask status --record    # append only a distinct source/evidence snapshot
```

If a sandbox cannot write Go's system cache, use a writable workspace cache for the oracle command: `GOCACHE="$PWD/target/go-build" cargo xtask run oracle`. This changes the cache location, not the required build and smoke checks.

CI automation and nightly publication are **planned, not installed**. A future workflow should validate schemas/provenance, run producers and publish derived reports with their raw artifacts. It must not require unfinished sprints to pass on every development change. Source-branch bot commits are not needed to publish a dashboard.

History records pin, repository context and underlying artifacts. Re-rendering the same context and evidence adds no new point. A policy or documentation edit can create a distinct history snapshot while reusing the same current run artifacts; it is not a new measurement. The chart separates upstream pins and legacy history from the current series. A publication date is not a measurement date: nightly publication of old results cannot satisfy the four consecutive weekly measurement runs required for cutover. That future gate must inspect distinct run artifacts and their execution timestamps against the approved workload matrix.

ADRs change when their architectural decisions change. An amendment records a refinement; a replacement decision supersedes the old ADR. Ordinary implementation fixes that follow an existing decision need neither a replacement ADR nor a new approval ceremony.
