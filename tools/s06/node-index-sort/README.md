# Pinned Go node-index sorting observations

`NodeIndexTable.GetIndex` assigns semantic node IDs from inside the sorting
comparator. The permutation and comparator-call order therefore both matter.
These fixtures observe `slices.SortFunc` from the Go version selected by
`data/s04/toolchains.toml`, including comparators that assign IDs on first use.

Run `python3 tools/s06/node-index-sort/generate.py --check` to verify the frozen
JSON and Rust fixtures against the installed pinned Go. Omit `--check` to
refresh both outputs for an explicitly reviewed pin or fixture change. The
script forces `GOTOOLCHAIN=local` and records the two standard-library source
hashes. The JSON keeps every `(left input index, right input index, result)`
comparison, final permutation, and exceptional outcome.

The Go overlay adds only access wrappers and a test file; it changes no standard
sort implementation. Four additional fixtures call the original heap fallback,
partial insertion, pattern breaking, and equal partition helpers directly. They
exercise branches independently of the main-sort input selection. The partial
insertion fixture includes an out-of-order element below the selected subrange,
which preserves Go's loop boundary at index 1. The panic fixture observes the
partially modified slice when the comparator fails on its second invocation.

The Rust port retains the Go copyright and BSD license in
`licenses/GO-BSD-3-Clause.txt`.
