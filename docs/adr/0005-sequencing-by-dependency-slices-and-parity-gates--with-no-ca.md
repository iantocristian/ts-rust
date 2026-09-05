# ADR 0005: Sequencing by dependency slices and parity gates, with no calendar or staffing model

Status: Accepted (2026-09-05)
Plan: sections 8 and 9

## Context

Earlier drafts carried a thirty-month calendar and a staffing table. The owner rejected planning by engineer-days; pace is set by the owner and the tooling, and progress must be measurable rather than scheduled.

## Decision

Phases are ordered by dependency, not time. Each phase names the tested slices it needs to start and the complete coverage it needs to finish; a component may use a supported, tested slice of another component before that component reaches full parity, and unsupported operations fail explicitly and are recorded in the slice's manifest. The spike is the initial go/no-go point; every later phase closes only on its stated acceptance tests, and the cut-over gate in ADR 0003 is mandatory.

## Consequences

There are no dates in the plan. Sprints are files with machine-checked exit criteria (see ADR 0018). The dependency table in section 9 is the authority on what may start when.

## Evidence

Plan section 9; `data/topological-order.txt` from `go list` on the pinned module.
