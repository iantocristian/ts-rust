# Inline parser-list buffers: experimental combined result

The [result record](../../../../../docs/S07-bis-compact-storage.md) reports
5.387 / 1.205 s against same-screen CP1 at 4.748 / 1.137 s. Allocation is
2.244 GB and peak RSS 2.322 GB. CPU upper 95% ratios are 1.162 / 1.146;
the combination remains experimental and CP1 remains the control. All final
Go-relative CPU and memory gates remain open.

All 13,094 graphs match at both worker counts, with every file binding in place.
All eight warmups and 56 measured samples are retained, and receipt replay and
independent verification of raw observations and statistics pass. Successful and
failed development checks are preserved, including the intentionally stopped
broad Miri run and the subsequent passing scoped instrumentation.

- Archive: 19,260,764 bytes, 458 files.
- SHA-256: `6d16fde893fd1f38dae543c05ee6ad0fb2bb68a86d0b499befd50522bfd667cc`.
- Every member was read back and verified against [archive.json](archive.json).
- Exact candidate source/binaries and unchanged generator inputs are included.
- Unchanged CP1/helpers are pinned in the [first archive](../2026-09-09-compact-typed-first/README.md).
- Reused physical census inputs are pinned in the [CP6 archive](../2026-09-09-cp6-accounting/README.md).
- Workload sources remain local; receipt replay requires original paths and
  immutable permissions. Scoped instrumentation is not a complete producer claim.
- The subsequent phase/live-requested-memory diagnostic is separate from this
  complete performance screen.

```sh
python3 tools/s07/performance-experiments/runner.py verify target/s07-bis/parser-list-screen
```

The 86.59 MB request difference from the preceding auxiliary capture is a
cross-capture observation, not an isolated list-buffer measurement. Separate
wall-time medians are not added or subtracted to predict another combination.
