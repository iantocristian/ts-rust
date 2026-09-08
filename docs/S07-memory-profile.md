# Comparative S07 memory attribution

This investigation compares Rust and pinned Go on the frozen 13,094-file
parse-and-bind workload. It is diagnostic evidence for choosing the next change;
it does not replace E5/E6 acceptance captures or complete S07.

The clearest next experiment is binding side-table storage: the full-record-map
region requests 633 MB, while that map and flow slots retain at least 445 MB,
including 177 MB beyond used payload. The remaining RSS domains are still
unattributed; this supports a bounded storage experiment, not a complete
allocator decomposition.

The acceptance measurements remain 4.729 GB versus 2.907 GB allocated and
4.565 GB versus 3.156 GB peak RSS at one worker. These are different questions:
request traffic, retained compiler storage and resident process memory need
separate accounting. All GB/MB values below are decimal.

## Measurement boundaries

The adapters preserve per-file parse→publish→bind order, bounded round-robin
queues, preloaded bytes and retained endpoint roots. Every child must match
13,094 files, 161,740,237 source bytes, 19,593,488 nodes, 2,459,867 symbols,
423 parse diagnostics, 5,250 bind diagnostics and loaded-input SHA-256
`d4ff2ad3b0d590babb596522a7105d961dbb3c4f2ebc987488d88b7ebd8e0c88`.

Rust uses safe, additive census observers in a staged copy of the compiler.
They count every allocated owned arena slot, actual boxed payload variants and
backing capacity, without evaluating lazy getters. Shared Arc backing is counted
once. Arc headers are a separate estimate; private tree/hash layouts remain
unreported. The compiler checkout and its acceptance executable are unchanged.

Go uses an access-only observer in the pinned AST package and standard sampled
heap profiles. Its reachable-object census cannot discover unused arena backing
kept alive by interior pointers. Constructor allocation stacks supply an
independent view of that backing and allocator rounding. A concrete Go AST
struct includes its embedded Node and binding fields; its Node header must not
be added again. Visible string ranges may include static/shared bytes and are
not automatically heap allocations. Go deduplicates within each file; shared
global objects can recur across files, so its subtotal is logical storage rather
than an exact live-heap lower bound.

Both children pause before the pipeline and immediately after its retained
endpoint. The parent reads RSS and a VM summary before either census executes.
Go records its native endpoint separately from a deliberate two-GC retained
snapshot, to address the runtime heap profile's documented lag. That deliberate
GC state is not the acceptance benchmark's native endpoint. Profile sampling
is fixed at 64 KiB for the process.

A separate Rust build records requested allocation traffic in 58 source regions,
including all 37 actual payload-boxing sites. It uses the MIT-licensed
`alloc_tracker` 0.5.25 wrapper around the same mimalloc implementation. Both it
and cap forward native allocation, zeroing and reallocation calls with their
original layouts. Nested source rows overlap; an independently measured
outermost-site union supplies each phase's unclassified remainder.

## Results

All 18 fresh processes passed: three Rust/Go pairs at each worker count and
three Rust source-scope runs at each worker count. The production source
fingerprint remained
`17b7559a34741eb8fcf73932c4af928d264981dc2607c154143bf69567152fa3`.

The primary finding is that most of the compared extra retained allocation
appears during Rust binding. The following one-worker values compare Rust's
signed live-request growth across each operation with Go's sampled retained
allocation stacks after two GC cycles. They are different accounting methods,
so this is approximate origin attribution, not an exact heap decomposition.

| Allocation origin | Rust live-request growth | Go sampled retained bytes | Difference |
| --- | ---: | ---: | ---: |
| Parse | 2.385 GB | 2.197 GB | +189 MB |
| Publish | 0.007 GB | Included in Go parse | +7 MB |
| Bind | 1.413 GB | 0.666 GB | +747 MB |

Binding accounts for approximately 79% of this compared difference. Go's
one-worker retained parser estimates span 2.188–2.222 GB and binder estimates
0.663–0.677 GB across the three runs. Its eight-worker medians are 2.185 and
0.670 GB, respectively, supporting the same allocation-origin conclusion.

**Allocation origin is not semantic ownership.** Go embeds many binding fields
in parser-created concrete nodes; Rust adds side tables during binding. For
example, Go's 6,580,335 reachable Identifier records already contain a FlowNode
pointer, occupying about 52.6 MB within their existing concrete storage. The
Rust side tables' Go counterparts therefore have a real cost. The percentage
locates where the extra allocation appears; it does not prove that this
percentage can be removed by changing the binder.

