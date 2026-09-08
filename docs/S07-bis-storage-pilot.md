# S07-bis: physical owner census and storage construction pilot

Date: 2026-09-08. This checkpoint continues the
[performance plan](S07-bis-performance-plan.md) from the retained A0-b control.
Production compiler storage and all S07 gate definitions remain unchanged.

## New physical observations

The [untimed owner census](../tools/s07/performance-experiments/owner-census/README.md)
executes the immutable A0-b source on all 13,094 frozen files. It preserves
physical per-file counts, including unreachable/obsolete backing records, and
checks the same loaded-input identity and work totals as the control. The
observer reads existing runtime IDs and never requests lazy JSDoc.

| Observation at the retained parse/bind endpoint | Result |
| --- | ---: |
| Physical core nodes | 19,593,488 |
| Nodes with assigned runtime IDs | 2,207 in 399 files |
| Symbols with assigned runtime IDs | 117 |
| Identifier/private-identifier rows | 6,795,224 |
| Rows whose bytes match a checked source suffix | 6,795,224 |
| Identifier exceptions at this endpoint | 0 |
| Physical syntax backing records / edge slots | 3,114,989 / 6,047,867 |
| Physical declaration backings not referenced by physical symbols | 5,442 |

Sparse runtime-ID storage now has an observed occupancy to price. It still
needs the original lazy assignment, concurrency and identity semantics, with
measured lookup/initialization costs. The zero identifier-exception count supports
the ordinary source-suffix representation on this workload; it does not establish
that synthetic, cooked, imported or subsequently mutated nodes can discard text
ownership. The census's 68.819 MB of selected identifier byte occurrences is not
an additional source allocation that can be subtracted from memory.

The native child completed once. A later host-metadata query was blocked, so
the producer recovered metadata from the unchanged raw output and retained
distinct build/capture/finalizer snapshots. The archive documents this recovery;
no rerun or replacement of the physical observations is presented as the first
capture. Root review checked the staged read-only accessors, physical coverage,
suffix predicate, runtime getters and lazy boundaries; it found no substantive
observation defect. Archive-only replay verifies the durable package.

## Identifier construction and updated node sizes

The [text prototype](../tools/s07/performance-experiments/storage-pilot/README-text.md)
implements four-byte words with source suffixes and an explicitly owned exception
pool. It checks tag overflow and raw bounds, preserves text across range edits,
and supports factory observations before final positions exist. Ordinary reads
borrow bytes without per-identifier Arc operations; escaping text explicitly
retains independent bytes.

Factory staging is a real cost: if every identifier enters through the generic
factory route, every name is initially pooled. Compaction after range assignment
does not recover those requests or retained obsolete pool entries. The parser's
ordinary path must supply its known end before insertion where the no-hook
contract allows; the public factory/hook path retains its earlier observation.
This is an integration requirement, not a new assumption that hooks run later.

