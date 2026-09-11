# S08 P0: frozen execution contracts

P0 is complete. This checkpoint freezes the work and its native expectations;
Rust checker parity, the full storage census and the reference relater remain
pending. The initial 524-type storage pilot is plumbing, not a performance result.
The implementation continues in [PR #13](https://github.com/iantocristian/ts-rust/pull/13).

## Frozen work and authority

| Contract | Frozen scope |
| --- | --- |
| Primary acceptance | 9,369 variants, unchanged from the owner-approved S07-3 amendment |
| Informational observations | 1,359 variants outside E2: 1,325 executed, 34 native option failures |
| Ordered baseline walker | 1,438,509 calls: 402,402 type queries, 417,960 symbol queries, 389,784 type-node displays, 228,363 symbol displays |
| Required declaration diagnostics | 1,270 variants; transformer and checker emit resolver remain S08 work |
| Types/symbols outcomes, each | 9,167 content, four NoContent, 198 disabled |
| Library obligation audit | All 675 original IDs, all 16 capability families |
| Native supplemental relations | 21 source fixtures × five modes × four ordered actions = 420 calls |
| Residual comparator fixtures | Nine named cases |
| Printer byte fixtures | Eight cases through original-source, synthetic and checker type-display paths |
| Rust ownership/recursion contracts | Ten named production scenarios, execution still pending |

The primary expected bytes and diagnostics remain in the existing authenticated
`tools/s08/results/acceptance-amendment/` archives. `query-contract.json` freezes a
deterministic projection and its hash. A fresh checkout can materialize the
schedule under `target/` without running the whole Go baseline capture again.
The 6.3 MB compressed projection is a disposable cache, not another committed
copy of the baseline archive. Preparation rejects capture mismatches rather than
merely printing them in a review report.

A query range is an observation, not a unique node identity. The consumer must
port the native walker, including reparsed nodes, and compare ordered operations,
locations, flags, absence and exact outcome bytes. It cannot look up one node by
range and omit traversal. Raw native type IDs remain diagnostic metadata because
the earlier observer does not carry a checker identity; they are not normalized
across files. Empty content, NoContent, disabled and unavailable are distinct.

S07's source inventory and acceptance amendment are unchanged. The join finds
that 673 obligations have a library witness still loaded by acceptance. The two
`lib.d.ts` ES5-only structure/token obligations have informational witnesses.
Both remain in the audit; E7/E8 and the original S07 obligation artifact are not
rewritten or silently reduced.

## Typed dependency and host audit

`tools/s08/inventory/main.go` uses the pinned module's existing `x/tools`
dependency, typed packages, SSA with generic instantiation and conservative CHA.
Function-valued arguments and imported project initializers are included. The
static audit target is explicitly linux/amd64 so host build tags do not change
the source inventory. Native execution and eventual four-target CI remain
separate checks.

The frozen closure has 18,548 project functions and 69,327 distinct call sites.
Its dictionary-encoded archive is about 748 KiB. It records target sets,
external boundaries and the complete source inventory. This is a conservative
upper bound, not a claim that 18,548 functions need fresh Rust implementations
or run on the acceptance subset. Package-level homes distinguish the existing
parser/binder/loader, pending checker/printer/declaration paths, harness code and
conservatively included incremental/transport/output paths.

All 134 expanded members of six interfaces are recorded: checker Program and
Host, declaration emit host, printer emit host/resolver and SymbolTracker. Each
has its source signature, navigation to native same-named methods and an explicit
Rust implementation status. A declared Rust trait is not a completed compiler
adapter. In particular, no missing JSX/import-helper/module-specifier/metadata
callback is filled with an empty default. Project-reference callbacks retain
explicit scope guards. Declaration-diagnostic callbacks are pending actual
implementation, not waived as general Phase 3 emission.

There are **77 static sites without a CHA target**. The audit does not conceal
that limitation or call them executed coverage. Every site has a stable identity
and reviewed boundary: runtime cancellation/reflection/locale operations,
iterator continuations, an unused pool-factory override, an overincluded generic
encoder template, pre-checker extended-config caching, source-map host callbacks,
or excluded mapper transport. The review explains their source role and required
implementation checkpoint. A newly unresolved site fails preparation until
classified. Matching method names are navigation aids, not a dispatch proof.
A native integration test independently checks direct calls, instantiated generic
calls, interface dispatch, package initialization and a callback passed to an
external library; a missing root is an error.

## Supplemental observations

All comparisons execute the original Go algorithms through access-only overlays.
The relation observer records native ternaries immediately after `isRelatedToEx`;
it does not implement a substitute relation. Each mode gets a fresh checker,
then cold A→B, immediate repeat, B→A and A→A. Display follows the relation sequence,
and ordinary diagnostics use another fresh checker, so neither prewarms the
cold relation. Ordered diagnostics include keys, arguments, chains and related
information. Cache counts/flag multisets and creation/instantiation counters
record state, not allocator measurements.

The cases cover object members, recursive positive and negative relations,
unions/intersections, tuple flags, signatures/variance, lazy generics, mapped,
conditional, indexed-access, infer and template-literal types, JSDoc imports,
merges, flow-derived return types, literal bytes, numeric edges and native cycle
and instantiation limits. Depths 64, 256, 1,024 and 4,096 also execute relation
and type-node display paths. P6 must run their Rust counterparts on reserved
stacks; the native depth observations do not certify Rust stack safety.

The residual cases construct comparator-domain inputs explicitly: nil and
intrinsic creation order, foreign checkers with the exact native panic reason,
duplicate-name symbols with lazy IDs, reverse-mapped records without symbols or
mappers, deferred and instantiation-expression locations, nested mapper trees,
same-location deferred mappers and nil nodes. These are not claims that a source
program naturally creates those exact records. Actual union constituents are
also compared pairwise, reversed and shuffled ten times before native sorting.

The byte cases preserve escaped/raw spellings, quotes, WTF-8 surrogates, malformed
UTF-8, BOM, NUL, emoji and byte truncation. Original printer output, synthetic
printer output and default/full/short checker display are independently captured.
For example, the long multibyte literal yields 320 bytes by default and 1,202
without truncation. Transport stays explicit hex; no UTF-8 repair or text
normalization hides a mismatch.

One preparation fixture initially used `Map<T>`, which collided with the loaded
standard-library name. It was renamed `UnwrapProperties`; unexpected fixture
diagnostics now fail preparation. This was a fixture-preparation error, not a
Rust divergence or a selected passing result. The later full native captures
agree on every supplemental observation.

## Measurement and ownership decisions

The four authored manifests freeze the decisions before Rust results:

- `type-footprint.json`: the 0.80 gate is unchanged. Aggregate charged type
  storage divided by aggregate logically retained types, then Rust/Go. Charge
  complete concrete records, aliases, owned text/backing, type caches, directory
  capacity, page slack and unreachable occupied slots. Unreachable slots are not
  extra logical types in the denominator. Shared allocations are counted once;
  bound source backing is reported in the bound-input bucket. Unknown backing
  extents or unaccounted families make the census unavailable. Fixed Go struct
  sizes and allocator heap endpoints do not substitute for this census.
- `checker-workload.json`: exact native lifecycle and ordered work, serial
  variants, prepared bound inputs outside each checker interval, seven fixed
  measured samples per runtime plus one excluded warmup, deterministic
  alternation. Separate normal timing, exclusive phase timing, allocation and
  retained checkpoints. Output digests/counts prove the full work ran. Report
  unstable batches; no repeat-until-good sampling. Checker metrics still have
  measurement-presence gates, not new performance thresholds.
- `relater-fixtures.json`: the same source-backed cold/repeated lazy work for the
  production ID relater and isolated reference/interior-mutability alternative.
  Setup and relation costs are separate; throughput is reference/ID. No
  precomputed matrix, delegation, leaked storage or bypass of lazy allocation.
  P1 must prove a safe construction/mutation API before the full experiment.
- `ownership-fixtures.json`: independent merges over shared bound inputs,
  foreign same-slot identities, escaped result and original-node lifetimes,
  real-query reentry, ordinary contention, panic after stack growth, exhaustion,
  depth limits and cold caches. These are Rust design obligations, explicitly
  pending execution. They cannot make S09 or the whole E3 harness pass.

The seven newly noted dead-code suppressions have a P2 removal deadline. The
record also identifies the two alias-display suppressions and the test-only ID
exception, so removing seven lines cannot hide remaining deferred state.

## Verification and next checkpoint

The committed verifier checks source and native fingerprints, exact artifact
inventories/counts, all obligation IDs, expanded interface members and static
boundaries, projected query order, supplemental state and the methodology's
agreement with the unchanged experiment ledger. Adversarial tests cover every
output class: missing/reordered work, forged success/counts, broken cold/repeat
state, omitted diagnostic chains, wrong panic reasons, malformed bytes,
incomplete ordering matrices and unusable measurement denominators. Independent native closure and supplemental captures reproduce the frozen
semantics exactly. All 13 focused tests pass, including the opt-in typed Go
integration test. The full self-test producer passes 64 tracker tests and 356
Python tests (the native integration test is opt-in there and was run separately).
The archive also retains the exact native commands, requests,
raw observations, logs and access bridges.
Reproduction commands and the compact evidence archive are listed in
[`data/s08/README.md`](../data/s08/README.md).

The next checkpoint is P1: complete owner-bound result handles and lifetime
contracts; implement the actual Go per-family census and extend the production
storage pilot through compound types, signatures, aliases, lists and caches;
prove the reference relater can perform lazy allocation safely. Then proceed to
the first complete query slice. No further optimization experiment is justified
by the tiny intrinsic/string pilot alone.
