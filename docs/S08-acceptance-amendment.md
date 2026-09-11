# S07-3 amendment: native baseline eligibility in S08

Status: **accepted by the repository owner, 2026-09-11**, and implemented on the
S08 PR. Amendment ID: `S07-3-2026-09-11-native-baseline-authority`.
This resolves [the P0 authority review](S08-P0-baseline-review.md). It supersedes
that review's proposal to make projected/skipped variants ordinary required cases.

## Decision and scope

Correct duplicate-file extraction to native root-before-auxiliary ordering.
Retain native option-policy skips, rejected-option variants and filename skips
as **informational Go-versus-Rust differentials without an E2 gate**. Their
actual results may be collected beyond test-selection guards. Failure, mismatch
or unavailable output in that group neither passes nor fails E2 acceptance.
There is no obligation to implement unsupported behavior just for that group.

Keep the complete S06/S07 source inventory, parser/binder/loader coverage and
E7/E8 selection. The source inventory is not the checker acceptance denominator.
No performance threshold or baseline-divergence allow-list was changed.

| Classification | Variants |
| --- | ---: |
| Original source-selected inventory, preserved | 10,728 |
| Required E2 acceptance | 9,369 |
| Informational only | 1,359 |

The informational set is a **union**, not a sum of reason counts. The original
Go option guard skips 1,324 effective configurations; 34 configurations have
recorded rejected options; 44 appear in the original filename skip list. Nine
of the rejected-option configurations also match the option guard, explaining
why its count is larger than the earlier 1,315 post-compilation skips. All 34
rejections are filename skips, leaving ten additional filename-only cases.

## Executable rule at the pin

Upstream remains `1f70213d4922b434345f639b441681e470c7cfc1`.
[`e2-acceptance.json`](../data/s07/e2-acceptance.json) records every variant's tier
and all matching reasons. [`e2-policy-observations.json`](../data/s07/e2-policy-observations.json)
contains a compiled Go observation for every ID, ordered exactly like the source
inventory. Both are bound into the S07-3 review and frozen rule.

The classifier uses these predicates, independently of Rust or baseline success:

- Execute original `harnessutil.SkipUnsupportedCompilerOptions` on the frozen
  effective options. Its skip predicates at this pin are module UMD/System;
  Node10/Classic module resolution; explicit `esModuleInterop=false` or
  `allowSyntheticDefaultImports=false`; nonempty `baseUrl`; ES5 target; or
  explicit `alwaysStrict=false`. The original function calls its fatal AMD and
  `outFile` guards first. An unexpected fatal/panic aborts policy observation and
  requires review; it does not invent another permitted exclusion.
- Read original `testrunner.skippedTests` and test the physical source basename.
- Require a frozen `option_outcome=rejected` with nonempty original Go option
  diagnostics for the rejected-option predicate.

The manifest hashes both native source files, the bridge and classifier sources,
and the exact option requests and observations. Regeneration must reproduce the
full partition. Missing/duplicate/reordered observations, truthy non-booleans,
unknown outcomes, stale source or altered classifications fail validation.
The frozen policy contains portable observations only; the full native report
and archive retain the actual Go version, OS and architecture as provenance.
`select_acceptance` requires every acceptance outcome in order; informational
rows may be absent or contain failures. They cannot substitute for an acceptance
row. The S08 phase manifest carries the tier on every request, with separate
acceptance counts: **1,270 declaration-diagnostic obligations** and **9,171
requested type/symbol baselines**. Required checker workload/census manifests
will use this acceptance partition; informational diagnostics remain separate.

## Duplicate-file correction

S06 and S07 now share the native fixture partition used before virtual-FS
insertion. Roots are inserted first, auxiliary units second. A regression test
compares the exported bytes with the actual native `newCompilerTest` program,
including the empty duplicate root and the absence of its formerly imported file.
Physical bytes and extracted unit definitions remain intact, including the
populated duplicate. Only the virtual-FS insertion order changes; baseline
formatting must still retain the native distinction between raw test units and
the source bytes the program actually loaded.

Complete regenerated-corpus comparison changes only
`compiler/augmentExportEquals2.ts`: two S06 parser requests now contain the empty
`file3.ts` Go actually loads. S06 still has 12,829 primary rows and 22,343 parser
requests. All S07 IDs/dispositions are identical. The ordered loader census
changes from 290,799 to 290,797 files and distinct observations from 11,260 to
11,259. Library identities remain 111 and source-derived checker obligations 675.
File-table indices are compared through their referenced identities, so index
renumbering cannot conceal a changed program.

## Actual baseline collection after the fix

A fresh full native capture, with informational collection enabled, records:

| Tier | Completed raw baselines | Failed option parsing |
| --- | ---: | ---: |
| E2 acceptance | 9,369 | 0 |
| Informational | 1,325 | 34 |

All completed rows match their frozen options and loaded source identities;
there are no input mismatches. All 10,694 complete pre/post diagnostic payloads
match. Acceptance type/symbol outcomes each comprise 9,167 content, four
`NoContent` and 198 explicitly disabled baselines. Acceptance error outcomes are
5,280 content and 4,089 `NoContent`.

The 34 rejected configurations keep their original settings and named native
failure. They are not reinterpreted under projected options to manufacture a
semantic result. The additional 1,325 informational executions have actual raw
Go baselines and query traces; **no Rust differential exists yet**, since the
semantic checker remains to be implemented. These observations are not E2 parity.

A second full capture reproduces every raw baseline, diagnostic payload and
query operation/position/flag in order. Twelve query records in four acceptance
cases have different native numeric `type_id` values. Both captures are retained.
Those allocation identities are diagnostic metadata, not expected cross-run
type identities; P0 must freeze query actions and compare identity relationships
within an owner instead of treating the raw numbers as semantic output.

## Reproduction and evidence

```sh
python3 scripts/s07_acceptance.py check --output target/s08/policy-review
python3 scripts/s08.py check --output target/s08/phase-review
python3 scripts/s08_baselines.py --include-informational --output target/s08/baselines-review
```

Policy/phase commands require new output directories. The reviewed source-input
regeneration follows the existing S06 freeze, S07 source/loader observation and
S07 reviewed-freeze commands. Ordinary correctness producers verify the frozen
artifacts without rewriting them.

The shared extraction change also refreshes the direct-config provenance
manifest. Its request and observation bytes are unchanged; the first program
verification stopped on the old source hashes, which are retained in the archive.

Final local verification passes parser and binder parity over all 22,343 parser
requests; loading and option diagnostics over all 10,728 source variants; 25
program helper tests; the source-only E2 freeze; and 343 Python tests plus tracker
tests. The immutable records are identified in the archive directory's
`validation.json`. These checks do not certify the pending Rust semantic checker
or replace the four-target CI run.

Raw baseline capture, native policy/phase observations, semantic source-diff
reports and validation logs are under
[`tools/s08/results/acceptance-amendment`](../tools/s08/results/acceptance-amendment/).
The previous conflicting capture remains archived as history. P0 still needs the
typed closure and remaining query/fixture/workload freezes; P1 and S08 are not
complete. No result is inferred from a missing baseline or stale evidence.
