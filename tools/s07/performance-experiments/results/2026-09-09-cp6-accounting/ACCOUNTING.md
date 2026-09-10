# CP6: current typed-page allocation accounting

**Keep the four-row payload policy. Select the already bounded core auxiliary compaction for the next combined candidate.** The current page store has a modeled 46.905 MB of directory growth churn. It is not a remaining 399 MB allocation problem by itself. The auxiliary candidate projects 95–107 MB retained savings; it is useful progress, not sufficient evidence that either memory gate will pass.

Decimal MB throughout. This is a static model using verified earlier per-owner counts and the exact frozen current layout, **not a new current allocation census or CPU measurement**. No workload capture, Cargo build, or policy matrix was run.

## Inputs and exact layout

`target/s07-bis/owner-census-capture/owners.ndjson` matches its manifest SHA-256, `835823ecaba55715ac69ee2efaafdb71acc47a24b52bdd6b73fd55f1f2ddd10d`. Its 13,094 owners contain 19,593,488 physical nodes. The latest construction/access screen reports the same file/node/symbol counts and loaded-input SHA; this supports reusing the owner distribution as a model input, not claiming a newly observed current distribution.

The frozen construction/access candidate contains the current `compact/pages.rs` unchanged (SHA `865963a98a1bbf8be02d6c3df2a5958ddf4121bcaa6da36c1c98659c4278c77b`) and current generated rows (SHA `94a9757e78f318843bcbeaf52a102da4ff848786890c26b5c358d10823348190`). A single standalone layout check compiled that unchanged page implementation and all 192 generated row field declarations with the pinned Rust 1.97.1. The NodeKind and CompactSlice helper definitions preserve their actual transparent i16 / three-u32 representations. Results:

- `Directory<T>` is 24 bytes and `RowPages<T>` is 32 bytes, not 40. Compiler enum niches matter here.
- `AstPayloadStore` is 1,472 bytes: 184 optional boxed stores. Eight shapes, including Token, have zero-sized rows and allocate no page store.
- Current binding fields and AtomicU32 facts are included. Identifier is 8 bytes, PropertyAccessExpression 20, and CallExpression 24. All 192 widths and alignments are retained in `cp6-accounting/current-layouts.txt`.

Artifacts and hashes are in `target/s07-bis/cp6-accounting/`. Reproduce the layout with `rustc +1.97.1 --edition=2021 target/s07-bis/cp6-accounting/current_layout.rs -o target/s07-bis/cp6-accounting/current_layout`, run that binary into `current-layouts.txt`, then run `python3 target/s07-bis/cp6-accounting/account.py`. The script verifies source/helper, compiled output, executable, compiler record and raw census hashes, reconciles every shape count, and writes `accounting.json`. It does not run the workload. `layout-artifacts.json` binds the original compiled artifacts; a new reproduction whose executable differs must retain its own inventory rather than silently replacing that record.

## Current policy costs

For each populated owner/shape, pages = ceil(count / 4). The first page lives directly in the boxed store's `One` variant; a second page creates `vec![first, page]` with capacity two. Later directory growth doubles its pointer capacity. Existing rows never move and directory growth copies pointers only.

| Component | Retained bytes, MB | Requested bytes, MB | Allocator calls, modeled |
| --- | ---: | ---: | ---: |
| Four-row payload pages | 284.471 | 284.471 | 4,786,400 |
| Per-shape pointer directories | 51.146 | 98.052 | 753,502 |
| Boxed RowPages roots | 16.143 | 16.143 | 504,462 |
| Heap subtotal | **351.760** | **398.665** | **6,044,364** |
| Embedded per-owner shape-slot arrays | 19.274 | 19.274 attributed charge | No separate allocation |
| Total including embedded charge | **371.034** | **417.940** | **6,044,364** |

There are 504,462 populated stores: 239,410 use exactly one page, and 265,052 need a pointer directory. Logical current row fields occupy 266.403 MB. Page tail slack is 18.068 MB. The 46.905 MB difference between requested and retained directory arrays is the entire modeled growth churn of this policy; the pages and roots are allocated once and retained.

The request model charges the complete new allocation size on each Vec growth, matching `cap` 0.1.2's successful-realloc `update_stats(new_size)` behavior. “Allocator calls” means calls at the Rust allocator interface, including directory reallocations; it does not mean OS allocations or prove an elapsed-time cost. The embedded slot array already belongs to the owner allocation and must not be added to an independently counted whole-owner total a second time.

## What changing only this policy can accomplish

As an intentionally unattainable upper bound, retain all typed row fields and the owner slot array but make every page tail, pointer directory and boxed store free. That saves only **85.357 MB retained** and **132.262 MB requested**. These ceilings are about 29% of the ~293 MB RSS deficit and 33% of the ~399 MB allocation deficit; actual RSS does not equal requested retained bytes. Any implementable typed page policy retains some of those costs. Eliminating growth churn alone saves at most 46.905 MB of requests and no retained payload bytes.

The 6.044 million allocator calls warrant attention if a native profile identifies allocation call overhead. They do not establish a CPU saving or justify repeating the rejected per-shape Vec policy. Larger/capped pages, chunk carving and directory changes exchange tail slack, roots and indirection; this record does not choose or price another matrix. Keep the four-row policy for the next combination.

The existing auxiliary audit (`cp4-memory-next-audit.md`) gives a larger concrete retained target: **95–107 MB**, using an eight-byte mixed-ID core locator, 24-byte list headers and eight-byte backing descriptors with a cold full-width fallback. It requires the core/lazy auxiliary representation seam and semantic list reads. Its prior capacity assumptions, escape costs and current request savings remain unmeasured; do not add 107 MB directly to the allocation gate estimate or claim that modeled retained savings equal an RSS drop. Combine this implementation with the active parent-attachment work, preserve the current controls and judge the complete candidate together.

## Limits

- The census predates compact storage. Current same-shape edits replace rows in place; public shape-changing edits can leave old typed rows retained, which the old final-header census cannot count. The ordinary parser/binder source changes fields within an existing payload shape, but this model is not proof that no exceptional row exists in any workload path.
- The model prices successful one-row-per-physical-node construction. Panic-aborted work, lazy payload allocation, text/reference escapes, allocator size classes and cache behavior are outside it. Existing census owners had no lazy slots; broader API correctness still requires them.
- Requested bytes, retained requested capacity and peak RSS are different quantities. The latest complete screen, not this model, remains the measurement authority: about 2.434 GB allocated and 2.506 GB peak RSS.
- The model's exact compiled row widths do not make the reused per-owner distribution a current physical census. No per-component performance improvement is claimed.
