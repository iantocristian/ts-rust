# S07-bis integrated compact storage

This is the implementation record for the typed-storage candidate selected in
[S07-bis](S07-bis-performance-plan.md). The retained control is CP1, manifest
`3a57976f667b9857891edbe9f76de5261c43480ac8c62d946b4545d9ee19d931`.
The lookup-reuse candidate remains rejected. Expanded tracing and replay remain
on hold. This record does not change any sprint or performance gate.

## Physical representation

All 192 core syntax shapes use a 24-byte header and generated typed payload rows.
The header stores independent kind and shape tags, flags, signed int32 positions,
a local parent word, and a payload ordinal. Its internal shape flag records
explicit binding-record presence without changing the concrete shape. Owned
`Node`/`NodeData` remain factory construction values; their sizes do not describe
retained core nodes.

Payload rows use four-row typed pages and ordinary scalar fields. Only the 72
composite shapes carry an atomic subtree-facts word. Applicable binding fields
are generated from the audited 84-shape inventory, including the direct
fallthrough-flow field. The five shapes with no syntax fields but with binding
fields receive distinct mutable rows. Truly empty rows need no allocation.

Core headers currently retain the existing arena policy (2, 4, ... 256 slots,
then 256-slot pages). This is not the model's fixed 32-header-page candidate;
its actual slack and directory cost must be charged by the integrated run.
The model is not an allocation or RSS result for this implementation.

Local node and auxiliary references use u32 words. Zero remains nil, and the
maximum word selects a field-specific full-ID escape for foreign namespaces or
the full u32 slot boundary. Public IDs stay owner-qualified. Identifier text uses
an even source-suffix length or an odd exception-pool handle; all other text
uses the pool. Finalizing a matching identifier range releases its temporary
pool slot, which later identifiers can reuse. Changing its range cannot change
its semantic bytes. Extended pool handles preserve the full supported domain.

Core syntax lists store u32 edges in 256-word pages, preserving stable auxiliary
backing identities, copied slice headers, nil versus empty, and overlapping
ranges. Reads resolve their physical owner and backing before indexing. Parser
construction still uses temporary vectors: removing the final boxed backing
is not the same as eliminating this temporary request traffic.

## Publication and mutation

Exclusive parsing and binding borrow the core's typed storage directly. Narrow
header and binding setters avoid reconstructing payloads; binding setters check
new link identities while retaining the parse proof. The result's symbol, table,
and flow namespaces are distinct from the syntax namespace. Ordinary eligible
core binding fields need neither the old binding map nor paged flow slots.

Already-published compatibility binding keeps cold side records. Lazy nodes
created under the publication lock retain owned payloads; their node and
auxiliary records publish together. A retained lazy node holds that payload
before later reads, so reading it cannot reenter the lazy lock. Lazy nodes created
during exclusive binding use the same checked cold binding path.

`NodeRead` carries physical-owner context; shared read helpers use sealed
`NodeAccess`. `BindResult` observations take an AST view rather than maintain a
node-to-row locator. Escaping text and nodes still require explicit ownership.
Unrestricted `NodeMut` edits stage one construction value and write it back when
the guard drops. Same-shape edits preserve binding fields and caches. Rare shape
changes preserve otherwise unrepresentable binding values in a cold override;
already-computed facts crossing a noncomposite shape are parked only for explicit
owned conversion and mutation, without adding a hot lookup to facts reads.
Runtime identity is assigned lazily in an owner-side map. Replacing a complete
node with a fresh construction value resets that comparison identity as before.

## Review and validation

Independent backend review found and reproduced three compatibility regressions:
subtree facts across a composite/noncomposite/composite mutation, runtime identity
after complete node replacement, and staged mutation's owner-error precedence.
All three have fixes and regression tests. Binder depth testing also exposed an
unguarded recursive initialized-pattern helper: its 464-byte debug frame at
1,200 levels exceeds the unchanged 512 KiB test stack. It now uses the existing
stack-growth guard; neither the test depth nor stack limit was relaxed.

