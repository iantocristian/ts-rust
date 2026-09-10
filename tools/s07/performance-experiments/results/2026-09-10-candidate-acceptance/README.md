# S07-bis: current candidate paired native acceptance

This archive preserves one standard paired Go/Rust capture of revision
`26b6c4efc507e002130330aad30f9a5496d20f1e`. Both 13,094-file graph modes pass.
The original stopping rule completes after 56 recorded samples, with no extension
or repeat. E5/E6 consumers and native verification succeed; the performance gates
remain open.

| Ratio (Rust / Go) | One worker | Eight workers |
| --- | ---: | ---: |
| Wall time | 1.213911 | 1.404619 |
| Requested bytes | 0.738066 | 0.737854 |
| Peak RSS | 0.737440 | 0.735532 |

CPU ratio 95% intervals are 1.149962–1.235345 and 1.280184–1.442985. The
producer's `stable` metric is false because its CPU upper bounds exceed 1.0;
all four runtime/mode timing relative MADs remain below 2.66%. These are the
same-batch candidate/Go results, not arithmetic against another batch.

Cursor was stopped by the user; Codex remained open at the user's explicit
instruction. Initial load average was 4.933594 / 5.233398 / 5.277832 on the
18-CPU, 64-GiB host. The standard producer checked for competing compilers and
benchmarks. No closed-Codex or 95%-idle claim is made. `run-context.json` retains
this limitation and states that no detached wrapper, sampler hook, or producer
modification was used.

## Contents and verification

`review.tar.xz` contains 417 content-addressed files (135,892,952 bytes before
compression): the three exact executables, 289 production source files, both
raw graph modes, the complete sample ledger and reports, command logs and source/
host provenance, E5/E6/graph evidence, and frozen validator dependencies. Full
workload and compiler build trees, previous backups, and unrelated experiments
are omitted. The frozen manifest retains the original workload inventory. Replay
checks recorded workload identities and graph content; it does not reread the
omitted source bytes or rerun native code.

The native replay is the previously reviewed `replay_performance.py` with only
its two frozen manifest/revision constants updated. `archive.py` and
`archive_helpers.py` are byte-identical to the prior reviewed archive machinery.
Their frozen copies and provenance are included. The archive was extracted and
all members were hashed; replay then revalidated both graph modes and recomputed
all statistics and gate metrics from the 56 recorded samples. Both verification
reports beside this file are the actual successful outputs. The manifest keeps
its original packaging-stage status; `verification.json` records completed replay.

Requires Python 3.11 or newer, approximately 140 MB free space, and no Rust/Go
compiler, workload checkout, network, or native executable invocation. From the
repository root, use a new output directory:

```sh
python3 tools/s07/performance-experiments/results/2026-09-10-candidate-acceptance/archive.py replay \
  --manifest tools/s07/performance-experiments/results/2026-09-10-candidate-acceptance/archive.json \
  --output target/s07-candidate-acceptance-offline-replay
```

The replay refuses to overwrite its output and verifies archive/member/anchor
hashes before executing its single declared Python validator. The retained
executables are hashed, never executed. Read `archive.json` for the full member
inventory, recipe, anchors, and explicit omissions.

- Archive SHA-256: `fd53fb29fd5c1fad9f46816c5b428bfbf126aaaf73439888e0df216aef6df283`.
- Frozen candidate manifest: `1c04605dcd248d78dcec1e42c4c6f82f036843a054cf5e27d81c1b3b229753b5`.
- Native graph report: `0269757498dced1af8910c4ca529876d216527a47a5211881f1e1ed889f2947a`.
- Native measurement report: `fb7ebaa62baa8e30e1a0168b0b6d9994f693ed713e7af1aff5cfe2f45c2f47ad`.
