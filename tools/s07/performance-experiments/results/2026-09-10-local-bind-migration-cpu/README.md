# Local binder migration: one native profile

The [diagnostic record](../../../../../docs/S07-bis-local-bind-migration-cpu.md) describes one untimed, one-worker Time Profiler capture of the previously selected normal binary. No rebuild, timing screen or additional workload invocation was performed.

- Archive: 1,755,860 bytes; 232 hash-verified members.
- Archive SHA-256: `5cd77638f829aed41be69b2801c70264dcac3b2ccb297a85319ea9555910b223`.
- Selected binary SHA-256: `0929bdf9db11558def92d5a32b680af073eb15607a5b8371f52cfe288fa94bdf`.
- Exact selected binary/source are in the referenced and hash-verified [migration archive](../2026-09-09-local-bind-migration/archive.json).

The archive retains the raw native trace, exported XML, full frame analysis, displayed-leaf caller attribution, setup/export failures, commands, provenance, layout witness and report. Full disassembly and duplicate binary/source/workload bytes are excluded. `archive.json` hashes every member. Phase weights are sampled CPU attribution, not elapsed durations or a gate result.

Offline replay after extraction (no compiler or workload process):

```sh
python3 target/s07-bis/local-bind-migration-cpu/attribute.py
```
