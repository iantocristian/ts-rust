# S07 binding operations

Binding publishes one result for one logical parsed SourceFile. Nodes and symbols
retain their stable identities, and parsed views remain available separately.

| Operation | S07 contract |
| --- | --- |
| Bind a parsed source | Supported; one winner publishes its private result. |
| Bind mapped members independently | Supported; another member is neither initialized nor waited on by a read. |
| Bind independent logical sources in one physical arena | Supported; parsed parent chains identify each source and cross-source writes are rejected. |
| Read a completed foreign or sibling source through a bound view | Uses the target source's completed overlay, including source metadata and diagnostics. |
| Read an unbound sibling through an AST view | Returns parsed data and does not initialize binding. |
| Retain a bound node | Requires an unambiguous parsed parent chain and a completed binding result for that source; orphan, cyclic, unbound and failed sources are rejected. |
| Clone or update a SourceFile | Preserves the separate S06 AST factory and shared-list semantics, including the original's completed bound headers and CommonJS marker. Copies start unbound. |
| Bind a shallow transformation result whose children still belong to another source | Outside the current binding-root contract; deferred to transformer integration. |

Local reads in a physical owner with one logical source retain their constant-time
overlay lookup. Cross-owner reads and the uncommon owner with multiple logical
sources follow checked parsed parent links. They do not guess a canonical root
when a node is parentless or its parents form a cycle.

Factory reads select completed binding state for retained imported nodes without
changing ordinary parsed views or initializing an unbound source. The direct
[bound-source witness](../tools/s07/ownership/bound-clone.go) and native factory
regression compare actual bound JavaScript source/function flags, source metadata,
shared children, unchanged-update identity and unbound copy state. Reproduce the
17 observations with `python3 scripts/s07_bound_clone.py --check`.

The final row is an unsupported operation, not an upstream invalid-input panic.
At pinned commit `1f70213d4922b434345f639b441681e470c7cfc1`,
`tsc/internal/ast/ast.go:2814` (`SourceFile.Clone`) shares the statements and EOF
token. `ast.go:2825` (`UpdateSourceFile`) also preserves supplied shared children.
The ordinary update callers are source transformers and `SourceFile.VisitEachChild`
(`ast.go:2784`), including JSX, declaration, ECMAScript and module transformations.

The independent [Go witness](../tools/s07/ownership/clone-bind.go) parses
`let x=1; x;`, clones the source, then binds only the clone. Go 1.27.1 completed
successfully with the following observations:

```json
{
  "cloneBound": true,
  "cloneLocalsHasX": true,
  "originalBound": false,
  "originalLocalsNil": true,
  "sameEOF": true,
  "sameStatements": true,
  "sharedFlowIsNowPresent": true,
  "sharedFlowWasNil": true,
  "sharedStatementParentIsOriginal": true
}
```

Thus Go exposes binding writes through a shared child even while its original
source remains unbound. S07 does not claim clone-plus-rebind parity, and its typed
ownership rejection must never be scored as a matching Go panic. A future
transformer implementation must decide how this shared mutable source behavior
interacts with immutable published binding results before adding that operation.

To reproduce the observation, place the witness in a temporary Go module named
`github.com/microsoft/TypeScript/tsc/s07-clone-bind`, require the upstream module at
`v0.0.0`, and replace it with this repository's pinned `upstream/tsc` directory.
Run `GOTOOLCHAIN=local go run -mod=mod .` with the pinned Go 1.27.1 toolchain.
The witness only uses the upstream public parser, factory and binder APIs.
