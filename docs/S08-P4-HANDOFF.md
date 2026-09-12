# S08 P4 handoff

The owner requested a pause after active work. **P4 is incomplete; P5–P8 have
not started.** This is an implementation checkpoint, not an acceptance result.
The original request was P4 on PR #19, then one stacked PR per later checkpoint.

## Starting point and scope

- Branch: `codex/s08-p4`; parent PR #19 is `codex/s08-p3-remaining` at
  `37097c4211c1fef3e0ef4e68bdfb7aac68ef8f1e`.
- Pinned Go tree: `1f70213d4922b434345f639b441681e470c7cfc1`.
- Read `docs/CODEX-RUST-GUIDELINES.md` before implementation and
  `docs/S08-implementation-plan.md` for the accepted checkpoint scope.
- Acceptance remains **9,369** variants, with **1,359** informational variants
  separate. No thresholds, eligibility predicates or divergence policy changed.
- Requested declaration diagnostics are required, including the declaration
  transformer and checker emit resolver. An unexecuted phase is a failure even
  when native diagnostics are empty. Focused fixtures cannot certify P4 or E2.
- Never rewrite pushed history. Preserve unrelated `.playwright-mcp/` and
  `four-robots.png`; neither belongs to this checkpoint.

## What is implemented

The checker now has production expression, call, contextual-type, statement,
flow, return/predicate inference, value declaration, class, enum, module and
JavaScript/JSDoc paths. These call the P3 type/instantiation/relation machinery.
Named unsupported branches remain and must be resolved from the full inventory.

`ts_pseudochecker` ports native syntactic inference separately from semantic
checking. `ts_transformers` implements the declaration transform used by
`Program::declaration_diagnostics`; `ts_printer::emit_resolver` defines its checker
callbacks. Output is constructed and graph-validated, rather than emulating
only a list of expected diagnostics. The Program phase retains successful
results; callbacks execute outside the result-cache lock.

Node-builder work includes symbol accessibility, referenced-value resolution,
class/function object serialization, annotation reuse with recovery boundaries,
pseudo-type equivalence, and lexical scopes for signature serialization. Emit
metadata retains original nodes, generated-name identity and comment ranges.
Generated placeholder text must not be printed as a real name; remaining source
printing/name-generation work belongs to P5.

Serialization scopes are checker-owned factory nodes and binding tables; source
parents are explicitly retained. Published bound files stay immutable. Scope
allocations are charged by the census but currently live until the checker
generation is dropped, unlike native builder-local scope links. P7 must measure
this retention. Avoid premature storage redesign.

The last active implementation slice is declaration module-specifier generation
and import-attribute closure. It preserves path bytes, import resolution modes,
package JSON identity and native path/extension ordering. The final verification
section below distinguishes its compilation from native parity coverage.

## Evidence already collected

The first full inventory is **an older immutable binary**, not the final source:
`target/s08/p4-inventory-01`. Its authenticated replay covers all 10,728 variants.
Capture manifest SHA-256:
`f251dd64c0651536ee57ecac85ef19375517f819410d0e1ea2682cd2668e653c`.
Acceptance: 6,185 executed semantic diagnostics, 3,023 failed there, and 161
panicked before the phase. Only 5,433 completed all requested diagnostic phases.
Informational: 932, 413, 14, and 827 respectively. These are execution counts,
**not parity counts**. Many of the named failures have since been implemented.
Do not quote these numbers as the current pass rate.

Focused results on recorded earlier combined binaries include:

- All **420** frozen P0 relation actions: results, calls, cache/state transitions,
  type creation counts and diagnostics. Latest pre-pause replay is
  `target/s08/p4-relations-comparison-reuse-01.json`. Remaining full relation
  protocol/display/performance obligations are P5/P7, not certified here.
- **850** complete native pseudo-type tree/error-location observations. A
  4,000-level const tuple is constructed and released on a 512 KiB stack.
- **504** native declaration-accessibility selector observations.
- **90** downlevel/private-field/helper/options programs and queries.
- Assignment declarations **23/23**, isolated declarations **7/7**, and tagged
  calls **10/10** on `target/s08/p4-build-reuse-01/p2_checker`.
