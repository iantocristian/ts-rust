# Recorded access workload for the compact node slice

Date: 2026-09-08. This replaces the small fixture as the **performance driver**
of the next node-layout comparison. Small fixtures remain semantic regressions.
No trace capture or layout timing is claimed by this plan.

Fable is right that population counts and a tiny fixture omit working-set size
and real access order. Record actual binder operations from a separately staged,
instrumented copy of the accepted implementation over the full frozen workload.
Use the latest retained candidate as the source; record its immutable manifest,
patch, actual executable, tools and loaded-input identity.

An operation trace gives access-frequency/order coverage for its instrumented
surface. It does **not** by itself give CPU coverage: it omits uninstrumented
symbol/string/control work, and recording changes execution/cache behavior.
Trace replay also has its own decoding and memory traffic. Keep these limits
explicit when interpreting any layout speed difference.

## Capture contract

The tooling home is `tools/s07/performance-experiments/access-trace/`, with
`stage.py` for checked observer additions and `probe.py build/capture/verify` for
immutable artifacts. Store the operation registry and format version there;
do not widen `cargo xtask gen` or leave tracing on production hot paths.

1. Freeze all 13,094 files and their original order/options. Capture one-worker
   operation order first. Mark file and parse/bind/publication boundaries, and
   retain ordinary endpoint roots. Do not invent a deterministic global
   eight-worker interleaving or claim that serial replay measures contention.
2. Preserve the full physical initial state needed to reconstruct each candidate:
   node identity/allocation order, kind and shape separately, scalar fields,
   lists and shared subranges, text/backing identity, source bytes and required
   binding state. Include unvisited/obsolete storage in working-set accounting.
   Births during binding, including lazy records, must be explicit events.
3. Instrument actual reads/writes: header/kind/flags, payload field reads,
   child-list resolution and enumeration, and narrow flag/symbol/flow/locals/
   container writes. Include observed inputs/results and operation identity so
   replay can detect a changed value or order. Never replace a real field read
   with an unconditional synthetic read and call it captured behavior.
4. Use owner-qualified stable local node/list identities, not runtime IDs whose
   observation would allocate or initialize them. Copies and temporary values
   need explicit provenance; unmapped operations are counted and reported,
   never silently attributed to the preceding node. Define nil, foreign,
   imported, failed and lazy identities in the format.
5. Use per-owner/worker sequence numbers and bounded binary output blocks with
   lengths, hashes and a completion footer. Estimate event volume before full
   capture and enforce a declared byte limit; exceeding it produces an incomplete
   diagnostic, not a sampled or truncated successful trace. Preserve observer
   failures and never silently filter files, operations or shapes to fit.
6. Count captured operations by type, shape, caller/phase and owner independently
   of population counts. Match every workload input and resulting graph/work
   observation to the accepted semantic protocol. Check event ordering, births,
   references, values and final state; include missing/duplicate/reordered event
   counterexamples in the producer tests.

The staged observer should reuse the owner-census access-only pattern where it
fits, but must inventory actual accessor instrumentation rather than treating
every `Binder::n()` resolution as a read of every payload field. Cover helpers
that bypass `Binder::n()` through `AstView`, subtree-facts and generated accessors.
List every uninstrumented surface. Report an unknown-operation count only where
a separate observation measures it; otherwise mark the count unavailable. Preserve facts'
current sequentially consistent atomics and never materialize lazy/runtime state
merely to obtain a trace value.

## Replay and integration decision

Reconstruct current storage, typed rows and mixed plain/atomic rows with identical
logical state and matched list/text policies. Page-256 is the leading list
candidate, with page-64 a bounded memory challenger. The nine proposed shapes are
a first implementation slice, not sufficient CPU coverage because they occupy
62.23% of physical nodes. Select/extend supported operations from the captured
frequencies; use explicit unchanged fallbacks for the remainder and charge their
working set. Do not drop unsupported shapes or renormalize covered events to 100%.

Replay the actual operation sequence, comparing values, mutations, child order
and final graph state. Preserve narrow writes and disjoint read/write capabilities
where available. Charge reconstruction, directories, identity/escape/text tables,
trace buffers, scratch, stack-copy traffic and disposal in named domains. A field
registry derived from Go bases must also include direct binder-written fields;
an embedding count alone is not a complete generator specification.

Keep trace decoding and trace-buffer residency matched across adapters and
report them separately. A separately observed decoder baseline can expose a
dominating driver cost; do not subtract its elapsed time as a corrected native
benchmark or claim identical cache interference. Preserve file/phase order and
retention so replaying only binding after preconstructing every file does not
silently replace the production parse→bind-per-file working set. If parse activity
is omitted, document that remaining cache-state difference and confirm the
layout on an integrated parser/binder before promotion.

Publish absolute milliseconds and MB first, operation coverage/unknowns,
full-pipeline control distances and all raw samples. A recorded-trace result
selects the integrated experiment; only actual whole-pipeline parity, ownership
and CPU/memory measurements can promote production storage or pass S07 gates.
