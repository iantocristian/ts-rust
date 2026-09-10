# S07-bis: current-control parse and bind CPU attribution

Status: one exact-control Time Profiler capture and offline attribution complete.
The sample supports the selected scanner-drain and canonical-name-hash trial.
It identifies parent attachment as a bounded question for the later parse-side
step; it does **not** establish the proposed 100–300 ms construction saving, a
route to CPU parity, or a reason to remove final validation. No third-step code
or native experiment is part of this attribution.

## What was measured

The sole capture used the current acceptance control's exact normal executable,
SHA-256 `c03dd0dbe44386265dab8f23d737bd45d3eeef0bbe7d91ad0753091fe97580c2`.
Its frozen bundle is
`1c04605dcd248d78dcec1e42c4c6f82f036843a054cf5e27d81c1b3b229753b5`,
from revision `26b6c4efc507e002130330aad30f9a5496d20f1e`, with production
fingerprint `1c661958f0982af9d34dcfd562f6b2c7d6d87c664e4d7dc3fc6dac682ceefbd3`.
The binary was not rebuilt. Concurrent workspace edits do not enter its source
attribution.

The workload matched all 13,094 files, 161,740,237 loaded bytes, 19,593,488 nodes,
2,459,867 symbols, 423 parse diagnostics and 5,250 bind diagnostics. The input
manifest hash and decoded loaded-input digest matched the capture receipt and
frozen manifest. Offline verification also rehashed all 289 frozen production
source files, 13,094 workload members and 205 native-trace members.

Time Profiler exported 5,928 target-process Running samples of 1 ms each.
Native parse/bind ancestry, including the exact executable's retained physical
symbol aliases, partitions them as follows. These are **sample weights**, not
elapsed phase durations:

| Partition | Sample weight, ms |
| --- | ---: |
| Parsing | 2,796 |
| Binding, including validation/publication | 2,089 |
| Other worker work | 17 |
| Main thread and other non-worker work | 1,026 |
| Total | 5,928 |

All 55 missing-stack samples belong to the non-worker partition and remain in
the denominator. The 4,902 ms worker sum agrees with the native analyzer.
Every displayed leaf contributes exactly once to its phase's self ranking.
Physical aliases establish ancestry without manufacturing extra self samples.
The normal executable's recorded 5.872 s profiled wall time is retained in raw
stdout; it is not a latency observation for acceptance or a comparison with
earlier profiles.

## Rankings and candidate mechanisms

The largest displayed self entries are:

| Parse leaf | ms | Bind leaf | ms |
| --- | ---: | --- | ---: |
| Scanner `scan` | 160 | `DefaultHasher::write` | 102 |
| `AstView::node` | 149 | `bind_target_head` | 98 |
| Scanner `scan_identifier` | 136 | `node_kind` | 88 |
| `mi_page_free_list_extend` | 114 | `bind_each_child_target` | 68 |
| Scanner-diagnostic Drain drop glue | 82 | `bind_local_entry` closure | 63 |
| Stored-node arena `push` | 80 | `mi_page_free_list_extend` | 61 |
| Parser `scan_operation<scan>` | 77 | `bind_worker_target` | 48 |
| `finish_header` | 72 | `try_set_target_flow` | 45 |

The complete parse and bind self and inclusive rankings, group selectors,
nearest matching frames, leaf drilldowns, intersections and unions are retained
in `target/s07-bis/drain-hash/profile/phase-attribution.json` and its adjacent
`attribute_phases.py`. Inclusive groups overlap and cannot be summed as savings.

| Selected group | Phase | Inclusive ms | Displayed group self ms |
| --- | --- | ---: | ---: |
| Constructors / `new_node_before_hook` | Parse | 557 | 79 |
| Parent attachment | Parse | 330 | 57 |
| Payload insertion | Parse | 237 | 100 |
| Header finishing | Parse | 181 | 72 |
| Parse completion validation | Parse | 81 | 16 |
| Scanner-diagnostic Drain drop glue | Parse | 82 | 82 |
| Hashing with canonical-name-storage ancestry | Bind | 112 | 112 |
| All named hashing | Bind | 124 | 124 |
| Final binding validation | Bind | 92 | 0 |
| Contextual identifier check | Bind | 85 | 41 |

“Displayed group self” counts leaves whose displayed function matches the group
selector. The remainder consists of descendant leaves or a selector established
only by physical aliases; it is not a separate avoidable overhead estimate.
In particular, zero self for the validation entry does not make its graph walk
free.

The Drain group is the exact scanner-diagnostic `Option<Drain<_>>` drop symbol;
the unrelated channel-waker Drain is excluded. Its 82 ms self weight supports
examining the already-identified empty-drain creation mechanism. Stacks cannot
tell whether any individual Drain was empty, and 82 ms is not a predicted
speedup. Necessary diagnostic delivery remains part of parser behavior.

Canonical-name hashing accounts for 98 ms in `DefaultHasher::write` and 14 ms in
`hash_one<&[u8]>`, under symbol-table insertion (37 ms), lookup (35 ms), name-pool
interning (21 ms), or symbol insertion (19 ms). Another 12 ms of bind hashing
has no displayed canonical-name-storage ancestry. The retained physical aliases
are considered by the group selector, but missing inlined ancestry still limits
scope. A hash replacement retains hashing and byte-equality work; neither 112 ms
nor the broader 124 ms is a removable-work claim. See the separate hash decision
for collision-policy and semantic requirements.

