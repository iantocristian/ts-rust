# Rejected CP1 bounded list-copy experiment

**Reject this production shortcut; retain the earlier CP1 implementation as the
accepted control.** Copying up to 16 syntax-list IDs before recursive binding
reduced the measured wall medians by 35.842 ms at one worker and 1.854 ms at eight
workers. Neither mode met the predeclared 5% median improvement threshold.
Nonregression checks passed; allocation and RSS were effectively unchanged.
This decision concerns this bounded CPU experiment, not future list-storage
designs or the final Go-relative gates.

| Workers / metric | CP1 control median | Rejected candidate median | Candidate − control | Candidate / control | 95% ratio interval |
| --- | ---: | ---: | ---: | ---: | --- |
| 1 / wall | 4,344.062333 ms | 4,308.220625 ms | −35.841708 ms | 0.991749 | [0.989408, 0.995231] |
| 8 / wall | 970.697959 ms | 968.843792 ms | −1.854167 ms | 0.998090 | [0.989107, 1.001143] |
| 1 / requested allocation | 4,642.569475 MB | 4,642.569091 MB | −0.000384 MB | 0.999999917 | — |
| 8 / requested allocation | 4,642.570910 MB | 4,642.572046 MB | +0.001136 MB | 1.000000245 | — |
| 1 / peak RSS | 4,506.697728 MB | 4,506.615808 MB | −0.081920 MB | 0.999981823 | — |
| 8 / peak RSS | 4,509.220864 MB | 4,509.204480 MB | −0.016384 MB | 0.999996367 | — |

MB means 1,000,000 bytes. The one-worker timing interval remains below one, but
the observed 0.825% improvement is below the required 5%. The eight-worker median
improves by 0.191%, with an interval crossing one. All seven samples per variant,
metric and worker mode remain present. No samples were filtered, no batch was
extended, and no timing rerun replaced this result. The earlier CP1 capture's
4.590853292-second candidate median belongs to another batch; subtracting it from
this candidate's median would not measure the list-copy change.

The screen covered all 13,094 files, eight warmups and 56 measured observations,
with separate normal/allocation executables at one and eight workers. Its
[predeclared plan](../../../../../docs/S07-bis-list-copy.md) required relative MAD
at most 5% for every metric/variant/worker, wall time's 95% upper ratio at most
1.02, and allocation/RSS median ratios at most 1.02. A targeted wall win required
a median ratio at most 0.95 in at least one mode and its upper bound below 1.0.
The infrastructure exception was unavailable. Archive preparation happened after
capture and does not establish a new screen policy.

The host was macOS arm64 with 18 physical CPUs and 64 GiB memory. Initial load
averages were 6.604 / 7.734 / 8.484. Capture ran after the coordinated workload
and build jobs stopped. These observations describe this host and workload;
they do not establish cross-platform performance or attribution to contention.
The [historical Go-distance diagnostic](../../gate-distance/README.md) keeps the
same-screen control and rejected candidate separate and supplies no fresh
cross-runtime confidence bounds.

The complete graph replay compares every file again at both worker counts.
Parity is 1.0 in each mode under the existing normalization. One-worker graphs
were raw-exact for all 13,094 files; eight-worker graphs were raw-exact for 13,035,
with 59 normalized raw differences. Both binding-path observations reported
13,094 exclusive bindings and zero fallbacks.

The final binder capture passed every declared metric, including depth, and
retained 22,343 primary request comparisons across 12,829 rows plus 18 supplemental
requests. Depth replay additionally validates all 12 native counter observations
against the corrected producer and recounts all 12 pinned graph-comparison rows.
The binder/depth source inventories are tied to the frozen candidate or separately
retained source bytes. Full benchmark graph streams are archived; full binder and
depth graph streams are omitted, so those latter checks replay recorded comparison
observations rather than reconstructing the original graphs.

