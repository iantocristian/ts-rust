# Local binder migration: one native CPU diagnostic

The migrated list-resolution, binary-operand and narrow-flow access mechanisms have no displayed direct caller samples under `AstView::node` or `NodeRead::data` in this capture. The remaining samples identify shared declaration/name/modifier helpers, strict-mode checks, container work and final validation. Dispatch still invokes some of those general helpers; this is evidence for continuing the planned migration, not evidence that the binder is entirely free of the general AST facade.

This is one Time Profiler capture of the already selected normal release binary from the [migration record](S07-bis-local-bind-migration.md). It is not a timing screen, promotion decision, Go comparison or gate result. The [review archive](../tools/s07/performance-experiments/results/2026-09-10-local-bind-migration-cpu/archive.json) preserves the raw trace, XML, analysis, failed setup attempts and provenance.

## Capture and accounting

The binary SHA-256 is `0929bdf9db11558def92d5a32b680af073eb15607a5b8371f52cfe288fa94bdf`. It was not rebuilt. Source attribution uses its frozen 486-file snapshot, not the newer workspace edits being made concurrently. The input manifest and selected binary were hash-checked before and after capture. The one-worker workload matched 13,094 files, 161,740,237 bytes, 19,593,488 nodes, 2,459,867 symbols, 423 parse diagnostics and 5,250 bind diagnostics; its loaded-input digest also matched.

Time Profiler 16.0 (17F113) recorded 5,401 target-process Running samples at 1 ms weight. Native parse/bind ancestry, supplemented by physical symbol ranges from this exact executable, partitions them as follows. These are **sample weights**, not elapsed phase durations:

| Partition | Sample weight (ms) |
| --- | ---: |
| Parse | 2,356 |
| Bind, including validation/publication ancestry | 1,899 |
| Other worker work | 22 |
| Main thread and other non-worker work | 1,124 |
| Total | 5,401 |

The 57 missing-stack samples all belong to the non-worker partition. They remain in the denominator. The offline attribution script checks that its phase sum and missing-stack sum equal the native analyzer's totals. A displayed leaf contributes exactly once to self weight; physical aliases support ancestry and never create additional self samples.

## Remaining general accesses

| Displayed leaf | Bind self samples / ms | Fraction of bind sample weight | Other partition |
| --- | ---: | ---: | --- |
| `AstView::node` | 87 | 4.58% | 129 ms in parse |
| `NodeRead::data` | 48 | 2.53% | None observed |

All 135 focal bind samples also resolve to the corresponding physical leaf function in the selected binary. Their direct callers divide without overlap as follows:

| Immediate caller family | `AstView::node` (ms) | `NodeRead::data` (ms) |
| --- | ---: | ---: |
| Shared free AST helpers | 59 | 23 |
| AST name/modifier/body methods | 5 | 9 |
| Binder methods, including unresolved inlined callers | 9 | 16 |
| Binding validation | 14 | 0 |
| Total | 87 | 48 |

The largest named pairs are `get_name_of_declaration` (7 node + 19 data), `has_dynamic_name` (7 + 3), `is_ambient_module` (9 node), modifier access (7 data), and symbol/binding validation (14 node). The complete direct-caller inventory and every focal sampled stack are in `attribution.json`.

No focal sample has an immediate displayed caller in local edge resolution, typed binary operand extraction or narrow flow access. Calls that merely have recursive `bind_each_child_target` or `bind_worker_target` farther up the stack are not charged to list or dispatch mechanics. For example, a sampled `AstView::node` beneath `is_left_hand_side_expression`, `check_strict_mode_binary_expression`, and then `bind_target_head` belongs to the still-general strict-mode helper.

Five `NodeRead::data` samples show `bind_worker_target` as the direct displayed caller; one shows the local-entry closure. The frozen code deliberately retains general helper/container boundaries, and this normal binary has no packed source/inline debug information. Those six samples cannot be assigned to a finer inlined operation. The result therefore supports the narrow mechanism claim above, not a zero-overhead claim for whole dispatch or whole binary binding. Binary destructuring/logical/assignment helpers still cross the general view in this snapshot.

Other displayed bind self entries remain substantial: `LocalBind::general_node` 58 ms, `Binder::n` 40 ms, and `CompactContext::decode_node` 20 ms. They are separate leaves, not extra weight to add to an inclusive access estimate. Remaining hashing, declarations, symbols and helper migration are outside this diagnostic's scoped question.

## Continuation layout ledger

The selected binary's `RawVec<binary_trampoline::Step>::grow_one` passes alignment `0x8` and element size `0x18` to `finish_grow`. Thus its continuation vector stores **24-byte `Step` values with 8-byte alignment**. The retained `step-grow.asm` gives the exact instructions; it does not confuse the function's 64-byte stack frame with the vector element size.

`Exit` is also **24 bytes in this build**, derived from the frozen source and that assembly bound: `BindingNode` distinguishes a `NonZeroU32` local ID from a `NonZeroU64` checked ID, requiring at least 16 bytes at 8-byte alignment; `Exit` adds three booleans, requiring at least 24, and it must fit the observed 24-byte `Step`. This is a current-layout ledger entry, not a stable Rust ABI promise or measured allocation saving. No extra compilation or benchmark was used to obtain it.

## Limits and retained failures

The first sandboxed profiler setup failed before child launch because the host kernel/template services were unavailable under that sandbox. Its empty trace and logs are retained. Host access then produced the single completed workload invocation. The first sandboxed export reported `Missing features`; export with host access succeeded from the same raw trace without another workload execution.

There was no quiet-host requirement, repetition, paired control, phase timer adapter or additional instrumentation. Concurrent implementation/build activity was permitted. The raw benchmark stdout contains its normal timing fields; they are retained but are not interpreted here. Sampler weight is neither a function-call count nor a latency estimate. Absence among 1 ms samples is not proof of no executions, and hidden inline callers limit fine attribution. No CPU speedup, significance or new acceptance claim follows from this one capture.
