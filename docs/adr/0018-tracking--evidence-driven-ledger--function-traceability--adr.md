# ADR 0018: Tracking: evidence-driven ledger, function traceability, ADRs and a dashboard

Status: Accepted (2026-09-05)
Plan: docs/TRACKING.md

## Context

The owner asked for a stricter way to know what is implemented. This project has an unusually good oracle, so implemented can mean the tests say so, and progress can be computed instead of claimed.

## Decision

Four parts, chosen from the options offered: a port ledger (`PORTS.toml`, one entry per upstream Go file with status, Rust path, tests and pin, regenerated from the checkout without losing progress); function-level traceability (`port:` markers in Rust doc comments matched against `data/go-functions.tsv`, the inventory generated from the pin, reported per package with the unported functions listed); Architecture Decision Records in `docs/adr/`; and a status tool, `cargo xtask status`, that computes every number from those inputs plus `status/experiments.toml` and the sprint files, writes `STATUS.md`, `status/status.json` and the dashboard `docs/status.html`, and with `--record` appends to `status/history.jsonl`. Sprints are TOML files whose exit criteria are machine checks over the computed metrics; a sprint closes only when they pass. Only `status` and `rust` in the ledger, and `measured` in the experiments file, are edited by hand, and only with the evidence in the same change.

## Consequences

Nobody, human or agent, can mark work done. Unknown `port:` markers fail the status run, which catches stale annotations after a pin bump. Rejected: hand-maintained checklists as the source of truth; Jira or Linear.

## Evidence

`docs/TRACKING.md`; `xtask/src/main.rs`; `scripts/ledger-init.py`; `scripts/go-inventory/main.go`.
