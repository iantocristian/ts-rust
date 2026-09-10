# Current compact candidate: elapsed phases and live requests

Diagnostic source: `03f9aa3`, the frozen inline parser-list combination. This
does not change its completed performance screen or promote the implementation.
The [screen record](S07-bis-compact-storage.md) remains the source of paired
CPU/memory results and historical gate distances.

## Method and provenance

A reviewed driver-only patch places elapsed timers around the existing parse
and consuming bind calls. The allocation build also snapshots the existing
`cap` counters immediately before the pipeline and at the retained barrier,
before releasing workers or allocating diagnostic output. The source closure,
Cargo manifests, lockfile, toolchain, features and release profile are preserved.
The frozen source omitted unused workspace members; 95 missing blobs were added
from exact commit `03f9aa372e5fa9c93ee9a16fd618a644fed9311a`, with no manifest
pruning. All 263 original files match the freeze except the reviewed driver.
An earlier rejected metadata-pruning attempt failed under `--locked` before Rust
compilation; its files and failure are retained separately.

- Build manifest: `06b20e77448bef2dbdb149e302029275a42d7609f0a7edc43d7dc587b82b5c9c`.
- Capture manifest: `5c4b0e454df5a435e1d28b5b5e1697303dfb0448b0f8a63226046686fe6a0119`.
- Normal executable: `c9f4e7dd23ecf9a29ee22a9e37c1692e46021550760c213039bd5f0fc1553c75`.
- Allocation executable: `38cc05aee2864f95f695d0801f33900b1a2c1e61e12b180386cb0250b9eaf6eb`.

Both builds and allocation-enabled Clippy pass. A small protocol check compares
the unchanged frozen driver with the diagnostic and checks the worker guard.
Exactly one full one-worker invocation of each diagnostic variant succeeds.
Both retain all 13,094 files, 161,740,237 loaded bytes, 19,593,488 nodes and
2,459,867 symbols, with 423 parse and 5,250 bind diagnostics. The loaded digest
matches `d4ff2ad3b0d590babb596522a7105d961dbb3c4f2ebc987488d88b7ebd8e0c88`.
Independent review verifies source/binary inventories, raw protocol, work totals
and arithmetic. Recorded commit/push times follow the two captures; complete
host quiet was not measured.

## Results

| Normal-build elapsed component | Seconds |
| --- | ---: |
| Parsing, including completion | 2.748578340 |
| Consuming binding, validation and publication | 2.741912765 |
| Other time inside the measured pipeline | 0.018874103 |
| Whole measured pipeline | 5.509365208 |

Both phase groups remain substantial. These are single-run elapsed observations,
with per-file timer overhead; they are not sampled CPU or a fresh paired phase
comparison against CP1 or Go. In particular, subtracting the old CP1/CP4 phase
capture from these values cannot attribute individual changes. The earlier
matched CP1/CP4 result established that both phase groups regressed at that
checkpoint; this capture fills the current split without rewriting that result.

| Allocation-build snapshot or derived delta | Bytes |
| --- | ---: |
| Pre-pipeline cumulative requests | 339,507,751 |
| Retained-endpoint cumulative requests | 2,583,685,824 |
| Pipeline requests | 2,244,178,073 |
| Pre-pipeline live requests | 166,637,709 |
| Retained-endpoint live requests | 2,077,083,258 |
| Pipeline live growth | 1,910,445,549 |
| Pipeline freed or superseded requests | 333,732,524 |

The identity is `2,244,178,073 = 1,910,445,549 + 333,732,524`.
Successful `cap` reallocations add the entire new request, including in-place
or shrinking reallocations; the final term therefore includes superseded
requests and does not measure physical copying. The pinned crate source/checksum
audit records this behavior.

Live growth is the net change in process-wide live requested bytes during the
pipeline, including channels and main-thread work. It is not a complete retained AST census.
Endpoint live requests include the preload/runtime baseline. Neither figure
is current RSS. The normal screen's peak RSS is a different process's high-water
observation, so subtracting live requests from it would not isolate allocator
overhead. The allocation driver's original second stderr record occurs after
release/join/reporting; the new captured snapshots above are the endpoint used.

Temporary traffic remains material, but a zero-traffic assumption is not a
design. With this live-growth value unchanged, the historical one-worker
allocation limit would allow about 124.781 MB of freed/superseded requests,
versus 333.733 MB observed. That arithmetic does not close the separate RSS or
CPU gates. Further work needs a concrete removable cost and a combined screen,
not another predicted sum of earlier savings.

## Native CPU follow-up and selected change

One native Time Profiler sample runs the unmodified frozen normal executable
`ef4149e8bee81dc5a40d182bb0dc8d1a11e731f08415c24abca41c254d862236`.
All work counts and the input digest match. The command/receipt inventories 207
native trace files; exported sample XML SHA-256 is
`c879db02159ca27445b23e2594197f38130e9c099e373dc5ac426ec8f9231c97`.
The profiled pipeline wall is 5.451934334 s. There is no comparative timing or
negligible-profiler-overhead claim.

The sample records 6,197 ms process CPU weight, including 5,371 ms on the worker:
2,718 parsing, 2,640 binding/publication and 13 unassigned. AST access accounts
for 115/188 ms within parse/bind. Full payload-enum construction has 1/123 ms;
that alone does not justify another broad AST accessor migration.

| Distinct operation and visible caller | Parse weight | Bind weight |
| --- | ---: | ---: |
| Keyword-search byte comparison | 172 ms | 82 ms |
| UTF-8 validation under `JsString::slice` | 93 ms | 51 ms |

These 398 ms are distinct samples. Required source-suffix comparison (71 ms),
other string validation and name/table comparisons remain outside this selected
target. Other broader profile groups overlap and must not be added.

Proceed with the reviewed [shared text-processing candidate](S07-bis-text-processing-plan.md):
generated byte matching for keywords and UTF-8 slice validity inherited only
when the parent and boundaries prove it. The source identifies avoidable work
used by both parser and binder; the profile establishes workload relevance.
Replacements still perform comparisons and boundary checks, so the sampled
weight is not an expected saving. Keep the compact layout and measure the
combined candidate once. This does not promise closure of the remaining gates.

The [complete diagnostic archive](../tools/s07/performance-experiments/results/2026-09-09-current-compact-costs/README.md)
retains 747 files with read-back hash verification: exact source/binaries,
build/capture receipts, failed staging attempt, raw observations, native exports
and the counter/source/caller audits. Native trace and workload sources remain
local; unchanged original candidate artifacts are pinned in its preceding archive.
