# S07-bis: remove small parser-list construction buffers

This is a bounded next candidate after recording the parent/auxiliary combined
screen. It changes list construction, not traversal or payload-page policy. The
complete compact implementation remains the candidate against retained CP1;
this component has no separate promotion gate. Final CPU and memory gates stay
unchanged.

## Evidence and limit

`parse_list_index` and `parse_delimited_list` currently grow a `Vec<NodeId>`.
`new_node_list` maps it to `Option<NodeId>`, validates the IDs and copies their
compact words into owner edge pages, then releases the temporary allocation.
Pinned Rust's in-place iterator specialization already reuses the Vec allocation
for that map; do not count a second buffer or conversion copy as removable work.

The earlier verified census contains 3,114,897 nonempty backings, of which
2,947,490 have one to four elements. A fresh push-grown Vec requests at least
32 bytes before spilling beyond four IDs. Eliminating that initial request from
every represented nonempty backing would remove 99.677 MB of requests (94.320 MB
from the one-to-four population). This is a conditional opportunity bound, not
a predicted result: the census has no caller attribution, temporary capacities,
or aborted parses, and these two parser functions do not construct every backing.
It also says nothing about peak RSS or CPU savings. The request budget is close
enough that one implementation and combined screen are justified; no preliminary
microbenchmark, expanded trace or buffer-policy matrix is needed.

## Implementation

1. Add a private parser list accumulator with four inline optional-ID slots and
   explicit length. It stores only nonnil parser IDs. On the fifth push, spill
   once to a Vec with capacity eight and preserve subsequent doubling. Support
   exactly the existing operations: length/index for parse callbacks, push,
   drain/append of reparsed nodes, and consuming finalization. Avoid introducing
   an external small-vector dependency or unsafe uninitialized storage.
   A known-length append reserves once for the complete suffix, as Vec::append
   does today; spilling a short prefix and pushing the suffix individually must
   not introduce avoidable intermediate reallocations.
2. Make the private `ParserFactory` opt in. AstBuilder enables inline buffers;
   lazy and custom factories retain the existing Vec construction and
   `RuntimeFactory::alloc_nodes` dispatch. `BorrowedFactory` forwards the choice
   and finishing method to its wrapped parser factory. Keep the public
   RuntimeFactory trait unchanged.
3. Add an AstBuilder borrowed optional-ID slice constructor that shares the
   current node-slice validation and packing implementation. Validate every ID
   and length before appending any edge or auxiliary record. Preserve physical
   owner checks, nil elements for public callers, and allocated-empty identity.
   The existing consuming Vec constructor delegates without changing its
   successful or failing semantics. The parser continues to use
   `NodeSlice::empty()` for an empty parsed list.
4. Use the accumulator in the two named parse functions and their source-file
   completion caller. Finish short buffers directly into edge pages only after
   all recursive parsing has returned. Do not stream edges while parsing nested
   nodes: nested lists would interleave the backing range. Preserve reparse
   insertion order, the callback's current index, source EOF/JSDoc creation
   before the final reparse append, and the delimited-list None-abort path.
   Leave unrelated fixed constructor vectors and scanner buffers alone.

## Validation and decision

Countertest inline-to-spill boundaries and recursive/reparse order, early abort,
empty versus allocated-empty lists, invalid owner/slot and failure before edge
insertion, borrowed-factory dispatch, and unchanged custom/lazy Vec operations.
Run affected parser/AST tests, encoder integration, ownership/doctests, Clippy,
MSRV and generated-source checks as required by the changed surfaces. Use the
existing full 13,094-file graph comparison in both worker modes.

Freeze the whole combined candidate, then run the established fixed paired
normal/allocation screen against CP1 and verify its receipt. Record all samples,
failures, source and binaries, absolute gate distances and any regression. Do not
subtract medians from different capture batches to attribute this component,
add its estimate to earlier measured savings, or call an allocation improvement
final acceptance while CPU gates remain open. Keep or remove it using the
combined result and maintenance cost; do not extend the direction into another
small-buffer matrix if its remaining opportunity is too small.

Independent plan review found no material issue with the private factory opt-in,
borrowed constructor, ordering obligations or conditional accounting. Its concrete
append-reservation note is incorporated above. Implementation and measurement
remain pending until the parent/auxiliary combined result is recorded.
