# S07-bis CP1: isolate access costs before changing storage

Date: 2026-09-08. This continues the [storage pilot](S07-bis-storage-pilot.md)
and [performance plan](S07-bis-performance-plan.md). A0-b remains the control.
The work has two distinct experiments; neither substitutes for the other or
emits a new Go-relative gate result.

## Questions and frozen comparisons

The earlier checked page and chunk sweeps both took about 129 ms against about
95 ms for legacy lists. Contiguity did not recover that gap. Separate repeated
identity/backing resolution from physical range access and already resolved
slice reads before choosing another storage layout.

If owner-descriptor traversal recovers the gap, test a checked owner capability
in real callers. If only cached slices recover it, first establish a disjoint
borrow across recursive binding; a cache of every list is not automatically
justified. If neither recovers it, inspect the traversal/layout before migrating
APIs. These comparisons isolate access paths, not the instruction-level cost
of each removed check: inlining and optimization can change with the path too.

The [access replay](../tools/s07/performance-experiments/storage-pilot/run_access.py)
uses the same 13,094 files, 3,114,989 physical backing lengths and 6,047,867
synthetic edges. Each mode retains all roots and visits all physical backings,
including empty and obsolete ones, in the same completion order:

| Mode | Resolution during each sweep | Additional retained storage |
| --- | --- | ---: |
| `legacy` | Existing boxed slices | None |
| `chunk256` | Existing checked public slice-ID path | None |
| `chunk256-owned` | Descriptor directly borrowed from that owner's backing vector; checked chunk and initialized range | None |
| `chunk256-borrowed` | Nested per-file cached `&[u32]` slices, resolved through the public path before sweeping | 50,154,080 requested bytes |

Borrowed-cache setup belongs inside construction time and the request/live
endpoint. It explicitly reserves every descriptor and outer vector entry. The
cache remains alive through all eight sweeps, then drops before its roots.
No self-referential owner, unsafe lifetime extension or uncharged side cache is
introduced. The explicit callback lifetime refers to the immutable published
owner; ordinary Rust borrowing prevents retirement while cached slices survive.

The owned path accepts no external backing descriptor. Public slice reads still
check ownership, backing identity and range, and the owned iterator still checks
the physical chunk and initialized extent. Counterexamples cover nested-list
alias writes, empty/oversized backings, corrupt physical descriptors and a borrow
attempting to escape its owner.

The producer fixes eight warmups and 56 measured children: seven normal and
seven allocation observations for each of four modes. It fixes eight complete
sweeps per child, retains every output and uses same-batch controls. It requires
identical physical storage and allocation totals for checked/owned modes, and
an exact additional cache request/live charge for the borrowed mode. Reads must
allocate nothing in the allocation binary; disposal must balance the endpoint.
High host load or unstable timing limits conclusions rather than allowing sample
replacement or an extended batch.

This is a diagnostic decomposition. A cached view of every list duplicates
metadata and is not a proposed production design. Nor does enumerating a private
owner's descriptors establish that arbitrary raw IDs can skip validation.

## Real binder contract and bounded production candidate

The production audit finds per-element complete slice resolution in
`Binder::syntax_node` and both functions-first passes. Those calls merit work,
but current `NodeSliceRead` borrows the entire `BindBuilder`/`ParsedFile` writer.
That borrow cannot survive recursive `self.bind(...)` calls in safe Rust.
Retaining list borrows requires a disjoint immutable auxiliary reader plus a
restricted core/binding writer and a compatible AST read facade. A longer
`AstView` lifetime would not establish this capability.

An exclusive full-node borrow also cannot survive mutation of that node. Keep
the existing small immediate-child/scalar snapshots where recursion requires
mutation. Published parsed syntax and effective bound overlay reads have
different lifetimes; a borrow into the growing overlay map cannot outlive a
mutation of that map. Imported, mapped, multiple-source and lazy paths retain
their ownership, retention, failure and initialization rules.

The first bounded production candidate changes only `BindBuilder::node`:
when its storage is exclusive and the requested ID belongs to the source's
core arena, call the existing checked `ParsedFile::core_node` and return a short
borrow. Every other read keeps the existing view/overlay routing. This skips
inactive routing, preserves owner/slot checks, and introduces no persistent
borrow or new layout. Symbol/locals reads and production list storage are unchanged.

A0-b's `direct_nodes` path already skips the overlay hash. The independent
review of the frozen normal arm64 binaries confirms that the control calls
`AstView::node`, while the eligible candidate path performs checked core
owner/slot/page access without that call. The fallback still calls the same
view routine. This establishes a structural routing change, not its elapsed
cost, and does not attribute the earlier lookup-union profile to a hash probe
that A0-b already removed. The inspection excludes the allocation binary and
other inlined callers.

Focused tests cover observed flag/text mutations and runtime identity, unretained
owners, published/sibling overlays, successful lazy creation after exclusive
selection and permanently invalid failed lazy slots. The E3 inventory adds those
two runtime cases, increasing S07 coverage from 27 to 29; the old result cannot
certify the new inventory.

## Validation and decision sequence

1. Review and test both candidates before freezing their source/binary identities.
   Keep the prior list captures and A0-b binaries unchanged.
2. Run the access replay under its declared fixed schedule. Interpret construction,
   traversal and allocation separately; do not add synthetic timings to binder
   results or claim production retention from borrowed local words.
3. For the production candidate, run relevant binder parity and all 29 S07 E3
   cases through the actual debug/release/Miri/ASan producer. Compare all 13,094
   frozen Go graphs at one and eight workers with unchanged work/input identity.
