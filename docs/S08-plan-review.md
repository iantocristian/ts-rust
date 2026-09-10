# S08 plan review

Reviewed 10 September 2026 by Codex, against merged `main` `4c0818d` and pinned
Go source `1f70213d4922b434345f639b441681e470c7cfc1`. This is a review of the
[implementation plan](S08-implementation-plan.md), performed by its author.
It is not an independent review or evidence that checker code has been ported.

## Source and requirement checks

- Read all ten [S08 items](../sprints/S08.toml), their experiment consumers,
  [S09](../sprints/S09.toml), [S10](../sprints/S10.toml), and the current producer
  registrations/CI path. E2 currently verifies only the frozen denominator.
- Read the Rust guide, accepted symbol/ownership contracts, mutation model,
  comparator, recursion, divergence and performance-threshold ADRs.
- Inspected the concrete compiler loader API, checker identity/lease and symbol
  arena primitives, plus the compact binder's local access contract. There is
  no production Rust checker or printer to extend yet.
- Checked all 675 obligation records by family and read their provenance. Their
  111 library identities are actual loader observations; capability records
  explicitly say execution is required on use, not already observed.
- Read the source checker initialization/host interface, type allocation and
  representations, merge ownership contract, resolution-stack implementation,
  relation modes/cache structure, comparator branches, type display and builder
  release behavior. Read the compiler runner and actual type/error baseline
  walkers rather than assuming what `.types` files represent.

The source package inventory contains 60,678 non-test, non-generated Go lines
in 24 checker files and 11,692 in 16 printer files. These figures indicate the
surface to audit, not a mandate to port every function in S08 or a completion
percentage. The source-derived execution closure determines required work.

## Issues addressed in the plan

| Risk or incorrect shortcut | Resolution in the plan |
| --- | --- |
| Treat S08 as only literal/object/union evaluation | Preserve all 10,728 variants, complete loaded libraries and 675 capability obligations. Required generic/conditional/mapped/inference work has explicit checkpoints; unknown operations fail by name. |
| Claim syntax census records prove semantic execution | Keep obligation provenance immutable; join it to actual capabilities and direct supplemental observations. Supplemental tests do not inflate the primary baseline denominator. |
| Assume existing checker lease/symbol arenas form a concrete owner | Name the missing concrete state/retained-result APIs and the identity-reservation mismatch. Require a private one-time adoption path, exact-checker checks, one operation permit and non-owning internal links. |
| Copy the exclusive binder's local namespace directly into a multi-file checker | Separate the validated checker universe from immutable retained file/bundle resolution and lazy AST access. Keep mutation confined to checker-owned state. |
| Choose a uniform type enum and discover its memory cost after porting | Select small common records plus typed payload storage, then measure a real early slice including lists, capacity and transient traffic. No guessed byte size or inherited page constant becomes a target. |
| Let a reference-relater experiment succeed through precomputed graphs or calls into the ID implementation | Require the real algorithm, lazy resolution/allocation where fixtures demand it, Go observations, matched cache state, and early safe-reference feasibility. Its complete measured result is required even if slower. |
| Build generic signatures first and defer their relations to a later independent phase | P3 now grows signatures, inference, instantiation and their required relations together in executable query slices. P4 completes body checking and dependency closure. |
| Add stack protection only when final stress tests are written | Install guards with each recursive production path. P5 completes the adversarial stress inventory and growth/unwind observations. Keep a dispatch-independent entry for S10. |
| Compare just semantic values or normalize away baseline differences | Freeze source walker/query order and distinguish disabled output, `NoContent`, empty content and missing implementation. Compare exact Corsa output separately from old Strada baseline fixups. |
| Implement `TypeToString` as a shortcut formatter | Require real node building and printing, source flags/truncation, diagnostic-triggered serialization, builder generation rotation and emit metadata retention. |
| Certify checker-local merges only because two Rust checkers agree | Compare Go merge/symbol observations, use different additional declarations, and verify that the shared file graph remains unchanged before/after both operations. |
| Reuse S07's relaxed limits for checker storage or invent a checker CPU target | Keep per-type footprint ≤0.80. Checkerbench and relater numerical criteria remain usable-measurement checks. State throughput direction and raw endpoints explicitly. |
| Complete E3/S09 accidentally while adding merge tests | Audit every shared consumer; compose exact inventories and add negative gate tests. Full registry, builder-retention and retirement integration stays pending until its own complete tests run. |
| Replace E5 with a type-only record or silently reuse stale native samples | Compose validated parse/bind and type captures. Schedule a final native refresh if the new Cargo/source closure invalidates S07 evidence; preserve ADR 0021 limits. |
| Repeat expensive profile/trace work before any integrated checker exists | Bound P1 to storage/ownership feasibility. Later diagnostics require a specific unresolved mechanism; measure complete candidates and retain negative outcomes. |

## Implementation-time decisions that remain explicit

1. **Exact source closure and fixture manifests:** P0 must generate and review
   them from the pin. The plan's source inspection is not a completed typed
   closure audit. New required dependencies become work, not exclusions.
2. **Type census accounting:** P0 freezes type/payload/list/cache membership and
   sharing rules before Rust results. P1 records actual capacities and counts.
   Runtime heap endpoints, structural retained storage and requested traffic
   remain distinct. An unaccounted family cannot be assigned zero bytes.
3. **Payload sizes/page policy:** select from the first real Go/Rust census and
   query slice. The plan promises neither a 24-byte type nor a particular page
   count, and does not use type-count inflation to satisfy a per-type mean.
4. **Reference storage feasibility:** P1 must prove safe recursive access and
   relation-time allocation, selecting and reviewing a dependency if required.
   A prototype that cannot execute the frozen contract is unfinished; it is not
   a measured unfavorable result.
5. **Corpus runtime and deterministic CI partitioning:** learn the cost from
   the first full Go capture. Any shards must merge into the exact frozen
   request set and preserve per-program query order. Hosted CI does not supply
   performance acceptance samples.

These are scheduled work with concrete exits, not requests for permission to
skip requirements. No thresholds, divergence approvals, scope changes, source
code or evidence records are changed by this planning task.

## Review conclusion

The plan covers every S08 item and the dependencies identified by source
inspection. It separates actual baseline authority, ownership proof, semantic
parity and measurement. Proceed to P0 before implementation; do not interpret
this review as satisfying P0's generated manifests or later acceptance gates.
