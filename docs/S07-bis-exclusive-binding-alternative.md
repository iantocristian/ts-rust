# S07-bis alternative: exclusive binding before publication

Status: **on hold; not selected for implementation**.
Date: 2026-09-08. This is the alternative derived from Claude's proposal A–E.

The [selected S07-bis plan](S07-bis-performance-plan.md) supplies the unchanged
gates, workload, measurement protocol, regression obligations and final acceptance
procedure. This document records only the different architecture and its order
of implementation. It is not an approval to change observable behavior.

One explicit measurement difference: this alternative changes the internal
order from parse→publish→bind to parse→bind→publish. All three operations stay
inside the same combined per-file measured interval; preload, worker setup,
retained endpoint roots and acceptance metrics remain unchanged. Its diagnostic
phase timers must identify that order and assign final publication to its actual
phase. Update phase adapters and their provenance checks if activated; never
compare old phase labels as though the lifecycle were identical. A hybrid must
record which path each file executed. No binding work moves into excluded setup.

## 1. Proposal and reason to keep it

Put a logical file's exclusive parsed construction behind its initialization
cell. The winning initializer binds through direct mutable access and publishes
the frozen core and BindResult together. Generated concrete payloads contain
only the binding fields applicable to that shape, analogous to Go's
FlowNodeBase, DeclarationBase and LocalsContainerBase. Remove full node overlays,
the general binding-record map and the all-node flow-slot structure.

Combined with compact per-kind syntax, text handles, symbol/flow records and
better capacity policies, this could eliminate field-overlay routing as well as
storage overhead. It is worth retaining because it targets both measured axes
with one ownership model. It does not by itself imply Go-like timing or zero
replacement binding bytes: Go already allocates many such fields during parse.

## 2. Why this is on hold

The original S07 plan proposed exclusive binding, but commit `1637157` replaced
it after the mapped-sibling review. Current S07 exposes parsed observations
before binding, initializes logical siblings independently, and leaves parsed
state intact after a failed bind. These properties currently work through
immutable parsed storage and private per-source staging.

A `OnceLock` around `ParsedFile` is not a complete replacement protocol:

- A borrowed parsed node cannot remain accessible while the same fields are
  mutated in place. Waiting only on new bind callers does not revoke old reads.
- Independent logical sources can share one physical arena. Moving that entire
  arena into one member's bind operation can accidentally bind or block siblings.
- Panic can leave an exclusively mutated graph partially bound. Publishing a
  failure marker does not restore the parsed fields that callers can inspect.
- Lazy JSDoc may already be materialized and retain stable IDs/cache entries;
  existing core-only `node_mut` is insufficient for all binder-visible nodes.
- Imports and retained mapped results must preserve their actual owner graph
  and complete bundle independently of who called binding.

The selected plan first tests the major representation changes without these
lifecycle changes. Correctness obligations are not relaxed to make this
alternative cheaper.

## 3. Activation conditions

Reconsider this alternative after selected-plan checkpoints 2/3 or 7 if one of
the following is demonstrated:

1. Compact typed binding storage and validated local access still leave
   publication/field-overlay routing as a leading measured CPU cost, and the
   remaining current-design costs cannot plausibly meet both CPU modes.
2. The complete replacement-storage model exceeds the memory budget because
   preserving parsed/bound state requires material duplication that exclusive
   ownership could remove; count the alternative's recovery/reader storage too.
3. A bounded ownership prototype demonstrates all existing observations with
   lower complexity and measured cost than the selected path.

Record the specific evidence and decision before activation. The user has
delegated selection between plans, so changing the engineering selection does
not itself require a new permission question. If the alternative requires an
actual observable divergence, prepare its concrete witness and ADR and use the
existing owner-approval process; that is a separate decision. An internal Rust
API migration that preserves behavior does not automatically imply a divergence.

## 4. Step-by-step alternative

### A0 — Resolve the lifecycle before porting the binder

Inventory every parsed/bound caller, retained read, mapped source partition,
lazy node source and panic/reentry path. Draw the state machine and identify
exactly when exclusive access is valid and which readers remain permitted.

Prototype ordinary single-source initialization first as a feasibility test,
with states for unbound, binding, bound and terminal failure. Test two competing
bind callers, a parsed read already in scope, initiator panic, later failure,
recursive initialization and final owner disposal.

Compare two explicit possibilities: an exclusive normal parse-cache path with
the existing general published-unbound fallback, or a fully revised read/publication
protocol. A hybrid is allowed as an experiment, but its retained duplicate code,
fallback coverage and future maintenance are real costs. It must be an actual
production API used by compiler/cache callers, not a benchmark-only shortcut.

