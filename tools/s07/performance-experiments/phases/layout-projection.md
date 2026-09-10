# Actual core occupancy and compact-storage projections

These are requested-byte models, not replacement storage or acceptance results.
The 780/880 MB category allocations below are provisional. Their margins and
misses remain useful diagnostics; they do not select a storage family before
the complete owner budget is modeled. The
[subsequent budget review](../../../../docs/S07-bis-plan-review.md#6-layout-budget-review-after-a0)
keeps typed pages in the comparison and adds tighter identifier text as a
separate candidate. The recorded matrix and its arithmetic are unchanged.
The [additive pilot review](../layout-followups/README.md) subsequently adds
mixed scalar/atomic rows and tests the source-density premise for chunk sizing;
it preserves this historical matrix and does not select a production layout.

The frozen original A0 executable supplied an **untimed** census of all 13,094
files, with every completed root retained. All files used consuming binding;
19,593,488 core nodes and the loaded-input digest matched the frozen workload.
Concurrent semantic work makes its incidental clocks unsuitable for timing.

The census observes 158 of 192 generated shapes and 529,450 file/shape pairs.
Excluding shapes whose bound payload is empty leaves 504,462 active stores.
There are 41 shapes per file at the median and 104 at the maximum. This resolves
the old merged census's 253,333 overlay copies exactly: compact syntax uses
224,855,928 bytes, and syntax plus inline binding uses 298,695,596 bytes before
capacity, directories, escape storage and other costs. The model uses the
compiled schema layouts from [CP0](../../../../docs/S07-bis-CP0.md).

## What is charged

The executable Rust sketch measures the complete page/store envelopes. Existing
`Page` directory entries cost 40 bytes. Alternatives use 16-byte `Box<[T]>`
entries for variable 1/2/4/.../N pages, or eight-byte `Box<[T; N]>` entries for
constant N-slot pages **including the first page**. The latter never silently
uses a thin pointer for a variable allocation. All payload capacity is charged.

Each active Box-page store keeps an eight-byte logical length. A 16-byte ledger
envelope holds a counter handle and tracked-page count, either per store or once
per owner. The owner variant still charges its ledger in the header store; it
requires a proof that core pages retire together and that partial construction,
drop order and observable counters remain correct. Published lazy pages keep
their existing tracking fallback. This is a projected accounting design.

The matrix compares fixed typed vectors, optional boxed stores, packed indices,
one-or-many stores, and an inline first page. It charges complete store boxes,
directory spare capacity, padding, and full replacement requests on directory
growth. Packed indices include alignment rounding, including the 15-class case.
Page and vector growth sequences are explicit proposed policies, not promises
about ordinary `Vec::push` behavior. Flat payload-vector candidates also charge
every replacement payload allocation; requested bytes do not imply a physical
copy or claim anything about in-place realloc.

## Result

Decimal MB below include the 24-byte header, bound payload capacity and all
modeled directories. “Traffic” is requests minus retained modeled bytes.
Remaining escape, runtime-ID, compatibility and owner costs are excluded and
must still be charged against the provisional 880 MB syntax-plus-binding
allocation or an explicitly revised whole-owner budget.

| Storage candidate | Retained | Requests | Traffic |
| --- | ---: | ---: | ---: |
| Existing page envelope, ordinary 2→256 growth, fixed shape vectors | 1,060.92 MB | — | — |
| Best existing 40-byte page envelope | 956.16 MB | 1,009.21 MB | 53.05 MB |
| Best variable Box pages, owner ledger | 897.67 MB | 924.14 MB | 26.46 MB |
| Best constant Box pages, owner ledger | 887.49 MB | 941.08 MB | 53.59 MB |
| Same constant pages, per-store ledgers | 895.57 MB | 949.15 MB | 53.59 MB |
| Best flat payload vectors | 936.80 MB | 1,381.62 MB | 444.82 MB |
| Scalar word classes plus indexed atomic facts | 866.46 MB | 898.88 MB | 32.42 MB |
| Atomic word classes | **830.89 MB** | **862.18 MB** | **31.30 MB** |

The best per-shape thin-page candidate uses 32-slot header pages, four-slot
payload pages, and optional one-or-many stores. It still misses the byte budget
before unknowns and entails 6.71 million modeled allocation calls. The flat
payload-vector policy alone exceeds the entire 350 MB transient ceiling. These
results show which envelopes miss the original category allocation. They do
not reject an envelope under a different, explicitly reconciled whole-owner
budget. The high allocation-call count remains a separate concern to measure
in the implementation.

## Word-class candidates for a bounded prototype

The generated compact payloads have four-byte alignment and 15 nonempty word
widths, plus zero-payload shapes. Sharing pages by width reduces the active
file/store pairs to 137,282. The existing header keeps independent open syntax
kind and concrete shape tags; its payload ordinal indexes the selected class.
The shape-to-width mapping is static, so no extra per-node locator is assumed.
Generated accessors must decode checked scalar values; this design cannot return
typed payload references by reinterpreting a word array.

The scalar variant replaces each composite facts word with an index into a
separate `AtomicU32` table. All 7,953,475 composite entries are charged: 31.81 MB
used, plus table capacity, directories, logical lengths and growth requests.
Its syntax subtotal is 792.62 MB, above the 780 MB sub-budget even though the
combined subtotal fits 880 MB.

The all-atomic variant stores `[AtomicU32; N]` rows and keeps facts inline. The
compiler confirms the same size/alignment as scalar word arrays. Private writes
can use `get_mut`; shared scalar getters require explicit loads, while existing
facts operations keep their required atomic ordering. This avoids the separate
facts table, but relaxed loads must not be assumed equivalent to plain loads for
compiler optimization or CPU cost.

Its best policy uses 32-slot thin header pages, eight-slot thin payload pages,
and optional one-or-many stores. It charges 475.18 MB header capacity, 315.97 MB
payload capacity, and 39.73 MB directories/lengths/ledger. Assigning all shared
metadata and spare to syntax gives **757.05 MB syntax + 73.84 MB binding**. That
leaves 22.95 MB syntax margin and 49.11 MB combined margin before unknowns.
There are still 3.47 million modeled allocation calls. Default-constructed spare
occupies a 22.22 MB storage span; this is neither measured initialization traffic
nor a promise that padding is written or that no stack temporary is created.

Carry two existing-grid policies into the pilot, both with 32-slot header pages:

| Payload page size | Retained | Requests | Modeled allocation calls | Syntax subtotal |
| --- | ---: | ---: | ---: | ---: |
| 8 rows | 830.89 MB | 862.18 MB | 3.47 million | 757.05 MB |
| 16 rows | 840.04 MB | 858.43 MB | 2.25 million | 766.20 MB |

The 16-row policy spends 9.16 MB more retained storage for 1.22 million fewer
allocation calls and slightly fewer replacement requests. It leaves only
13.80 MB of syntax margin, so neither policy can be selected from the model
alone. Measure native allocation calls, elapsed CPU and retained/peak RSS for
both; requested-byte totals do not predict allocator rounding or RSS.

After the whole-owner and identifier-text models support a candidate selection,
proceed only to a small safe accessor/storage pilot: representative identifier,
token, binary-expression and function payloads; checked shape/class/ordinal
lookups; private writes and shared reads; facts operations; page growth and
partial-construction/drop tests. Measure actual hot-read/write and traversal
costs against the retained A0-b control before broad generator/accessor migration.
The existing full-range link/text escape sketch must be integrated and priced,
and runtime-ID storage must receive its own explicit budget.

This does **not** resolve the baseline's 257.48 MB unattributed requested live
bytes, allocator overhead/RSS, or scanner/list/string/symbol/flow traffic. The
31.30 MB directory traffic would leave 318.70 MB of the independent 350 MB
traffic ceiling for those other sites; it does not show they fit. Feasibility
remains unproved and no tracker metric is emitted.

## Artifacts and replay

`core-shapes.ndjson.gz` is the deterministic compressed raw per-file census.
`core-shapes-provenance.json.gz` preserves the exact raw child stdout/stderr,
capture record and immutable build manifest. `layout-projection.json.gz` contains
the full candidate matrix, compiler/layout hashes and source/input provenance.
The census comes from original A0's immutable build, not the newer checkout's
launch fingerprint. B's separate phase attribution and exact captured helpers
are archived with [the B results](../results/2026-09-08/a0b-README.md).

The following command recompiles the small layout sketch and replays every
recorded candidate from the committed raw census, without a workspace build,
download, benchmark or timing claim:

```sh
python3 -m unittest discover -s tools/s07/performance-experiments/phases -p 'test_*.py' -v
```

To regenerate with the local immutable build/capture present:

```sh
python3 tools/s07/performance-experiments/phases/project_layout.py \
  --capture target/s07-bis/a0-core-shapes-host \
  --build target/s07-bis/a0-phase-build \
  --build-sha 710c3840979f1226b4ff5c745cf59c181d5e55e63ac57bef9e7936be66612d63
```

The human-readable expanded output is also written under
`target/s07-bis-directory-layout/layout-projection.json`. Eight projection tests
cover capacity boundaries, full realloc accounting, raw provenance, compiled
envelopes, ledger arithmetic, packed-root padding, the complete facts-table
removal and every recorded candidate. The five phase contract tests remain
separate. An independent review checked both ledger scopes and both word-row
variants; its packed-root padding finding is fixed and regression-tested.
