# Corsa in Rust

Engineering plan, draft 3.1, 5 September 2026. Measured against microsoft/TypeScript at commit `1f70213d49`. Draft 3 sets the project's terms: an independent repository with upstream pinned as a dependency, four targets, all three benefits proven in the spike, divergences approved by the repository owner, and sequencing by dependency and parity gates rather than by calendar or headcount.

A plan to replace the Go implementation of the TypeScript 7 compiler and language server (the `tsc/` module of microsoft/TypeScript, codename Corsa) with a Rust implementation that ships as a drop-in `tsc` binary for macOS and Linux: the same 48,075 baseline files, the same LSP and JS-API wire protocols, usable by the upstream JS client and VS Code extension without changes.

| | |
|---|---|
| Source to replace | 317,820 lines of Go across 82 packages, of which 47,442 are generated; 5,082 Go files; no cgo; twelve files of `unsafe` confined to file watching and paths |
| Tests to satisfy | 12,721 compiler and conformance cases producing 48,075 baselines; 4,356 fourslash tests (761,930 lines of Go); tsc, build, watch, project, LSP and JS-API suites; 342 MB of test data |
| Wire contracts to keep | LSP 3.17 with negotiated UTF-16 or UTF-8 positions; the `--api` protocol (msgpack framing, binary AST encoder version 8 with WTF-8 strings and UTF-16 positions, 144 methods); the content-mapper plugin RPC |
| Upstream | microsoft/TypeScript, pinned at a commit and consumed for test data, schemas, lib files, the JS client, the extension and the Go oracle binary; 113 to 237 commits per month to absorb by moving the pin |
| Targets | macOS arm64 and x64, Linux x64 and arm64 with glibc. Nothing else. |
| Pace and gates | Ordered by dependency, gated by parity. The spike must prove memory, WebAssembly and embedding, all three. No calendar or staffing model. |

## Contents

