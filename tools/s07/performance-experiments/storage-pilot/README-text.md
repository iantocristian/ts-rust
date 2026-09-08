# Four-byte identifier text prototype

`text.rs` implements owner-local byte storage, independently of production AST
storage. A `TextWord` is exactly four bytes. Even words encode a raw byte length;
the owner supplies the node's current end and retained source, checks subtraction
and source bounds, and borrows that suffix. Insertion first compares the actual
selected bytes, without UTF-8 decoding or normalization.

Odd words index owner-local `(usize offset, usize length)` entries into an owned
byte vector. Exceptional text is copied once into that vector, so scanner
temporaries and foreign source owners may retire. There is one source `Arc`
per owner and no per-identifier Arc for raw names. `bytes()` borrows; `retain()`
explicitly copies into independently owned bytes when an API needs to escape.
Imports/clones read through their source owner and re-encode in the destination;
they never copy a local word across owners.

The enclosing node store must resolve public owner/slot identities before using
this crate-private API. These four-byte words do not themselves authenticate an
owner. The store supplies the current node slot, word and end together. All raw
source and pool accesses remain bounds-checked. An even word can use the entire
31-bit length domain. The odd all-ones value selects an extended table keyed by
the full `u32` node slot, whose value is the full `usize` pool index. No slot bit
is stolen and oversized lengths use pooled bytes. Countertests lower the inline
pool threshold to exercise the real extended path without enormous allocations.

`insert(slot, bytes, Some(end))` creates a verified raw suffix or a pooled value.
`insert(..., None)` makes text observable before a factory's final range exists.
`change_end(slot, word, old_end, new_end)` computes a replacement word **before**
the caller changes its header: it preserves the selected bytes, even if the new
suffix differs or the new range is synthetic. It can also finalize factory text
as a verified raw suffix. The old pool entry then remains allocated and counted.
Calling the factory path for every ordinary parsed identifier would therefore
retain every staged name; integration must measure that path and cannot treat
its later raw compaction as recovered allocation. This prototype does not
silently assume hooks can observe a node only after its range is finalized.

`stats()` exposes source length, pool byte length/capacity, entry length/capacity
in bytes and extended-table length/capacity in entries. These are structural
observations, not allocator or RSS measurements. Charge the owner object,
source Arc header, hash bucket/control bytes, superseded allocation requests,
and independently retained escapes through the eventual allocator harness.
Source backing must be charged once when shared. No pool-size, CPU or memory
acceptance claim follows from these tests.

Tests cover raw source borrows, exact tag boundaries, full-slot extended entries,
decoded escapes, JSX names, WTF-8/malformed/partial-codepoint bytes, arbitrary
factory text, finalization, synthetic/range edits, foreign clones, explicit
retention after owner retirement, and pool growth. They test this Rust contract;
they are not new upstream parity or sprint evidence.
