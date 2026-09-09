# Rejected lookup-reuse experiment

The [result and decision](../../../../../docs/S07-bis-lookup-reuse.md) preserve
one fixed candidate/control screen. Wall differences were −52.010 ms at one
worker and −12.893 ms at eight (1.13% / 1.21%); neither meets the committed 5%
criterion. Memory did not meaningfully change. Production was restored to CP1.

`review.tar.xz` preserves 726 files: both frozen source/binary bundles, complete
one/eight-worker Go/Rust graph streams, all eight warmups and 56 measured child
outputs, the removed patch, focused validation logs and the original failed
test-fixture setup. It also includes the unchanged runner and Python helper
sources. Every member was read back and checked against [archive.json](archive.json).

- Archive bytes: 27,663,472.
- SHA-256: `70c2392f1a8742cd67990d1356cd78f307bfd7437ed290095ecfed03cf9ff97f`.
- Native reruns require the original frozen workload referenced by `inputs.json`;
  its 13,094 source files are not duplicated here.

The existing runner replays the local preserved report without rebuilding or
executing either benchmark:

```sh
python3 tools/s07/performance-experiments/runner.py verify target/s07-bis/lookup-reuse-screen
```

That command checks original bundle paths/permissions, manifest inventories,
graph prerequisites and every raw sample before recomputing the screen. The
successful replay log is archived. This is not a new archive-only replay adapter:
extracting this package elsewhere does not rewrite the recorded absolute paths
or restore every containing directory's immutable permissions automatically.

The full workload graphs passed under the existing normalization. The candidate
also passed 30 debug/release binder tests, formatting, Clippy and Rust 1.96.
The broader binder and ownership producers were reserved for a promising screen
and were not run. Nothing here certifies new S07 ownership coverage or final
Go-relative CPU/memory acceptance.
