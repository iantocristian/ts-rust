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
changed candidate still needs its frozen graph comparison and fixed screen.
