# P0 native contract evidence

`native.tar.xz` retains two equal supplemental Go captures (21 cases, 420
relations, nine residual comparator cases and eight byte-printing cases), the
final typed-audit command/report, strict verifier log and focused test log.
`native.tar.xz.manifest.json` records every member's length and SHA256.

The canonical typed graph and supplemental expectations are in `data/s08/`.
The raw static graph is not duplicated here: its compact representation retains
function identities, call sites, target sets and unresolved boundaries. Go aliases
and parameter names are canonicalized by type identity to remove nondeterministic
SSA spellings; the source and native hashes identify that generator exactly.

The existing `../acceptance-amendment/` archives remain the authority for primary
baselines, diagnostics, requests and source snapshots. Reproduction and all
remaining P1/P2 work are described in `docs/S08-P0-completion.md` and
`data/s08/README.md`. None of these P0 observations is Rust checker parity or a
per-type memory result.
