# ADR 0012: Panics, recovery and generation retirement

Status: Accepted (2026-09-05)
Plan: section 6, decision 7; section 13, item 10

## Context

Corsa has about 600 panic sites and recovers at request boundaries; after a recovered panic it logs, answers an error and keeps the checker. The API checker deliberately persists so that type and symbol handles keep reference identity, handle registration panics on id collisions, and `Project.Clone` shares the program and checker pool across snapshots while API registries are per snapshot.

## Decision

Invariants stay `panic!` and `debug_assert!`; every LSP request, API call and test step has a `catch_unwind` boundary with stack sanitizing; I/O and configuration failures are `Result`; locks are `parking_lot`, and absence of poisoning is not evidence of valid state. A caught checker panic retires the checker and pool generation before the checker can be acquired again, and every snapshot, project, in-flight lease and API registry referencing that generation observes the retirement. Registries validate generation before resolving handles, so old registries cannot attach to a replacement checker with reused sequential ids; retired storage stays owned until leases drain; if mutated shared state cannot be isolated, the affected session is retired. Generation metadata stays server-side; wire shapes and handle syntax are unchanged, and invalidated requests return the existing protocol error form.

## Consequences

Rebuilding a checker is a full initialization; only disposal is cheap. E3 injects a panic through one of two retained snapshots sharing a pool; Phase 6 repeats it through the real client.

## Evidence

`tsc/internal/lsp/server.go` (`recover`), `tsc/internal/project/checkerpool.go` (persistent API checker), `tsc/internal/project/project.go` (`Clone`), `tsc/internal/api/session.go` (`registerType`, per-snapshot registries).

## Amendments

Draft 3.1 evicted only the panicking checker; draft 3.2 extended invalidation to the pool generation and all referencing registries after the third review.
