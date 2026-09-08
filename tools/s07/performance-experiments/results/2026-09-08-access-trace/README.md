# Scoped access-trace review artifacts

The [checkpoint record](../../../../../docs/S07-bis-access-trace-results.md)
describes the result and its limits. No CPU, allocation or RSS gate is satisfied
by these diagnostic artifacts.

`review.tar.xz` contains 4,189 files: successful and failed recorder builds,
source/tool snapshots, native executables, sizing/full capture receipts and
graphs, the independent verifier's reports, the 109 preserved equivalence
attempts, and post-capture wrapper failure regressions. Every member was read
back and checked against [archive.json](archive.json).

- Archive size: 21,815,652 bytes.
- Archive SHA-256: `d5123fab109d9d63921ca32d98c9b20fd4029e24d9068ea3f313aae034fd1584`.
- Three large `trace.bin.gz` files are excluded, with their paths, sizes and
  hashes listed in `excluded_raw_traces`. The immutable originals remain under
  `target/s07-bis/`. The archive alone is **not a full trace replay package**.

The first full recording was sealed with a pending-verification manifest. Its
separate successful verification and [composition.json](composition.json) link
the original receipt, expected registries, raw hash, native executable and each
file's work/retained graph. The old pending manifest is deliberately unchanged.

To repeat receipt composition using the original immutable local bundles:

```sh
python3 tools/s07/performance-experiments/results/2026-09-08-access-trace/compose.py
```

This rechecks all four pinned inventories, executed recorder/helper hashes,
control/native invocation identities, registry/config/report relationships,
trace hash and every per-file graph/work comparison. It **does not decode the
raw trace again**. It refuses current helper drift rather than silently changing
the meaning of the historical check.

To repeat the raw verification, use the [native wrapper](../../access-trace-native-verify/README.md)
with a fresh output directory, the recorded native build and SHA
`1e8bd155938150d3f478cb43e6eed8a7e6f9a9fdf7adb196eecfa95728249732`,
`target/s07-bis/access-trace-full-1/trace.bin.gz`, its compressed SHA
`2292410213c8e42269131cb06a131199751f51e188c11bc8eeeb59678216d35b`,
and `target/s07-bis/native-verifier-full-1/config.json`. The three registries
must come from `target/s07-bis/access-trace-build-4/tool-snapshot/tools/s07/performance-experiments/access-trace/`,
in protocol/state/hooks order. The decoder remains the exact frozen executable;
the current wrapper adds the separately tested launch/config failure handling.
The run creates a new receipt and does not replace the archived result.

The native build's compiler/MSRV/Clippy/equivalence receipts and the sizing
byte-equality receipt are preserved under `target/s07-bis-native-verify/` within
the archive. Earlier development equivalence logs are also retained and are
distinct from the 109-case run against the final frozen binary. The two later
wrapper regressions did not rerun or relabel the full recording.
