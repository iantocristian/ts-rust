# Local binder migration diagnostic

The [migration record](../../../../../docs/S07-bis-local-bind-migration.md) describes the selected normal release binary, which passed all 13,094 canonical graph obligations at one and eight workers. Each mode also entered the local binding scope for all files, with zero checked/fallback files. This archive contains no performance screen or completed sprint/producer evidence.

- Archive: **13,918,744 bytes**, **596 verified members**.
- Archive SHA-256: `c916b970a4d4196aa3c067fc69b43affac7c53e2cea6461d8c7f11d35e59a1b6`.
- Selected binary SHA-256: `0929bdf9db11558def92d5a32b680af073eb15607a5b8371f52cfe288fa94bdf`.
- Pre-build source/configuration/protocol snapshot: 486 files, unchanged through all captures.
- Supplemental protocol schema and generator: two files verified separately against the accepted previous milestone archive and current originals.
- Captures: exactly one graph run and one local-path run per worker mode.
- Scoped instrumentation: Miri 3 arena + 21 AST tests; ASan 21 AST + 13 binder tests. Its 330 recorded source files match the normal-build snapshot.
- Raw graph equality: 13,094/13,094 at one worker; 13,028/13,094 at eight workers. The existing canonical protocol validates recognized qualified-name identities.

The first wrapper attempt omitted schema support and stopped after the successful worker-1 graph execution. Its logs and traceback remain in the archive. `resume.py` added independently hash-verified protocol support and reused that graph stream; it ran only the three still-missing workload invocations. The source snapshot and selected binary did not change.

`archive.json` records every member's size and SHA-256. `review.tar.xz` includes source, selected binary, raw streams, command/exit receipts, validation logs, protocol support, documentation and replay code. Cargo output/intermediate trees and full frozen workload bytes are excluded.

After extraction at a chosen directory, replay only the recorded streams:

```sh
python3 target/s07-bis/local-bind-migration-validation/replay.py --output /tmp/local-bind-migration-replay
```

The replay output directory must not already exist. Replay checks archived hashes and source continuity and launches no compiler or workload process.
