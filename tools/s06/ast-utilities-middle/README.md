# AST middle utility observations

`python3 tools/s06/ast-utilities-middle/generate.py --check` executes an access-only
test overlay in the clean pinned Go AST package and compares its outputs with
the frozen fixtures. Omit `--check` only after reviewing a source or fixture change.

The binary table stores 27 predicate observations in the low bits of one
little-endian `u32` for each signed `i16` kind, in increasing order. The Go and
Rust tests list the predicates in the same explicit order. The separate TSV
records actual factory-built graph, modifier-callback order, literal flags,
JSDoc list identity, JSX filtering and position-search observations. These
supplemental utility checks contribute no primary E1 rows.
