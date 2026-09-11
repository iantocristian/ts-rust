# P1 storage-families capture

`report.json` is the complete report of one `scripts/s08_families.py --freeze`
run on the recording host (darwin/arm64, Go 1.27.1, Rust 1.97.1 release, locked
dependencies, mimalloc with requested-allocation counters): per-family structural
bytes and counts for the Go checker and the Rust checker over the two frozen
traces in `data/s08/storage-families.json`, the creation counters, the
allocation traffic of `NewChecker`'s prefix and of the trace on both sides, the
Rust retained endpoints and every source and native input hash.

`capture.tar.xz` retains the raw run: the exact trace requests, the Go overlay
and command, Go stdout/stderr and observations, and the Rust observations.
`capture.tar.xz.manifest.json` records every member's length and SHA256.

The two runtimes agreed on all 121 roots, the 57 named `NewChecker` types and
the creation counters of both traces. The census numbers are a measurement of
the first production storage candidate on a synthetic trace; they are not the
subset census, not E5 and not a footprint gate result. `docs/S08-P1.md` reads them.
