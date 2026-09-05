# ADR 0019: Test-host protocol and transport contract tests

Status: Proposed (2026-09-05)
Plan: section 8 (oracle seams) and section 14, step 6
Sprint: S11

## Context

The Go fourslash harness injects an in-memory filesystem, symlinks and configurable case sensitivity, a parse cache and plugin spawners, inferred-project options and an initialization signal. Semantic fourslash coverage belongs to Phase 5, but the transport that carries those injections has to be designed before the language service exists, so that Phase 5 connects a tested endpoint instead of inventing one.

## Decision

To be written in S11 with its design note: the test-host endpoint, its callback filesystem operations (`readFile`, `fileExists`, `directoryExists`, `getAccessibleEntries`, `realpath`), test-only initialization, options and plugin controls, cancellation and callback progress, and the transport contract fixtures (case-insensitive, symlink and mapper cases). Transport contract tests verify these operations without asserting language-service results.

## Consequences

Until accepted, S11 stays open. Acceptance requires the owner's review of the design note; semantic fourslash assertions remain deferred to Phase 5 and are not counted as coverage by this ADR.

## Evidence

`tsc/internal/fourslash` and `tsc/internal/testutil` in the pinned checkout; the transport fixtures once they exist.