The candidate's E3 inventory has 32 distinct S07 cases and all 128 named test
observations across debug, release, Miri and AddressSanitizer. Its validation logs
also preserve 182 library tests and six AST doctests. These are historical
candidate results. After restoring the production source and ownership inventory,
they do not become new evidence for the retained control's 29-case E3 inventory
or 179-test library suite. No future E3 coverage or native CI matrix is implied.

Two earlier attempts remain visible. The initial build preflight refused to run
while Cargo validators were active; it created no candidate binary or timing
measurement. The first complete binder capture had a valid envelope and passing
corpus graphs but reported `depth=false`. The injected-unwind row observed 468
guard entries, one actual stack growth and terminal failure; the old producer
incorrectly demanded more than 500 entries even though that probe stops at its
first growth. Its original evidence, native log/report, request observations,
producer and cases remain archived with `accepted=false`. The corrected producer
requires positive guard entries, actual growth and terminal failure for that
probe, while retaining the greater-than-500 requirement for completed depth
probes. A separate complete binder retry passed. The original failed result was
not rewritten or counted as accepted validation.

The static-code review retains all 37 files, including every stdout/stderr from
16 successful commands, both frozen source excerpts and its findings. The receipt
identifies the exact control/candidate normal arm64 binaries. It confirms chunk
resolution and ID copying before recursive calls, lazy guard release, two whole
functions-first passes and kind reads at visitation. Named caller-frame subtotals
increase from 176 to 496 bytes for functions-first traversal and from 960 to 1,200
bytes for a child-list path. These exclude descendants, temporary resolution
frames and stack-growth slow paths. Static inspection establishes no elapsed-time
attribution and does not inspect the allocation binary.

| Frozen identity | SHA-256 |
| --- | --- |
| Retained CP1 control manifest | `3a57976f667b9857891edbe9f76de5261c43480ac8c62d946b4545d9ee19d931` |
| Rejected list-copy candidate manifest | `b03b64ebf3faed4bb2245fa7b76db4289c3a1e510e796e86337c0782fb5f3578` |
| Full graph report | `c6ce91ed6eff72b9dfb771fb2951e0996b09dc7d4d6d317b21622b349daa5875` |
| Screen report | `8ccc065b2c33b45ddfed83ca3b678c7d0c87d927f017aaf7e6026ab7680f4f10` |
| Final binder evidence | `fd6389d0fb7035c747e2991e37b6e595d87dad2c91e16fc2730c5f970b668334` |
| Candidate E3 evidence | `31cde17440f7568517c8a5ad81ab473e959848d05bb80cac5774b66e2dc66914` |
| Archive | `f318c1ab8f75f3ef4a223a0547452c0254cc8b4ee7f6d6c280964e9cd88171cb` |

The 14,059,400-byte archive contains 745 members totaling 243,647,596 uncompressed
bytes. It preserves complete nonbinary build inventories, all 15 benchmark graph
streams/report, every screen ledger/raw output, binder/depth observations and
source closures, E3 evidence, validation/build logs, failed attempts and static
inspection. Executable bytes and original workload files are omitted. Frozen
input recipes and loaded-input identities remain in the build/capture records.

The adapter composes the unchanged SHA-pinned [CP1 replay](../2026-09-08-cp1/replay.py)
for build/source verification, graph comparison and screen schedule/guard checks.
Its 26-file helper closure includes both adapters/tests, imported modules and
subprocess-only capture helpers; exact snapshots are retained. Replay requires
matching checked-out helpers, fails closed on drift, and needs no `target/`
artifacts or native binaries. It preserves an explicit rejected checkpoint
decision without asserting final production promotion or Go-relative acceptance.

```sh
python3 -m unittest discover -s tools/s07/performance-experiments/results/2026-09-08-list-copy -p test_replay.py -v
python3 tools/s07/performance-experiments/results/2026-09-08-list-copy/replay.py replay
```

`manifest.json` binds every archive member by size and SHA-256. `replay.json`
contains the complete recorded-observation replay, including all sample values
and the rejected decision. Packaging refuses to overwrite an existing archive;
temporary preparation remains under `target/`.
