# S07-bis: bounded current name/table attribution

Status: single diagnostic complete; name/hash-backing tuning is not selected
as the next gate-closing change. No storage change or promotion is made. Use implementation `56e2127`, whose
measured source is the corrected `8f7236e` freeze. CP1 remains the control.

The combined candidate requests 2.244 GB and peaks at 2.322 GB RSS. Historical
limits still require approximately 209 MB less allocation and 105–110 MB less
RSS. Its one-worker CPU nonregression is unresolved. The earlier one-run
allocation diagnostic found 333.733 MB freed/superseded requests; that total is
not automatically avoidable. Existing row accounting is insufficient to close
the request gap and does not justify another page-policy matrix.

## Decision this measurement resolves

Measure the current canonical-name pool and symbol-table storage once, including
actual growth. They share names between symbols and table keys; charge them once.
Do not reconstruct owned syntax nodes or port the obsolete owner census. This
family has several independent growable structures and is not covered by the
current typed-row accounting. Its measured waste must establish relevance before
another representation or reservation change is considered.

Capture only:

- Name bytes and range entries: lengths, capacities and requested growth.
- Canonical-name hash and compact/full table maps: entry/bucket counts, safe
  `HashTable::allocation_size()` bytes and allocation-size transitions.
- Table cardinality histogram (`0`, `1`, `2`, `3–4`, `5–8`, `9–16`, `17+`),
  owner/table counts, variant conversions, removals and wide-name escapes.
- Explicit name-to-owned calls and selected bytes. These counts are not Arc
  allocation requests; actual backing costs remain unassigned unless measured.

Live entries, required name bytes and canonical identity support are not
removable overhead. Report retained allocation, logical element bytes and growth
separately. Include full new allocation requests on successful reallocation,
consistent with the pinned counter; same-capacity hash-table rehash may allocate
nothing. Inspect the pinned implementation and calibrate initial growth,
replacement, removal and compact-to-full conversion on a tiny fixture before
assigning request totals. Table-record arenas and owner roots are separate from
hash backings. Wide structures need explicit counts and unassigned costs if
observed; do not guess their capacity or treat them as zero implicitly.

## Implementation boundary and checks

Use a disposable copy of the exact frozen source, with a diagnostic-only patch
to `symbol_tables.rs` and a small existing benchmark-driver hook. Fixed scalar
counters and histograms must not allocate per event or retain names/nodes. Read
retained tables while every completed file remains alive; serialize only after
sampling. Growth counters do not change the production storage policy, binder
lifecycle, work or source ranges. Keep new diagnostic/test code near 200–250
handwritten lines in this storage family, plus the reused driver/build adapter.
If it requires a broader profiler, stop and document the missing counts.

Build through fresh final and intermediate Cargo directories, preserve the
unmodified manifests/lockfile and exact workspace closure, and retain the patch,
source inventory, compiler profile, executable hash and build/check logs. Run
the tiny allocation-counter fixture and affected symbol-table tests. Then run
exactly one full 13,094-file, one-worker allocation-enabled diagnostic, verifying
all counts, loaded digest and exclusive/fallback binding counts. No CPU timing
claim, eight-worker run, access trace, page matrix or repeated workload capture.
Record observer costs separately; this cannot amend the existing timing screen.

## Exit

Report absolute retained/requested/growth totals and clearly conditional
ideal-capacity bounds. If measured waste is far below the remaining gaps, end
this family's gate-closing investigation. If it is material, choose one concrete
storage or lifecycle change with its compatibility obligations, then screen the
complete candidate. Removing a construction-only index would not erase its
already-issued requests, and rebuilding it for later mutation could add costs.
No implementation saving is assumed in advance.

A bounded independent decision audit selected this family and named these
limits. Review the concrete patch and calibration before the single workload
capture. The shared-text screen, all failed attempts and final gate definitions
remain unchanged.

## Measured result (2026-09-09)

One full allocation-enabled invocation from the corrected `8f7236e` freeze
retains all 13,094 files, 19,593,488 nodes and 2,459,867 symbols, with the expected
161,740,237 input bytes, 423 parse diagnostics and 5,250 bind diagnostics.
All files bind in place; fallback count is zero. The loaded SHA-256 is
`d4ff2ad3b0d590babb596522a7105d961dbb3c4f2ebc987488d88b7ebd8e0c88`.
Five tiny calibration phases exactly reconcile the allocator's total/live deltas
with recorded backing requests/releases. The invalid-worker check, seven
symbol-table tests and allocation-enabled Clippy with denied warnings pass.

