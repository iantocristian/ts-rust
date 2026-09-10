# CP1 follow-up review disposition

Date: 2026-09-08. This records Fable's latest review against the retained CP1
candidate. It does not change historical experiment results or final S07 gates.

| Claim or proposal | Verification and decision |
| --- | --- |
| Show the remaining distance to Go at each checkpoint | Accepted. The hash-pinned [distance replay](../tools/s07/performance-experiments/gate-distance/README.md) adds absolute excess and historical ratios to A0-b and CP1. Fresh paired Go captures are still required for acceptance. |
| Two routing fixes bought 12%; the remaining gap is mostly binding | The historical one-worker gap is 1.653 seconds. Separate screening percentages cannot be compounded into a measured cumulative change, and current phase attribution requires a new profile. Neither stronger claim is adopted. |
| Copy IDs before recursion instead of waiting for a disjoint reader | Accepted as a bounded candidate. Existing syntax backing is immutable during binding, so stack copies preserve the IDs while kind/flags remain live reads. The [list-copy record](S07-bis-list-copy.md) fixes width, counterexamples and full-pipeline criteria before timing. The completed screen shows 0.83% / 0.19% wall-median reductions without memory savings; reject this standalone shortcut for missing its 5% win requirement. |
| A real recorded trace should drive the next node-layout comparison | Accepted with limits. The [trace plan](S07-bis-access-trace.md) records actual operation order and reconstructs the retained working set. Instrumentation coverage is not CPU coverage, and replay cannot replace integrated measurements. Small fixtures remain semantic tests. |
| Adding inline fields to today's enum costs about 157 MB | The [compiled audit](../tools/s07/performance-experiments/binding-layout-audit/README.md) confirms NodeData 40→48 and Node 80→88 when an enlarged Identifier stays inline: 156,747,904 additional occupied-node bytes. Capacity, replacement storage, requests and RSS need separate accounting. Inline binding fields therefore remain coupled to compact payloads. |
| There are 14 nonboxed 32-byte payloads | Corrected to 21, of which 16 have the audited binding fields. All five named examples are 32 bytes. |
| The Go base partition is 83 with fields and 109 without | The disjoint base groups reproduce 23/24/9/27 when locals takes precedence, but direct fields also matter. CaseOrDefaultClause carries FallthroughFlowNode; the complete audited field set spans 84 shapes, leaving 108 without those fields. Generate from the complete flattened field inventory. |

Independent review of the list-copy implementation found no semantic defect in
order, nil/empty handling, lazy/foreign ownership, live node-field reads or the
two complete functions-first passes. The added buffer costs up to 128 bytes per
active traversal plus copying and loop work. Focused tests and code inspection
do not establish a speedup. The complete screen establishes only the small
observed benefit above and rejects the candidate under its unchanged policy.
