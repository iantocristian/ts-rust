# S07-bis: bind-only CPU comparison

Status: completed offline comparison, with a bounded combined CPU candidate
selected in [the implementation plan](S07-bis-bind-cpu-plan.md). No new compiler
run, profiler capture, timing screen or gate result was produced.

## Scope and verification

The Rust input is the current compact/shared-text normal executable captured in
[the post-text CPU archive](../tools/s07/performance-experiments/results/2026-09-09-post-text-cpu/README.md).
Its executable SHA-256 is
`3b375719ca8c3334c443f77b934c1388eec42ec1a94d12971586bc792056e8c2`.
The comparison selects worker samples with consuming `bind_parsed_file`
ancestry, including its validation/publication. The displayed innermost frame
receives self cost; each distinct ancestor receives inclusive cost once per
sample, even with recursion. Physical aliases supplement operation-group
membership but cannot replace the displayed self symbol.

The Go inputs are all three archived one-worker pipeline profiles from
[2026-09-08](../tools/s07/cpu-profile/results/2026-09-08/manifest.json).
Selection uses the recorded `phase=bind` label, retaining runtime/systemstack
work when the user-level binder ancestry is absent. Unlabeled background GC
remains in the whole-profile accounting, outside this bind partition.

The script verifies source archive member hashes, loaded digest and work counts;
reconstructs Rust's whole-worker self totals against the preceding audit; and
checks Go's phase/category totals plus 90 flat rows from native pprof exports.
All inputs cover 13,094 files, 19,593,488 nodes and 2,459,867 symbols with matching
diagnostic counts. Rust has no missing worker stacks.

Rust has **2,528 ms bind CPU weight** in one 1 ms sampling capture. Go has
**730, 670 and 690 ms**, respectively, in three historical 10 ms sampling
captures. These are different captures and samplers, not a paired speed ratio
or elapsed phase comparison. Tables use the selected bind denominator. Go's
pooled weight covers three runs; its mean below divides by three.

## Rust self-cost ranking

| Displayed function, abbreviated | Self ms | Bind share |
| --- | ---: | ---: |
| `Binder::bind_node_head` | 174 | 6.88% |
| `AstView::node` | 164 | 6.49% |
| `Binder::check_contextual_identifier` | 131 | 5.18% |
| `DefaultHasher::write` (leading instance) | 109 | 4.31% |
| `Binder::bind_worker` | 88 | 3.48% |
| `NodeRead::data` | 83 | 3.28% |
| `Binder::bind_each_child` | 57 | 2.25% |
| `BindBuilder::set_binding_field` | 56 | 2.22% |
| `StorageView::aux_here` | 53 | 2.10% |
| `CompactContext::decode_node` | 52 | 2.06% |
| `BindingWrite::store` | 51 | 2.02% |
| `mi_page_free_list_extend` | 50 | 1.98% |

Self costs partition sampled weight. They still include inlined callees that
the native display does not distinguish. A high self rank does not imply the
whole function is avoidable overhead.

## Go self-cost ranking

| Displayed function, abbreviated | Mean self ms/run | Pooled bind share |
| --- | ---: | ---: |
| `runtime.madvise` | 493.3 | 70.81% |
| `runtime.heapSetTypeSmallHeader` | 43.3 | 6.22% |
| `Binder.newFlowNode` | 33.3 | 4.78% |
| `runtime.memclrNoHeapPointers` | 20.0 | 2.87% |
| `Binder.declareSymbolEx` | 16.7 | 2.39% |
| `Binder.newSymbol` | 16.7 | 2.39% |
| `Binder.addDeclarationToSymbol` | 13.3 | 1.91% |
| `runtime.(*mspan).typePointersOfUnchecked` | 6.7 | 0.96% |

`runtime.madvise` contributes 510/490/480 ms across the three runs. At this Go
pin on Darwin, `sysUsedOS` calls `MADV_FREE_REUSE` for kernel accounting. The
sampled PC/path does not establish GC cost, waiting, or a particular amount of
kernel CPU. Removing this weight would misrepresent Go's bind partition.
The complete ranking and runtime categories remain in the result JSON.

