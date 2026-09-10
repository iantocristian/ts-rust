# S07-bis: first recorded access workload

Date: 2026-09-08. Diagnostic observation checkpoint; no production storage change
and no new CPU, allocation or RSS result.

The full frozen workload was recorded and verified: **300,052,431 records across
13,094 files**, with every retained per-file graph matching the original accepted
CP1 executable. The observation preserves the original input order/options and
parse → export → bind → retain sequence. All files used exclusive binding; none
used the fallback. The inserted state export changes cache behavior, so this run
cannot measure production speed.

This completes the first milestone of the [access-trace plan](S07-bis-access-trace.md).
It does not yet support ranking typed payload pages against mixed plain/atomic
word rows. Both remain candidates; page-256 remains the leading list policy.

## What was captured and checked

| Full workload observation | Result |
| --- | ---: |
| Loaded source bytes | 161,740,237 |
| Physical nodes | 19,593,488 |
| Symbols | 2,459,867 |
| Parse / bind diagnostics | 423 / 5,250 |
| Retained per-file graph matches | 13,094 / 13,094 |
| State export records, separate from binder operations | 119,788,388 |
| Framed payload / decoded stream | 16,023,693,771 / 16,024,488,815 bytes |
| Compressed trace | 1,767,600,066 bytes |

The staged exporter covers all 192 physical core payload shapes and 494 stored
fields, including unreachable slots, list backing identities/subranges and
selected raw text bytes. It observes existing caches without assigning runtime
IDs, computing subtree facts or materializing lazy nodes. The
[state registry](../tools/s07/performance-experiments/access-trace/state-registry.json)
names omissions: backing-sharing identity for text, lazy/imported payloads,
some source metadata/cache contents and parse-time allocation interleaving.

The binder hooks record actual successful operations with one evaluation of
the original expression. Counts are source-level events, not hardware accesses,
payload-field consumption or CPU coverage. Resolving a list does not count as
reading every element.

| Named operation | Events |
| --- | ---: |
| `Binder::n` lookup | 122,786,331 |
| Named dispatch kind reads | 19,207,597 |
| Named dispatch/state flag reads | 19,588,772 |
| Successful narrow flag writes | 253,532 |
| `syntax_slice` resolution | 541,948 |
| `syntax_node` indexed access | 7,476,065 |
| `syntax_nodes` resolution | 178,200 |
| Successful node flow assignment | 10,126,845 |

All named node observations map to initial physical headers. Counts for
uninstrumented operations are **unavailable**, not zero. The verifier checks
framing, integrity, phases, physical ordering/counts, blob continuity and named
kind/flag values and updates. It does not replay list values, reconstruct complete
binding state or prove every payload field read. Recording a flow ID assignment
does not reconstruct the referenced flow graph.

## Validation and provenance

The recorder's 30 Python tests pass, including 17 hook scenarios comparing
original/staged evaluation and drop behavior in debug and release. Six staged
native AST tests pass. The independent native verifier passes Clippy with
warnings denied and the declared Rust 1.96 check. Its frozen executable passes
109 preserved corpus attempts in 19 test methods against the Python reference;
the complete 23,574,043-record sizing report is also byte-identical between them.
These are bounded equivalence checks, not a proof of identical acceptance for
all possible inputs. Native configuration integers and JSON sizes have narrower
limits that accommodate this capture's declared configuration.

The full verifier requires successful child exit, complete single-member gzip
with CRC/trailing checks, the expected compressed hash, matching decoded hashes,
block/footer hashes, and declared full-workload totals. A separate receipt
composition checks every bundle inventory and reconciles each trace file's
work/diagnostics/publication path with its accepted-control graph. It does not
decode the trace again or add semantic coverage.

Two independent read-only reviews found no defect affecting the completed
verification. They identified wrapper hardening for checking configuration
stability after execution and preserving process-launch failures. Those fixes
are tested separately; the original build, capture and verification receipts
remain unchanged.

The first three build attempts failed on isolated test-fixture/target selection
or a missing direct driver dependency. The first sizing attempt completed its
native recording and graph check but the producer incorrectly sent its subset
through the full-workload validator. These failures remain preserved and failed.
The corrected sizing path derives selected recipes from the validated full
parent. The successful sizing and full recordings retained all declared events
within the original 32 GiB payload / 3 GiB compressed limits.

The full recording manifest intentionally still says **pending verification**:
it was sealed before the separate native verification. The composed result links
these two immutable records; it does not rewrite the first one's history.

