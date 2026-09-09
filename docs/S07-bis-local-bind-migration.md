# S07-bis: local dispatch, lists and flow state

This continues the [first local-scope milestone](S07-bis-local-bind-milestone.md)
under the [selected implementation plan](S07-bis-local-bind-plan.md). It is part
of the combined CPU candidate. CP1 remains the accepted control; this record
contains no timing screen, promotion or new Go-relative gate claim.

## Implementation

The shared binder carries a branded syntax target through ordinary dispatch,
container/child dispatch and binary continuations. Local targets read headers and
generated typed payloads directly; checked targets preserve the existing path.
The target is an internal enum, so this still has an adapter branch. It does not
claim that all binder access overhead has disappeared or that all helpers have
been migrated. Unmigrated algorithms receive a raw identity at an explicit
boundary rather than being copied into a second binder.

Container classification has one shared kind/rule inventory. Each adapter
supplies only the dynamic fact that rule requests: method parent, property
initializer or block parent. Fixed rules do not start reading irrelevant
payloads or parents. Open kinds, kind/shape mismatches, signature/static-block
parents and nil-parent failures retain their existing behavior. Flags are read
after head binding and again after recursive child binding where the algorithm
requires live state; they are not cached across mutation.

Source and block traversal keeps the statement-list handle local through both
functions-first passes and visits source EOF afterward. List-specific bind paths
also use resolved local edge ranges. The immutable range can cross recursive
narrow writes, with safe per-element bounds checks, without re-resolving its
backing or copying the whole list. Raw list/slice imports validate ownership,
record kind and ranges before minting handles; exceptional records retain the
checked path. Nil, missing, allocated-empty and nil-element distinctions remain.

Binary continuation frames retain local child handles. Their observations copy
only four optional links from the typed row, removing the trampoline's owned
payload snapshots. The original evaluation, head/error and continuation order
remain shared. Public destructuring/assignment-target and other unmigrated
helpers still cross explicit checked boundaries. The target enum can enlarge
continuation frames; the complete candidate must pay for that cost in its
pipeline and memory measurements.

Current flow, return/break/continue/exception/condition targets and active labels
now carry scoped flow identities. Flow-list identities remain scoped through
antecedent construction. Target reads and narrow writes use safe local slots;
newly allocated identities are minted in the scope. Identifier flow writes no
longer re-import the current flow for every leaf. Public flow payloads and link
reads still have explicit raw-reference conversions; this step does not claim
complete removal of reference codecs or import checks on those boundaries.

The AST API also supplies scoped symbol and table identities and narrow state
services. The binder's general symbol/table/declaration algorithms still use
their checked identities. Table allocation/mutation now exposes only narrow
operations, so a caller cannot replace an arena while its branded handles are
live. No whole flow arena replacement or unrestricted syntax mutation is added.

Entity-name and left-hand-side traversal used by narrowing now stays local.
These helpers preserve name-first short circuiting, nil failures, partially
emitted wrappers and the shared kind predicate. Only malformed shape conversions
take the explicit cold checked helper.

## Review correction

Derived representation equality was insufficient for flow identities. A checked
reference to a not-yet-allocated local slot can become valid after allocation;
its checked and branded representations must then name the same flow. Otherwise
antecedent deduplication and unreachable-flow comparisons can diverge. The binder
now compares local/local handles directly and canonical full identities for
mixed representations. Regression tests cover the allocation transition, mixed
aliases and antecedent deduplication. Missing or foreign links survive parent
record reads and fail only when the existing algorithm dereferences them.

Independent source review found no additional dispatch/list or flow-consumer
regression after that correction. The shape, nil, growth, escape, alias and deep
recursion cases supplement the independent Go workload; they are not counted as
Go oracle observations.

## Validation and remaining work

The affected release checks pass 30 arena, 152 AST and 41 binder library tests,
29 AST integration tests and 37 doctests. The final binder debug run passes all
41 tests, including the alias regressions. Earlier debug checks and their unused
method warning remain recorded. Rust 1.96 checking, denied-warning Clippy and
formatting pass. The first check caught a shadowed flow/node target name; lint
then caught explicit-default style in tests and a nested optional compatibility
result. Those failed logs are retained beside the fixes and passing checks.

Scoped strict-provenance Miri passes three mutable arena-scope and 21 AST local
binding tests. AddressSanitizer passes 21 AST and 13 binder local/binary/flow/
classification tests, including both 20,000-node binary chains on a 512 KiB
native stack. The pinned nightly and existing sysroot were reused, with separate
fresh Cargo output and intermediate directories. The instrumentation source
inventories match before/after and across modes. These are scoped checks, not a
complete ownership-producer or LeakSanitizer claim; Darwin ASan disables leak
detection, while tracked disposal and Miri remain tested.

All 13,094 workload graphs match the pinned Go streams in both worker modes.
Both modes report 161,740,237 loaded bytes, 19,593,488 nodes, 2,459,867 symbols,
423 parse diagnostics and 5,250 bind diagnostics. Separate path probes report
13,094 local-scope/in-place files and zero checked-scope/publication fallbacks.

| Workers | Canonical graphs equal | Raw hashes equal | Local-scope files |
| --- | ---: | ---: | ---: |
| 1 | 13,094 | 13,094 | 13,094 |
| 8 | 13,094 | 13,028 | 13,094 |

The 66 eight-worker raw differences are scheduling-dependent qualified-name
counter bytes recognized and validated by the existing graph protocol. The
selected normal executable is
`0929bdf9db11558def92d5a32b680af073eb15607a5b8371f52cfe288fa94bdf`.
All 486 pre-build source/configuration/protocol files stayed byte-identical
through the isolated build and capture. The 330-file instrumentation inventories
also match the selected source. This step changes no generated output; the
existing all-shape generator remains unchanged.

The [capture archive](../tools/s07/performance-experiments/results/2026-09-09-local-bind-migration/README.md)
retains the executable, exact snapshot, graph/path streams, replay support and
successful/failed checks. The capture wrapper initially omitted two comparator
support files. Their supplemental copies were then verified against both the
prior accepted milestone archive and the current originals. Validation resumed
from the existing one-worker stream. There were exactly two graph captures and
two path probes; no graph workload was repeated after that wrapper failure.

Remaining migration includes declaration/expression binding and shared helper
paths that still use `Binder::n`, general `NodeRead` payloads and raw node
identities. Symbol/table/declaration state is not yet carried through the binder
as local handles. Raw flow payload/link conversion and other general helper
boundaries remain visible. A local scope being selected for every workload file
does not certify that every read inside that scope is local.

The [allocation traffic report](S07-bis-allocation-traffic.md) remains the memory
attribution result. This migration chooses no new memory backing policy. The
complete candidate still needs combined CPU/parse-versus-bind assessment,
complete required ownership/producer checks and fresh final Go-relative
acceptance. No individual component saving is predicted or added to an earlier
measurement.