Go's contextual-identifier check has no separately named self sample. That
does not mean the operation is free: inlining and 10 ms sampling limit direct
function correspondence. This comparison locates Rust mechanisms to inspect;
it cannot assign every Rust millisecond a corresponding Go instruction.

## Which proposed suspects survive inspection

| Proposed cost | Current bind evidence | Decision |
| --- | --- | --- |
| Stack query on every bind | 4 ms displayed guard/stacker/psm self, 13 ms inclusive; inlining hides some work. Neither call count nor per-node cost is measured. | Defer changing stack growth. The claimed 19.6M queries and large saving are unproven. |
| Repeated source metadata in identifier checking | Contextual check: 131 ms self / 373 ms inclusive. Source metadata: 48 / 120 ms, with 86 ms inside the contextual check. | Cache only immutable facts for this logical source, as one component. No isolated screen. |
| Generic flow-field writes | Binding-write group: 123 ms self / 166 ms inclusive. `NodeRead::data` additionally has 30 ms self with `Binder::set_flow_node` as immediate caller. | Replace payload decoding for capability checks and repeated generic dispatch with narrow generated access. |
| SipHash and name-pool comparisons | Hashing: 132 ms self; name-pool group: 28 ms self / 94 ms inclusive. Groups overlap. | Retain current keyed hashing for this candidate. The memory attribution did not establish a backing rewrite; changing collision protection is a separate decision. |
| Payload clones | `NodeRead::data` is 83 ms self, including the flow-capability check above. Many typed `to_owned` operations copy scalar IDs. | Target the confirmed enum materialization, not a blanket clone removal or inferred heap-allocation saving. |
| Parent/read helpers and `Result` plumbing | `AstView::node` has 164 ms self, including immediate callers `bind_worker` (23), `parent` (22), declaration-name (15), child binding (14), text (13), contextual check (11). | Do not equate this with `Binder::n()`: the exclusive builder already reads the checked core directly. Preserve checked errors; no broad facade rewrite. |

Operation groups overlap and are not summed into a predicted saving. In
particular, removing a capability read does not remove required flow encoding,
and caching a source fact does not remove identifier semantics. This evidence
does **not** yet demonstrate a complete path to Go's CPU gate.

## Selected next work and remaining state

Build one bounded candidate combining the flow-write path and immutable
source-fact caching. The [plan](S07-bis-bind-cpu-plan.md) identifies the existing
safe split, generated shape authority, compatibility behavior, counterexamples,
review and one complete-pipeline screen. These mechanisms have a smaller proof
burden than changing stack growth, hashing or the general read API. Stop this
direction if the complete result does not justify it; do not extend it into
field tracing, another profile or a tuning matrix.

CP1 remains the accepted comparison control. The current compact implementation
remains experimental: its previous one-worker upper CPU ratio was 1.056, which
does not satisfy the plan's 1.02 infrastructure condition. The next CPU additions
are measured together with that implementation, against CP1 in the same batch.
All earlier candidates, failed attempts and verdicts remain recorded.

The name/table attribution is complete, but a new memory implementation was not
selected. Whole-pipeline live/replacement traffic is known; a comprehensive
parse-versus-bind allocation split is still absent. The last fixed screen still
needs about 209 MB less allocation, 105–110 MB less RSS and 2.243/0.446 s less
one/eight-worker wall time against historical Go-derived limits. Fresh final
Go-relative gates remain open; no claim that one more screen will close them is
supported.

## Reproduction

See [the comparison archive](../tools/s07/performance-experiments/results/2026-09-09-bind-only-comparison/README.md)
for the complete ranks, immediate callers, input hashes, checks and retained
intermediate analysis attempts. The checked-in command is:

```sh
python3 tools/s07/cpu-profile/compare_bind.py --output target/s07-bis/bind-only-comparison/reproduced.json
```

Restore the four Rust input members at their recorded paths from the preceding
archive when absent locally. Go exports are already checked in. The command is
offline and refuses to overwrite a result. Percentages from native pprof's old
whole-profile display intentionally differ from the bind-only percentages here.
