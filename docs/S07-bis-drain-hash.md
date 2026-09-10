# S07-bis: bounded CPU diagnostic and drain/hash candidate

Status: steps 1 and 2 authorized on 2026-09-10; implementation and measurement
in progress. Step 3 is a recommendation to report after this result, not an
authorized second implementation. No acceptance thresholds change.

## Frozen control and scope

Use the [current paired acceptance](S07-bis-candidate-acceptance.md) bundle at
`target/s07-bis/current-candidate-acceptance-2026-09-10/frozen`, manifest
`1c04605dcd248d78dcec1e42c4c6f82f036843a054cf5e27d81c1b3b229753b5`.
It contains the exact graph-tested executables and current production source
`1c661958f0982af9d34dcfd562f6b2c7d6d87c664e4d7dc3fc6dac682ceefbd3`.
Its paired CPU ratios are 1.213911 / 1.404619; worst allocation and RSS ratios
are 0.738066 / 0.737440. The existing 1.0 / 0.70 final gates remain open.

The earlier profile weights identify questions, not promised savings. Do not
add estimated component gains, infer an architectural checked-access floor or
attribute the eight-worker gap to dispatch from scaling arithmetic alone.

## Step 1: two bounded diagnostic questions

1. Capture exactly one one-worker Time Profiler run of the frozen normal Rust
   executable. Reuse the existing XML/symbol analyzer. Verify binary, input and
   workload identities before/after; retain all raw outputs and trace inventory.
   Report parse/bind displayed self rankings, overlapping inclusive groups and
   missing/unassigned samples. Examine construction, parent attachment, empty
   diagnostic drains, hashing, final validation and contextual identifiers.
   No diagnostic rebuild, trace expansion or repeated sample is scheduled.
2. Build a diagnostic copy of each frozen Rust/Go driver, preserving consuming
   binding, round-robin bounded queues, full input order, retention and pipeline
   endpoints. Execute each at eight workers exactly once. Record worker work,
   receive elapsed and completion offsets plus sender-call elapsed. Verify the
   actual file assignment and all existing workload identities. These elapsed
   clocks include scheduling and timer overhead; receive/send elapsed is not
   pure blocked CPU. Preserve source patches and effective builds. This answers
   whether measured waiting/imbalance merits a later candidate; it cannot
   promote a scheduler change or replace acceptance timing.

Serialize native captures with all builds and other measurements. The user has
authorized keeping Codex open; record observed host conditions without claiming
complete machine inactivity. Analyze existing traces offline rather than
starting another diagnostic when a displayed inline operation remains unresolved.

## Step 2: one combined production candidate

### Empty diagnostic drain

`Scanner::drain_diagnostics` currently creates a `Vec::Drain` even when its
buffer is empty. Return the existing absent-drain iterator for that case.
Keep the buffer and capacity, nonempty emission order, full-drain drop behavior,
callback/ignored sinks and checkpoint behavior. Existing callback-versus-buffer
and parser error-order tests supply independent counterexamples. No parser
diagnostic is deferred or dropped to improve the benchmark.

### Canonical-name hashing

Apply the [hashing decision](S07-bis-hash-decision.md): keep each pool's existing
randomized keyed hasher and hash exactly one complete byte string with one
`write` followed by `finish`. Omit the generic slice's extra length-prefix block
in both ordinary and growth paths. Keep byte equality and collision comparisons;
this private whole-key operation is not a composable `Hash` implementation.
No dependency, fixed seed, hash caching, table layout or collision-policy change
is selected. Full block processing and finalization still cost time.

## Validation and one complete screen

- Review both production changes independently, including arbitrary-byte names,
  empty/prefix/NUL keys, growth, callback delivery and error ordering. Preserve
  exact ownership and validation contracts; do not remove a final graph scan.
- Run affected scanner/parser and AST/binder tests, strict Clippy, formatting and
  the declared Rust minimum. Run scanner and binder differential obligations,
  full workload graphs at both worker counts and the existing E3 ownership
  matrix. Check the new key-boundary test under instrumentation without claiming
  it widens the frozen 83-case S07 inventory.
- Freeze the complete candidate's normal/allocation binaries against the control.
  After full graphs pass and all competing work stops, use the unchanged runner
  for one fixed combined screen: eight warmups and 56 samples, both worker modes
  and all memory metrics. No component-only screen or selective extension.
- Apply the existing screening/small-improvement retention policy. Preserve any
  regression and the runner's raw verdict. A close memory result does not permit
  changing the gates or omitting memory measurements.
- If retained, refresh standard paired Go/Rust E5/E6 once on the final source;
  it is a separately budgeted acceptance batch, not an experimental rerun.
  Otherwise restore the retained production control and keep the failed candidate
  artifacts. Commit results and update the existing S07-bis PR with a normal push.

## Report before step 3

State what was measured, what changed and whether a specific remaining mechanism
justifies a second combined candidate. Distinguish required parse/edge work from
removable checks, and necessary symbol/flow validation from proofs actually
carried by local handles. Recommend scheduling work only if the matched worker
diagnostic supports it. If no proportionate candidate emerges, recommend stopping.
Do not implement step 3 or begin another profiling series in this task.
