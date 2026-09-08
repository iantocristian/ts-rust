# S07 comparative memory results — 2026-09-08

See [the interpretation](../../../../../docs/S07-memory-profile.md).

The archive preserves all 18 matched full-workload diagnostic runs:
12 native Rust/Go captures and six Rust source-scope captures, with three
repetitions at one/eight workers. It does not replace or certify E5/E6.

- `raw-captures.tar.xz`: 470 files, 138,266,792 uncompressed bytes; 43,196,436
  compressed bytes. `native/` and `sites/` contain capture reports, stdout/stderr,
  every checkpoint VM summary, censuses, Rust scope rows and all Go heap/alloc
  profiles. `native/go-memory-analysis/` contains 90 raw and native pprof exports.
- `provenance/` inside the archive contains both build manifests, final build logs,
  the diagnostic sites Cargo lock and the combined derived summary. Every original
  registry dependency retains the root lock version/checksum; the sites build
  additionally pins MIT `alloc_tracker` 0.5.25.
- `summary.json.gz` and `go-memory-comparison.json.gz` provide convenient copies of
  the combined summary and cross-language comparison. The comparison uses medians
  from the Go summary's exclusive stack-domain rows and Rust's one-worker native
  live-growth counters; its binding fraction divides bind difference by the sum
  of parse, publish and bind differences.
- `census.patch.gz` and `sites.patch.gz` preserve the exact additive observers and
  source-region changes applied only to disposable staged source trees.
- `manifest.json` identifies every archive member by uncompressed size/SHA-256,
  plus archive/patch hashes. Compression/extraction was verified against the
  complete member inventory. `replay-verification.json` records an independent
  extraction and exact replay of all 18 run summaries and all 90 native Go exports,
  including resolved Go/pprof executable hashes.

The production source fingerprint is
`17b7559a34741eb8fcf73932c4af928d264981dc2607c154143bf69567152fa3`.
The upstream gitlink is `1f70213d4922b434345f639b441681e470c7cfc1`.
Compiler toolchains, actual binaries, effective configuration and staged input
hashes are in the build manifests. Executables and the workload input cache stay
local and can be rebuilt with [the diagnostic tools](../../README.md).

Extract without changing acceptance inputs:

```sh
mkdir -p target/s07-memory-replay
tar -xJf tools/s07/memory-profile/results/2026-09-08/raw-captures.tar.xz -C target/s07-memory-replay
python3 tools/s07/memory-profile/analyze.py target/s07-memory-replay/native target/s07-memory-replay/sites --output target/s07-memory-replay/summary.json
GOCACHE=/private/tmp/ts-rust-s07-go-cache python3 tools/s07/memory-profile/go/analyze.py --capture target/s07-memory-replay/native
```

Choose a writable `GOCACHE` for pprof. On this pinned installation,
`go tool -n pprof` resolves the tool from the Go build cache; an unrelated empty
or unwritable cache can report the tool as unavailable. The capture and verified
re-export use the cache shown above.

The analyzers use extracted paths for files while preserving original child
paths as provenance. Raw Go profiles can also be inspected with pinned
`go tool pprof -sample_index=inuse_space` or `-sample_index=alloc_space`.
The exact live-byte counters, sampled byte estimates, logical storage censuses
and RSS must not be substituted for each other. Later retirement snapshots
include census/profiler cache effects, and Go's deliberate retained two-GC
snapshot is separate from its native retained endpoint.
