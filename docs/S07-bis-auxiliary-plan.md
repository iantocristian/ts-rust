# S07-bis: compact auxiliary storage

Implemented to combine with parent attachment; no independent component promotion gate. The concrete auxiliary policy projects 115.677 MB less retained storage and 96.154 MB fewer requested bytes, while adding 1,812,781 allocator-interface calls. These are static accounting results, not measured allocation/RSS savings or a CPU improvement. Keep the current four-row syntax payload policy.

## Physical boundary

Add `NodeRecord::CoreAux`; retain `NodeRecord::Aux` for full lazy auxiliary values. `StoredNode` uses an eight-byte `StoredAux` for `CoreAux` and `AstStorageData` for `Aux`; generic arena `Node<T>` uses `()` for both. This is an actual core/lazy type distinction, not an eight-byte locator enum containing a Box that silently expands to sixteen bytes.

Core `StorageOwner::auxiliary`, `StorageBuilder::{push_aux,aux_mut}` and `CoreDataRead::auxiliary` use CoreAux. Lazy Pages/Staging, `StorageTransaction::{push_aux,aux_mut,staged_aux,node_and_aux_mut}` keep Aux. The transaction's immutable core auxiliary arena uses CoreAux. Introduce a scoped result along these lines:

```rust
pub enum AuxiliaryRead<'a, N: NodeRecord> {
    Core(&'a N::CoreAux),
    Lazy(StorageRead<'a, N::Aux>),
}
// Existing owner selection, namespace and slot checks remain inside these calls.
StorageView::aux(id) -> Result<AuxiliaryRead<'a, N>, Error>
StorageView::aux_with_owner(id) -> Result<(AuxiliaryRead<'a, N>, Self), Error>
StorageTransaction::aux(&self, id) -> Result<AuxiliaryRead<'_, N>, Error>
```

Transaction lazy reads use `StorageRead::borrowed` under its existing exclusive transaction; they do not reacquire the publication lock. Add a checked builder accessor splitting mutable CoreAux and Store only where compact list writes need both. Do not widen the already active parent-attachment borrow API or remove its checks; change its auxiliary associated type mechanically after that work is handed off.

## Stored records and reads

`StoredAux { kind: u32, row: u32 }` retains the existing mixed AuxId allocation sequence. Internal typed ordinals are not new public arenas and must not consume global IDs. Put list rows, backing rows and cold full values in the selected CoreStore. Use existing checked typed storage machinery, with the actual directory/root costs charged when the implementation is concrete; do not introduce a new page-policy family for this change.

- List row: i32 pos/end, u32 local backing slot, u32 start/len, u32 modifier flags = 24 bytes. These preserve TextRange's signed int32 representation and wrapping rules.
- Backing row: u32 global edge start/len = 8 bytes. A start that does not fit u32 selects a cold full-width backing, rather than truncating or changing an existing successful API call into an error.
- Cold values retain AstStorageData for File, SourceFiles, SourceMetadata, Text and exceptional full backings. Lazy values retain today's whole enum and Arc-backed fallback nodes.
- Local nonnil backing slots fit u32, including u32::MAX. Foreign backing IDs need a full-width escape keyed by the list row; a tag bit can mark the exceptional case so ordinary nil/local reads avoid a map probe. Never steal a valid slot bit. Nil and missing descriptors both have no backing and remain distinguished by start/len. Preserve allocated-empty backing identity.

At the AST layer, introduce an `AuxRead` owning either the checked borrowed core record plus physical CoreStore/owner context, or the full `StorageRead<AstStorageData>` guard. No public Deref to an invented reconstructed AstStorageData enum. Cold-value access borrows through this wrapper. `NodeListRead` provides the existing `loc()`, `nodes()`, `modifier_flags()`, `is_missing()` methods and explicit `to_owned() -> NodeList`; it loses physical Deref. `NodeSliceRead` and `TextSliceRead` keep semantic APIs and hold the wrapper when they need a lazy/full backing. Local core list reads only decode their fixed header fields; no Node reconstruction, Arc increment or allocation.

FileInfo remains a cheap copied value. SourceFile/metadata accessors that return borrowed full records must retain the wrapper until their returned borrow ends. Core validation iterates the mixed-ID locator order and dispatches to the same list/backing/metadata obligations, preserving error ordering and CP5's private proof boundary. No second full validation pass is added.

## Writes and compatibility

The existing `set_list_nodes`, `set_list_location`, `set_list_modifier_flags` and `mark_list_missing` become direct row writes. `set_list_nodes` must resolve/validate its supplied backing **before** resolving the target list, as today. Location/flags/missing writes preserve the construction-edge proof; successful unrestricted mutation dirties it, and failed owner/slot/kind resolution does not.

For unrestricted editing, **promote only the selected list header into the pooled cold `AstStorageData::List` representation before returning its existing `&mut NodeList`**. Update the same mixed AuxId's locator; neither public identity nor its backing changes. AstBuilder, AstTransaction and RuntimeFactory therefore keep their current mutable-list return type, and custom factories need no new guard adapter. The old compact row remains unreachable until owner drop: charge its 24 bytes per promoted header, plus the cold replacement. Lazy lists are already full values and require no promotion. Ordinary parser writes use the existing narrow setters and stay compact; modifier-list cloning can directly copy modifier flags after alloc_list rather than invoke unrestricted mutation.

This is the compatibility answer to the mutable-guard problem. A staged compact guard could require allocation to encode a new foreign backing during Drop, including while unwinding; avoid introducing that failure point. Cold promotion returns a real mutable header, so edits retain their existing immediate and unwind visibility. Do not add backing validation during promotion: unrestricted editing permits invalid intermediate edges and completion must catch them. Resolve owner/slot/kind before promotion or proof invalidation. Do not hold a decoded read while requesting a mutable Store borrow.

