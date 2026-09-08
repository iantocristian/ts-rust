# Go memory adapter

Copy `main.go` into `internal/s07memoryprofile` and `ast_bridge.go` into
`internal/ast/s07_memory_bridge.go` in the exact pinned source export; the parent
build command performs these steps. Run as `INPUTS.json WORKERS OUTPUT_PREFIX`.
The driver emits five named JSON checkpoints and waits for one empty stdin line
at each. It writes a report and all four sample types in heap/alloc profiles.

`MemProfileRate` is fixed once at 64 KiB. Native retained state precedes profiling
and census work. A separate retained snapshot follows two requested GC cycles;
the census and retirement snapshots come later. Baseline profiles can lag and
retain delayed preload/digest allocations in their signed subtraction. The
analyzer preserves that difference and classifies actual allocation ancestors.

The observer counts embedded Node bytes within each concrete object, deduplicates
identities within each file and unions visible slice-capacity ranges. It caches
reflection field paths. Shared globals can recur across files; private arena
backing and map internals remain outside the logical census. Source strings and
other visible strings are separate and must not be added as independent heap
allocations. Census work can retain allocator/runtime scratch capacity after its
logical buffers are collected, so later retirement RSS is inspection-perturbed.

For bridge counterexamples, copy `ast_bridge_test.go` beside the staged bridge and
run `go test ./internal/ast -run '^TestMemoryCensus'`. Analyzer tests run with
`python3 -m unittest discover -s tools/s07/memory-profile/go -p 'test_*.py'`.
`analyze.py --capture CAPTURE_DIRECTORY` exports native pprof raw/flat/inclusive
views and a hash-bound summary of the completed Go matrix.
