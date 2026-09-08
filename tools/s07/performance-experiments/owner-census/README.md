# Untimed physical owner census after A0-b

This additive observer resolves storage counts for the whole-owner model. It
stages the immutable A0-b source snapshot, appends feature-gated read-only
accessors in that copy, then parses and binds the complete frozen workload once.
It changes no compiler checkout file, historical phase capture, tracker metric
or gate. It measures no elapsed CPU, allocation traffic or RSS.

The single native capture matches 13,094 files, 161,740,237 loaded bytes,
19,593,488 core nodes, 2,459,867 symbols, 423 parse diagnostics, 5,250 bind
diagnostics and the independently frozen input/options digest
`d4ff2ad3b0d590babb596522a7105d961dbb3c4f2ebc987488d88b7ebd8e0c88`.
Every file takes A0-b's consuming binding path. All completed owners stay alive
until every census record has been written. These are physical owned records,
including records no longer reachable from AST edges; the observer does not
materialize lazy JSDoc or assign runtime IDs.

## Observations

| Observation | Complete workload |
| --- | ---: |
| Assigned core-node runtime IDs | 2,207 across 399 files |
| Assigned symbol runtime IDs | 117 |
| Identifier rows | 6,792,761 |
| PrivateIdentifier rows | 2,463 |
| Rows eligible for source-suffix text at this endpoint | 6,795,224 |
| Fallback text rows | 0 |
| Selected identifier byte occurrences | 68,819,262 |
| Physical syntax node-slice backings, including empty | 3,114,989 |
| Physical syntax edge slots | 6,047,867 |
| Physical declaration backings | 2,446,623 |
| Physical declaration backing slots | 2,454,685 |
| Declaration backings not referenced by physical symbol declarations | 5,442 |
| Symbol-table records | 644,333 |
| Flow records: AST / no payload | 1,788,477 / 1,145,538 |
| Flow records: reduce-label / switch-clause | 1,988 / 14,556 |

The suffix predicate checks a nonnegative node end, checked subtraction of the
selected text byte length, source bounds, exact byte equality and a length no
greater than `u32::MAX >> 1`. It neither decodes bytes nor uses node start.
Zero observed fallback rows only describes these completed workload files.
Constructed identifiers, cooked text, foreign owners, synthetic positions,
observations before range assignment and later range mutation still need the
text pilot's explicit fallback contract. The byte-occurrence total is not a
backing-allocation measurement or removable source-storage budget. Unique
fallback values are deduplicated by selected bytes within each file only.

All observed lazy reserved/initialized node/auxiliary slot counts are zero and
remain unchanged across the observer. Runtime IDs are read with the existing
non-assigning getters. Runtime-ID shape histograms preserve the actual nonzero
consumers for a sparse storage model. Overlay node-map length is zero in every
completed file, so the core census covers every physical node in this capture.

## Per-file schema

`owners.ndjson.gz` contains one version-1 object in frozen input order. Each has:

- `index`, `bound_in_place`, `core_shapes`, `node_runtime_ids_by_shape` and
  `symbol_runtime_ids_assigned`.
- `arenas.{core_nodes,core_aux,symbols,tables,declarations,flows,flow_lists}`:
  actual logical `len`, slot `capacity`, `pages`, `directory_capacity` and
  `element_bytes`. Widths remain widths in aggregate summaries; other fields sum.
- `aux_variants`, `source_metadata_variants`, `text_slice_elements` and
  `metadata_text_slice_elements`, so changing lists does not erase other aux data.
- `node_lists.exposed_length_histogram`, `distinct_descriptors` and nil, missing,
  allocated-empty and shifted-slice counts. Descriptors deduplicate the complete
  `(backing_id, start, len)` tuple, preserving distinct allocated-empty identities.
- `node_backings.physical_length_histogram`, `nonnull_elements` and
  `physical_lengths_in_aux_order`. The array includes every physical `Nodes`
  backing, including empty and unreachable backings. It is auxiliary allocation /
  completion order, **not** initial parser Vec capacity or nested-list start order.
- `declarations.physical_length_histogram`, physical nonnull elements, backings
  not referenced by physical symbols, distinct symbol slice descriptors and
  `symbol_length_capacity_histogram`. Physical backing lengths include nil tail
  capacity and obsolete allocations; exposed symbol slices are counted separately.
- `tables.length_capacity_histogram`, with each public table length/capacity pair;
  `flow_data.{None,Ast,SwitchClause,ReduceLabel}`; current binding-map and flow-slot
  capacities; and before/after lazy slot counts.
- `identifiers`: physical shapes, predicate classes, selected/suffix/fallback byte
  occurrences, per-file unique fallback selected bytes and fallback length bins.

