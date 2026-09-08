# Recorded list access-cost decomposition

The 2026-09-08 access trial finds that direct owner-descriptor traversal removes
most of the checked chunk path's gap to legacy lists. Cached slice traversal is
faster again, with an explicit 50.154 MB metadata cost. Neither result selects
a production representation: the recursive binder needs a disjoint immutable
list reader and mutable node writer before it can reuse these borrows safely.

This is a third, separate fixed capture. Its same-batch legacy and checked
chunk256 controls are not combined with the earlier page/chunk timing batches.
It retains all eight warmups and 56 measured children, seven observations per
mode and binary role. All children consume the same sealed 13,094-file physical
census: 3,114,989 backings and 6,047,867 synthetic edge words in per-file auxiliary
completion order, including obsolete and empty backings. Every child performs
eight complete checksum sweeps; none was removed, replaced or added.

Times are median milliseconds, with relative MAD in parentheses. Allocation
figures are decimal MB; all seven observations per mode are identical.

| Access path | Construction ms (MAD) | Eight-sweep traversal ms (MAD) | Requested MB | Retained MB | Freed/superseded requests MB |
| --- | ---: | ---: | ---: | ---: | ---: |
| Legacy boxed slices | 47.558 (0.17%) | 92.747 (0.16%) | 310.418 | 119.564 | 190.854 |
| Checked chunk256 IDs | 38.480 (1.05%) | 128.041 (0.59%) | 145.703 | 91.293 | 54.410 |
| Owner descriptors | 38.258 (0.62%) | 98.353 (0.62%) | 145.703 | 91.293 | 54.410 |
| Cached borrowed slices | 49.124 (1.26%) | 85.833 (0.90%) | 195.857 | 141.447 | 54.410 |

Owner-descriptor traversal is 23.2% lower than the checked chunk control and
remains 6.0% above legacy. Cached slices are 33.0% lower than checked chunks and
7.5% below legacy, but retain 18.3% more bytes than legacy. This separates the
access paths, not the instruction-level cost of each removed check: changed
inlining and optimization can contribute too. These sweeps are not a measured
weighting of binder work and cannot be added to parser/binder timing.

The owner iterator borrows descriptors from its own private vector. It skips
reminting a public ID and resolving that backing again, while checking the
physical chunk and initialized range. Arbitrary external IDs retain their
existing ownership/backing/range checks. The iterator allocates nothing.

The cached mode resolves every slice through the checked public path once. Its
3,114,989 borrowed-slice entries cost 49,839,824 bytes, plus 314,256 bytes for
the per-file vector headers: **50,154,080 bytes**. Cache construction belongs
inside construction timing and the request/live endpoint. The cache remains
alive for every sweep and drops before the owners. Ordinary Rust lifetimes
enforce that relationship; no self-reference or lifetime extension is used.
Replay requires the exact extra request/live charge and unchanged freed bytes.

All three chunk variants have identical physical storage: 31,443 chunks,
2,022,625 spare edge words and 446,764 retained scratch words. Checked and owned
allocation reports must match exactly. Every allocation child also verifies
zero allocation during traversal and balanced live bytes after dropping the
cache and owners. The separate allocator preflight measures the exact
1,200,050-byte request/growth sequence. Disposal and JSON formatting remain
outside the timed construction/traversal intervals.

Initial host load averages were **7.71 / 8.78 / 10.93** on the 18-CPU macOS arm64
host with 64 GiB memory. Builds, tests, graph comparisons and other coordinated
diagnostics had stopped. Process-name checks do not exclude unrelated host
activity or systematic timing bias. Results are exploratory even with low MAD.
No full AST retention, foreign-owner access, RSS, production binding or final
S07 gate is measured by this trial.

The next node-access slice must preserve production facts ordering (currently
sequentially consistent), short payload borrows and narrow binding writes.
Payloads themselves cannot stay immutably borrowed across recursive binding:
flow, symbol and container fields also change. Reusable borrows belong to the
disjoint list reader. Do not justify a permanent all-list cache from this result.

The [durable archive](results/2026-09-08-access/manifest.json) preserves original
manifests, all raw stdout/stderr, build/preflight logs and exact Rust/helper
snapshots. It references the existing verified owner-census gzip; native binaries
are omitted. Replay checks the recorded observations without a native child,
original workload checkout or `target/` directory:

```sh
python3 tools/s07/performance-experiments/storage-pilot/replay_access.py replay
python3 -m unittest discover -s tools/s07/performance-experiments/storage-pilot -p 'test_*.py'
```

Build manifest SHA:
`1ac32e32fa3c2f4ff44e2dbe53bcc020f46dc0773c1c7c31ad3dff62709ec89b`.
Capture manifest SHA:
`940222dee95061632f272b92353774b0bce5b94a5a5b983e6b0d77f4c533e055`.
The older runners, replayers and captures remain unchanged.
