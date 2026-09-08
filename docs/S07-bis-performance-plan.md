# S07-bis: measured storage and CPU improvements

Status: selected implementation path; implementation has not started.
Date: 2026-09-08. Work branch: `codex/s07-bis`.
Baseline: `53b523a` from `codex/s07-binder` / PR #11.
Review: [independent plan findings and amendments](S07-bis-plan-review.md).

Keep this work on the separate S07-bis branch. The plans are currently
uncommitted; do not commit them or subsequent S07-bis work to the S07 branch.

S07-bis is the performance follow-through for S07, not a new tracker sprint or
a replacement acceptance definition. Its objective is to make the existing
S07 parse-and-bind implementation pass both memory criteria and both CPU modes,
while keeping every existing correctness and ownership prerequisite current.

## 1. Decision

Choose **compact storage with unchanged per-source publication**, implemented
through bounded experiments. First expose cheap borrowed field access, replace
the binding maps with compact typed storage, and establish a private validated
owner scope. Then replace the uniform node representation, compact internal
links and text, and remove demonstrated duplicate work and excess allocation.

Claude's diagnosis identifies the right major costs. This plan adopts its
per-kind field storage, smaller nodes and identifier handles, validation audit,
symbol-table experiment and capacity audit. It separates those choices from
the proposal to move binding before parsed publication. Moving the publication
boundary is not necessary to test the storage changes, and it introduces
additional questions about existing readers, mapped siblings and failure.

The [exclusive-binding alternative](S07-bis-exclusive-binding-alternative.md)
is **on hold**. It is a real fallback, with activation conditions and a bounded
first prototype, not a second implementation to develop concurrently.

This refines the earlier recommendation to prototype A first: the first
experiment still targets A's measured binding costs, but preserves publication
while testing compact fields and direct access. A and B need a joint layout
budget. Simply moving eight more bytes into today's inline identifier payload
can enlarge the enum or trigger millions of boxes under the current generator.

### Contract history

