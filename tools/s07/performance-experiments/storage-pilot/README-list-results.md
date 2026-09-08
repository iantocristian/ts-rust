# Recorded list-distribution pilot

This diagnostic replays every physical node-backing length in the sealed owner
census, in its per-file auxiliary allocation/completion order. Each backing gets
the same synthetic sequence `index % 17`, including nil words. It compares the
legacy grow-Vec/box path with owner-local edge pages of 64, 256 and 1024 words.
This is **not parser or binder CPU**, and it does not recreate nested list starts,
actual edge identities, list headers, imports or AST ownership. The nested LIFO
construction contract is tested separately in `lists.rs`.

The 2026-09-08 capture replays **13,094 files, 3,114,989 physical backings and
6,047,867 edge slots**. All 64 children passed the exact count, loaded-byte hash,
checksum, storage and schedule checks. Eight children are retained warmups;
the other 56 provide seven normal and seven allocation observations per policy.
The allocation preflight requested exactly **1,200,050 bytes** and returned
live bytes from 824 to 824. Every allocation child also balanced its storage drop.

**Timing limitation:** initial load averages were **27.79 / 30.06 / 29.03** on
the 18-CPU macOS arm64 host (release 25.6.0, 64 GiB memory). The coordinated
capture passed the process-name checks, but those checks and low MAD do not
establish absence of unrelated load or systematic timing bias. These results
remain exploratory; no additional batch or filtered samples replaced them.

The following are medians. Times are milliseconds; memory is decimal MB.
Parentheses contain relative MAD for timing. All allocation samples were
identical within each policy, so their relative MAD is zero. Traversal is the
total for **eight complete sweeps**, not a single sweep.

| Policy | Construction ms (MAD) | Eight-sweep traversal ms (MAD) | Requested MB | Retained MB | Freed/superseded requests MB |
| --- | ---: | ---: | ---: | ---: | ---: |
| Legacy | 49.768 (1.48%) | 94.160 (0.59%) | 310.418 | 119.564 | 190.854 |
| Page 64 | 33.129 (0.47%) | 129.895 (0.85%) | 104.672 | 67.298 | 37.374 |
| Page 256 | 32.894 (1.10%) | 127.652 (0.79%) | 109.476 | 72.780 | 36.696 |
| Page 1024 | 35.659 (0.66%) | 129.840 (0.27%) | 141.286 | 104.684 | 36.602 |

Paged variants reduce construction/request traffic in this replay, while their
recorded traversal is approximately **35–38% slower**. This supports investigating
the allocation strategy, but does not select a production list representation.
The relative number and shape of real AST traversals are absent here. The small
construction difference between 64- and 256-word pages does not establish a
robust CPU winner. All samples remain recorded, including the 40.152 ms page-64
construction observation and 138.249 ms page-256 traversal observation.

All page variants retained **446,764 scratch words (1.787 MB)**. Their spare edge
words were 439,301 / 1,979,269 / 9,978,757 respectively; the 1024-word policy's
39.915 MB of spare edge storage is visible in its larger retained endpoint.
There were no wide backing starts in this input, so their runtime cost remains
unmeasured despite separate boundary tests.

The complete [durable archive](results/2026-09-08/manifest.json) contains original
capture/build manifests, raw stdout/stderr, compiler/preflight logs and source/
helper snapshots. It references the already verified owner-census gzip instead
of duplicating its 44,899,778 raw input bytes. Native binaries are omitted;
offline replay verifies the recorded observations and provenance without
launching a native child or requiring a `target/` directory.

```sh
python3 tools/s07/performance-experiments/storage-pilot/replay_lists.py replay
python3 -m unittest discover -s tools/s07/performance-experiments/storage-pilot -p test_replay_lists.py -v
```

Recorded build manifest SHA:
`14805a05dfe79ff38ea32d10aba0a2030119e60ff3f9fdadb9c57c329ed09dfc`.
Recorded capture manifest SHA:
`d9ba830ed1cea663e78988f3ddc28c69027e6c969c1508417d0ab1a64d705eda`.
The earlier build without the final per-sweep optimization guard was superseded
before capture and supplied no timing samples.

`run_lists.py` freezes the standalone Cargo sources, lockfile, measurement
helpers and two actual Cargo-reported release executables. Native environment
and release configuration come from the existing S07 benchmark helpers. Compiler
overrides are rejected; caller registry/cache configuration is preserved. The
allocation binary must pass its exact 1,200,050-byte request/reallocation/drop
preflight before any allocation observation is accepted. Every build log and
preflight stdout/stderr is retained. Binaries are copied immediately and sealed,
so building the other mode cannot overwrite the measured executable.

Capture requires the externally recorded build and census manifest hashes.
The runner validates the census inventory and owner records, then freezes the
exact NDJSON bytes. Children independently report the loaded-byte digest,
file/backing/edge counts and edge-order checksum. Python computes the checksum
using modular affine composition, independently of the child's edge loop.
It also checks page/slack/overflow counts and scratch-capacity bounds.

The fixed schedule runs **one warmup and seven measured children per policy and
binary mode**: eight warmups and 56 recorded children in total. Policy order
alternates forward/reverse; the allocation order starts reversed. Normal and
allocation metrics cannot substitute for each other. Warmups, failures and all
stdout/stderr remain in compressed raw files. No sample extension or selection
is supported. Verification reconstructs the exact schedule and summaries from
the raw bytes, and rejects changed commands, counts, hashes or metric modes.

Both building and capturing hold the existing S07 measurement lock. Full
capture must wait until other builds, tests and profiles stop; it also rejects
visible compiler/diagnostic children before each run. These checks supplement
the coordinated quiet window. They do not prove the absence of unrelated load.

```sh
python3 -m unittest discover -s tools/s07/performance-experiments/storage-pilot -p test_run_lists.py -v
python3 tools/s07/performance-experiments/storage-pilot/run_lists.py build --output target/s07-bis/list-pilot-build
python3 tools/s07/performance-experiments/storage-pilot/run_lists.py capture \
  --build target/s07-bis/list-pilot-build --build-sha BUILD_MANIFEST_SHA \
  --census target/s07-bis/owner-census-capture --census-sha CENSUS_MANIFEST_SHA \
  --output target/s07-bis/list-pilot-capture
python3 tools/s07/performance-experiments/storage-pilot/run_lists.py verify \
  --build target/s07-bis/list-pilot-build --build-sha BUILD_MANIFEST_SHA \
  --output target/s07-bis/list-pilot-capture --capture-sha CAPTURE_MANIFEST_SHA
```

Construction timing excludes input
parsing/checksum setup and root disposal; traversal timing covers the declared
number of complete sweeps. Allocation observes original request bytes and the
retained endpoint immediately after construction, including retained scratch.
Neither the internal timings nor these bytes establish whole-owner RSS, parser
improvement, final allocation gates, or a selected production layout.
