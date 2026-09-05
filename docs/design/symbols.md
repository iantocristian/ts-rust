# Design note: symbol, type and signature ownership

Backs ADR 0007. Status: reviewed and accepted by the owner on 2026-09-05. Citations are to the pinned checkout, paths relative to `tsc/internal`.

## 1. What Corsa does

1. **The binder owns a file's symbols, flow nodes and declaration lists.** `Binder` has `symbolArena`, `flowNodeArena`, `flowListArena` and `singleDeclarationsArena` (`binder/binder.go:77`); `newSymbol` allocates from `symbolArena` (`binder/binder.go:132`). `BindSourceFile` runs once per file (`ast/ast.go:2895`, `BindOnce`). Symbols point at their declarations, members, exports, parent and export symbol (`ast/symbol.go:10`); `SymbolTable` is a plain `map[string]*Symbol` (`ast/symbol.go:45`).
2. **Binding happens before programs exist and the result is shared.** The parse cache binds on entry (`project/parsecache.go:74`); incremental builds, auto-import and source-definition code bind files outside any program (`execute/incremental/programtosnapshot.go:236`, `ls/autoimport/registry.go:1489`, `ls/sourcedefinition.go:450`).
3. **Merging is checker-local.** The checker has its own `symbolArena` (`checker/checker.go:669`) and allocates transient symbols with `newSymbol` and `newSymbolEx` (`checker.go:14285`). `mergeSymbol` mutates the target only if it is transient; otherwise it clones the resolved target with `cloneSymbol` first (`checker.go:14361`, `14552`). `cloneSymbol` copies the declaration slice with a clamped capacity, the parent, the value declaration, and `maps.Clone` of members and exports, then `recordMergedSymbol` writes `c.mergedSymbols[source] = target` (`checker.go:14581`); `getMergedSymbol` consults that map (`checker.go:14564`). Two checkers therefore merge independently over the same files.
4. **Types and signatures are per checker, but results can outlive a lease.** `Type` carries a checker-local `TypeId` and a back-pointer to its checker (`checker/types.go:680`); `Signature` has a checker-local `SignatureId` and references other checker-owned objects (`types.go:1290`). The compiler's pool says it plainly: "it isn't possible to mix types obtained from different checkers" (`compiler/program.go:586`). The language-service API returns symbols and types after releasing its checker lease (`ls/api.go:18–43`), and the pool reuses idle query checkers across requests (`project/checkerpool.go:296`). The dedicated API checker is never idle-cleaned so that handles keep reference identity (`project/checkerpool.go:38`). Request lifetime alone therefore cannot define type provenance or storage lifetime.
5. **Links are side tables.** The checker keeps 26 `core.LinkStore` tables keyed by node or symbol pointer (`checker/checker.go:674` onward) and two arena-indexed variants, `nodeLinkStore` and `symbolArenaLinkStore`, for its hottest tables (`checker.go:676`, `685`). The index-keyed form already exists upstream.
6. **Handles on the wire.** `SymbolHandle` is `ast.GetSymbolId`, a lazily assigned process-global counter (`api/proto.go`, `ast/utilities.go:34`); the snapshot registers symbols snapshot-wide and panics on a collision (`api/session.go:184`). `TypeHandle` is the checker-local `TypeId`, registered per project (`api/session.go:230`), which is why a rebuilt checker reusing ids would collide (ADR 0012).
7. **The checker also owns synthetic AST storage.** Its `factory` is an `ast.NodeFactory` distinct from the node builder (`checker/checker.go:673`). `newCallSignature` creates and retains a synthetic declaration (`checker.go:30869–30871`), and `createSyntheticExpression` embeds a checker type in an AST node (`checker.go:30466–30470`). Keeping only the program's source files does not retain these nodes or their type dependencies.

## 2. Contract

### 2.1 File-owned binding

Binding produces a `BindResult` owned by the `FileOwner` through `OnceLock::get_or_init`. The winning initialization closure performs the bind and publishes the completed result; competing callers wait rather than running additional binders that mutate the same file. This matches `BindOnce`. It contains:

- `symbols: Arena<Symbol>` in its own arena with a process-global `ArenaId` (same counter as node arenas, ADR 0006);
- `flow_nodes`, `flow_lists`, `declaration_lists`, each an arena;
- `symbol_of_node`, a slot-indexed table from declaration node to symbol slot, replacing upstream's `Node.Symbol()` accessor;
- the file's locals and exports tables, `HashMap<JsString, SymbolId>` (ADR 0013 for the key type).

Everything in the bind result is immutable after publication. Programs and snapshots share it by sharing the `FileOwner`.

### 2.2 Symbol ids

A `SymbolId` uses the same encoding as `NodeId`: a full 32-bit `ArenaId` in the high bits and a 32-bit slot in the low bits. The Rust wrapper stores a `NonZeroU64`; slot 0 is forbidden for live symbols and the zero encoding is the `Option` niche. There is no owner-kind bit. The never-reused process-global arena counter and checked exhaustion rule are shared with node arenas; allocation fails before the counter can wrap. In particular, crossing `ArenaId = 2³¹` does not change a symbol's owner kind or alias another arena.

