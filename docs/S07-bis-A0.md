# S07-bis A0: exclusive binding pilot

The selected first candidate consumes `ParsedFile`, runs the existing binder on
its reserved worker stack, and publishes a `CompletedFile`. This is an experiment;
the S07 memory and CPU gates remain unchanged and unclaimed.

## Candidate scope

Ordinary single-source parser results bind node headers directly in the existing
core pages. Eligibility is checked before mutation: imported owners, several
logical sources, content mappers, factory hooks and already allocated lazy records
use the existing publication/overlay path. Eager JSDoc caches referring to core
nodes are eligible. The pilot does not add mutable access to committed lazy pages.

`BindBuilder` selects an exclusive or published storage backend. Both run one
binder algorithm. Exclusive node access preserves arena and slot checks; it skips
the overlay map and logical-source parent walk. Symbol, locals and flow fields
still use the existing field maps. Those allocations have not disappeared.

`CompletedFile` and its retained node/symbol handles expose the bound view only.
They do not offer `BoundFile::parsed_file()`, whose existing published-path contract
continues to preserve parsed headers. No completed cache entry is installed until
binding and publication finish. The compiler's weak reuse cache is still exclusively
borrowed; the pilot does not invent a concurrent pending cell or negative cache.
An exclusive initializer error or panic drops its private owner. Published-file
contention, reentry, terminal failure and independent mapped siblings retain their
existing binding-cell behavior.

Recursive binder traversal can no longer hold an immutable syntax borrow across
a direct node mutation. It snapshots immediate child descriptors on the stack
(the schema maximum is nine) and reborrows indexed list elements. It copies no
whole node, string or list backing. The descriptor stack and repeated list access
are replacement costs included in the candidate, not assumed free.

The public mutation surface still permits arbitrary node mutation. Consequently,
requesting the mutable parsed builder invalidates its parse validation proof and
publication validates the entire core again. This additional scan is deliberately
included in A0. A later narrow mutation API or validation consolidation needs its
own correctness evidence and measurements.

## Validation and measurement

The candidate must retain both routes' behavior, preserve full frozen graphs at
one and eight workers, and record direct/fallback counts from the same immutable
normal executable. The `ts_bench --binding-paths` diagnostic runs after the measured
endpoint and uses a separate protocol; normal benchmark output is unchanged.

The experiment runner freezes actual binaries, source/configuration inventories
and all 13,094 inputs. It records graph capture separately from fixed paired
normal/allocation measurements. The accepted original control is never rebuilt
in place. CP0 layout projections do not substitute for measured A0 allocations.

The first review caught fragment-root handling before `BindResult` construction:
a completed non-source fragment must return `InvalidGraph`, matching the published
binding entry point, rather than panic while looking up source metadata. That
validation now precedes candidate selection.

Results and the promotion decision will be recorded after semantic and measurement
captures finish. No performance result is implied by compilation or unit tests.

## Predeclared follow-up: A0-b narrow flag mutation

The initial A0 binary remains frozen at commit `58db8c2`. A0-b adds a narrow
`BindBuilder::set_node_flags` API: only this edge-free write preserves the parsed
validation proof. Public unrestricted `node_mut` still calls `builder_mut` and
invalidates the proof before yielding access. Preexisting dirtiness is never reset.
Final binding-result validation remains mandatory. Installing its validated result
in the source's `OnceLock` does not require unrestricted source metadata mutation.

Hypothesis: removing the redundant post-bind core scan reduces complete-pipeline
wall time while preserving the same bound graphs and all owner checks. This
variant is declared before seeing the initial A0 timing result. It will have its
own immutable build, full graph check and fixed screen. The original A0 outcomes
will remain available, including a failure or regression.

A separate phase probe compares published and consuming binding on one source
revision, grouping binding and publication in the same elapsed interval. Its
numbers explain the change; only the normal binaries determine the pipeline
screen. The proof tests cover fresh and previously dirty parses, foreign narrow
writes, invalid parent/payload mutations and invalid binding metadata.

## Initial A0 result

Both worker modes matched all 13,094 frozen Go graphs and used direct binding for
every file. The fixed screen retained eight warmups and all 56 observations;
replay validation passed. Compared with the separately frozen original control:

| Metric | One worker | Eight workers |
| --- | ---: | ---: |
| Wall median, candidate / control | 1.01159 | 1.00612 |
| Timing bootstrap upper 95% ratio | 1.03507 | 1.05067 |
| Requested-allocation median ratio | 0.98183 | 0.98183 |
| Lifetime peak-RSS median ratio | 0.98733 | 0.98741 |

The observed allocation reduction is 85.93 MB and the one-worker RSS reduction
57.85 MB. There is no demonstrated wall-time win, and both timing upper bounds
exceed the 1.02 non-regression guard. **Do not promote this initial candidate.**
The already-declared A0-b revision is the next experiment; these samples will
not be extended or selectively replaced. Field maps remain in both candidates,
so neither claims the large savings from future inline binding fields.

Local capture: `target/s07-bis/a0-screen`; graph capture:
`target/s07-bis/a0-graphs`. Candidate manifest SHA-256:
`ee3399a930061772ca08d912d8bb1f3283169ef467066c7880f1df6d79fdde94`.
