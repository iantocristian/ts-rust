# Empty-drain and whole-name-hash experiment — 2026-09-10

The [complete result](../../../../../docs/S07-bis-drain-hash-result.md) records an
inconclusive fixed screen. The candidate is **not retained**. Production, cached
Rust executables and source-bound evidence pointers were restored to the control.
No second candidate or new Go acceptance batch was run.

The CPU medians improved 1.96% / 1.31%, but both 95% ratio intervals include 1.0;
the eight-worker upper bound is 1.022502. Memory is effectively unchanged. The
raw `regressing_or_uncertain` verdict remains intact. No measured component
savings are added together or converted into new Go-relative acceptance ratios.

## Distribution and replay

The exact 116,284,096-byte archive is split into three parts of at most 48 MiB.
`distribution.json` binds every part and the assembled archive, whose SHA-256 is:

`bed12778f6fe1b9a88ff8c3b5fecef5708596e6088f2f8a5dda57922502ac489`

With Python 3.11 or newer, run:

```sh
python3 verify.py --output /tmp/s07-drain-hash-replay
```

The output directory must be new. The launcher verifies part/manifest/archive
identities and runs the unchanged archive engine. Allow about 1.1 GB for temporary
assembly and extraction. No Rust/Go build, native workload, network, live profiler
or source checkout is needed. Historical absolute paths remain provenance strings.

Exactly one extracted-root replay passed: all **2,053 members** and all **five
obligations**, with zero failures:

- Original CPU XML, frozen symbol map, full summary and phase attribution.
- One Rust and one Go worker diagnostic: derivative build identity, raw work/index
  inventories and elapsed accounting.
- Full 13,094-file graphs and binding-path observations at one and eight workers.
- All eight warmups and 56 fixed-screen samples, raw outputs and recomputed statistics.
- Candidate validation: all 539 before/after source/input files, recorded command
  exits and raw hashes, scanner/binder/E3 metrics and frozen inventories, plus the
  exact single hash-boundary test in both Miri and ASan.

`verification.json` records the identities and outcomes; `verification-report.json`
preserves each validator's command and exit. `launcher-origin.json` identifies the
reused split launcher and its two bootstrap-path changes.

## Retained evidence and limits

Both immutable source/binary bundles, original profile, worker setup failures,
all screen/graph observations, candidate validation, source/binary restoration
receipts and final decision documents are preserved. One passive validation
adapter development failure is recorded separately; no native measurement was
repeated to repair it. Existing exact validation source files are reused by
explicit paths, with all bytes rehashed during replay.

Duplicate workload byte trees and rebuildable compiler/toolchain/export outputs
are omitted by the pinned recipe. Profile/worker transport checks replay recorded
per-file identities and captured loaded-input digests; they do not reopen omitted
workload bytes. Earlier standalone unit/build logs lack machine-readable command
and exit receipts and are explicitly supplementary. Capture-time immutable file
permissions are not reasserted after extraction. Candidate evidence is not claimed
to validate restored production or replace its prior E5/E6 acceptance records.