The completed focused checks include 24 arena tests, 86 AST library tests,
subtree and other AST integration tests, seven compile-fail ownership doctests,
25 parser tests, and 28 binder tests. Workspace all-target/all-feature Clippy with
warnings denied, Rust 1.96 all-target compilation, formatting, generator drift,
and generator tests pass. The frozen candidate also passed the 13,094-file graph
comparison at one and eight workers, with all files binding in place and zero
fallbacks. These checks do not replace the broader ownership producers or final
Go-relative performance gates.

## First integrated screen: memory reduction, CPU regression

Candidate manifest:
`3f7e1f072e9d4e10fa247aefa0485ba68ebcba58333e11d5fe3bdf3ef357373e`.
The fixed screen retained all eight warmups and 56 measured children. Receipt
verification passed. CPU MAD is below 1% in both variants and worker modes; the
regression is not a noisy borderline result.

| Metric | CP1 control | Compact candidate | Candidate/control |
| --- | ---: | ---: | ---: |
| One-worker wall | 4.524 s | 7.265 s | 1.606 |
| Eight-worker wall | 1.053 s | 1.624 s | 1.543 |
| Allocated bytes, worse median | 4.643 GB | 3.185 GB | 0.686 |
| Peak RSS, worse median | 4.509 GB | 2.837 GB | 0.629 |

The wall-ratio upper 95% bootstrap bounds are 1.635 and 1.584. This candidate
**cannot replace CP1**: its measured memory saving comes with a large CPU
regression. The raw capture remains preserved while a bounded repair of the new
read path is tested; none of the acceptance thresholds changes.

Distance to the historical Go planning limits remains explicit:

| Metric | Candidate | Historical gate limit | Remaining reduction |
| --- | ---: | ---: | ---: |
| One-worker wall | 7.265 s | 2.938 s | 4.327 s |
| Eight-worker wall | 1.624 s | 0.634 s | 0.990 s |
| One-worker allocated bytes | 3.184523 GB | 2.035226 GB | 1.149296 GB |
| Eight-worker allocated bytes | 3.184523 GB | 2.035767 GB | 1.148757 GB |
| One-worker peak RSS | 2.834252 GB | 2.208948 GB | 0.625304 GB |
| Eight-worker peak RSS | 2.837283 GB | 2.217045 GB | 0.620238 GB |

These rounded historical denominators are planning comparisons, paired with the
same worker mode. Final E5/E6 acceptance uses the worse worker-mode memory ratio
and still requires fresh qualified Go/Rust captures.

A single Time Profiler capture of the exact normal candidate executable produced
8,165 running-CPU rows, including 7,215 ms on the worker. It adds no source hooks,
adapter or field replay. It identifies new read overhead: `AstView::node`,
`NodeRead::resolved`, and `StorageView::node_here` account for 594, 537, and 352 ms
of self samples, respectively. `AstPayloadStore::read` contributes another 272 ms
and `NodeRead::data` 99 ms. These samples justify inspecting the new routing and
known-shape payload-selection paths; they do not establish achievable savings.
The normal binary has no phase wrappers, so this capture does not claim exact
exclusive parse/bind attribution. Field-key hashing also appears in self samples
and warrants checking whether supposedly exceptional operations run on ordinary
local-field writes.

The subsequent [hashing audit](../tools/s07/performance-experiments/results/2026-09-09-compact-typed-first/fieldkey-audit.txt)
attributes a 423 ms stack union to field-key hashing, including its nested hasher
frames. The pinned `HashMap::remove` hashes before checking table emptiness.
Ordinary node construction, parent updates and inline binding writes were calling
it to clear nonexistent escapes. An explicit empty-map guard preserves nonempty
overwrite cleanup while avoiding that work. Tests cover foreign/full-slot to
local/nil overwrites in all five reference namespaces, with another escape still
present, and reuse of an allocated-but-empty map. This diagnoses a removable
operation; the combined repair's pipeline effect remains to be measured.

## Bounded read-path repair

