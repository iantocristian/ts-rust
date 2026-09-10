# Compact auxiliary records and parent attachment: experimental

The [result record](../../../../../docs/S07-bis-compact-storage.md) reports
5.263 / 1.158 s against same-screen CP1 at 4.704 / 1.083 s. Allocation is
2.331 GB and peak RSS 2.322 GB. CPU upper 95% ratios are 1.175 / 1.102;
the combination remains experimental and CP1 remains the control. All final
Go-relative CPU and memory gates remain open.

All 13,094 graphs match at both worker counts, with every file binding in place.
All eight warmups and 56 measured samples are retained, and receipt replay passes.
The archive also keeps successful and failed development checks, scoped Miri and
AddressSanitizer outputs, and the independently reviewed auxiliary accounting.

- Archive: 19,257,184 bytes, 467 files.
- SHA-256: `36b2ec1244ba247ffa3b645440a45e2f7cce7ee408440409dbbeed7da19683ff`.
- Every member was read back and verified against [archive.json](archive.json).
- Exact candidate source/binaries and generator inputs are included.
- Unchanged CP1/helpers are pinned in the [first archive](../2026-09-09-compact-typed-first/README.md).
- The reused physical census is pinned in the [CP6 accounting archive](../2026-09-09-cp6-accounting/README.md).
- Workload sources remain local; receipt replay requires original paths and
  immutable permissions. Scoped instrumentation is not a complete producer claim.

```sh
python3 tools/s07/performance-experiments/runner.py verify target/s07-bis/auxiliary-parent-screen
```

The static model, earlier candidate captures and this combined result have
different scopes. Their savings are not added to predict another combination.