Histogram keys are decimal lengths or `len/capacity` pairs; values are
multiplicities. Table capacity is the public guaranteed-entry capacity. Private
hash buckets/controls, allocator rounding and overhead are not observed here.
The raw order array supports replaying completed backing sizes; it cannot
reconstruct Vec growth or nested parser construction without additional tracing.

## Build, capture and replay

All Cargo registry versions/checksums come from frozen A0-b's lock. The standalone
crate uses the pinned compiler, explicit release profile and actual Cargo-reported
executable; caller Cargo/Rustup homes and offline configuration are preserved.
The stage retains workspace package/lint settings and narrows workspace members
to the source closure actually frozen by A0-b. The patch, copied tools,
configuration, compiler, original/staged sources and executable are hash-bound.
Builds and the untimed child take the timing runner's exclusion lock.

```sh
python3 -m unittest discover -s tools/s07/performance-experiments/owner-census -p 'test_*.py' -v
python3 tools/s07/performance-experiments/owner-census/probe.py build \
  --output target/s07-bis/owner-census-build-NEW
python3 tools/s07/performance-experiments/owner-census/probe.py capture \
  --build target/s07-bis/owner-census-build-NEW --build-sha PRINTED_SHA \
  --output target/s07-bis/owner-census-capture-NEW
python3 tools/s07/performance-experiments/owner-census/replay.py
```

The last command verifies the durable compressed package without compiling,
reading workload files or executing a native child. Seven protocol tests reject
missing/duplicate/reordered owners, changed work/digest, invalid runtime counts,
missing backing order, incorrect histogram/capacity totals and changed lazy
state. Two compiled Rust contracts cover raw-byte suffix bounds, unattached
physical backing/list records, stable pre/post runtime-ID values and uninitialized
lazy storage. Package tests additionally reject missing/changed artifacts and
replaced build/test logs even if the outer package checksum is updated. Receipt
tests reject nonzero/boolean exit statuses, missing recovery explanations,
changed raw/tool identities and missing or changed JSON-helper source closure;
the finalizer itself is covered by a failed-receipt test.

The native child succeeded once. Its first producer reached the metadata step,
where sandbox policy blocked the reused `sysctl hw.memsize` query. The original
raw output and capture-tool snapshots were retained. Metadata was finalized from
those unchanged bytes with portable host identification; the finalizer also
corrected aggregate element-width presentation. No second workload child ran.
The receipt explicitly records that the first attempt had no persisted exit
receipt. Build, capture and finalizer helper snapshots are distinct, so later
formatting or producer fixes are not presented as the originally executed code.

An independent review then tightened archive replay and future finalization.
The **version-2 package envelope** binds the active JSON helper and receipt
validator, checks retained build/test logs against the original build inventory,
and validates receipt/build/raw/tool relationships. Future finalization requires
an integer zero exit status or an explicit, nonempty explanation for an unknown
status; nonzero exits cannot be sealed. The original capture remains classified
as `documented_unknown_exit_recovery`, not a newly recorded zero exit status.
The future producer persists the actual return code before rejecting a failed
child, including a null raw hash when no raw output exists. Recovery checks that
receipt before reading output, so neither complete valid output nor missing
output can turn a known failure into an unknown exit. Both cases are exercised
through the capture/finalizer path without executing a native child.

The version-1 envelope and replay receipt remain in `prior-envelope.json` and
`prior-replay.json`. `validation-tools.tar.xz` separately stores the stricter
validator/finalizer/packager sources and the preceding replayer. The original
`owners.ndjson.gz`, `provenance.json.gz`, `tools.tar.xz`, build manifest and capture
manifest are byte-for-byte unchanged. This revision executed archive-only checks;
it did not rerun or revise the native observation.

## Durable artifacts

[The result package](results/2026-09-08/manifest.json) inventories compressed raw
records, provenance and every archived tool/patch member. It contains no executable
or workload source bytes. `replay.json` records successful archive-only validation.

- A0-b manifest: `124956f668540814b95e6f677bdeae98bb2cad4f7586d3cb87f869e5c5af118f`
- Build manifest: `9577741534ffebfdfeac27b4d5d35597fc771b8bb007b3de2d3ab433b780e45c`
- Capture manifest: `5a2aeab597be444b5906b675bc7ce2d20c53605c9ab6b636bfe6d45d4d43d339`
- Raw census: `835823ecaba55715ac69ee2efaafdb71acc47a24b52bdd6b73fd55f1f2ddd10d`

The existing syntax capacities agree exactly with the earlier physical census.
This report supplies the additional per-file distributions and field occupancy;
it does not resolve the remaining live-byte residual or prove the replacement
whole-owner budget.
