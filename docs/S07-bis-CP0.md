# S07-bis CP0: executable layout and request budget

Date: 2026-09-08. This is a diagnostic result for the
[performance plan](S07-bis-performance-plan.md), not acceptance evidence.
The model executes safe Rust layout and link tests, consumes the archived S07
memory captures, and emits no tracker metrics. Production storage is unchanged.

## Decision

With A0-b retained as the new control, the next bounded layout experiment is a
safe word-class accessor/storage pilot. The actual core census now prices this
option at 830.89 MB for modeled syntax, binding and directories, leaving only
49.11 MB before escapes, runtime IDs and other unknowns. Its CPU cost is unproved.
Do not start broad accessor migration or declare compact storage feasible yet.

The straightforward 32-byte-header candidate exceeds the syntax/binding ceiling
before payload page slack or escape storage; reject that particular layout.
The 24-byte-header candidate's actual file/shape occupancy and lean page policies
are now priced in the [follow-up projection](../tools/s07/performance-experiments/phases/layout-projection.md).
Even thin per-shape Box pages miss the combined ceiling at 887.49 MB. Sharing
15 nonempty payload word widths reduces directory and page costs; all-atomic
word rows avoid a separately indexed facts table, but require measured accessor
and traversal tests before choosing that design. Explicit escape and runtime-ID
budgets still remain.
The entire native-live residual and temporary-traffic target also remain open.
These findings constrain CP3; the normal-binary A0-b result is recorded separately.

## Reproduce

```sh
python3 tools/s07/performance-experiments/layout_model.py
python3 -m unittest discover -s tools/s07/performance-experiments -p test_layout_model.py -v
```

The first command writes `target/s07-bis-layout/report.json`, the exact generated
Rust source, a size probe executable and the link/layout test output. It reads
four named files from the committed memory archive and checks each against its
recorded size/SHA-256. It requires the initialized upstream schema and a working
`rustc`; it neither downloads a toolchain nor rebuilds the workspace. Effective
compiler, generated AST/schema inputs, archive inputs and executable hashes are
recorded. `--rustc` and `--output-dir` are explicit overrides.

The checked-in [initial result](../tools/s07/performance-experiments/layout/baseline-model.json)
records Rust 1.97.1 on `aarch64-apple-darwin`. Layouts on the other three native
targets have not yet been measured. The baseline accounting uses archived
repetition 0 at each worker count; it does not call one sample a new median.

## Compiled shapes and weighted census

The model reads all 192 concrete payload structures and deferred fields from
`data_generated.rs`, and checks their inventory against the pinned upstream
schema. It preserves the separate open `i16` syntax kind and `u16` payload-shape
tag. Each syntax payload is a separate `repr(C)` sketch with compact links,
eight-byte text ranges and composite-only facts; each bound payload adds the
schema's applicable symbol, local, container and flow fields. The `any` checker
field on `SyntheticExpression` remains explicitly unresolved. No new enum's
maximum variant size is charged to every token.

| Component | Measured layout |
| --- | ---: |
| Compact-parent header | 24 / 4 bytes |
| Full-parent control header | 32 / 8 bytes |
| Ordinary source-range text handle | 8 / 4 bytes |
| Identifier syntax / bound payload | 8 / 12 bytes; both alignment 4 |
| Empty token syntax / bound payload | Both zero bytes, alignment 1 |
| Per-shape `Vec` header | 24 / 8 bytes |

There are 19,593,488 core nodes and 253,333 additional overlay copies in the old
shape census. The old collector did not split their payload shapes. The model
removes exactly that many records and computes lower/upper weighted bounds by
removing the largest/smallest eligible sizes. It does not count overlays as new
syntax, invent their shape distribution, or use Go's smaller reachable census
as the Rust denominator. The bounds are mathematical extrema, not predictions
of which particular payloads will disappear.

Compact syntax payloads occupy 224.2–233.9 MB of used storage; including inline
binding fields gives 297.5–312.7 MB. These are projected payload bytes, excluding
their future page slack, page directories and full-range fallback structures.

| Candidate using current header-slot capacity | 24-byte header | 32-byte header |
| --- | ---: | ---: |
| Header storage | 503.2 MB | 671.0 MB |
| Existing header page directory capacity | 8.6 MB | 8.6 MB |
| 192 shape `Vec` headers × 13,094 files | 60.3 MB | 60.3 MB |
| Total with deduplicated used bound payloads | 869.7–884.9 MB | 1,037.4–1,052.6 MB |
| Upper total if payload spare matched current header spare fraction | 906.8 MB | 1,074.6 MB |

