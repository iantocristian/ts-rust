# S07-bis: current allocation traffic

Status: one calibrated full-workload diagnostic complete. No memory implementation
or performance promotion is made. The current compact candidate remains
experimental, CP1 remains the accepted control, and all final Go-relative gates
are unchanged.

The diagnostic attributes **225.443 MB of the 333.733 MB freed or superseded
requests**. Typed-row directory growth and text-pool entry growth are the largest
newly observed mechanisms. Neither family establishes a change capable of closing
the approximately 209 MB allocation deficit. Another 108.289 MB of traffic remains
explicitly unclassified. This result does not establish that memory is one change
away from passing; RSS is a separate, unmeasured endpoint here.

## Accounting result

One invocation used a disposable instrumented copy of the corrected `8f7236e`
source freeze. All 13,094 files, 161,740,237 source bytes, 19,593,488 nodes,
2,459,867 symbols, 423 parse diagnostics and 5,250 bind diagnostics match.
Every file binds in place; fallback count is zero. The loaded-input SHA-256 is
`d4ff2ad3b0d590babb596522a7105d961dbb3c4f2ebc987488d88b7ebd8e0c88`.

The allocator records the complete requested replacement layout on growth,
including reallocations. “Replaced” below is the old requested capacity
superseded by that operation, not the number of bytes physically copied.
“Released” is the requested backing freed without replacement. The family
retained value is requests minus replaced and released layouts, observed while
all completed files remain alive. The pipeline's net live growth is the
increment from its preloaded start, not total process live memory or RSS.

| Accounting partition | Requested bytes | Net live growth / retained backing bytes | Freed or superseded bytes |
| --- | ---: | ---: | ---: |
| Complete pipeline | 2,244,181,769 | 1,910,449,245 | 333,732,524 |
| Observed backing families | 1,835,651,882 | 1,610,208,809 | 225,443,073 |
| Explicit residual | 408,529,887 | 300,240,436 | 108,289,451 |

Observed traffic is 207,747,201 replaced bytes plus 17,695,872 released bytes.
The five name/table families reproduce the preceding diagnostic's 143,318,442
requested and 57,476,017 replaced bytes exactly. The complete pipeline's request
and net-live totals are each 736 bytes above that earlier diagnostic; its
333,732,524 freed/superseded bytes are unchanged. The small retained difference
is recorded without assigning it to an assumed cause.

## Families and phases

Decimal MB below are rounded; the archive preserves every exact counter.
Rows are disjoint allocation backings. In particular, table-record arena pages
and symbol-table hash buckets are separate allocations.

| Backing family | Requested MB | Retained MB | Replaced MB | Released MB |
| --- | ---: | ---: | ---: | ---: |
| Core header pages | 503.243 | 503.243 | 0.000 | 0.000 |
| Core header directories | 15.122 | 8.608 | 6.513 | 0.000 |
| Auxiliary slot pages | 62.325 | 62.325 | 0.000 | 0.000 |
| Auxiliary slot directories | 8.621 | 5.358 | 3.263 | 0.000 |
| Symbol pages | 176.888 | 176.888 | 0.000 | 0.000 |
| Symbol directories | 5.942 | 4.015 | 1.928 | 0.000 |
| Flow pages | 72.764 | 72.764 | 0.000 | 0.000 |
| Flow directories | 5.985 | 4.040 | 1.945 | 0.000 |
| Flow-list pages | 13.394 | 13.394 | 0.000 | 0.000 |
| Flow-list directories | 3.754 | 2.660 | 1.094 | 0.000 |
| Other arena pages | 85.982 | 85.982 | 0.000 | 0.000 |
| Other arena directories | 9.691 | 6.931 | 2.760 | 0.000 |
| Typed-row pages | 394.298 | 394.298 | 0.000 | 0.000 |
| Typed-row directories | 135.496 | 70.068 | 65.429 | 0.000 |
| Edge pages | 51.006 | 51.006 | 0.000 | 0.000 |
| Edge directories | 1.081 | 0.958 | 0.123 | 0.000 |
| Text-pool entries | 121.569 | 61.620 | 59.949 | 0.000 |
| Text-pool free slots | 0.208 | 0.208 | 0.000 | 0.000 |
| Eager parser list buffers | 24.962 | 0.000 | 7.266 | 17.696 |
| Canonical name bytes | 37.837 | 19.398 | 18.439 | 0.000 |
| Canonical name ranges | 22.190 | 11.304 | 10.886 | 0.000 |
| Canonical-name hash | 28.920 | 14.537 | 14.384 | 0.000 |
| Compact symbol-table hashes | 54.371 | 40.604 | 13.767 | 0.000 |
| Full symbol-table hashes | 0.000 | 0.000 | 0.000 | 0.000 |

