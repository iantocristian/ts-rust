# S07-bis plan review

Date: 2026-09-08. Scope: implementation plans, not new production code or new
performance evidence. The [revised primary plan](S07-bis-performance-plan.md)
now selects the early single-source exclusive prototype. The
[exclusive-binding record](S07-bis-exclusive-binding-alternative.md) activates
that bounded path and keeps a general published-file rewrite on hold.

Sections 1–3 below record the initial review of the plan frozen by the user in
`4173b89`. The subsequent Claude review and new decision are recorded afterward;
the initial reviews are not presented as approval of changes made later.

## 1. Initial inputs and review questions

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

## 2. Initial findings and amendments

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

## 3. Initial remaining decisions and validation

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
in their respective scopes. Initial delivery was uncommitted on `codex/s07-bis`;
the user subsequently committed the reference plans in `4173b89`. That commit
and the S07 history remain unchanged.

## 4. Claude's second review: decision and disposition

The author accepts the sequencing criticism and changes the selected path to
**CP0 → single-source A0 → immediate architecture decision → shared layout work**.
CP2's replacement columns/locator/patch store is now on hold as the fallback.
The initial plan unnecessarily made a general publication protocol replacement
the prerequisite for trying the ordinary exclusive cache path. The original
mapped-sibling finding required independent binding, not overlays for every file.

Two new independent audits checked actual production callers, exclusive/lazy
storage and binder borrowing; and separately checked request/live accounting,
header arithmetic and phase milestone definitions. Their code findings support
trying A0 first. They do not establish its implementation cost or eventual gain.

| Finding | Disposition in the revised plan |
| --- | --- |
| Cache and benchmark bind before exposing a result | Accepted; both use one proposed consuming production entry, tested before CP2 |
| A once-cell swap describes the whole change | Qualified: FileCache is an exclusively borrowed weak cache of completed entries, not a pending once cell. Preserve retry/no-entry-on-failure behavior |
| Existing published readers/mapped inputs prevent the exclusive pilot | Rejected as a universal obstacle; keep those inputs on the existing path with eligibility decided before mutation |
| Retaining today's binder is not a second full implementation | Accepted; require one shared binder algorithm with two storage backends, while measuring dispatch/code-size and testing both |
| The binder can immediately use mutable construction | New concrete issue: parsed node/list borrows currently span recursive mutable calls. A0 must prove traversal without per-node/list copies or a duplicate graph |
| Lazy transaction mutation supplies committed lazy access | Corrected: it handles pending nodes only. Implement checked owner-exclusive committed-page access or explicitly select fallback before mutation |
| No pre-bind read escapes, therefore view semantics need no changes | Qualified: BoundFile::parsed_file still exposes parsed state after success. Specify a truthful new bound-result capability and retain old published-file guarantees |
| A0 removes the large binding-field maps | Not yet: initial A0 retains them and isolates node-overlay/access costs; compact inline payload fields remove them in CP3 |
| Temporary request traffic needs a CP0 budget | Accepted; model the full native counter identity and adopt a provisional 350 MB traffic ceiling with charged list/buffer/map/conversion work |
| 1.70 GB live inside 1.90 GB allocation leaves exactly 200 MB for traffic | Corrected: pipeline-start live is excluded from allocation. At the observed 166.647 MB starting live, the allowance is 366.647 MB; remeasure for every candidate |
| The 24-byte compact-parent header should lead | Accepted; retain the 32-byte header as a control/fallback only when its complete weighted budget fits |
| Per-phase CPU continuation exits are needed early | Accepted: targeted A0 improvement, near-1 s bind and near-2 s parse milestones at the relevant storage steps, with explicit immediate review on a miss |
| 1 s bind + 2 s parse predicts passing | Corrected: 3 s already exceeds the old 2.938 s pipeline median gate before orchestration; full-wall headroom and the unchanged statistical gate remain required |

The phase/corpus adapters also need explicit dual-path coverage. The current
publish-first corpus adapter would otherwise continue proving only fallback
behavior while cache/benchmark used the new entry. Exclusive measurements map
parse→bind→publish into the same combined interval and retain all original work.

The revised native endpoint budget includes retained driver/queue allocations
and every remaining requested-live byte. Syntax and inline-binding storage share
one physical allocation budget; the 780/100 MB sub-budgets are not counted twice.
Request traffic is planned concurrently with layout; known list/growth costs are
implemented with CP3/CP4 rather than deferred automatically to CP6.

Both new reviewers verified the amended plans in their respective scopes.
Final clarifications restrict disjoint global phase-counter attribution to one
worker and distinguish syntax payload bytes from inline binding bytes. Eight
workers require whole-pipeline reconciliation unless separate phase attribution
is validated. Replacement projections deduplicate old overlay copies.

Revised-document validation passes 34 local links/anchors, both budget sums and
the request/live equation, whitespace checks, `cargo xtask validate` and
`cargo xtask status --check-committed`. No implementation, benchmark run, new
acceptance result or history rewrite is claimed by these amendments.
