# Complete local-bind candidate evidence

This archive retains the fixed diagnostic screen, both complete graph modes, local-scope observations, scoped validation, one phase observation and the documented decision. It does not publish sprint or acceptance evidence.

The v1 screen is eligible for review and the full binder producer passed. The broader E3 producer failed its exact test-inventory checks; that failure is retained. V1 was not promoted, and CP1 remains the control. Later text and inventory corrections are excluded.

Every extracted member is verified against archive.json before numerical replay. The source hashes are authoritative; the build was frozen before its staged source was committed as 7eae473. CP1 control source/binaries and full workload bytes are not duplicated. Generator/schema and dynamically loaded comparator support are included.

After extracting review.tar.xz into a chosen directory, run from that directory:

```sh
python3 target/s07-bis/local-bind-completion-validation/package.py replay --manifest /absolute/path/to/archive.json --output /tmp/local-bind-complete-replay
```

The output directory must not exist. Replay launches only Python analysis, never a compiler, benchmark or workload process. It recomputes graph parity, all fixed-screen summaries, historical gate distances, local-scope selection and phase arithmetic. Initial failed validation logs remain alongside successful final checks.
