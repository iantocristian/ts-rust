# CP1 exclusive-core node-read experiment

**Retain the production fast path as the CP1 checkpoint.** The fixed full-workload
screen passed the declared improvement and nonregression checks, and independent
review, binder parity and the 29-case ownership inventory passed. This is a local
performance experiment. It does not establish the final Go-relative gates or
replace the native CI matrix.

| Workers / metric | A0-b control median | CP1 candidate median | Candidate / control | 95% ratio interval |
| --- | ---: | ---: | ---: | --- |
| 1 / wall | 4.943355667 s | 4.590853292 s | 0.928692 | [0.879316, 0.991721] |
| 8 / wall | 1.144259042 s | 1.079901417 s | 0.943756 | [0.917400, 0.952266] |
| 1 / requested allocation | 4,642,570,211 B | 4,642,570,307 B | 1.000000021 | — |
| 8 / requested allocation | 4,642,570,846 B | 4,642,570,848 B | 1.000000000 | — |
| 1 / peak RSS | 4,506,681,344 B | 4,506,681,344 B | 1.000000 | — |
| 8 / peak RSS | 4,509,253,632 B | 4,509,220,864 B | 0.999993 | — |

The observed wall medians improved by 7.13% and 5.62%. The one-worker confidence
interval is broad; those median percentages are observations, not precise
improvement guarantees. Allocation and RSS are effectively unchanged. All raw
observations remain present, including slower samples. No samples were discarded
and no rerun replaced this screen.

The screen used all 13,094 files, eight warmups and 56 measured observations:
seven control/candidate pairs for separate normal/allocation executables at one
and eight workers. Timing comes only from normal binaries. The declared policy
requires relative MAD at most 5% for every metric/variant/worker mode, timing's
95% upper ratio at most 1.02, and memory median ratios at most 1.02. A targeted
wall-time win requires a median ratio at most 0.95 and an upper confidence bound
below 1.0. **The CP1 infrastructure exception is unavailable to this shortcut.**
The plan criteria were committed in `ff6b5be` before measurement; the archive
helper preparation was made after capture and finalized after peer review.

The host was macOS arm64 with 18 physical CPUs and 64 GiB memory. Initial load
averages were 6.91 / 8.40 / 10.58. Capture ran in a coordinated quiet window, but
these results still describe this host and workload. They do not establish
contention attribution or behavior on the other CI architectures.

Full graph comparison covered every file at both worker counts and replayed to
parity 1.0 under the existing normalization. Eight-worker graphs had 49 raw
differences accepted by that normalization; the graphs were not byte-identical.
Both binding-path captures reported 13,094 exclusive files and zero fallbacks.
The separate binder corpus retained 22,343 primary request observations across
12,829 rows, plus 18 supplemental requests. The archive recounts those recorded
comparison observations; it does not reconstruct the omitted full binder graph
streams. Full benchmark graph NDJSON streams are retained and compared again.

The E3 replay checks every exact named test in all four modes: 29 distinct S07
cases, 116 successful observations. This includes both new exclusive-read tests
in debug, release, Miri and AddressSanitizer. The earlier binder attempt failed
because the sandbox denied Go-cache access (`Operation not permitted`); its
original evidence and log remain labelled as an environment failure. The retry
completed successfully. No failed attempt is counted as semantic parity or
ownership success.

| Frozen identity | SHA-256 |
| --- | --- |
| A0-b control manifest | `124956f668540814b95e6f677bdeae98bb2cad4f7586d3cb87f869e5c5af118f` |
| CP1 candidate manifest | `3a57976f667b9857891edbe9f76de5261c43480ac8c62d946b4545d9ee19d931` |
| Control normal binary | `4682c2d57af5fb180ce9cec93b6f1468535bc82388968b870fc784765b13160b` |
| Candidate normal binary | `36c8eaf369846e811287be7da517555ca742e40e88dd2599adee31b3450278c5` |
| Graph report | `d9c97ebcd5920c7b8a03c2ac1941ad4fca2bb0d5637d3baf8ff0e797789d9e8b` |
| Screen report | `66b6547a29aafaddbbc01cc834ee7460a79126dcb0dba133d5e6606b144cfc6b` |
| Successful binder evidence | `4a778e65f27886a66c0b6bbd2bd956d41ef6c4ae8e3bbaa671a7235c0082b72d` |
| Successful E3 evidence | `75dde69056152a7d7e876c4e28f4200ca3be9f4f3b90be07f4ce7dc7dd35f77a` |

The archive retains complete nonbinary inventories from both immutable builds,
all graph streams and child stderr, every screen/warmup ledger and raw output,
binder observations and production inputs, E3 evidence, validation/build logs,
and the complete capture/replay helper closure (including subprocess-only helpers). Executable bytes and the
original workload files are omitted. Frozen input recipes and the loaded-input
SHA `d4ff2ad3b0d590babb596522a7105d961dbb3c4f2ebc987488d88b7ebd8e0c88`
bind observations to the full workload. The candidate's frozen source SHA and the
screen's current-checkout source SHA agree:
`fcae7873d405b67d59421e523e4efd2379e77a648da5613248516ed374bff84b`.
Archive preparation does not claim a later working checkout was executed.

Independent generated-code inspection of the normal arm64 binaries found that
the control's `BindBuilder::node` calls `AstView::node`, while the candidate's
exclusive/same-core branch performs checked owner/slot/page access without a
call. Other IDs retain the `AstView::node` fallback. A0-b already bypassed the
overlay hash table through `direct_nodes`; this candidate removes remaining
routing, not overlay hashing. The observation does not inspect every inlined
caller or the allocation binary, and does not attribute elapsed time. Exact
`objdump --disassemble --disassemble-symbols=…` commands, binary hashes, successful
return codes and unmodified output are under `generated-code/` in the archive.

`manifest.json` inventories every archive member by size and SHA-256; `replay.json`
records the complete graph, sample, threshold, binder-observation and named-E3
replay. The replay executes no native child and needs no `target/` artifacts or
workload files. It requires the matching checked-out Python helper closure and
fails closed on drift. Helper snapshots are retained in the archive for recovery.
Temporary archive preparations are separate from the completed capture; the
archived helper snapshots are the authoritative replay implementation.

```sh
python3 -m unittest discover -s tools/s07/performance-experiments/results/2026-09-08-cp1 -p test_replay.py -v
python3 tools/s07/performance-experiments/results/2026-09-08-cp1/replay.py replay
```

Packaging requires independently recorded graph/screen hashes and an explicit
keep/reject decision. It refuses to overwrite an existing results archive.
