# Oracle tooling

Go programs that answer "what does the pinned Corsa do?" for the experiment
harnesses. They are built against the `upstream/` submodule at the provenance
pin and never modify it: this module declares the path
`github.com/microsoft/TypeScript/tsc/oracle` and replaces
`github.com/microsoft/TypeScript/tsc` with the submodule directory, which is what
lets it import the pinned `internal/...` packages without carrying a patch or
writing anything into the checkout. `go.sum` is only needed for the pinned
module's own dependencies.

Each program reads a checked-in fixture manifest and writes one JSON object of
`probe -> value` pairs to stdout. The producer script compares that object with
the Rust harness's object of the same shape; neither side can see the other's
answers, and a probe missing from either side is a mismatch.

| Program | Fixtures | Used by |
|---|---|---|
| `e4/` | `data/e4-fixtures.json` | `scripts/e4-harness.py`, the `e4` producer |

Build and run them through the producer scripts, which pin `GOWORK=off`,
`GOFLAGS=` and `CGO_ENABLED=0` the way the S01 oracle producer does.
