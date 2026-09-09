# S07-bis: local binding access and current allocation traffic

Status: the [selected binding/declaration/expression/statement slice is implemented](S07-bis-local-bind-completion.md),
including scoped syntax/list and symbol/table consumers, after the
[dispatch/list/flow milestone](S07-bis-local-bind-migration.md) and
[first local scope](S07-bis-local-bind-milestone.md). Its combined screen qualifies
for review, but the full ownership capture rejects stale test inventories. The
[owned-text and inventory repair](S07-bis-local-text-repair.md) is implemented;
its final validation and new combined screen are pending. Explicit shared-helper, diagnostics, lazy and
public-reference boundaries remain. This is not a claim that the entire binder
is facade-free. This supersedes the standalone experiment sequence in
[the flow/source-fact plan](S07-bis-bind-cpu-plan.md). Its semantic counterexamples
remain applicable; those changes are components of this design.

The [current allocation attribution](S07-bis-allocation-traffic.md) is complete:
225.443 MB of 333.733 MB traffic is assigned to observed backings; 108.289 MB
remains unclassified. Typed-row directory and text-entry growth are the largest
newly observed mechanisms. Even eliminating both replacements completely would
save at most 125.378 MB, below the roughly 209 MB request deficit, without proving
any RSS saving. No single memory rewrite is selected from those ceilings. The
local binder implementation proceeds; attribution is no longer its prerequisite.

## Decision and limits

The main CPU candidate is a local binder view over the existing compact storage.
The main memory investigation is attribution of current allocation requests,
retained backing and replacement traffic. A small proof burden is no longer a
reason to exclude the binder-wide access work. Bounded means explicit contracts,
an early implementation milestone and a decision after a fixed complete screen.

The [bind-only comparison](S07-bis-bind-cpu-comparison.md) supports investigating
repeated access machinery, but does not prove a removable two-second cost.
Its Rust total includes allocation/validation; dividing it by Go's
runtime-subtracted remainder does not establish a 20–25x algorithm ratio.
Likewise, eliminating the allocation deficit requires about 63% of observed
freed/superseded traffic if retained storage is unchanged. That arithmetic
does not establish a saving or close the separate RSS gate.

## Memory attribution on the current freeze

Use a disposable diagnostic copy of the existing corrected `8f7236e` freeze,
whose normal executable is `3b375719ca8c3334c443f77b934c1388eec42ec1a94d12971586bc792056e8c2`.
Keep that source distinct from subsequent production edits. Reuse fresh Cargo
output/intermediate isolation, loaded-work verification, allocator semantics and
the calibrated name/table measurements.

Record allocation requests and replacement releases for actual growable backing
families: core/auxiliary arenas, typed rows and their directories, edge pages,
symbols/flows/declarations, text/name/table backing and parser temporary lists.
Separate payload pages from their directories. Charge each backing once; do not
sum nested function-scope observations. Record parse versus consuming-bind
phase for events where it is observed, and keep main-thread/unassigned events
explicit. Replacement means the old requested capacity superseded by growth,
not bytes physically copied. Record final backing separately from logical data.

Use safe capacity/size observations at actual storage operations and fixed
counters, calibrated against the same cap allocator on small growth/shrink/drop
fixtures. A family whose lifetime or request layout is not observed remains
unclassified. The old scope collector records requests only and depends on the
obsolete enum census; do not port that census or mislabel its scopes as free
attribution. Global phase snapshots and family sums must reconcile with explicit
residuals; do not assign residual bytes to an assumed cause.

After calibration, execute one full one-worker diagnostic with matching counts,
digest and binding-path selection. Retain the patch, inventory, executable,
logs and failures. This run supplies neither timing nor RSS acceptance. Select
one memory change only if its measured mechanism and replacement-storage budget
are relevant to the remaining 209 MB request / 105–110 MB RSS deficits. No page
matrix, general backtrace profiler or another per-field access trace is planned.

## Local binder access contract

1. Enter through the consuming binder's validated, eligible single-source path.
   Mint an invariant scope tied to that owner and binding result. Local syntax,
   symbol and flow handles remain distinct types and cannot be forged, escape
   the scope or cross owners. Raw public IDs are checked when imported into it.
2. Read core headers and generated typed rows through local slots with safe
   bounds checks. Keep local child/parent/list links local through traversal and
   helpers, avoiding conversion to a full ID followed by general owner routing.
   Returning `Result` is not itself an overhead diagnosis: inspect the optimized
   path for the checks/context construction the new contract actually removes.
3. Make generated narrow binding writes use those local namespaces. Preserve
   shape/kind separation, field capability, optional links, materialized-empty
   versus flow-only records and cleared-field semantics. Flags and binding
   fields may change; syntax edges retain their completion proof. No unrestricted
   mutation, unsafe indexing or validation-scan reintroduction is permitted.
4. Preserve checked public, published, mapped and imported behavior. Eligibility
   at entry does not exclude lazy records created during initialization. Lazy
   or escaped references cross an explicit checked boundary; they do not become
   unchecked local handles. Text exception pools also remain required. Record
   compatibility use rather than quietly benchmarking a different workload.
5. Use short borrows or small typed observations across recursive writes. Typed
   rows contain both syntax and binding fields, so an immutable whole-row borrow
   cannot coexist with mutation of that row. Resolve this with the existing safe
   storage split and narrow APIs; do not recreate owned nodes or duplicate an
   entire AST to satisfy borrowing.

