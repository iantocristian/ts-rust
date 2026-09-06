# ts_arena

S04's storage and ownership primitives for the compiler. Payloads come from the
future AST and binder crates; the generic `Node<T>` carries only storage-relevant
metadata. This is a leaf implementation, not a semantic checker or API handle
registry. It follows the accepted [ownership design](../../docs/design/ownership.md),
[symbol design](../../docs/design/symbols.md), and [panic policy](../../docs/adr/0012-panics--recovery-and-generation-retirement.md).

## Public contracts

- `NodeId` and `SymbolId` are distinct `NonZeroU64` wrappers with full 32-bit
  arena and slot fields. Zero fields are rejected. Their bits describe storage,
  not ownership proof or protocol handles. Every actual arena, generation token,
  and checker identity consumes a checked process-global identity that is never
  reused. Arena identity exhaustion aborts with a diagnostic; slot exhaustion
  fails before truncation or reuse.
- `FileBuilder` permits exclusive construction and mutation, then transfers
  immutable core nodes, bound symbols, and source bytes into a `FileHandle`.
  Lazy nodes live in separate fixed 256-slot pages. `NodeRef`, `SymbolRef`, and
  `NodeListRef` retain the file or its whole mapped bundle.
- A mapped `BundleOwner` owns the canonical and supplemental files. Links
  between those files are non-owning IDs; any retained member keeps every sibling
  alive. `Scope` retains its admitted files and resolves IDs with ownership and
  published-slot checks in release builds. It does not change a program's
  observable file membership or ordering.
- Lazy JSDoc graphs and source tokens use one `parking_lot::RwLock` for caches,
  directory access, allocation, and publication, with a second cache lookup after
  acquiring the write guard. Pages use `OnceLock` slots, so growing the directory
  or initializing another slot cannot invalidate an existing reference. The
  initializer must not reenter that file's lazy APIs while holding the lock.
- JSDoc construction stages payloads before publishing the complete graph and
  cache entry. Every minted slot is burned immediately. An initializer error,
  invalid root, or panic leaves unresolvable tombstones, never reusable IDs.
  Recovery preserves previously published lazy data and permits a later retry.
  Source token kind and reparsed-parent assertions retain the upstream behavior.
- `ScratchOwner` offers exclusive append and borrowed lookup. IDs never retain
  storage: callers retaining links to a different owner must retain that owner
  separately, for example in a `Scope`. Generic payloads must use non-owning IDs
  for graph edges rather than creating owning cycles.
- `CheckerIdentity` and `CheckerLease` provide an exact identity, an exclusive
  operation permit, and active-generation checks. `check_slot` is a bounds
  primitive; the future checker supplies its real published arena length and
  provenance. `Generation::retire` permanently rejects subsequent checks and
  lease acquisitions while existing owners remain alive. A non-poisoning lock
  does not establish checker validity. The caller must retire a panicking
  checker's generation before releasing its permit to another operation.

All external imports remain checked. No unsafe implementation, unchecked fast
path, or generative brand optimization is included. The only dependency is
`parking_lot`, as required by ADR 0012.

## Evidence boundary

Seven reusable scenarios in `scenarios` back the release tests and the `e3`
example. They check actual file/bundle/lazy/scratch components, checker identity
primitives, injected ID boundaries, same-slot cross-owner rejection, retirement,
failed-transaction ABA rejection, concurrent first use, held references during
page growth, panic recovery, every mapped-file drop ordering, and final disposal.

`Counters` use an explicit observation domain, so concurrently running scenarios
do not reset or contaminate one another. An owner count tracks each published
file, bundle, scratch owner, checker identity, and generation object. Allocation
counts track owner storage, each nonempty core/symbol slab, and each lazy page.
They do not count every allocator call, payload allocation, cache/directory
bookkeeping allocation, or RSS. Each scenario records a nonzero peak and measures
the post-disposal delta against its starting snapshot; the JSON reports the
maximum measured delta across all seven scenarios. Payload-drop assertions also
check failed transactions and final storage destruction.

```sh
cargo test -p ts_arena --release --lib
cargo run -p ts_arena --release --features harness --example e3
cargo clippy -p ts_arena --all-targets --all-features -- -D warnings
```

The seven test names begin `tests::e3_` and correspond to `id_exhaustion`,
`wrong_owner_rejected`, `stale_and_recycled_ids_rejected`,
`concurrent_lazy_storage`, `mapper_bundle_disposal`,
`owners_return_to_baseline`, and `allocations_return_to_baseline`. The example
writes a single JSON report to stdout and measured observations to stderr;
expected caught assertion panics also use stderr. Miri and AddressSanitizer run
the same scenario functions through the tests. Their completion metrics belong
to the external evidence runner, never to the example's self-report.

These seven checks do not complete E3. Semantic checker and program symbols,
builder/transform owner dependencies, generation retirement across real shared
snapshots and registries, reentry integration, the gate serializing retirement
with result publication and response commitment, and actual API wire-handle
compatibility remain later integration work. In particular, `validate()` alone
is not an atomic publication gate.
