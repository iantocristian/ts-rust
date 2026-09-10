# S07-bis: refresh the paired acceptance control

Status: complete; retained as the next comparison control. This precedes the
[combined page/text/helper experiment](S07-bis-pages-text-helpers.md).

The six binder source mappings were restored in `be74975`. The first standard
graph attempt failed because its sandbox could not write the existing Go cache.
The cache-enabled attempt passed all 13,094 files in both modes, with unchanged
counts and input identity. Its evidence record is
`8b8579f26caac8868b77e6d683d86e6c4080d8de8cdae5926c5f3afcf0d3a778`.

The following timing capture correctly stopped before sampling because its
freshly rebuilt Rust executable differed from the graph executable. A second
normal build with matching target/intermediate environment also differed.
Comparing the two available normal executables finds 123 changed bytes in seven
ranges: mimalloc's build-time string, the temporary output path, Mach-O UUID and
code-signature data. Both executables are 6,655,928 bytes. The pinned mimalloc
source prints `__DATE__` and `__TIME__` in its build diagnostic. A fresh build is
therefore not a byte-identical substitute for an already validated graph binary.

Keep exact binary checks. The producer repair consumes the Go and normal Rust
executables built for the source/configuration-bound graph pass, then builds the
allocation variant separately. Validate identities before and after capture;
changed sources, configuration, graph receipts or executables must fail closed.
The prior failed timing artifacts and raw graph observations remain preserved.
No failed or incomplete attempt supplies a performance sample or metric.
The experimental runner already builds once and uses the same frozen executable
for graph validation and its Rust/Rust screen. This repair does not invalidate
those earlier immutable candidate/control observations.

The renewed graph and timing capture will use the repaired producer and its own
source identity. Previous controls, experimental screens and the failed refresh
remain historical evidence; none is overwritten or relabelled as the new run.

## Completed refreshed control

The repaired producer passes both full graph modes and the complete 56-sample
acceptance batch. Its fixed stopping rule requested no extension. Source and
configuration remain unchanged throughout; verification and both E5/E6 consumers
pass. This is a valid capture with failing performance gates.

| Metric | Workers | Go median | Rust median | Rust / Go |
| --- | ---: | ---: | ---: | ---: |
| Wall seconds | 1 | 3.120026583 | 3.985223917 | 1.277304 |
| Wall seconds | 8 | 0.730155083 | 1.037084958 | 1.420363 |
| Requested bytes | 1 | 2,907,465,936 | 2,243,575,625 | 0.771660 |
| Requested bytes | 8 | 2,908,244,312 | 2,243,578,132 | 0.771454 |
| Peak RSS bytes | 1 | 3,154,821,120 | 2,318,958,592 | 0.735052 |
| Peak RSS bytes | 8 | 3,165,896,704 | 2,322,284,544 | 0.733531 |

CPU 95% ratio intervals are 1.244227–1.299550 and 1.033938–1.456533.
The eight-worker interval is wide and is retained as measured. Timing relative
MAD remains below 2.12% for each runtime/mode. The producer
`stable` field also requires a CPU upper bound at or below 1.0; it is false here,
not a claim that the sample dispersion alone failed. All final gates remain open.

The frozen revision is `eaf50b754a023c081efa902247f96e77ff6d0e4f`. Control bundle:
`target/s07-bis/pages-text-helpers-control`, externally recorded manifest SHA
`957421942258d954765fc88b494b2031982dfac849d9358850dfdba7078edcd4`. Its source fingerprint is
`018042a3d0dbaac4a5ba55c0961a5d9f1e41cdddc69d6accd0bf7cfe3ff2858b`.

- allocation executable: `25e71cdad6421e91492ab081f6cb29706ff28267dd7b83b071c42faaedf9337b`.
- go executable: `35172610a07a040b41d11aa7771b2f604120b0c2eca04044cb6672aa93cb0cf4`.
- normal executable: `ee5c0162a556898439896309cc55bee2b93a7715f8ada52c9285a42b974e44aa`.

Fresh evidence: graph `86651e2b7aab76b5cfeea9b1e42ed92d540ee9d2dd439ad55a7dfa566d1dd305`,
E5 `db8056ff57887bd1014878cad024e52e2ba8731b9ea3275d9ecddc02aa49e099`,
E6 `b9e13035971f0c440ad5733644fa85df18f55b9298251168cf968983bb5dc0e2`.
The native report hash is `11d5ded2c24c35d56d5cf625e0553e0eae913264713e94ac90123b70c35c532b`.

The raw graph and benchmark directories, external identity record, failed
attempts and logs are retained in `target/s07-bis/current-control-refresh`.
The six source comments and producer repair change source identity; the Rust
implementation otherwise matches the previously accepted local/text control.
No candidate storage/helper code entered this baseline.
