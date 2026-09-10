# One CPU sample after shared-text changes

The [result](../../../../../docs/S07-bis-post-text-cpu.md) records one Time
Profiler capture of the exact current frozen normal executable. All work counts
and the loaded digest match. No source or executable was rebuilt or modified.

Sampled worker CPU is 4,835 ms: 2,288 parsing, 2,528 consuming binding/publication,
and 19 unassigned. There are no missing worker stacks. Keyword lookup accounts
for 57 ms; no sampled UTF-8 validation has a slice caller. Remaining broad groups
include required work and overlap; none is a promised saving. No substantial
rewrite is selected from inclusive totals alone. The paired performance screen
and all final gates remain unchanged.

- Archive: 498,800 bytes, 31 files.
- SHA-256: `1a1fe624ace7eb576b8bce32c15a368e77a98e42d27b3d97ba3bd8d20a288214`.
- Every member was read back and hash-verified against [archive.json](archive.json).
- Includes command/receipt/output, exported sample XML, symbolized summary,
  exact reused analysis helpers, plan review and independent result review.
- The 205-file native trace remains local; the receipt inventories every file.
  The summary records physical/display symbol mappings for offline analysis.
- Original frozen source/binary remains in the
  [shared-text archive](../2026-09-09-compact-text-processing/README.md).
  Workload/toolchains remain external. No packed-debug rebuild is implied.

The retained `target/s07-bis/post-text-cpu-audit.py` recomputes the groups from
exported XML without launching a workload, with its pinned helper and original
frozen bundle restored at their recorded paths. The independent review recounts
all samples, phases, groups, intersections and unions, and verifies binary/input/
helper hashes. These are sampled CPU weights, not elapsed phase timers or
comparative speedups. Full-host quiet was not continuously measured.