The [recomputed page model](../tools/s07/performance-experiments/layout-followups/README.md#four-byte-text-in-the-page-matrix)
preserves the old physical census and existing page policies. Four-byte text
removes 27.181 MB of used payload, including private identifiers. Recharging
capacity and directories gives 808.688 MB for eight-row mixed pages and
860.233 MB for the leading four-row typed-page policy. These include modeled
headers and inline binding payloads; text owner metadata, runtime IDs and
full-range links remain additional. Both fit the provisional 880 MB live budget,
so that category does not decide the storage family.

## Compiled binding and auxiliary components

The [owner model](../tools/s07/performance-experiments/owner-layout/README.md)
now prices concrete records against physical per-file capacities. The following
examples use thin 16-record pages and four-byte canonical symbol names. These
are conditional layout projections, not measurements of replacement storage.

| Component | Live MB | Requested MB |
| --- | ---: | ---: |
| Binding: inline symbol IDs, 28-byte flows | 345.746 | 368.273 |
| Binding: sparse symbol IDs, outlined 20-byte flows | 302.671 | 325.201 |
| Binding: sparse symbol IDs, packed 16-byte flows, before escapes | 290.748 | 313.278 |
| Auxiliary: direct 24-byte list headers, per-kind arena routing | 177.803 | 181.838 |
| Auxiliary: direct headers, original global slots plus locator | 237.276 | 245.703 |

The binding estimates support testing smaller records, but they do not make the
entire 440 MB provisional allocation available as headroom. Canonical symbol-name
keys require a byte-semantic interner whose construction, retention and capacity
are still unpriced. Source-suffix identifier words are not canonical names. The
packed flow's full-domain escape distribution is unmeasured; its all-flows
escape bound adds 115.701 MB live and 230.774 MB requested. Sparse runtime-ID
getters still need their concurrency, identity and CPU checks.

The auxiliary alternatives expose a cost that was understated by the provisional
140 MB allocation. They preserve individual boxed backings, and another 21.689 MB
of owned text/metadata/source-map children remains additional. Per-kind routing
requires a checked owner dispatcher; retaining the current global auxiliary
identity adds a locator. Moving slices into distinct 12-byte descriptors costs
190.345 MB before those children, so indirection alone does not save memory.

The node-ID table at observed occupancy costs 0.584 MB live; the text prototype's
empty per-owner state adds 1.467 MB. Neither includes all owner state or full-range
link escapes. A joined model must deduplicate the shared owner ledger and source
records, price the canonical-name interner and all temporary traffic, and
reconcile the old 257.480 MB unexplained native-live residual. These component
results do not yet prove the 1,700 MB whole-owner target.

## List construction contract

The [list prototype](../tools/s07/performance-experiments/storage-pilot/README.md)
uses one nested scratch buffer and appends completed lists into fixed owner edge
pages. It removes individual list scratch vectors from that construction path
while preserving distinct backing/header identities, shared slices, nil versus
allocated-empty versus missing, locations and modifier fields. All scratch,
page initialization, page spare and directory storage remain charged.

Its public construction identities are internally unique, distinct from the
supplied node domain. Wrong nesting preserves the caller's frame token, stale
serials are rejected, ownerless sentinels are reusable, and an unfinished builder
cannot publish. The immutable result exposes no mutation. A full-`usize` escape
preserves oversized backing offsets without enlarging ordinary descriptors.

The remaining access-contract change is explicit: a logical backing may span
several pages. The pilot resolves checked elements and contiguous page spans;
it cannot return one borrowed `&[Option<NodeId>]` across those allocations.
Production integration must port the consumers that require that slice, preserve
actual AST retention and foreign/lazy fallback, and avoid reconstituting a Vec
on every read. This prototype does not stand in for that integration.

## First list-policy measurement

The [fixed capture](../tools/s07/performance-experiments/storage-pilot/README-list-results.md)
retains eight warmup children and 56 measured children: seven normal and seven
allocation observations per policy. All 64 reproduce the physical backing/edge
counts and checksum; allocation runs balance their storage drop and reject
allocations during reads. No samples were discarded or added.

These are medians for replaying the actual backing-length distribution with
synthetic edges. Times are milliseconds, memory is decimal MB, and traversal
covers eight complete sweeps. This replay does not reconstruct parser vector
capacities or nested construction order, and it excludes AST nodes and headers.

| Policy | Construction ms | Eight-sweep traversal ms | Requested MB | Retained MB |
| --- | ---: | ---: | ---: | ---: |
| Growing Vec then box | 49.768 | 94.160 | 310.418 | 119.564 |
| 64-word pages | 33.129 | 129.895 | 104.672 | 67.298 |
| 256-word pages | 32.894 | 127.652 | 109.476 | 72.780 |
| 1,024-word pages | 35.659 | 129.840 | 141.286 | 104.684 |

The 256-word policy cuts requests by 64.7% and construction time by 33.9%, but
traversal is 35.6% slower. All timing relative MADs are below 1.5%; nevertheless,
the 18-CPU host's initial load averages were 27.79 / 30.06 / 29.03. Process-name
checks and low dispersion do not rule out unrelated load or systematic timing
bias. Treat this as an exploratory comparison, not a production CPU result.

Decision: do not promote the page-spanning list representation. Preserve this
capture and test a separate contiguous chunk variant: finish each backing into
one typed chunk, expose a checked contiguous `&[u32]`, and charge larger backing
descriptors and abandoned chunk tails. The hypothesis is that direct backing
reads can retain construction savings without the page-span traversal penalty;
the first capture does not establish that hypothesis.

## Contiguous-chunk result and next decision

The [second fixed capture](../tools/s07/performance-experiments/storage-pilot/README-chunk-results.md)
tests that hypothesis with fresh legacy and page-256 controls in the same batch.
The safe implementation preserves nested construction and checked slice reads;
each complete backing fits within one typed chunk. It uses 12-byte backing
descriptors and 24-byte chunk-directory entries. All retained scratch, spare
words, directories and descriptor growth are included in its measured requests.

| Policy | Construction ms | Eight-sweep traversal ms | Requested MB | Retained MB |
| --- | ---: | ---: | ---: | ---: |
| Growing Vec then box | 48.889 | 95.462 | 310.418 | 119.564 |
| 256-word pages | 33.683 | 128.679 | 109.476 | 72.780 |
| 256-word contiguous chunks | 38.184 | 129.338 | 145.703 | 91.293 |
| 1,024-word contiguous chunks | 41.015 | 129.429 | 177.002 | 122.877 |

All 64 children pass the input, checksum, storage and schedule checks. The
independent next-fit simulation exactly predicts 31,443 / 15,654 chunks and
2,022,625 / 9,989,656 spare words. Allocation observations are identical within
each policy. Timing relative MAD stays below 1.5%, but initial host load is again
high (27.40 / 27.23 / 29.98 on 18 CPUs); the exploratory timing limit still applies.

Decision: reject these contiguous-chunk candidates as replacements for the page
policy. They add storage/construction cost without a material traversal gain;
the checked chunk traversal still takes about 35.5% longer than legacy lists.
The 1,024-word candidate even retains more bytes than the legacy control. This
result rejects these policies, not every possible allocator chunk design.

Contiguity alone did not recover read performance. Both compact paths still
resolve backing identity and validate ranges for each backing. The next bounded
experiment should distinguish that work from the actual word reads: compare
checked public-ID access with access through a borrow already resolved within
its owner, retaining safe bounds checks and rejecting foreign/stale inputs at
the boundary. It must establish where a reusable proof exists in real callers;
removing checks in an artificial traversal is not a production improvement.
The representative node-access pilot should use the resulting contract before
the generated-storage migration. No third timing batch or production promotion
is part of this checkpoint.

## Validation and continuation

The original text/list/page code passes 16 tests in debug, release, Rust 1.96
and strict-provenance Miri on macOS ARM. The contiguous-chunk implementation adds
12 tests that also pass all four modes; the combined crate passes all 28 debug
tests and all-target/all-feature Clippy with warnings denied. Miri reports unused
driver dependencies when testing only the library; its executed contract tests
pass. The page model has an independent
arithmetic replay against the historical controls and all five new candidates.
The owner census and list producer have separate malformed-input/provenance
tests. These results do not replace production E3 or full graph parity.

The next trial isolates resolved-owner access, followed by representative hot
flag reads, child enumeration and narrow binding writes for typed versus mixed
payload rows.
Broad generated-accessor migration remains gated on viable whole-owner accounting
and that node-access comparison. A0-b remains the production control, and the
four final Go-relative memory/CPU criteria remain open.