1. [The short version](#1-the-short-version)
2. [Why do it](#2-why-do-it)
3. [What is in the upstream repository](#3-what-is-in-the-upstream-repository)
4. [What makes this hard](#4-what-makes-this-hard)
5. [Goals and exit criteria](#5-goals-and-exit-criteria)
6. [Architecture decisions](#6-architecture-decisions)
7. [Compatibility contracts](#7-compatibility-contracts)
8. [Strategy and crate map](#8-strategy-and-crate-map)
9. [Sequence and gates](#9-sequence-and-gates)
10. [Risk register](#10-risk-register)
11. [Consumed from upstream, not rewritten](#11-consumed-from-upstream-not-rewritten)
12. [Alternatives considered](#12-alternatives-considered)
13. [Open questions and pending technical decisions](#13-open-questions-and-pending-technical-decisions)
14. [First steps](#14-first-steps)

## 1. The short version

1. **This is a rewrite in the hard third and a port in the easy two thirds.** The Go code is a near line-for-line translation of the original TypeScript source kept alive by a garbage collector: nodes, symbols, types and the checker form cyclic, lazily mutated pointer graphs, and the checker keys dozens of caches by pointer identity. Rust cannot express that shape. The checker's data layer and the project system's snapshots must be re-architected onto arenas and integer ids while their control flow is preserved. Scanner, parser, binder, printer, transformers, module resolution, the command line and the formatter port mostly mechanically once the AST design exists.
2. **The TypeScript team evaluated Rust for this codebase in 2024 and chose Go for exactly that reason.** Their reasoning still holds, and the spike exists to settle with evidence what it costs to do it anyway.
3. **The test corpus makes it tractable.** The inputs and baselines are language-agnostic text, and three harness seams (the binary AST encoding, LSP through a test-host protocol, and the JS-API test suite that spawns the binary) let the Go binary built from the pinned upstream act as an oracle for the Rust one from the first week.
4. **The critical path is the checker** (60,703 lines, one 32,523-line file, 2,885 functions, a struct with about 320 fields), followed by the language service (41,500 lines) and the project system (12,300 lines). Emit, the command line and the API server run as parallel workstreams.
5. **Order-sensitive output is enumerated, not assumed.** Corsa orders union constituents, members and diagnostics by sorting at the point of observation with structural comparators: type flags, names, declaring file and source position, tuple shape, type arguments. It falls back to ids only for intrinsic types and rare residual ties. The Rust checker must port those comparators exactly and build the intrinsic types in Corsa's order; it does not have to reproduce Go's allocation sequence.
6. **Independent repository, pinned upstream.** This repository is the Rust workspace. microsoft/TypeScript is pinned at a commit and consumed for test data, schemas, lib files, locales, the JS client, the extension and the Go oracle. Upstream changes are absorbed by moving the pin and porting the diff against a file ledger. Contributing upstream is a later option, not a precondition of anything here.
7. **Four targets.** macOS arm64 and x64, Linux x64 and arm64 with glibc. That removes the Windows host layer (named pipes, directory-change watching, Windows realpath), musl, and every best-effort target from scope. Windows-style path semantics in the path utilities stay, because baselines exercise them.
8. **Three contracts come before the AST is generated.** Node ownership across file, transform, builder and session arenas; the three string representations (UTF-8 source text with byte offsets, WTF-8 JavaScript string values, UTF-16 or negotiated wire positions); and file-owned binding with program and checker overlays. Draft 1 got all three wrong, and the spike exercises each.

## 2. Why do it

Corsa already captured the large win of leaving JavaScript; Rust competes with Go, not with the old compiler. The three benefits below are all primary, and the spike proves each with a measured experiment before the rest of the plan starts.

- **In-process embedding.** The Rust tools that now dominate the JavaScript toolchain (oxc, rolldown, rspack, turbopack, biome, deno) can link a Rust checker directly instead of driving the `--api` server over a socket. Today anything that wants in-process access must itself be written in Go, which is why the typescript-eslint native linter is a Go program. Experiment E8.
- **WebAssembly.** A `wasm32` build of the compiler for the playground, browsers and hosts that cannot spawn native processes. Go's WebAssembly output is large and slow, so Corsa has no such story and the JS API requires a native binary. Experiment E7.
- **Memory and tail latency.** Arena-owned type universes free wholesale, there is no collector headroom (Go's collector by default lets the heap grow to twice the live set between collections), and no collection pauses inside editor requests. Expect 30 to 50 percent lower peak memory; CPU gains of 1.2 to 2 times are plausible from data layout but not guaranteed. Experiments E5 and E6.
- **Compile-time concurrency rules.** "Never mix types from different checkers" and "requests run on immutable snapshots" are comments and race-detector CI jobs today. With `Send` and `Sync` they become facts the compiler enforces.
- **Portability without cgo tricks.** Corsa reaches FSEvents on macOS through a hand-rolled, cgo-free foreign function layer. Rust reaches system APIs natively.

## 3. What is in the upstream repository

Non-test Go lines per area, measured on the pinned commit. Generated code is counted separately because it is regenerated from a schema rather than rewritten.

| Area | Go packages | Lines | Notes |
|---|---|---:|---|
| Type checker | `checker` | 60,703 | `checker.go` alone is 32,523 lines; `relater.go` 5,044; `flow.go` 2,761; node builder 3,705. 2,885 functions. |
| Language service | `ls`, `ls/autoimport`, `ls/lsutil`, `ls/change`, `ls/lsconv` | 41,536 | Completions 6,900 lines, string completions 2,247, find-all-references 2,774, auto-import registry 5,400. |
| Emit | `printer`, `transformers/*`, `sourcemap` | 37,163 | ES transforms 11,251 (class fields 3,618, decorators 2,751); declaration transform 4,212; TS, module and JSX transforms 7,813. |
| AST | `ast` | 21,219 | 10,058 lines generated from `tools/scripts/tsc/ast.json`; utilities 4,631; pointer-based nodes with parent links and per-kind arenas. |
| Program, options, resolution | `compiler`, `tsoptions`, `module`, `modulespecifiers`, `packagejson`, `outputpaths` | 19,236 | Program construction, checker pool (FENNEL partitioning), about 160 compiler options, node and bundler module resolution. |
| LSP protocol types | `lsp/lsproto` | 18,325 | 17,566 lines generated from the LSP 3.17 metamodel by `_generate/generate.mts`. |
| Syntax | `parser`, `scanner`, `binder` | 17,002 | Includes the JSDoc reparser that rewrites JSDoc into synthetic TypeScript nodes; positions are UTF-8 byte offsets. |
| Command line, build, watch | `execute/*`, `fswatch`, `diagnosticwriter` | 14,656 | `tsc`, `-b` orchestrator, incremental `.tsbuildinfo`, native watchers for FSEvents, inotify, fanotify, kqueue and Windows. Only FSEvents, inotify and fanotify are in scope. |
| Project system | `project`, `project/ata`, `project/dirty`, `project/logging` | 12,309 | Immutable, reference-counted snapshots built copy-on-write; a reference-counted parse cache that binds files on entry; automatic type acquisition spawns npm. |
| JS API server | `api`, `api/encoder`, `ipc`, `jsonrpc` | 12,168 | 144 methods, msgpack framing, binary source-file encoder (protocol 8), unix sockets and named pipes, callback file system. Only unix sockets are in scope. |
| Diagnostics | `diagnostics` | 9,658 | 8,854 generated from 2,135 messages plus 85 extra; 13 embedded gzipped locales. |
| Formatter | `format` | 4,259 | Rule-based formatter with its own baselines. |
| LSP server | `lsp`, `lsp/lspwatcher` | 3,734 | Request queue with cancellation, progress, watcher registration, API-over-LSP connection, stack sanitizer. |
| Content mappers | `contentmapper`, `spanmap` | 2,795 | Child-process plugins (for example `.vue`) that produce virtual TypeScript over JSON-RPC. |
| Utilities and hosts | `core`, `collections`, `stringutil`, `tspath`, `jsnum`, `vfs/*`, `glob`, `semver`, `json`, `locale`, `bundled`, `astnav`, `evaluator`, `pseudochecker`, `transpile`, `tracing`, `pprof`, others | about 26,000 | Unicode 15.1 identifier tables (3,496 generated lines), WTF-8 string helpers, JavaScript number semantics, virtual file systems, 108 embedded `lib.*.d.ts` files. |
| Test harnesses (Go, not counted above) | `fourslash`, `testrunner`, `testutil/*`, `execute/tsctests` | about 17,000 | Compiler runner with directive parser and baseline diffing, fourslash harness with an in-memory file system and an in-process server, fake system with clock for tsc tests, project test utilities. |
| **Total non-test Go** | **82 packages** | **317,820** | 47,442 generated; about 270,000 hand-written. |

Alongside the Go module: `packages/typescript`, the npm package and JS API client (53,200 lines of TypeScript, 12,863 generated, including a WTF-8 decoder), `packages/vscode-typescript` (4,184 lines), the hereby build file, generators, and the release pipelines. None of that is rewritten; the client and the extension are consumed from the pin and pointed at the Rust binary.

## 4. What makes this hard

### The Go code is a port, and Rust cannot be ported to

Every load-bearing structure in Corsa is a pointer graph with cycles and late mutation. `ast.Node` carries a `Parent` pointer and an interface-typed payload; the binder writes symbols and flow nodes into nodes after parsing; `ast.Symbol` points at its declarations, members, parent and export symbol; `checker.Type` points back at its checker and symbol. The checker itself is a 320-field struct whose caches are maps keyed by `*ast.Node`, `*ast.Symbol` and `*Type`, plus 26 `core.LinkStore` side tables keyed by pointer, plus function-valued fields that swap behavior at runtime. Go's collector frees the whole tangle when the checker is dropped.

None of that survives the borrow checker unchanged. The data layer becomes arenas and ids (which Corsa already half-uses through `core.Arena` and `LinkStore`), and every function that today mutates through a pointer while holding other pointers must be rewritten to work with ids. The control flow can and must stay the same; the plumbing cannot.

### Order is made deterministic by sorting, and the comparators are the contract

Symbol tables in Corsa are plain Go maps, which iterate in random order, so the checker sorts wherever order is observable. `CompareTypes` in `checker/utilities.go` orders union constituents by type flags, then names and alias arguments, then per-kind structure: declaring file and source position through `compareSymbols`, tuple element shape, type-argument lists, deferred-reference location and type mappers. Members are sorted with `compareSymbols` (first declaration position, then name), diagnostics with `CompareDiagnostics`. The compiler runner's `union ordering` sub-test shuffles every union and re-sorts it to prove the comparator is a consistent total order, which is what lets parallel checkers agree with each other. `compareTypeIds` still exists in `checker.go` but has no callers.

Type ids remain the last resort in three places: intrinsic types, which are created in a fixed order when a checker is constructed; reverse mapped types with neither symbol nor mapper, which the source itself calls unstable and rare; and symbols with no declaration and duplicate names. Draft 1 of this plan read the ordering as id-based and demanded that the Rust checker reproduce Go's allocation sequence. That was wrong. The real requirement is narrower and testable: port the comparators line by line, construct intrinsics in Corsa's order, and treat the three residual cases as known instabilities with their own tests.

### Strings are WTF-8, not UTF-8

JavaScript strings can contain lone surrogates. Corsa keeps them in Go strings as three-byte WTF-8 sequences (`EncodeJSStringRune` in `stringutil/util.go`), the relater and the JSX transform special-case them, the API encoder's string section is documented as UTF-8 with WTF-8 for such values, and the JS client ships its own WTF-8 decoder. A Rust `String` cannot hold `"\uD800"`. Positions come in three encodings as well: UTF-8 byte offsets inside the compiler, UTF-16 code units in the API encoder (it converts every node position through a per-file position map), and UTF-16 or UTF-8 for LSP depending on what the client negotiates.

### Files are bound before any program exists

The project system's parse cache binds a file as it is created (`project/parsecache.go`) and hands the same bound file, symbols included, to every program and snapshot that references it while its reference count is positive. Incremental builds, auto-import and source-definition code also bind files outside any program. Symbols are therefore owned by files. Anything a program or checker adds on top, such as merged symbols or resolved types, is an overlay that must be disposable without touching the shared file.

### Nodes have several owners

Source-file arenas are only the first kind. Transforms allocate synthetic nodes through their own factory, and the emit context records each synthetic node's original in a side map (`printer/emitcontext.go`). The checker's node builder, which produces the trees behind `typeToString` and hover, allocates in a factory whose `ReleaseArenas` call drops everything not still referenced by a cache (`checker/nodebuilder.go`). The API session, the options parser and the formatter create scratch factories of their own. Go's collector makes those retention rules implicit; Rust has to state them.

### Deep recursion on fixed stacks

Go stacks grow on demand up to 1 GB, and Corsa never had to think about recursion depth beyond TypeScript's own instantiation limits. It even removed the trampolines the TypeScript-in-TypeScript compiler used: the binder, `checkBinaryLikeExpression` and the printer's binary-expression emit all recurse on the left operand, so a long concatenation chain recurses once per term in each of them. Rust threads have fixed stacks and a stack overflow aborts the process. Long binary-expression chains, deeply nested JSX and recursive conditional types will find this on day one unless the design reserves large stacks and guards the deepest paths.

### The target moves

The Go tree received between 113 and 237 commits a month this year from about fifteen regular contributors, with roughly a quarter of commits authored by coding agents. This repository absorbs that by moving its upstream pin and porting the diff between two pins against a file ledger, which is why the Rust code keeps Go's file, function and variable names wherever possible: the diff between two pins must be transplantable.

### Calibration

The Go port's first commit landed on 30 September 2024. The upstream repository switched to the Go-only layout on 19 August 2026, after 2,546 commits to `tsc/`. That was a one-to-one translation by the people who wrote the original, with the original test suite, and with heavy agent assistance. A Rust rewrite has the same suite and can use the same assistance, but it pays for the re-architecture and for tracking. This plan does not restate that history as a schedule; it orders the work and gates it.

## 5. Goals and exit criteria

### Goals

- A `tsc` binary for the four targets that the upstream `packages/typescript` client and the VS Code extension, built from the pin, can be pointed at without changes.
- Identical baselines across all suites, with intentional differences recorded in an allow-list approved by the repository owner, the way `testdata/submoduleAccepted.txt` records Corsa's accepted divergences from TypeScript 6.
- Wire compatibility for LSP, the API protocol and the content-mapper protocol, verified by running the existing clients and test suites unchanged.
- Performance at or above Go on the TypeScript-benchmarking scenarios (vscode, self-compiler, mui-docs, xstate, bluesky) at 2, 4 and 8 checkers, with at most 70 percent of Go's peak memory.
- A `wasm32` build of the compiler and an embeddable crate API, both proven in the spike and delivered in the hardening phase.
- Upstream parity maintained by moving the pin, with a ledger that maps every Go file to its Rust counterpart and the pin it was last synchronized to.

### Non-goals

- Changing TypeScript semantics, diagnostics text, emit or the differences documented in `tsc/CHANGES.md`. The Rust compiler targets Corsa, not TypeScript 6.
- Rewriting the JS API client, the VS Code extension or the test data; they are consumed from the pin.
- Adopting a third-party AST or parser. The AST shape is pinned by `ast.json`, the binary encoder and the generated TypeScript AST package.
- Windows, musl, and every best-effort target. The Windows host layer is not built.
- Publishing under upstream package names, code signing, and upstream's release pipelines. Whether and when any of this goes upstream is a later question.
- New features before parity. Rust follows Go until the checker gate.

### Exit criteria for cut-over

- 100 percent of compiler, conformance, transpile, config, tsc, build and watch baselines, modulo the allow-list.
- At least 99.5 percent of fourslash tests, with the remainder triaged and listed.
- The `packages/typescript` sync and async suites, the project and LSP suites and the replay corpus pass against the Rust binary.
- Benchmark and memory targets met on every scenario for four consecutive weekly runs.
- Four weeks of the owner's own projects and the VS Code extension running on the Rust binary with no open P1 crash.

## 6. Architecture decisions

Each decision names what it replaces. The through-line is that Corsa's runtime model is kept wherever it is observable, and only the memory model changes. Decisions 1 to 9 are unchanged from draft 2; the pending technical items in section 13 may amend 1, 2, 3, 6, 8, 10, 12 and 13.

### 1. AST: index-based, generated from `ast.json`, with explicit arena ownership

- **Decision.** Nodes live in arenas of four kinds: a file arena per source file, immutable after binding and shared across programs by reference count; a transform arena per emit context for synthetic nodes; a builder arena per checker node builder for the trees behind `typeToString`, hover and declaration emit; and a session arena for nodes the API creates. A `NodeId` is a 64-bit value that names its arena and its index, so a node can be resolved without knowing where it came from. Node headers (kind, flags, pos, end, parent, payload index) and per-kind payload tables are emitted by a generator in this repository that reads the pinned `ast.json`. Children are node ids and node-list ranges; positions are UTF-8 byte offsets.
- **Rules.** References point only from newer arenas to older ones: a transform or builder node may reference file nodes, never the reverse. Original-node links and emit-node data are side tables in the emit context keyed by id. A builder arena is released only when no cache entry references it, which Rust makes explicit with a reference count held by the caches, where Go relied on the collector. Encoding for the API walks the tree once and converts positions to UTF-16 through the file's position map, as Corsa does; it is not a memory copy.
- **Instead of.** `Rc<RefCell<Node>>` graphs (slow, unergonomic); bump-arena references in the oxc style, `&'a Node<'a>`, which are excellent for parse and transform but push a lifetime into the checker, the language service and long-lived snapshots; adopting oxc's AST outright, whose shape does not match `ast.json`; a single arena per file, which was draft 1's design and has no place for synthetic nodes.

### 2. Symbols, types and signatures: file-owned binding, checker-owned types

- **Decision.** The binder allocates a file's symbols into that file's own arena at bind time, because Corsa binds files in the parse cache before any program exists and shares them across programs and snapshots. A `SymbolId` names the file and the index within it. Programs add an overlay for merged symbols, and each checker owns an arena for the transient symbols it creates, distinguished by a high bit. Types and signatures are checker-owned: `TypeId` and `SignatureId` index the checker's arenas, and every `map[*ast.Node]*Type` and every `LinkStore` becomes a side table keyed by node or symbol id. The `TypeData` and `nodeData` interfaces become enums with per-variant payload tables. Symbol tables stay unordered hash maps keyed by WTF-8 names, as in Go; order is produced by sorting at the observation points (decision 5).
- **Because.** This is the memory model Corsa already uses through `core.Arena` and `LinkStore`, minus the pointers. Dropping a checker frees its whole type universe, which is what the checker pool relies on, and dropping a program never touches a shared file.
- **Verified by.** Three spike scenarios: two programs sharing one bound file; disposal of one of them while the other keeps checking; and an edit that produces a new version of the file while an old snapshot stays alive and answers requests.
- **Instead of.** `Arc<Type>` graphs (cycles leak, an atomic on every access); a program-wide symbol arena, which was draft 1's design and cannot hold a symbol that a cached file carries into a second program.

### 3. Mutation inside the checker

- **Decision.** The checker is a single-threaded state machine that takes `&mut self` and never holds a borrow into its arenas across a call: accessors copy small values out, scalar lazily-resolved fields use `Cell`, collections are addressed by id. `isRelatedTo`, instantiation, inference and control-flow analysis keep Corsa's structure, and keep its order of side effects wherever decision 5 lists the output as order-sensitive.
- **Because.** The port ledger depends on Go and Rust having the same shape so that upstream diffs transplant, and the sort points in decision 5 depend on the same values being present when the sort happens.
- **Instead of.** `RefCell` on every table (runtime borrow panics inside 32,000 lines of recursion); splitting the checker into pure passes, which does not match TypeScript's demand-driven semantics.

### 4. Concurrency: keep Corsa's model exactly

- **Decision.** Parse and bind in parallel with scoped threads. Partition files across checkers with the same FENNEL heuristic and the same tuned constants from `compiler/checkerpool.go`; one checker per thread, `Send` but not `Sync`. Emit per file in parallel. The language server serves requests on `Arc<Snapshot>` values built copy-on-write, with `project/dirty`'s boxes becoming persistent maps. Cancellation is a token polled wherever Corsa polls its context.
- **Because.** These behaviors have baselines and a race-mode CI job, and the partitioning constants were swept on real projects. There is nothing to gain from a different model before parity.
- **Instead of.** A query-based incremental architecture in the rust-analyzer style. Attractive later; incompatible with landing a drop-in replacement.

### 5. Order-sensitive outputs are enumerated, not assumed

- **Decision.** Port `CompareTypes`, `compareSymbols`, `compareNodes` and `CompareDiagnostics` line by line, and sort at exactly the points Corsa sorts. Construct the intrinsic types in Corsa's order so their ids compare the same way. List the residual id-sensitive cases (reverse mapped types without symbol or mapper, symbols without declarations and with duplicate names) and give each a test. Keep a creation-trace mode in both binaries for diagnosing those residual cases, scoped to them rather than imposed on the whole checker.
- **Because.** Corsa's own comment on reverse mapped types calls their id ordering unstable and accepts it as rare. That is the size of the real problem. Demanding identical allocation traces, as draft 1 did, would have constrained every allocation in the checker to fix a problem that the comparator already solves structurally.

### 6. Deep recursion

- **Decision.** Run parsing, checking and emit on threads created with large reserved stacks (256 MB to 1 GB; reservation is virtual and committed lazily on both target operating systems), and add growth guards in the deepest recursive paths.
- **Because.** Go grows stacks on demand up to 1 GB by default (`core.ApplyDebugStackLimit` only lowers it for debugging). A fixed 8 MB stack would introduce a new crash class that Corsa never had. Pending item 12 in section 13 decides whether the removed trampolines come back at the known sites.

### 7. Panics and errors

- **Decision.** Invariants stay `panic!` and `debug_assert!` (Corsa has about 600 panic sites and twelve recovers). Each LSP request, API call and test step runs under `catch_unwind` with the same stack sanitizing as `lsp/stack_sanitizer.go`. Locks are `parking_lot` (no poisoning) and are never held across checker calls. I/O and configuration failures are `Result`. Pending item 11 in section 13 decides what happens to a checker after a caught panic.

### 8. Three string representations, and numbers, Unicode and collation

- **Decision.** Source text is a `str` and positions inside the compiler are UTF-8 byte offsets, which is already Corsa's model. JavaScript string values, meaning string-literal values, identifier names, symbol names, string-literal types, template text and interned atoms if any, are a `JsString` newtype over WTF-8 bytes with a fast path when the bytes are valid UTF-8, because a lone surrogate must round-trip through the scanner, the checker and the encoder as it does in Go. Wire positions are converted at the edge: UTF-16 code units for the API encoder, UTF-16 or UTF-8 for LSP as negotiated, through the same per-file position maps Corsa uses. Port `jsnum` with a shortest-round-trip printer and golden tests against V8. Regenerate identifier tables from the Unicode 15.1 data the scanner uses today. Port the organize-imports comparer as written: it normalizes with NFD, builds natural-number keys and applies the case-first and accent rules itself, ignores the locale preference, and uses no collation library.
- **Because.** The encoder's string section is documented as UTF-8 with WTF-8 for such values, the JS client decodes it accordingly, and the relater and JSX transform special-case lone surrogates. "UTF-8 everywhere", draft 1's wording, cannot represent `"\uD800"` at all.

### 9. Host seams

- **Decision.** Object-safe, `Send + Sync` traits with the same shapes as `vfs.FS`, `compiler.CompilerHost`, the tsc `System` with its fake clock, `checker.Program` and the content-mapper host.
- **Because.** Every harness injects through these seams. Keeping them identical makes the harness ports mechanical and lets one test corpus drive both implementations.

### 10. Generated code stays single-source, generated here from the pinned schemas

- **Decision.** The generators live in this repository and read the schema files from the pinned upstream checkout: `ast.json` to AST, visitor, factory and encoder; `diagnosticMessages.json` to `diagnostics.rs`; the LSP metamodel to a serde `lsproto`. The API surface is captured once from `proto.go` into a declarative `api.json` maintained here, which emits both the Rust dispatcher and the `proto.generated.ts` the JS client needs; the client is built from the pin with that file substituted. The enum tables upstream scrapes from Go source are derived from the pinned Go source by the same generator. Generated output is checked in with a drift check in CI. Embedded libs come from the pin with `include_str!` behind a `noembed` feature; the 13 locale files stay gzipped and are inflated on demand.
- **Because.** The schemas are upstream's and must stay upstream's; this repository cannot add backends to upstream's generators without upstream's cooperation, which is not assumed. Pending item 13 in section 13 decides the generator's implementation language.

### 11. This repository is the workspace; upstream is a pinned submodule

- **Decision.** A Cargo workspace at the root of this repository, one crate per Go package where the boundary carries meaning, tiny leaves merged into `ts_core`. Test-only packages become test crates. microsoft/TypeScript is a git submodule at `upstream/`, pinned to a commit (1f70213d49 today), providing test data, schemas, libs, locales, the JS client, the extension, and the Go source from which the oracle binary is built and against which the ledger is kept. See the crate map below.
- **Because.** Cargo compiles and caches crates independently; a monolithic crate makes the 60,000-line checker the compile-time bottleneck of every change. A submodule keeps 342 MB of test data out of this repository's history while pinning exactly which upstream commit every baseline and schema came from.

### 12. Toolchain and lints

- **Decision.** Stable Rust, minimum supported version one behind current stable. `clippy` at pedantic minus an allow-list. `rustfmt`. `cargo nextest` for the suites, `lld` or `mold` plus `sccache` in CI. `#![forbid(unsafe_code)]` everywhere except the same leaf crates where Corsa uses `unsafe` today (file watching, paths, platform FFI). Release profile: `panic = "unwind"` (required by decision 7), fat LTO, one codegen unit, mimalloc. Pending item 15 in section 13 decides how the custom Go lints are replaced.

### 13. Targets and distribution

- **Decision.** Four targets: `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`. CI runs on macOS and Linux runners and cross-compiles the second Linux architecture. The output layout mirrors upstream's platform packages (`lib/tsc`) so the pinned JS client's `getExePath` and the extension resolve it. Not built: the Windows host layer (named pipes in `ipc`, the Windows watcher and realpath), musl, and every best-effort target. Windows-style path semantics in `tspath` stay because baselines exercise them. Code signing and publishing under upstream names are out of scope. The Linux binaries link against glibc 2.28, the floor of Node 18 and later, and CI rejects any binary whose ELF requires a newer symbol version (item 15 of the pending decisions).

### 14. Dependency policy

- **Decision.** Few, vetted crates, each recorded with the reason it exists. `cargo-deny` enforces an Apache-2.0-compatible license list, bans duplicate versions and known advisories; `cargo-vet` records an audit for every crate; `Cargo.lock` is committed; no build script touches the network; the minimum supported Rust version is checked in CI. The expected direct dependencies mirror the roles of Corsa's twelve Go modules: scoped threads or rayon, parking_lot, mimalloc, serde with a JSON crate matched to the Go JSON v2 usage, an msgpack crate, an xxh3 implementation that produces the same hashes the API encoder header carries today, a patience diff for baselines, flate2 for the locale files, a Unicode normalization crate for the organize-imports comparer, stacker, and the platform crates for file watching on macOS and Linux.
- **Because.** The Go module builds without cgo, without Node and with twelve direct dependencies, and CI proves it. A Rust tree that pulls in hundreds of transitive crates would be a regression in auditability.

## 7. Compatibility contracts

What must hold at cut-over, what pins it, and whether the existing verification can be reused as-is, bridged to the Rust binary, or must be ported.

| Contract | Pinned by | Verified with | Reuse |
|---|---|---|---|
| Type-check results: `.errors.txt`, `.types`, `.symbols` | 12,721 cases; 45,447 baseline files under `compiler` and `conformance`; `union ordering` and parent-pointer sub-tests | Ported compiler runner; Go oracle built from the pin; comparators ported line by line and proved to be total orders by the `union ordering` sub-test | Data as-is; runner ported |
| Emit: `.js`, `.map`, `.d.ts`, source-map records | Same corpus; 41 transpile baselines | Runner sub-tests | Data as-is |
| Command line: `tsc`, `--watch`, `-b`, `--incremental`, pretty output, locales | 517 baselines across `tsc`, `tscWatch`, `tsbuild`, `tsbuildWatch`; `.tsbuildinfo` JSON | Ported `tsctests` harness with fake system and clock | Data as-is; harness ported |
| Configuration parsing | 309 baselines under `config` and `tsoptions` | Ported unit tests | Data as-is |
| Language server over LSP 3.17 | 4,356 fourslash tests; 1,749 fourslash baselines; project and LSP suites; replay corpus | First: the Go fourslash suite, carried as a small patch on the pinned harness, drives the Rust server through a test-host protocol that serves the virtual file system and the test hooks over the connection. Later: suite regenerated in Rust from the original fourslash sources | Test host, then regenerate |
| JS API: msgpack framing, encoder protocol 8 with WTF-8 strings and UTF-16 positions, 144 methods, snapshots, batches, callback FS | `packages/typescript` sync and async suites and benchmarks; `api` baselines; `proto.generated.ts` | The suites spawn whichever binary `getExePath` resolves | Unchanged |
| Content-mapper plugins | JSON-RPC child-process protocol; `spanmap` fidelity; 15 contentmapper baselines | Ported `contentmappertest` | Harness ported |
| Diagnostic text and localization | 2,135 + 85 messages; 13 locales | Baselines; message-format unit tests | Regenerated |
| Semantics relative to TypeScript 6 | `tsc/CHANGES.md` | The corpus; Strada divergences are not "fixed" | Spec as-is |
| Small semantics: Unicode 15.1 identifiers, WTF-8 string values, JS number formatting, the organize-imports comparer, paths, case sensitivity, symlinks | Go unit tests: `jsnum` 1,347 lines, `tspath` 1,258, `vfsmatch` 2,171, `symlinks`, `semver` 1,106; the JS client's `wtf8.test.ts` | Ported unit tests plus differential fuzzing against the Go oracle | Tests ported |

## 8. Strategy and crate map

Strangler fig, with the Go binary as the oracle. The Go oracle is built from the pinned upstream; Rust grows bottom-up in the dependency order of the Go packages, and nothing moves up a tier until the tier below has parity.

- **Separate repository, pinned upstream.** This repository holds the workspace; `upstream/` is the submodule. Moving the pin is a deliberate operation: bump, rebuild the oracle, regenerate from the schemas, list the Go files whose contents changed since their ledger entry, port those diffs, re-run every suite, then commit the new pin.
- **Every crate lands with its tests.** The Go unit tests for the package are ported, and where output is observable a differential test runs both binaries.
- **Three oracle seams from week one.** Parser parity: encode every test input and lib file with both binaries' API encoders and compare bytes. Editor parity: the Go fourslash harness does not talk to a server over stdio; it builds an in-memory file system with symlinks and configurable case sensitivity, injects a parse cache and plugin spawners, sets inferred-project options through a test hook and waits on an in-process initialization signal. Reusing it against a Rust server therefore needs a test-host protocol: the Rust server in test mode serves file-system calls from the harness over the connection, the way the API's callback file system already serves `readFile`, `fileExists`, `directoryExists`, `getAccessibleEntries` and `realpath`, plus test-only methods for inferred-project options, initialization and plugin spawning. The harness side is a small patch carried against the pinned upstream. It is validated first on representative case-sensitivity, symlink and plugin tests, before any claim about the other 4,300. API parity: the TypeScript test suite already spawns the binary.
- **A port ledger.** `PORTS.toml` maps each upstream Go file to its Rust module and the pin it was last synchronized to. Bumping the pin produces the list of files to re-port before the bump lands.
- **Agent-assisted translation with a hard oracle.** The mechanical tiers are good candidates for agent-driven translation, exactly as the Go port used. The rule is that nothing merges without its baselines, and the checker's data-layer redesign is human-led.
- **Parity gates instead of a calendar.** Section 9 orders the work by dependency; each phase ends in a gate that is a parity check, and the only stop-or-go point is the spike.

### Crate map, leaves first

Tiers follow the topological order of the 82 Go packages, verified with `go list` (see `data/topological-order.txt`). The phase column is the phase in which a crate reaches parity. Crates marked "generated" are emitted from a schema.

| Tier | Crates | Phase |
|---|---|---|
| 0, leaves: no compiler knowledge | `ts_core`, `ts_collections`, `ts_tspath`, `ts_stringutil`, `ts_jsnum`, `ts_json`, `ts_locale`, `ts_glob`, `ts_semver`, `ts_packagejson`, `ts_vfs`, `ts_vfs_os`, `ts_vfs_match`, `ts_diagnostics` (generated), `ts_bundled` (generated) | 1 |
| 0, leaves: contracts the AST depends on | `ts_jsstring` (WTF-8 values and position maps), `ts_arena` (arena ids, reference counting, cross-arena rules) | 0 |
| 0, leaves: platform | `ts_fswatch` (FSEvents, inotify, fanotify), `ts_nativepath` | 4 |
| 1, syntax: source text to bound trees | `ts_ast` (generated), `ts_scanner`, `ts_parser`, `ts_encoder` (generated) | 0 |
| 1, syntax | `ts_binder`, `ts_astnav`, `ts_evaluator` | 1 |
| 2, semantics: resolution | `ts_module`, `ts_tsoptions` | 1 |
| 2, semantics: types | `ts_modulespecifiers`, `ts_checker` | 2 |
| 2, semantics: output trees | `ts_pseudochecker`, `ts_sourcemap`, `ts_printer`, `ts_transformers`, `ts_declarations` | 3 |
| 3, programs: compile, build, watch | `ts_outputpaths`, `ts_transpile` | 3 |
| 3, programs | `ts_compiler`, `ts_incremental`, `ts_build`, `ts_execute`, `ts_diagnosticwriter`, `ts_tracing`, `ts_pprof`, `tsc` (binary) | 4 |
| 4, editor: language service and projects | `ts_format`, `ts_ls`, `ts_autoimport`, `ts_lsproto` (generated), `ts_project`, `ts_ata`, `ts_contentmapper`, `ts_spanmap` | 5 |
| 5, servers: processes and protocols | `ts_lsp`, `ts_testhost` (the test-host protocol, test builds only) | 5 |
| 5, servers | `ts_ipc` (unix sockets), `ts_jsonrpc`, `ts_api` | 6 |
| 7, deliverables beyond parity | `ts_wasm` (the `wasm32` entry points), `ts_embed` (the embeddable crate API) | 7 |
| Test crates, not shipped | `ts_testutil`, `ts_testrunner` | 1 |
| Test crates | `ts_tsctests` | 4 |
| Test crates | `ts_fourslash`, `ts_projecttest` | 5 |

## 9. Sequence and gates

Phases are ordered by dependency, not by time. Anything a phase does not depend on can run at the same time as it; the table says what each phase needs before it can start and what proves it done. The spike is the only stop-or-go point.

| Phase | Needs first | Can run alongside | Gate |
|---|---|---|---|
| 0. Spike | The three contracts | Nothing; it goes first | E1 to E8 |
| 1. Foundations | 0 | Its own tail overlaps 2 | All cases parse identically; binder dumps match; config baselines; test-host protocol on representative tests |
| 2. Type checker | 1's AST, binder, module resolution and options | 3, 4, 5 | 100 percent of `.errors.txt`, `.types`, `.symbols` |
| 3. Emit | 1; the declaration transform needs 2's emit resolver | 2, 4, 5 | 100 percent of `.js`, `.map`, `.d.ts`, transpile |
| 4. Programs and CLI | 1 for the driver, build and watch; 2 for full checking | 2, 3, 5 | All tsc, watch, build baselines; smoke test; sanitizer run |
| 5. Editor | 1 for the test host, framing and project design; 2 for the language service | 2, 3, 4 | 99.5 percent fourslash; project, LSP, replay suites |
| 6. JS API server | 2 and 5's project system | 5's tail | `packages/typescript` suites pass unchanged |
| 7. Hardening, WebAssembly, embedding, cut-over | All of the above | Nothing | Every exit criterion for four consecutive weeks |

### Phase 0: spike

- **Scope.** Workspace skeleton with the `upstream/` submodule, CI on macOS and Linux, lint and format. The three contracts from section 1, item 8, written and reviewed before any code is generated. Rust AST generated from the pinned `ast.json`; scanner, parser and JSDoc reparser; the API encoder. A checker vertical slice on the decision 2 and 3 data model: literal, object, union, intersection, array and tuple types; `getTypeOfSymbol`; assignability through `isRelatedTo`; a subset of narrowing; `typeToString`; the four comparators ported. Eight experiments, each with a threshold fixed in advance; memory, WebAssembly and embedding are all primary.
- **Experiments.**

| | Measures | Threshold | Nature |
|---|---|---|---|
| E1 Parser parity | Encoder-identical parses of 12,721 cases and 108 lib files | At least 99.9 percent | Measured |
| E2 Checker slice | `.types` and `.errors.txt` on the curated conformance subset; comparators proved total orders by the `union ordering` sub-test | 100 percent on the subset | Measured |
| E3 Ownership | Two programs sharing a bound file; dropping one; an edit while an old snapshot answers requests; a builder-arena release with a live cache entry | No dangling ids under Miri and AddressSanitizer; memory returns to baseline after each drop | Measured |
| E4 Strings | Every string literal in the corpus, including lone surrogates, through scanner, literal types and encoder; positions in UTF-8 and UTF-16 | Byte-identical to the Go encoder's output | Measured |
| E5 Memory | Peak RSS and bytes allocated for parse and bind of the VS Code repository, Rust against Go; the slice's per-type footprint on the subset | Parse and bind at most 70 percent of Go; per-type at most 80 percent | Measured for parse and bind; extrapolated for checking |
| E6 CPU | Parse and bind wall time on the VS Code repository at 1 and 8 threads | At least Go's at both | Measured for parse and bind; extrapolated for checking |
| E7 WebAssembly | Parser crate built to `wasm32`; binary size and parse throughput in Node against a Go `GOOS=js` build of the same entry point | At most 25 percent of Go's size; at least twice its throughput | Measured for parsing only |
| E8 Embedding | A Node in-process binding for parse-and-encode against the same request over the `--api` socket | In-process latency at most 10 percent of the socket path for a 10 KB file | Measured for parsing only |

- **Gate.** E1 to E4 pass; E5 to E8 meet their thresholds for the parts they measure, with the extrapolations written down as extrapolations. Otherwise stop.

### Phase 1: foundations

- **Scope.** `core`, `collections`, `tspath`, `stringutil` with regenerated Unicode tables, `jsstring`, `jsnum`, `json`, `locale`, `glob`, `semver`, `packagejson`, the `vfs` family including `vfstest`, generated `diagnostics`, `bundled`, the complete `ast` with all four arena kinds, `scanner`, `parser`, `binder` with file-owned symbol arenas, `astnav`, `evaluator`, `module` resolution and `tsoptions`. Harness: baseline diffing with a patience diff, the test-case directive parser, the compiler runner limited to parse and bind sub-tests and syntactic `.errors.txt`. The test-host protocol and LSP framing, so that the Go fourslash suite can be pointed at a Rust server as soon as one answers requests. Generator drift checks like upstream's `generate` job.
- **Gate.** All 12,721 cases parse identically; binder output matches through a debug dump of symbol tables and flow graphs produced by both implementations; the 309 config and options baselines pass; the test-host protocol runs a representative set of case-sensitivity, symlink and plugin fourslash tests against the Rust server.

### Phase 2: type checker, the critical path

- **Scope.** In an order where each step unlocks a slice of baselines: symbol resolution, declared and inferred types, literal, enum, tuple and array types; the relater (assignable, subtype, strict subtype, comparable, identity) and variance; signatures, overloads, inference and contextual typing; control flow and narrowing from `flow.go`; mapped, conditional, template-literal and indexed-access types with instantiation, mappers and depth limits; classes, interfaces, enums, namespaces, late binding, declaration merging and the JavaScript semantics in `CHANGES.md`; JSX; grammar checks and all 2,220 diagnostics; the node builder behind `typeToString`, `typeToTypeNode` and hover, with its builder arena and release protocol; the emit resolver and the services surface in `services.go` and `symbolaccessibility.go`; the checker pool, parallel checking, cancellation and `--generateTrace`.
- **Tooling.** A burn-down harness that runs both binaries per case, aligns `.types`, `.symbols` and `.errors.txt`, buckets differences by diagnostic code and checker area, and publishes a pass-rate dashboard per area. The creation-trace diff from decision 5 for the residual id-sensitive cases.
- **Gate.** 100 percent of compiler and conformance `.errors.txt`, `.types` and `.symbols` baselines with an approved allow-list; `union ordering` and `source file parent pointers` sub-tests green.

### Phase 3: emit

- **Scope.** `printer`, `sourcemap`, the TypeScript, ES, module, JSX and inliner transforms with their transform arenas and original-node side tables, the declaration transform (which depends on the emit resolver and `pseudochecker`), `outputpaths` and `transpile`; the `.js`, `.map` and `.d.ts` sub-tests and the transpile runner. Starts once the parser and binder are stable and finishes only after the emit resolver exists.
- **Gate.** 100 percent of `.js`, `.map`, `.d.ts` and source-map-record baselines and the 41 transpile baselines.

### Phase 4: programs, command line, build and watch

- **Scope.** `compiler` (program, file loader, project references, checker pool, emit orchestration); `execute/tsc` with pretty diagnostics, colors and locales; the `-b` orchestrator; `incremental`, reading and writing Corsa's `.tsbuildinfo` so both binaries can share build state; the watch manager and `fswatch`, porting the FSEvents, inotify and fanotify backends rather than substituting a generic crate, because their behaviors are baselined; `diagnosticwriter`, `tracing`, `pprof`; the `tsctests` harness with its fake system and clock.
- **Gate.** All 517 tsc, watch, build and build-watch baselines; the smoke test compiles upstream's fixture project single- and multi-threaded; a ThreadSanitizer build mirrors upstream's race-mode job; the `--singleThreaded` and concurrent-test-programs configurations both pass.

### Phase 5: language service, project system, LSP server

- **Scope.** First the test-host protocol, JSON-RPC framing, the request queue with cancellation, and the project system's snapshot and ownership design against decisions 1 and 2. Then the language service in order of fourslash weight: completions including string completions and the auto-import registry; hover; the definition family; references, highlights, rename and file rename; signature help; diagnostics; code actions and fixes; organize imports; inlay hints; call hierarchy; code lens; folding; selection ranges; semantic tokens; linked editing; document and workspace symbols; JSDoc snippets; `format`. Project system: snapshots, config-file registry, inferred projects, cross-project references, automatic type acquisition, overlay file system, parse cache with file-owned binding, the service-side checker pool, background work and logging. Server: progress, client and native watchers, the API-over-LSP connection, the stack sanitizer.
- **Testing.** The Go fourslash suite drives the Rust server through the test-host protocol from Phase 1 onward, giving a live pass rate against all 4,356 tests without translating one, with the caveat that the protocol is validated on representative file-system and plugin tests before the number is quoted. Then regenerate the suite in Rust from the original fourslash sources, the way the Go tests were generated, rather than translating Go to Rust. Replay files from the LSP replay corpus run against both servers.
- **Gate.** At least 99.5 percent fourslash with a triaged allow-list; project and LSP suites; replay corpus; no request-latency regression against Go on the benchmarking editor scenarios.

### Phase 6: JS API server

- **Scope.** `ipc` over unix sockets with both sync and async connections; msgpack framing; the generated encoder at protocol 8 with WTF-8 strings and UTF-16 positions; the 144 methods with snapshot, temporary-snapshot, batch, pagination and disposal semantics; the session arena for API-created nodes; the callback file system; the schema-driven `proto.generated.ts` and enum generation for the pinned TypeScript client.
- **Gate.** The `packages/typescript` sync and async suites and benchmarks pass unchanged against the Rust binary; `api` baselines.

### Phase 7: hardening, WebAssembly, embedding, cut-over

- **Scope.** The TypeScript-benchmarking scenarios at 2, 4 and 8 checkers; allocation and layout tuning; fuzzing of scanner, parser and formatter, differential against the Go oracle; the crash replay corpus; the four-target build and a binary-size budget. The `wasm32` build of the whole compiler and the embeddable crate API, both prototyped in E7 and E8, become deliverables here. Then the owner's projects and the extension run on the Rust binary.
- **Gate.** Every exit criterion in section 5 holds for four consecutive weeks.

## 10. Risk register

| Rank | Severity | Risk | Evidence in the repository | Mitigation |
|---:|---|---|---|---|
| 1 | High | The checker cannot be made to fit the borrow checker without changing behavior | 320-field `Checker`, 26 pointer-keyed link stores, function-valued fields, mutation anywhere through `*Checker`; the reason Go was chosen | Decisions 2, 3 and 5; the Phase 0 slice before committing; structure-preserving translation; comparators ported line by line |
| 2 | High | Upstream outruns the port | 113 to 237 commits a month; thousands of commits over the life of the port | Pin bumps at a cadence; the ledger lists exactly what to re-port; Go names kept in Rust; agent-assisted diffs with baselines as the gate |
| 3 | High | Ownership contracts fail under real sharing | Files are bound in the parse cache and shared across programs and snapshots; builder arenas are released while caches may still hold nodes | Decisions 1 and 2; experiment E3 in the spike with Miri and AddressSanitizer; ownership scenarios kept as permanent tests |
| 4 | High | The benefits do not materialize | Corsa already took the large win; Go's checker pool and arenas are tuned | Experiments E5 to E8 with thresholds, all three benefits primary; extrapolations labeled; stop at the spike gate if they fail |
| 5 | Medium | Stack overflow as a new crash class | Go stacks grow to 1 GB; Corsa removed the old trampolines and has no depth guards beyond TypeScript's own limits | Decision 6: large reserved stacks and growth guards; pending item 12 on trampolines; fuzz deep nesting |
| 6 | Medium | Editor parity stalls on the long tail | 4,356 fourslash tests, 41,500 lines of service code, many features with few tests each | Test-host protocol from Phase 1 for a live pass rate; regenerate tests from fourslash sources; allow-list with the owner's approval |
| 7 | Medium | The test-host protocol underestimates the harness | The harness injects an in-memory file system with symlinks and case sensitivity, a parse cache, plugin spawners, an inferred-options hook and an in-process init signal | Validate on representative tests before quoting coverage; carry the harness patch against the pin; regenerate the suite in Rust as the durable path |
| 8 | Medium | WTF-8 values leak into `String` and are corrupted | Lone surrogates are stored as WTF-8 sentinels in Go strings and round-trip through the encoder | Decision 8; `JsString` newtype with no lossless conversion to `String`; experiment E4; lint against `String` in symbol and literal APIs |
| 9 | Medium | Small-semantics drift | JS number formatting, Unicode 15.1 tables, the organize-imports comparer, path and symlink rules each have baselines | Decision 8; port the Go unit tests first; differential fuzzing |
| 10 | Low | Compile times slow iteration | 60,000-line checker crate; generic-heavy code | Crate split per decision 11; sccache; lld or mold; a compile-time budget in CI |
| 11 | Low | The generators fall behind upstream schema changes | `ast.json`, the LSP metamodel and the diagnostics file change upstream; `api.json` is maintained here | Generation is part of every pin bump; the drift check fails the build until it is regenerated |

## 11. Consumed from upstream, not rewritten

Everything below comes from the pinned submodule and is used as-is.

- **Test data and baselines** under `tsc/testdata`: 342 MB of language-agnostic inputs and expected outputs, reused byte for byte.
- **`packages/typescript`**, the npm package and JS API client, built from the pin with the regenerated protocol file substituted and pointed at the Rust binary.
- **`packages/vscode-typescript`**, pointed at the Rust binary.
- **Schemas and data**: `ast.json`, the LSP metamodel, `diagnosticMessages.json`, the 108 lib files, the 13 locale files. This repository's generators read them.
- **The Go source**, from which the oracle binary is built and against which the ledger is kept.
- **`CHANGES.md`** as the specification of Corsa's intentional divergences from TypeScript 6.

Upstream's build file, release pipelines, signing tools and the small Go module under `tools/` are not used. The custom Go linters, `gen-proto` and the enum scraper are replaced by this repository's generators and lint configuration.

## 12. Alternatives considered

- **Hybrid: Rust only for scanner, parser and emit behind an FFI.** The parser is not the bottleneck, the FFI boundary would cross on every node access, and two toolchains would ship in one binary. Rejected.
- **Adopt an existing Rust AST and parser** such as oxc's. The AST shape is pinned by `ast.json`, the binary encoder and the generated TypeScript AST package, and thousands of checker and service code paths are written against it. Borrow the allocator and layout ideas; do not adopt the types. Rejected as a dependency.
- **Rewrite from the TypeScript 6 source rather than from Go.** The Go tree is the current semantics, the baselines are Corsa's, and the Go port already answered every question about JavaScript-specific idioms. Rejected.
- **A query-based incremental architecture** in the rust-analyzer style. The right long-term shape for an editor service, and incompatible with landing a drop-in replacement on a moving target. Deferred until after cut-over.

## 13. Open questions and pending technical decisions

Open until a measurement or the owner settles them:

1. **The spike's curated subset.** Pending item 16 below.
2. **Performance budgets per benchmark scenario.** Set after experiments E5 and E6, not before them.
3. **Where the WebAssembly build and the embedding crate API sit.** Both are deliverables; whether they land inside Phase 7 or as the first work after cut-over.
4. **The residual id-sensitive cases in practice.** Whether reverse mapped types without symbol or mapper, and undeclared duplicate-name symbols, ever change observable output in the corpus. Measured in the spike.
5. **The fourslash harness patch.** Whether the test-host transport is carried as a patch against the pinned Go harness or the harness is forked into this repository.

Pending technical decisions, revised after a second review; the recommendation on the table for each:

6. **Storage layout for nodes and types.** An experiment, not a decision. Generated accessors are independent of the representation, so the layout can change without touching callers. A fully inline enum sizes every slot to its largest variant, so large payloads (source files, JSDoc, classes) are boxed or split out from the start. E5 and E6 measure parsing and binding; the layout of types is chosen only after the checker slice reports throughput, allocations and retained memory. One generator with a representation switch, one production output at a time.
7. **Lists, borrows and reentrancy.** Mutable builders, immutable published lists: Go sorts, compacts and modifies type and signature lists while constructing them, then never again. Published lists are independently owned handles, `Arc<[TypeId]>` rather than `Rc`, because the checker must be `Send`; a slice borrowed from `self` would still block a recursive `&mut self` call. Lazy resolution keeps Corsa's `pushTypeResolution` and `popTypeResolution` guard, since `getTypeArguments` and its peers can re-enter themselves. The arena-references-with-interior-mutability alternative is prototyped on the relater core in the spike, so that it is a measured fallback rather than an assumed one.
8. **Interning.** Deferred, with a corrected rationale: files outliving programs does not force an immortal process-global interner; file-owned interners, session-scoped interning and reclaimable shared entries are all available later. Start without interning, but never deep-copy bytes where Go copies a string header: `JsString` values are cheaply clonable shared immutable buffers. Measure string duplication separately from interning before deciding.
9. **Threads versus async.** Synchronous compiler and service APIs. Not a thread per goroutine: bounded worker pools for parse, bind, check and emit, and transport readers that progress independently of workers, because the API transport handles responses while requests are executing and a blocked worker must not stall cancellation or callback traffic. An async transport is allowed; it does not spread into the service.
10. **Panic policy.** Eviction of the panicking checker is necessary but not sufficient. The API checker deliberately persists so that type and symbol handles keep reference identity; a rebuilt checker would reuse sequential ids while old handles are still registered, and registration panics on such a collision. A caught panic therefore invalidates the owning snapshot and project registry with a generation bump, so stale handles fail explicitly and wire-compatibly instead of aliasing. Shared state touched before the panic is discarded with the snapshot. Rebuilding a checker is a full initialization, globals and lib types included; arenas make disposal cheap, not the rebuild.
11. **Deep recursion.** Reserved stacks and growth guards in general, plus trampolines at every site that recurses on the left operand: the binder, `checkBinaryLikeExpression`, the transforms and the printer. The stress tests cover the full parse, bind, check, transform and emit path, left- and right-associative chains, and deep nesting of parentheses, JSX and conditional types; native and `wasm32` stack strategies are validated separately, because WebAssembly stacks do not grow.
12. **Generator language.** The pinned upstream resolver in `tools/scripts/tsc/schema.ts` is the only interpreter of `ast.json`: it already resolves inheritance, aliases and kind expansion, and a second reader would disagree silently. A small script in this repository runs that resolver from the pin and exports normalized schema data; the Rust emitter consumes only that. The same applies to the LSP metamodel generator.
13. **The API schema during the overlap.** One authority: the Go dispatch code, from which the pinned extractor derives contracts, annotations, serialization tags and special mappings. `api.json` is exported by that extractor (a patch on the pin adds a JSON output), and Rust consumes it. The plan states which DTOs and dispatch metadata are generated and which codecs stay handwritten. Ownership moves to this repository only after the regenerated TypeScript is byte-identical and wire fixtures pass.
14. **Custom lints.** Upstream registers seven analyzers. `bitclear`, `shadow` and `cleanup` are Go- or Go-testing-specific and moot; `unexportedapi` is rustc's private-interfaces lint; `emptycase` is clippy configuration. `forbidparentaccess` becomes structural, transform crates get a node API without a parent accessor, with the declarations transform exempt as upstream exempts it. `checkchildren` is a compiler-specific analysis that flags returns which skip child checks and would make diagnostics depend on traversal order; clippy cannot express it, so a replacement is budgeted, a `rustc_driver`-based lint or dylint, chosen in the spike.
15. **Linux glibc floor: decided, 2.28.** musl is out of scope, so each Linux architecture ships one glibc binary linked against glibc 2.28, the floor of Node 18 and every later release line, which excludes Node 16's 2.17 even though the pinned client still declares Node 16.20 as its minimum. In distribution terms that is Debian 10, Ubuntu 20.04 and RHEL 8 or newer, and it excludes Amazon Linux 2. CI inspects the versioned symbol requirements of the final ELF and fails on anything above 2.28, and the suites run on a glibc 2.28 image. Static musl remains the measured alternative if a single universal Linux binary is ever wanted.
16. **The spike subset.** A checked-in rule applied across directories by feature criteria, including `conformance/controlFlow` since narrowing is in the slice, producing a manifest with exclusions and option combinations that is frozen before implementation so failures cannot disappear by changing eligibility. Mandatory ownership, recursion and reentrancy cases sit alongside it.

## 14. First steps

1. Write the three contracts, node ownership, string representations and file-owned binding, as short design notes with the Go evidence attached; the owner reviews them before any AST code is generated.
2. Set up this repository: the Cargo workspace skeleton, the `upstream/` submodule pinned at `1f70213d49`, CI on macOS and Linux, clippy, rustfmt, and a task runner for generation and the oracle build.
3. Build the Go oracle from the pin. Write the generators that read the pinned `ast.json` and `diagnosticMessages.json` and emit the AST, visitor, factory, encoder and diagnostics, on top of the `jsstring` and `arena` crates.
4. Build the parser-parity harness: encode every test input and lib file with the oracle's API encoder, keep the bytes as fixtures, and make the Rust scanner and parser converge on them (E1 and E4).
5. Start the checker slice on the arena and id model from decisions 2 and 3, port the four comparators, and set up the ownership scenarios of E3 as permanent tests.
6. Design the test-host protocol and prototype the callback file system over the LSP connection with three fourslash tests, one case-insensitive, one with symlinks, one with a content-mapper plugin, carrying the harness change as a patch against the pin.
7. Create the allow-list file with the owner as approver, and the ledger format.
8. Run E5 to E8 and record the numbers, good or bad, with the extrapolations marked as such.

---

Numbers were measured on commit `1f70213d49` of microsoft/TypeScript (5 September 2026), excluding `testdata`, `node_modules` and test files unless stated. The package dependency order was produced by `go list` with Go 1.27.1 and is stored in `data/`. Draft 3 changes relative to draft 2: an independent repository with upstream pinned as a submodule; four targets and no Windows host layer; memory, WebAssembly and embedding all primary, with the last two becoming Phase 7 deliverables; the owner approves divergences; the staffing table, calendar, velocity experiment and funding language are removed, and the sequence is expressed as dependencies and gates. Draft 3.1 revises technical items 6 to 16 after a second review: the ICU collation assumption is removed in favor of porting the organize-imports comparer, the binder joins the trampoline sites, the API schema and AST generator keep upstream's extractors as the only interpreters, panic recovery bumps handle generations, and the spike subset is frozen as a manifest.
