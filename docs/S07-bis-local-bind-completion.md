# S07-bis: selected local binder migration completion

Status: implementation and source review of the selected slice are complete;
final validation and the combined performance screen are pending. This records
`bindings.rs`, `declarations.rs`, `expressions.rs` and `statements.rs`, with their
dispatch/container/flow adapters, under the
[local binding plan](S07-bis-local-bind-plan.md). “Completion” names that slice,
not an entirely facade-free binder or completed S07-bis gates. CP1 remains the
accepted performance control.

## Implementation and shared contracts

Expression/statement binding now carries `BindingNode`, `BindingList` and
resolved `BindingEdges` through operand evaluation, optional/logical branches,
loops, switch clauses, labels and recursive binding. The typed payload adapter
copies only the selected scalar fields and non-owning links needed across
mutation. It does not materialize an owned payload. Narrow flag/flow writes
remain separate from immutable syntax edges, and flags are reread after binding
where the algorithm requires live state.

Declaration/property/function binding carries the same syntax handles together
with `BindingSymbol` and `BindingTable`. New symbols/tables are minted locally;
reads and narrow mutations address their stable arenas by local slot. Ordinary
node symbol/locals reads and symbol/local-symbol/locals writes use generated
u32 fields directly; they do not reconstruct a public ID and pass it through the
general encoder. Checked variants remain available for exceptional references. State rows and table
entries still contain public links: following those links imports them into the
scope or retains the checked variant. This does not claim removal of every
reference codec or owner check from symbol processing.

The semantic inventories have one source rather than separate checked/local
algorithms:

| Inventory | Source and adapters |
| --- | --- |
| Node getters, kind-driven failure rules, shape-driven modifiers/body/function fields, locals-container shapes | `ts_ast::node_semantics`, expanded by `NodeAccess` and `BindRead`; storage adapters select borrowed checked or local typed rows. |
| Concrete shape names, tags, declaration-name capability and payload/child access | The existing AST schema and `xtask::gen::ast_local_read`; generated local reads include the shape metadata needed by the shared rules. |
| Parentheses/partially-emitted wrappers, entity/dotted names, left-hand-side classification, optional-chain and logical/coalescing predicates | `ts_ast::syntax_helpers`, used by checked `AstView` and scoped `LocalBind`; only storage reads and error types differ. |
| Fixed kind predicates and container classification | Shared utility predicates and the existing container-rule inventory; dynamic reads occur only for rules that need them. |

Mixed checked/local identities compare their canonical public identities;
local/local comparisons use their scoped slots. An invalid or not-yet-allocated
compatibility link is not silently promoted into a valid local reference.
Optional table results preserve absent keys versus present nil entries. Scoped
symbol/locals writes use the existing binding-storage rules, including an
explicit empty binding, promotion of a flow-only record, unsupported payloads,
field clearing and the precedence of materialized compatibility records.
Whole-arena replacement and unrestricted syntax mutation remain unavailable
inside the callback. Entry also verifies that the core's stored binding
namespaces match the current symbol/table/flow arenas. A checked caller can
replace arenas between callbacks; re-entry must reject that mismatch before
minting local handles, even when replacement slots have identical numbers.

## Review and remaining boundaries

Independent source review of the expression/statement/flow/dispatch integration
found no actionable regression in selected-field snapshots, nil/list ordering,
initializer early exits, conditional and try/finally flow restoration, binary
continuations or live error flags. The review checked behavior against the prior
implementation; it is not independent Go execution or a substitute for the
pending full checks. Independent state review identified the arena-replacement
re-entry case above and six changed malformed-payload panic messages. The entry
check and regression now cover rejection, preserved checked IDs, escaped writes,
clearing and restored-namespace re-entry. Selected payload observations accept
the original call-site panic message; malformed infer-parent/export fixtures
exercise both backends. Final validation results are recorded below only after
their runs finish.