Exit: all required observations have a concrete safe implementation or a named
unresolved contradiction. Stop this alternative before a full port if preserving
already escaped parsed reads requires restoring the same graph copies/overlays.

### A1 — Prove sibling, lazy and failure behavior

Handle several logical sources in one physical arena, independent sibling
initialization and failure, and complete mapped-bundle retention. Do not solve
this by eagerly binding every sibling or converting to one group-wide once cell.

Integrate pre-materialized and newly requested lazy nodes without renumbering or
losing cache identity. Define whether the initializer can invoke lazy work and
how lock/reentry order prevents a wait cycle. Keep imported queries narrowed to
the imported owner's retained graph.

For unwind, identify how unchanged parsed observations survive partial writes.
If an undo log or private patch journal is necessary, charge every byte and
recovery operation. A journal is not automatically cheaper than today's overlays.
Run the actual implementation through E3 debug/release/Miri/ASan scenarios.

Exit: no remaining unpublished-reader, sibling, lazy or rollback obligation is
deferred to the later binder port.

### A2 — Bind directly into a compact concrete representation

Build the borrowed facade and generated per-kind storage shared conceptually
with selected-plan checkpoints 1 and 3. Compact identifier text before adding
flow fields to it; avoid widening a uniform enum or mechanically boxing millions
of identifiers. Measure headers, concrete payloads, location metadata, page
capacity and embedded unused binding fields together.

Port each audited binder write to exclusive field mutation. Read local nodes
through the validated owner scope; retain checked foreign/lazy entry and bounds.
Publish the core, symbols, flow graph, diagnostics and bind completion atomically.
Remove overlay/map/slot storage only after every field's read/write parity passes.

Exit: full supported binder graphs and source behavior pass; total requested
allocation, RSS and both CPU modes improve in a matched diagnostic comparison.
Measure against the best selected-plan candidate, not only the original baseline.

### A3 — Compact remaining records and validate once where proved

Apply the selected plan's internal-link/text, symbol/flow/table and list-backing
experiments. Do not assume symbols become 80 bytes or flows 32 bytes without
compiled layouts and total backing accounting.

Audit construction and final validation separately. Initially retain completion
validation while removing only repeated checks proved by private handles.
Exclusive access alone does not validate an arbitrary imported ID or a later
mutated list edge. Preserve checked public factory failures.

Exit: full parity/ownership and a combined memory/CPU budget with actual margins.

### A4 — Capacity, request traffic and allocator diagnosis

Reserve from available exact counts; evaluate bounded per-file estimates where
needed. Treat nodes/8 symbols and nodes/6.6 flows as workload observations, not
universal constants. Preserve stable page identities and sparse/tiny-file behavior.

Measure all request traffic, including conversion/journal/compaction storage.
Do not attribute 923 MB to arena copying: both runtimes already allocate stable
successive pages. Inspect VM/allocator domains if RSS remains above the gate.

Exit: the combined alternative passes diagnostic headroom targets or has a
specific measured blocker. Do not promote it on structural elegance alone.

### A5 — Compare, select and accept

Compare the complete alternative and best selected-plan candidate using the same
frozen workload, pins, measurement boundaries and fixed screening policy.
Require unchanged semantic work and owner behavior. Select one production path;
if a hybrid remains necessary, state why and test each path plus transitions.

Review independently, remove obsolete paths, renew every invalidated producer,
run current full workload graph parity and the unchanged E5/E6 capture, then
require the whole S07 gate to pass. Follow the selected plan's final acceptance
and delivery procedure. No gain forecast or passed local microbenchmark replaces it.

## 5. Claude suggestions adopted, qualified or deferred

| Suggestion | Treatment |
| --- | --- |
| A: exclusive binding with concrete binding fields | This held alternative; compact concrete fields and access improvements are already in the selected plan |
| B: smaller nodes, selective caches and compact text | Adopted in the selected plan, with replacement/alignment budgets and cooked/foreign text support |
| C: validate once | Adopted as a proof-driven duplicate-work experiment; not a promised 15% gain |
| D: faster name hashing and table reservation | Adopted as measured, byte-semantic, keyed-hash and capacity experiments |
| E: pre-size arenas | Adopt exact/bounded reservations; reject the premise that Rust repeatedly copies old arena pages while Go allocates once |
| Predicted 1.9 GB retained and Go-like CPU | Hypotheses only; require all replacement storage, missing RSS domains and direct paired measurements |
