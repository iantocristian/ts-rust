# Current canonical-name and symbol-table attribution

The [result and decision](../../../../../docs/S07-bis-name-table-attribution.md)
record exactly one full one-worker allocation diagnostic after an exact tiny
allocator calibration. All workload counts and the loaded digest match; all
13,094 files bind in place. Seven symbol-table tests and Clippy pass.

Five measured backing families request 143,318,442 bytes, retain 85,842,425 bytes,
and replace 57,476,017 bytes during growth. Logical elements occupy 45,194,194
bytes; required hash controls/load factor prevent treating the remaining backing
bytes as entirely removable. The impossible exact-element/no-growth ceiling is
98,124,248 requested bytes, below half the roughly 209 MB allocation deficit.
No canonical-name/hash-backing rewrite is selected. CPU and RSS gates are unchanged.

- Archive: 10,063,932 bytes, 882 files.
- SHA-256: `dd0783fd71b42f7ae38abf176d7ab62ac3eda22e5a2c45f86d43ac4f136048d6`.
- Every member was read back and hash-verified against [archive.json](archive.json).
- Contains exact patched source/binaries, fixed-counter patch/calibration, source
  and result reviews, all raw output/check logs, and the first failed build.
- The first build lacked four test fixtures; the corrected build restores their
  exact `8f7236e` blobs. The failure is preserved and no full capture ran from it.
- The original source freeze is in the preceding
  [shared-text archive](../2026-09-09-compact-text-processing/README.md).
  Workload and toolchains remain external. Re-running the capture's bundle
  validation requires that frozen bundle restored at its original path.
- Table-row/owner logical sizes do not assign their physical arena capacity.
  Name-to-owned byte counts do not infer Arc requests. These are not RSS or CPU
  measurements suitable for acceptance; no second workload capture occurred.

Recompute the result without launching a workload using the retained
`target/s07-bis/name-table-summarize.py`. The result review independently checks
both sealed inventories, exact fixture/workspace sources, helper hashes, all five
calibration phases and every raw accounting total.
