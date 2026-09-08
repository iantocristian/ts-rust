# Isolated storage construction pilot

This crate implements two pieces of the compact-storage access contract before
the generated AST migration: four-byte identifier text and compact owner-local
list construction. It is outside the production workspace. It emits no sprint
metrics, and none of its timing or allocation results are parse/bind acceptance
measurements. The separate [text contract](README-text.md) documents its source,
pool, range-edit and factory behavior.

## Nested lists and backing identity

`Builder` has one reusable `Vec<u32>` scratch buffer and a stack of active list
frames. An inner list starts after its parent's current scratch prefix. Finishing
the inner list appends its words to owner-held edge pages, records its backing,
and restores that scratch prefix. The parent can then append its next node.
Every completed edge is copied from scratch once; scratch growth is shared
across lists rather than allocating a fresh vector for each list.

The completed edge storage uses fixed typed pages of 64, 256 or 1,024 `u32`
words. Spare entries are initialized and charged. Each backing has an eight-byte
descriptor, with an explicit full-`usize` offset map for positions at or above
the reserved `u32::MAX` marker. Logical slices can span pages, so this pilot
offers checked element/sweep access, **not a contiguous borrowed
`&[Option<NodeId>]`**. Porting consumers that require that slice remains an
integration obligation; materializing it per read would defeat the design.

Copied slice headers share their backing. Mutation through one slice remains
visible through overlapping slices. Nil, allocated-empty and the process-wide
missing sentinel remain distinct, while `same` preserves the current empty-slice
comparison. NodeList header identity is separate from backing identity; copying
a header preserves location/modifier fields without coupling subsequent header
updates. The actual local header in this prototype costs 24 bytes, not an
assumed 16 bytes.

List construction identities are minted internally, independently of the
supplied node domain, so two builders cannot accidentally accept each other's
frames or allocated backings. Nil/missing sentinels have no owner. Frame serials
prevent reuse after success; invalid nesting returns an error without consuming
the caller's token. Aborting a frame restores its scratch prefix. An incomplete
builder cannot publish. The published wrapper exposes no mutation authority.

The supplied node domain validates local node owner and slot bounds, including
the full nonzero `u32` slot range. It does **not** retain actual AST nodes. A
production owner must supply that retention, public-ID routing, foreign/imported
fallback and lazy construction. The prototype does not establish those missing
contracts by assigning a numeric owner token. Local descriptor/ID routing and
all allocator requests must be included in the eventual whole-owner budget.

Scratch stays retained at publication and is charged. Moving it into reusable
worker storage or dropping it at publication is a separate policy choice; the
probe cannot obtain a lower endpoint by silently excluding it.

## What the list replay measures

`list-pilot` consumes the owner census's per-file physical backing lengths in
core auxiliary allocation order, including empty and unreachable backings.
It constructs the same deterministic synthetic edge values for every policy.
The control uses a growing eight-byte-element Vec followed by `into_boxed_slice`;
the candidates use reusable scratch and compact pages. This compares backing
construction policies with the real size distribution. It does not reconstruct
the parser's initial Vec capacities or nested begin/end order; unit tests cover
nested construction separately. It excludes AST nodes, NodeList headers, symbol
lists, binding and foreign imports in every policy.

All completed file roots stay alive at the construction endpoint. Separate
uninstrumented and allocation binaries use the workspace's mimalloc/cap versions.
The allocation binary measures original layout requests and retained requested
bytes after construction, rejects allocations during read-only sweeps, and
requires storage drop to return to the starting live counter. Its exact
preflight requests 1,200,050 bytes through ordinary allocation and growth.
Allocation builds suppress their incidental timings. JSON report construction
is outside the measured storage interval and is separately removed from the
drop check.

The immutable build/capture runner, raw observations and decision record supply
the interpretation of these numbers. A faster or smaller replay alone cannot
promote a production storage representation or pass S07's CPU/RSS gates.

The [first recorded comparison](README-list-results.md) finds lower construction
requests but slower checked traversal. The separate [contiguous-chunk trial](README-chunk-results.md)
tests whether placing a whole backing in one typed chunk recovers that read cost.
It does not: the tested chunk policies cost more than pages without a traversal
improvement. Both captures remain preserved, and neither representation is
promoted into the compiler.

`chunks.rs` keeps completed backings contiguous with next-fit allocation of
`max(policy_words, backing_length)`. It exposes checked `&[u32]` borrows, preserving
nested staging and shared mutation before publication. Backing descriptors cost
12 bytes and chunk-directory entries cost 24 bytes; unused tails, oversized
chunks and retained staging capacity are charged. These words still require
a production local-ID facade and actual AST retention; they are not a borrowed
`&[Option<NodeId>]`. See the module's documented integration limits.

## Contract checks

```sh
cargo test --offline --locked --manifest-path tools/s07/performance-experiments/storage-pilot/Cargo.toml
cargo test --offline --locked --release --manifest-path tools/s07/performance-experiments/storage-pilot/Cargo.toml
cargo clippy --offline --locked --manifest-path tools/s07/performance-experiments/storage-pilot/Cargo.toml --all-targets --all-features -- -D warnings
```

The original 16 tests cover nested-list isolation, failed-frame recovery, distinct
owner/header/backing identities, shared slices, all empty states, invalid inputs,
wide-offset encoding, page-span boundaries, raw/pooled text, range edits, factory observations, foreign
text lifetime and tag overflow. Debug, release, Rust 1.96 and the pinned nightly's
strict-provenance Miri pass on macOS ARM. This is instrumentation of the isolated
prototype, not another run of the production E3 ownership harness.

The chunk module adds 12 tests for contiguous/oversized backing ranges, abandoned
tails, full-width domains, frame errors, mutation aliases and publication. They
also pass debug/release, Rust 1.96 and strict-provenance Miri. The combined crate
passes all 28 debug tests and all-target/all-feature Clippy with warnings denied.
