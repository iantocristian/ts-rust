# S07-bis: page, text and helper result

Status: complete combined experiment; retained after independent review under the existing reviewed-small-improvement rule. Final S07 gates remain open.

Follow-up: a [standard paired candidate/Go batch](S07-bis-candidate-acceptance.md)
now measures CPU ratios 1.213911 / 1.404619 and refreshes E5/E6 on this source.
It supersedes the historical cross-batch gate distances below; the original
candidate/control screen and its retention decision remain unchanged.

The [selected combination](S07-bis-pages-text-helpers.md) improves both CPU modes
and requested allocation, with a small, measured RSS increase. Keep the entire
combination. Preserve the runner's `no_demonstrated_win` verdict: its largest
median improvement is 4.67%, below its unchanged 5% substantial-candidate rule.
The previously agreed [small-improvement retention rule](S07-bis-performance-plan.md#6-screening-and-decision-rules)
allows this result: both timing upper bounds are below 1.0, all declared noise
and 1.02 non-regression conditions pass, and the reviewed changes do not sit on
a surface scheduled for replacement. This is not a retrospective threshold edit.

## Complete-pipeline screen

Eight warmups and all 56 observations were retained. Each cell below is the
median of seven observations for that artifact/mode/domain. Units are decimal.

| Metric | Workers | Refreshed control | Combined candidate | Change | Candidate/control |
| --- | ---: | ---: | ---: | ---: | ---: |
| Wall | 1 | 3.798938417 s | 3.621678708 s | −177.260 ms | 0.953340 |
| Wall | 8 | 0.826130333 s | 0.793662292 s | −32.468 ms | 0.960699 |
| Requested bytes | 1 | 2,243.576 MB | 2,145.921 MB | −97.655 MB | 0.956473 |
| Requested bytes | 8 | 2,243.577 MB | 2,145.907 MB | −97.670 MB | 0.956467 |
| Lifetime peak RSS | 1 | 2,318.975 MB | 2,326.020 MB | +7.045 MB | 1.003038 |
| Lifetime peak RSS | 8 | 2,322.235 MB | 2,329.313 MB | +7.078 MB | 1.003048 |

The timing 95% intervals are **0.949335–0.970727** at one worker and
**0.946349–0.999330** at eight. The eight-worker upper bound is only just below
1.0. Timing relative MAD is at most 0.971%; allocation/RSS dispersion also meets
the declared limits. No confidence interval is claimed for allocation or RSS.
Their complete sample ranges remain in the report.

The RSS increase is real relative to the narrow observed ranges; it is accepted
within the existing 2% allowance, not described as noise or as an improvement.
About 97.7 MB of requested allocation is saved for approximately 7.1 MB more peak
RSS. This reduces the allocation deficit by about 47%, while widening the RSS
deficit. No claim is made that every distance improved.

The representation adds tagged text-entry decoding and an exceptional-text map
lookup; sixteen-row pages charge default construction and unused tails. Shared
helper rules reduce duplicate checked/local algorithms. Those maintenance costs
and all compatibility paths are covered by the complete result. The experiment
does not isolate savings from pages, text or helpers, measure allocation-call
counts, or turn the old 4.84-million-call ceiling into an observation.

## Matched elapsed phases

One consuming observation per variant used the same unchanged adapter, compiler,
release profile, features, allocator, timer domain and frozen input. Both raw
captures replay successfully and select the consuming path for all 13,094 files.

| Elapsed total | Control | Candidate | Candidate − control |
| --- | ---: | ---: | ---: |
| Parse | 2.196316798 s | 2.184826535 s | −11.490 ms |
| Bind, publication and final validation | 1.574505534 s | 1.493337895 s | −81.168 ms |
| Complete adapter pipeline | 3.784141125 s | 3.692323708 s | −91.817 ms |

This observation suggests a larger change in binding than parsing. It supplies
no phase confidence bounds, sampled self-cost ranking or attribution to individual
components. The adapter's 91.8 ms difference does not replace the normal binary's
177.3 ms median difference, and neither is added to earlier measured savings.

## Historical cross-batch gate distances

The [fresh Go baseline](S07-bis-control-refresh.md) is preserved separately.
The following arithmetic compares this screen's candidate medians with that
batch's Go medians. It is **not a paired candidate/Go acceptance capture**.

| Gate distance | Workers 1 | Workers 8 |
| --- | ---: | ---: |
| Wall above Go median | 501.652 ms | 63.507 ms |
| Requested allocation above 0.70 × Go | 110.695 MB | 110.136 MB |
| RSS above 0.70 × Go | 117.645 MB | 113.186 MB |

Cross-batch wall changes are material: the same frozen control measured
1.037085 s at eight workers in the native refresh and 0.826130 s in this screen.
Only the same-screen 32.468 ms / 3.93% improvement belongs to this candidate.
The apparent candidate/Go ratio of 1.087 is contextual arithmetic, not a new
confidence bound or evidence that the eight-worker gate is nearly closed.
At this screen's completion, source-bound E5/E6 belonged to the preceding control
and were stale on the new source. The later paired acceptance batch linked above
supplies current evidence without reassigning these historical observations.

## Correctness and review

- Affected release libraries: 30 arena, 162 AST and 50 binder tests. Also 29 AST
  integration tests, 37 documentation tests and 64 xtask tests pass. Debug tests
  passed before the final semicolon-only Clippy fixes; final scoped debug coverage
  is included in E3. Rust 1.96, denied-warning Clippy, formatting, all 22 generated
  outputs and pinned client bytes pass on unchanged final sources.
- Full binder producer: 12,829 primary rows / 22,343 requests, 18 supplemental
  requests, 18 helper tests and 32 protocol tests; depth, graph contracts and
  resolver checks pass.
- E3: seven measured scenario rows, all 29 S06 and 83 S07 ownership cases pass
  debug, release, Miri and ASan. Final owner/allocation deltas are zero. The
  expanded inventory is checked exactly, rather than inferred from a substring.
- Both full workload graphs match: 13,094 files at each worker count, all using
  local/in-place binding. Eight-worker raw differences in 50 files satisfy the
  existing scheduler/name-counter qualifications; one-worker rows are raw-exact.
  The fresh control's verified Go streams are reused explicitly. They were not
  rerun or silently assigned to a different native source receipt.
- Independent source review covered page/text ownership and shared helper
  semantics. A separate result/retention review rechecked the complete raw screen, bundle and graph identities, source overlap, ownership receipts and matched phases. It found no blocker to retention and required the RSS regression and phase limits to remain explicit.

The initial sandbox failure and native binary-identity mismatch from the refresh,
preliminary compile failures, two semicolon-only Clippy failures, and graph/phase
lock rejections before execution remain retained. The lock rejections produced no
graph or timing observations. No failed sample was removed, no identical batch
was repeated, and no performance threshold was changed.

## Reproduction and disposition

Rust implementation: `02490d8`; immutable candidate frozen at documentation-only
HEAD `9a394bfe8542c098f1dd48b8ba6f3c2968302187`.

| Identity | SHA-256 |
| --- | --- |
| Refreshed control manifest | `957421942258d954765fc88b494b2031982dfac849d9358850dfdba7078edcd4` |
| Candidate manifest | `8d837bc0ab365972d2d5e0eeb0ee95255a505e44dea01964fca2198e78850898` |
| Candidate graph report | `68c004d4867a95d8423bb1d58ef817e0636ae86b10dbb60ccc341f457fadadf0` |
| Fixed screen report | `37d6d91f897f33a4ede09046b8ff7c14883cae7e6a77eddd750ddf8be9ba528b` |

The immutable bundles, raw measurements, matched phases, source and compiler
provenance, correctness evidence and failures belong to the
[review archive](../tools/s07/performance-experiments/results/2026-09-10-pages-text-helpers).
Its recipe replays graphs, native baseline, screen, phases and prerequisites from
retained files. The runner's automatic verdict remains unchanged; this record
provides the separate retention decision.

Retain this complete candidate as the next experimental control. This bounded
experiment is finished; it did not close the memory or CPU gates. Further work
requires a plan that addresses the remaining retained-memory and CPU costs. No
page-size matrix, new trace pipeline or repeated batch is queued by this result.
