# First local binder implementation milestone

The [milestone report](../../../../../docs/S07-bis-local-bind-milestone.md)
records the first integrated local-scope binder slice. The binder-wide migration,
complete ownership-producer checks and combined CPU/memory screen remain open.
CP1 remains the accepted control; no gate or tracker evidence changes here.

Both worker modes pass the existing canonical graph protocol for all **13,094
files**. Raw graph hashes match all files at one worker and 13,029 at eight;
the remaining **65** differences are qualified-name runtime-counter bytes.
The original exact-JSON checker failure is retained. Replaying the same streams
through the existing validated comparator passes; neither graph capture was
rerun. Separate path probes report **13,094 local-scope files, zero checked-scope
files and zero publication fallbacks** in each mode.

- Archive: **14,234,212 bytes**, **475 files**.
- SHA-256: `800614c212c725e595a3c55f3d59ee8222197aac57b2ae8c053fd1f088974ca0`.
- Every member was read back and hash/size-verified against [archive.json](archive.json).
- Selected normal executable SHA-256:
  `30a6baa1d114d9629fd5bf1e00fa80f36b21b95bdcfc1a4add090aecc6bf879a`.
- Includes the selected Cargo artifact record and executable, **337 graph-time
  source/configuration files**, both Rust and Go streams, path probes, protocol
  replay results, disassembly, generation logs and successful/failed checks.
- The packaged independent verification reconstructs the existing comparator's
  result offline for every file. It verifies the selected artifact against
  Cargo output, every snapshot hash, both path probes, the four recorded
  comparator hashes and both Go streams against their members of the earlier
  [shared-text archive](../2026-09-09-compact-text-processing/README.md).
- Exact comparator/support scripts, schema, frozen workload/options/recipes and
  input list are included. Workload contents and toolchains remain external.
  Large Cargo build directories are excluded.

The source boundaries are explicit. One `cfg(test)`-only contextual-keyword
fixture changed between the normal build and graph capture. The other **336**
source/configuration hashes match the compile-time inventory, and the normal
artifact excludes that test module. The snapshot matches the graph-time
inventory exactly; the earlier test-only file contents are not separately
retained. A later formatting-only workspace diff is included as
`post-graph-formatting.patch`.

The earlier executable is retained only as a **compile-only attempt**. It has no
complete earlier source snapshot and was not selected for these graph captures.
The failed generator/default-Go-cache stage, stale-metadata Clippy output,
earlier fixture failures, initial raw-JSON comparison failure and subsequent
successful checks remain distinct. Passing these checks is not a fresh complete
Miri/ASan producer result or a CPU/RSS performance claim.

For review, extract paths relative to the repository root and inspect
`target/s07-bis/local-bind-validation/package-verification.json` alongside
`graph-result-validated.json`, the source inventories and raw streams.
`target/s07-bis/local-bind-package.py` records the packaging and offline
verification procedure. Packaging refuses to overwrite retained evidence.
Its `verify()` function performs no build or workload execution; it also checks
the recorded snapshot-to-workspace formatting boundary, so it expects the
corresponding source state. The historical `graphs.py` and `replay-graphs.py`
are retained execution records, not unconditional offline-only entry points.

There were two graph invocations (one/eight workers) and two separate local-path
probe invocations. No performance screen ran, and no individual saving or
complete-candidate performance conclusion is inferred from this milestone.