## Parse-side question for the later step

Construction and parent attachment have an 887 ms distinct union in this sample.
The construction group includes 112 ms with visible allocator ancestry and
229 ms under payload insertion. Its largest leaves include stored-node arena
`push` (75 ms), allocator page initialization (70 ms), auxiliary mutation (45 ms),
text-pool insertion (29 ms), `AstView::node` (29 ms), node-reference encoding
(23 ms), and identifier-row insertion (17 ms). These weights mix necessary
allocation, compact representation construction and access checks. The visible
allocator selector is not a census of all allocation cost. The separate header
finishing group also contains necessary text/range work; it is not folded into
the constructor selector after seeing the result.

Parent attachment's leaf breakdown gives a more concrete mechanism to inspect:

| Leaf beneath parent attachment | Self weight in that group, ms |
| --- | ---: |
| Parent-attachment entry | 57 |
| `CoreParents::visit_node_slice` | 51 |
| Generated child enumeration | 46 |
| `CoreParents::visit_node` | 24 |
| `InitializationGuard::assert_inactive` | 21 |
| `LazyArena::read` | 16 |
| Auxiliary list lookup | 14 |
| Compact node decode | 12 |
| Compact auxiliary decode | 9 |
| `CoreParents::visit_list` | 9 |
| Auxiliary value lookup | 8 |
| Thread-local address lookup | 8 |
| `node_slice_read` | 7 |

This names a bounded **proof investigation**: determine which parent-fixing
reads during exclusive construction are known to refer to the mutable core,
and whether they can use that fact without repeated lazy/initialization routing.
Preserve the general path for references that do not carry that proof. The
existing child walk and parent writes are required; a sample is not evidence
that they can disappear. The profile alone does not authorize a parser-wide
facade rewrite or establish the correctness contract for a specialized path.
The source audit and the selected combined trial's result determine whether this
becomes the later candidate. No additional profiler is needed to identify this
particular question.

## Items that remain required work

Final binding validation has 92 ms inclusive weight. Its leaves include symbol
validation (35 ms), `AstView::node` (13 ms), flow validation (12 ms), flow-node
access (8 ms), flow/list decode (7 ms each), declaration-range checks (6 ms) and
name-pool byte access (4 ms). These are graph obligations, not a redundant owner
check established by the local syntax scope. This profile supplies no new proof
for removing them.

The contextual-identifier check is 85 ms inclusive: 41 ms in the entry, 19 ms in
keyword classification, 13 ms in identifier-name-role classification, 8 ms in
name access and 4 ms in text-pool byte access. The keyword and role values are
already inside 85 ms. They establish no 50–80 ms removable remainder and no
separate candidate is selected from them.

The sample therefore supports testing the named Drain/hash mechanisms while
retaining the later parent-attachment question. It does not justify summing
sampled weights into a forecast or changing any acceptance threshold. The
[paired acceptance result](S07-bis-candidate-acceptance.md) remains the current
Go-relative evidence.

## Offline replay and limits

The capture directory retains the native trace, command, stdout/stderr, capture
receipt, XML exports, original analysis helper, copied historical phase helper
and the new offline-only attribution script. The original helpers remain
byte-identical to their capture-receipt hashes. From the repository root:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 target/s07-bis/drain-hash/profile/attribute_phases.py \
  --bundle target/s07-bis/current-candidate-acceptance-2026-09-10/frozen --check
```

This verifies inputs and source/binary/trace identities, recomputes the complete
attribution, checks all denominator and leaf/group partitions, then compares
the result exactly with the retained JSON. It does not launch native code,
re-export the trace, resymbolize the binary, compile or access the network.
Exact replay passed, as did the existing native analyzer's 13 synthetic
contracts and independent checks of the displayed table totals. The adjacent
`attribution-verification.json` records those checks; they add no workload
observations.

| Artifact | SHA-256 |
| --- | --- |
| Capture receipt | `668fec87dfbd117ce4ab77bff52c003bb18b72721b79915120bedc0a15a2f884` |
| Sample XML | `a2194c5567e4d0506d2daca4c4f7b3e03148e47d20c5d0410e2864a9ed88f6ba` |
| Native summary | `75f947397acd6c44c99086aca896a4b8fb99b65dda3877b747447b2933baa3c7` |
| Phase attribution | `6728ef53f755bc8ac841f556763b691fcaafe0b5c53e483fda8ca3034851b5df` |
| Offline attribution helper | `1fdc4a4475f04aa89ab4f6d6a16c674dae55731db5a3a572cf04c9f4e29a5cc2` |

This is one one-worker profiled invocation. Normal release symbols expose no
complete source/inline information. Sampling perturbs execution; initial host
load was 5.856 / 5.235 / 4.944 on an 18-CPU host, and the receipt does not prove
continuous host inactivity or thermal isolation. Absence among 1 ms samples is
not proof that a function never runs. No confidence interval, eight-worker
imbalance result, Go comparison, phase speedup, savings forecast or performance
retention decision follows from this diagnostic.
