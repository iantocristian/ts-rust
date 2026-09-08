# Comparative S07 memory attribution

Diagnostic adapters for the frozen 13,094-file S07 parse→publish→bind workload.
They retain every file through the endpoint, preload the same bytes, use the
original bounded round-robin dispatch and run with one or eight workers. These
tools neither replace acceptance executables nor publish E5/E6 evidence.

## Measurement plan and review constraints

1. Freeze production source hashes, the upstream gitlink, transport and loaded
   byte digest, toolchain settings, registry lock checksums and actual executable
   artifacts. Verify all six workload counters on every fresh process.
2. Add read-only ownership observers in disposable Rust/upstream copies. Keep
   original record layouts and compiler operation order. Count owned allocation
   payload once; report used slots, spare capacity and unknown private layouts
   separately. Go concrete AST structs embed their Node header; never add the
   header again. Deduplicate backing storage, rather than counting slice views.
3. Pause immediately before the pipeline and at its retained endpoint. The parent
   records RSS and VM summaries before census/profile serialization allocations.
   Keep requested allocator bytes, Go HeapAlloc/HeapInuse, OS RSS and physical
   footprint as distinct measurement domains.
4. Capture Go native endpoint and deliberate post-GC state separately. Explicit
   GC serves retention diagnosis; it does not replace the benchmark's native GC
   policy. Preserve Go sampled allocation/heap profiles and exact MemStats
   counters. Census runs only after native and retained-GC snapshots.
5. Use cap's native mimalloc forwarding for Rust request/live counters. A separate
   allocation-scope build uses the MIT alloc_tracker 0.5.25 wrapper, which also
   forwards original allocation layouts and native zero/realloc calls. Preserve
   its overhead and inclusive-scope limitations. Neither census bytes nor scope
   request traffic are retained allocation backtraces.
6. Repeat matched worker configurations in fresh processes, alternating runtime
   order. Reconcile known storage categories and capacities against observed
   runtime totals; leave unmeasured bytes as explicit residuals. Report census
   and scope perturbation rather than subtracting invented overhead estimates.
7. Independently review the accounting and reproduction artifacts before
   proposing production changes. Storage redesign is outside this investigation.

Review decisions: tracking-allocator 0.4.0 adds per-block headers, changes native
realloc behavior and uses MPL-2.0, outside this workspace's dependency policy.
alloc-track 0.4.0 also replaces native zero/realloc with GlobalAlloc defaults.
Neither is used. alloc_tracker provides source scopes, not object lifetimes or
sampled call stacks. Nested site scopes are inclusive; only the explicitly
measured site union can be subtracted from its enclosing phase.

## Run

The source census and Go bridge are documented in their subdirectories. Native
capture currently requires macOS, the pinned Rust/Go versions and permission to
inspect child processes using `ps` and `vmmap`. Caller Cargo/Rustup homes and
registry configuration are preserved. Full Xcode selection is per-command.

```sh
python3 tools/s07/memory-profile/build.py
python3 tools/s07/memory-profile/capture.py --output target/s07-memory-profile/capture
python3 tools/s07/memory-profile/build.py --sites --rust-only
python3 tools/s07/memory-profile/capture.py --build-manifest target/s07-memory-profile/build-sites.json --runtime rust --output target/s07-memory-profile/sites-capture
GOCACHE=/private/tmp/ts-rust-s07-go-cache python3 tools/s07/memory-profile/go/analyze.py --capture target/s07-memory-profile/capture
python3 tools/s07/memory-profile/analyze.py target/s07-memory-profile/capture target/s07-memory-profile/sites-capture --output target/s07-memory-profile/summary.json
```

Build staging lives below `target/s07-memory-stage` and
`target/s07-memory-go-stage`. Rebuilding replaces those disposable copies. The
build manifest binds additive source patches, executable hashes and the unchanged
acceptance source fingerprint. Capture directories must be new: failed captures
and stderr remain available for diagnosis.

RSS samples are cooperative checkpoints, not stop-the-world suspension: runtime
background threads can still run during `vmmap`. Both RSS readings around each
VM summary are preserved. The VM tool rounds displayed units and its total
resident regions include shared mappings; do not equate that total with `ps` RSS.
Post-retirement snapshots occur after census/profile work and therefore include
its effects on allocator/runtime caches. Native retained endpoint snapshots do
not include that later work.

Rust's `total_requested` counts the full new size of successful realloc requests,
even shrink or in-place realloc. Subtracting live-byte growth measures requests
freed or superseded during the interval, including any freed preloaded objects;
it is neither physical copy volume nor an allocation-cohort lifetime census.
Go's TotalAlloc has different realloc/size-class semantics. The two request
metrics are informative but are not identically defined physical byte traffic.
