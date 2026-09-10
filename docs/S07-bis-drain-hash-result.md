# S07-bis: empty-drain and whole-name-hash result

Status: steps 1 and 2 are complete. All correctness and workload graph checks
pass, but the fixed screen is inconclusive. The candidate is **not retained**;
production and evidence pointers are restored to the preceding control. No
step-3 code or scheduling change has been implemented. No second candidate is
selected from these results.

## Scope and identities

The [bounded plan](S07-bis-drain-hash.md) authorizes one current-control CPU
profile, one eight-worker diagnostic per runtime, and one combined candidate.
The production changes are in `187d956`: avoid an empty scanner diagnostic
`Vec::Drain`, and hash a complete canonical name without the generic slice
length prefix, using the same per-owner randomized hasher. The [hash decision](S07-bis-hash-decision.md)
records byte equality, collision policy and rejected alternatives.

| Identity | SHA-256 |
| --- | --- |
| Control manifest | `1c04605dcd248d78dcec1e42c4c6f82f036843a054cf5e27d81c1b3b229753b5` |
| Candidate manifest | `7d44dbaa8695fd6b714539a00878207b4c26312ee0138527642481d91c2ea4a5` |
| Candidate production fingerprint | `3860325014cc85cc99d8c123b6cacd7f4096d9595d66bf8c8fe155213ad62266` |
| Candidate normal executable | `a8a3e85172f4d94435378d308635c58057062f771e018e3dadc3a6b387522bff` |
| Candidate allocation executable | `04d593edf4d025820599ad9f7e615f86b77efea81d2f42ec0dea59120931e5c0` |

The frozen candidate lives at `target/s07-bis/drain-hash/candidate` and uses the
control's verified Go executable and all 13,094 workload files. Both Rust
artifacts were built in fresh isolated output/intermediate directories with the
declared native release profiles, after allocator/MSRV preflight. No diagnostic
instrumentation is present in these binaries.

## What the diagnostics establish

The [single native CPU profile](S07-bis-drain-hash-profile.md) attributes 2,796 ms
of sampled Running CPU to parse and 2,089 ms to bind/publication. It finds 82 ms
of scanner Drain self weight and 112 ms of canonical-name hashing self weight.
These are costs to investigate, not additive savings forecasts. Necessary
hashing, diagnostic delivery and symbol/flow validation remain.

The [matched worker diagnostic](../tools/s07/performance-experiments/worker-timing/RESULT.md)
preserves round-robin queues, work and retention in both drivers. It records
mean receive-call elapsed of 458.2 ms for Rust and 348.5 ms for Go, or 37.1% and
39.5% of their respective instrumented pipeline endpoints. The same worker gets
the most input bytes and work in both. Completion spreads are only 2.65 / 1.70 ms.
Send/receive elapsed includes scheduling and channel work; it is not dispatcher
CPU. One observation per runtime cannot establish a Rust-specific scheduling
defect, a causal queue-policy gain or an acceptance ratio. There is no scheduling
candidate selected by this result.

## Correctness

The 327 affected release tests pass, including 24 AST compile/documentation
examples. The scanner state suite also passes in debug. The eight symbol-table
tests pass in debug and release. Formatting, strict affected-target Clippy and
the declared Rust 1.96 checks pass. The full scanner producer passes 33,332
frozen cases with 4,982,432 matching observations, including diagnostics,
token-value bytes and rescans. These are semantic observations, not timing
samples.

The full binder producer also passes: 22,343 primary requests / 12,829 frozen
rows, 18 supplemental requests, 18 helper tests and 32 protocol tests, including
depth, resolver and graph-contract obligations. Scanner evidence is
`62e095bff787913c61ecc053b9b327b1910bf29cac7b88adc9037fda1752a7ab`;
binder evidence is
`22ff134ca25bbd4caefe7568b350d291751e0d3f093ffcca2e6030034a193beb`.

The preserved validation harness is
`target/s07-bis/drain-hash/validation/run.py`. The existing 7 / 29 / 83 ownership
inventories pass in debug, release, Miri and ASan, with zero final owner and
allocation deltas. E3 evidence is
`8732e7bcae4fbb764f3140393b08de496d691db1ae429d205d8f1317df0f1252`.
The separately filtered hash-boundary test also passes exactly once under Miri
and once under ASan; neither run widens the frozen E3 inventory. The complete
validation receipt is
`f3d32cab5dcbb555f3b37b92ed5265e4b52cba2c508bc4e2449371ba7cafe2ce`.
Full workload graphs pass all 13,094 files at both worker counts. Every file
selects in-place binding, with zero fallback files. One-worker graphs are all
raw-exact; 55 eight-worker raw differences meet the existing scheduling/name-counter
qualifications, with no failed graphs. Graph report:
`a349005513d8e2f051727a16e27cc4a15310ef0a8279a55803767309e201a7ae`.

## Fixed combined screen and disposition

All eight excluded warmups and 56 scheduled samples completed. There was no
extension, retry, component-only screen or changed threshold. The unchanged
runner's receipt and statistics replay passed. Report:
`ac09070803a527972a071e4fdc56f869041daa8144a9eb87d7812f4a189c0cba`.

