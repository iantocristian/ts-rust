# S04 synthesis implementation plan

Branch: `codex/s04-synthesis`, based on main at `b8313e7`.

This is a fresh integration of the accepted S04 contracts, informed by the four
implementations and both reports in `~/git/ts-rust-s04-report`. Existing branches
remain references. The current main's license, CI target matrix and tracking
infrastructure remain the baseline. No old execution evidence will be copied.

## Selection and rationale

| Source | Keep | Correct or avoid |
| --- | --- | --- |
| Fable 5.1 | Small modules grouped by responsibility/upstream file; invariant scoped-access brands; borrowed core lookup; typed errors; stable `OnceLock` pages | Unsigned positions; partially initialized slots reused after failed lazy transactions; public test counters that can mint duplicate identities; missing CI integration |
| Astra | Signed Go arithmetic and edge behavior; immutable builder-to-file publication; bundle-retaining handles; transactional cyclic lazy graphs; borrowed/Cow string fast paths; pinned Unicode generation and differential oracle | Broad owner/string/position modules; retained Arc cloning on every core lookup; decimal character literals in escapes; parking_lot's Miri provenance limitation |
| Opus | Explicit token key/kind/reparsed-parent validation; source text and position map owned with the file; fallible cache checks outside panic paths; independent CI metric gates | Duplicated read/write cache checks; partial graph publication; claiming bind completion before initialization; Unicode 15.1 simple-lower divergence; premature checker implementation |
| Sol | Descriptive local names and straightforward decoder/escape control flow; standard-library-only ownership dependencies | Monolithic modules, per-node Arc allocation and locked core reads; hardcoded harness outcomes; absent Go oracle |

The design notes, not a majority vote between implementations, decide behavior.
In particular API, LSP and scanner conversion remain distinct; a failed lazy
transaction must never make a previously exposed id refer to a different node.

## Implementation sequence

1. **Text leaf.** Separate `jsstring`, `source_text`, `wtf8`, `helpers`, `escape`,
   `position_map`, `line_map`, `lsp`, and `scanner_positions`. Retain byte-sharing
   slices and reclassify their exact bytes. Preserve signed `int`/`int32` range,
   wrapping, rounding, clamping and panic behavior from Go. Use pinned upstream
   JavaScript casing tables and a generated, pinned Go simple-lower table for
   `LowerFirstChar`. Return slices/Cow where unchanged output can be borrowed.
   Use named constants or character literals where they clarify byte operations.
2. **Ownership leaf.** Separate ids, counters, storage, file/bundle ownership,
   lazy publication, scopes and leases. Consume mutable builders when publishing
   immutable files. Keep process-global nonzero arena identities and checked slot
   limits; injected counters belong only to tests. Borrow core nodes for scoped
   reads and provide an explicit retained handle for escaped references. Add the
   invariant HRTB brand for repeated validated core access, with compile-fail
   checks for cross-arena use and escape.
3. **Lazy publication and bundles.** Use one standard-library `RwLock` for caches,
   directory, reservations and publication. Construct whole graphs in private
   staging, publish complete slots/cache entries together, and burn ids from
   failed attempts. Validate token parents through retained storage, including
   kind and reparsed checks. Keep node addresses stable on `OnceLock` pages and
   keep the file/bundle alive for escaped references. Test returned errors and
   initializer panics, subsequent retry, retained readers and page growth.
4. **Evidence pipeline.** Adapt Astra's clean exported-pin Go oracle and frozen
   scenarios, including private helper bridges and strict ordered/type-sensitive
   result comparison. Preserve exact-key/coverage checks from the strongest
   comparators. Keep Go, Unicode generation, MSRV and nightly inputs pinned.
   Derive metrics from executed tests and measured final-drop counters; run the
   same arena scenarios in debug, release, Miri and ASan. Exercise producer
   rejection paths so malformed or incomplete evidence cannot pass.
5. **Integration.** Add workspace members, accurate PORTS mappings, crate docs,
   README/PLAN updates and CI producer/metric gates without reverting current
   main changes. Regenerate tracking views only from new executions. Review
   the combined API and failure paths before recording S04 completion.

## Acceptance checks

- Text unit/regression tests and every frozen differential probe agree with the
  pinned Go functions. Include malformed BOM payloads, every byte-slice boundary,
  lone sentinels, Final_Sigma context, Garay simple lowercasing, negative/extreme
  positions and the emoji API/LSP/scanner results 1/0/4.
- Arena scenarios establish wrong-owner/stale/never-reused identities in release,
  exhaustion before wrapping, invariant brands, immutable publication, cache
  miss recheck, failed-transaction isolation, stable page references and complete
  bundle retention in all release orders.
- Miri uses strict provenance with the production standard-library lock; ASan
  uses a rebuilt instrumented standard library. Missing tooling or skipped
  scenarios cannot produce passing instrumentation metrics.
- Counters record owner objects and explicit storage units, including pages and
  nonempty slabs. They are reclamation checks, not heap-byte or RSS claims.
- Workspace tests, formatting, Clippy with warnings denied, dependency policy,
  tracker self-tests, oracle smoke, MSRV build, fresh producers and
  `cargo xtask check S04` pass. Committed views reproduce from captured evidence.

## Scope limits

S04 implements contract leaves. It does not establish scanner/parser/checker or
printer integration, semantic checker retirement, builder dependency retention,
compiler throughput, heap-byte targets or full E3/E4 completion. Tests and docs
must preserve that distinction. Generic payloads provide room for generated AST
types without inventing their future layout or a parallel toy compiler.

## Progress

- [x] Inspect reports and four implementations; establish the synthesis design.
- [x] Create the new branch and record this plan.
- [x] Implement modular text and ownership leaves.
- [x] Integrate and strengthen oracle, instrumentation and CI.
- [x] Review APIs and regressions; run all acceptance checks.
- [x] Capture fresh evidence, regenerate views and document actual results.

Completed on 7 September 2026: all 14 S04 exit checks and 7 required items pass
on `aarch64-apple-darwin`. The differential oracle matches 76,001 probes in 401
scenarios. Strict-provenance Miri, ASan, brand doctests, workspace tests, Clippy,
dependency policy, tracker validation and the Rust 1.96 build pass. See
[S04 results and reproduction](S04.md) for evidence links and scope limits.
