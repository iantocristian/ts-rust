# Whole-owner layout components

This additive diagnostic prices the binding and auxiliary portions that were
previously represented by provisional 440 MB and 140 MB allocations. It uses
compiled safe Rust records and the [physical owner census](../owner-census/README.md),
including capacities, obsolete declaration backings, empty roots, directory
replacement requests, hash buckets and sparse runtime-ID storage. It does not
implement compiler storage, measure CPU/RSS or certify an S07 gate.

`owner-layout.json.gz` contains the complete policy grid. Generate a readable
copy, without rerunning the workload:

```sh
python3 tools/s07/performance-experiments/owner-layout/model.py \
  --output target/s07-bis/owner-layout/replayed.json
python3 -m unittest discover \
  -s tools/s07/performance-experiments/owner-layout -p test_model.py
```

The model replays the durable census before consuming its 13,094 records. It
pins capture manifest `5a2aeab597be444b5906b675bc7ce2d20c53605c9ab6b636bfe6d45d4d43d339`
and raw record SHA `835823ecaba55715ac69ee2efaafdb71acc47a24b52bdd6b73fd55f1f2ddd10d`.
The census observes the frozen A0-b build with additive access-only bridges; it
is separate from ordinary benchmark evidence. The census documentation records
its metadata finalization after a sandbox `sysctl` failure. No child was rerun
by this model. Compiler identity, binary hash and every included source are
recorded; the model checks source stability across compilation and generation.

## Physical layouts and contracts

The compiled sketches preserve independent fields rather than assuming the
proposed widths:

| Record | Bytes | Included fields |
| --- | ---: | --- |
| Symbol, four-byte name | 56 | Flags/check flags, name, full 16-byte declaration slice, five links, `AtomicU64` ID |
| Symbol, eight-byte name | 64 | Same fields with eight-byte name |
| Symbol without inline ID | 48 / 52 | Four-/eight-byte names; charged side ID table is additional |
| Declaration slice | 16 | Backing identity, start, length, capacity |
| Inline flow | 28 | Open flags, independent union tag, largest inline synthetic payload, antecedent/list links |
| Outlined flow | 20 | Open flags, independent tag, data slot, antecedent/list links |
| Packed flow | 16 | Open flags, separately tagged data word, antecedent/list links |
| Switch / reduce payload | 12 / 8 | Full local link words and signed switch endpoints |
| Flow-list cell | 8 | Flow and next links |
| Node-list header | 24 | Signed position/end, full 12-byte slice, modifier flags |
| Indirect list / slice descriptor | 16 / 12 | Separate header identity and backing/range descriptor |
| Box backing / local cell | 16 / 4 | Stable physical backing descriptor and nullable local word |
| Scalar table header / bucket | 24 / 8 | Box+length; complete `Option<Entry>` bucket uses nonzero key niche |
| Sparse runtime table / bucket | 40 / 16 | Per-owner mutex/table header; nonzero slot and `AtomicU64` |
| Packed-flow escape table / bucket | 24 / 24 | Full tagged data keyed by flow slot, including foreign identities |
| Typed store | 32 | Page vector, checked `u32` used length, independent arena ID |
| Owner ledger | 16 | One counter handle and page count, shared by modeled stores |

Four-byte symbol-name keys require a canonical byte-name interner. They are
**not** interchangeable with the text prototype's source-suffix words: equal
bytes at different source endpoints can have different words. Canonicalization,
malformed-byte equality, generated names, overflow/foreign handling, retention,
interner capacity and construction traffic still require pricing and the
ADR0007 API audit. The scalar map's collision/nil-value tests establish its
bounded representation, not a complete replacement for the current map API.

The packed flow word assigns two independent bits to Ast/Switch/Reduce and
uses a 30-bit local ordinal; nil and an escape marker remain distinct. Full
flags are untouched. Tests cover boundary ordinals, full-width links, foreign
owners, signed synthetic ranges and exact escape preservation. All synthetic
rows remain charged. The census has variant counts, but does not classify all
flow link domains or mutation histories: **zero packed-flow escapes is not an
observation**. A complete all-flows escape-table bound is reported alongside
that conditional subtotal. General compact links still require the complete
owner/foreign/lazy validation and retention design.

## Results and limits

Decimal MB below use constant 16-record thin pages and explicit table growth
from two buckets at at most 7/8 occupancy. Every replacement bucket allocation
and page-directory array is counted. The endpoint-length map policy assumes
that endpoint occupancy captures required growth; the report also prices the
old public-capacity target. Neither policy claims to reproduce `HashMap`
bucket layout, transient deletion history or CPU cost.