The existing branded arena is an invariant reference, not a ready mutable
binder implementation. Extend the arena boundary only as required for actual
core borrows. Generate local payload/edge access from the current schema; avoid
a handwritten second inventory. Keep shared helper semantics in one source
where possible; make exceptional general-view conversion explicit.

## Implementation milestones

First implement scope entry, checked import, direct header/typed payload reads,
child traversal and narrow flow writes together. Wire the real binder entry and
identifier/property-access paths through it, preserving the compatibility path.
This milestone establishes the borrowing/namespace contract and exposes whether
general IDs/routing remain inside the intended local path. Inspect optimized
code and exercise real parsed input; do not use a synthetic read loop to promise
a full-workload speedup. It is not a separately promoted performance candidate.

Then carry local handles through remaining binder dispatch, containers, flow
operations and the shared AST helpers they call. Incorporate immutable
source-fact caching and flow capability work from the preceding plan. Record
which paths still convert to the general view and why. A nominal local wrapper
that repeatedly calls `AstView::node` is not completion of this migration.

Review compile-time scope/namespace isolation and runtime bounds, invalid-ID
ordering, escape overwrites, lazy initialization, mapped siblings, updates,
unwind/publication and deep recursion. Run affected debug/release and doc tests,
generator drift, lint/MSRV checks and the real relevant Miri/ASan cases. Compare
full consuming/published binding graphs and then all 13,094 Go/Rust workload
graphs in both worker modes before performance screening.

## Measurement and completion

Preserve CP1 as the accepted control and the current compact freeze as the
experimental starting point. Neither the control nor a gate changes through
documentation. Build the complete local-view combination and any separately
justified memory component with fresh isolated artifacts. Use the primary plan's
one fixed eight-warmup/56-sample schedule at one/eight workers, keeping every
result and applying the combined-tradeoff rules. No standalone flow/source-fact
screen precedes it; no historical component percentages are added. A CPU
regression in a memory component is not an automatic retention veto while its
named dependent CPU work is integrated. Promotion judges the actual combination
across all gate distances, with the existing noise tolerances; a weighted score
cannot waive a final gate. Component diagnostics explain costs and do not decide
promotion independently.

Keep parse versus bind attribution alongside the resulting pipeline assessment:
a binder win does not prove that parsing or either complete CPU mode meets Go.
Report all four gate distances, the remaining general-access paths, current
memory attribution and maintenance cost. If the local design cannot satisfy its
contract or fails to improve the complete result enough, record that result and
replan explicitly. Do not silently continue a succession of small candidates.
Final CPU and memory gates remain in S07-bis and require fresh Go-relative
evidence; neither is promised by this plan.


### Current evidence and cost ledger

The [selected completion record](S07-bis-local-bind-completion.md) inventories the
shared getter/helper algorithms and remaining general boundaries, and records
the completed fixed screen. Earlier graph or instrumentation results retain
their original source scope; broader producer results remain separate from
the passing scoped validation.

| Item | Established evidence / outstanding measurement |
| --- | --- |
| Accepted control | CP1 remains unchanged. The compact/local implementation remains the experimental candidate. |
| Selected migration scope | Four binder files plus their adapters carry scoped syntax/list and symbol/table handles; shared semantic inventories replace duplicated checked/local rules. Affected native checks, scoped instrumentation and both full-workload graphs pass. |
| Earlier migration native profile | [One untimed capture](S07-bis-local-bind-migration-cpu.md), no rebuild or paired screen. Remaining general helper/validation callers were identified; it does not measure this later source or establish CPU improvement. |
| Continuations and temporary observations | Earlier selected binary: `Step` 24 bytes, alignment 8; `Exit` 24 bytes. Final artifact layout, peak capacity, tuple/adapter stack costs and request traffic must be charged; no saving is assumed. |
| Allocation mechanism evidence | [Earlier frozen attribution](S07-bis-allocation-traffic.md): 333.733 MB traffic, 108.289 MB unclassified. The complete local candidate requests 2.302 GB, about 57.9 MB more than that older compact capture; this increase is not yet attributed by family. |
| Final graph/ownership/toolchain validation | Both 13,094-file graph modes, all-local probes, affected native checks, scoped Miri/ASan and full binder parity pass. Full E3 rejects three stale exact inventories, including an unintended substring-filter selection; its failed evidence is retained. |
| Both CPU modes, allocation and RSS | Completed fixed screen qualifies for review: CPU ratios 0.859609 / 0.837626, upper bounds 0.880953 / 0.854378; allocation 2.302 GB and RSS 2.320 GB. All distances improve against same-screen CP1; final Go-relative gates remain open. |
| Parse versus bind | One source-matched elapsed diagnostic: parse 2.230553404 s, bind/publication/validation 1.639655704 s. It is not a paired phase speedup or a Go ratio. |

The fixed schedule remains eight warmup executions and 56 recorded executions
across the normal/allocation binaries, both worker counts and both variants.
Freeze and hash-check the selected compiler artifacts and their source/configuration
before measuring. Report all rows, failures and all metric distances. Do not
launch another small screening sequence or silently lengthen an inconclusive
batch. Retain the completed combination as a measured candidate, without
promotion while E3 is false. The next revision corrects its ownership inventory
and filtering, and forwards local owned text through the existing source/pool
retention operation. It needs its own source-bound validation and combined
measurement. No fresh Go gate is claimed from this CP1 comparison.
