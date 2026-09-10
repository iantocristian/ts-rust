# A0b: retained A0 checkpoint

**A0b passes the diagnostic screen and has been retained after review.** Full
workload graph comparisons passed at both worker counts, and all 27 S07
ownership cases passed in debug, release, Miri and AddressSanitizer. This is the
A0 checkpoint, not completion of S07-bis: the Go-relative CPU and memory gates
remain unfinished. The [initial A0 result](README.md) remains unchanged and was
not promoted.

A0b keeps exclusive binding and preserves the completed parse-validation proof
during narrow flag writes. Unrestricted parent/payload mutation still invalidates
that proof, and binding-result validation remains required.

| Domain | Workers | Original Rust control median | A0b median | A0b / control | Timing upper 95% bound |
| --- | ---: | ---: | ---: | ---: | ---: |
| Pipeline wall time | 1 | 5.183442 s | 4.921846 s | 0.949532 | 0.968732 |
| Pipeline wall time | 8 | 1.232323 s | 1.165151 s | 0.945492 | 0.958537 |
| Lifetime peak RSS | 1 | 4.564468 GB | 4.506714 GB | 0.987347 | — |
| Lifetime peak RSS | 8 | 4.566729 GB | 4.509221 GB | 0.987407 | — |
| Requested allocation | 1 | 4.728501 GB | 4.642569 GB | 0.981827 | — |
| Requested allocation | 8 | 4.728503 GB | 4.642572 GB | 0.981827 | — |

GB is decimal. Normal binaries provide wall time/RSS; allocation observations
come from separate instrumented binaries. All 56 samples and 8 warmups remain
present. The declared wall-time target improves by 5.05% / 5.45%, with both timing
upper bounds below 1.0. Every screening non-regression and noise condition passes.
The measured memory reductions remain about 1.27% in RSS and 1.82% in requests;
substantial storage work is still required.

Both complete graph modes compared all 13,094 files against the exact frozen Go
executable. Separate untimed observations recorded 13,094 exclusive bindings and
zero fallbacks in each mode. Archived graph files contain complete per-file
graph observations/digests, counts, diagnostics and qualified identities; they
do not contain full serialized ASTs for every successful file.

## Phase attribution

One diagnostic binary selected published or consuming binding at runtime, using
the same A0b source revision. Each backend has one warmup and seven measurements
at one worker. These per-file elapsed timers include scheduling/timer overhead
and group binding with publication. They are neither sampled CPU nor the normal
binary's acceptance timing.

| Timer | Published median | Consuming median | Consuming / published |
| --- | ---: | ---: | ---: |
| Parse | 2.524859 s | 2.523302 s | 0.999384 |
| Binding plus publication | 2.811072 s | 2.437283 s | 0.867030 |
| Diagnostic pipeline wall time | 5.380690 s | 4.974198 s | 0.924454 |

The grouped binding/publication interval is 13.3% lower, while parse is nearly
unchanged. The published backend here is the current revision's compatibility
path, not the original frozen control used in the preceding screening table.
Do not combine these different timing domains to predict further savings.

## Provenance

The candidate and phase binary share production-source fingerprint
`ea16eb22eae1e951cd9f3e4892512171be21d61d360bd592b6acb8447711417c`,
frozen at revision `584a7feea65f602453d4c2e604e89acf6d63dd20`.

| Identity | SHA-256 |
| --- | --- |
| A0b candidate manifest | `124956f668540814b95e6f677bdeae98bb2cad4f7586d3cb87f869e5c5af118f` |
| A0b normal executable | `4682c2d57af5fb180ce9cec93b6f1468535bc82388968b870fc784765b13160b` |
| A0b allocation executable | `7ac3f3a1e961c4a9f26f146bb66b69dd74f73792f3214b6d64e305b09fd142c7` |
| Phase build manifest | `8b42fd1109b96fbb56ef2411e39ee2b5c15ef93fd10f84948d66ada334c8dda7` |
| Phase executable | `b2e0aef7598b0b33d3a7eb0c12a693cc6306efcd381778a7ab0399f46240ed42` |
| Recorded E3 evidence | `14950fa0ae8904282d890070c360c79380fa38a9da753ef21dfa751231ac95b5` |

Phase build and capture reports name different `directory_sketch.rs` versions:
`a1fc900d…` at build and `3ed6b077…` at capture. The archive preserves both exact
versions under distinct tool directories. Those files and the original
`project_layout.py` (`78ebc208…`) were recovered after capture and checked against
the originally recorded hashes. Later projection edits are excluded. Neither
the sketch nor projection script is consumed by the phase executable or phase
statistics; the original probe, Rust driver, manifests and input transport are
retained separately. The reports themselves have not been rewritten.

The actual core census and later layout projection belong to the separate
[CP0 artifacts](../../phases/README.md); they are not duplicated or relabeled as
A0b phase observations here.

## Archive and replay

- [a0b-raw.tar.xz](a0b-raw.tar.xz): all 214 members, 67,751,163 uncompressed
  bytes; 6,985,008 compressed bytes.
- [a0b-manifest.json](a0b-manifest.json): per-member byte sizes/hashes and all
  top-level artifact identities.
- [a0b-replay.json](a0b-replay.json): replay receipts and original-bundle checks.
- [replay_a0b.py](replay_a0b.py): checks member integrity and replays graph,
  screening, phase and S07 ownership observations without running captured
  executables.

Archive SHA-256:
`3ac3279d1d7e5116690708962d1622b2cb605c9245067ce8359fdcbea9f10ecb`.

The archive includes exact control/candidate manifests, all graph and screen
files, phase build/capture records, the original phase input transport and tool
snapshots, and complete E3 evidence with separately preserved stdout/stderr.
The E3 replay verifies 108 successful named observations: five suites containing
27 distinct S07 tests across four modes. It does not certify every future E3
criterion. Executables, production source snapshots and workload bytes remain
outside this compact review archive and were checked in their original immutable
bundles before packaging.

```sh
python3 tools/s07/performance-experiments/results/2026-09-08/replay_a0b.py
```

Replay retains every observation, validates both 13,094-file graph modes,
recomputes screening statistics and all 14 phase measurements, checks the exact
18 build/capture tool snapshots, and checks the named S07 test output underlying
the recorded ownership metrics. It does not download, build or execute either
compiler and does not emit tracker metrics.