| Binding candidate | Live MB | Modeled requested MB | Modeled allocation calls |
| --- | ---: | ---: | ---: |
| Inline symbol IDs, inline 28-byte flow | 345.746 | 368.273 | 4,422,072 |
| Inline symbol IDs, outlined 20-byte flow | 322.621 | 345.151 | 4,426,815 |
| Sparse symbol IDs, outlined 20-byte flow | 302.671 | 325.201 | 4,426,922 |
| Sparse symbol IDs, packed 16-byte flow; escapes additional | 290.748 | 313.278 | 4,426,922 |

These rows use four-byte canonical symbol names, full 16-byte declaration slice
headers, four-byte cells, full table buckets and all observed synthetic payloads.
The packed row's all-flows escape bound adds 115.701 MB live and 230.774 MB in
modeled requests; no distribution between zero and that bound is inferred.
Outlined synthetic replacement history is also unmeasured. The 117 assigned
symbol IDs make sparse storage plausible, but do not establish its concurrent
getter semantics or CPU cost.

The same outlined/sparse candidate with 32-record pages costs 313.572 MB live,
332.848 MB in modeled requests and 4,095,138 calls. This is a useful byte/call
tradeoff pair for a bounded prototype. Millions of calls remain: modeled
requested bytes do not predict allocator usable bytes, CPU time or RSS.

Auxiliary storage exposes a different constraint:

| List representation, same 16-record policy | Live MB | Modeled requested MB |
| --- | ---: | ---: |
| Direct 24-byte headers, per-kind arena identities | 177.803 | 181.838 |
| Direct headers, original global auxiliary slots plus locator | 237.276 | 245.703 |
| Indirect 16-byte headers plus distinct descriptors, per-kind identities | 190.345 | 196.256 |

Each row preserves all 3,114,989 physical boxed backings and 6,047,867 initialized
cells. They include unused capacity, not merely reachable edges. Add **21.689 MB**
of unchanged owned text/metadata/source-map children; private BTree layout remains
unreported. Indirection increases bytes because there are 3,125,958 distinct
slice descriptors. Its interning traffic would be additional. The separate
list-backing pilot investigates replacing physical boxes; this control model
does not assume its result.

Per-kind arena IDs can avoid the eight-byte locator, but require a new checked
owner dispatcher. Each modeled store already charges its arena ID in its root.
The prototype must preserve header/backing aliasing, reject foreign and
unpublished IDs, retain the correct complete owner and maintain lazy/failure
behavior. This is a conditional design alternative, not a claim that existing
storage routing can remain unchanged.

## Joining a complete owner budget

These modeled rows are components, not established headroom. Add the selected
node/header/payload envelope from the separately reviewed four-byte-text row
[projection and its provenance](../layout-followups/text-rows-result.json.gz)
([generator](../layout-followups/text_rows.py)), then the appropriate auxiliary children, source and non-source
text backings/refcount headers, parser/binder temporaries, remaining owner data
and full-domain escape costs. Do not use the earlier eight-byte-text envelope
or subtract the identifier used-byte saving as if it were a measured page
capacity saving. The current four-byte-text projection keeps both typed pages
and mixed rows as candidates.

Two often omitted components are separately reported:

- The observed 2,207 core runtime IDs cost **0.584 MB live / 0.631 MB requests**
  in a sparse table, including a mutex/table header in every owner. This is
  additional to the node layout; getter/identity/concurrency behavior is unproved.
- The actual text prototype's owner type compiles to **112 bytes**, or
  **1,466,528 bytes** across all owners even with empty fallback pools. It
  contains the inline `Arc` handle, two vector headers and the extended-map
  header. The heap refcount header beside the source bytes, pool allocations
  and canonical symbol-name interner are additional.

The binding group charges one owner ledger; auxiliary stores share it. A node
projection that already charges the same owner ledger must not add it twice.
Existing source records and retained owner roots require the same reconciliation.

The old census's 257.480 MB unexplained native live residual is not replaced by
a 50 MB allowance or presumed eliminated. Parser/scanner buffers, temporary
vectors before boxing, map construction history, obsolete synthetic payloads,
lazy/fallback graphs and allocator behavior remain real obligations. The native
identity remains `requested = end_live - start_live + transient_traffic`; the
page/table request columns above cover only named modeled construction sites.
No whole-owner memory or CPU feasibility result follows from these subtotals.
