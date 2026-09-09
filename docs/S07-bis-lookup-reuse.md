# S07-bis: bounded binder lookup reuse

Date: 2026-09-09. Control: retained CP1 bundle
`target/s07-bis/cp1-node-read-candidate`, manifest
`3a57976f667b9857891edbe9f76de5261c43480ac8c62d946b4545d9ee19d931`.

The hypothesis is that local read consolidation and forwarding already-known
kinds remove enough repeated routing to improve the complete pipeline. The
recorded 122,786,331 `Binder::n` operations do not establish their machine cost
or how many are redundant. Neither two reads per physical node nor a 200–250 ms
saving is an expected result asserted by this experiment.

## Contract and scope

Use one candidate covering known-kind dispatch/container classification and
the audited narrowing/function-expression read sites. Retain the public checked
container classifier. Internal kind forwarding relies only on the binder's
stable syntax; actual payload-shape checks must survive unusual factory
kind/data pairs. Preserve short-circuiting and failed-access order.

Consolidate short borrows within mutation-free regions. Copy only the scalar or
ID needed to cross recursion/mutation. In particular, do not keep a lazy arena
read guard across another resolution, and do not introduce a generic 16-byte
header passed everywhere. Kind/parent/syntax edges are stable in this binder;
flags are not. Source/module entry changes export context, child/container work
changes reachability and `CONTAINS_THIS`, and error writes must use current
flags. Preserve fresh reads across those boundaries. The owner, publication,
binding storage, recursion policy and syntax-list representation are unchanged.

Counterexamples include short-circuited absent payloads, mismatched kind/shape,
optional chains, binary continuation stages, recursive container flag changes,
and lazy/published reads. Existing independently generated Go helper and binder
observations remain the authority; add only focused regressions for new behavior
boundaries rather than a new event inventory.

## One screen, then a decision

Review the code, run focused binder tests and affected lint/MSRV checks, freeze
the normal/allocation binaries, and compare all 13,094 workload graphs with Go
at one and eight workers. Use the existing performance runner without changing
its metric definitions, schedule or helper implementation.

The fixed screen retains eight warmups and 56 measured children with alternating
candidate/control order. Both modes require relative MAD at most 5%, wall upper
95% ratio at most 1.02, and allocation/RSS median ratios at most 1.02. A targeted
wall win requires median ratio at most 0.95 in at least one mode and its upper
bound below one. No sample replacement, extension or post-result variant tuning.
Report absolute milliseconds and bytes first. A noisy or failed screen is
inconclusive/rejected, not permission to keep sampling until it wins.

If the screen is promising, run the broader binder parity and current S07
ownership instrumentation before promotion. Inspect generated code only where
needed to substantiate a claimed mechanism; do not build a new trace consumer
or turn this into exhaustive disassembly attribution. If it misses the screen,
restore production code, preserve the experiment and proceed to compact storage.
This ordering spends the expensive ownership producer on a candidate that may
be retained; it does not waive correctness requirements for promotion.

The result does not close a Go-relative gate, attribute the entire remaining
bind gap, or quantify a standalone lookup latency. Fresh paired Rust/Go evidence
and complete owned allocation/RSS attribution remain required for S07 acceptance.

## Result: reject and proceed to storage

The criteria were committed as `18510dd` before measurement. All eight warmups
and 56 measured children completed, with no filtering, replacement or extension.

| Workers | CP1 control | Candidate | Difference | Candidate/control | 95% timing ratio interval |
| --- | ---: | ---: | ---: | ---: | --- |
| 1 | 4,604.502 ms | 4,552.492 ms | −52.010 ms (1.13%) | 0.988705 | [0.968801, 0.996433] |
| 8 | 1,062.411 ms | 1,049.518 ms | −12.893 ms (1.21%) | 0.987864 | [0.958147, 1.001338] |

Both modes meet the noise/non-regression conditions; neither meets the 5%
targeted-win criterion. The eight-worker interval includes no benefit. Requests
remain about 4.643 GB and RSS about 4.51 GB; the byte differences are immaterial
(requests +368 / −2,110 bytes; RSS −16,384 / −32,768 bytes).

The four changed binder files and their two new regressions were restored to
CP1. Immediately after restoration, before starting the compact-storage API
migration, the complete benchmark-source fingerprint matched the retained control:
`fcae7873d405b67d59421e523e4efd2379e77a648da5613248516ed374bff84b`.
Do not retain this leaf optimization through an infrastructure exception. No
further variant, lookup trace or timing run is scheduled to rescue the result.

Full workload graph parity passed for all 13,094 files at one and eight workers,
using the existing normalization. Each mode observed 13,094 exclusive bindings
and zero fallbacks with the frozen loaded-input identity. The candidate passed
30 debug and 30 release binder library tests, formatting, all-target Clippy with
warnings denied, and the all-target Rust 1.96 check. Two independent code reviews
found no substantive issue. One newly added fixture initially omitted its source
file name and failed setup; the original failure log is retained. After fixing
the fixture, the tests and the single build/graph/screen sequence passed.

The broader promotion-only binder and ownership producers were not run, as
predeclared. This rejected candidate is not new S07 instrumentation evidence.
The existing runner successfully replays the full screen from preserved raw rows.
The [review archive](../tools/s07/performance-experiments/results/2026-09-09-lookup-reuse/README.md)
contains both source/binary bundles, graph streams, samples, validation logs and
the removed patch. No new replay harness was built.

The host had 18 CPUs and 64 GiB RAM; initial load averages were 10.46 / 9.46 /
7.94. Our builds/tests had stopped. Other host activity remains possible; the
screen met its declared variability requirements but does not establish a
cross-platform effect or a lookup latency. Compare variants within this batch,
not this control's 4.605 s with the earlier batch's 4.344 s.

| Frozen result | SHA-256 |
| --- | --- |
| Candidate manifest | `44c48308bd4cc909b5f1adc17d88f81cbd1ccb554b8c33335209620e60b23bc6` |
| Graph report | `2dd763f412fce66b48c4caca1d67a20029b22793c521d8d1103ec26a8e9b01ae` |
| Screen report | `1c81005d55ae3a4782a21ddd7288ee75a51777135b0df704cbe7654f5263226c` |

Against historical Go medians, the retained control from this batch is 1.667 s
above the one-worker wall target and 0.428 s above the eight-worker target.
Requests remain about 2.607 GB above the historical 0.7× allocation limits;
RSS remains about 2.298 / 2.292 GB above the corresponding limits. These are
historical distances, not fresh paired Go acceptance results. All final gates
remain open. The next implementation is the contextual read/construction
boundary needed by the complete typed compact backend.