### Concrete retained structures

The Rust census measures these disjoint payload/capacity rows:

| Rust structure | Used payload | Capacity payload | Capacity beyond used |
| --- | ---: | ---: | ---: |
| General node-binding map | 188.5 MB | 274.7 MB | 86.2 MB |
| Paged flow bindings | 79.3 MB | 170.6 MB | 91.3 MB |
| Binding node-overlay map | 22.3 MB | 31.6 MB | 9.4 MB |
| Symbol value pages | 275.5 MB | 353.8 MB | 78.3 MB |
| Flow-node value pages | 141.6 MB | 174.6 MB | 33.0 MB |
| Core syntax-node pages | 1,567.5 MB | 1,677.5 MB | 110.0 MB |
| Core auxiliary pages | 267.3 MB | 311.6 MB | 44.3 MB |

The general binding map and flow slots together account for at least **445.3 MB**
of retained payload/capacity. The flow table uses 9,909,924 of its 21,324,288
allocated slots (46.5%). The general map holds 2,618,417 entries with a public
capacity of 3,815,009; its private bucket/control allocation is additional.
Across the whole Rust census, capacity exceeds used payload by **538.9 MB**.
That is measured spare storage, not an estimate of what a replacement can save.

Go's reachable census is insufficient for the corresponding physical backing.
For example, reachable symbol records occupy 236.1 MB, while their constructor
stacks account for a sampled retained 306.1 MB; reachable flow records occupy
66.2 MB, while their constructor stacks account for 116.7 MB. The corresponding
Rust value-page capacities are 353.8 and 174.6 MB. Go's symbol-table entries have
47.7 MB of logical key/value payload, while the actual map-insertion allocation
site retains a sampled 152.0 MB. These observations demonstrate that Go also
pays for arena backing, map internals and allocator rounding.

The complete Go concrete-node census is 1.521 GB; all NodeFactory constructor
stacks retain an estimated 2.127 GB, including node/modifier-list constructors.
The two totals have different scope. The latter observes backing that the
reachable-object walk cannot see. Rust's boxed NodeData payloads account for
155.8 MB, about 3.9% of its requested live bytes; boxing alone is not an adequate
explanation of the measured gap.

### Allocation traffic

The source-scope adapter preserves the native Rust request total within about
2 KB at the group medians. Its retained RSS differs by less than 0.3 MB at those
medians. This bounds observed memory perturbation; it is not a CPU calibration.

| Rust region | Requested bytes | Interpretation |
| --- | ---: | --- |
| Full binding-record map | 633.4 MB | Map insertion/growth region |
| Flow-slot storage | 172.0 MB | Paged slots and foreign-owner fallback |
| Binding node overlays | 85.9 MB | Copies and map storage |
| Symbol-table insertion | 219.4 MB | Inclusive insertion region |
| Symbol arena insertion | 399.1 MB | Inclusive symbol/page region |

These rows can overlap and must not be summed. The explicit source-region union
covers 1.840 of 1.872 GB of binding requests (98.3%) and 2.219 of 2.849 GB of
parsing requests (77.9%). Unclassified regions remain 32.1 MB in binding,
630.1 MB in parsing and 6.9 MB in publication. Full per-site rows are archived.

The native Rust interval requests 4.729 GB while live requests grow by 3.805 GB.
The **923.3 MB** difference is freed or superseded requested storage, despite
peak live bytes being close to endpoint live bytes. Within the one-worker
operation intervals, this difference is about 464.0 MB in parsing and 459.3 MB
in binding. It confirms that a flat live-memory high-water mark did not establish
low temporary allocation traffic. Source regions identify where to investigate
this traffic; they do not reconstruct individual allocation lifetimes.

### Runtime and resident-memory reconciliation

These are one-worker medians from the diagnostic captures:

| Domain | Rust | Go |
| --- | ---: | ---: |
| Native retained endpoint RSS | 4.565 GB | 3.180 GB |
| Native requested live / HeapAlloc | 3.972 GB | 3.068 GB |
| Retained HeapAlloc after deliberate two GC cycles | — | 3.038 GB |
| Census payload/capacity plus separate Rust Arc estimate | 3.714 GB | 2.226 GB |
| Live minus that census subtotal | 257.5 MB | 812.1 MB after two GC cycles |

Rust's subtotal consists of 3.703 GB of payload/capacity and an 11.7 MB Arc-header
estimate. The Go subtotal excludes contained Node headers and other visible
strings whose backing may be static. The larger Go census residual is why
subtracting those two census totals would overstate the actual live-byte gap.
Go's sampled retained heap total independently tracks its exact post-GC heap
closely; the raw estimates and ranges are preserved.

