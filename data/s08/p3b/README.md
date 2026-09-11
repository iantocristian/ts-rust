# S08 P3b checkpoint evidence

The archive contains the original pinned-Go capture and source snapshot, Rust
observations, the full comparison and focused validation logs. `record.json`
binds the archive to the implementation revision and toolchains.

Nineteen complete programs (64 queries) and four shared-file merge modes match.
Four named programs retain explicit Unsupported operations; the full comparison
is `matched: false`. Readonly property query graphs match, while their source
grammar checking remains unported. No failure is excluded or graded as parity.

`review.tar.xz` and `review.json` record the subsequent PR review fixes. This
expanded capture contains 73 queries in 24 programs, including all original
requests and four new `object` queries. Twenty complete programs (68 queries)
and all four merge modes match, with the same four explicit failures. Original
program/merge observations are unchanged. The review archive includes the new
42-test checker run and checker Clippy; earlier semantic/protocol results remain
in the initial archive and were not rerun for these focused fixes.

See [the record](../../../docs/S08-P3.md) and
[replay instructions](../../../tools/s08/p3b/README.md). This is a P3 increment,
not a full S08 acceptance or performance result.
