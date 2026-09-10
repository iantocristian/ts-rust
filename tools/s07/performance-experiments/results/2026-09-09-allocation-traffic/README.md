# Current allocation traffic attribution

The [report](../../../../../docs/S07-bis-allocation-traffic.md) records exactly one
full one-worker diagnostic after expanded allocation/lifetime calibration.
All workload counts and the loaded digest match; all 13,094 files bind in place.
Seven symbol-table tests, nine parser tests and Clippy passed before capture.

The pipeline requests **2,244,181,769 bytes**, with **1,910,449,245 bytes** of net
live growth and **333,732,524 bytes** freed or superseded. Observed families
attribute **225,443,073 bytes** of that traffic; **108,289,451 bytes** remain
unclassified. Typed-row directory growth and text-entry growth account for
65.429 MB and 59.949 MB respectively. Neither establishes a 209 MB saving or a
passing RSS result. CPU/RSS acceptance and the retained control are unchanged.

- Archive: **13,332,668 bytes**, **1,269 files**.
- SHA-256: `5381e1322061f7661b0fd91a4297da33a68e6553e93ba274d50a8ac5bcbe305b`.
- Every member was read back and checked against [archive.json](archive.json).
- Contains the exact successful staged source and executables, frozen adapter,
  calibration/test/build logs, capture tools, raw output, per-family/per-phase
  counters, summaries and verification records. Executables are included because
  this observer changes the frozen binary; they compress with the sources to the
  archive size above.
- Both failed builds are retained. The first stopped on Clippy and had a
  review-found eager-buffer lifetime hole; its original driver was not copied
  before failure, an explicit provenance limitation. The second failed on a
  calibration type path. Neither ran the workload. A sandbox `ps` preflight
  failure happened before launch; the successful capture used approved process
  inspection and invoked the workload once.
- The original source freeze is supplied by the preceding
  [shared-text archive](../2026-09-09-compact-text-processing/README.md). The
  imported name/table observer is verified against its
  [existing archive](../2026-09-09-name-table-attribution/README.md). Workload files
  and toolchains remain external; the frozen source bundle is unchanged.

Restore paths relative to the repository root and run
`tools/s07/allocation-traffic/summarize.py` with the capture manifest hash
`236acf700e24569b562bdc692055d8ffbf8c0ff2212600bea9e135397c073887`
and a new `--output` path to recompute the result without executing a workload.
The report gives the full command. The summarizer verifies the sealed inventory
and refuses to overwrite existing output. To validate or reuse the restored
capture/build executables, also restore the sealed directories' read-only state
(`chmod -R a-w` on the two directories listed in `archive.json`); tar preserves
file modes but this archive stores files without directory entries.

`tools/s07/allocation-traffic/package.py` records the packaging procedure and
refuses to overwrite the archive. Its inputs include all failed attempts and
raw observations; no repeated run was used to select this result.
