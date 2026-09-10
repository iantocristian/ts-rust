# Page, text and helper experiment — 2026-09-10

The [complete result](../../../../../docs/S07-bis-pages-text-helpers-result.md)
retains candidate `8d837bc0…` under the existing reviewed-small-improvement rule.
The runner's `no_demonstrated_win` verdict is preserved. Same-screen wall time
improves 4.67% / 3.93%, allocation requests fall about 97.7 MB, and peak RSS rises
about 7.1 MB (0.30%). Final S07 gates remain open.

The screen contains eight warmups and all 56 observations. Full 13,094-file graph
parity at both worker counts, the full binder corpus, and E3's 29 S06 / 83 S07 cases
in debug/release/Miri/ASan pass. One matched consuming phase observation per
variant is included, with its elapsed-only limitations.

## Distribution and replay

The unchanged 134,605,924-byte archive is split into three parts, each at most
48 MiB, to stay below GitHub's per-file limit. `distribution.json` identifies every
part and the exact assembled archive. Its SHA-256 is:

`21e0ad82aa4b31a625c149a71fcb1a7d9e4f3817e222d610f67e7633335f634f`

With Python 3.11 or newer, run from this directory or pass the script's full path:

```sh
python3 verify.py --output /tmp/s07-pages-text-helpers-replay
```

The output path must be new. The launcher checks all part, manifest and assembled
archive hashes; it then runs the archive's unchanged verifier. No Rust/Go build,
network fetch, workload checkout or native benchmark execution is required.
Allow roughly 1.2 GB of additional space for assembly and extraction. The extracted
review tree remains in the selected output directory; temporary assembly is removed.

All 2,368 members and all eight replay obligations passed locally:

- Refreshed native Go/Rust graph and timing baseline.
- Complete candidate graphs and local binding paths.
- Fixed combined screen and raw sample arithmetic.
- Both independent phase captures.
- Full binder and E3 producer records, sources and inventories.
- Final compiler, generator and test prerequisites.

`verification.json` records the checked identities and outcomes;
`verification-report.json` preserves the full local replay receipt. Absolute
original paths inside receipts are provenance, not dependencies of offline replay.

## Retained evidence and limits

The archive includes both frozen source/artifact bundles, baseline refresh
attempts, all raw measurements, complete producer evidence, matched phases,
validation/review records, decision documents and tracker views. It preserves the
initial sandbox/build-identity failures, preliminary compile/lint failures and
pre-execution lock rejections. Compiler caches, toolchain installations and duplicate
workload copies are excluded explicitly by the pinned recipe.

Only same-screen Rust/control changes are attributed to this candidate. Go gate
distances use the separately refreshed native batch and are not fresh paired
candidate/Go acceptance. The RSS increase remains explicit, the automatic screen
verdict is unchanged, and stale acceptance rows remain pending in the tracker.
