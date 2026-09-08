# Initial A0 diagnostic result

The separately captured [A0-b revision](a0b-README.md) passes its checkpoint
screen and is retained. This page preserves the initial candidate's outcome.

**Initial A0 is not promoted.** The complete graph comparisons passed, but this
candidate did not demonstrate its declared timing improvement or satisfy the
screening timing bounds. Its recorded status is `regressing_or_uncertain`.
These results are diagnostic comparisons with the original Rust implementation;
they publish no E5/E6 metrics and do not establish passing Go-relative gates.

The experiment measured exclusive binding while retaining the original binding
field maps. A later A0b variant preserves the parse-validation proof during
narrow flag writes; it is a separate candidate and is not included here.

| Domain | Workers | Control median | Initial A0 median | Candidate / control | Timing upper 95% bound |
| --- | ---: | ---: | ---: | ---: | ---: |
| Pipeline wall time | 1 | 5.197467 s | 5.257710 s | 1.011591 | 1.035069 |
| Pipeline wall time | 8 | 1.218762 s | 1.226224 s | 1.006122 | 1.050673 |
| Lifetime peak RSS | 1 | 4.564500 GB | 4.506649 GB | 0.987326 | — |
| Lifetime peak RSS | 8 | 4.566761 GB | 4.509286 GB | 0.987414 | — |
| Requested allocation | 1 | 4.728500 GB | 4.642574 GB | 0.981828 | — |
| Requested allocation | 8 | 4.728503 GB | 4.642575 GB | 0.981828 | — |

GB is decimal. RSS comes from the normal binaries, and allocation from separate
instrumented binaries. Each cell uses all seven fresh-process observations.
Both variants' relative MADs satisfy the 5% condition, but both timing upper
bounds exceed the 1.02 screening limit. The timing intervals include 1.0, so the
screen does not establish a CPU regression either. Memory improved by about
1.27% in RSS and 1.82% in requests; neither reaches the 5% pipeline improvement
threshold, and the declared target was wall time.

All 56 measured observations and 8 warmups remain present. In particular, the
eight-worker candidate's normal-build sample at index 3 took **1.423901 s**; it
remains in the raw output, ledger, medians and bootstrap. Nothing was removed,
replaced or extended after observing the result.

The untimed graph capture compared every one of the 13,094 files against the
frozen Go executable at both worker counts. Per-file observations validate the
complete graph digests, graph counts, diagnostics and qualified identities.
Both modes matched. Separate untimed binding-path reports observed 13,094
exclusive bindings and zero fallbacks in each mode, with the expected loaded
input digest. The archive retains these per-file reports; it does not contain
full serialized ASTs for every successful graph or substitute for the broader
ownership evidence required before promotion.

## Exact candidate identity

| Identity | SHA-256 |
| --- | --- |
| Original control manifest | `c8e5dd18abb62a9cdd3c3af47051cc933c7744995a9a7029ac20a53bf8ea6363` |
| Initial A0 manifest | `ee3399a930061772ca08d912d8bb1f3283169ef467066c7880f1df6d79fdde94` |
| Initial A0 source | `1b800c9ee0cdaa4cb7176dda8e168d2a3adf65b60897310a853dbe61c79ad50a` |
| Initial A0 normal executable | `e022f07e60b9412475f58747f154f2eeb27ff36cd9900f407849b2459948936e` |
| Initial A0 allocation executable | `4a3e612508a7e2d8a08aaeb199a22e9e7e3e4c1d62e3bcf0b2ad707a67261739` |
| Frozen Go executable | `35172610a07a040b41d11aa7771b2f604120b0c2eca04044cb6672aa93cb0cf4` |

Initial A0 was frozen at revision `58db8c237388480ef179a3068bf03285c44f1079`.
By the time its screen ran, the checkout already contained the A0b changes:
`screen/report.json` therefore records `current_source_fingerprint.sha256` as
`ea16eb22eae1e951cd9f3e4892512171be21d61d360bd592b6acb8447711417c`.
That field guards against checkout edits during the screen. It is **not** the
measured candidate's source identity. Every child used the distinct immutable
initial A0 executables identified above, whose actual source fingerprint is in
the candidate manifest and graph report. The A0b fingerprint is not evidence
that A0b was timed in this batch.

The initial graph process was launched before graph-specific helper hashes were
added to the runner. Its existing report remains unchanged. Screen helper hashes
and the later archive replay implementation are recorded separately; packaging
does not retroactively manufacture graph-capture metadata.

## Contents and replay

- [a0-initial-raw.tar.xz](a0-initial-raw.tar.xz): all 150 original members,
  63,392,435 uncompressed bytes; 6,728,548 compressed bytes.
- [a0-initial-manifest.json](a0-initial-manifest.json): byte size and SHA-256 for
  every member, the archive, both variant manifests and external frozen
  obligations.
- [a0-initial-replay.json](a0-initial-replay.json): replay receipts, comparison
  hashes and the separate original-bundle verification receipt.
- [replay_a0.py](replay_a0.py): reproduces the inventory, complete per-file graph
  comparisons and screen statistics without executing the captured programs.

Archive SHA-256:
`20641807f0567a453f2322acabb2fe14e071bf7b10f70cab8b1aea70ade32aff`.

Archive sections are `manifests/`, `graphs/` and `screen/`. They retain both
variant manifests, both graph worker modes, every raw stdout/stderr, graph
failures files, original reports and the original screen replay output.
Executables, source snapshots and workload files are deliberately excluded.
Their original immutable bundles were verified before packaging; the archive
alone cannot revalidate bytes that it does not contain.

From the repository root:

```sh
python3 tools/s07/performance-experiments/results/2026-09-08/replay_a0.py
```

Replay checks every archive member, validates the checked-in frozen obligation
hashes, revalidates all 52,376 per-file graph observations, recomputes both graph
comparisons and both binding-path reports, checks raw child output against all
64 observation rows, and recomputes the fixed bootstrap/medians using every
sample. It confirms the original non-promoted outcome. No runtime, source or
workload data is fetched or executed.