| Artifact under `target/s07-bis/` | Manifest SHA-256 |
| --- | --- |
| `access-trace-build-4` | `0b585e058ed95e73e77d1d230d92e17c48e80291d7054192d84527beb2baa2fe` |
| `access-trace-full-1` | `d37189f0581fa11d52978b7f05d581f4886919c3bb3d0889ef69c7d8923c7c9c` |
| `native-verifier-build-1` | `1e8bd155938150d3f478cb43e6eed8a7e6f9a9fdf7adb196eecfa95728249732` |
| `native-verifier-full-1` | `ddc6a4c990ca6b2f5e39f2e1652fbbc0c1d8da49b677b29c92e3368c56b5c658` |

Full raw gzip SHA-256:
`2292410213c8e42269131cb06a131199751f51e188c11bc8eeeb59678216d35b`.
Decoded stream SHA-256:
`abaa36526892c9ec422115bf55aa3633f8f542bf61f9e09a218d032e4b156efb`.
Loaded input SHA-256:
`d4ff2ad3b0d590babb596522a7105d961dbb3c4f2ebc987488d88b7ebd8e0c88`.

The [review bundle](../tools/s07/performance-experiments/results/2026-09-08-access-trace/README.md)
contains manifests, source/tool snapshots, binaries, verification/graph reports,
test receipts and failed attempts. Large raw workload traces remain in the
immutable local bundles and are explicitly excluded from Git; the review archive
alone cannot rerun full trace verification. The recorder, consumer and composition
commands are documented beside the tools and review bundle.

## Next bounded slice

Superseded on 2026-09-09: the field-level stages below are deferred by the
[revised sequence](S07-bis-performance-plan.md#1-decision). Use the existing counts
to inform one bounded lookup-reuse candidate, then integrate typed compact rows.
Do not expand the observer merely to decide between the modeled row families.

The highest `Binder::n` shape counts are Identifier (36,264,661), PropertyAccess
(19,393,029), Call (8,602,656), Binary (4,574,170), ExpressionStatement (3,993,446)
and Token (3,841,111). These guide the next hook inventory; they do not identify
which fields the resolved values supplied. In particular, the shared `n` hook
does not identify individual callers, and helper/visitor `AstView` reads bypass
it. The previous nine-shape population share is not CPU coverage.

1. Give the 17 selected binder `payload!` sites and the explicit diagnostic
   payload clone distinct snapshot origins. Associate later field consumption
   and continuation transfers with those temporary identities, not with a
   global last-read node. Preserve short-circuiting. A logical clone event is
   not a measurement of the machine bytes the optimizer copied.
2. Capture child extraction and later consumption for the nine-shape slice:
   generated child fields, explicit SourceFile/Block/ExpressionStatement paths,
   immediate-child descriptors and actual list elements. Keep absent fields,
   functions-first double passes and early exits distinct. An emitted callback
   alone misses earlier absent-field checks.
3. Add the named borrowed expression/name/list/text helper paths needed by that
   comparison. The binder's `payload!` clones; the similarly named AST helper
   macro borrows. Identifier text access also bypasses `n` and must distinguish
   a borrowed byte view from a cloned string handle.
4. Recapture with the expanded registry, then replay supported operation
   families over current storage, typed rows and mixed rows. Keep remaining
   operations/storage explicit and charge their working set. Compare values,
   order and narrow writes before measuring access cost; confirm any selected
   layout on the integrated pipeline.

## Distance to acceptance: unchanged

These are the last retained production measurements from the preceding CP1
screen, compared with **historical** Go medians. This trace supplies no fresh
paired performance evidence. All final CPU and memory gates remain open.

| Metric | One worker | Eight workers |
| --- | ---: | ---: |
| Retained Rust wall | 4.344 s | 0.971 s |
| Historical Go wall | 2.938 s | 0.634 s |
| Wall distance / ratio | +1.406 s / 1.479× | +0.337 s / 1.531× |
| Rust requested bytes | 4.643 GB | 4.643 GB |
| Historical 0.7× allocation limit | 2.035 GB | 2.036 GB |
| Allocation above limit | 2.607 GB | 2.607 GB |
| Rust RSS | 4.507 GB | 4.509 GB |
| Historical 0.7× RSS limit | 2.209 GB | 2.217 GB |
| RSS above limit | 2.298 GB | 2.292 GB |

GB is decimal here. A production candidate still needs fresh paired Rust/Go
measurements, full parity and ownership checks before promotion.