The native RSS gap is 1.385 GB at one worker and 1.376 GB at eight. Approximately
0.48 GB of the one-worker gap lies in the difference between each runtime's RSS
and native live-byte counter. That portion is still **unattributed**; it is not a
measurement of mimalloc fragmentation or GC overhead. Go separately reports
about 46 MB between native HeapInuse and HeapAlloc. OS VM summaries and complete
MemStats records remain available for a later allocator-specific investigation.

### Next experiment

Prioritize the binding-record map, flow-slot occupancy and their growth policy.
Measure replacement metadata and lookup cost alongside any payload savings;
preserve owner validation and publication behavior. The 633 MB request site,
445 MB retained subtotal and 177 MB of spare capacity in these two structures
make this a concrete experiment that can address both retained bytes and
allocation traffic. The earlier CPU profile's binding lookup costs give a second
reason to measure that path.

The parser's 630 MB of unclassified allocation regions is the next attribution
gap if allocation traffic remains a priority. An allocator-specific VM census
would be needed to name the remaining RSS domains. These are explicit remaining
questions; they do not prevent a bounded binding-storage experiment. None of the
current measurements demonstrates that such a change will meet S07's unchanged
memory or CPU thresholds.

## Interpretation limits

A Rust–Go census subtraction is not a retained-heap delta. Rust visits allocated
owned slots; Go visits reachable objects and visible ranges. In Go, the missing
arena capacity, unused records and map internals are material. In Rust, hash
bucket/control bytes and BTree node capacity also remain outside the payload
census. Neither residual is synonymous with allocator fragmentation.

Likewise, RSS minus requested live bytes includes several unmeasured domains:
allocator rounding/caches, native/runtime metadata, resident free pages, stacks,
code and mapping/accounting differences. The VM summary records those domains
where the OS exposes them; its rounded resident-region total includes shared
mappings and is not the same metric as RSS.

Rust's request counter charges the full new size of successful reallocations,
even an in-place grow or shrink. Request traffic minus live growth therefore
counts freed or superseded requests, including any released preload allocation;
it is not necessarily copied bytes or a pipeline-allocation lifetime cohort.
Go's TotalAlloc uses different allocator accounting. Go sampled alloc_space and
inuse_space are estimates, not exact request counters.

The ordinary Go baseline profile can lag behind preloading. Subtracting it from
a post-GC profile can retain delayed preload/digest samples. The analyzer
preserves signed values and classifies actual compiler/diagnostic/preload stack
ancestors; it never forces sampled totals to equal the exact pipeline counter.

Both censuses allocate inspection buffers. Their cost is recorded after the
retained snapshots. Subsequent retirement RSS includes inspection effects on
allocator/runtime caches; it is not a clean measurement of ordinary compiler
retirement. Live counters can still establish root release. No CPU-performance
claim is made from these instrumented or paused processes.

## Validation and reproduction

The [archived results](../tools/s07/memory-profile/results/2026-09-08/README.md)
contain all capture records, raw Go profiles, native pprof exports, VM summaries,
censuses, source-scope rows, build provenance and hashes. The
[memory tools](../tools/s07/memory-profile/README.md) contain the build,
capture, census and analysis commands. Builds always create fresh staged copies,
check actual staged bytes and external Cargo configuration before and after
compilation, and bind the executable to its Cargo artifact and dependency lock.
Captures reject changed source/tool/configuration/binary identities, incomplete
checkpoint sequences, inconsistent counters and missing or altered profiles.

Independent review found and fixed stale-stage provenance, overly permissive
completion records, missing source-scope artifact validation and cold-operation
mutex allocations contaminating enclosing scopes. Validation passes 29 Python checks, three Go census checks, four Rust census
checks and the release allocation-scope probe. Fixed-request tests cover
nested regions, source unions, native grow/shrink requests and eight-thread
counts. Census tests cover shared backing, embedded Go headers, cycles, visible
slice unions, lazy state and unknown-layout accounting. Reflection field paths
are cached in the Go observer to avoid repeatedly allocating promoted-field
lookup metadata while inspecting millions of nodes.

Independent extraction and analysis replay reproduce all 18 run summaries and
all 90 native Go exports exactly; resolved Go/pprof hashes are retained in the
replay receipt.

No production representation, acceptance threshold, sprint routing or stored
E5/E6 evidence changed in this investigation.