S06 and the original S07 plan described exclusive binding before publication.
S07 commit `1637157` explicitly changed that to per-source binding cells over
published parsed cores following the mapped-sibling review. The current
[S07 lifecycle](S07-implementation-plan.md#31-initialization-before-immutable-publication)
and [review disposition](S07-implementation-plan.md#claude-fable-51-review-disposition)
record the implemented choice. For S07-bis, retain that current lifecycle;
the historical S06 wording does not authorize changing it silently. Reconcile
those cross-references in the implementation's contract update without
overwriting unrelated working-tree edits.

## 2. Exact destination and starting gap

The authority remains [S07](../sprints/S07.toml),
[E5/E6](../status/experiments.toml), and the existing
[aggregation](../scripts/s07_benchmark_measure.py) and
[statistics](../scripts/s07_benchmark_stats.py). Every ratio is Rust / Go.

| Criterion | Required, unchanged | Current one / eight workers |
| --- | --- | --- |
| Pipeline wall time | Each median ratio <= 1.0 | 1.789 / 1.931 |
| Timing qualification | Both runtimes' relative MAD <= 5%; bootstrap upper 95% ratio bound <= 1.0, in both modes | Fails |
| Lifetime peak RSS | Worse worker-mode median ratio <= 0.7 | 1.447 / 1.442 |
| Pipeline allocated bytes | Worse worker-mode median ratio <= 0.7 | 1.626 / 1.626 |

At the old one-worker Go medians, the limits would be 2.035 GB of allocation,
2.209 GB peak RSS and 2.938 seconds. Eight-worker wall time would need to be at
most 0.634 seconds. These are planning conversions, not fixed future Go
denominators. Rust needs approximately 57% fewer requested bytes, 52% lower RSS,
and 44% / 48% less wall time than its current one/eight-worker results.

Internal engineering targets provide headroom: allocation <= 1.90 GB, RSS
<= 2.10 GB on this workload/host, and median CPU ratios <= 0.90. These targets
help choose candidates; they neither modify nor replace the real gates.

The frozen workload remains 13,094 files, 161,740,237 loaded bytes, 19,593,488
nodes, 2,459,867 symbols, 423 parse diagnostics and 5,250 bind diagnostics.
The loaded-byte/options digest is
`d4ff2ad3b0d590babb596522a7105d961dbb3c4f2ebc987488d88b7ebd8e0c88`.
Matching counts is a screening check; complete graph parity is still required.

### What the profiles justify

The [memory report](S07-memory-profile.md) and [CPU report](S07-cpu-profile.md)
are the starting evidence, not results for the future implementation.

| Observation | Consequence for the plan |
| --- | --- |
| Binding map + flow slots retain >= 445 MB, including 177 MB spare; full-map insertion requests 633 MB | Replace representation and growth policy first; include replacement indexes in the comparison |
| Binding node-access union is 38.9% / 36.4% of binding CPU | Measure fewer resolutions and cheaper access; the whole union is not removable overhead |
| Uniform core nodes use 1,567 MB before spare capacity and separate payloads | Small map optimizations alone cannot meet the memory gate |
| Completion validation uses 10.1% / 8.8% of parse CPU; construction edge checks use 4.4% / 4.4% | Establish a construction proof before removing repeated checks; do not claim their sum as savings |
| 923 MB of requests are freed or superseded | Attribute and reduce request traffic; this is not a measurement of physical copies |
| About 480 MB of the RSS gap remains outside the compared live-byte counters | Measure allocator/VM domains if RSS fails after storage improves |

Both Go and Rust already allocate successive stable arena pages. Neither
pre-sizing nor a different allocator is assumed to recover the whole gap.

## 3. Invariants and intended API costs

Follow the [Rust working guide](CODEX-RUST-GUIDELINES.md),
[ADR 0006](adr/0006-node-ownership--arenas--lazy-file-storage--bundles-and-check.md),
[ADR 0007](adr/0007-symbol-ownership--file-owned-binding--checker-local-merges.md)
and [binding operation record](S07-binding-operations.md).

- Preserve byte, integer, missing/null/list-backing, source-order and failure
  semantics. Keep current unsupported clone-plus-rebind behavior explicitly
  unsupported; do not turn it into a new passing comparison.
- Keep full public 64-bit arena/slot IDs, with both full 32-bit components.
  Internal compression must have a checked full-range fallback, including
  imported and lazy references. No workload-sized ID limit or stolen slot bit.
- Keep per-logical-source once initialization, independent mapped siblings,
  initiator panic versus waiter failure, reentry handling, and unchanged parsed
  observations after success or failure. Never mutate through a published borrow.
- Retained nodes/symbols keep the complete required file or bundle. Cache
  validation uses the cache owner's retained graph. No new owning cycles.
- Preserve lazy JSDoc/token identity and rollback, including nodes materialized
  before binding. Do not force lazy caches or runtime identities to initialize
  during ordinary parsing merely to simplify indexing.
- Use safe Rust. Proven private scopes can remove repeated owner checks;
  public imports, scope entry, callbacks and foreign/lazy fallbacks stay checked.
  Bounds safety remains separate from identity validation.

The intended common access path is a borrowed header or typed payload plus a
direct local index: no allocation, owner Arc increment, hash lookup or lock.
Binder-written fields may additionally read their applicable compact column.
That is an objective to measure, not a claim about all mapped/lazy operations.

## 4. Storage design and budget

### 4.1 Separate the read contract from the physical record

Today `NodeRead` and retained-node dereferencing expose a physical `Node`, and
factory mutation returns `&mut Node`. Replacing the enum alone is insufficient.
Introduce an AST-specific borrowed read/mutation facade behind generated
accessors. Migrate binder, parser, visitors, factory/clone/update, utilities and
encoder callers deliberately. Preserve semantic APIs where possible, but
replace physical `Deref<Target = Node>` assumptions where necessary.

The facade must borrow actual headers and typed payloads. It must not rebuild
an 80-byte temporary node, clone a payload or retain an Arc on every read.
Owned factory inputs can remain convenient construction values; convert them
once on insertion. Keep the generic arena API's existing contracts available
to its other consumers rather than making every arena AST-specific.

### 4.2 Compact typed binding storage

Use the audited Go binder-write inventory to define separate storage for
declaration symbols, locals/containers, flow links and actual syntax-field
overrides. Preserve all fields currently represented by `NodeBinding`, including
return/end/fallthrough flow links and source metadata. No field may disappear
because the benchmark rarely uses it.

Here, "per-kind" storage means the concrete payload shape, independently of
syntax `NodeKind`. Keep a separate payload-shape discriminator: `TokenData`
paired with an identifier or unknown syntax kind must not select identifier
storage. Field eligibility follows the audited payload and source operations,
with a checked fallback for unusual constructed combinations.

The first default is direct storage indexed by an eligible shape's local ordinal,
with source-local symbol/flow references where the owning result proves their
arena. Read-only syntax access bypasses binding storage; flag/parent/payload
queries apply their specific overrides. A sparse patch store replaces full
node copies only where a field really changes.

Compare this against occupancy-mask pages with packed values for sparse fields.
Account for rank lookup, insertion movement, directories and absent values.
Do not adopt a new all-node `u32` rank array casually: it costs about 78.4 MB.
Reuse the eventual typed payload locator or charge a temporary index explicitly,
with its removal scheduled in the node-layout step. Before typed payload pages
exist, checkpoint 0 must choose the provisional core-slot-to-shape/source-row
mapping. Compare a direct index with eligibility bitmaps/rank checkpoints. Build
it during construction or fuse it with required completion work, charging the
scan and allocations; it is not free because another validation pass may later
be removed. At checkpoint 3, store the same row ordinal in the typed locator
and delete the provisional core index, or retain and charge one shared index
if physical payload ordering cannot supply the binding row. Lazy additions and
unusual source partitions use a separately measured checked fallback.
Keep a checked general
fallback for uncommon mapped, imported, constructed and lazy shapes.

Logical source ownership and physical arena membership are different. The
single-source common case may use the established source scope. Multi-source
owners need an audited source partition/index or the existing checked fallback;
an arena-ID comparison alone must never authorize sibling writes. Avoid a new
whole-tree scan on every field access or every binding request.

### 4.3 Compact syntax, links and text

Prototype a small common header and generated per-kind payload pages. Tokens
have no general payload allocation. A header-held payload ordinal, or a charged
slot directory, maps the stable public node identity to its concrete payload.
Do not change identity when a physical page or directory grows.

Model a roughly 32-byte header with a full parent ID first. A target near
24 bytes additionally requires a compact parent link and its escape mechanism;
include a separate payload-shape tag as well as the open syntax kind. Measure
the complete weighted representation,
including payloads, locators and page slack. These are provisional design sizes;
actual compiler layout on all supported targets decides them. Node kind remains
an open source-defined value; nullable fields and unusual factory kind/data
pairs must retain their current behavior.

Internal same-owner edges can use private compact references. A separate escape
mechanism preserves foreign/lazy references and the full slot range, rather
than reserving a high bit from the accepted public ID domain. Entry and exit
conversion must recover the exact public identity and retained dependency.

Use owner-relative text handles for source ranges and a separate pool for cooked,
synthetic and arbitrary-byte names. Borrow the selected bytes during lookup;
materialize an independently retained `JsString` only at an escaping API that
requires it. Test Unicode escapes, malformed bytes, WTF-8 slices, missing names,
shared source ownership, clones and foreign-owner text. Reducing the identifier
payload alone does not shrink an enum while other variants set its maximum.

Store subtree caches only on applicable composite payloads. Prototype lazy
owner-held runtime-identity storage, including the cost of initialization and
lookup; preserve lazy process-global assignment, clone identity, overlay
identity and concurrent queries. Reject a hot locking side table that trades a
small field saving for a larger CPU regression.

### 4.4 A budget that includes replacements

At checkpoint 0, produce an executable layout model using the owned-slot census
and actual `size_of`/alignment. The following is an **initial requested-live
storage budget**, not a measured achievable layout or a sum of predicted savings.
Values are decimal MB; categories are disjoint and include replacement storage.

| Retained compiler category | Working ceiling |
| --- | ---: |
| Syntax headers, actual payloads, locators and their capacity | 780 |
| Syntax lists and all auxiliary backing/capacity | 140 |
| Binding fields, syntax overrides and their indexes/capacity | 100 |
| Symbols, flow records/lists, declaration backing and symbol tables | 440 |
| Unique source and other text backing, including pooled names | 190 |
| File metadata, caches exercised by the workload, directories not charged above, allocator-request residual | 50 |
| **Total requested live compiler storage target** | **1,700** |

Hash bucket/control allocations, Arc headers and temporary indexes must be
included, not hidden outside the model. Unique source backing is charged once
even when preloaded. Preload is excluded from pipeline allocation but included
in lifetime RSS, so 1.70 GB live storage and 1.90 GB pipeline allocation are
separate checks. Measure transient request traffic and process peak RSS directly.
The residual row is a hard modeling challenge: if it is unknown or exceeded,
report that fact and revise the design, not the observed counter.

Do not spread a missed budget across categories without an explicit record of
the replacement tradeoff. A design that fits only by assuming zero map overhead,
perfect page occupancy or zero temporary allocation does not pass this checkpoint.
The model must also show tiny-file and rare-kind overhead, not only a giant file.

## 5. Execution sequence

Each checkpoint ends with a recorded keep/reject decision and the remaining
distance to all four performance gates. A locally useful improvement is not
S07 completion. Retain one approved candidate as the next control; discarded
experiments remain documented, not enabled in the production path.

### Checkpoint 0 — Freeze the experiment and check feasibility

1. Record current source inventory, binary/configuration hashes, pins, workload
   digest and archived baseline captures. Preserve unrelated user changes.
2. Add diagnostic-only experiment manifests/results under
   `tools/s07/performance-experiments/`, with a short index linking this plan.
   Fields: hypothesis, parent control, touched components, predeclared variants,
   semantic counterexamples, layout budget, sample policy, actual artifacts,
   outcome, and separate wall/RSS/allocation results. No tracker success metrics.
3. Extend the existing census with payload-kind distributions, field occupancy,
   actual per-file capacities, map control bytes where observable, and an
   explicitly unknown remainder. Use existing captures where sufficient.
4. Build safe layout sketches for compact typed bindings and compact nodes/text.
   Compare dense eligible-kind columns and packed sparse pages; do not integrate
   both complete designs. Include nil entries, rare fields and multi-source cost.
   Freeze the provisional locator's construction timing and later replacement.
   Include open kind/payload pairings, lazy additions and empty per-shape
   directories: allocating a Vec header for every shape in every tiny file can
   consume substantial storage even before the first payload is inserted.
5. Review whether the 1.70/1.90/2.10 GB working budgets have a plausible route.
   If not, identify the specific category that needs a different representation
   before porting thousands of accessors. No feasibility claim from Go census
   subtraction or adding overlapping profiler rows.

Exit: a reviewed ownership/API sketch, concrete module boundaries, measured
layout sizes, candidate budget and frozen diagnostic protocol. No production
optimization is claimed at this stage.

### Checkpoint 1 — Make field reads cheap and representation-independent

1. Introduce the borrowed facade in `ts_ast::storage`/`node_accessors`; change
   generated accessors and factory mutations at their emitter in
   `xtask/src/gen/ast.rs` and related generator modules. Use `cargo xtask gen`;
   do not hand-edit generated outputs or alter upstream schema semantics.
2. Migrate the binder's repeated `n()`/symbol/locals reads first, then all callers
   required for the representation swap. Carry an already resolved borrow or
   small field observation across adjacent operations where mutation permits.
3. Keep backing storage unchanged for this experiment. Inspect optimized code
   and count lookups/retentions in a separate diagnostic build to detect hidden
   materialization, repeated field resolution or Arc increments.
4. Run accessor, factory, encoder, identity, byte-string and binder countertests
   in debug/release. Screen on the full workload at both worker counts.

Exit: equivalent observations with no per-read allocation/retention and no
unexplained wall-time or memory regression. This is infrastructure for the next
change; it is not justified by a large speculative future gain alone.

### Checkpoint 2 — Replace binding maps and repeated owner resolution

1. Implement the selected typed binding columns and sparse field patches. Include
   promotion/removal behavior currently split between full records and flow slots.
   Replace general overlay probing on ordinary syntax reads with field-specific
   resolution. Keep source metadata and diagnostic publication in the same cell.
2. Enter a validated binder source scope once. Mint non-forgeable local handles
   from checked imports or already validated edges; retain safe bounds access
   and debug assertions. Define callback/reentry boundaries and the fallbacks
   for imported, mapped and lazy nodes before eliding a check.
3. Resolve all field writes through the same owner proof. Keep explicit wrong
   sibling/wrong owner rejection and no partial publication after a panic.
4. Measure total replacement bytes, occupancy, request growth, patch count,
   lookup count and binder CPU. Compare complete wall/RSS/allocation results;
   removing 445 MB of old storage does not imply a 445 MB net saving.
5. Add compile-fail tests for scope escape/cross-owner mixing and execute actual
   new storage through the S06/S07 ownership instrumentation, not a test-only map.

Exit: full binder-field parity and ownership, a measured reduction in binding
storage/request traffic, and a defensible CPU result. If compact columns still
leave accessor/publication routing dominant, retain the evidence for the held
alternative's activation decision; do not accumulate more replacement maps.

### Checkpoint 3 — Integrate compact syntax and owner-relative text

1. Introduce generated per-kind storage and small headers. Share the payload
   locator with applicable binding indexes; remove the temporary all-node index
   if checkpoint 2 needed one. Keep one production representation after migration.
2. Port common tokens/identifiers/edge payloads as a vertical slice, including
   parser construction, mutation, publication, binding and encoding. Include
   TokenData with identifier/unknown kinds to test the independent shape tag. Measure
   actual end-to-end costs before extending the pattern to every shape.
3. Migrate the remaining concrete shapes and auxiliary/list storage. Preserve
   backing identity, nil versus allocated-empty lists, parent updates, cycles,
   shared children and clone/update behavior. Do not compact mutable lists by
   copying backing whose identity is observable.
4. Introduce source/pool text handles and compact internal edge forms. Charge
   foreign escape maps, pooling metadata and conversions; keep general owned
   `JsString` APIs where they are required. Avoid a global interner or global lock.
5. Move composite caches and test the runtime-ID storage candidate separately.
   Retain the existing runtime-ID field if the proposed replacement loses its
   CPU/memory tradeoff; record how the remaining budget is then met.
6. Update diagnostic observers for the new representation and recalibrate them.
   Their old patches must fail visibly on incompatible source, not silently
   omit new payloads. Recheck actual layouts on all four supported native targets.

Exit: full AST/parser/binder/encoder parity and ownership, a new complete census,
and a measured candidate whose remaining categories can fit the total budget.
Do not continue polishing a node representation that makes the memory target
impossible even with optimistic remaining categories.

### Checkpoint 4 — Compact symbols, flows and list/table storage

1. Audit actual fields and source-defined nil/union distinctions in `symbols.rs`,
   `flow.rs` and list backing. Use owning-result-local references where proven;
   preserve explicit full IDs at imports and escaping results.
2. Prototype compact flow discriminants and union payloads rather than assuming
   every current 48-byte record can become 32 bytes. Likewise, do not budget an
   80-byte symbol until the current 112-byte record has a measured replacement.
3. Reduce declaration-list and flow-list overhead while preserving ordered
   entries, shared backing, cycles and obsolete backing still reachable by
   observations. Count allocated records as well as reachable records.
4. Audit table keys and clone traffic. Start by borrowing name bytes on lookup
   and avoiding redundant retains; symbol-table ordering remains produced at the
   existing observation points. Under ADR 0007, tables remain byte-semantic
   `JsString`-keyed maps unless an explicit equivalent facade/ADR amendment is
   recorded. Internal text pooling does not authorize identity-based equality.
5. Compare the current randomized hasher with at most one justified fast keyed
   candidate. Verify licenses, MSRV, hostile-name collision behavior and the
   real end-to-end benefit before adding a dependency or changing hashing.

Exit: all symbol/flow graph and declaration-backing observations preserved;
measured combined storage/request reduction. Hashing is optional if its gain is
below the diagnostic decision threshold or another path remains dominant.

### Checkpoint 5 — Remove proved duplicate validation work

1. Inventory construction, mutation, imported-edge, source-metadata and final
   validation obligations. Construction validation is not equivalent to final
   validation: parents and payloads can change after insertion, including hooks.
2. Add a private trusted parser-construction path only where local handles and
   retained imports prove the same edges valid. Keep checked public factory
   entry points and their immediate ownership errors.
3. Initially retain completion validation and remove only repeated edge checks
   covered by that proof. A later reduction of completion scanning needs proof
   for every mutation/auxiliary path and its own measured experiment.
4. Countertest wrong owner, out-of-range IDs, post-construction mutation, invalid
   metadata/list edges, callback reentry and lazy commit rollback in release.
5. Reprofile parse CPU after the layout changes. Compare actual elapsed time;
   do not translate the old 14.5% combined attribution into expected savings.

Exit: unchanged failure/ownership behavior and a measured parse improvement.
Skip or reject check elision if the proof requires broad new unsafe authority.

### Checkpoint 6 — Reduce allocation traffic and capacity waste

1. Re-run the allocation-source census on the surviving design. Expand the
   parser's currently unclassified regions if they are still material. Locate
   actual map rehashes, vector replacement, unused page capacity, cooked-string
   temporaries and repeated list compaction before choosing reservations.
2. Prefer exact counts already available from construction. Compare bounded
   per-file hints only when counts are unavailable; use file-size/kind histograms
   and tiny-file/adversarial cases. Do not hardcode nodes/8 or nodes/6.6 as a
   universal allocation requirement or tune to individual workload paths.
3. Compare page size/directory reservation separately from payload representation.
   Both runtimes already keep old pages stable; reserve only where it reduces
   measured request traffic without excessive slack or new giant allocations.
4. Consider reusable per-worker scratch for genuinely temporary parser/binder
   work. Include initialization and retained scratch in the measured pipeline
   and RSS. No excluded warmup may preallocate compiler storage for a child.
5. Do not compact by allocating a second complete graph then discarding the
   first: that can lower endpoint live bytes while failing allocation and peak
   RSS. Any freeze-time compaction must report its transient/request costs.

Exit: request traffic and capacity fit the combined candidate's budget, with
bounded small-file overhead and no hidden work outside the measurement interval.

### Checkpoint 7 — Resolve the remaining CPU/RSS gap

1. Take new CPU and retained-memory profiles of the combined candidate. Rank
   current costs; the baseline's ordering is no longer assumed correct.
2. If CPU misses, separate useful traversal, remaining field resolution,
   allocations, hash work, stack guards and waiting. Capture waiting/lock evidence
   if needed; the existing Running-only samples cannot rule out contention.
3. If live storage fits but RSS misses, inspect allocator statistics and native
   VM domains before changing allocation strategy. Report stacks, mappings,
   rounding, retained free pages and unknowns separately where observable.
4. Prefer fixing demonstrated size-class/page/retirement behavior under the
   existing mimalloc policy. Mimalloc remains fixed in the selected plan and
   existing acceptance validators. A different allocator needs a separate
   reviewed amendment to ADR 0016, the producer and report validator, artifact
   configuration, native accounting tests and diagnostic calibration before
   it can become an eligible production candidate. That amendment must preserve
   thresholds, work and measurement domains; both normal and allocation builds
   must use the same selected allocator. Never force collection, trim
   after the fact, drop roots early, subtract baseline RSS or alter Go's GC.
5. Reassess the held alternative using its activation conditions. If no examined
   candidate can pass, report the measured blocker and next specific experiment.
   Do not declare the targets impossible from this plan or relax them to finish.

Exit: either a combined candidate ready for acceptance, or a documented decision
to activate the alternative / investigate a named remaining cost. This checkpoint
prevents an endless sequence of small optimizations with no route to the gates.

### Checkpoint 8 — Independent review and final acceptance

1. Review ownership/semantics and performance/accounting independently. Reviewers
   receive the final representation, all failed candidate records and raw results.
   Review borrowed facade materialization, fallback correctness and retained roots
   explicitly; a generic code review is insufficient for these changes.
2. Remove obsolete production paths and temporary dual-representation features.
   Update generator manifests, provenance/coverage mappings and current design
   documentation. Stage the final file inventory before recording evidence.
3. Run the affected correctness producers and all native checks listed below.
   Fix failures before freezing the exact candidate for full workload parity.
4. Run fresh full `bindworkload` graph comparison on the exact normal binaries,
   then one complete unchanged E5/E6 capture. Consume that same capture for both
   producers, inspect actual gate results, and verify the whole S07 exit.
5. Deliver implementation through the separate S07-bis branch and PR, based on
   the S07 lineage. Use new commits and normal pushes there; do not add these
   plans or performance work to the S07 branch/PR. Do not rewrite published
   history or stage unrelated work. Preserve failed captures and final raw
   artifacts with their provenance.

Done means `cargo xtask check S07` passes with current evidence. It does not
mean full E3, E5's checker per-type footprint, or any future checker sprint passes.

## 6. Screening and decision rules

Implement a small diagnostic runner under the checkpoint-0 tools directory,
reusing existing build/input/protocol validation. It may compare isolated
checkouts/build artifacts; it must not infer an executable from a guessed path
or rebuild concurrently with timing. Commands for this new runner will be
documented when implemented; it does not exist yet.

Existing build helpers copy executables into shared cache destinations. After
each variant build, immediately copy its normal/allocation executable and
required runtime artifacts into a distinct immutable experiment directory.
Bind each to that variant's source/configuration and check hashes before and
after every sampling batch. Never measure both variants from the shared helper
paths. Negative runner tests must reject overwritten/swapped binaries, changed
loaded bytes/options, partial sample sets and missing allocation observations.

Use the full frozen 13,094-file workload for screening. Its native runs are
short enough to avoid introducing a conveniently selected performance subset.
For each predeclared candidate, use one validated warmup and seven fresh-process
control/candidate pairs at both worker counts, separately for normal and
allocation builds; alternate order. Keep all rows, failures and stderr.
Screening validates work counts and loaded input identity; it emits no E5/E6
metrics and cannot substitute for complete graph comparison.

The default promotion rule is a demonstrated reduction in the targeted cost
and a median candidate/control ratio <= 0.95 in at least one full-pipeline
metric/mode. Every timing mode must additionally have bootstrap upper 95%
candidate/control ratio <= 1.02 and both variants' relative MAD <= 5%; a claimed
timing win needs upper < 1.0. Use the existing fixed bootstrap algorithm for this
diagnostic comparison, with candidate/control identities named explicitly.
For RSS and allocation, require median candidate/control ratios <= 1.02 and
both variants' relative MAD <= 5% in every mode; report all samples/ranges and
make no confidence-bound claim for those two domains. These are screening rules,
not the final E6 requirement, whose upper bound remains <= 1.0 against Go.

If those noise/upper-bound conditions fail, mark the screen inconclusive or
regressing; do not silently extend it. A necessary representation-infrastructure
step can be retained after review without a 5% win only if it meets every 1.02
non-regression/noise condition above, with its dependent checkpoint named.
Any other tradeoff requires a recorded combined-candidate result showing a
better route to all gates; no individual savings are added to predict that result.

Before promoting a complete storage slice as the next implementation control,
run full relevant parity and ownership evidence. A diagnostic winner alone is
not permission to leave its semantic fallback untested. Re-run screening only
for a changed candidate or documented environmental/instrumentation defect;
do not repeat an identical candidate until a favorable batch appears.

Useful supplementary guardrails are tiny files, declaration-heavy files,
escaped-name/JSX/JSDoc cases, sparse multi-source owners, imported/lazy graphs,
deep input and hostile identifier sets. Their selection is committed before
results; they never replace the acceptance denominator. Existing graph
`--diagnostic` mode still uses the full request set, not a subset selector.

## 7. Validation and unchanged final measurement

Each checkpoint runs affected debug/release tests and focused semantic and
compile-fail counterexamples. Representation/ownership changes must also enter
the real E3 scenario manifests and execute under strict-provenance Miri and
ASan with rebuilt std. Preserve existing S04/S06/S07 scenario scope; no bare
instrumentation boolean may complete unrelated future criteria.

Before final acceptance, run workspace debug/release and doc tests, pinned
MSRV/all-target/all-feature checks, generator verification, formatting, Clippy,
dependency policy and tracker self-tests. Build and run affected native paths
on macOS arm64/x86_64 and Ubuntu arm64/x86_64, including stack growth and
allocator accounting. CI checkouts must initialize embedded upstream assets.

At minimum, AST/arena changes require fresh `scanner`, `e1`, `binder`,
`program`, `e3`, `bindworkload`, `e5`, `e6` and affected quality evidence.
AST/generator edits also renew `gen`; an arena-only implementation edit does
not invalidate its deliberately narrowed source closure.
Inspect [every declared input closure](../status/runs.toml): renew `e4`, `e2`,
`workspace`, `oracle` and prerequisite sprint evidence whenever those inputs
change. Do not broaden source globs merely for convenient bookkeeping.

Final commands, after all source and file-inventory changes are frozen:

```sh
cargo xtask run bindworkload
python3 scripts/s07_benchmark.py capture
python3 scripts/s07_benchmark.py verify-capture
cargo xtask run e5
cargo xtask run e6
cargo xtask validate
cargo xtask status
cargo xtask check S07
cargo xtask status --check-committed
```

Archive each previous capture before using the standard output directory; the
consumer commands above read that explicit complete current capture. Evidence
must bind current sources, actual normal/allocation executables, registry lock,
effective Cargo configuration, toolchains, workload transport and full graphs.

The final statistical policy remains one warmup, seven recorded samples per
runtime/mode/configuration, alternating runtime order. Timing extends to 14 or
21 only under the existing fixed uncertainty/noise rule. Clear failures stop;
unresolved uncertainty at 21 fails. Keep the 10,000-resample bootstrap, seed
`0x5307`, fixed order-statistic bounds and both-runtime MAD conditions unchanged.

Under the selected, unchanged allocator policy, retain the normal Rust mimalloc
executable for timing/RSS, a separately
identified cap build for original-layout allocation requests, and normal Go
`GOGC=100`, no imposed `GOMEMLIMIT`, `GOMAXPROCS` equal to workers, and
`GOTOOLCHAIN=local`. Preserve per-file parse→publish→bind order, bounded queues,
retained endpoint roots, and the distinct preload/worker/pipeline boundaries.
Requested allocation, live storage and lifetime peak RSS are never substituted
for one another. Toolchain pins come from their existing manifests, not this doc.

## 8. Decisions deferred to measured checkpoints

The implementation path is selected. These are bounded engineering decisions
within it, not reasons to postpone starting checkpoint 0:

| Decision | Default / bounded challenger | Resolve by |
| --- | --- | --- |
| Binding field indexing | Eligible-kind direct columns / packed sparse pages | 0 layout model, 2 pipeline measurement |
| Node layout | Small header + per-kind pages / reject if facade/directory cost defeats budget | 0 model, 3 vertical slice |
| Internal edge/text width | Owner-relative handles + full-range escapes / retain full ID where compression loses | 3 correctness and census |
| Runtime identity cache | Lazy owner-held storage / existing inline field | 3 identity traces and timing |
| Symbol hashing | Current keyed hashing / one reviewed fast keyed candidate | 4 observed hash cost |
| Validation | Keep final scan, remove proved repeated construction checks / later completion proof | 5 counterexamples and profile |
| Capacity | Available exact counts, bounded hints / current pages | 6 per-file traffic and slack |
| Publication lifecycle | Current per-source parsed/bound cells / held exclusive alternative | 2, 3 and 7 activation review |

No checkpoint is credited in advance. The implementation record must state the
last completed checkpoint, accepted/rejected candidates, current four ratios,
remaining uncertainty and the next experiment capable of changing the decision.
