# S07-bis: preserve shared identifier text in local binding

Status: accepted as the next experimental control after source review,
correctness/ownership validation and the complete fixed screen. Final S07
CPU and memory gates remain open.
This follows the [completed local binder screen](S07-bis-local-bind-completion.md).
That version remains recorded without promotion: its full binder producer passed,
but its E3 capture rejected stale test inventories. This corrected combination
was measured against CP1; its immutable freeze now supplies the next control.

## Mechanism and scope

The local Identifier branch of `Binder::target_text` converted borrowed bytes
with `JsString::from_bytes`. Unlike the checked text operation, this allocated a
new backing. Ordinary declaration-name processing can then discard that copy
after interning or table lookup. This is a confirmed extra allocation mechanism,
not an attribution of the entire approximately 58 MB request increase observed
between independently measured compact candidates.

Generate explicit owned accessors beside the borrowed text accessors. They pass
the same field key, encoded word and node end through `BindContext` and
`CoreStore` to the existing `TextPool::owned`. Raw identifiers share a source
slice; exceptional and extended-pool values clone their existing backing. The
binder uses this accessor only after its existing Identifier kind and shape
checks. Other kinds, malformed payloads and lazy nodes retain their checked
operation. No new interning, table, storage or lifetime policy is introduced.

Tests cover source and pooled pointer identity, bytes and validity, survival
after owner drop, lazy fallback, exact mismatch panic messages, extended-pool
entries and generated-name collisions in either schema-field order. The real
generator supplies the output and its manifest hash.

## Ownership inventory correction

The failed capture executed passing tests but could not prove its declared
denominators. S06 now names all 29 cases in `storage_tests`. S07 separately names
local AST, local core state, binder, flow and symbol cases, alongside publication,
exclusive binding, completion proof and program ownership: 76 cases in total.

Cargo's substring filter `bind_tests::` also selected `local_bind_tests::`.
The publication suite now declares that exact module skip; local cases execute
in their own group. The manifest schema validates the permitted skip and rejects
skipping a declared case. Missing, extra, duplicate or failed outcomes still
invalidate the result. The expanded groups must pass in every required mode.
This correction does not turn the earlier failed evidence into passing evidence.

The 39 Python checks pass. One native `--no-run` build and exact test listings
match all 11 selections (29 S06 / 76 S07). Listing is not test execution. The
initial listing wrapper's overly narrow artifact-path assertion is retained;
the corrected verifier consumed the same successful Cargo artifacts without
rebuilding or executing suites.

## Validation and measurement sequence

1. Finish affected native and release suites, documentation tests, generator
   tests and drift check, Clippy, minimum Rust and formatting. Preserve source
   inventories and failures. Renew E3 with the corrected exact inventories.
2. Freeze new normal and allocation executables from the repaired source. Keep
   the previous completion freeze unchanged. Compare full Rust binding graphs
   at both worker counts with the immutable, verified Go streams; record reused
   Go provenance explicitly. Confirm local path selection on the new executable.
3. Run one complete fixed screen against CP1: eight warmups and 56 samples.
   Judge the combination on CPU, requested allocation and RSS at both worker
   counts. Per-component estimates cannot qualify it. Recompute historical Go
   distances without presenting them as fresh gate evidence.
4. Archive and replay the new observations, distinguish validation failure from
   measured regression, and record whether the combined candidate can become
   the next control. S07-bis gates remain open until their own evidence passes.

No new native profile or field-level trace is required for this repair. The
preceding completion's single phase observation remains bound to that version;
it is not a phase measurement of this revision.

## Completed checks

Affected debug and release checks pass: 30 arena tests, 158 AST unit tests,
29 AST integration tests, 47 binder tests, 64 generator/tooling tests and
37 documentation tests in each build mode. Clippy with warnings denied,
Rust 1.96 checks, all 22 generated outputs and pinned client bytes, `xtask
validate`, and formatting pass. Source inventories are unchanged across the
final checks. Initial missing-import, test-source-filename and formatting
failures remain recorded separately from that successful final run.

An independent read-only review found no actionable defect in ownership,
validity, checked fallback errors, generated field keys or name collisions.
It checked the actual generated output against its manifest. The patch author's
separate follow-up review is recorded as self-review, not independent evidence.

The fresh E3 producer passes all seven scenario rows, all 29 S06 cases and all
76 S07 cases in debug, release, Miri and ASan. Every measured ownership boolean
is true, and both live-owner and live-allocation deltas are zero. Its immutable
record is `162878622bd056fea3130d3fd4e7c50c40c6f174df424df0f02796ccf7803882`.
The complete capture, prior failed evidence and before/after source inventories
are retained. The 529.568-second capture duration is diagnostic, not a benchmark.

## Frozen revision

