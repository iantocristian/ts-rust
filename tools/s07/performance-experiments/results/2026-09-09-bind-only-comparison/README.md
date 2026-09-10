# Offline bind-only CPU comparison

The [report](../../../../../docs/S07-bis-bind-cpu-comparison.md) compares the
current compact Rust bind self-cost ranking with all three historical one-worker
Go bind partitions. The [plan](../../../../../docs/S07-bis-bind-cpu-plan.md)
selects one combined flow-write/source-fact candidate. Its production changes
and timing screen are pending. No new compiler/profile process ran for this
comparison; the current accepted control and final gates are unchanged.

- Archive: 79,328 bytes, 16 files.
- SHA-256: `8ec4f02f70fb77da1e5f3153168250008341be667a3ae8a169881080375b1abc`.
- Every member was read back and hash-verified against [archive.json](archive.json).
- Includes the full result, selectors/analyzers, unit tests/log, plans and local
  review. The review is by the implementing agent, not an independent agent.
- Earlier comparison outputs and the failed optional native-display check are
  retained. That check was corrected for pprof's unitless zero formatting;
  the final result validates 90 native rows across three profiles.
- Original Rust exported samples/symbols remain in the preceding
  [CPU archive](../2026-09-09-post-text-cpu/README.md); Go profiles/exports remain
  in their [original archive](../../../cpu-profile/results/2026-09-08/README.md).
  Their hashes and each exact input/helper hash are recorded. Neither source
  executables nor native traces are duplicated here.

Rust's selected bind weight is 2,528 ms. Go's three values are 730/670/690 ms.
These are sampled weights from different captures and sampling intervals, not a
paired CPU ratio. Percentages use the selected bind denominator; pooled Go
weights cover three runs. Runtime/systemstack work stays in its labeled phase,
with unlabeled background GC preserved separately. Inclusive groups overlap;
self ranks do not isolate all inlined work or promise a speedup.

For reproduction in a checkout of this revision, restore these four members
from the preceding Rust archive when absent:

```sh
tar -xJf tools/s07/performance-experiments/results/2026-09-09-post-text-cpu/review.tar.xz \
  target/s07-bis/post-text-cpu/benchmark.stdout \
  target/s07-bis/post-text-cpu/benchmark.summary.json \
  target/s07-bis/post-text-cpu/benchmark.time-profile.xml \
  target/s07-bis/post-text-cpu/benchmark-audit.json
python3 -m unittest discover -s tools/s07/cpu-profile -p test_compare_bind.py -v
python3 tools/s07/cpu-profile/compare_bind.py --output target/s07-bis/bind-only-comparison/reproduced.json
```

The command refuses to overwrite an existing result. Complete Go raw exports
and their native top displays are checked in; no Go executable is needed.
