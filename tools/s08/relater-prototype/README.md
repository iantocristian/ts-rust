# S08 relater prototype (P1 feasibility)

The plan (§6.3, ADR 0008) requires an isolated relater built on stable arena
references with interior mutability, measured against the production ID-based
relater in P7. P1 owes the feasibility proof: a safe construction and mutation
API, retained graph and result ownership, failure behavior, and the algorithm's
recursion and assumption stack running on a real recursive fixture.

## What is here

- `Graph` owns every `TypeCell` through `Rc`; edges between types are `Weak`.
  The graph is the only strong owner, so recursive types form no cycle and
  dropping the graph frees every cell (tested).
- Members resolve lazily through a `OnceCell` and a consumed resolver, and the
  resolver allocates records into the graph while an `&TypeCell` borrow is live,
  which is legal because cells never move (tested: four records per fixture,
  exactly one resolution per type, during the first relation).
- `Checker::check_type_related_to` ports the core of `checkTypeRelatedToEx`,
  `isRelatedToEx`, `recursiveTypeRelatedTo`, `resetMaybeStack`,
  `propertiesRelatedTo`, `propertiesIdenticalTo` and `isDeeplyNestedType` for
  object and primitive types in all five relation modes, with one diagnostic per
  failed reported relation.
- No `unsafe`, no lifetime transmute, no leaked storage, no unchecked
  self-reference (`#![forbid(unsafe_code)]`; edges upgrade or fail with
  `Error::Released`).

## What it proves

On the frozen fixtures `recursive-objects` and `recursive-mismatch`
(`tools/s08/contracts/relations.json`) the prototype reproduces the Go-observed
top-level ternaries, boolean results, per-mode cache entry counts and result-flag
multisets, and diagnostic counts for the cold, repeated, reversed and reflexive
actions of every mode (`data/s08/supplemental-observations.json.xz`). A
1,000-level chain relates on a 256 MiB thread and stops at upstream's
100-level assumption limit with the same cache growth.

## Failure behavior recorded

- A panicking resolver unwinds through `catch_unwind`, publishes no partial
  members and caches no relation. Its cell rejects subsequent reads with
  `Error::ResolutionFailed`; a consumed resolver never becomes an empty object.
  Other cells stay usable. Production retires the generation instead (ADR 0012).
- An escaped `Rc<TypeCell>` outlives the graph but its edges do not: following
  one fails with `Error::Released`. The production design therefore couples every
  escaped result to its owner (ADR 0007), which the reference alternative must
  keep: results would be `(Arc<Graph>, Rc<TypeCell>)`, never a bare cell.
- Reading a member whose declared type was never created is
  `Error::UndeclaredMember`; subsequent reads remain failed rather than publishing
  an empty default.

## What it does not do

No measurement, no diagnostics text parity, no unions, intersections, generics,
signatures or index signatures, and no comparison with the production relater.
The `relater-fixtures.json` experiment (21 cases, five modes, setup and relation
intervals, seven samples) is P7 work over this API.
