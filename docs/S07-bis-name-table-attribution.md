# S07-bis: bounded current name/table attribution

Status: diagnostic preparation after the shared-text screen; no storage change
or performance promotion is proposed. Use implementation `56e2127`, whose
measured source is the corrected `8f7236e` freeze. CP1 remains the control.

The combined candidate requests 2.244 GB and peaks at 2.322 GB RSS. Historical
limits still require approximately 209 MB less allocation and 105–110 MB less
RSS. Its one-worker CPU nonregression is unresolved. The earlier one-run
allocation diagnostic found 333.733 MB freed/superseded requests; that total is
not automatically avoidable. Existing row accounting is insufficient to close
the request gap and does not justify another page-policy matrix.

## Decision this measurement resolves

Measure the current canonical-name pool and symbol-table storage once, including
actual growth. They share names between symbols and table keys; charge them once.
Do not reconstruct owned syntax nodes or port the obsolete owner census. This
family has several independent growable structures and is not covered by the
current typed-row accounting. Its measured waste must establish relevance before
another representation or reservation change is considered.

Capture only:

- Name bytes and range entries: lengths, capacities and requested growth.
- Canonical-name hash and compact/full table maps: entry/bucket counts, safe
  `HashTable::allocation_size()` bytes and allocation-size transitions.
- Table cardinality histogram (`0`, `1`, `2`, `3–4`, `5–8`, `9–16`, `17+`),
  owner/table counts, variant conversions, removals and wide-name escapes.
- Explicit name-to-owned calls and selected bytes. These counts are not Arc
  allocation requests; actual backing costs remain unassigned unless measured.

Live entries, required name bytes and canonical identity support are not
removable overhead. Report retained allocation, logical element bytes and growth
separately. Include full new allocation requests on successful reallocation,
consistent with the pinned counter; same-capacity hash-table rehash may allocate
nothing. Inspect the pinned implementation and calibrate initial growth,
replacement, removal and compact-to-full conversion on a tiny fixture before
assigning request totals. Table-record arenas and owner roots are separate from
hash backings. Wide structures need explicit counts and unassigned costs if
observed; do not guess their capacity or treat them as zero implicitly.

## Implementation boundary and checks

Use a disposable copy of the exact frozen source, with a diagnostic-only patch
to `symbol_tables.rs` and a small existing benchmark-driver hook. Fixed scalar
counters and histograms must not allocate per event or retain names/nodes. Read
retained tables while every completed file remains alive; serialize only after
sampling. Growth counters do not change the production storage policy, binder
lifecycle, work or source ranges. Keep new diagnostic/test code near 200–250
handwritten lines in this storage family, plus the reused driver/build adapter.
If it requires a broader profiler, stop and document the missing counts.

Build through fresh final and intermediate Cargo directories, preserve the
unmodified manifests/lockfile and exact workspace closure, and retain the patch,
source inventory, compiler profile, executable hash and build/check logs. Run
the tiny allocation-counter fixture and affected symbol-table tests. Then run
exactly one full 13,094-file, one-worker allocation-enabled diagnostic, verifying
all counts, loaded digest and exclusive/fallback binding counts. No CPU timing
claim, eight-worker run, access trace, page matrix or repeated workload capture.
Record observer costs separately; this cannot amend the existing timing screen.

## Exit

Report absolute retained/requested/growth totals and clearly conditional
ideal-capacity bounds. If measured waste is far below the remaining gaps, end
this family's gate-closing investigation. If it is material, choose one concrete
storage or lifecycle change with its compatibility obligations, then screen the
complete candidate. Removing a construction-only index would not erase its
already-issued requests, and rebuilding it for later mutation could add costs.
No implementation saving is assumed in advance.

A bounded independent decision audit selected this family and named these
limits. Review the concrete patch and calibration before the single workload
capture. The shared-text screen, all failed attempts and final gate definitions
remain unchanged.