Production revision: `6de761290d20ba58b05d5b725a5d1d9066412a86`.
The freeze covers 288 unchanged source/configuration/protocol files, fingerprint
`4ab8e03c9d8b1b591da5d0ee088e0a8e4b0f4408ad95352b9d7a472b79767cd0`.
Manifest: `99d11d1f2efd383919663bece6459a0230bc0fe815723ba65b7178e5973c5abc`.
Normal executable: `9e0992cf79e861ecaa6080bcecaa6bc85691a7679dc214497747f04611eb50a2`.
Allocation executable: `1e8d4d986feac6c604603701067aa196198cd276632b36057cf34521b31f47f3`.
Both executables were selected from fresh, isolated Cargo builds; the allocation
accounting preflight passes. The Go executable and workload remain unchanged.

## Complete result and decision

Both 13,094-file graph comparisons pass. At one worker every raw row is exact;
at eight workers 13,057 are exact and 37 differ only under the existing validated
scheduling/name-counter rules. Counts remain 19,593,488 nodes, 2,459,867 symbols,
423 parse diagnostics and 5,250 bind diagnostics. Both local-path probes observe
13,094 local/in-place files and zero checked/fallback files. The Go streams are
the byte-identical archived v1 outputs; no new Go process was executed. The new
Rust streams and local-path probes use this freeze. Offline graph replay passes.

The fixed screen contains all eight warmups and 56 recorded executions. The
standard verifier replays every raw sample and summary. No sample was replaced,
extended or dropped. Screen SHA:
`8a5a851ad6b3d343f3c1a2642fc164041d70ac045c0dd0657fb9de4d1a24d676`.

| Metric | Workers | Same-screen CP1 | Complete candidate | Ratio |
| --- | ---: | ---: | ---: | ---: |
| Wall | 1 | 4.408072416 s | 3.765817209 s | 0.854300 |
| Wall | 8 | 0.991912250 s | 0.814478917 s | 0.821120 |
| Requested bytes | 1 | 4,642,569,187 | 2,243,575,049 | 0.483262 |
| Requested bytes | 8 | 4,642,570,542 | 2,243,578,084 | 0.483262 |
| Peak RSS bytes | 1 | 4,506,664,960 | 2,318,663,680 | 0.514497 |
| Peak RSS bytes | 8 | 4,509,237,248 | 2,321,956,864 | 0.514933 |

CPU upper 95% ratio bounds are 0.855997 and 0.831178. Timing relative MAD is
below 0.8% for both variants/modes; allocation and RSS variation also satisfy
the fixed screening limits. The complete combination saves 642.255 ms and
177.433 ms against its paired control, while reducing both memory measures.
These are whole-candidate results, not an isolated local-view or text-repair win.

Relative to the recorded v1 candidate, requests fall by 58.495 / 58.493 MB,
consistent with the removed text-copy mechanism. There is no isolated
paired CPU claim for that repair. The older traffic attribution remains bound
to its own binary; the reduction does not reassign its residual by assumption.
Wider continuation/adapter traffic remains included in the complete screen;
peak continuation capacity has not been separately measured.

| Distance to historical Go medians | Workers 1 / 8 |
| --- | --- |
| CPU ratio, gate 1.0 | 1.281925 / 1.284742 |
| CPU time still above historical budget | 828.190 / 180.516 ms |
| Allocation ratio, gate 0.7 | 0.771660 / 0.771456 |
| Requests still above historical budget | 208.349 / 207.811 MB |
| RSS ratio, gate 0.7 | 0.734768 / 0.733124 |
| RSS still above historical budget | 109.715 / 104.912 MB |

These distances use the frozen original Go report, not a fresh Go-relative
sample or confidence bound. They do not pass a final S07 gate.

Accept this immutable complete candidate as the next experimental control under
the normal substantial-candidate rule. Both CPU modes establish a win, all
memory/noise conditions pass, and the removed ownership-copy operation and
scoped access contracts have source review and relevant executed coverage.
Checked compatibility has the unchanged v1 full binder corpus, the new paired
checked/local fixtures and full corrected ownership coverage; no fresh v2 full
binder-producer result is claimed. Preserve CP1, v1 and earlier rejected screens.

The [review archive](../tools/s07/performance-experiments/results/2026-09-10-local-text-owned-repair/README.md)
retains source, executables, raw results, reused-Go provenance, failures and
offline replay. Final acceptance still needs fresh Go-relative evidence after
the remaining CPU and memory gaps are addressed.

Six implemented binder functions lost `// port:` markers when moved or
consolidated. The generated worklist reports them as unmapped, not unimplemented.
Their reviewed six-comment repair is retained unapplied with source/Go references;
restore it with the next substantive source revision before final S07 acceptance.
No source edit was made during this freeze or its capture to conceal that count.
