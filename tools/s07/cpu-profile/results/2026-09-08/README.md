# S07 diagnostic profiling archive — 2026-09-08

This directory retains reviewable inputs and observations from the diagnostic
profiling capture. It does not supply E5/E6 acceptance evidence or change their
existing measurements. `manifest.json` records 64 validated artifacts: about
6.62 MB archived from 90.07 MB of original observations.

`capture/report.json.gz` contains the original completed capture report, including
build provenance and all 30 process observations. `capture/runs.ndjson.gz` retains
the exact original row stream. The six files under `go/` are unchanged original
Go CPU profiles. Their final analysis is `go/summary.json.gz`; all 36 referenced
pprof text exports are also retained under `go/exports/`. The raw dumps and
tags exports are gzip-wrapped; decompression preserves every byte, including
the tags exports' trailing blank lines. The six Rust sample
exports, TOCs and final summaries are under `rust/`, together with the physical
symbol map. Every row reproduces the frozen input digest and all six scalar
workload counters. No samples or elapsed-time outliers were removed.

The capture metadata retains compiler and configuration hashes, package
identities, local project/configuration paths, and host/load facts. It contains
no private environment values. The Rust TOCs remove only the
`name` and `uuid` attributes on `device` elements. The manifest retains each
original TOC hash, the redacted content hash, and the archived-file hash. Other
TOC content, target-only CPU sample XML, and original profiles remain unchanged.
Native `.trace` bundles, executables, and debug-symbol bundles stay in `target/`.

JSON, NDJSON and XML archives use gzip level 9, modification time zero, and no
embedded filename. Manifest `stored_sha256` values identify the archived bytes;
`content_sha256` identifies the decompressed content. `source_sha256` identifies
the original local artifact before any stated TOC redaction. For unchanged
artifacts the source and content hashes are equal. Original Go `.pprof` files
already use their producer's compressed protobuf representation and are copied
without recompression.

Elapsed worker timers include descheduling and are not CPU time. Their sums can
exceed pipeline wall time. Rust profiles cover the process lifetime, while Go
profiles cover the pipeline interval; analysis must preserve these distinct
scopes. Three diagnostic control repetitions do not replace the fixed acceptance
sampling and uncertainty rules.

## Offline review

The Go profiles contain symbolized locations. With the pinned Go 1.27.1 toolchain,
`go tool pprof -sample_index=cpu -top go/go-1-0.pprof` reads a profile without
rerunning the workload or locating its executable. The final summary records the
exact commands and hashes for the additional text views.

For Rust, decompress one TOC, its CPU sample export, and
`rust/physical-symbols.json.gz` to a scratch directory, then run from the
repository root:

```sh
python3 tools/s07/cpu-profile/analyze_xctrace.py analyze \
  --toc /path/to/scratch/rust-1-0.toc.xml \
  --samples /path/to/scratch/rust-1-0.time-profile.xml \
  --symbol-map /path/to/scratch/physical-symbols.json \
  --output /path/to/scratch/rust-1-0.replayed.json --top 200
```

Repeat for all six worker/repetition pairs. This uses the archived text ranges,
Mach-O UUID, raw symbol-tool output and displayed-name mapping; it requires no
executable, debug bundle, Xcode or live demangler. The manifest records the final
analyzer hash. Original summaries are archived unchanged, so their TOC hashes
identify the original private TOCs. Replay against the redacted TOCs changes
TOC provenance fields and local artifact paths; CPU samples and derived results
remain the same. Use the manifest to verify the original-to-redacted hash link.

All six archived inputs were independently replayed with `PATH=/nonexistent`
and only the offline map. Every derived CPU, phase, union-query, frame and
ranking field matched the original summaries exactly. The only differences
were the TOC hash and byte count after device-attribute removal, and local TOC,
sample and map paths. All 64 archived-file and decompressed-content hashes were
checked again. The exact exclusions and per-profile results are retained in
`manifest.json`.
