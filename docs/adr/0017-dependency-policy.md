# ADR 0017: Dependency policy

Status: Accepted (2026-09-05)
Plan: section 6, decision 14

## Context

The Go module builds without cgo, without Node, with twelve direct dependencies, and CI proves it. A Rust tree that pulls in hundreds of transitive crates would be a regression in auditability.

## Decision

Few, vetted crates, each recorded with the reason it exists. `cargo-deny` enforces an Apache-2.0-compatible license list and bans duplicate versions and known advisories; `cargo-vet` records an audit for every crate; `Cargo.lock` is committed; no build script touches the network; the minimum supported Rust version is checked in CI. Expected direct dependencies mirror the roles of Corsa's twelve Go modules: scoped threads or rayon, parking_lot, mimalloc, serde with a JSON crate matched to the Go JSON v2 usage, an msgpack crate, an xxh3 implementation producing the same hashes the API encoder header carries, a patience diff, flate2, a Unicode normalization crate for the organize-imports comparer, stacker, and the platform crates for file watching on macOS and Linux.

## Consequences

Adding a dependency is a reviewed change with a recorded reason. The repository task runner (`xtask`) uses only `serde`, `serde_json` and `toml`.

## Evidence

`tsc/go.mod` in the pinned checkout.
