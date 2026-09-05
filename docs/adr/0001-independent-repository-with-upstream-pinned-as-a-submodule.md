# ADR 0001: Independent repository with upstream pinned as a submodule

Status: Accepted (2026-09-05)
Plan: sections 1 and 6, decision 11

## Context

The rewrite is owned and funded by the repository owner. Upstream (microsoft/TypeScript) is a moving target of 113 to 237 commits a month, and its cooperation is not assumed. The plan needs the upstream test data, schemas, lib files, locales, JS client, VS Code extension and Go source as inputs.

## Decision

This repository is the Cargo workspace. microsoft/TypeScript is a git submodule at `upstream/`, pinned to a commit (`1f70213d49` today). Moving the pin is a deliberate operation: bump, rebuild the unmodified oracle, reapply tooling and test patches in a disposable worktree, re-export schemas, verify the untouched client's generated protocol file, list changed Go files against the ledger, port those diffs, re-run every suite, then land the bump. Contributing upstream is a later option, never a precondition.

## Consequences

Upstream changes are absorbed through `PORTS.toml`, which records the pin each ported file was last synchronized to. Harness and exporter patches are carried against the pin and revalidated on every bump. Nothing in the plan depends on upstream accepting anything.

## Evidence

Upstream commit velocity measured from `git log` on the pinned checkout; the plan's section 1, item 6.
