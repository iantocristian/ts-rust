# S07-bis: one CPU sample after shared-text changes

Status: one capture and analysis complete; diagnostic only. The retained control and all gates remain
unchanged. The current exact normal executable is the corrected `8f7236e` freeze,
SHA-256 `3b375719ca8c3334c443f77b934c1388eec42ec1a94d12971586bc792056e8c2`.

The name/table attribution does not justify a backing rewrite. Existing native
CPU evidence predates the generated keyword matches and proven UTF-8 slices,
which targeted 398 ms of distinct sampled work. The source-metadata lookup subset within contextual-identifier checks covers
about 107 ms; the broader source-info group covers 151 ms in binding and 1 ms
in parsing. Neither weight is a promised saving or justifies another isolated
experiment. Do not select a large implementation from those stale rankings.

Run exactly one one-worker Time Profiler capture of the already-frozen normal
binary. Reuse the existing capture/analysis helpers and selectors; do not rebuild,
add field hooks, collect a replay trace, run eight workers or extend a timing
batch. Verify executable/input/helper hashes, work counts and the loaded digest.
Keep command/output/native trace inventory and exported samples. Record parsing
versus consuming binding/publication sampled CPU and distinct remaining costs;
no weight is a promised saving or an elapsed phase measurement.

This sample resolves whether a concrete remaining CPU path supports the next
combined implementation. Prefer changes with an explicit removable mechanism
and proportionate proof burden. If the sample still supplies only small or
ambiguous opportunities, state that limitation rather than opening a tuning
matrix or another capture. The completed allocation diagnostic and all previous
screens retain their original verdicts.

## Result

The one current normal-binary capture completes the same 13,094 files,
161,740,237 loaded bytes, 19,593,488 nodes and 2,459,867 symbols, with 423 parse
and 5,250 bind diagnostics. The loaded digest and frozen executable match.
No source or binary was rebuilt or changed. Its profiled pipeline wall is
4.908396708 s; that value is not a comparative timing sample.

The exported trace contains 5,808 running CPU samples weighted at 1 ms each.
Worker weight is 4,835 ms: parsing 2,288 ms, consuming binding/publication
2,528 ms and 19 ms unassigned. Main-thread weight is 973 ms, including preload
and retirement. There are 57 process rows without stacks and no missing worker
stacks. These are sampled CPU weights, not the elapsed phase measurements
reported in the earlier [phase diagnostic](S07-bis-current-costs.md).

| Selected inclusive group | Parse ms | Bind ms |
| --- | ---: | ---: |
| Constructors | 464 | 0 |
| Parent attachment | 292 | 0 |
| AST reads/routing | 132 | 202 |
| Full payload-enum reads | 2 | 117 |
| Contextual identifier checks | 0 | 373 |
| Source-file/frame reads | 6 | 120 |
| Name/table/symbol storage | 0 | 327 |
| Hashing | 4 | 132 |
| Binding-field writes | 0 | 166 |
| Completion / binding validation | 68 | 68 |
| Keyword lookup | 38 | 19 |
| UTF-8 validation | 1 | 1 |

Groups overlap. For example, 86 ms of source reads occurs inside contextual
identifier checking; 127 ms of hashing occurs inside name/table/symbol storage.
Neither pair may be added. Construction-or-parent attachment covers 756 ms as
a union, including required allocation and edge work. The broad core/semantic
read union covers 605 ms across the process, including 3 ms of main-thread AST
reads. It does not isolate a removable facade cost. `AstView::node` is the leading
worker self symbol at 289 ms; `NodeRead::data` has 84 ms self weight.

The current sample has no UTF-8-validation stack with a `JsString::slice` caller
and no memcmp stack with keyword ancestry. The remaining memcmp partition is
64 ms under source-suffix matching, 9 ms under name/table/symbol operations and
2 ms elsewhere. No sampled call is not a proof that no calls occur. Keyword
lookup still costs 57 ms total including the generated comparison/dispatch work.
These observations support ending keyword/slice tuning. They do not establish
an isolated speedup by subtracting the earlier sample or change the failed screen.

## Decision

This was the decision at the end of the capture. The subsequent offline
[bind-only comparison](S07-bis-bind-cpu-comparison.md) now supplies concrete
flow-write and source-fact mechanisms and selects a bounded combined candidate.
It reuses this evidence; it does not change the capture or screen verdict.

Do not launch another profiler, per-field trace, row-policy matrix or standalone
lookup experiment. Immutable source-fact caching remains a possible small
component, now bounded by the 86 ms contextual source-read subset rather than
an assumed whole binder saving. That bound is not enough for a new isolated
experiment. The current sample identifies distributed required and avoidable
work, but does not yet justify another substantial implementation with a
credible route to the CPU gate. A broad read/storage rewrite is not selected
merely because its inclusive group is large.

The next implementation needs a concrete mechanism that removes enough of this
work with a manageable correctness proof; smaller components may join it and
must be measured together. No new storage saving is inferred from this CPU
capture. The measured [name/table bounds](S07-bis-name-table-attribution.md),
the completed shared-text screen and all final gates remain unchanged.

## Evidence

- Capture receipt SHA-256: `1a56be6abb6244d2839f0b92071466c74a2dd1c202ee03b713b8d8566f012b36`.
- Sample XML SHA-256: `d0a35c811733f6c1ff2b2031c009a2bb59724a47c0d42c4f9302d89aa1d2d119`.
- Summary SHA-256: `7a8ca7e153cee86747f40bcce9300ec6f9bf6a1cce06c5890adb47ea37283489`.

The [archive](../tools/s07/performance-experiments/results/2026-09-09-post-text-cpu/README.md)
retains the wrapper/command/output/receipt, exported XML, physical symbol map in
the summary, reused analysis helpers and independent result review. The native
trace remains local with its exact file inventory; the frozen source/binary is
pinned by the preceding shared-text archive. No packed debug rebuild is implied.
Helper, binary and input hashes are checked around capture. Full-host quiet was
not continuously measured, and profiling perturbs execution.
