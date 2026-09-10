# Compact shared-text experiment

Diagnostic evidence for implementation `aa6a5f6`, frozen with the Cargo
build-isolation correction `8f7236e`. The corrected full combination remains
experimental; all final Go-relative gates remain open.

| Metric | Same-batch CP1, one / eight workers | Candidate, one / eight workers |
| --- | ---: | ---: |
| Wall | 5.078450 / 1.087606 s | 5.180391 / 1.079982 s |
| Requested allocation | 4.642568 / 4.642573 GB | 2.244183 / 2.244184 GB |
| Peak RSS | 4.506599 / 4.509237 GB | 2.318565 / 2.321842 GB |

CPU ratios are 1.020073 / 0.992991; upper 95% bounds are 1.055861 / 1.006322.
One-worker nonregression is not established; neither mode demonstrates a CPU
win. The memory reduction does not promote the combination. This screen cannot
isolate the text change's effect from a different earlier batch.

`review.tar.xz` contains **813 files**, **33,145,092 bytes**.
SHA-256: `f05a8bff9d53ffc76eac7c0066ff9fc93969516b40b56c2e905cc16264d2fe5a`. `archive.json` inventories every member's
size/hash; all members were read back and verified after compression.

The valid inputs are `text-processing-clean-candidate` and
`text-processing-clean-graphs`; the complete `text-processing-screen` contains
all eight warmups and 56 samples. All 13,094 semantic graphs match at one/eight
workers, with 13,094 in-place bindings and zero fallback each. Root receipt
verification and independent raw-file/statistical recomputation pass. Raw
stdout/stderr, manifests, exact source/binaries, build logs, test scripts/logs,
S05 generation inputs, three failed/successful producer envelopes and review
notes are included.

**The initial `text-processing-candidate` and partial `text-processing-graphs`
are invalid evidence for this implementation.** Cargo reused binaries from the
preceding phase diagnostic through a shared intermediate cache. That attempt
was stopped before timing. Original dep-info/fingerprints and both independent
audits are retained. Corrected builds isolate final and intermediate output in
fresh directories; six regression tests cover the failure boundary. Earlier
frozen screens/profile and the freshly built phase diagnostic are unaffected.

The S05/E4 producer envelopes retain aggregates and stream digests, not every
raw token observation. Scoped Miri/ASan and affected tests do not claim a fresh
complete ownership producer. Workload sources and installed toolchains remain
local/external prerequisites. CP1 source/binaries and unchanged helpers are
pinned through the earlier archive identified in `archive.json`. Receipt replay
requires the original bundle paths and immutable permissions. No additional
performance batch or sample extension was used to rescue the result.
