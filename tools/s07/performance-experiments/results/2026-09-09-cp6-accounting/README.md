# CP6 current-policy accounting

The [accounting record](ACCOUNTING.md) combines the earlier verified per-owner
census with compiled widths from frozen revision `f3426ac`. It models 371.034 MB
retained typed-row storage and 46.905 MB of discarded directory allocations.
This is a static model, not a new allocation census or performance capture.

Keep the four-row payload policy. The
[next auxiliary plan](../../../../../docs/S07-bis-auxiliary-plan.md) is selected
to combine with owner-local parent attachment; its retained-memory projection
is insufficient by itself to close the gates and must be measured.

- Archive: 3,847,208 bytes, 16 files.
- SHA-256: `9c656a4b914c0f8128f7168bf0286aa564fd9f2921ce6442487dc7e46d9d8fad`.
- Every member was read back and verified against [archive.json](archive.json).
- Includes exact layout source inputs, Rust helper/executable/output inventories,
  model script/output, census bytes/manifest and reviewed outline.
- Reproduction requires Rust 1.97.1 and the original checkout paths. The record
  distinguishes modeled retained/requested bytes, allocation calls and RSS.
