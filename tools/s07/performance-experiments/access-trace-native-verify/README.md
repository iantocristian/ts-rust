# Native counterpart of the scoped access verifier

This standalone crate accelerates the existing Python verifier's recorded
operation checks. It does not change the recorder, production AST, historical
capture, or coverage claim. All Rust source forbids unsafe code. Its only
dependencies are the already locked `sha2`, `serde`, and `serde_json` closure.

The child reads **uncompressed stdin**, retaining at most a 1MiB block, bounded
per-file initial headers, bounded counters, and the declared file summaries.
Physical node order allows a dense header vector; lookup still compares the
owner before selecting the slot. It checks the same framing, block/footer hashes,
registry fields/sites, phase sequence, physical header counts, blob chunks and
named kind/flag observations as the Python reference. Unknown operation coverage,
unmapped births/fallback and complete list/payload semantic replay remain
unavailable. Progress goes to stderr; stdout contains only the final JSON object.

`run.py` streams a single gzip member through the original strict `GzipReader`.
Both gzip completion and the native child's successful exit are required. The
child's raw stream SHA must match the decompressor's SHA. The wrapper adds only
the gzip byte count and SHA to the report. A CRC error, trailing member, native
failure, mismatched hash or changed input leaves a failed receipt and the original
stdout/stderr. It never relabels the original pending capture as verified; the
verification result is a separate artifact.

Build an immutable artifact bundle with:

```sh
python3 tools/s07/performance-experiments/access-trace-native-verify/run.py build target/s07-bis/native-verifier-build-1
```

The build records source and helper snapshots, the dependency allowlist, effective
compiler/profile/configuration, Cargo's actual executable path and SHA, and the
system runtime-library inspection. Failed attempts remain in their distinct
output directories. Verification takes the build directory and its manifest
SHA, trace path and compressed SHA, a new output directory, the three **recorded
build registry snapshots**, and optional JSON limits/expected totals:

```sh
python3 tools/s07/performance-experiments/access-trace-native-verify/run.py verify BUILD BUILD_SHA TRACE OUTPUT --trace-sha256 TRACE_SHA --registry PROTOCOL STATE HOOKS --config CONFIG
```

The config keys are `max_payload`, `max_block`, `max_record`, `max_files`,
`max_active_nodes`, `allow_empty`, `expected`, and `progress_records`. Defaults
match the reference: 32GiB payload, 1MiB blocks/records, 13,094 files, two million
active initial nodes, and one million records between progress reports. The
wrapper separately defaults to a 3GiB compressed cap. `allow_empty` exists only
for protocol unit fixtures.

`test_equivalence.py` loads the unchanged original 16 test methods and executes
every trace/registry case through both decoders. Successful reports must compare
equal in full; failures must agree in acceptance. Additional cases cover native
duplicate JSON keys, gzip failure receipts, explicit field bounds, wildcard sites
and full-domain file indices. Each attempt keeps its input, config, registry
snapshots, command, stdout/stderr, status and report under `target/`. Set
`NATIVE_VERIFY_BINARY` to the exact built artifact and `NATIVE_VERIFY_TEST_OUTPUT`
to a fresh directory before running the unittest suite.

This is observer throughput work. Its elapsed verification time is not parser,
binder, storage or whole-pipeline performance evidence. Before using it for a
full recorded workload, compare its complete report with the already verified
sizing trace, then retain the new verification as a separate artifact.

The first frozen build (`1e8bd155…`) passed 109 dual-decoder fixture attempts;
its sizing report matched the existing Python report byte for byte. Its full
verification (`ddc6a4c9…`) checked 300,052,431 records. These captures and their
original wrapper snapshots are unchanged. A later wrapper revision checks that
the config file stays unchanged after the child and writes an explicit failed
receipt when process creation fails. Two targeted regressions exercise those
changes using the same frozen executable; the full trace was not rerun. The
completed full capture's config already matched its declaration, and its child
started and exited successfully, so these fixes do not change that result.
