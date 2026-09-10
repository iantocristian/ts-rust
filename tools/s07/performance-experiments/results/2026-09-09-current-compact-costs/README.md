# Current compact candidate: phase, live-request and CPU diagnostics

The [current-cost record](../../../../../docs/S07-bis-current-costs.md) describes
one full normal phase run, one full allocation phase/live-request run and one
native sample of the unchanged frozen normal binary. Every run validates the
same 13,094-file workload and loaded digest. These are diagnostics; the
[complete paired screen](../2026-09-09-compact-parser-lists/README.md) remains
unchanged and the compact combination remains experimental.

Normal elapsed phases are 2.748578340 s parsing and 2.741912765 s consuming
binding/publication. Pipeline requests of 2,244,178,073 bytes comprise
1,910,445,549 bytes of net live growth and 333,732,524 bytes of freed or
superseded requests. Live requested bytes are not RSS or a storage-domain census.
The native sample attributes 398 ms of distinct worker weight to keyword
comparisons and slice UTF-8 validation; this is not a promised saving.

- Archive: 9,756,864 bytes, 747 files.
- SHA-256: `4ad21747d75f75f9cf443bdcb6f81e2ffac3f0f107ee5d222fc1af9a60ef33d7`.
- Every member was read back and hash-verified against [archive.json](archive.json).
- Includes exact patched source/binaries, added source closure, original manifests,
  build/protocol/workload logs, failed metadata-pruning attempt, raw diagnostic
  output, CPU exports and source/counter/caller audits.
- The unmodified normal binary and original source freeze are pinned in the
  preceding [candidate archive](../2026-09-09-compact-parser-lists/README.md).
- Workload sources and the native trace directory remain local. Exported XML,
  symbolized sample data and its analysis are included.
- Native command/receipt files contain the exact invocation. `capture.py` was
  imported only for helpers; its old-adapter `main()` was not used for this run.

Recheck the phase source/observation receipt using the retained
`target/s07-bis/parser-list-phase-memory-adapter/verify.py`. Recompute native
sample aggregation using `target/s07-bis/parser-list-profile-audit.py` after
restoring its pinned input/helper paths. Neither command launches a workload.
Full-host quiet was not measured; single-run phase values and CPU samples do not
establish comparative timing or final Go-relative acceptance.
