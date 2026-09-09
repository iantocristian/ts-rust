# S07-bis: shared keyword and text-slice CPU candidate

Selected after the `03f9aa3` phase/live-request diagnostic and one native sample
of its exact normal executable. Implement these two changes together on the
current experimental compact backend, then take one fixed full-pipeline screen.
CP1 remains the retained control; no memory or CPU gate is waived.

## Evidence and scope

The current normal diagnostic spends 2.749 s parsing and 2.742 s consuming
binding/publication. The separate native sample has 5,371 ms of worker weight:
2,718 parsing, 2,640 binding and 13 unassigned. Visible keyword-search ancestry
accounts for 254 ms of `memcmp` sample weight (172 parse, 82 bind). Slice
revalidation accounts for 144 ms of UTF-8-validation weight (93 parse, 51 bind).
These 398 ms are distinct sampled work, not forecast removable elapsed time.

The source confirms two concrete differences from cheaper equivalent operations:
`keyword()` performs multiple lexicographic comparisons through the pinned
keyword table; Go uses its `textToKeyword` map. `JsString::slice` reclassifies
every selected byte range, even when the existing parent validity and endpoints
already prove UTF-8. In binding, the latter path includes source-backed text
retained for declaration names; in parsing it includes identifiers and literals.
Do not count raw-suffix matching, unrelated string validation or table hashing as
savings from this change.

Keep the row policy, AST facade, binder algorithms, source-file observations,
allocator and dependencies unchanged. The smaller generated scalar-projection
idea is deferred: `NodeRead::data` alone has only 124 ms of current weight and
does not justify another broad accessor migration. No component microbenchmark,
new tracing protocol, perfect-hash search or variant matrix is required here.

## Implementation

1. Extend `scripts/s05_tables.py` to emit a deterministic keyword byte-match
   function from the same validated `scanner.keywords` map. Retain the sorted
   `KEYWORDS` data for suggestions and independent semantic checks. Generate
   Rust byte literals with correct escaping for every UTF-8 byte, including
   non-ASCII, quote and backslash values; do not silently narrow the generator's
   accepted input domain. Return the same kind/unknown result as the existing
   function. Keep `get_identifier_token`'s length/lowercase filters and
   `identifier_to_keyword_kind`'s unfiltered behavior unchanged. `TOKENS`, Unicode
   tables, regex-property tables and suggestion ordering are unaffected.
2. Regenerate through `python3 scripts/s05.py tables --write-manifest` using the
   pinned local Go toolchain, then run its ordinary drift check. The authority
   remains the clean pinned Go table export and scanner behavior. The manifest
   stays at `data/s05/tables-manifest.json`; this does not widen S03 generation.
3. In `JsString::slice`, perform the existing byte-range bounds check first.
   If the parent tag is `Utf8` and both relative endpoints are UTF-8 boundaries,
   derive `Utf8` directly. End-of-view is a boundary; any in-view byte that is
   not a continuation byte is a boundary in a validated UTF-8 parent. Empty
   slices must still be UTF-8, including empty ranges inside a multibyte scalar.
   All other cases use the existing classifier. Keep byte offsets, Arc sharing,
   arbitrary byte-cut support, nested view offsets and `None` for invalid ranges.
   Never inherit `Wtf8` or `Raw`: their subslices can have different validity.
   `as_str` remains safe and unchanged; introduce no unchecked UTF-8 access.

## Review and validation

Review the concrete plan before editing Rust. Split implementation across the
scanner generator/consumer and string slice so each receives independent source
review. The UTF-8 proof relies on a private invariant already enforced by every
constructor; audit every constructor and all current slice-tag tests. Check
keyword lookups against the retained table for every key and adversarial
near-misses, including prefixes, suffixes, modified bytes, case changes and
non-UTF-8 input. Preserve filtered versus unfiltered lookup contracts.

Run the existing string tests and add only missing nested-view/invalid-range
counterexamples; existing exhaustive cuts across UTF-8 and surrogate input
already cover many cases. Run scanner tests and full E4/scanner parity producers,
then affected parser/binder/compiler tests, workspace Clippy, Rust 1.96 checking
and formatting. No new unsafe code or ownership mechanism is introduced; scoped
instrumentation should target the changed slice/caller cases rather than the
unrelated deep native stack-growth stress test. Keep all failed attempts/logs.

After source review and correctness checks, freeze the combined candidate,
verify full workload graphs at one/eight workers, and execute the established
eight-warmup/56-sample paired screen once. Record raw data, confidence bounds,
same-screen differences and all historical gate distances. Do not rerun or tune
a variant to rescue a weak result. A small, reviewed win may be retained under
the clarified retention rule, but it ends investment in this direction unless
new evidence identifies a separate material cost. No forecast claims gate
closure: the current combination still needs about 209 MB less allocation,
105–110 MB less RSS and substantial CPU improvement.

## Implementation and correctness record

