# ADR 0016: Toolchain and lints

Status: Accepted (2026-09-05)
Plan: section 6, decision 12; section 13, item 14

## Context

Corsa builds without cgo on stable Go with seven custom analyzers. Rust needs a comparable baseline of hygiene without pinning the compiler to a nightly toolchain.

## Decision

Stable Rust with a minimum supported version one behind current stable; `clippy` at pedantic minus an allow-list; `rustfmt`; `cargo nextest`; `lld` or `mold` plus `sccache` in CI; `#![forbid(unsafe_code)]` everywhere except the leaf crates where Corsa uses `unsafe` today (file watching, paths, platform FFI). Release profile: `panic = "unwind"` (required by ADR 0012), fat LTO, one codegen unit, mimalloc. Any compiler-private lint tool has its own pinned toolchain and does not change the compiler's stable-build requirement.

## Consequences

The replacement for upstream's `checkchildren` analyzer, which flags returns that skip child checks, is section 13, item 14, still open; `forbidparentaccess` becomes structural with the declarations transform exempt, three analyzers are Go-specific and moot, and two map to rustc and clippy.

## Evidence

`.golangci.yml` and `tools/customlint/*.go` in the pinned checkout; upstream CI runs with `CGO_ENABLED=0`.
