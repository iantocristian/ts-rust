# ADR 0010: Order-sensitive outputs are enumerated; comparators are ported exactly

Status: Accepted (2026-09-05)
Plan: section 6, decision 5

## Context

Symbol tables in Corsa are plain Go maps with random iteration order, so the checker sorts wherever order is observable: `CompareTypes` for union constituents, `compareSymbols` for members, `CompareDiagnostics` for diagnostics. Ids are the last resort only for intrinsic types, reverse mapped types without symbol or mapper, and symbols without declarations and with duplicate names. Draft 1 wrongly demanded that Rust reproduce Go's allocation sequence.

## Decision

Port `CompareTypes`, `compareSymbols`, `compareNodes` and `CompareDiagnostics` line by line and sort at exactly the points Corsa sorts. Construct intrinsic types in Corsa's order. List the residual id-sensitive cases and give each a test; keep a creation-trace mode in both binaries scoped to diagnosing those cases.

## Consequences

The Rust checker may allocate freely. The `union ordering` sub-test is a consistency check on exercised types, not a proof of total ordering for every type graph; the residual cases are known instabilities with their own tests.

## Evidence

`tsc/internal/checker/utilities.go` (`CompareTypes`, `compareSymbolsWorker`), `tsc/internal/checker/checker.go` (`getUnionType`; `compareTypeIds` with no callers), `tsc/internal/testrunner/compiler_runner.go` (`union ordering`).
