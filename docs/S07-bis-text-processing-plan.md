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
