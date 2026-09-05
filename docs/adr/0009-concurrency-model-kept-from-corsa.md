# ADR 0009: Concurrency model kept from Corsa

Status: Accepted (2026-09-05)
Plan: section 6, decision 4; section 13, item 9

## Context

Corsa parses and binds in parallel, partitions files across checkers with a tuned FENNEL heuristic, emits per file in parallel, and serves editor requests on immutable snapshots. Those behaviors have baselines and a race-mode CI job.

## Decision

Keep the model exactly: parallel parse and bind, the same partitioning heuristic and constants from `compiler/checkerpool.go`, one checker per thread (`Send`, not `Sync`), parallel emit, requests on `Arc<Snapshot>` values built copy-on-write, and a cancellation token polled wherever Corsa polls its context. Compiler and service APIs are synchronous; execution uses bounded worker pools plus transport readers that progress independently of workers, because the API transport handles responses while requests execute. An async runtime may exist in the transport layer only.

## Consequences

No query-based incremental architecture before parity; it is deferred until after cut-over. Blocked workers must never stall cancellation or callback traffic.

## Evidence

`tsc/internal/compiler/checkerpool.go`, `tsc/internal/core/workgroup.go`, `tsc/internal/project/snapshot.go`, `tsc/internal/ipc/conn_async.go`.
