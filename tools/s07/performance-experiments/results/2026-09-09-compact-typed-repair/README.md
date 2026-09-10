# Compact storage after the first read-path repair

The [result record](../../../../../docs/S07-bis-compact-storage.md) reports
6.099 / 1.315 s wall time against same-screen CP1 at 4.363 / 0.968 s. Allocation
and RSS remain about 3.185 / 2.837 GB. **Not promoted:** both CPU modes still
regress substantially. All 13,094 graphs pass at both worker counts, with zero
binding fallbacks; all eight warmups and 56 measured children are retained.

`review.tar.xz` preserves 436 files: the changed candidate's frozen sources and
binaries, complete new graph streams, raw measurements, validation logs, and
the second normal-binary CPU sample's exported XML/symbol map. Each member was
read back and verified against [archive.json](archive.json).

- Archive bytes: 19,491,336.
- SHA-256: `f96583e57ac2126246c8e745729d599215f849ae9d07061e90e49c53e1b5429a`.
- The unchanged CP1 bundle and measurement helper closure are retained in the
  [first archive](../2026-09-09-compact-typed-first/README.md), rather than copied
  again. The receipt records that archive's SHA-256.
- Frozen workload source files and native Instruments trace directories remain
  local. Original local paths and immutable permissions are needed for receipt
  replay; extracting the archive does not rewrite them.

```sh
python3 tools/s07/performance-experiments/runner.py verify target/s07-bis/compact-typed-repair-screen
```

The successful verification output is included. This is diagnostic evidence,
not a replacement for the broader ownership producers or fresh Go-relative
acceptance measurements.
