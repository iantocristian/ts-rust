# CP1 follow-up: bounded syntax-list snapshots

Date: 2026-09-08. Control: retained CP1 core-read candidate, manifest
`3a57976f667b9857891edbe9f76de5261c43480ac8c62d946b4545d9ee19d931` at
`target/s07-bis/cp1-node-read-candidate`. This is a production CPU experiment;
it changes neither list storage nor binding-field representation.

## Contract and implementation

Fable identifies an alternative to keeping a `NodeSliceRead` alive across
recursive binding: resolve a chunk, copy its IDs to a stack buffer, release the
borrow, and bind those IDs. The previous CP1 record correctly describes the
requirements for a persistent borrow, but omitted this bounded-copy option.

Use at most 16 `Option<NodeId>` entries (128 bytes on the supported targets).
Copy only IDs, not node kind, flags, payloads or binding fields. Evaluate those
fields at the same point as the existing algorithm. Preserve ordinary list
order and both complete functions-first passes; do not interleave the two passes
per chunk. Handle the final partial chunk and nil IDs without introducing a heap
allocation. The average backing length is not an upper bound on list length.

Target `bind_each` and `bind_each_statement_functions_first` in
`crates/ts_binder/src/containers.rs`. The existing child visitor delegates to
`bind_each`. Keep its upfront empty-slice validation, and preserve the direct
`bind_each` empty-descriptor no-op. Raw, mapped and lazy lookup still uses the
checked `parsed_view().node_slice(...)` route at every chunk boundary.

The production binder does not mutate syntax-list backings; it mutates node and
binding fields. That is what permits copying future IDs in a chunk before
recursing. A generic caller permitted to replace backing entries would need a
different contract. The copied IDs borrow no payload and retain no extra owner;
the existing binder owner continues to keep their graph alive.

The cost hypothesis is fewer complete slice resolutions, from one per element
to one per nonempty chunk, at the cost of stack space, copying and loop control.
For short lists, initialization/copy overhead can erase the saving. Recursive
list calls can have several buffers alive, so depth and actual generated stack
frames matter. Do not infer a pipeline gain from average list length alone.

## Counterexamples and validation

Test lengths around 16 and 32, partial tails, subranges, nil entries, empty and
missing descriptors, wrong owners and lazy reads. Observe callback order and
changes to node fields after the IDs have been copied. Verify functions-first
ordering across chunk boundaries, and compare the complete published/exclusive
binding graphs. Preserve existing fallback and terminal-failure behavior.

Add the ownership-relevant runtime cases to the actual S07 E3 inventory; execute
them in debug/release/Miri/ASan. Run affected MSRV, lint and depth checks. Before
timing, freeze the actual normal/allocation executables and source/configuration,
then compare all 13,094 Go workload graphs at both worker counts. Run the broader
binder producer as a separate semantic check. No new instrumentation or tracing
may run during the normal timing capture.

## Predeclared screen and decision

Use the existing full-pipeline runner with the immutable CP1 control. Keep its
eight warmups and 56 measured children, alternating variant order, with separate
normal and allocation binaries at one and eight workers. Retain all raw samples
and failures. No extended batch, sample filtering or retuning of chunk width
after looking at this candidate's timing is part of this experiment.

Report milliseconds and MB, the per-worker candidate/control ratios, and the
distance to historical Go budgets. The ordinary screen applies: both variants'
relative MAD at most 5%, wall upper 95% ratio at most 1.02 and allocation/RSS
median ratios at most 1.02 in both worker modes. A targeted wall win needs a
median ratio at most 0.95 in at least one mode and its upper bound below one.
This temporary leaf optimization does not itself provide the general borrowed
facade: do not invoke the infrastructure exception to keep an inconclusive
result. Keep only after the correctness, cost and screen reviews; otherwise
remove the production shortcut and retain its recorded experiment.

An accepted candidate becomes the next immutable control. Neither decision
completes the memory gates or proves where the remaining CPU gap resides. The
page-backed integrated slice and [recorded access trace](S07-bis-access-trace.md)
remain separate work, not prerequisites for this bounded copy experiment.
