# S07-bis: larger typed pages, compact text entries and remaining binding helpers

Status: selected combined experiment; implementation and measurement pending.
The user approved this sequence after the [complete local binder control](S07-bis-local-text-repair.md).

## Baseline before implementation

Restore the six moved binder source mappings before capturing evidence. Commit
`be74975` contains only those comments. Preserve previous captures and immutable
controls. Run the standard `bindworkload` producer, the paired Go/Rust benchmark
capture, and E5/E6 consumers on this revision while source is fixed. Use the
producer's quiet-host checks and declared stopping policy. Freeze the validated
executables, sources, inputs and raw observations as the next comparison control.
The preceding Rust/Rust improvements remain valid; this refresh replaces the
historical Go denominator for current gate distances.

## Combined implementation

1. Increase typed payload pages from four to sixteen rows through one named
   page-width constant. Keep stable boxed rows, checked ordinals, safe bounds
   access and the empty/single/multiple directory representation. Charge default
   construction and retained tail rows as well as fewer page/directory requests.
   The earlier 6,452,840 page calls give a naive 4,839,630-call reduction ceiling,
   not a prediction: each file/shape store rounds independently.
2. Replace 32-byte optional text-pool entries with eight-byte tagged entries.
   Recoverable source ranges retain the owner's source; exceptions retain their
   original `JsString` in separately charged storage. Preserve explicit owned
   conversion by source slicing or shared-string cloning, never by reconstructing
   owned strings from borrowed bytes. Preserve arbitrary bytes/validity, empty
   values, reuse, release, mutation of node ranges, synthetic/escaped text and
   extended field-key entries when compact encoding cannot represent a value.
   From the older traffic capture, slot shrinkage alone gives gross ceilings of
   91,176,672 requested and 46,214,784 retained bytes. Replacement storage and
   page-tail growth must be subtracted; no gate closure is assumed.
3. Migrate the remaining declaration-name, dynamic-name, ambient-module,
   modifier and strict-mode predicates used by local binding. Share algorithm
   rules with checked access; keep local node/list identities through ordinary
   reads. Declaration names include assigned JavaScript and anonymous-expression
   names, not only the node's name field. Modifier queries observe stored list
   flags and combined ancestor flags. Preserve short-circuit and malformed-shape
   failure order, nil behavior and checked lazy/foreign fallback. Keep diagnostic
   construction and unrelated module/expando operations outside this migration.

The earlier native profile establishes remaining checked routes, not their
current cost. Already-local basic getters, parameter traversal and contextual
identifier facts are not additional work or additional savings. This experiment
does not include another access trace, page matrix, profile or attribution tool.

## Correctness and ownership review

Exercise page transitions, out-of-range access, stable addresses through growth
and correct destruction. Text regressions must cover backing identity and values
that survive entry replacement and owner drop, including exceptional and malformed
bytes. Extend real ownership inventories if new cases are added; do not let a
substring filter or stale denominator silently omit them. Use paired checked/local
fixtures for shared helper rules and preserve the independently pinned Go corpus.

After the complete implementation, run affected debug/release and documentation
tests, lint, minimum Rust, generator drift and formatting. Review the ownership
and malformed-input boundaries independently; renew the required Miri/ASan
evidence on the final source. Run full graph parity for all 13,094 files at both
worker counts and confirm the selected binding paths before timing.

## Measurement and decision

Build and freeze the complete candidate once source/configuration stops changing.
Screen it against the refreshed immutable control with the existing fixed eight
warmups and 56 observations. Keep every result; no component screen decides
promotion and no independently estimated savings are added. Report absolute
time/request/RSS changes, confidence/noise qualifications and all Go gate distances.

Retain comparable consuming parse and bind/publication/validation elapsed
observations for control and candidate using the separate existing phase driver.
These explain the combined result; they do not modify the ordinary acceptance
executables or establish sampled CPU costs. Do not run the obsolete published
versus consuming comparison as a substitute for this attribution.

Apply the accepted combined-tradeoff and retention policy to the complete result.
Preserve failures and regressions, archive/replay the evidence and update the
tracker truthfully. A failed final gate remains failed regardless of a useful
experimental improvement. Stop this bounded experiment at its measured decision;
any subsequent direction requires an explicit plan based on that result.
