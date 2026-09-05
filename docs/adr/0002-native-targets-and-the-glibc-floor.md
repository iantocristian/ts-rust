# ADR 0002: Native targets and the glibc floor

Status: Accepted (2026-09-05)
Plan: section 6, decision 13; section 13, item 15

## Context

Upstream builds twenty targets from one host. The owner cares about four. The npm client selects a binary by platform and architecture only, so libc variants cannot be distinguished by the loader.

## Decision

Four native targets: `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`. Linux binaries link against glibc 2.28, the floor of Node 18 and every later release line; CI inspects the versioned symbol requirements of the final ELF and fails on anything newer, and the suites run on a 2.28 image. The output layout mirrors upstream's platform packages (`lib/tsc`) so the pinned client's `getExePath` and the extension resolve it. Not built: the Windows host layer (named pipes, the Windows watcher and realpath), musl, and every best-effort target. Windows-style path semantics in `tspath` stay because baselines exercise them. Code signing and publishing under upstream names are out of scope.

## Consequences

The `ipc` crate implements unix sockets only; `fswatch` implements FSEvents, inotify and fanotify only. Amazon Linux 2 (glibc 2.26) is not supported. Static musl remains a measured alternative if a universal Linux binary is ever wanted.

## Evidence

`packages/typescript/lib/getExePath.js` (platform and architecture only); the Herebyfile platform table; Node release floors (Node 16: glibc 2.17; Node 18 and later: 2.28).
