# Rejected vector row policy

The [result record](../../../../../docs/S07-bis-compact-storage.md) reports
5.828 / 1.307 s against same-screen CP1 at 4.535 / 1.038 s, with 3.536 GB allocated
and 2.869 GB peak RSS. **Rejected:** CPU still regresses substantially and vector
growth adds requests. All 13,094 graphs pass in both modes; all eight warmups and
56 samples are retained, and receipt verification passes.

The archive contains 425 files, including the frozen changed source/binaries,
complete graph streams, raw measurements and validation logs. It also contains
one supplementary native CPU capture's exports from the preceding **thin paged**
candidate; those exports must not be attributed to the vector executable.

- Archive bytes: 19,433,660.
- SHA-256: `4eb5eb6ab3e3952846137dbc822645f622dd370a25b38b8133677d93deab9c02`.
- Every member was read back and verified against [archive.json](archive.json).
- Unchanged CP1 source/binaries and helper closure are retained in the
  [first archive](../2026-09-09-compact-typed-first/README.md).
- Workload source files and native Instruments directories remain local;
  original paths and immutable permissions are needed for receipt replay.

```sh
python3 tools/s07/performance-experiments/runner.py verify target/s07-bis/compact-vec-screen
```

This is diagnostic evidence, not final S07 acceptance or broader ownership proof.
