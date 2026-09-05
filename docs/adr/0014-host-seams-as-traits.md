# ADR 0014: Host seams as traits

Status: Accepted (2026-09-05)
Plan: section 6, decision 9

## Context

Every upstream harness injects behavior through a small set of interfaces: the virtual file system, the compiler host, the tsc system with its fake clock, the checker's program interface and the content-mapper host.

## Decision

Object-safe, `Send + Sync` traits with the same shapes as `vfs.FS`, `compiler.CompilerHost`, the tsc `System`, `checker.Program` and the content-mapper host.

## Consequences

Harness ports are mechanical, and one test corpus drives both implementations through the same seams.

## Evidence

`tsc/internal/vfs/vfs.go`, `tsc/internal/compiler/host.go`, `tsc/internal/checker/checker.go` (`Program` interface), `tsc/internal/contentmapper/contentmapper.go`.