Already-checked exclusive core reads now construct their borrowed `NodeRead`
directly. Generic resolution keeps its lazy path separate, and source metadata
is stored only where it is not already present in the core context. This retains
the owner-before-slot error order and the lazy publication guard.

Generated `NodeDataSource` selectors borrow that read and resolve only the
requested typed row after checking the actual shape. Known-shape visitors,
factory updates and handwritten field readers use them; exhaustive `data()`
matches keep their existing semantics. Shape remains independent of public kind.
The empty exceptional-reference cleanup described above is part of the same
candidate. No new profiling hooks or field replay are introduced.

Independent review of these changes found no actionable correctness issue.
Validation passes: 88 AST, 28 binder and 25 parser library tests; 11 arena/AST
ownership doctests, including explicit direct-selector lifetime protection;
workspace all-target/all-feature Clippy with warnings denied; Rust 1.96 all-target
compilation; formatting; pinned generation and observer drift. The first lint
failure and its narrow generator correction are retained in the logs. The
changed candidate passed its frozen graph comparison at both worker counts, with
13,094 files binding in place and zero fallbacks. Its fixed screen and receipt
verification also completed, retaining all eight warmups and 56 samples.

| Metric | Same-screen CP1 | Repaired compact candidate | Candidate/control |
| --- | ---: | ---: | ---: |
| One-worker wall | 4.363 s | 6.099 s | 1.398 |
| Eight-worker wall | 0.968 s | 1.315 s | 1.358 |
| Allocated bytes, worse median | 4.643 GB | 3.185 GB | 0.686 |
| Peak RSS, worse median | 4.509 GB | 2.837 GB | 0.629 |

Candidate manifest:
`2e6b9eab562633c98c47ae5465abf3fa07e7da9370617ff1001d0865d9f9d1b9`.
The [second review archive](../tools/s07/performance-experiments/results/2026-09-09-compact-typed-repair/README.md)
retains the changed binary/source, complete graphs, samples and exported profile;
the unchanged CP1 bundle is referenced from the first archive.
The wall upper 95% bootstrap bounds are 1.408 and 1.376, with both variants' MAD
below 0.6%. **Still not promoted.** The controls differ between captures, so the
two candidate medians must not be subtracted as a paired measurement of the
repair. The remaining same-screen CPU regression is about 40% / 36%.

| Metric | Repaired candidate | Historical same-mode limit | Remaining reduction |
| --- | ---: | ---: | ---: |
| One-worker wall | 6.098883 s | 2.937627 s | 3.161256 s |
| Eight-worker wall | 1.314566 s | 0.633963 s | 0.680603 s |
| One-worker allocation | 3.184523 GB | 2.035226 GB | 1.149297 GB |
| Eight-worker allocation | 3.184523 GB | 2.035767 GB | 1.148756 GB |
| One-worker peak RSS | 2.834317 GB | 2.208948 GB | 0.625369 GB |
| Eight-worker peak RSS | 2.837316 GB | 2.217045 GB | 0.620270 GB |

A second single native CPU sample uses the repaired normal binary. Its worker
has 6,083 ms sampled CPU; `AstView::node`, `BindBuilder::node`,
`StorageView::node_here` and `for_arena` contribute 649, 265, 320 and 120 ms self
samples. No exact phase attribution or removable-cost sum is inferred. The
remaining repair therefore targets the physical read descriptor: borrow the
already-selected owner and defer payload context until a payload is requested.
Transaction reads must preserve their split owner borrows and lazy guard rules.

Completion validation still constructs `NodeDataRead` and immediately rematches
it. Generate stored-row reference validation directly while preserving every
check and its order. Restrict text-range dispatch to suffix-capable identifiers;
other text fields are independent of node ranges. This forms the next changed
candidate. Keep small typed pages and boxed public construction inputs unchanged
until their costs warrant a separate decision. CP1 stays the control; neither
completed compact screen qualifies for the infrastructure exception or promotion.