Explicit general boundaries remain: modifier and async/ambient queries,
assigned/dynamic/declaration-name helpers, diagnostic construction and source
metadata, unmigrated module/expando/resolver operations, declaration backing,
lazy or foreign nodes, and public flow/symbol/table links. Identifier text may
be borrowed locally before producing an owned `JsString`; other text forms keep
the checked text operation. Locals-container capability has a shared actual-shape
inventory (29 shapes), distinct from the 27 shapes with an inline locals field.
Unsupported state encodings retain their general storage path. Reaching a local scope for
every file does not establish that every operation inside it stays local.

## Cost and measurement ledger

The [single untimed native profile](S07-bis-local-bind-migration-cpu.md) covers
the preceding migration binary, not this later source. Its remaining
`AstView::node`/`NodeRead::data` samples primarily identify shared helpers and
validation; it provides no measured speedup for this completion slice. Its raw
trace, failed setup attempts and exact binary/source identity are archived.

The selected earlier binary stores 24-byte, 8-byte-aligned `Step` continuation
values; `Exit` is also 24 bytes. The current definitions retain that layout
structure. Final-artifact confirmation, peak continuation capacity and any
resulting request traffic remain unmeasured. Local/checked adapter enums and
selected-field tuples can enlarge live stack observations; local text conversion
can allocate. Borrowing typed rows alone does not establish a memory saving.

The [allocation attribution](S07-bis-allocation-traffic.md) still describes its
own frozen binary: 333.733 MB of traffic, including 108.289 MB unclassified.
This slice chooses no new backing policy and does not spend an assumed saving
from that report. All new or changed costs belong in the combined candidate's
allocation/RSS and parse/bind assessment.

| Obligation for this completion slice | Result |
| --- | --- |
| Affected debug/release/doctest, generator drift, formatting, Clippy and MSRV checks | **Pass.** Both modes: 30 arena, 158 AST, 45 binder and 63 tooling library tests, 29 AST integration tests and 37 doctests. All 22 generator outputs and pinned client bytes match. Final Clippy denies warnings; Rust 1.96 and formatting pass. Final check source inventories match. |
| Relevant strict-provenance Miri and ASan paths, including aliases/state presence and deep recursion | **Pass (scoped).** Miri: 3 mutable arena-scope and 27 AST local-binding tests. ASan: 27 AST and 20 binder tests, including deep binary chains. After lint cleanup, the 3 AST state cases were rechecked under Miri and all 20 selected binder cases under ASan. This is not a full ownership-producer or Darwin LeakSanitizer claim. |
| Frozen-source canonical Go/Rust graphs and local-path selection, one/eight workers | **Pending final record.** Preserve exact executable/source and raw/protocol-qualified differences. |
| One complete combined screen against CP1: both CPU modes, allocation and RSS | **Pending.** Use the fixed schedule and report every gate distance. |
| Matched parse-versus-bind elapsed attribution for that complete combination | **Pending.** Native sample weights are not phase elapsed times. |
| Retain/reject decision and final Go-relative acceptance | **Pending.** No promotion or gate claim follows from source completion. |

A component's CPU regression is not a hard veto on retaining a memory experiment
while its named dependent CPU work is integrated. Judge the actual combination
in one screen, using all gate distances and the existing noise/tradeoff rules.
Do not add separately measured savings or use a weighted score to waive a final
gate. If the fixed screen is inconclusive or regressing, retain that result and
make the next decision explicitly; do not extend it until a preferred answer
appears.

Initial checks are retained. Generator checking first hit the sandboxed default
Go cache; rerunning with the existing workspace cache passed. Clippy found
explicit-default test style, nested compatibility option states and ordinary
style issues. Compatibility selection now shares the existing `ScopedValue`
enum across flow, symbol and table adapters; actual table results retain nested
options because missing keys and present nil entries differ. One incomplete
enum conversion failed compilation and was corrected before the passing final
debug/release/lint/MSRV run. The final cleanup instrumentation has unchanged
before/after inventories. The complete earlier scoped run and its source
inventory remain recorded separately.