| Metric | Same-screen control | Candidate | Candidate/control |
| --- | ---: | ---: | ---: |
| One-worker wall | 3,953.943 ms | 3,876.564 ms | 0.980430 |
| Eight-worker wall | 875.090 ms | 863.625 ms | 0.986898 |
| One-worker allocation | 2,145.903 MB | 2,145.921 MB | 1.000008 |
| Eight-worker allocation | 2,145.912 MB | 2,145.910 MB | 0.999999 |
| One-worker peak RSS | 2,325.725 MB | 2,325.643 MB | 0.999965 |
| Eight-worker peak RSS | 2,329.035 MB | 2,328.936 MB | 0.999958 |

The median CPU differences are −77.379 / −11.465 ms, or −1.96% / −1.31%.
The 95% ratio intervals are **0.954296–1.008483 / 0.894933–1.022502**.
Neither establishes a timing win, and the eight-worker upper bound exceeds the
existing 1.02 non-regression allowance. Relative MAD remains within the 5%
limit: control/candidate are 1.378% / 1.052% at one worker and 3.890% / 3.220%
at eight workers. Memory changes are negligible: requests +0.018 / −0.002 MB,
RSS −0.082 / −0.098 MB.

Preserve the raw verdict **`regressing_or_uncertain`**. The correct interpretation
is **inconclusive**, not a demonstrated regression and not a demonstrated win.
The small-improvement rule requires at least one timing upper bound below 1.0
and all non-regression conditions; neither requirement is met. This is not a
necessary representation-infrastructure step. Independent result review agrees
that no retention path applies. The failure to retain is not merely that the
median improvement falls below 5%.

Both production edits, including the candidate-only boundary test, are restored
to their exact `799adbf` contents. The immutable candidate, its source commit
`187d956`, all samples and passing candidate correctness records remain available.
The scanner/binder/E3 latest pointers return to their preceding records; the new
records are retained as candidate evidence, not presented as checks of restored
source. The E5/E6 pointers remain unchanged. The restoration receipt proves all
289 benchmark-source hashes match the existing control fingerprint
`1c661958f0982af9d34dcfd562f6b2c7d6d87c664e4d7dc3fc6dac682ceefbd3`.

There is **no new Go acceptance batch** for the unretained candidate. The
[preceding paired acceptance](S07-bis-candidate-acceptance.md) still describes the
restored implementation. Its distances remain 653.1 / 245.1 ms CPU,
110.7 / 110.1 MB allocation and 118.1 / 112.5 MB RSS at one/eight workers.
Those are retained-control observations from that batch; this screen supplies
no new Go-relative candidate ratios. No separate measured component gains are
added together or combined with historical Go medians.

The two mutable Rust benchmark-cache binaries are also restored from the exact
frozen control. Their candidate predecessors remain preserved. Go and input
transport bytes are unchanged. The existing standard `verify-capture` replay
passes once, without a build or native workload invocation; restoration receipt
`10c8f216d43a0ef89d9c5f6424de87cbbf20d7be752d14ee72734d1458893db4`
records this separately from candidate validation and measurement.

## Step 3: bounded question, not a broad rewrite

Parent attachment already uses direct core payload reads and header writes in
`AstBuilder::override_core_parents` and `CoreParents`. Its eligibility check calls
`StorageBuilder::is_core_only`, then `LazyArena::has_records`, then a shared read
that checks initialization and acquires the lazy lock. The profile places 21 ms
in the initialization guard, 16 ms in the lazy read and 8 ms in thread-local
address lookup beneath parent attachment. The full 330 ms attachment group also
contains required child enumeration and parent writes.

An exclusive query through `RwLock::get_mut` is a concrete small proof question;
the lazy cache's exclusive seeding path already uses that API. Preserve imports,
node and auxiliary reservations, eager core-only JSDoc cache entries, poisoning,
dirty/escaped fallback behavior and error order. Exclusivity alone does not
justify removing the initialization assertion: callers can manually install the
public same-arena guard while retaining mutable builder access. Keeping that
assertion is the conservative equivalent change. This greatly limits what can
honestly be claimed removable from the profile.

**Recommendation: stop this follow-on after the completed screen.** The current
evidence identifies no proportionate second candidate. The 16 ms lazy-read self
weight is an opportunity indicator, not a removable-work estimate or a hard
ceiling, but it supplies no case for the proposed broad construction rewrite.
The unchanged initialization behavior further limits the obvious narrow query
change. Keep that concrete observation in the backlog rather than commissioning
another diagnostic series around it.

No evidence here establishes Claude's proposed 100–300 ms construction saving
or a fixed 1.05–1.10 performance floor. No further profile series, scheduling
change, final-validation removal or parser-wide accessor migration is selected.
This does not claim that further optimization is impossible; it records the
limit of what these authorized experiments justify. Any renewed investment or
acceptance-threshold decision is separate.

## Artifacts and replay

The [result package](../tools/s07/performance-experiments/results/2026-09-10-drain-hash)
provides the archive identity and extracted-root replay instructions. It retains
the immutable control/candidate artifacts and source, native profile/XML,
worker diagnostic and setup failures, complete graph streams, every timing
sample, correctness records and restoration receipts. The unchanged raw verdict
and the removed candidate remain inspectable.

Replay is passive: it recomputes graph comparisons, timing statistics, XML
attribution, worker accounting and correctness-record assessments. It does not
execute the benchmark or rerun test binaries. Profile/worker workload bytes are
not duplicated in this package; their original per-file hash recipes and loaded
digests are checked, with that omission stated in replay reports. All 539 inputs
in the correctness source inventory are retained and rehashed. Historical path
strings remain provenance; explicit relocation does not rewrite original
manifests or pretend to restore the original immutable filesystem permissions.
