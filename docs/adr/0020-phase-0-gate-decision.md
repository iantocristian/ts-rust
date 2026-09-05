# ADR 0020: Phase 0 gate decision

Status: Proposed (2026-09-05)
Plan: section 9, Phase 0 gate; section 14, step 8
Sprint: S12

## Context

The spike is the initial go/no-go point. E1 to E4 must pass; E5 to E8 must meet their thresholds for the parts they measure, with the extrapolations written down as extrapolations. Full checking and emit acceptance, the full WebAssembly and embedding acceptance and the release matrix remain Phase 7 gates (ADRs 0002 and 0003).

## Decision

To be written at the gate: the measured results of E1 to E8 with their evidence artifacts, the extrapolations and workload limits for E5 to E8, the full checking/emit acceptance matrix, the remaining Phase 7 performance budgets, and go or no-go.

## Consequences

Until accepted, S12 stays open and no Phase 1 sprint may name it as done. A no-go decision stops the port; the evidence stays in the repository.

## Evidence

`status/evidence/` artifacts behind `exp.E1.pass` to `exp.E8.pass` at the time of the decision.