## Required countertests

1. Core list and backing lookups distinguish foreign namespace, absent slot, wrong auxiliary kind and invalid range in their present order. Include an invalid supplied backing and invalid target list together to preserve `set_list_nodes` precedence.
2. Distinct nil, missing, allocated-empty and nil-element cases; clone a list header and verify it retains backing identity while later header edits do not change the clone. Reslice and trailing-comma behavior remain unchanged.
3. A retained imported/mapped backing is decoded using its physical owner's edge arena; dropping the original caller preserves it through the explicit owner. A caller's broader imports cannot validate a cache owner's dangling link.
4. Foreign and full-range local IDs round-trip; u32::MAX is not mistaken for an escape. Exercise the wide backing branch with synthetic descriptor metadata without allocating a multi-gigabyte edge arena.
5. A lazy transaction can read compact core lists, create/mutate full staged lists and nodes, publish them, and later read them while holding the correct guard. Failed or panicking initialization burns provisional IDs and exposes no partial auxiliary graph; retry does not resurrect them.
6. Successful unrestricted core edits dirty CP5's proof; failed list_mut does not. Narrow location/flag edits preserve it. Invalid backing introduced through an edit still fails completion. Unwind preserves the same edited header visibility as before. A promoted header keeps its AuxId/backing and is promoted only once; subsequent narrow edits work on the cold representation without allocating another row.
7. Compile-fail cases for escaping a borrowed list/text result beyond its read guard or owner; runtime checks for core reads not acquiring the lazy lock. Keep the existing declaration/list/ownership graph probes and full workload graphs in the combined candidate's validation.

## Integration boundaries

After the parent-attachment handoff, implement the arena CoreAux/Aux split and focused generic transaction tests first, then compact auxiliary storage and AST semantic read/write wrappers. Migrate handwritten NodeList consumers and generated construction/visitor templates to explicit semantic clones/getters, and regenerate through the pinned producer. Preserve the new direct parent-attachment path by supplying its immutable CoreDataRead with the compact auxiliary decoder. The final combined checks include ownership/doctests, parser/binder/encoder/compiler tests, generated-source checks, full workload graphs, Clippy/MSRV, then the already established frozen pipeline screen. No additional profiling or model matrix is a prerequisite.

## Implemented policy and validation record

The core/lazy associated auxiliary types now differ. Core mixed-ID slots are eight-byte locators; list and backing rows use the existing four-row RowPages policy, with inline store roots and no new public arena namespace. Lazy transactions retain full auxiliary values. Cold list promotion preserves the existing mutable-reference API, commits the same locator identity, and removes a now-unused foreign-backing escape. Generic cold metadata mutators reject compact list/backing targets without promotion or proof invalidation. Parent attachment retains its disjoint core/data borrow and the same checked fallback conditions.

The concrete projection reuses the verified earlier per-owner auxiliary census and includes the actual four-row directories. It replaces 316.984 MB retained / 320.247 MB requested with 201.307 MB retained / 224.094 MB requested. Typed pointer directories alone retain 18.921 MB and request 37.445 MB through growth. New embedded AuxStore roots cost 1.781 MB. The total modeled call count rises from 130,725 to 1,943,506: smaller requests do not establish cheaper construction. This replaces the earlier 95–107 MB estimate for the chosen implementation; it is not another policy matrix or a current physical allocation census.

The projection assumes no cold promotions, foreign list escapes or wide backing offsets. Each promoted list leaves an abandoned 24-byte row and adds a full cold record, including any Vec growth. Ordinary parser/binder sources use narrow list operations; unrestricted RuntimeFactory defaults and transforming modifier-list clones keep their existing custom-factory dispatch semantics. Allocator rounding, actual RSS response and elapsed-time effects remain unmeasured. The complete parent/auxiliary combination must be screened together; no standalone component threshold or added savings from separate captures apply.

Reproducible arithmetic and input/source hashes are retained in `target/s07-bis/cp6-accounting/auxiliary_account.py` and `auxiliary-accounting.json`; the explanatory record is `target/s07-bis/compact-auxiliary-accounting.md`. The previous current-payload accounting remains separately available and unchanged in scope.

Validation so far: workspace all-target/all-feature compilation and Clippy pass. Library tests pass for AST (126), parser (26), binder (28), compiler (12), and encoder (no library tests); AST ownership/doctests pass (13). The generic arena migration passes 26 library tests and eight doctests using different core and lazy auxiliary types. New counterexamples cover exact physical widths, full-range local IDs, foreign escape cleanup, wide offsets, repeated cold promotion, unwind-visible edits, backing-before-target errors, rejected metadata mutation, and retained lazy lists borrowing compact core backings. Independent arena, AST and arithmetic reviews report no findings. Pinned generation confirms all 21 generated files and client bytes unchanged; the first attempt's sandbox denial accessing the existing Go cache and the successful retry are both retained. Integration execution, MSRV/instrumentation, full graphs and the frozen combined screen are recorded as they complete; these test passes alone do not promote the candidate.

The subsequent integration run passes all 29 AST integration tests and five encoder layout tests. Workspace all-target/all-feature checking passes on the declared Rust 1.96 minimum. Pinned nightly Miri with strict provenance passes all 26 arena tests, nine selected auxiliary tests (including rollback and retained mapped owners), and seven parent-attachment tests. AddressSanitizer passes all 26 arena and 126 AST library tests; macOS leak detection remains disabled as in the existing producer. These are scoped validation runs, not a fresh complete E3/S06/S07 evidence claim. Commands, environment choices and all outputs are retained under `target/s07-bis/compact-aux-*`. Full graph comparison and the combined performance capture follow from the frozen source.