`other_arena_pages` includes declaration backing headers and table records.
`edge_pages` includes syntax and declaration edges. `payload_row_pages` includes
the generated typed stores reached through the shared row-page implementation.
Text-pool vector backing excludes nested `JsString`/Arc bytes and exceptional
maps. Raw parser vectors, caller-owned append suffixes and default/lazy
Vec-to-Box transfers are not attributed to the eager-buffer family.

The process phase windows independently partition requests and live changes:

| Process window | Requested bytes | Net live growth bytes | Freed or superseded bytes |
| --- | ---: | ---: | ---: |
| Parse | 1,607,937,053 | 1,402,594,484 | 205,342,569 |
| Consuming bind and publication | 636,244,396 | 507,854,441 | 128,389,955 |
| Outside those windows | 320 | 320 | 0 |

Each phase contains exactly 13,094 file windows. These are process-wide cap
snapshots around the sole worker's operations, so concurrent main-thread events
can fall inside a window. Family events instead use worker-local phase tags.
Those tagged families record 1,298,574,944 requested bytes during parse and
393,758,496 during bind/publication; none are unscoped. The reused name/table
observer has no phase tags and contributes its 143,318,442 bytes separately.
Do not subtract it from a phase based only on expectation, or add the process
windows to family totals. Phase net-live changes describe when events happened,
not which owner ultimately retains the backing.

## Implications for the next memory change

Typed-row directories replace **65,428,912 bytes**, and the text-pool entry
vector replaces **59,949,184 bytes**. Eliminating both replacements completely
while preserving final backings would save at most **125,378,096 requested
bytes**, below the approximately 209 MB deficit. Across every directory family,
replacement is **83,056,016 bytes**; adding the text-entry ceiling gives
**143,005,200 bytes**, still insufficient. These are conditional ceilings,
not savings demonstrated by a replacement representation.

Non-recopying directory or text-entry storage and justified reservation are
concrete mechanisms to consider, but their replacement storage, per-owner slack,
access cost and allocation-call count must be charged. Removing growth alone
also leaves final backing unchanged and does not establish an RSS improvement.
The previously investigated name/table traffic contributes another 57.476 MB;
its earlier ideal-capacity bounds and limitations remain applicable.

Eager parser list buffers now request **24,961,856 bytes** in total: 7,265,984
of growth and 17,695,872 of final release. They retain nothing at the endpoint.
For these eager buffers, this current measurement supersedes the much larger
precompact temporary-list estimate; other list-related allocations remain outside
this family. The residual's 108,289,451 traffic
bytes have no proven family assignment and must not be labelled scanner, Arc,
validation or parser traffic by subtraction alone.

Observed families make **9,503,058 allocation calls**, including **6,452,840**
for typed-row pages. This names an actual allocation-call mechanism, but the
instrumented run supplies no CPU attribution or speedup estimate for changing
it. No additional allocation capture, page-policy matrix or timing screen was
run to select a favorable result.

## Calibration and review

