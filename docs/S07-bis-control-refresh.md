# S07-bis: refresh the paired acceptance control

Status: capture in progress. This precedes the
[combined page/text/helper experiment](S07-bis-pages-text-helpers.md).

The six binder source mappings were restored in `be74975`. The first standard
graph attempt failed because its sandbox could not write the existing Go cache.
The cache-enabled attempt passed all 13,094 files in both modes, with unchanged
counts and input identity. Its evidence record is
`8b8579f26caac8868b77e6d683d86e6c4080d8de8cdae5926c5f3afcf0d3a778`.

The following timing capture correctly stopped before sampling because its
freshly rebuilt Rust executable differed from the graph executable. A second
normal build with matching target/intermediate environment also differed.
Comparing the two available normal executables finds 123 changed bytes in seven
ranges: mimalloc's build-time string, the temporary output path, Mach-O UUID and
code-signature data. Both executables are 6,655,928 bytes. The pinned mimalloc
source prints `__DATE__` and `__TIME__` in its build diagnostic. A fresh build is
therefore not a byte-identical substitute for an already validated graph binary.

Keep exact binary checks. The producer repair consumes the Go and normal Rust
executables built for the source/configuration-bound graph pass, then builds the
allocation variant separately. Validate identities before and after capture;
changed sources, configuration, graph receipts or executables must fail closed.
The prior failed timing artifacts and raw graph observations remain preserved.
No failed or incomplete attempt supplies a performance sample or metric.

The renewed graph and timing capture will use the repaired producer and its own
source identity. Previous controls, experimental screens and the failed refresh
remain historical evidence; none is overwritten or relabelled as the new run.