The last row is a sensitivity calculation, not measured per-shape occupancy.
Even the preceding row omits future payload page directories, escape maps,
runtime IDs and compatibility storage. The syntax-only 24-byte upper subtotal
is already 806.1 MB against its 780 MB sub-budget. The fixed directory consumes
4,608 bytes even for an otherwise tiny file. CP3 should compare a compact shape
directory and growth policy before choosing this storage arrangement. The
report also records the cost of giving each rare shape a 256-slot page, without
assuming that this would be an acceptable initial allocation policy.

## Full-range compression obligations

The executable link sketch keeps ordinary local links in a `u32` and uses a
separate exception bitmap plus full-ID map for foreign/lazy references and the
local `u32::MAX` slot. No public slot bit is stolen. Round-trip tests cover both
halves of the slot domain, `u32::MAX`, maximum arena identity, nil, foreign and
lazy links, and checked out-of-range access. Header tests pair an arbitrary open
kind with an independent shape tag and preserve negative positions.

This proves an encoding option, not the production ownership or performance
contract. The toy `Vec` is where sketch words live; production words would be
the already charged header/payload fields. The bitmap and map are additional
storage. One all-node bitmap alone is 2.45 MB before per-file rounding; an added
all-node `u32` index would be 78.37 MB. Escape counts, map controls, source/pool
ownership, full-range text fallbacks and runtime-ID behavior need actual designs
and counters. The result leaves them unknown instead of assuming zero.

## Native accounting and traffic

The archived one-worker run reconciles exactly:

`4,728,501,051 = 3,971,798,478 - 166,647,126 + 923,349,699`

The terms are pipeline requests, endpoint live, starting live, and freed or
superseded requests. These are native requested-byte counters, not RSS, physical
copy volume, allocator usable bytes or Go heap objects. Successful realloc can
charge a full replacement request without a physical copy.

At 1,700 MB endpoint live and the same starting counter, 350 MB of traffic gives
1,883.353 MB of pipeline requests. The independent 1,900 MB ceiling permits
366.647 MB of traffic. The required changes are therefore 2,271.8 MB less live
storage, 2,828.5 MB fewer requests, and 573.3 MB less freed/superseded traffic
against the internal targets. Every candidate must remeasure its starting live
counter; these equations must not hide changed preload or retained scratch.

| Disjoint one-worker global phase counters | Requests | Live growth | Freed/superseded |
| --- | ---: | ---: | ---: |
| Parse | 2,849.320 MB | 2,385.319 MB | 464.002 MB |
| Bind | 1,872.267 MB | 1,412.919 MB | 459.348 MB |
| Publish | 6.914 MB | 6.914 MB | 0 MB |
| Outside measured phase intervals | 320 bytes | 320 bytes | 0 bytes |

The initial traffic ceilings remain parse 200 MB, bind 125 MB and
publication/scheduling 25 MB. The eight-worker whole-pipeline identity is also
checked, but its overlapping phase intervals are unavailable for this purpose.
No missing phase value becomes a measured zero.

The separate source-scope capture identifies list compaction, cooked scanner
buffers, string backing, map insertion and arena/directory requests. Its parse
source union leaves 630.067 MB unclassified and its bind union 32.143 MB.
Individual source rows are inclusive and must not be added or described as
freed traffic. In particular, 3.1 million list backings do not establish that
every `Vec`-to-box conversion copies. CP3/CP4 must measure and address necessary
list/buffer/map request changes alongside the new retained layout; CP6 only
handles the remaining costs.

The old census reconciles 3,702,577,529 bytes of visible capacity plus an
11,740,848-byte Arc-header estimate. It leaves **257,480,101 native live bytes
unattributed**, exceeding the complete 50 MB other/unknown budget. Keep this
residual visible while adding map-control, directory and ownership accounting.
The roughly 480 MB difference between the runtimes' RSS-minus-live domains
remains a separate question, outside this requested-byte layout model.

## Validation and next measurements

Nine Python checks cover exhaustive overlay-bound counterexamples, generated
schema drift, unsupported types, changed starting-live accounting, corrupted
counter identities, overlapping-worker phase rejection and strict JSON parsing.
Three compiled Rust checks cover identity boundary round trips, open kind/shape
independence and checked bounds. All pass on the host above.

The follow-up census separates physical core shapes from the old merged overlay
counts and supplies actual per-file occupancy. It does not measure replacement
capacities: those are explicit policy projections, including every directory
growth request. Lazy/fallback occupancy, name-backing modes, exception/control
allocation and actual replacement storage still need measurement. Keep
feasibility unproved until those costs and the native residual reconcile.
