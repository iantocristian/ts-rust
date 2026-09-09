# First integrated typed-storage screen

The [implementation record](../../../../../docs/S07-bis-compact-storage.md)
reports the outcome: allocation falls by 1.458 GB and peak RSS by about 1.67 GB,
but wall time regresses 61% at one worker and 54% at eight. **Not promoted.**
CP1 remains the retained control while the new read overhead is repaired.

`review.tar.xz` preserves 756 files: both frozen source/binary bundles, complete
one/eight-worker Go/Rust graph streams, all eight warmups and 56 measured child
outputs, validation logs, and the unchanged measurement helpers. It also retains
the single native CPU profile's exported XML and physical symbol map. Every
member was read back and checked against [archive.json](archive.json).

- Archive bytes: 29,324,276.
- SHA-256: `6b8434a01d5e9f9231c6954cfca565bcd551c9dd5852a498559626491d412e79`.
- The frozen workload files and native Instruments trace directory remain local.
  Complete graph streams and exported CPU samples are included.
- Replaying the original receipt requires the recorded local paths and immutable
  bundle permissions. Extracting the archive does not rewrite those identities.

```sh
python3 tools/s07/performance-experiments/runner.py verify target/s07-bis/compact-typed-screen
```

The successful receipt replay is archived. Full workload graph comparison passed
with all 13,094 files on the exclusive binding path. Broader E3/binder producers
and fresh Go-relative performance acceptance remain outstanding. Neither the
memory improvement nor the single CPU sample is a final sprint gate result.
