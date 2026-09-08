# S07-bis performance experiments

This directory implements the diagnostic screening policy in
[the S07-bis plan](../../../docs/S07-bis-performance-plan.md#6-screening-and-decision-rules).
It writes no tracker metrics and does not replace full graph or ownership parity.
`layout_model.py` and its inputs implement the separate CP0 layout model.

The first [reviewed implementation checkpoint](../../../docs/S07-bis-A0.md)
retains A0-b as the next experimental control. For a new candidate, pass
`--control target/s07-bis/a0b-candidate` and
`--control-sha 124956f668540814b95e6f677bdeae98bb2cad4f7586d3cb87f869e5c5af118f`
to `build`, `graphs` and `screen`. Its already sealed candidate bundle can serve
as the control without rebuilding or changing its manifest. Preserve the original
control as well: the commands below document the first experiment. This manual
checkpoint decision follows full graph, ownership and targeted-cost review; it
does not change `freeze-control` into an automatic promotion mechanism.

The original control is frozen under `target/s07-bis/control`; its externally
recorded identity is [manifests/control.json](manifests/control.json). It contains
the exact normal Rust, allocation Rust and Go executables from the accepted S07
capture, its raw observations and full graph prerequisite, a copy of every
fingerprinted source, and copies of all 13,094 workload inputs. Files and
directories are read-only. Artifact hashes are checked against the original
validated capture before freezing. Ordinary rebuilds cannot replace this copy.

The command below freezes a *new* accepted control only when the complete native
capture and graph prerequisite validate against the current source/configuration
and actual shared-helper binaries. Existing destinations always fail. Preserve
the returned manifest SHA outside that directory before any experiment.

```sh
python3 tools/s07/performance-experiments/runner.py freeze-control \
  --output target/s07-bis/another-control
```

Build a candidate only after its implementation and build configuration have
stopped changing. Use the printed SHA in the subsequent screen command:

```sh
python3 tools/s07/performance-experiments/runner.py build \
  --control target/s07-bis/control \
  --control-sha c8e5dd18abb62a9cdd3c3af47051cc933c7744995a9a7029ac20a53bf8ea6363 \
  --output target/s07-bis/a0-candidate \
  --label a0-exclusive-binding \
  --hypothesis 'Exclusive binding reduces node-overlay routing on ordinary parsed files.' \
  --target-metric wall_time_ns
```

The builder reuses the accepted profile enforcement and Cargo artifact selection.
It preserves caller Cargo/Rustup homes, registries, offline and cache configuration;
it does not install toolchains. It runs the allocator accounting preflight on the
current candidate, copies each normal/allocation executable immediately after its
build, then freezes source, configuration hashes and declared hypothesis. System
runtime dependencies are recorded; a non-system dynamic dependency must be
explicitly bundled before measurement. The Go executable remains the frozen
reference; diagnostic pairing is candidate/control Rust, not a fresh Go gate.

Capture full graph parity and binding-path provenance on the same immutable
normal candidate before timing. This reuses the original graph comparison and
first-mismatch witness machinery for all files at both worker counts. The extra
`--binding-paths` invocation is untimed and confirms exactly how many files used
exclusive binding or fallback; its loaded-byte digest must match the workload.
The Go and Rust graph streams run concurrently with a maximum of two children;
binding-path probes follow them. Timing screens remain serial. Graph reports
record the runner/helper hashes and reject helper drift during capture.

```sh
python3 tools/s07/performance-experiments/runner.py graphs \
  --control target/s07-bis/control \
  --control-sha c8e5dd18abb62a9cdd3c3af47051cc933c7744995a9a7029ac20a53bf8ea6363 \
  --candidate target/s07-bis/a0-candidate \
  --candidate-sha SHA_PRINTED_BY_BUILD \
  --output target/s07-bis/a0-graphs
```

```sh
python3 tools/s07/performance-experiments/runner.py screen \
  --control target/s07-bis/control \
  --control-sha c8e5dd18abb62a9cdd3c3af47051cc933c7744995a9a7029ac20a53bf8ea6363 \
  --candidate target/s07-bis/a0-candidate \
  --candidate-sha SHA_PRINTED_BY_BUILD \
  --graph-report target/s07-bis/a0-graphs/report.json \
  --output target/s07-bis/a0-screen

python3 tools/s07/performance-experiments/runner.py verify target/s07-bis/a0-screen
```

Run screens on a quiet host after all builds/tests/profiles have stopped. The
runner shares the S07 measurement lock, rejects visible competing compiler or
benchmark processes before every child, and rejects source/configuration/helper
drift across the capture. This check cannot prove that unrelated applications
are idle. The caller remains responsible for coordinating other work.

Each variant receives one validated warmup in each mode, retained separately.
There are exactly seven alternating control/candidate pairs at one and eight
workers, separately for normal and allocation binaries: 8 warmups and 56 samples.
Every fresh child validates the complete workload counts and its own loaded
bytes/options digest. Every raw stdout/stderr is retained, including failures.
Binary/source inventories are checked around sampling batches; the manifest hash
comes from outside each immutable bundle. Missing, additional, duplicated,
reordered or malformed rows fail replay; no sample is selectively dropped or
extended to improve a result.

Timing, lifetime OS peak RSS and requested allocation remain distinct domains.
The fixed S07 bootstrap is reused with explicit candidate/control labels only for
timing. Both modes require relative MAD <= 5%; timing upper 95% ratio <= 1.02,
and RSS/allocation median ratios <= 1.02. A targeted pipeline win requires a
median ratio <= 0.95, plus timing upper < 1.0 when claiming a timing win.
`eligible_for_review` is diagnostic: measured targeted-cost reduction and full
semantic evidence still require review. No result here promotes an implementation
automatically or changes any S07 acceptance gate.

`--graph-report` is optional for an early cost-only screen. When supplied, missing
files/modes, graph mismatches or artifact substitutions fail before timing, and
the graph report hash is rechecked afterward and during replay. A complete graph
diagnostic is still not a tracker capture: it names immutable build snapshots,
which can differ from the current checkout, and produces no `run.*` evidence.

```sh
python3 -m unittest discover -s tools/s07/performance-experiments -p test_runner.py
```
