# S07 CPU diagnosis

The adapters below preserve the original S07 published-file binding path. S07-bis
now uses consuming parse → bind → publish on eligible files, so these adapters
must not be used to attribute the current production path without updating and
revalidating them. The first compact-storage diagnosis instead samples the exact
frozen normal benchmark executable; see its
[result record](../../../docs/S07-bis-compact-storage.md). That capture has no
explicit phase wrappers and makes no exact exclusive phase-attribution claim.

These adapters profile the frozen parse-and-bind workload at one and eight
workers. They retain the benchmark's per-file parse → publish → bind order,
bounded round-robin queues, and all completed file roots through the endpoint.
They do not produce E5/E6 acceptance evidence or change its production binaries.

On the designated macOS host, with full Xcode and the repository's pinned Rust
and Go toolchains installed:

```sh
python3 tools/s07/cpu-profile/build.py
python3 tools/s07/cpu-profile/capture.py --output target/s07-cpu-profiles/capture
python3 tools/s07/cpu-profile/analyze_xctrace.py selftest
python3 tools/s07/cpu-profile/test_analyze_pprof.py
python3 tools/s07/cpu-profile/analyze_pprof.py
python3 tools/s07/cpu-profile/analyze_xctrace.py export \
  target/s07-cpu-profiles/capture/rust-1-0.trace \
  --output-prefix target/s07-cpu-profiles/capture/rust-1-0
```

Repeat the export for each Rust trace. The analyzer resolves physical functions
using the matching Mach-O UUID and executable address ranges, retaining the
displayed inline/source names separately. It uses the recorded executable when
available, or an archived `--symbol-map` for replay without the live executable.
Unassigned worker samples remain explicit. See the [archived capture](results/2026-09-08/README.md)
for replay instructions and the [reviewed findings](../../../docs/S07-cpu-profile.md).

The Go analyzer uses pinned `go tool pprof` raw/top/tags views and checks every
profile against the capture inventory and hash. Its `phase=parse` and `phase=bind`
labels distinguish operation scope, while unlabelled runtime/GC samples remain
in the CPU denominator. Runtime/GC costs can also occur inside labelled phases.

`capture.py` requires the existing validated S07 benchmark and graph captures,
and provisioned inputs in `../.ts-rust-workloads/s07-benchmark`. Run captures
without concurrent builds, other benchmark/profile processes, or edits to source
or profiling tools. Each fresh invocation must reproduce the frozen input
digest, 13,094 files, 161,740,237 bytes, 19,593,488 nodes, 2,459,867 symbols,
423 parse diagnostics and 5,250 bind diagnostics. An interrupted capture retains
its raw records and does not publish a complete capture report. A completed
capture still requires nonempty, valid CPU-sample analysis.

Rust uses optimized builds with packed debug symbols, named non-inline phase
frames and per-file elapsed timers. Time Profiler records running-thread CPU
samples across the process lifetime. Go records CPU samples only around the
pipeline, using `runtime/pprof` and phase labels. Keep Rust preload, reporting,
retirement, scheduling and missing-stack samples visible and distinguish them
from parse/bind work. The clocks in the adapters measure elapsed worker time,
including descheduling; their sum is not CPU time and can exceed pipeline wall
time with multiple workers.

Fresh controls retain the original Go and Rust binaries, plus the Rust adapter
without Time Profiler. They reveal Rust adapter and additional profiler effects;
the Go comparison measures the combined label/timer/profiler effect. These
small diagnostic samples do not replace the existing seven-sample acceptance
measurements. Raw traces stay under `target/`; summaries and a reviewed report
record the conclusions separately.
