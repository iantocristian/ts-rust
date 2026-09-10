# S07-bis: acceptance under ADR 0021

The retained implementation passes the four owner-approved parse/bind criteria
in one fresh paired capture of `9ff695b5546407c09f9912615f2489f5a016648a`.
The thresholds were committed before building or measuring. The original
0.70 memory / 1.00 CPU targets remain failed historical targets; this result
uses [ADR 0021](adr/0021-parse-and-bind-performance-thresholds.md).

| Metric | Go, one worker | Rust, one worker | Ratio | Go, eight workers | Rust, eight workers | Ratio | Limit |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Wall time, seconds | 2.895942208 | 3.568695125 | 1.232309 | 0.572663417 | 0.770867666 | 1.346110 | 1.25 / 1.45 |
| Allocated bytes, MB | 2907.465872 | 2145.915941 | 0.738071 | 2908.277544 | 2145.915720 | 0.737865 | 0.85 |
| Peak RSS, MB | 3154.132992 | 2326.298624 | 0.737540 | 3167.698944 | 2329.313280 | 0.735333 | 0.85 |

MB means decimal millions of bytes. Both CPU 95% intervals pass: **1.187214 to
1.242925** at one worker and **1.293945 to 1.387920** at eight workers. Every
timing relative MAD passes the unchanged 5% limit; the largest is Go's 3.082%
at one worker. One-worker timing extended from 7 to 14 to 21 samples per
runtime under the predefined rule. Eight-worker timing and each allocation
mode stopped at 7 per runtime. All **84 recorded samples** are retained;
eight checked warmups were excluded by the existing protocol. No second
performance capture, outlier removal or threshold adjustment followed the result.

The CPU upper-bound margins are 0.007075 and 0.062080 in ratio units. They
are observed margins, not a guarantee that a future host/run will pass.

## Review amendments

The supplied patch was applied with these corrections:

- Reject nonfinite/nonpositive or duplicate E6 ledger criteria before measurement;
  snapshot the two thresholds once for the capture's stopping loop.
- Include the threshold ledger in the graph producer's inputs as well as the
  capture fingerprint and E5/E6 inputs.
- Enforce the ADR's macOS ARM scope at capture and consumption. The previous
  producer accepted other native host classes; those cannot inherit the new
  limits without another decision.
- Test the per-mode criteria, exact boundary, required extensions, malformed
  ledger values, provenance inputs and host restriction. All 41 benchmark
  tests and `cargo xtask validate` passed before capture.
- Correct the ADR's attribution claims: peak RSS is not a retained-object
  census; the current elapsed gap is not the earlier profile's sample weight;
  worker timing does not exclude scheduler effects; Linux ratios are unknown.
  The older traffic census remains attached to its own binary.

The optional 1.30 / 1.50 limits were not adopted. The E5 checker per-type
criterion remains 0.80 and belongs to S08.

## Provenance and execution

Both full-workload graph modes passed all **13,094 files**, with no mismatched
files, before sampling. Each child matched 161,740,237 loaded bytes,
19,593,488 nodes, 2,459,867 symbols, 423 parse diagnostics and 5,250 bind
diagnostics. The loaded-input digest remains `d4ff2ad3b0d590babb596522a7105d961dbb3c4f2ebc987488d88b7ebd8e0c88`.

The first prerequisite attempt failed before graph testing when a previously
restored mutable cache executable was read-only. Its log is retained. Only the
mutable cache copies were made writable; frozen historical binaries were not
changed. The prerequisite was then rebuilt and passed. The standard sequence
`bindworkload → capture → verify-capture → e5 → e6` completed successfully.

The host was macOS ARM64, Darwin 25.6.0, 18 available/physical CPUs and 64 GiB
RAM. Initial load averages were 3.749 / 4.397 / 4.590. The user had stopped
Cursor and explicitly requested measurement with Codex open. The producer
checked for competing compilers/benchmarks; no local builds, archive packing
or other diagnostics ran during native samples. This is not a claim that all
background host activity was stopped. Rust and Go pins, GC policy, allocator,
measurement domains and sample protocol are unchanged.

- Source fingerprint: `dd6694563c731819a6708512960c3f2c6531080f60d31935f060147992bf84e5`.
- Frozen native bundle: `20f3e2b07096949654217b7b0fe5d7beee277ab75648fe42995925e881147890`.
- Graph evidence: `6c8627273ab14f7592eca85e92afb6d7ffb55859a2497cf909c9a82d476361c6`.
- E5 evidence: `b759d3fd808558dba3f4af4412874a414a65e12198d300fd3fd7b59088a43ff8`.
- E6 evidence: `2b01576766f38e65a440cc933442c5fd7df833d8c89924ff8448797ce8fbf36a`.

The [archive](../tools/s07/performance-experiments/results/2026-09-10-gate-rebase)
retains exact executables, source, ledger, raw graph streams, sample reports,
producer evidence and frozen Python validators. The preceding captures keep
their own validators and original failing thresholds. No old result is regraded.

## Tracking and CI

The initial S07 check passed S07-4 but had stale correctness prerequisites.
Refreshing them exposed two older bookkeeping defects: self-test counts still
expected 76 ownership cases instead of the inventoried 83, and 14 operation
inventory mappings held outdated Rust line locations. The assertions and line
locations were corrected. All other operation fields are identical; the subset
and checker-obligation documents are byte-identical. The accepted subset review
now carries an explicit metadata-only amendment and the prior review hash.
These changes do not alter the native benchmark source fingerprint.

All registered evidence is current, and the final S07 check passes every exit
condition and all six items. E2 retains 10,728 variants; program loading and
option verification pass all 10,728 each, plus 25 helper tests. Tracker
self-tests pass after the count correction. Four-target CI is reported on
PR #12 separately from this frozen native performance capture.