Text attribution identifies another new cost: converting raw-word `NodeText`
into owned `JsString` reclassifies the source slice, whereas the old owned text
cloned its existing validity tag. Of 240 ms under `from_utf8` in this sample,
136 ms occur under compact owned conversion, including 83 ms in contextual
keyword checking. Keyword lookup only needs borrowed bytes, so keep the text
borrow scoped to that lookup and release it before diagnostic mutation. Leave
declaration-name ownership and the S04 string contract intact. Ordinary borrowed
compact text reads already avoid classification. This caller change belongs
to the combined repair, not a separate claimed CPU win.

The thin physical-owner implementation is complete. A build-artifact size check
measures `NodeRead` at 40 bytes (previously 56) and `NodeDataSource` at 16 bytes.
Core and transaction reads borrow their selected physical owner/transaction;
lazy reads retain their publication guard. No source clone, owner retention or
wider import permission is added. Direct row validation preserves parent-first
and schema callback order, including nil handling and immediate error return.

Independent review finds no actionable ownership issue. The 89 AST, 28 binder
and 25 parser library tests pass, including the newly added validator regression
and raw/cooked contextual-keyword fixture. Eleven arena/AST ownership doctests,
workspace all-feature/all-target Clippy, Rust 1.96 all-target compilation,
generation/observer drift and formatting pass. These are correctness and size
checks. Its frozen full-workload graph comparison and fixed screen have now
completed, with all 13,094 files binding in place and zero fallbacks in both
worker modes. Receipt verification passes; all eight warmups and 56 samples
remain intact.


## Thin-owner result and one row-allocation trial

| Metric | Same-screen CP1 | Thin-owner candidate | Candidate/control |
| --- | ---: | ---: | ---: |
| One-worker wall | 4.909 s | 6.411 s | 1.306 |
| Eight-worker wall | 1.380 s | 1.802 s | 1.306 |
| Allocated bytes, worse median | 4.643 GB | 3.185 GB | 0.686 |
| Peak RSS, worse median | 4.509 GB | 2.837 GB | 0.629 |

Candidate manifest:
`5a96a275e555e3156520d02c90b02866a55f2323ac080cb8e4cde1e48a8a2750`.
The [third review archive](../tools/s07/performance-experiments/results/2026-09-09-compact-thin/README.md)
retains the frozen sources/binaries, complete graph streams and all measurements.
Both CPU modes still fail: upper 95% bootstrap ratios are 1.374 and 1.337,
with relative MAD below 2% for both variants. **Not promoted; CP1 remains the
control.** Both variants are slower in this capture than the preceding one;
compare their same-screen ratios, not candidate medians across captures.

| Metric | Thin-owner candidate | Historical same-mode limit | Remaining reduction |
| --- | ---: | ---: | ---: |
| One-worker wall | 6.410711 s | 2.937627 s | 3.473084 s |
| Eight-worker wall | 1.802183 s | 0.633963 s | 1.168220 s |
| One-worker allocation | 3.184524 GB | 2.035226 GB | 1.149298 GB |
| Eight-worker allocation | 3.184527 GB | 2.035767 GB | 1.148760 GB |
| One-worker peak RSS | 2.834350 GB | 2.208948 GB | 0.625402 GB |
| Eight-worker peak RSS | 2.837283 GB | 2.217045 GB | 0.620238 GB |

Test one allocation-policy change next: an ordinary `Vec<T>` for each populated
shape, replacing four-row boxed pages. Row references borrow the owner; exclusive
mutable construction cannot grow storage while those references are usable.
Published core rows never grow, and lazy payloads use separate storage. Stable
addresses during exclusive growth were an internal test condition, not a public
requirement. Preserve ordinal bounds, atomic facts and every owner/lazy lifetime
check. Keep normal vector growth, without workload-specific reserves or sizing.

This candidate can remove the page directory and per-four-row allocations, but
vector growth adds requests and capacity slack. Measure both costs with the same
full graph comparison and fixed pipeline screen against CP1. No page-size matrix,
field trace or replay harness is added. If it also fails, reassess the integrated
representation's remaining costs before any further policy tuning. Memory savings
alone do not qualify a CPU-regressing implementation for promotion.
