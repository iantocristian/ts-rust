# Recorded contiguous-chunk list pilot

The 2026-09-08 follow-up rejects **chunk256 and chunk1024 as replacements for
page256 at this checkpoint**. In this synthetic distribution replay, whole
backings stored contiguously did not recover the checked traversal gap to the
legacy control. Both chunk variants added construction time and allocation
relative to the same-batch page256 control. This does not show that chunk
allocation generally fails, identify the exact traversal cost, or select a
production representation.

**Timing limitation:** initial load averages were **27.40 / 27.23 / 29.98** on
the 18-CPU macOS arm64 host (release 25.6.0, 64 GiB memory). The coordinated
capture passed process-name checks after the other builds and diagnostics
finished. Those checks and low MAD do not establish absence of unrelated load
or systematic timing bias. Results remain exploratory. No sample was removed,
replaced, or added, and no third timing batch is part of this checkpoint.

The input is the same sealed census of **13,094 files, 3,114,989 physical
backings and 6,047,867 edge slots**. Each backing receives the synthetic sequence
`index % 17`, including nil words, and is completed in its per-file auxiliary
allocation/completion order. This does not reproduce real node identities,
parser nesting or list-start order. It excludes AST nodes, list headers, imports,
binding, and actual owner retention. The owner-local construction and borrowed
slice contracts are tested separately in `chunks.rs`.

Legacy and page256 were rerun as controls **within this new batch**. These values
are not combined with the [earlier page trial](README-list-results.md). The fixed
schedule retains eight warmups and 56 measured children: one warmup and seven
observations for each of four policies and each of two binary modes. Policy
order alternates forward/reverse; allocation order starts reversed. All 64
children passed exact input hash, count, checksum, storage, and schedule checks.

Times below are median milliseconds; parentheses give relative MAD. Traversal
is the total for **eight complete sweeps**. Allocation figures are decimal MB.
All seven allocation observations were identical within each policy, giving
zero MAD for each of the three allocation metrics.

| Policy | Construction ms (MAD) | Eight-sweep traversal ms (MAD) | Requested MB | Retained MB | Freed/superseded requests MB |
| --- | ---: | ---: | ---: | ---: | ---: |
| Legacy control | 48.889 (0.78%) | 95.462 (0.59%) | 310.418 | 119.564 | 190.854 |
| Page256 control | 33.683 (1.40%) | 128.679 (0.69%) | 109.476 | 72.780 | 36.696 |
| Chunk256 | 38.184 (0.36%) | 129.338 (1.08%) | 145.703 | 91.293 | 54.410 |
| Chunk1024 | 41.015 (0.94%) | 129.429 (0.40%) | 177.002 | 122.877 | 54.126 |

Against page256, chunk256 has **33.1% more requested bytes and 25.4% more retained
bytes**, with a 13.4% higher construction median. Chunk1024 has 61.7% more
requests, 68.8% more retained bytes and a 21.8% higher construction median. Their
traversal medians differ from page256 by only about 0.5–0.6%; that difference
does not establish a timing distinction on this host. All three checked compact
variants remain around 129 ms, versus about 95 ms for the legacy control.

All observations remain in the archive, including chunk256 construction at
42.136 ms and traversal at 134.381 ms, and legacy traversal at 101.091 ms.
The next bounded diagnostic should separate repeated owner/backing validation
and lookup from traversal through an already validated owner-scoped accessor.
This experiment has not measured that alternative, so these results do not
assign the gap to validation or authorize bypassing its ownership checks.

Chunk storage uses next-fit carving from the last chunk only. A backing larger
than the policy width gets an exact-size chunk; abandoned tails are never
revisited. Empty backings receive distinct descriptors without allocating edge
storage. The caller receives a checked contiguous borrowed slice per backing;
the page control performs a checked logical range visit across physical page
spans. No unsafe cross-allocation slice is constructed. The legacy control
traverses its retained boxed slices. Every checksum sweep black-boxes its input
root to prevent repeated pure-call elimination.

| Policy | Edge allocations | Spare edge words | Retained scratch words |
| --- | ---: | ---: | ---: |
| Page256 | 31,356 pages | 1,979,269 | 446,764 |
| Chunk256 | 31,443 chunks | 2,022,625 | 446,764 |
| Chunk1024 | 15,654 chunks | 9,989,656 | 446,764 |

Python independently reconstructs exact chunk counts and spare words from the
sealed lengths, including zero lengths, oversized backings and per-file reset.
The allocation-only binary passed its exact **1,200,050-byte** request/growth
preflight, with live bytes returning from 822 to 822. Every allocation child also verified
that traversal allocated nothing and dropping the retained storage returned to
its starting allocation endpoint. Native request totals include original
allocation and reallocation requests; retained endpoints include scratch,
frames, backing descriptors, edge chunks/pages and their directories.

Construction timing excludes input parsing, checksum setup, and root disposal;
traversal is timed separately. No full parser/binder CPU, whole-owner allocation,
RSS, foreign import, lifetime-retention cost, or memory gate is measured here.
Synthetic checksum sweeps are not a weighting of actual AST access patterns.

The [separate durable archive](results/2026-09-08-chunks/manifest.json) preserves
original manifests, all raw stdout/stderr, build/preflight logs and frozen
Rust/helper sources. It references the verified owner-census gzip rather than
duplicating its 44,899,778 raw input bytes. Native binaries are omitted. Offline
replay verifies recorded work and provenance without a native child, the
original workload files, or a `target/` directory.

```sh
python3 tools/s07/performance-experiments/storage-pilot/replay_chunks.py replay
python3 -m unittest discover -s tools/s07/performance-experiments/storage-pilot -p 'test_replay_*.py' -v
```

Recorded build manifest SHA:
`1e1f3cfcf0c53967f5959a5c04bd6127592abdac2f26bc7450c314632102d007`.
Recorded capture manifest SHA:
`835a97fa3d176383388c3d67717c5a7bfaf3779bc3b8c8cd42753d5713397f4c`.
`run_chunks.py` adapts a separate instance of the frozen list runner; its policy
set and exact chunk validator do not alter the first trial. The new archive
adapter similarly preserves the original list archive and replay helpers.
