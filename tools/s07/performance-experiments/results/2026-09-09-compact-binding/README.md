# Compact binding candidate: not promoted

The [result record](../../../../../docs/S07-bis-compact-storage.md) reports
6.202 / 1.399 s against same-screen CP1 at 4.546 / 1.058 s, with 2.574 GB allocated
and 2.505 GB peak RSS. Memory improves, but CPU median ratios 1.364 / 1.322 and
upper 95% ratios 1.380 / 1.344 fail non-regression. CP1 remains the control.
All 13,094 graphs pass in both modes; all eight warmups and 56 samples are
retained, and receipt verification passes.

The archive contains 446 files: the frozen changed source/binaries, complete
graph streams, raw measurements, validation logs and CPU exports from this exact
CP4 normal executable. Earlier failed development checks are retained too.

- Archive bytes: 19,530,016.
- SHA-256: `4a88a576f67ed6e5828fe554f32915644b6be90f08d6b2ef3d928d75a9865c7d`.
- Every member was read back and verified against [archive.json](archive.json).
- Unchanged CP1 source/binaries and helper closure are retained in the
  [first archive](../2026-09-09-compact-typed-first/README.md).
- Workload source files and native Instruments directories remain local;
  original paths and immutable permissions are needed for receipt replay.

```sh
python3 tools/s07/performance-experiments/runner.py verify target/s07-bis/cp4-binding-screen
```

This is diagnostic evidence, not final S07 acceptance or broader ownership proof.