| Backing family | Requested MB | Retained MB | Replaced MB | Logical elements MB |
| --- | ---: | ---: | ---: | ---: |
| Canonical name bytes | 37.837 | 19.398 | 18.439 | 13.529 |
| Name ranges | 22.190 | 11.304 | 10.886 | 7.879 |
| Canonical-name hash | 28.920 | 14.537 | 14.384 | 7.879 |
| Compact symbol-table hashes | 54.371 | 40.604 | 13.767 | 15.907 |
| Full symbol-table hashes | 0 | 0 | 0 | 0 |
| **Total** | **143.318** | **85.842** | **57.476** | **45.194** |

Decimal MB are rounded for display; the archive keeps exact bytes. These five
families issue 1,022,000 allocation events, including 683,459 first allocations.
Their 143,318,442 requested bytes are 6.386% of the pipeline's 2,244,181,033 bytes.
For every family, requests minus measured replacements exactly equals retained
backing; there are no unassigned backing releases in this capture. The pipeline
separately comprises 1,910,448,509 bytes of net live growth and 333,732,524 bytes
freed or superseded. These are allocator-request observations, not RSS.

The 984,900 canonical names use 13,529,162 selected bytes. All 644,333 tables
use compact entries, with 1,988,329 total entries. Their cardinality histogram is:

| Entries | 0 | 1 | 2 | 3–4 | 5–8 | 9–16 | 17+ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Tables | 0 | 267,731 | 161,640 | 128,561 | 57,839 | 17,999 | 10,563 |

There are no wide name ranges, full tables, variant conversions or successful
removals in this workload. Calibration and existing tests still exercise those
compatibility paths. The explicit name-to-owned API is called 7,904 times for
39,463 selected bytes; this does not assign its Arc backing allocation. Table
arena rows contain 25,773,320 logical bytes and owner roots 2,095,040 logical
bytes, reported separately from backing storage. Their arena capacity, directory
cost and allocation placement remain unassigned; these logical counts are not
additional measured physical allocations.

## Decision and bounds

Eliminating every observed replacement request, while preserving today's final
backings, has a conditional ceiling of **57.476 MB** fewer requests and no
retained-backing saving. Perfect final capacity could remove at most another
**40.648 MB** above logical elements. For hashes that deliberately omits required
control bytes and spare buckets, so the combined **98.124 MB** request ceiling
is unattainable with the current representation. It is still less than half the
roughly 209 MB historical allocation deficit. Retained requested bytes cannot
be subtracted from a separate process's peak RSS to promise its gate closure.

Do not start a canonical-name/hash-backing rewrite or reservation matrix on
these results. This ends their investigation as the next primary gate-closing
change, without claiming every table-related cost was measured or forbidding
a smaller component in a later justified combination. A small-table design may
save some of the 24.698 MB compact-hash backing excess, but its new representation,
lookup/mutation behavior and CPU tradeoff have no measured combined result here.
Do not assume that saving or allocate another tuning campaign to it now.

The complete compact combination remains experimental; CP1 remains the retained
control. Neither the shared-text screen nor final Go-relative gates are changed.
This diagnostic's fixed counters affect CPU work; its elapsed value has no
performance-acceptance interpretation. No second full workload capture occurred.

## Reproduction and retained failures

Build manifest:
`295328ae4a0154ac73402d66da0f2fc0b0dda60d074ce844be5bf8c2de71ee77`.
Capture manifest:
`50297ef478619b9b00eb12b3dc185a1d5b983476cab12a5dd414f98b691a5866`.
Allocation executable:
`6d7d2a295e07c8619dcc43ef705073340d948875a2858d6664d5fbc4c0019a1f`.

The first isolated build passed calibration but could not compile the library
test target because the frozen benchmark closure omitted four compile-time test
fixtures. That failed source, adapter, executables and logs are retained. The
corrected build restores exactly those four fixture blobs and missing workspace
members from `8f7236e`, recording them separately and preserving all manifests
and the lockfile. Fresh final and intermediate Cargo directories isolate both
builds. Independent review checks the source recovery, calibration, inventories
and arithmetic; no failed attempt is presented as an accepted build or capture.

The [diagnostic archive](../tools/s07/performance-experiments/results/2026-09-09-name-table-attribution/README.md)
contains the exact patched source, binaries, adapters, pinned dependency audit,
failed attempt, build/check logs and raw observations. Run its retained
`target/s07-bis/name-table-summarize.py` to recompute all totals without launching
a workload. Inputs and toolchains remain external, as in prior captures.