The resolver's arena metadata records whether a symbol arena belongs to a file or a checker. Checker metadata also carries the exact `CheckerId` and pool-generation state. `CheckerId` is the never-reused `ArenaId` of that checker's symbol arena; it is distinct from the pool generation, since multiple checkers can belong to one generation. A file symbol id is stable across programs sharing that file. A checker symbol id is usable only through its owning checker while that generation is active.

Declarations, parents and export symbols inside a merged or transient symbol are non-owning logical ids. They can refer to the program's files, mapped siblings retained as a bundle, or the checker's own synthetic AST storage. The checker retains all of those dependencies through its `CheckerOwner`; retaining source files alone is insufficient. An external file-symbol result retains the applicable file or bundle retention group, and an external checker-symbol result retains the `CheckerOwner` (section 2.4).

### 2.3 Checker-local symbols, merging and types

Each `CheckerOwner` owns its storage, one exclusive operation permit, and its node-builder context, as specified by ADR 0006. Its storage includes:

- `symbols: Arena<Symbol>` for transient symbols, with its own `ArenaId`; `clone_symbol` and `merge_symbol` port line by line, including the "mutate only if transient, else clone the resolved target first" rule and the unidirectional flag;
- `merged: HashMap<SymbolId, SymbolId>`, the `mergedSymbols` map; `get_merged_symbol` is the same lookup;
- `types: Arena<Type>` and `signatures: Arena<Signature>` with private `TypeSlot(u32)` and `SignatureSlot(u32)` indexes; `TypeData` and the per-kind payloads become an enum and tables whose layout is section 13, item 6;
- a checker AST arena for `Checker.factory`, with its own globally allocated `ArenaId`. Synthetic signature declarations live here; synthetic expressions refer to types in the same checker. Imports of these nodes validate the exact checker and generation as well as the node arena and slot. A result retaining such a node retains the `CheckerOwner`, including its type/signature storage and file dependencies;
- the link tables: `NodeLinks`, `SignatureLinks`, `TypeNodeLinks`, `EnumMemberLinks`, `AssertionLinks`, `ArrayLiteralLinks`, `SwitchStatementLinks`, `JsxElementLinks`, `ComputedNameNodeLinks`, the symbol-keyed families, and `SourceFileLinks`, each a table keyed by `NodeId` or `SymbolId`. Tables keyed by file symbols use a per-file slot-indexed vector inside the checker, `HashMap<ArenaId, Vec<Option<Links>>>`, which is what upstream's `symbolArenaLinkStore` already does; tables keyed by transient symbols index the checker's own arena directly.

Symbol tables stay unordered hash maps keyed by `JsString`; any place upstream sorts before observation sorts here with the ported comparators (ADR 0010). Iterating a table without sorting is allowed exactly where upstream does it.

The builder context borrows its enclosing checker; it does not hold an owning `Arc` back to it. Same-checker arena links and caches use private slots or ids, with builder-generation dependencies retained by the enclosing context or an explicit retention set as described in ADR 0006. External results hold the owning roots. This keeps a checker from retaining itself through a cached symbol, type list or generated node.

### 2.4 Provenance and mixing

The `u32` slots are private storage indexes, not service or external-cache handles. Fields and constructors remain private to the checker implementation; no public unchecked conversion accepts an integer slot. The boundary API is concrete:

- `RetainedType` and `RetainedSignature` carry an `Arc<CheckerOwner>`, its exact `CheckerId`, the pool generation and the private slot. `RetainedSymbol` carries its `SymbolId` plus the applicable file/bundle retention root or `Arc<CheckerOwner>` and generation. These are the results returned by language-service helpers, stored in external caches or registered by the API. Retaining a result keeps its dependencies alive without holding the operation permit.
- For a checker-owned result, an operation acquires a lease and permit for the handle's exact `CheckerOwner`; it never substitutes an available checker from the same pool. A file-symbol query may use any checker whose program retains that file or bundle. `lease.import_type`, `import_signature` and `import_symbol` validate owner identity, active generation and slot bounds in **both debug and release builds**. File symbols require membership in the lease's retained file/bundle set; checker symbols require matching `CheckerId`. A generation match alone does not permit mixing checkers.
- Import creates an opaque scoped value bound to that lease. Its operations use that lease, not an arbitrary checker supplied by the caller. Any operation accepting a retained value from outside the scope must import it; service code cannot extract a raw slot and feed it to another checker. Inside an uninterrupted validated operation, private slots may be used for bounds-checked indexing without repeating the owner lookup. Debug assertions supplement this restricted fast path; they do not replace release validation at a boundary.
- A callback or reentrant operation that releases the permit or can retire the generation ends the validated operation. Before resuming, the caller reacquires the same owner's permit and revalidates exact checker identity and active generation. Scoped values cannot be reused across that boundary. Reentrant paths therefore cannot silently import a same-numbered slot from another checker.
- `Arc<[TypeSlot]>` and `Arc<[SignatureSlot]>` may occur inside same-checker storage, but cannot escape by themselves. A retained list couples its immutable slots to the same owning root, exact checker identity and generation, with validation before use. Owner-internal caches use non-owning same-checker indexes to avoid an `Arc<CheckerOwner>` cycle.

