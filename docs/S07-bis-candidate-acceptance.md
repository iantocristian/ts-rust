# S07-bis: paired acceptance of the retained candidate

Status: one standard native Go/Rust batch completed on 2026-09-10. Capture
verification and both E5/E6 evidence consumers pass. All four performance gates
remain open; successful evidence recording does not mean the gates pass.

This measures the retained [page/text/helper implementation](S07-bis-pages-text-helpers-result.md)
at `26b6c4efc507e002130330aad30f9a5496d20f1e`. Its production source fingerprint,
`1c661958f0982af9d34dcfd562f6b2c7d6d87c664e4d7dc3fc6dac682ceefbd3`, is unchanged
from that experiment. Fresh native executables passed all 13,094 workload graph
comparisons at both worker counts before the exact binaries were measured.

## Paired results

The unchanged standard producer executed eight excluded, validated warmups and
56 recorded fresh-process observations. Each median below has seven samples.
The frozen stopping rule requested no extension. No samples were discarded and
no batch was retried. Units are decimal.

| Metric | Workers | Go median | Rust median | Rust / Go | Gate |
| --- | ---: | ---: | ---: | ---: | ---: |
| Wall seconds | 1 | 3.053027125 | 3.706103000 | **1.213911** | ≤ 1.00 |
| Wall seconds | 8 | 0.605686167 | 0.850758292 | **1.404619** | ≤ 1.00 |
| Requested MB | 1 | 2,907.466 | 2,145.901 | 0.738066 | ≤ 0.70 |
| Requested MB | 8 | 2,908.318 | 2,145.913 | 0.737854 | ≤ 0.70 |
| Lifetime peak RSS MB | 1 | 3,154.559 | 2,326.299 | 0.737440 | ≤ 0.70 |
| Lifetime peak RSS MB | 8 | 3,166.863 | 2,329.330 | 0.735532 | ≤ 0.70 |

The CPU 95% bootstrap intervals are **1.149962–1.235345** at one worker and
**1.280184–1.442985** at eight. Timing relative MAD is 2.499% / 2.659% for Go
and 1.118% / 0.768% for Rust, within the existing 5% dispersion limit. The
producer's `stable=false` also incorporates the requirement that the CPU upper
confidence bound be at most 1.0; it does not mean the dispersion test failed.

| Remaining distance | Workers 1 | Workers 8 |
| --- | ---: | ---: |
| Wall above Go median | 653.076 ms | 245.072 ms |
| Requested allocation above 0.70 × Go | 110.675 MB | 110.091 MB |
| RSS above 0.70 × Go | 118.107 MB | 112.525 MB |

The earlier screen's contextual eight-worker ratio of about 1.087 combined
0.793662 s Rust with 0.730155 s Go from another batch. This batch measures both:
0.850758 s Rust and 0.605686 s Go. **Use 1.405 for the current measured gate
distance, not 1.09.** This is not evidence that the implementation regressed
between captures: its production source is unchanged. The previous 3.93%
candidate/control gain remains a result of its own Rust-versus-Rust screen.
Neither comparison isolates the causes of the cross-batch variation.

## Host and measurement boundaries

The user stopped Cursor, then explicitly directed the run to proceed with Codex
open. No other assistant builds, reviews, profiling or archive work ran during
sampling. The standard producer's compiler/benchmark process checks remained
enabled; no detached wrapper, idle threshold, sampler hook or producer change
was used. This is not represented as a capture with every application closed.

The host reported macOS 25.6.0, arm64, 18 CPUs and 68,719,476,736 bytes of memory.
Initial load averages were 4.933594 / 5.233398 / 5.277832. The producer records
this initial load and its sample dispersion; it does not establish continuous
whole-host inactivity or thermal isolation.

Wall and RSS come from uninstrumented Rust. Original allocation requests come
from the separate allocation binary, following the existing cap/mimalloc and
Go TotalAlloc definitions. The allocation wrapper's observed eight-worker wall
overhead was 32.07%; that instrumented timing does not enter the CPU gate and
no adjustment is applied to uninstrumented timing. No phase attribution or new
optimization decision is inferred from this acceptance batch.

## Evidence and reproduction

The standard sequence was `cargo xtask run bindworkload`,
`python3 scripts/s07_benchmark.py capture`,
`python3 scripts/s07_benchmark.py verify-capture`, then
`cargo xtask run e5` and `cargo xtask run e6`. The graph and measurement producers
validated source, Cargo configuration, input, executable and report identities.

| Identity | SHA-256 |
| --- | --- |
| Frozen native bundle | `1c04605dcd248d78dcec1e42c4c6f82f036843a054cf5e27d81c1b3b229753b5` |
| Graph report | `0269757498dced1af8910c4ca529876d216527a47a5211881f1e1ed889f2947a` |
| Paired capture report | `fb7ebaa62baa8e30e1a0168b0b6d9994f693ed713e7af1aff5cfe2f45c2f47ad` |
| Graph evidence | `483f6951d8c982aff9ed3d4fe12fb321b9f62875ccf09a6d8da8b54dbffeb797` |
| E5 evidence | `ae0e9fc639b2dfa698e7171cd9319193407b42bbd6a21362ebf02ff5306284fa` |
| E6 evidence | `d5e7541f724caf1f157175370fca91f8988287c796cedf8c80d29c3d18378ab3` |

Raw reports, logs, exact native binaries and source are retained under
`target/s07-bis/current-candidate-acceptance-2026-09-10`; the
[review archive](../tools/s07/performance-experiments/results/2026-09-10-candidate-acceptance)
preserves the batch for offline replay. Extraction verified all 417 members,
replayed both complete graph comparisons and recomputed all 56 observations and
gate metrics successfully. Archive SHA-256:
`fd53fb29fd5c1fad9f46816c5b428bfbf126aaaf73439888e0df216aef6df283`.

An independent result review verified aggregation, stopping, source and binary
identities and found no blocking issue. Regenerated tracker views pass
`cargo xtask status --check-committed`. `cargo xtask check S07` remains failing,
including the measured performance failures and its other unfinished obligations.
Earlier controls and their raw results remain separate. E5/E6 now refer to
current source rather than the preceding control's stale evidence.
