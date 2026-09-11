# S08 P3a checkpoint evidence

`capture.tar.xz` retains the original Go capture and source snapshot, Rust
observations and exact comparison. `record.json` identifies the implementation
revision, runtimes and archive hashes. This is a named P3 increment, not full
P3/S08 acceptance, type-count parity or a performance result.

The expanded review capture has 30 queries in 13 programs. Eleven complete
programs (28 queries) and four merge modes match. The two native negative cases,
`union-duplicate-properties` and `union-missing-type`, retain explicit Rust
Unsupported outcomes. The whole comparison remains `matched: false`; no failure
is relabeled as parity. The pre-fix semantic regression output is retained in
`review/before-test.log` inside the archive.

See [the increment record](../../../docs/S08-P3.md) and
[replay commands](../../../tools/s08/p3/README.md).