The diagnostic changes only a disposable frozen source copy. It reuses the
hash-verified name/table observer and adds fixed counters at actual capacity and
lifetime operations. The auxiliary arena classifier recognizes the actual
`auxiliary::StoredAux` record; calibration checks its **8-byte** layout and its
family assignment, plus the core-header assignment.

The initial observer missed dropped eager buffers when `parse_delimited_list`
returned early. Corrected instrumentation follows the buffer's ownership with
a destructor and a guarded consuming transfer, including unwind. An additional
untracked diagnostic variant keeps default/lazy buffers outside this family's
claims because their boxed lifetime/shrink differs. Calibration verifies that
the frozen `ListBuffer` remains 48 bytes with 8-byte alignment and preserves Vec
capacity and transfer behavior. The observer adds counter and destructor work;
its elapsed time is not suitable for performance acceptance.

Successful checks before the sole workload run were:

- Exact cap reconciliation while rows, edge pages and arena pages are live and
  after they drop; a separate generic Vec fixture verifies shrink accounting.
- Text-entry/free-slot growth, release, reuse without a new request, and final
  backing drop, while shared text bytes remain outside the measured fixture.
- Eager inline/spill/abort/append/consumption and unchanged Vec-to-Option mapping;
  default/lazy boxed transfers remain deliberately unclassified.
- Seven symbol-table tests and nine parser list/factory/abort/unwind tests,
  including the real more-than-four-element aborted-list path.
- Allocation-enabled Clippy with denied warnings and rejection of an invalid
  worker count before input loading.

The pipeline endpoint and family snapshot occur before releasing worker roots;
serialization and the later join/report counters cannot replace those snapshots.
A separate arithmetic pass over the raw flattened counters matches the summary.
Every sealed build/capture member was read back, checked against its hash/size,
and checked for inventory completeness and read-only permissions.

## Reproduction and retained failures

Build manifest:
`fd9bd1e99907cba381bb70ad8ad3f8f3878530477260482c99a48b0c4d09f26a`.
Capture manifest:
`236acf700e24569b562bdc692055d8ffbf8c0ff2212600bea9e135397c073887`.
Allocation executable:
`31908e873ed41e0dde126d4ea26a5cffd54f9c9098a920ca5c00b761de0ee4d1`.

The [archive and member manifest](../tools/s07/performance-experiments/results/2026-09-09-allocation-traffic/README.md)
retain the actual instrumented source, executables, build/calibration/test logs,
raw capture, arithmetic summaries, tools and failed attempts. The original
corrected source freeze is referenced through the existing shared-text archive;
workload files and toolchains remain external. All restored workspace members
and fixtures came from the exact source revision, with unchanged manifests and
lockfile and fresh final/intermediate Cargo directories.

The first build passed its earlier calibration but failed Clippy on a wildcard
match. Review also found the missing early-drop accounting. Its staged source,
patch hashes, executables and logs are retained; the build failed before copying
and sealing its original driver, and that provenance limitation is explicit.
The next build failed on a calibration-only type path (`crate::StoredNode`
instead of `crate::compact::StoredNode`); its exact adapter and failure are also
retained. Neither executed the workload. The successful build passed the expanded
checks above. A sandbox `ps` preflight failure occurred before child launch;
a subsequent approved process-inspection execution performed the only workload
capture. These failures are not presented as accepted captures.

To recompute accounting without running the workload, restore the archive paths
at the repository root and run:

```sh
python3 tools/s07/allocation-traffic/summarize.py \
  236acf700e24569b562bdc692055d8ffbf8c0ff2212600bea9e135397c073887 \
  --output target/s07-bis/allocation-traffic-summary-recomputed.json
```

The summarizer validates the sealed capture and refuses to overwrite an existing
output. Its scope, full per-phase family counters and explicit residuals are
recorded in the machine-readable summary. The first two failed builds and the
preflight failure remain distinct from the successful sealed build/capture.
