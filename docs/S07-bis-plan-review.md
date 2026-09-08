# S07-bis plan review

Date: 2026-09-08. Scope: implementation plans, not new production code or new
performance evidence. The [primary plan](S07-bis-performance-plan.md) is selected;
the [exclusive-binding alternative](S07-bis-exclusive-binding-alternative.md)
remains on hold.

## Inputs and review questions

The author re-read the current Rust guide, S07/ADR ownership and symbol contracts,
the S07 implementation and binding operation records, CPU/memory reports, actual
storage/generator interfaces, and the benchmark producers/statistics/consumers.
Existing local edits outside these plan documents are not part of the change.

Two independent review passes covered separate areas:

- Storage/API/ownership: whether compact fields and payloads can preserve current
  read, mutation, identity, mapped-source, lazy and failure contracts; whether the
  sequencing supplies the indexes required by an earlier checkpoint.
- Measurement/gates: exact thresholds and stopping rules, replacement accounting,
  control/candidate provenance, evidence dependencies and final acceptance.

The reviews found no unresolved contract or measurement blocker in the selected
plan. Architectural selection remains the author's decision. The reviews did
not establish that provisional layout budgets or eventual gate results are
achievable; checkpoint 0 must test that.

## Findings and amendments

| Finding | Amendment |
| --- | --- |
| "Per-kind" could incorrectly imply selecting physical payloads from syntax kind | Require a separate concrete payload-shape tag; retain open kind/data pairings, with TokenData + identifier/unknown-kind counterexamples |
| Typed binding columns need a locator before typed AST pages exist | Require an explicitly selected provisional index, charged construction/scan timing, source partition and lazy fallback, and a concrete reuse/removal decision in checkpoint 3 |
| A 24-byte header silently depends on compressed parent storage | Model a full-parent candidate first; state the compact-parent/escape and separate-shape requirements for the smaller target |
| Existing build helpers overwrite shared executable destinations | Require immutable per-variant normal/allocation artifacts with before/after hashes and negative overwritten/swapped-binary tests |
| Optional allocator replacement contradicted the fixed final producer/validator policy | Keep mimalloc selected; any alternative requires a separate ADR/configuration/producer/validator/accounting/calibration amendment with unchanged gates |
| The held alternative inherited an incompatible parse→publish→bind ordering | Explicitly map parse→bind→publish into the same combined interval; preserve all work, roots and excluded setup boundaries, and update diagnostic phase provenance |
| Arena-only edits were said to invalidate generator evidence | Distinguish AST/generator edits from arena-only edits; renew evidence from actual narrowed input closures |
| Screening's 5% improvement / 2% regression wording was statistically ambiguous | Define median thresholds, timing bootstrap upper bounds, both-variant MAD limits, RSS/allocation uncertainty treatment and neutral-infrastructure exceptions numerically |

Author review also added per-shape empty-directory costs, rejected all-node
index duplication, distinguished source-backed from cooked/synthetic text,
and kept full public ID range/foreign escapes explicit. Source census/live
bytes, cumulative requests and OS peak RSS remain separate budgets.

## Remaining decisions

No reviewed correctness or gate requirement is intentionally deferred. Physical
representation choices remain experimental: direct columns versus packed sparse
pages, the provisional locator, compact edge/text encodings, runtime-ID storage,
and page/capacity policy. Each has an assigned checkpoint and rejection rule.

The 1.70 GB live-storage, 1.90 GB allocation, 2.10 GB RSS and 0.90 CPU-ratio
working targets are engineering headroom objectives, not changed sprint gates
or forecasts. The plan cannot guarantee success before implementation and
measurement. A missed budget triggers a named design decision, not optimistic
addition of independently estimated savings.

Documentation validation checks local links/anchors, arithmetic, exact gate
definitions, whitespace and existing tracker/provenance consistency. No compiler
tests, new profiles or acceptance captures are claimed for this planning change.

Validation passed: 29 local links/anchors across the four affected documents,
the 1,700 MB budget sum, whitespace checks, `cargo xtask validate` and
`cargo xtask status --check-committed`. Both reviewers verified the amendments
in their respective scopes. The planning files remain uncommitted on the new
`codex/s07-bis` branch, as requested; S07 retains its existing commit history.