- Enum **15 programs/66 queries**, classes **11/27**, and contextual expressions
  **7/28** on `target/s08/p4-build-class-emit-01/p2_checker`.

Each copied build directory records a source fingerprint and executable hash.
Do not compare new source against an old binary and call it verification.
`target/s08/p4-inventory-01` retains its executable, source snapshot, request
rows and phase outcomes. Other local native captures and results also live in
`target/`; they are not silently promoted to committed evidence. Preserve them
when handing the checkout over. Request specs and oracle/replay code are tracked.

## Shared declaration failure and its fix

Several suites failed with `Checker(Arena(InvalidGraph))` when declarations
contained ordinary calls, including zero-argument calls. The failure was in the
transformer's **eager accessibility selector**, not an arena/storage invariant:
it read the special call context's second argument while installing the context,
even though native defers that read until a diagnostic actually uses it.
The fix defers selector results/errors and propagates an error only if that
selector is selected. Its regression is
`ordinary_call_context_defers_an_unused_accessibility_selector`.

Other recent native fixes include the CommonJS repeated-export assignment of
`undefined`, tagged-call diagnostic guards, contextual annotation reuse for
qualified generic constraints, and native class/private-member diagnostic
selection. Keep their focused request cases when refactoring.

## Resume order

1. Read the final verification and agent handoff notes below. Inspect the draft
   PR diff and current tree before changing anything. Do not repeat all native
   captures or broad lint suites merely to rediscover their recorded outcomes.
2. Address any compile or focused-parity failures explicitly retained below.
   Native module-specifier/attribute parity and remaining node-builder boundaries
   need particular attention; compilation is not proof of their behavior.
3. Build a **new immutable** `p4_inventory` executable and source snapshot, then
   run the full frozen inventory once. `python3 scripts/s08_p4.py --help` gives
   the capture/replay interface. Start from the largest *current* failure buckets,
   not the obsolete rankings in inventory-01. Preserve acceptance/informational
   partitions and distinguish harness failures, panics and named unsupported
   operations from native diagnostic differences.
4. Finish P4's required operations and direct lazy-library fixtures. Inspect
   `Error::Unsupported` paths in the checker and declaration transformer against
   actual requests; do not weaken eligibility to make them disappear.
5. Review and complete P4, then start P5 on its PR. P5 owns exact type/error
   baseline decoration, remaining printer/metadata/byte closure and nested stack
   tests; P6 owns exact E2 acceptance and ownership instrumentation; P7 owns the
   full relation/type-storage/performance measurements; P8 refreshes stale
   evidence, status and S07 captures, then runs the four native CI targets.

Typical focused replay (use the canonical request emitted by capture):

```sh
BINARY=/absolute/path/to/immutable/p2_checker
NATIVE=/absolute/path/to/native-capture
ACTUAL=/absolute/path/to/new-results.json
"$BINARY" "$NATIVE/requests.json" "$ACTUAL"
python3 scripts/s08_p2.py compare --native "$NATIVE" --actual "$ACTUAL" --output /absolute/path/to/comparison.json
```

Do not pass the pretty-printed `tools/s08/p4/*.json` specs directly to the Rust
adapter; it deliberately requires the canonical native request bytes. The
comparison command reports the first failed phase; the actual-results JSON
retains all program outcomes for diagnosis.

## Final verification and remaining work

The final immutable build is `target/s08/p4-pause-build-02`. It compiled
`p2_checker`, `p3_relations`, `p3_comparators` and `p4_inventory` with all compiler
features and `ts_checker/storage-pilot`, without warnings. Source fingerprint:
`651693857c9e281fbe7e21dafb8b6d359cfcf3eb6960adc60b0e902720840bc9`.
The `p2_checker` binary SHA-256 is
`e0404c3f821a1712d33698bf39df6c8e9bceaed41c9779cc1f163c39d2d36949`.
`build.json` includes every source hash; `source-snapshot/` retains those bytes.
`inventory-build.json` records that same snapshot. A subsequent bookkeeping-only
edit corrected 18 source-reference comments; a checked diff confirms non-comment
lines are unchanged (`p4-pause-checks/comment-only-delta.json`). The strict source
fingerprint consequently changed: rebuild before a new inventory capture, rather
than editing the recorded hashes.
The earlier pause-build-01 failed compilation and contains no copied binaries.

