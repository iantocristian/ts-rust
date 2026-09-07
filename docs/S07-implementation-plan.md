# S07 implementation plan

Prepared 8 September 2026. This plan is stacked on S06 PR #10 and follows
[S07](../sprints/S07.toml), the [personal working guide](CODEX-RUST-GUIDELINES.md),
[PLAN §13](../PLAN.md#13-open-questions-and-pending-technical-decisions), and accepted ADRs
[0006](adr/0006-node-ownership--arenas--lazy-file-storage--bundles-and-check.md),
[0007](adr/0007-symbol-ownership--file-owned-binding--checker-local-merges.md),
[0009](adr/0009-concurrency-model-kept-from-corsa.md),
[0011](adr/0011-deep-recursion--reserved-stacks-and-growth-guards.md),
[0012](adr/0012-panics--recovery-and-generation-retirement.md),
[0013](adr/0013-source-bytes--javascript-strings-and-wire-positions.md),
[0014](adr/0014-host-seams-as-traits.md), [0016](adr/0016-toolchain-and-lints.md)
and [0017](adr/0017-dependency-policy.md). The exact accepted
[symbol](design/symbols.md) and [ownership](design/ownership.md) contracts take
precedence over an implementation shortcut in this document.

The source authority remains TypeScript/Corsa commit
`1f70213d4922b434345f639b441681e470c7cfc1`, with shared toolchain pins in
`data/s04/toolchains.toml`. No canonical submodule edits are part of this work.

## 1. Outcome, scope and existing gaps

S07 completes binding, the dependency operations needed to load a frozen checker
subset, and measured parse-and-bind performance. It does not implement the
checker merely because later checker consumers need binding APIs.

| Existing sprint requirement | Concrete deliverable | Acceptance |
| --- | --- | --- |
| S07-1 | File-owned symbols, tables, flow graphs, bind diagnostics and both binder resolver helpers | Exact binder corpus parity plus separate resolver observations |
| S07-2 | Object-safe in-memory host, options, paths, resolution and bundled-library closure in `ts_compiler` and its dependency crates | Every frozen subset variant loads the same ordered files, options, references and diagnostics as Go |
| S07-3 | Feature rule, primary manifest, effective options, exclusions and future-consumer operation inventory | Independently regenerated frozen subset; no skipped required rows |
| S07-4 | Identical pinned VS Code parse+bind runs with retained results, one/eight workers, actual allocation and RSS samples | RSS and allocated bytes each <=0.7 of Go; wall time <=1.0 at each worker count |
| S07-5 | Real bound files shared between program owners and retained through snapshot replacement | Two separate E3 criteria, with release import checks and final-drop evidence |
| S07-6 | Handwritten Go function mappings to actual implementations | At least 176/195 binder functions; target all 195 |

The source-only denominator is 167 functions in `binder.go`, 13 in
`nameresolver.go` and 15 in `referenceresolver.go`. `binder.go` alone cannot meet
90%. Generated functions remain separate provenance; mapping a wrapper does not
establish its behavior. Scope all three files explicitly in `PORTS.toml`.

Several advertised prerequisites do not yet exist: `data/workloads.toml`, a
frozen E2 subset, concrete symbol/flow payloads, a file binding cell, and a
program/snapshot owner. S06 has generic prepublication symbol primitives but
uses unit symbol payloads, and `ParsedFile` retains exclusive mutable AST
construction. These are implementation work, not presumed completed contracts.

S06's independent follow-up review also found shared-kind decoder construction
and a generic-storage escape from the concrete AST API. Those fixes must be on
PR #10, with current evidence, before this branch's implementation depends on
its retained-node or decoder APIs. The S07 PR stays stacked on PR #10 while that
base is open; no published history is rewritten.

## 2. Source audit and component boundaries

Before algorithm porting, generate a machine-readable dependency inventory from
the three pinned binder files, resolving each called AST, scanner, core and path
function against existing Rust mappings. Review actual semantics and call sites,
not marker presence alone. The initial audit found 126 direct AST helpers and
six scanner helpers. Missing families include full assignment-declaration
classification, module-instance state, expando initialization, declaration
containers, symbol-table operations, declaration-name formatting and token/error
ranges. Parser-private partial helpers do not cover the full binder contract.

| Component | Home | Responsibility |
| --- | --- | --- |
| Symbols/flow types and immutable bind result | `ts_ast::{symbols,flow,bind_result}` | Shared types usable by AST, binder and later checker without a dependency cycle |
| File binding lifecycle | `ts_ast` plus generic owner primitives in `ts_arena` only where needed | One initialization, stable identities, exclusive writes, publication, retention |
| Binder | `ts_binder::{state,declarations,containers,flow,expressions,statements,jsdoc,diagnostics,expando}` | Pinned bind traversal and state transitions |
| Name/reference lookup | `ts_binder::{name_resolver,reference_resolver}` | Real hook-driven source algorithms; no fabricated checker answers |
| Source-facing helpers | `ts_ast`, `ts_scanner`, `ts_core` | Correct shared dependency slices rather than binder-local copies |
| Options/configuration | `ts_core` option values; `ts_tsoptions` interpretation | Pinned defaults, configuration diagnostics and option projection |
| Paths/filesystem/resolution | `ts_core::path`, `ts_vfs`, `ts_module` | Canonical identity, immutable in-memory files, supported resolver operations and caches |
| Libraries/programs/snapshots | `ts_bundled`, `ts_compiler` | Embedded source, file inclusion, bound-file sharing, immutable snapshot roots |
| Producers | `scripts/s07_*.py`, `scripts/s07_oracle/`, Rust examples | Frozen observations, strict comparison, measured benchmark runs |

Keep `lib.rs` as the public boundary. Do not add a new crate merely for each Go
file. Implement safe Rust under the existing unsafe prohibition; use typed
ownership errors separately from upstream panic contracts.

## 3. Binding lifecycle and ownership

### 3.1 Initialization before immutable publication

Binding changes more than a symbol side table: it writes parents/container
links, local/export symbols and tables, flags, CommonJS indicators, flow links,
source symbol counts and diagnostics. Inventory every assignment in the Go
binder before freezing the Rust storage shape.

The compiler path moves `ParsedFile` into a shareable file binding cell. That
cell owns the exclusive parsed construction, a once-initialized completion and
a synchronization guard for the one winning initializer. Its public bind entry
accepts shared access: concurrent callers wait for the same result instead of
requiring each caller to own or clone the parsed tree. The winner takes private
construction, binds it on the production worker, validates the complete result
and publishes the immutable AST and `BindResult` together. IDs allocated during
parsing remain unchanged. The cell may be dropped once callers retain the bound
file; no back-reference from the bound result retains the cell.

Use `OnceLock::get_or_init` for this single-publication mechanism as required by
the accepted symbols design. The prototype must establish where the file-owned
cell lives across construction and publication; a separate program-local
binding cache is unacceptable. `BindResult` and the symbol arena are owned by
the same canonical file/bundle root. A wrapper may express the lifecycle, but
must not leave an independently owned parse tree or bind result outside that
root after publication.

`publish_unbound` remains the explicit parser-tool path from S06. A shared
unbound AST is not silently mutated in place or automatically copied in order
to bind it. Compiler/parse-cache entry points must create the binding cell while
exclusive construction still exists. Expose this distinction in Rust types and
compile-fail documentation. Test the shared bind entry itself, not just a
consuming `bind(ParsedFile)` convenience function.

For mapped files, one group binding cell owns all pending `ParsedFile` members
and their canonical ordering. Per-member handles retain that group plus an
index. The first bind request for any sibling initializes every member exactly
once, validates all cross-member references and publishes one `AstBundle`.
No member can escape as an independently published file before group completion.
A member failure prevents group publication and leaves a terminal failed group;
unpublished sibling staging is dropped, its reserved identities are never reused,
and subsequent sibling requests cannot restart it. Test concurrent first requests
through different siblings, a failure after an earlier sibling has completed its
private bind, and final release through a retained sibling symbol. This is the
Rust ownership publication unit, not a claim that Go binds all mapped files in
one call.

No user-supplied initializer receives raw generic storage authority. Retained
AST/symbol handles expose checked concrete reads and explicit retention; they
cannot reopen generic lazy writers. The S06 foreign-edge discriminator applies
also to new symbol, table, declaration-list and flow edges.

### 3.2 BindResult and links

The immutable result contains, or resolves through its file owner:

- Its own symbol arena with a fresh process-global `ArenaId`, declaration lists,
  symbol tables and ordered member/export relationships.
- Flow nodes/lists and synthetic flow payloads, including cycles and shared
  nodes. A checked `FlowId` names its file and slot; private slots cannot escape
  without their owner. Sentinel/unreachable and incomplete loop-label behavior
  must match Go.
- Slot-indexed declaration/local/export symbol maps, locals maps, container
  chains, flow/return/end/fallthrough links and all binder-written node/source
  flags. Move writes into the private core before publication when they belong
  to the Node contract; use side tables for fields the accepted design replaces.
- Source symbol count, bind diagnostics and deferred-expando state required by
  later queries. Temporary worklists disappear after completion.

Logical links use non-owning IDs. Ordinary same-file reads borrow; no symbol
lookup should allocate, clone a retention root or acquire the binding lock.
Cross-file imports validate identity, owner kind and published bounds in release.
File symbols stay stable across programs retaining that file. A retained symbol
keeps the complete mapped bundle if applicable, not only the symbol page.

Symbol names are `JsString`: upstream internal names use byte `FE`. Preserve nil
versus allocated-empty tables and lists, nil entries where the source permits
them, declaration order, duplicate suppression and slice aliasing during
construction. Published maps remain unordered; sort only at upstream observation
boundaries or in an explicitly canonical comparison representation.

### 3.3 Failure, contention and retirement

Freeze the successful, contended and failing state transitions before choosing
lock nesting. The winning binder runs once; repeated calls return the same
symbol identities and do not replay diagnostics or deferred work.

Go's `BindOnce` consumes its `sync.Once` after a panic but sets `isBound` only on
success. A bare Rust `OnceLock` retry would differ. Catch the initializer unwind
inside the one-time completion closure, discard unpublished staging and publish
a terminal failed completion, then resume the original unwind outside the
closure. Subsequent accesses must not rerun the initializer, expose a partial
bound tree or report `is_bound = true`. Rust can report the typed failed-file
state at its ownership boundary; the Go probe compares call count and bound
state rather than inventing a source returned-error value.

Detect same-thread initialization reentry before waiting; ordinary contention
waits. Dispatch to a worker before acquiring the binding guard. Initializers
must not wait for work that needs the same binding cell. Test panic cleanup,
retry prevention, reentry diagnostics and two racing first callers. Do not claim
checker generation retirement from a failed binder; that remains a separate
later contract.

## 4. Binder and resolver semantics

Port all three source files with real operations, naming Rust helpers by
behavior while retaining exact source mappings. Preserve at least:

- Declaration conflict tests and includes/excludes masks; default exports,
  replaceable JavaScript properties, ambient merges, enum and namespace state.
- Value-declaration preference for nonassignment/nonnamespace declarations,
  declaration append order and uniqueness, local/export symbol relationships.
- Function-first statement traversal; container and block-container save/restore;
  strict-mode checks, labels, unreachable diagnostics and parse-error suppression.
- Assignment/destructuring state, narrowing flow labels, antecedent order,
  exception/finally/switch paths, short-circuit and optional-chain effects.
- JavaScript CommonJS/export/expando rules; delayed expando binding after the main
  walk; JSDoc namespace/type-alias/import declarations and their source identity.
- Signed source positions and open integer kinds/flags at the same widths as Go.
  Reuse S04/S05 text operations by their actual decoder/formatting contract.

NameResolver and ReferenceResolver are not module resolution and must not be
substituted by a table lookup. Preserve globals/excludeGlobals, meaning masks,
parameter/type-parameter scopes, use-before-declaration callbacks, arguments and
require fallback, deferred contexts, export-default locals and type-only aliases.
Define object-safe hook traits matching the source's injected operations. Test
absent hooks, hook result precedence, nil inputs, callback sequence and arguments
against Go. Future checker integration supplies real checker hooks; S07's test
hooks are identified as fixture hosts, not checker implementations.

Reuse or extract a helper only after proving the full source domain matches.
In particular, full `GetAssignmentDeclarationKind` cannot simply reuse the
parser's private binary-only branch. Add independent discriminators for every
new helper family, including invalid bytes, synthetic locations, malformed
payloads and exceptional results.

## 5. Binder observation protocol and corpus

Freeze `data/s07/binder-cases.json`, `binder-requests.json` and a probe manifest
from the existing authoritative S06 input expansion. Use the same 12,829 primary
physical/library IDs and 22,343 primary parser requests unless independent Go
preflight exposes a necessary, reviewed binding-specific representation change.
A source panic is an observed outcome, not grounds to remove a case. Every
required variant must match for its physical row to pass.

The clean pinned Go adapter parses, binds and dumps actual results. Rust calls
production APIs. Dump:

1. Node structural identities and every binder-mutated observable field.
2. Symbol identity graph: raw name bytes, flags, declarations, value declaration,
   parents, export symbols, members/exports/locals and table nil/empty state.
3. Flow graph: variant/flags, linked node identities, shared labels, ordered
   antecedents, synthetic payloads and nil edges.
4. Source counts/indicators and ordered bind diagnostics with byte-preserving
   arguments, range, code/category and related information.
5. Named resolver hook traces and repeated-bind/concurrency observations in
   separate supplemental groups.

Use cycle-safe canonical IDs derived from a declared traversal of roots and
ordered edge labels, not allocation addresses or global runtime IDs. For map
edges, canonicalize by raw key bytes in the dump only. Preserve edge multiplicity,
list order and graph aliasing. If upstream map order changes diagnostics, retain
raw outcomes and identify the exact source ambiguity; do not sort production
operations or grant a blanket diagnostic exemption.

Strict records include request ID, stage, sequence and terminal outcome. Reject
missing/duplicate/extra/reordered records, unknown stages, non-finite metrics and
unclassified panics. Compare stable source panic messages exactly and runtime
bounds only through narrow classes. Use absolute per-request deadlines; a
hang/aborted request cannot pass as a matching error. Preserve raw stderr and the
first divergent graph path. Add negative tests that delete a flow edge, change
an alias target, reorder declarations, drop a variant or manufacture a panic.

Publish `run.binder.parity` only from the frozen primary test map. Require
separate `run.binder.resolvers`, `run.binder.graph_contracts` and
`run.binder.depth` metrics in S07 and the relevant ledger consumers. Supplemental
probes never inflate primary parity.

## 6. Frozen E2/E7/E8 subset

The selection rule must exist before Rust binding/loading results select any
case. Implement a pinned-Go syntax/configuration classifier under
`tools/s07/subset/`, with `python3 scripts/s07.py freeze-subset` checking and an
explicit `--write-manifest` operation writing the reviewed outputs. Do not widen
S03's generator command.

Version 1 is a feature rule across the entire compiler/conformance corpus,
including `conformance/controlFlow`; it is not a hand-picked list of passing
files. Before freezing, check in a total syntax/option classification table:
every encountered pinned SyntaxKind and directive has an explicit allow, reject,
or irrelevant-to-this-phase disposition with a reason. Missing classification
is a classifier error, never implicit eligibility or silent exclusion.

The proposed allowed feature families are primitives/literals, variables,
functions and overloads, object/array expressions, classes/interfaces,
enums/namespaces and their declaration merges, direct imports/exports,
unions/intersections, named type references without type arguments, simple
`typeof` type queries, ordinary control flow and narrowing. Explicit type
parameters and nonempty type-argument lists are rejected in source cases;
conditional/infer/mapped/template-literal types, indexed-access types, import
types, decorators and JSX are rejected by their exact node kinds. Project
references, content-mapper execution and emitted-output-only configurations are
rejected by explicit option/baseline predicates. All other syntax/directives
must be assigned deliberately in the table at checkpoint B; the rule is not
frozen until that table is complete and independently reviewed. Keep malformed
input whose recovered tree otherwise fits; do not exclude it for diagnostics.

The classifier records actual required dependency operations, including operations
introduced through declarations and libraries. Library dependency features are
an explicit future checker obligation, not silently erased from the inventory.
At checkpoint B, map each library-induced operation (including generic
instantiation and conditional types reached through library declarations) to
S08's planned checker scope. An obligation with no implementation scope blocks
the subset freeze; source-case exclusions cannot disguise it.
Freeze every qualifying case and all qualifying effective variants; a feature
exclusion applies at the documented physical/variant boundary and records why.
Do not cap counts or shrink eligibility after observing Rust failures.

Before accepting version 1, independently inspect resulting counts by directory
and feature, ensure `conformance/controlFlow` contributes cases, and ensure all
required operation families and mandatory stress fixtures are represented. An
empty family or unsupported program operation found in an otherwise eligible
case requires implementing that operation or revising the proposed rule before
freezing; it cannot become a per-case success-based exclusion.

`data/s07/subset-rule.json` and `subset.json` retain raw/loaded hashes, virtual
files, options, exclusions, effective lib closure, baseline identities and the
E2/E7/E8 operation matrix. Keep original diagnostics and unsupported settings;
the seven S06 legacy `module:none` parser qualifications are not an E2 permission.
An unsupported option must be recorded as such with its source diagnostic, not
silently changed to a successful configuration.

Mandatory ownership, lazy storage, mapped bundle, declaration merging, malformed
input, recursion, reentry and panic-retirement fixtures accompany the selected
compiler cases with separate identities and later-sprint completion status. S07
freezes future checker fixtures without marking their behavior implemented.

The `e2` producer initially publishes only independently measured
`frozen_subset`; no checker parity, errors, types or performance metric appears
until those implementations execute. All consumers must continue to see the
other E2 criteria as absent.

## 7. Options, paths, module resolution and in-memory program

### 7.1 Host and operation matrix

Implement object-safe `Send + Sync` host traits matching the relevant source
seams. In-memory file entries contain immutable byte owners, canonical and
original paths, directory entries and declared symlink/case behavior. Host methods
return typed missing/unsupported outcomes; no hidden disk/network fallback.

Freeze `data/s07/operations.json` mapping required host/config/resolver/program
operations to source functions, supported inputs, observations and exclusions.
Support every operation used by the frozen subset and its library/dependency
closure, with negative tests for explicitly unsupported calls. This matrix is
also the dependency contract for E2/E7/E8.

### 7.2 Required semantic slices

- Preserve path normalization, drive/root and URL handling, case sensitivity,
  relative path rules and canonical cache keys using pinned `tspath`/core code.
  Both case-sensitive and case-insensitive virtual hosts get fixtures.
- Preserve compiler-option tristates/defaults and config diagnostics. At this
  pin, target defaults to LatestStandard and Unknown/Classic/Node10 module
  resolution settings are mapped according to module mode; do not import an
  older TypeScript default table.
- Support relative/absolute and package imports, extension substitution,
  package.json type/main/types/exports/imports and applicable conditions, type
  directives, `types`/`typeRoots`, `baseUrl`/`paths`, rootDirs, moduleSuffixes,
  allowJs and resolution mode wherever the frozen operation closure needs them.
  Freeze successful and failed lookup traces, not only the final filename.
- Resolver caches include containing directory, name, mode and redirect inputs;
  tracing bypasses cache hits as in Go. Do not reuse a cache across immutable
  filesystem versions unless its inputs prove equivalence.
- Load roots, references, imports, automatic types and lib references in the
  source's inclusion order. Preserve missing/duplicate/case-conflict diagnostics,
  source reasons and default-lib/no-default-lib handling. `bundled.LibNames`
  alphabetic listing is different from the compiler's lib dependency/load order.
- Embed the pinned library bytes under `bundled:///libs` with their license.
  These bytes bypass filesystem BOM decoding. Source filesystem loading retains
  the S06 physical/virtual parser-text boundary.

`ts_compiler::Program` owns its immutable host/options view and retained bound
files/bundles. It exposes source lookup, ordered files, resolution results, bind
results and the eventual checker.Program dependency seam without a fake checker.
Equivalent file requests reuse the same parsed/bound owner through an explicit
cache key including canonical filename, source bytes, parse options, script kind and applicable
mapper context. Separate programs may share owners while keeping their own
resolution/global/checker state.

The Go loader comparison executes actual pinned program creation/file-loading
operations, with checker work disabled at the source seam where possible. It
must not replace the loader with a second handwritten model. Compare file order,
bytes, effective options, references, module modes and diagnostics. Keep source
inclusion parity separate from binder parity.

## 8. Program and snapshot ownership evidence

Extend E3 with real S07 payloads and two independently gated scenario groups:

- `shared_bound_file`: two program owners share one completed bind result and
  the same SymbolIds; source/binder-name lookup agrees before and after either
  program drops. A retained symbol and mapped sibling survive their intermediate
  roots; final drops return measured counters to baseline.
- `retained_snapshot_edit`: an old immutable snapshot answers using old bound
  files while a new snapshot owns edited files. Cross-import of old/new file and
  symbol identities fails in release, even for matching numeric slots. Dropping
  the active snapshot does not invalidate retained old results.

Use deterministic barriers, not sleeps, for contended tests. Include concurrent
bind-once, bind panic and reentry, malformed symbol/flow references, mapped bundle
roots and a retained symbol outliving a program. Run the exact inventory in
debug, release, Miri strict provenance and ASan with rebuilt std.

These scenarios exercise actual program/file ownership and binder queries. Do
not manufacture checker instances merely to name a test: independent checker
merges, checker-result retention, pool retirement and full E3 remain later work.
The future checker-specific fixtures retain their separate pending metrics.

## 9. Pinned VS Code workload

There is no existing workload pin to reuse. Freeze the stable VS Code 1.136.1
source revision `a44adf7f53e00964ab890f9f8758a334f1fc15bc`, resolved from its
[release tag](https://github.com/microsoft/vscode/releases/tag/1.136.1) before
performance measurements. Record the full commit, repository URL, license,
archive digest, selection rule, options, library pin and ordered source hashes
in `data/workloads.toml` and `data/s07/vscode-files.json`.

The measured operation is parse and bind of all tracked `.ts`, `.tsx`, `.mts`
and `.cts` files in that tree, including declaration/test sources, with each
physical file included once. No node_modules/build output download, dependency
installation, module loading, checking or emit runs in this workload. These are
separate source files with the pinned parser's extension-derived script kind,
ESNext target and preserved declaration-file status; the exact effective parse
options are frozen and shared by both adapters. This explicit source-tree
workload avoids giving either implementation a different project-resolution
closure while still measuring the actual VS Code repository's parser/binder
work. Report its file count and byte count after independent export, before any
Rust result is used. The untruncated Git tree for this commit independently
reports 13,094 selected regular files and 161,740,237 source bytes: 12,750 `.ts`,
267 `.tsx`, 77 `.mts` and no `.cts` files. Archive reconstruction must reproduce
those totals and then freeze SHA-256 content hashes; a Git tree listing alone
is not a runtime workload capture.

Do not call upstream `BenchmarkBind` an E5/E6 measurement: it binds a handful of
preparsed optional fixtures. Both S07 binaries must parse and bind the frozen
VS Code inputs from the same bytes/options and retain every bound file until the
measurement endpoint. Require an independent, untimed Go/Rust comparison of the complete per-file
binder-observable graph representation from section 5 over this exact VS Code
workload before accepting any performance result. Hash the canonical graph
records with a cryptographic digest, preserve counts and first differences, and
require every file digest/outcome to agree. This is a separate required
`run.binder.workload_parity` gate; counters alone cannot satisfy it.

Run that graph validation in separate processes. Measured RSS children do no
postphase graph dump or hashing allocation: after retaining the results they
read existing counters into fixed-size accumulators, emit a bounded scalar
report and exit. The process high-water mark includes that bounded report
cost, as it includes startup; it cannot be inflated by an unbounded validation
buffer. Input/work counters and frozen binary/source fingerprints connect the
measured operation to the independently validated operation.

Exports and reusable toolchains/caches live outside disposable Cargo cleanup.
Every download verifies its frozen source identity; offline runs use an already
provisioned archive/cache or fail with a prerequisite message. A moving branch,
missing source, partial export or unknown file never becomes a smaller workload.

## 10. Measurement contract and performance work

### 10.1 Allocator and worker integration

Install ADR 0016's pinned mimalloc allocator in native driver binaries, not as a
global allocator inside reusable compiler libraries. Proposed dependency is
`mimalloc = 0.1.48` with default features; the cached release and its
`libmimalloc-sys` closure must pass license/advisory/MSRV checks and all four native
builds. The native C build requirement is explicit. Miri/ASan continue exercising
the production compiler/owner code under their compatible instrumented allocator;
they do not certify mimalloc internals.

Allocation measurement uses a separate instrumented driver with
`tracking-allocator = 0.4.0` wrapping that allocator through its safe API. Its
tracker performs only atomic accounting, reports original requested bytes and
wrapper overhead separately, and covers all worker threads. Keep counter overflow
checked and treat missing events as invalid evidence. Its wrapper changes layouts,
so neither E6 timing nor peak RSS is taken from this allocation-instrumented
binary. Validate alloc/zeroed/realloc/dealloc and cross-thread accounting with
known allocations before using it. Verify constructor/MSRV/dependency behavior
in a minimal preflight before accepting this dependency choice.

The capability reason is actual allocator-request measurement without custom
unsafe Rust in compiler crates. Arena counters are rejected as a substitute;
`allocation-counter`'s thread-local System-allocator model would not match the
chosen allocator and eight-worker contract; `stats_alloc`'s generic const
constructor currently requires its nightly feature. The selected wrapper's
[tracker contract](https://docs.rs/tracking-allocator/0.4.0/tracking_allocator/trait.AllocationTracker.html)
distinguishes object and wrapper sizes. If preflight exposes an incompatible
contract, resolve and record a measurement mechanism before producing ratios;
never silently use a proxy or weaken the gate.

Use one reusable bounded worker pool for a whole batch with exactly one or eight
workers. Each Rust worker enters the reserved-stack/growth-guard environment once;
per-file `on_parser_worker` must not create additional threads. Go uses the same
worker count with GOMAXPROCS fixed accordingly. Record available CPUs; a host with
fewer than eight schedulable CPUs cannot certify the eight-thread gate.

### 10.2 Raw samples and statistics

Freeze the configuration before sampling: one unrecorded process warm-up per
runtime, then seven fresh-process samples per runtime/mode, alternating Go/Rust
order. Each process parses/binds the full frozen workload once and retains the
same logical roots. The median is the fixed aggregation rule; retain all samples,
not just the fastest or passing subset. A process failure, mismatched workload
count or invalid numeric sample invalidates that capture. A valid slow/high-memory
sample remains a measured failure.

Preload source bytes/options identically before the phase boundary. Report
preload size and process startup separately. Start wall time and allocation
snapshots at a barrier immediately before parse+bind; stop when all bound roots
are retained and workers have completed. Result serialization is outside the
phase. Preserve Go's pinned normal GC behavior (GOGC=100, no imposed memory limit)
and report effective values; do not disable GC to make Rust look smaller.

Measure allocation passes with Go runtime.MemStats.TotalAlloc deltas and Rust
original-layout allocation request bytes, recording their precise measurement
domains and realloc accounting. Include worker setup inside the measured phase
or exclude it on both sides by a pre-created-worker barrier; use the latter
consistently. Keep preload bytes outside both allocation deltas. Report Rust
wrapper overhead separately and never subtract real compiler allocations.

Measure peak RSS in fresh, uninstrumented processes using OS child resource usage
with macOS byte/Linux KiB normalization. Peak RSS includes startup and preload;
report those components but do not subtract an incomparable guessed baseline.
The launcher measures one child per resource-accounting process so a previous
child's high-water mark cannot contaminate a sample. Record the allocator,
architecture, OS, toolchains, build profile, GC settings, worker count, commit
identities and exact source/benchmark fingerprints.

Compute each ratio as the Rust sample median divided by the Go sample median
for the same metric and worker count. `e5` records separate one/eight-worker RSS
and allocation ratios; its existing scalar `peak_rss_ratio` and
`allocated_bytes_ratio` are each the maximum of their two per-mode ratios.
Thus both worker modes must meet the memory gate, with no favorable-mode
selection. `e6` emits each mode's median wall-time ratio separately. Both
producers include every numerator/denominator, raw sample, sample count and
workload completeness result. The existing thresholds remain <=0.7 for both
memory metrics and <=1.0 for both time metrics. S07 does not emit
`type_footprint_ratio`, and therefore does not complete full E5.

### 10.3 Responding to measured failures

Measure an early complete parser/binder vertical slice before polishing all
interfaces. Profile allocations, node/payload sizes, map growth, copied bytes,
lookup costs and lock/atomic counts. Optimize the observed costs: per-kind arena
layout, large payload separation, reserved vectors, borrowed text/lookup paths
and removal of duplicated metadata are candidates, not predetermined wins.
Change authoritative generators when representation changes; retain one
production representation and rerun semantic/ownership evidence after changes.

Do not pool live bound results across samples, remove source files, compare
parse-only Rust against parse+bind Go, change GC defaults asymmetrically or
relax thresholds. Failure to meet the performance gates means S07 is not done;
record the failure and continue concrete optimization or report the remaining
measured limit truthfully.

## 11. Recursion, portability and validation

Extend the cycle audit to binder recursion and combineFlowLists. Implement the
binder's explicit binary-expression trampoline required by PLAN §13.11, plus
production stack guards for other recursive components. Fixtures include long
left/right binary chains, nested blocks/functions/types/JSX, destructuring,
namespace/JSDoc nesting, deeply shared/cyclic flow labels and exceptional paths.
Small-stack optimized tests must prove actual segment growth; a large test stack
alone is not evidence. Avoid recursive graph serialization/drop paths that
circumvent the binder's protection.

All four supported native targets build and run binder/host tests, generator and
corpus producers, instrumented ownership and MSRV checks. Preserve independent
capture and artifact-upload paths after a measured failure. Benchmark captures
must identify host capacity and availability; do not assert portable performance
from one machine. WebAssembly growth strategy, browser execution, embedding and
full checker/emit acceptance remain E7/E8 and later phases.

Required checks: workspace debug/release tests and compile-fail docs; strict
Clippy and formatting; dependency/MSRV checks; generation; S04 text and S05
scanner regressions where their selected inputs change; S06 E1 and E3 current;
all new producers, negative harness tests, traceability and generated views.
Inspect metric truth and freshness after capture, not just command exit status.

## 12. Evidence registration and consumers

| Producer | Declared input groups | Measured output and consumer |
| --- | --- | --- |
| binder | Binder/AST/scanner/core/owner implementation closure, clean upstream pin, exact corpus/supplemental manifests, VS Code pin/options/file manifest and benchmark adapters | Derived corpus parity; required graph, resolver, depth and separate full-workload graph parity; S07-1/6 and source-file ledger |
| program | VFS/options/module/bundled/compiler closure, subset rule/manifest/options/operations and pinned libs | Complete load/resolve parity and `subset_loads`; S07-2 |
| e2 | Independent subset classifier, clean oracle preprocessing, frozen rule/manifest/baseline and operation identities | Only `frozen_subset` in S07; S07-3 and future E2 consumers |
| e3 | Existing real runtime closure plus binder/program owners and exact added scenario inventory | Actual shared-bound-file and retained-edit outcomes in all required modes; S07-5 |
| e5/e6 | Both benchmark adapters, complete production closure, allocator/pool integration, sample/aggregation configuration, immutable VS Code workload and toolchain pins | Actual ratios and completeness; S07-4 |

Audit every existing consumer before adding a metric. Add regressions for absent,
false, partial, stale and wrong-workload evidence; a seven-scenario S04 capture
must still not close full E3 or S09. The binder workload-parity capture must match the VS Code workload/options
fingerprints and the corresponding production driver/source fingerprints used
by E5/E6; a parity result for an earlier workload or implementation is not a
performance prerequisite. A subset-manifest success cannot close full
E2, and parse/bind measurements cannot close per-type E5 or E7/E8. Source-set
selection must include every helper read by a producer without invalidating
unrelated leaf evidence needlessly. Generated output-scope files are inputs too.

Keep raw reports, failures, effective toolchains and input hashes in immutable
artifacts. Registered producers write no tracker status by hand. Update editable
ledger fields only for real implemented scopes, then regenerate STATUS.md,
status/status.json, the unmapped-function worklist and docs/status.html.

## 13. Work checkpoints and exits

| Checkpoint | Work | Required exit before dependent work |
| --- | --- | --- |
| A | Fresh S06 review, valid fixes and source/API baseline | PR #10 pushed, affected evidence current; concrete review disposition recorded |
| B | Dependency inventory, frozen subset proposal, VS Code pin and measurement preflight | Independent Go-derived manifests/counts, named operation closure; allocator/counter/worker prototype verified |
| C | File binding cell, Symbol/Flow storage, publication and failure state machine | Real concurrent bind-once prototype, stable IDs, import/retention and panic tests; Miri/ASan on actual payloads |
| D | Binder traversal/declarations/flow plus missing helper families | Frozen binder graph/diagnostic parity over full corpus, exact failure records; no unimplemented grammar branches |
| E | Name/reference resolvers | Independent callback and lookup traces, all 195 source functions audited |
| F | VFS/options/module/bundled/program and immutable snapshots | Every subset variant loads with Go-equivalent file/order/options/diagnostics; both E3 program scenarios pass |
| G | Full VS Code benchmark and optimization | Usable frozen samples, ratios meet existing gates; final source layout retains semantic parity |
| H | Independent implementation review and final verification | Valid findings fixed, all affected captures current; S01–S07 pass locally, native CI results stated per target |

Useful parallel boundaries are binder control-flow versus resolver algorithms
once the shared API is fixed; host/module loading versus graph-observation tools;
and independent Go fixture construction versus Rust implementation. They share
one authoritative contract and frozen request identities. Avoid concurrent edits
to common owner/generator APIs or regenerating evidence while source changes.

## 14. Plan review protocol and unresolved prototype decisions

This is a plan, not evidence of implementation. Before implementation, post this
plan in a PR based on S06 PR #10. Resume the Claude session that reviewed the S06
plan, using Fable 5.1 at high effort. Ask it to read this plan only, without source
code review, subagents or extra model calls, and post concrete plan findings to
the new PR. Wait for its completed review, verify each finding and amend the
plan where valid before starting S07 code.

The structural choices needing prototype validation are the file-owned binding
cell's placement across mutable construction/publication, exact symbol/flow
storage sizes, the source-derived subset counts/operation closure, and allocator
instrumentation preflight. Each has a concrete checkpoint above. None permits
invented success metrics or a silent change to an accepted contract.

After Claude's review, append a disposition table linking its comment and
recording the accepted/rejected findings with reasons. During implementation,
record new contract corrections separately from the historical plan review.

### Internal plan review before publication

An independent plan-only review and bounded reread found the following issues;
the author checked them and applied these corrections before Claude's review.

| Finding | Disposition |
| --- | --- |
| E5 had no rule combining one/eight-worker memory results | Both per-mode median ratios are retained; each scalar gate is their maximum |
| Postphase graph hashing could inflate lifetime RSS | Complete workload graph parity runs separately; measured children emit only bounded scalar reports |
| Mapped siblings could publish through separate binding cells | One group cell owns all pending members; concurrent sibling initialization and partial failure have explicit tests |
| Feature families did not form a total subset rule | Complete syntax/directive classification is required before freeze; unclassified input is an error and library obligations must map to S08 |
| Workload counts were not a concrete correctness gate | Exact full-workload graph parity is a separate required binder metric |
| Workload parity could stay current after benchmark inputs changed | Its source/input declaration includes the VS Code pin, options, manifest and adapters; fingerprints must agree with E5/E6 |

Claude review is pending. No S07 runtime implementation or measured S07 success
is implied by this plan.
