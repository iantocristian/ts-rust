# S07-bis: bounded binder lookup reuse

Date: 2026-09-09. Control: retained CP1 bundle
`target/s07-bis/cp1-node-read-candidate`, manifest
`3a57976f667b9857891edbe9f76de5261c43480ac8c62d946b4545d9ee19d931`.

The hypothesis is that local read consolidation and forwarding already-known
kinds remove enough repeated routing to improve the complete pipeline. The
recorded 122,786,331 `Binder::n` operations do not establish their machine cost
or how many are redundant. Neither two reads per physical node nor a 200–250 ms
saving is an expected result asserted by this experiment.

## Contract and scope

Use one candidate covering known-kind dispatch/container classification and
the audited narrowing/function-expression read sites. Retain the public checked
container classifier. Internal kind forwarding relies only on the binder's
stable syntax; actual payload-shape checks must survive unusual factory
kind/data pairs. Preserve short-circuiting and failed-access order.

Consolidate short borrows within mutation-free regions. Copy only the scalar or
ID needed to cross recursion/mutation. In particular, do not keep a lazy arena
read guard across another resolution, and do not introduce a generic 16-byte
header passed everywhere. Kind/parent/syntax edges are stable in this binder;
flags are not. Source/module entry changes export context, child/container work
changes reachability and `CONTAINS_THIS`, and error writes must use current
flags. Preserve fresh reads across those boundaries. The owner, publication,
binding storage, recursion policy and syntax-list representation are unchanged.

Counterexamples include short-circuited absent payloads, mismatched kind/shape,
optional chains, binary continuation stages, recursive container flag changes,
and lazy/published reads. Existing independently generated Go helper and binder
observations remain the authority; add only focused regressions for new behavior
boundaries rather than a new event inventory.

## One screen, then a decision

Review the code, run focused binder tests and affected lint/MSRV checks, freeze
the normal/allocation binaries, and compare all 13,094 workload graphs with Go
at one and eight workers. Use the existing performance runner without changing
its metric definitions, schedule or helper implementation.

The fixed screen retains eight warmups and 56 measured children with alternating
candidate/control order. Both modes require relative MAD at most 5%, wall upper
95% ratio at most 1.02, and allocation/RSS median ratios at most 1.02. A targeted
wall win requires median ratio at most 0.95 in at least one mode and its upper
bound below one. No sample replacement, extension or post-result variant tuning.
Report absolute milliseconds and bytes first. A noisy or failed screen is
inconclusive/rejected, not permission to keep sampling until it wins.

If the screen is promising, run the broader binder parity and current S07
ownership instrumentation before promotion. Inspect generated code only where
needed to substantiate a claimed mechanism; do not build a new trace consumer
or turn this into exhaustive disassembly attribution. If it misses the screen,
restore production code, preserve the experiment and proceed to compact storage.
This ordering spends the expensive ownership producer on a candidate that may
be retained; it does not waive correctness requirements for promotion.

The result does not close a Go-relative gate, attribute the entire remaining
bind gap, or quantify a standalone lookup latency. Fresh paired Rust/Go evidence
and complete owned allocation/RSS attribution remain required for S07 acceptance.