The consequential deferred-selector regression passed. The existing pinned-Go
path-comparison observations passed after sharing the relative-path helper.
Tracking views were regenerated after correcting the source-reference comments,
and `cargo xtask validate` passed. Existing correctness/performance evidence is
stale where the changed inputs require it; no producer outcomes were fabricated.
No full corpus, fresh Go captures, broad lint rerun, performance capture or
four-target CI was performed for this pause. CI is deliberately deferred on the
draft checkpoint (`[skip ci]` commit), not reported as passing.

Latest full-program focused comparisons total **94/98 exact**:

| Suite | Exact | Comparison under `target/s08/` |
| --- | ---: | --- |
| Ordinary declarations | 13/13 | `p4-pause-checks/declaration-emit-comparison.json` |
| Generated declarations | 8/8 | `p4-pause-checks/declaration-generated-comparison.json` |
| Pseudo/semantic serialization | 12/12 | `p4-pause-checks/declaration-serialization-comparison.json` |
| Annotation reuse | 14/14 | `p4-annotation-reuse-comparison-pause-02.json` |
| JavaScript declaration classes | 5/5 | `p4-pause-js-classes-comparison-01.json` |
| CommonJS declarations | 13/14 | `p4-pause-common-js-comparison-01.json` |
| Import attributes | 16/18 | `p4-pause-attributes-comparison-01.json` |
| Class expansion | 13/14 | `p4-class-expansion-final-verified.json` |

Four known focused failures are ready for the next implementer:

1. **CommonJS destructured `require`:** `module_aliases.rs`'s
   `get_target_of_alias_declaration` handles `VariableDeclaration` but not
   `BindingElement`, producing the named Unsupported external/CommonJS alias
   result. Native capture: `target/s08-p4-declaration-common-js-native-01`.
2. **Two import-attribute diagnostic chains:** `nonstring-value` and
   `module-attribute-method` omit intermediate TS2530. In
   `relater_properties.rs::members_related_to_index`, a failed `self.related`
   returns false without reporting `Property_0_is_incompatible_with_index_signature`.
   Pinned `relater.go:4688–4693` adds that wrapper using `symbolToString(prop)`.
   Native capture: `target/s08/p4-import-attributes-native-01`.
3. **External unique symbol declaration:** `external-unique-value` emits extra
   declaration TS2527 for `outer` at `/case.ts:42..47`; native emits none. All
   semantic/query results match. Native capture:
   `target/s08/p4-declaration-class-expansion-native-03`.

The new `module_specifiers*.rs` generation/inversion and compiler host data APIs
compile, but dedicated native module-specifier parity is **unmeasured**. The host
retains the loader resolver's package cache and uses its existing package-directory
algorithm only for native runtime-dependency discovery. Existing import resolution
is not recomputed. Contextual export-equals selection, external-container ranking
and generic qualified-value paths still contain named node-builder boundaries.
These are follow-ups; the new path generator does not certify the entire name
serialization closure.

The current source still needs a new full execution inventory. The next
implementer can build a fresh authenticated snapshot and run:

```sh
python3 scripts/s08_p4.py build --output target/s08/p4-resume-build-01
python3 scripts/s08_p4.py run --tier all --build-record target/s08/p4-resume-build-01/build.json --output target/s08/p4-inventory-02
python3 scripts/s08_p4.py replay target/s08/p4-inventory-02
```

After a source edit, run `scripts/s08_p4.py build --output <new-build-directory>`
first; the runner correctly rejects stale binary/source pairs. The old full
inventory is useful for provenance and historical attribution, not the current
remaining-work count. All subagent tasks and local build/replay commands have
completed; no background continuation is scheduled.