4. Use the retained A0-b bundle as the control for the existing fixed eight-warmup,
   56-sample full-pipeline screen. Require timing upper 95% ratios at most 1.02,
   requested/RSS median ratios at most 1.02, and relative MAD at most 5% for both
   variants in both worker modes. The default win is a targeted wall median at
   most 0.95 in at least one mode with timing upper bound below 1.0.
5. Check generated code or separate diagnostics before attributing a wall change
   to the targeted routing. This leaf shortcut does not itself implement the
   borrowed facade, so do not invoke CP1's infrastructure exception merely to
   keep an inconclusive optimization. Record keep/reject explicitly; do not tune
   criteria after reading samples.

The reusable-borrow and compact-node decisions remain open until these results
and their real caller contracts justify them. The complete owner/traffic budget
and final S07 memory/CPU gates remain separate requirements.

## Recorded outcomes and decision

The criteria above were committed as `ff6b5be` before either timing capture.
Both fixed batches completed; no sample was discarded, replaced or added.

The [access result](../tools/s07/performance-experiments/storage-pilot/README-access-results.md)
records eight-sweep medians of 92.747 ms for legacy, 128.041 ms for checked
chunks, 98.353 ms for owner descriptors and 85.833 ms for cached borrows.
The owner path recovers most of the gap without extra allocations, but remains
6.0% above legacy. Cached borrows beat legacy by 7.5% while adding exactly
50,154,080 requested/retained bytes; their retained endpoint is 18.3% above
legacy. Preserve both candidate access paths as diagnostics. Neither constitutes
a production list-storage selection or authorizes a cache of every list.

**Keep the production exclusive-core node lookup.** The
[full-pipeline archive](../tools/s07/performance-experiments/results/2026-09-08-cp1/README.md)
replays all eight warmups and 56 measured samples against the unchanged A0-b
bundle. Normal binaries supply wall time/RSS; allocation binaries supply requests.

| Metric | Workers | A0-b control | CP1 candidate | Candidate/control | Upper 95% timing ratio |
| --- | ---: | ---: | ---: | ---: | ---: |
| Wall time | 1 | 4.943356 s | 4.590853 s | 0.928692 | 0.991721 |
| Wall time | 8 | 1.144259 s | 1.079901 s | 0.943756 | 0.952266 |
| Peak RSS | 1 | 4.506681 GB | 4.506681 GB | 1.000000 | — |
| Peak RSS | 8 | 4.509254 GB | 4.509221 GB | 0.999993 | — |
| Requested allocation | 1 | 4.642570 GB | 4.642570 GB | 1.000000 | — |
| Requested allocation | 8 | 4.642571 GB | 4.642571 GB | 1.000000 | — |

Wall medians are 7.1% and 5.6% lower. All predeclared non-regression/noise tests
pass; both upper timing bounds are below one. The one-worker interval is broad
(0.879316–0.991721), so this capture does not establish a precise 7.1% effect.
Allocation and RSS are effectively unchanged: claim no memory improvement.
Initial host load was 6.91 / 8.40 / 10.58 on 18 CPUs; coordinated work stopped,
but unrelated host activity and systematic bias remain possible. Do not combine
this batch with earlier A0 timing or infer fresh Go-relative ratios.

All 13,094 Go graph comparisons pass at each worker count, with 13,094 exclusive
bindings and zero fallbacks. Graph parity uses the existing normalization:
49 eight-worker raw differences are normalized, so it does not imply raw byte
identity. The broader binder producer passes all 12,829 rows
across 22,343 primary requests, plus 18 supplemental requests, 32 protocol tests,
18 helper tests and depth checks. All 29 named S07 ownership cases pass in
debug/release/Miri/ASan (116 observations); the prior 27-case record is unchanged.
The first binder invocation failed on sandbox access to the existing Go cache;
the archive preserves that failure and the successful cache-access retry.

The candidate's frozen normal arm64 code confirms the intended routing change,
and independent implementation and measurement reviews support this bounded
promotion. It becomes the next experiment control at
`target/s07-bis/cp1-node-read-candidate`, manifest
`3a57976f667b9857891edbe9f76de5261c43480ac8c62d946b4545d9ee19d931`.
A0-b and the earlier controls remain immutable. CP1's general borrowed facade,
compact generated storage and the four final S07 CPU/memory gates remain open.

## Next bounded node-access slice

Compare current AST storage, typed payload rows and mixed plain/atomic word rows
over Token, Identifier, PropertyAccessExpression, CallExpression,
BinaryExpression, ExpressionStatement, Block, FunctionDeclaration and SourceFile.
Those shapes represent 12,194,095 frozen physical nodes (62.23% of population,
not CPU coverage). Use a real small syntax fixture plus declared diagnostic
operations; the census cannot reconstruct node-level edge order, flags or an
actual binder execution trace.

Keep the 24-byte header, text/list policy and page size matched across compact
candidates initially. Expose an immutable list reader disjoint from a restricted
node/binding writer. Preserve short payload borrows and scalar/immediate-child
snapshots across recursive writes. Include functions-first passes, flag/text
reads, narrow symbol/flow/container writes, child enumeration and cold/warm
facts access. Preserve production's sequentially consistent facts ordering;
the earlier relaxed-atomic layout sketch is not a CPU control.

Charge initialization, scratch, directories, runtime-ID/escape storage and
disposal. A binary-expression payload includes five syntax references, facts
and symbol (28 bytes), not just three child words. The pilot's existing Vec
directories do not implement the model's optional one-or-many directory policy.
Resolve these actual storage/caller costs before broad accessor migration.
