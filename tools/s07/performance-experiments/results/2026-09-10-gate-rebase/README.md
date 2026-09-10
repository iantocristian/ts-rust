# S07-bis: acceptance under ADR 0021

This archive preserves one standard paired Go/Rust capture of revision
`9ff695b5546407c09f9912615f2489f5a016648a` under the owner-approved thresholds in
[ADR 0021](../../../../../docs/adr/0021-parse-and-bind-performance-thresholds.md).
Both full-workload graph modes pass all 13,094 files. E5/E6 pass the new criteria;
the original memory and CPU-parity targets remain failed historical targets.

| Ratio (Rust / Go) | One worker | Eight workers | Limit |
| --- | ---: | ---: | ---: |
| Wall time | 1.232309 | 1.346110 | 1.25 / 1.45 |
| Allocated bytes | 0.738071 | 0.737865 | 0.85 |
| Peak RSS | 0.737540 | 0.735333 | 0.85 |

CPU ratio 95% intervals are 1.187214–1.242925 and 1.293945–1.387920. The
predefined stopping rule extended the one-worker timing samples from seven to
14 to 21 per runtime. All 84 recorded samples are retained; no second capture
or sample removal was used. Replay reads the frozen threshold ledger, whose
hash participates in the capture fingerprint. The limits apply to the measured
macOS ARM host class; hosted CI runs correctness/build checks separately.

Cursor was stopped by the user; Codex remained open at the user's explicit
instruction. Initial load average was 3.749023 / 4.396973 / 4.589844 on the
18-CPU, 64-GiB host. No local builds, profiling or archive work ran during
sampling. No completely idle-host claim is made; `run-context.json` records
the conditions and the retained native metadata records the host load.

## Contents and verification

`review.tar.xz` contains 468 content-addressed members: the three exact
executables, 290 native source files, both raw graph modes, the complete sample
ledger and reports, threshold policy and frozen validators, command logs and
source/host provenance, and graph/E5/E6 evidence. It also preserves the final
correctness receipt and metadata-only review of stale mapping locations.
Full workload and compiler build trees, previous backups and unrelated
experiments are omitted. The frozen manifest retains the original workload
inventory. Replay checks captured identities and graph content; it does not
reread the omitted workload bytes or execute native code.

`archive.py` and `archive_helpers.py` are unchanged from the preceding reviewed
archive. The native replay adapter updates the frozen manifest/revision constants
and points the threshold reader at the extracted source ledger. All validator
dependencies are frozen in the archive. Extraction verified every member and
anchor hash, then replay revalidated both graph modes and recomputed the
statistics and gate metrics from all 84 samples. `verification.json` and
`native-replay.json` are the actual successful outputs; the manifest retains
its original packaging-stage status. The other correctness receipts are
preserved evidence, not additional test executions by the archive replay.

Requires POSIX Python 3.11 or newer, space for the extracted archive, and no
Rust/Go compiler, workload checkout, network or native executable invocation.
From the repository root, use a new output directory:

```sh
python3 tools/s07/performance-experiments/results/2026-09-10-gate-rebase/archive.py replay \
  --manifest tools/s07/performance-experiments/results/2026-09-10-gate-rebase/archive.json \
  --output target/s07-gate-rebase-offline-replay
```

The replay refuses to overwrite its output and verifies archive/member/anchor
hashes before executing its single declared Python validator. The retained
executables are hashed, never executed. `archive.json` contains the full member
inventory, recipe, anchors and omissions.

- Archive: 22,658,180 bytes; SHA-256 `8848bf1d4485110ab6f46716ca35ce22dbe93304aeaae85299473de1f2d9972d`.
- Frozen candidate manifest: `20f3e2b07096949654217b7b0fe5d7beee277ab75648fe42995925e881147890`.
- Native graph report: `bacd70115cd9fb2535ae5632fccc82359b7a7e48c69db491ebd5fdd2639a0e72`.
- Native measurement report: `13d5f9e44065945a4b8b632faf7ba517033071cda836a1a9cc89676333751ce8`.
