# ts-rust

The Rust rewrite of the TypeScript 7 native compiler and language server (the Go module under `tsc/` in microsoft/TypeScript, codename Corsa). Upstream is consumed as a pinned dependency; this repository is the workspace.

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
4. Section 9 for the dependency order, the gates and the spike experiments.
5. Section 13 for what is deliberately still open and why.

## Status

Draft 3.1, 5 September 2026, measured against microsoft/TypeScript commit `1f70213d49`. Draft 3 sets the project's terms: this repository is the Rust workspace, upstream is to be pinned as a git submodule at `upstream/`, the targets are macOS arm64/x64 and Linux x64/arm64 (glibc), the spike must prove memory, WebAssembly and embedding, the owner approves baseline divergences, and the sequence is expressed as dependencies and parity gates rather than a calendar. Technical items pending the owner's feedback are listed in section 13 of the plan.
