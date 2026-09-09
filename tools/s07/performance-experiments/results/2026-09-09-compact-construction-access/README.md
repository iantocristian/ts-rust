# Combined compact construction and access: experimental

The [result record](../../../../../docs/S07-bis-compact-storage.md) reports
5.178 / 1.504 s against same-screen CP1 at 4.714 / 1.397 s. Allocation is
2.434 GB and peak RSS is 2.506 GB. CPU median ratios are 1.098 / 1.076, with upper
95% bounds 1.123 / 1.113, so the combination remains experimental. CP1 remains
the control. All 13,094 graphs match in both worker modes, with zero binding
fallbacks. All eight warmups and 56 samples are retained and receipt replay passes.

The archive contains 456 files: the complete frozen candidate source/binaries,
exact changed generator sources and generated manifest, complete graph streams,
raw observations, validation logs (including failed development checks), and
native CPU exports/audit from this exact normal executable. The one native sample
is diagnostic and visibly perturbed; it provides no paired performance claim.

- Archive bytes: 19,654,108.
- SHA-256: `143580e99a15f4c69dcff860c04186843e900d963dac41f99c4bf292b187e63c`.
- Every member was read back and verified against [archive.json](archive.json).
- Unchanged CP1 source/binaries and measurement helper closure are pinned in
  the [first archive](../2026-09-09-compact-typed-first/README.md).
- Workload sources and the native Instruments directory remain local; original
  paths and immutable permissions are required for receipt replay.

```sh
python3 tools/s07/performance-experiments/runner.py verify target/s07-bis/construction-access-screen
```

Earlier CP4 phase/timing captures are historical diagnostics, not the measured
control for this experiment. The result does not establish individual component
savings, broader ownership instrumentation or final S07 Go acceptance.
