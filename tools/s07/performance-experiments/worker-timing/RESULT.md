# Current-baseline worker timing: one observation per runtime

The frozen baseline is the final paired-acceptance candidate, source fingerprint
`1c661958f0982af9d34dcfd562f6b2c7d6d87c664e4d7dc3fc6dac682ceefbd3`.
The diagnostic ran **one Rust then one Go invocation**, each with eight workers.
Both completed, every original work counter and loaded-input digest matched,
and every actual ordered worker assignment matched `index % 8` independently.
No native observation was repeated. Offline verification passed.

| Elapsed observation (ms) | Rust | Go |
| --- | ---: | ---: |
| Original whole-pipeline endpoint | 1,234.091 | 881.310 |
| Mean worker parse/bind/retention work | 773.997 | 531.292 |
| Mean worker receive-call elapsed | 458.160 | 348.487 |
| Mean completion-to-pipeline-end tail | 1.836 | 1.285 |
| Main sender send-call elapsed | 1,231.049 | 879.020 |
| Main sender completion offset | 1,231.429 | 879.598 |
| Slowest–fastest worker work difference | 226.747 | 146.978 |
| Latest–earliest worker completion difference | 2.647 | 1.702 |

These are **elapsed clocks**, including descheduling and runtime activity, not
CPU self time. Receive/send elapsed includes channel work and scheduling; the
sender's value cannot be interpreted as dispatcher CPU or pure blocked time.
Additional per-file clocks and preallocated index-buffer writes perturb these
adapters. The fixed order and single observations do not give confidence bounds,
component savings, a new gate ratio, or a causal scheduler diagnosis.

The same input distribution is uneven in both runtimes. Worker 6 receives
26.680 MB; each other worker receives 18.119–21.699 MB. It has the most elapsed
work in both. Every worker receives 1,637 files except workers 6 and 7, which
receive 1,636. The receive-call mean is 37.1% of Rust pipeline elapsed and 39.5%
of Go pipeline elapsed. This is not CPU utilization. Both runtimes spend
substantial time inside receive calls throughout work, while their final
completion spread is only a few milliseconds. The data do not support a claim
that Rust has a uniquely larger dispatcher/receive fraction or that end-of-run
tails explain its scaling difference. Changing dispatch would need its own
approved candidate and measurement; this diagnostic changes no policy.

| Worker | Source MB | Rust work ms | Rust receive ms | Go work ms | Go receive ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| 0 | 18.902 | 760.398 | 471.972 | 521.951 | 357.261 |
| 1 | 18.509 | 745.778 | 485.575 | 514.211 | 365.165 |
| 2 | 19.237 | 706.277 | 525.916 | 495.525 | 384.319 |
| 3 | 18.119 | 726.270 | 507.699 | 492.463 | 388.645 |
| 4 | 19.447 | 755.921 | 476.428 | 525.862 | 353.710 |
| 5 | 21.699 | 799.471 | 432.621 | 544.120 | 336.025 |
| 6 | 26.680 | 933.024 | 298.290 | 639.441 | 239.953 |
| 7 | 19.148 | 764.839 | 466.776 | 516.764 | 362.819 |

The measurement lock excluded cooperating captures/builds and the producer
observed no named competing compiler/benchmark processes. Host load averages
were `[10.329, 16.415, 12.132]` before and `[10.623, 16.375, 12.143]` after.
That is recorded context, not a proof that the host was idle or a reason to
repeat observations.

## Validation and provenance

- Work: 13,094 files, 161,740,237 bytes, 19,593,488 nodes, 2,459,867 symbols,
  423 parse diagnostics and 5,250 bind diagnostics in both runtimes.
- Loaded-input SHA-256:
  `d4ff2ad3b0d590babb596522a7105d961dbb3c4f2ebc987488d88b7ebd8e0c88`.
- Frozen baseline manifest:
  `1c04605dcd248d78dcec1e42c4c6f82f036843a054cf5e27d81c1b3b229753b5`.
- Successful build at `target/s07-bis/drain-hash/worker-timing/build-v4`:
  `fdad7a1933b2b1a24ce7f952e4ad1f4a74e7e748502b4bd0cf0bebc5d7e13ce9`.
- Instrumented Rust:
  `4923fe97d02f67e75d35bf1177db65514f06bd47e58cf65f736904e025f30fc9`.
- Instrumented Go:
  `01ff899ed6d8c9668bdcadd97ee98f27f9a5f8839aec05d0c71981324196780a`.
- Capture report at `target/s07-bis/drain-hash/worker-timing/capture/report.json`:
  `914ea60b6681b7d854c67f8292930e2e45993daa33ee3e03ab8af3efcb20eb49`.
- Rust 1.97.1, normal release features/profile with mimalloc; Go 1.27.1,
  GOTOOLCHAIN=local, GOMAXPROCS=8 and GOGC=100. All 39 resolved Rust registry
  packages retain frozen-lock versions/checksums. Rust output and intermediate
  build paths are newly isolated and Cargo's executable is checked inside them.
- Thirteen producer counterexample tests pass. They include wrong/reordered
  assignments despite matching counts, loaded-digest differences, missing
  workers, overlapping/out-of-endpoint clocks, changed allocator/domain
  policy, ambiguous/repeated patching, and partial timeout/launch-error retention.
- Root and an independent agent reviewed the final isolated-build and failure
  recording fixes before the two measured invocations.

Setup failures are retained separately: `build` rejected the missing uncompiled
parser test-only encoder manifest; `build-v2` rejected a lock not pruned by
metadata's earlier `--no-deps`; `build-v3` compiled Rust but Go could not access
its established cache in the sandbox. None executed a workload. The unselected
build-v3 Rust binary also predates the final review's fresh build-directory
requirement. `build-v4` applied the reviewed preparation and build rules and
used escalation for the existing Go cache. The successful capture was the
first and only native worker-timing capture.

```sh
python3 tools/s07/performance-experiments/worker-timing/probe.py verify \
  target/s07-bis/drain-hash/worker-timing/capture
```