The two changes are implemented and independently reviewed with no findings.
The generated byte match contains the same 85 keyword entries; removing that
function leaves the previous generated table bytes unchanged. The table manifest
changes only that output hash. Byte-safe Rust string and byte-literal emitters
also compile and run a synthetic Unicode/quote/backslash/control fixture; the
existing pinned table and fold outputs are unaffected. Keyword tests cover every
entry, cuts/suffixes and all single-byte replacements, plus filtered/unfiltered
helpers and suggestion ordering.

The slice implementation checks the relative byte range before indexing either
boundary. Its private valid-parent proof does not inspect the whole parent;
fallback classification handles empty interior cuts, malformed parents and
surrogate fragments. Nested views retain the same Arc bytes and are checked
against fresh construction, including a valid UTF-8 child of raw backing and
reversed/oversized ranges.

Validation passes 128 tests across string/scanner/parser/binder/compiler libraries
and integrations, the 27-test S05 Python suite, pinned table regeneration/drift,
workspace Clippy, Rust 1.96 compilation and formatting. Full E4 passes 76,001
probes; full scanner parity passes 33,332 cases and 4,982,432 observations with
zero failures. The E4 slice subset checks Go bytes/bounds/string-view behavior;
the three-way Rust-only validity tag is covered by local classification tests,
not presented as an upstream Go classifier. Strict-provenance Miri passes three
slice tests and AddressSanitizer passes all 12 string core-helper tests. Initial
Go-cache sandbox failures and successful authorized retries remain recorded.
These scoped instrumentation runs do not claim the complete ownership producer.

The corrected freeze and its fixed paired screen are now complete; the result
below supersedes the pending status, not any earlier experiment verdict.

## Build provenance correction before timing

The first freeze for `aa6a5f6` is invalid: Cargo returned the exact normal and
allocation binaries from the preceding `03f9aa3` phase/live diagnostic. Its
reported manifest pointed at the working checkout, while the executable's
retained dep-info named the diagnostic source tree. The diagnostic had shared
`CARGO_TARGET_DIR` with the workspace. Snapshotting current source and validating
Cargo's selected artifact were insufficient to detect that reused output.

No performance samples were collected from this freeze. Its bundle, logs and
interrupted graph attempt are retained as an invalid build, not correctness or
performance evidence for the keyword/slice implementation. Benchmark executable
builds now use a fresh temporary target directory, overriding configured or
environment output/intermediate paths while preserving registry caches. Both
`--target-dir` and `build.build-dir` point at the empty directory. The executable must
resolve inside that directory, is copied before cleanup, and receives the same
native profile checks. A real Cargo regression fixture checks the isolation.
The corrected freeze uses a new output directory and repeats full graphs before
the single fixed timing schedule; this is a build-defect retry, not extra timing
samples selected for a favorable result.

## Combined screen outcome

Implementation `aa6a5f6`, frozen with build-isolation fix `8f7236e`, passes all
13,094 workload graphs at one and eight workers. All files bind in place with
zero fallback; work counts and input digest match the retained CP1 control.
The corrected manifest is
`bab5c54fc779b86ebdb1f2bd5a2289dcd7a4066612eb004ceb02e16e1c906fdc`.
Eight warmups and all 56 planned samples completed without a timing retry.

| Metric | CP1 one worker | Candidate one worker | CP1 eight workers | Candidate eight workers |
| --- | ---: | ---: | ---: | ---: |
| Wall | 5.078450 s | 5.180391 s | 1.087606 s | 1.079982 s |
| Allocated bytes | 4.642568 GB | 2.244183 GB | 4.642573 GB | 2.244184 GB |
| Peak RSS | 4.506599 GB | 2.318565 GB | 4.509237 GB | 2.321842 GB |

CPU candidate/control ratios are 1.020073 and 0.992991. Their 95% intervals are
[0.973742, 1.055861] and [0.951295, 1.006322]. CPU relative MAD is 1.604%/3.370%
(candidate/control) at one worker and 1.056%/0.405% at eight. The single-worker
upper bound does not satisfy the 1.02 nonregression screen. Neither interval
establishes a CPU win. Memory medians remain about 51.7% lower for allocation
and 48.5% lower for RSS than this batch's control.

The complete combination remains experimental. This result does not establish
an isolated keyword/slice saving: the preceding parser-list batch had a
different paired control and host conditions. No separate medians are added or
subtracted to claim a component effect, and no extra batch is run to rescue the
verdict. Keep the reviewed coherent candidate as the ongoing experiment; end
keyword/slice tuning here. All final gates remain open. Against historical
Go-derived limits, one/eight-worker wall still needs 2.243/0.446 s improvement,
allocation 208.956/208.417 MB, and RSS 109.617/104.797 MB. Those are planning
distances, not fresh Go-relative acceptance measurements.

The [complete review archive](../tools/s07/performance-experiments/results/2026-09-09-compact-text-processing/README.md)
retains both build attempts, the interrupted invalid graphs, the corrected
frozen source/binaries, all graph and sample data, receipt and correctness logs.
The initial stale-artifact bundle is explicitly invalid and is not used by the
complete screen. Prior frozen observations keep their original verdicts.
