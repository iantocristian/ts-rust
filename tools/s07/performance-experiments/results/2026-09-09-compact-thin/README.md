# Thin physical-owner candidate

The [result record](../../../../../docs/S07-bis-compact-storage.md) reports
6.411 / 1.802 s wall time against same-screen CP1 at 4.909 / 1.380 s. Allocation
and RSS remain about 3.185 / 2.837 GB. **Not promoted:** both CPU median ratios
are 1.306, and the upper 95% bounds exceed 1.02. All 13,094 graphs pass at both
worker counts, with zero binding fallbacks. All eight warmups and 56 measured
children are retained and receipt verification passes.

`review.tar.xz` preserves 426 files: frozen changed-candidate sources and
binaries, complete new graph streams, raw measurements and validation logs.
Every member was read back and verified against [archive.json](archive.json).

- Archive bytes: 19,000,368.
- SHA-256: `e024718fa11eab7747cad4abe55656f5735c510a401d6c92d1d55253df1893e8`.
- The unchanged CP1 bundle and measurement helper closure are retained in the
  [first archive](../2026-09-09-compact-typed-first/README.md); the receipt records
  its SHA-256.
- Frozen workload source files remain local. Original local paths and immutable
  permissions are needed for receipt replay; extraction does not rewrite them.
- No CPU profile was captured for this third candidate.

```sh
python3 tools/s07/performance-experiments/runner.py verify target/s07-bis/compact-thin-screen
```

The successful verification output is included. This diagnostic evidence does
not replace the broader ownership producers or fresh Go-relative acceptance.
