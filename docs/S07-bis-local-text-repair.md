# S07-bis: preserve shared identifier text in local binding

Status: implemented; affected checks pass. Full ownership validation and a new
combined screen are pending.
This follows the [completed local binder screen](S07-bis-local-bind-completion.md).
That version remains recorded without promotion: its full binder producer passed,
but its E3 capture rejected stale test inventories. CP1 remains the control.

## Mechanism and scope

The local Identifier branch of `Binder::target_text` converted borrowed bytes
with `JsString::from_bytes`. Unlike the checked text operation, this allocated a
new backing. Ordinary declaration-name processing can then discard that copy
after interning or table lookup. This is a confirmed extra allocation mechanism,
not an attribution of the entire approximately 58 MB request increase observed
between independently measured compact candidates.

Generate explicit owned accessors beside the borrowed text accessors. They pass
the same field key, encoded word and node end through `BindContext` and
`CoreStore` to the existing `TextPool::owned`. Raw identifiers share a source
slice; exceptional and extended-pool values clone their existing backing. The
binder uses this accessor only after its existing Identifier kind and shape
checks. Other kinds, malformed payloads and lazy nodes retain their checked
operation. No new interning, table, storage or lifetime policy is introduced.

Tests cover source and pooled pointer identity, bytes and validity, survival
after owner drop, lazy fallback, exact mismatch panic messages, extended-pool
entries and generated-name collisions in either schema-field order. The real
generator supplies the output and its manifest hash.

## Ownership inventory correction

The failed capture executed passing tests but could not prove its declared
denominators. S06 now names all 29 cases in `storage_tests`. S07 separately names
local AST, local core state, binder, flow and symbol cases, alongside publication,
exclusive binding, completion proof and program ownership: 76 cases in total.

Cargo's substring filter `bind_tests::` also selected `local_bind_tests::`.
The publication suite now declares that exact module skip; local cases execute
in their own group. The manifest schema validates the permitted skip and rejects
skipping a declared case. Missing, extra, duplicate or failed outcomes still
invalidate the result. The expanded groups must pass in every required mode.
This correction does not turn the earlier failed evidence into passing evidence.

The 39 Python checks pass. One native `--no-run` build and exact test listings
match all 11 selections (29 S06 / 76 S07). Listing is not test execution. The
initial listing wrapper's overly narrow artifact-path assertion is retained;
the corrected verifier consumed the same successful Cargo artifacts without
rebuilding or executing suites.

## Validation and measurement sequence

1. Finish affected native and release suites, documentation tests, generator
   tests and drift check, Clippy, minimum Rust and formatting. Preserve source
   inventories and failures. Renew E3 with the corrected exact inventories.
2. Freeze new normal and allocation executables from the repaired source. Keep
   the previous completion freeze unchanged. Compare full Rust binding graphs
   at both worker counts with the immutable, verified Go streams; record reused
   Go provenance explicitly. Confirm local path selection on the new executable.
3. Run one complete fixed screen against CP1: eight warmups and 56 samples.
   Judge the combination on CPU, requested allocation and RSS at both worker
   counts. Per-component estimates cannot qualify it. Recompute historical Go
   distances without presenting them as fresh gate evidence.
4. Archive and replay the new observations, distinguish validation failure from
   measured regression, and record whether the combined candidate can become
   the next control. S07-bis gates remain open until their own evidence passes.

No new native profile or field-level trace is required for this repair. The
preceding completion's single phase observation remains bound to that version;
it is not a phase measurement of this revision.

## Completed checks

Affected debug and release checks pass: 30 arena tests, 158 AST unit tests,
29 AST integration tests, 47 binder tests, 64 generator/tooling tests and
37 documentation tests in each build mode. Clippy with warnings denied,
Rust 1.96 checks, all 22 generated outputs and pinned client bytes, `xtask
validate`, and formatting pass. Source inventories are unchanged across the
final checks. Initial missing-import, test-source-filename and formatting
failures remain recorded separately from that successful final run.

An independent read-only review found no actionable defect in ownership,
validity, checked fallback errors, generated field keys or name collisions.
It checked the actual generated output against its manifest. The patch author's
separate follow-up review is recorded as self-review, not independent evidence.
