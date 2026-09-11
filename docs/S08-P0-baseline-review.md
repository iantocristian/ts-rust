# S08 P0: baseline authority review

Status: **historical review, resolved by the owner's [accepted amendment](S08-acceptance-amendment.md)** (2026-09-11).
The proposals below preserve the reviewed alternatives; the owner chose
informational, non-gating treatment instead of ordinary required projected cases.
PR: [#13](https://github.com/iantocristian/ts-rust/pull/13).
Upstream: `1f70213d4922b434345f639b441681e470c7cfc1`; Go 1.27.1, darwin/arm64.

P0 now has a complete native Go observation capture. It exposes a conflict
between the frozen S07 inputs and the plan's native-runner baseline authority.
Freezing an expected query schedule now would bake that conflict into S08.
No denominator, threshold, frozen input or divergence entry has been changed.

## What actually ran

`scripts/s08_baselines.py` invokes the original compiler test constructor,
pre/post diagnostic collection, emit, and raw baseline writers through read-only
Go overlays. It observes the original walker's type/symbol pulls and display
calls, preserving order. It omits old-Strada reference comparisons and records
failures/skips instead of converting them to absent baselines. These are **Go
observations, not Rust parity results**.

| Full capture outcome | Variants |
| --- | ---: |
| Requested, in frozen order | 10,728 |
| Raw baseline generation completed | 9,379 |
| `SkipUnsupportedCompilerOptions` skipped after compilation | 1,315 |
| Harness option parsing failed before checking | 34 |
| Full pre/post diagnostic payloads compared | 10,694 |
| Pre/post differences | 0 |
| Completed rows with a different loaded-file closure | 1 |

The 34 failures exactly match the frozen `option_outcome=rejected` variants.
Thirteen use removed `module: none`; the other 21 involve other rejected options.
All 34 are in the native runner's filename skip list. Ten additional variants
in that list complete when called directly. The observer records this metadata;
it does **not** claim those ten are exercised by the normal test suite.

The 1,315 skips are an independent policy in
[`SkipUnsupportedCompilerOptions`](../upstream/tsc/internal/testutil/harnessutil/harnessutil.go),
covering options such as ES5, System/UMD and older module resolution. Compilation
and diagnostic collection have already run before this guard. This demonstrates
that the guard prevents baseline collection, not that checker results are absent
or correct. Their type/symbol baselines have not yet been collected.

Among completed rows, types and symbols each have 9,167 content outcomes,
208 disabled outcomes and four `NoContent` outcomes. Errors have 5,290 content
and 4,089 `NoContent` outcomes. The walker recorded 402,402 `GetTypeAtLocation`,
417,960 `GetSymbolAtLocation`, 389,784 `TypeToTypeNode` and 228,363
`SymbolToStringEx` calls: 1,438,509 calls in total. These counts cover completed
rows only; they are not the eventual complete query denominator.

## The duplicate-file discrepancy is an extractor defect

`compiler/augmentExportEquals2.ts#configuration=0` contains two consecutive
`@filename: file3.ts` directives. The first creates an empty unit; the second
contains 79 bytes with imports of `file1.ts` and `file2.ts`.

The native runner chooses the populated last unit as the root.
[`CompileFilesEx`](../upstream/tsc/internal/testutil/harnessutil/harnessutil.go)
then inserts roots into its virtual FS **before** auxiliary files. The empty
auxiliary duplicate overwrites the root. Its program loads an empty `file3.ts`
and the library closure, makes no walker queries, and emits no errors.

The earlier extractor in
[`scripts/s06_oracle/export_test.go`](../scripts/s06_oracle/export_test.go)
inserts units in source order. The populated duplicate wins there. The frozen
S07 closure therefore contains the populated file plus both imported source
files. Identical raw source and options are not sufficient to establish identical
program inputs. A separate one-case Go process reproduces the discrepancy,
excluding cross-case cache contamination. This is unrelated to declaration emit.

## Proposed decision

Keep all **10,728 variant IDs**, full diagnostic obligations, and existing gates.
Approve the following authority contract before freezing P0:

1. **Correct the duplicate-file extraction to native input/auxiliary ordering.**
   Regenerate the affected S06/S07 observations and S08 joins, inspect the complete
   diff, and rerun their required correctness producers. Keep the variant in the
   denominator. Do not add a divergence entry to preserve the extractor defect.
   Any affected historical evidence must become stale normally.
2. **Use the pinned compiler APIs and original raw baseline writers for the
   frozen effective programs, with native-runner skip metadata retained.** For
   the 1,315 option-policy skips and ten executable filename skips, collect
   actual results beyond the test-selection guards, without changing compiler
   options or algorithms. Compare them against Rust as ordinary required cases;
   a panic, failure or missing result remains a pending failure.
3. **Keep rejected-option observations separate from semantic execution for the
   34 projected variants.** S07 already records exact Go option rejection and
   effective options after rejected settings are omitted. Execute the semantic
   oracle on those explicit effective options, and retain/compare the original
   option rejection separately. Do not call these native-runner baselines, drop
   their queries, infer `NoContent`, or silently accept a failed harness run.
   This extends the documented S07 projection into S08 and needs explicit review.

This chooses a compiler-API oracle over test-runner eligibility while preserving
raw Go baseline algorithms. If native-runner execution must instead remain the
sole authority, the 1,349 skipped/failed variants have no completed native raw
baseline here; they must remain named pending outcomes until a different
owner-approved scope or source decision is recorded. Removing them or declaring
an empty baseline is not an implementation fix.

## Evidence and review checks

[`observed-inventory.json`](../tools/s08/results/baseline-review/observed-inventory.json)
lists every skipped/failed variant, all filename skips, exact file-closure
mismatches, counts and provenance. The adjacent archive retains the complete
ordered requests, 376 MiB of uncompressed baseline/query observations, source
overlays, source snapshots, commands and logs, including the first smoke run.
The archive README describes integrity checks and reproduction.

Validation: tracker tests and all 335 Python tests pass, including 11 observer
contract tests for input/output tampering, partial inventories, skipped-row
diagnostics, malformed queries, empty content and false success. The final native
smoke preserves its expected option failure and reproduces the preceding smoke's
observations byte for byte. Archive members are decompressed and hash-verified.
Ledger validation and committed-view checks accompany this increment. None of
these tooling checks supplies a checker parity metric.

Review the three proposed decisions above and the observer's correspondence to
native constructor/phase/walker order. The observer is deliberately not an E2
producer. It does not yet freeze a complete query schedule, typed source closure,
ownership/relater fixtures or memory/benchmark contracts. P0 and P1 remain open;
the production checker still has only the earlier constructor slice.

After approval: fix and verify input identity first, capture the complete
compiler-API baseline schedule under the reviewed contract, then continue P0/P1
and the first semantic slice. There is no reason to expand checker algorithms
against a silently incomplete or inconsistent oracle in the meantime.
