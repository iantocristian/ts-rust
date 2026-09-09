# S07-bis: first local binder implementation milestone

Status: first implementation milestone complete; the binder-wide migration and
combined performance screen remain open. CP1 remains the accepted control.
The [selected plan](S07-bis-local-bind-plan.md) still owns both CPU and memory
acceptance; this milestone changes no gate or tracker evidence.

## Implemented behavior

Eligible consuming binds enter one fresh invariant scope for the entire file.
Core headers, all 192 generated typed payload readers, parent/child words and
list descriptors use that scope's namespace. Raw IDs are checked on import;
local reads retain safe slot/page bounds checks. Local syntax, list, text-slice
and flow handles cannot be forged, exchanged between scopes or returned from
the callback. Payload borrows prevent overlapping writes.

The scope admits validated exclusive single-source cores without escaped syntax
references, incompatible list representations or pre-existing side records.
Published, imported, mapped, multi-source and previously lazy owners retain the
existing checked path. Lazy records created after entry cross the explicit
checked adapter. No unrestricted core mutation or whole-flow-arena replacement
is exposed by the scope.

The production consuming binder now uses this entry, rather than leaving the
generated readers as an unused API. Its identifier leaf path uses local flags,
typed identifier text, local parent/name classification and direct flow writes.
The receiver-chain narrowing helper keeps ordinary property/parenthesized/non-null
links local. Immediate child enumeration uses generated local descriptors;
resolved edge ranges survive recursive narrow writes without repeatedly resolving
their backing or copying every list into a temporary vector.

Flags and binding fields preserve the completed syntax proof. The generated flow
setter shares the existing schema capability inventory. It preserves flow-only
versus materialized-empty records, compatibility-record precedence and clears.
A rare checked write removes a preceding full-reference escape before replacing
its word. Public raw-ID writes retain target-before-value validation. Shared
`BindResult` helpers keep the two adapters' validation and promotion logic aligned.

Each binder caches only immutable parse-diagnostic presence and external-module
presence. CommonJS detection, evolving node flags and parse-error propagation
remain live state. Parent/name classification still precedes keyword text reads.

Generated child traversal follows the public kind dispatch and checks the
corresponding shape. Flow capability follows actual shape. Declaration-name
selection shares the existing schema predicate. These differ intentionally in
the pinned API: constructed kind/shape mismatches must not silently acquire a
different traversal or failure. Rare mismatch paths call the checked helper to
preserve its interface-conversion panic.

## Workload and validation

The unchanged 13,094-file workload matches the archived pinned-Go graph results
in both worker modes. Each mode reports 161,740,237 source bytes, 19,593,488 nodes,
2,459,867 symbols, 423 parse diagnostics and 5,250 bind diagnostics. The independent
path probes report **13,094 local-scope files, zero checked-scope files and zero
publication fallbacks** in each mode.

| Worker mode | Canonical graphs equal | Raw graph hashes equal | Files using local scope |
| --- | ---: | ---: | ---: |
| 1 | 13,094 | 13,094 | 13,094 |
| 8 | 13,094 | 13,029 | 13,094 |

The 65 raw differences at eight workers are scheduling-dependent runtime counters
inside qualified names. The existing graph protocol validates their raw name
shape/references, canonical identities, graph hashes, diagnostics, counts and
input recipes. The initial ad-hoc exact-JSON comparison rejected these differences;
that failed check is retained. Replaying the same streams through the existing
comparator passes. Neither full graph capture was rerun to obtain a different
result. The Go streams are hash-verified members of the preceding archived
capture, not fresh Go timing measurements.

Checks cover arena/AST/binder runtime behavior, scope-escape/cross-owner/cross-
namespace compile failures, overlapping row borrows, valid short borrows, page
boundaries, nil/missing/allocated-empty slices, dynamic JSDoc ordering, escaped
text, flow promotion/clear, post-entry lazy records, invalid-ID error order,
panic publication and deep recursion. Explicit malformed-tree regressions compare
the local and checked panic payloads. Parsed contextual-keyword cases compare
diagnostics and header flags across consuming and published binding, including
a clean external module and subsequent independent files.

The fixture review also corrected an eligibility assumption: top-level `await`
reparsing leaves multiple source records even when core storage has no lazy
records. Such sources correctly retain the pre-existing multi-source fallback.
The diagnostic identifying that cause and the earlier failing fixture assertions
are retained.

Rust 1.96 checking, denied-warning Clippy, generator publication and scope
doctests pass. Debug checks pass 29 arena and 144 AST tests; all 30 binder tests
pass after correcting the fixture assumption. Release checks pass all 144 AST
and 30 binder tests. The 31 doctests include the compile-fail scope contracts.
Shared-target Clippy initially resolved stale dependency metadata;
isolated validation resolved that failure. This milestone is not a fresh complete
Miri/ASan ownership-producer claim. Those checks and current producer evidence
remain prerequisites before the complete candidate is screened/promoted.

## Remaining migration and measurement

The ordinary receiver loop in the retained optimized ARM64 disassembly performs
direct local header/typed-row loads and preserves safe bounds checks. General
helper calls remain visible in exceptional and not-yet-migrated branches. This
inspection establishes which implementation was compiled; it predicts no saving.

Remaining checked boundaries are concrete:

- Non-identifier dispatch still converts to the shared `NodeId` worker. Many
  container, declaration and expression operations still use `Binder::n`,
  `NodeRead`, owned field snapshots and general AST helpers.
- The functions-first/list-specific binder paths still use ordinary slice IDs.
  Only generated immediate-child traversal currently carries resolved local
  ranges through recursive writes.
- Binder flow state and symbol/table/declaration services retain their public
  identities; a flow value is imported before a local inline write. Local
  symbol handles and complete local flow-state propagation are not implemented.
- Entity-name and left-hand-side helpers called by narrowing still cross the
  general view. Diagnostic construction also retains its checked boundary.

Continue with those dispatch/state/helper paths as one combined CPU candidate.
No component timing screen, promotion or CPU/MiB tradeoff conclusion is made
for this foundation. The [allocation attribution](S07-bis-allocation-traffic.md)
is complete and names measured traffic, but selects no single rewrite capable
of closing the remaining allocation and RSS deficits.

The [milestone archive](../tools/s07/performance-experiments/results/2026-09-09-local-bind-milestone/README.md)
contains the selected compiler artifact,
337 exact source/configuration files, both graph streams, protocol replay,
path probes, disassembly and successful/failed logs. A subsequent formatting-only
diff removes one blank line and formats the added tests; that diff is retained
separately. This is a diagnostic snapshot, not current-head tracker evidence.
The graph-time source inventory also includes a correction to the `cfg(test)`-
only contextual-keyword fixture made after the normal executable build; the
remaining 336 source/configuration hashes agree with the build inventory.