These rules support returned types and symbols after the original lease ends, as upstream requires. If the pool drops an idle checker reference while an external result still retains that owner, subsequent access uses that retained owner under its permit, subject to generation retirement; it does not reinterpret the result in a replacement checker.

### 2.5 Wire handles

The API's `SymbolID` is a registry-assigned dense counter, as upstream's lazily assigned `GetSymbolId` is, mapped to the internal packed id and retained owner in the snapshot's symbol registry. Each symbol entry retains its own applicable file/bundle or checker root; the snapshot-wide registry can contain symbols from several projects. The packed 64-bit id is not sent on the wire: JavaScript numbers lose precision above 2^53, and a packed id with a high arena counter would exceed it. `TypeID` stays the checker's 32-bit id, registered per project. Each per-project type/signature registry is bound to the exact `CheckerId`, pool generation and retained `CheckerOwner`; it resolves the complete snapshot/project/id handle in that scope and validates before use. Registry reads of type or signature properties also acquire the owner's operation permit, even where the Go implementation accesses a stored pointer directly (`api/session.go:2363–2403`). A retired registry is never rebound to a replacement checker with reused slots (ADR 0012).

Registry lookup clones the retained handle under the generation gate and registry lock, releases both, then acquires the exact checker's operation permit and revalidates under the gate. It never waits for that permit while holding either lock. Final registry insertion and response commitment follow the retirement-serialized protocol in the ownership note, including its nonblocking queue commitment.

### 2.6 Retirement

A checker belongs to a pool generation. On a caught panic the pool retires the generation (ADR 0012): new leases go to a fresh checker set, existing leases observe retirement at their next boundary, and registries and retained handles reject further operations on the retired generation. Retirement does not make retained storage disappear. Arenas and their dependencies remain allocated until the final lease or external owning root, including a retained type, symbol, list or registry entry, drops. Retaining storage grants no permission to continue using a retired checker. Nothing in a file's bind result is touched by retirement, because the checker never mutated it: merged symbols were clones in the checker's arena.

## 3. What E3 asserts for this note

| Scenario | Assertion |
|---|---|
| Two programs share one bound file | both checkers resolve the same file symbol ids; each merges declarations into its own arena; `merged` maps differ; dropping one program leaves the other's answers unchanged |
| Two checkers merge declarations over shared files | the file's symbols are unchanged after both merges; each checker's clone has the merged flags, declarations and exports upstream's `.symbols` baselines show |
| Edit while an old snapshot answers | the old checker keeps resolving the old file's symbol ids; the new file's ids are rejected by the old checker's `import` and vice versa |
| Two checkers in one pool generation have the same numeric type/signature slot | importing one checker's retained handle into the other's lease fails in debug and release builds, despite matching pool generation and in-range slot |
| Returned type, signature, symbol and immutable type list outlive a lease | retaining the result keeps the exact owner and dependencies alive; releasing or replacing the pool slot does not rebind the result; access reacquires that owner's permit; final result/cache drops release storage without cycles |
| Checker-created AST node retained with a signature or type | a synthetic signature declaration and a synthetic expression keep their checker AST/type dependencies; imports by another checker fail; dropping the checker context alone cannot free retained result storage |
| Callback reentry changes or retires a checker | resumption reacquires the original owner's permit and revalidates exact identity and generation before using scoped values; same-numbered foreign slots and retired handles fail explicitly in release builds |
| Retired generation with live results and registries | leases, retained results and registry lookups fail at their next operation boundary; storage remains valid while retained; counters return to baseline only after all leases, caches and external roots drop |
| Arena counter boundaries | symbol/node encodings preserve identity and metadata owner kind across `2³¹−1` and `2³¹`; allocation at the full `u32` exhaustion boundary fails before wrap or reuse; zero-slot ids are rejected |

## 4. Choices deliberately left to measurement

The `Type` payload layout and whether the per-file link vectors are allocated eagerly or on first touch (section 13, items 6 and 7); whether symbol tables use `FxHash` or a keyed hash for `JsString` keys (section 13, item 8).

## 5. Evidence index

`binder/binder.go` 77, 95, 120, 132; `ast/ast.go:2895`; `ast/symbol.go` 10, 45; `checker/checker.go` 669, 673, 674 to 700, 14285, 14324, 14361, 14552, 14564, 14581, 30466, 30869; `checker/types.go` 164, 680, 1290; `compiler/program.go:586`; `compiler/checkerpool.go`; `project/checkerpool.go` 38, 296, 345; `project/project.go:291`; `ls/api.go` 18 to 43; `api/proto.go` (`SymbolHandle`, `TypeHandle`, `SignatureHandle`); `api/session.go` 45, 184, 230, 2363 to 2403; `ast/utilities.go` 18 to 40.
