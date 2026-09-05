# ts-rust

Planning repository for rewriting the TypeScript 7 native compiler and language server (the Go module under `tsc/` in microsoft/TypeScript, codename Corsa) in Rust.

Nothing here is compiler code yet. The repository holds the plan and the measurements it rests on.

## Layout

| Path | What it is |
|---|---|
| `PLAN.md` | The plan. This is the canonical document; edit it here. |
| `docs/corsa-in-rust.html` | The same plan as a designed, self-contained page. Published as a private page at https://claude.ai/code/artifact/6c72abf7-0d30-43fe-a7a8-6457828dcce8. Regenerate or edit by hand when `PLAN.md` changes. |
| `data/import-graph.txt` | Internal import edges of the Go module (`importer imported`), produced by `go list`. 766 edges. |
| `data/topological-order.txt` | The packages in dependency order, leaves first, produced by `tsort` over the graph. The crate map in the plan follows this order. |
| `data/MEASURED.txt` | Which TypeScript commit and Go version the data was measured with. |
| `scripts/import-graph.sh` | Regenerates `data/` from a TypeScript checkout: `scripts/import-graph.sh ~/git/TypeScript`. |

## Reading order

1. `PLAN.md`, section 1, for the seven-point summary and the estimate.
2. Section 6 for the architecture decisions, which are where the plan differs from a straight port.
3. Section 9 for the phases, gates and timeline.
4. Section 13 for what is deliberately still open and why.

## Status

Draft 1, 5 September 2026, measured against microsoft/TypeScript commit `1f70213d49`. The next concrete step is the decision memo and sponsorship described in section 14.
