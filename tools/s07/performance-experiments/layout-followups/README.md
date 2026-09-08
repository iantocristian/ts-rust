# Follow-up layout diagnostics

These additive diagnostics review the proposed pilot after A0-b. They do not
modify the historical page matrix, compiler implementation or captured timing
results. The whole-owner budget remains unfinished and no tracker metrics are
emitted.

## Mixed rows: one inline atomic facts field

`CompositeRow<N>` holds one `AtomicU32` and `N` plain `u32` words. Here `N`
excludes the facts word already charged in the old payload width. Non-composite
`PlainRow<N>` has no facts field. Generated concrete-shape tags select the
appropriate family and width; shape identity is not inferred from syntax kind.
The compiled sketch proves the expected sizes/alignment and that ordinary
fields can be borrowed as `&u32` while facts retain atomic operations. It does
not implement the complete AST access contract or measure CPU.

All-atomic and mixed rows both contain **298,695,596 used payload bytes**.
Mixed rows have **23 classes / 177,234 active file-class pairs**, compared with
**15 / 137,282** for all-atomic. Splitting classes changes rounding, directories
and allocation calls even though each used row has the same byte size.

The following matched policies retain 32-slot header pages and the same
optional one-or-many directory policy. Values are modeled, decimal MB:

| Payload policy | Retained | Requests | Allocation calls |
| --- | ---: | ---: | ---: |
| All-atomic, 8 rows | 830.885 | 862.181 | 3,472,288 |
| Mixed, 8 rows | 835.907 | 866.745 | 3,575,251 |
| All-atomic, 16 rows | 840.041 | 858.431 | 2,253,665 |
| Mixed, 16 rows | 849.147 | 867.197 | 2,339,578 |

Mixed rows add **5.022 / 9.106 MB** retained and **102,963 / 85,913** allocation
calls at the two page sizes. Their used rows contain 7,953,475 atomic facts
words, versus 74,673,899 atomic words in the all-atomic design. These are
structural counts, not measured load counts or predicted execution time.
Four-byte identifier text is a separate, unapplied variant; escape pools,
runtime identities and other missing owner storage remain unpriced.

Shared atomic loads constrain optimization, but the claim that they can never
be merged or kept in registers is too absolute. LLVM permits some CSE/DSE for
Monotonic operations and documents additional special cases; private exclusive
access may also use `get_mut`. Compare actual generated accessors and traversal
before claiming a CPU penalty or benefit. See the
[LLVM atomic guide](https://llvm.org/docs/Atomics.html#monotonic).

Reproduce the compiled layout and recorded matrix with:

```sh
python3 tools/s07/performance-experiments/layout-followups/mixed_rows.py
python3 -m unittest discover -s tools/s07/performance-experiments/layout-followups -p test_mixed_rows.py -v
```

The compressed result records source/input/compiler provenance. Seven Python
tests and two compiled Rust tests cover layout, atomic placement, unsupported
widths, census/model consistency and replay. Independent review found no
arithmetic error in this additive model. Actual allocation, initialization,
drop/counter behavior, CPU and RSS remain implementation questions.

## Source-sized initial chunks

The global ratio is **8.2548 source bytes per physical node**. It is not a
per-file bound. Reading the frozen manifest's input sizes and the independently
captured per-file core counts gives:

| File-weighted statistic | Source bytes / node |
| --- | ---: |
| 5th percentile | 5.895 |
| Median | 8.489 |
| 95th percentile | 31.469 |

Using the global ratio directly to predict each file's node count undershoots
6,042 files and exceeds twice the actual count for 1,635 files. Four empty
inputs still have nodes. A source-size heuristic therefore needs a minimum,
a cap and an explicitly tested growth policy. These statistics do not show
that source-sized chunks lose; they show that the aggregate ratio alone does
not price the initial allocation or its slack.

```sh
python3 tools/s07/performance-experiments/layout-followups/source_density.py
```

`source-density.json` records the complete distribution, algorithm, script hash
and archived source/census identities. This is arithmetic on previously verified
input-size metadata, not a new runtime capture. The current shape census lacks
allocation order. Exact next-fit chunk counts, nested-list interleaving and
locality need an order trace or implementation; volume and conservative padding
bounds can be modeled without one.

## What the allocator and list evidence establishes

The proposed 3.47/2.25 million counts include page, directory and store
allocations. Today's 156,916 core-node pages alone are an incomplete comparison:
the archived source scopes also report **3,149,779 payload-box allocations** and
**191,769 core-node allocation/reallocation calls**. Those two regions already
contain 3,341,548 calls. Their scope is not identical to the proposed envelope,
so neither a 15–20× increase nor an exact net change follows from these counts.

Existing one-worker CPU traces attribute approximately 0.10 s of parse sampled
CPU and 0.09 s of bind sampled CPU to named mimalloc leaves in the truncated
top-200 rankings. This describes existing allocator work, not a marginal
per-call cost or 0.1–0.2 s of predicted additional work. Replacement allocation
size, initialization, cache behavior and eliminated allocations all matter.

There are **3,114,989 node-slice backing records**, with 3,114,897 nonempty
backings retaining 48,382,936 bytes. The node-slice compaction/auxiliary scope
requests **178,788,096 bytes**, including auxiliary growth; initial parser Vec
construction occurs outside that scope. The **630,066,971-byte unclassified
parse remainder** could include that construction, but is not a measurement
of list traffic. Core-node allocation alone requests 1,692,598,400 bytes, so
lists are not the largest measured parse-request scope. Their contribution to
the 464 MB parse freed/superseded total needs separate attribution.

List edges belong in the representative pilot. A single append-only edge
buffer does not automatically give each recursively parsed list a contiguous
slice: inner parameter/argument/body lists interleave with outer construction.
Price staging, segment chains, reservations or completion compaction explicitly.
Preserve distinct list/backing identities, shared slices, nil/allocated-empty/
missing lists, locations, modifiers, cycles and foreign references.

Source scopes and the physical census are in the
[memory archive](../../memory-profile/results/2026-09-08/raw-captures.tar.xz);
the allocator samples are in the
[CPU archive directory](../../cpu-profile/results/2026-09-08/README.md).

## Residual memory and safe chunk scope

The **257,480,101-byte native-live residual** is already after adding
11,740,848 bytes of Arc-header estimates. Inline Vec/Arc fields and known backing
capacities are included through their containing records; page directories
alone account for 32,606,440 bytes. Hash bucket/control layouts, BTree capacity
and other omitted storage remain unresolved. This does not establish that the
residual is mostly map controls and headers, or that those bytes disappear with
the binding maps. It also does not make 257 MB a permanent cost of every future
layout. Remeasure and reconcile the changed owner instead of deleting or blindly
carrying forward the residual.

Add chunk allocation as a policy experiment. Under the safe-Rust constraint,
homogeneous encoded-word backing can use checked offsets/value accessors;
separately typed arenas preserve typed references. Arbitrary typed payloads
cannot simply be reinterpreted from shared byte chunks. A different allocator
dependency or unsafe mechanism is a separate design decision, not assumed by
this pilot. Charge descriptors, row/chunk mapping, alignment/tails, initialization,
cross-chunk handling, full-range escapes, partial-construction/drop accounting
and the published-lazy fallback. Compare complete replaced storage regions and
observed calls/requests/RSS as well as CPU before promotion.
