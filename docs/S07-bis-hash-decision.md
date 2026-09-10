# S07-bis: canonical-name hashing decision

Status: tested in the combined empty-drain/hash candidate at `187d956`.
Correctness passed, but the [fixed screen](S07-bis-drain-hash-result.md) did not
establish a reliable improvement. The candidate was removed and the existing
`hash_one` implementation restored. The rationale below records the tested
design; no hasher replacement is currently selected.

## Decision and scope

Keep `std::hash::RandomState` in each result-owned `NamePool`. Replace its
`hash_one(bytes)` calls with one private `hash_name_bytes(&RandomState, &[u8])`
function that builds the existing keyed hasher, writes the complete selected
bytes once and finishes it. Both ordinary lookup/insertion and the name-pool
growth callback use that function. Symbol-table rehashes already go through the
pool, so they use the same mapping.

This is a narrower candidate than an algorithm replacement. On the pinned
Rust 1.97.1 implementation and current 64-bit native targets, slice hashing
feeds an eight-byte length prefix before the bytes. Removing that prefix
removes one input compression block;
all byte blocks and finalization remain. The old profile's 132 ms of sampled
hashing work is an opportunity size, not an expected saving. The new CPU
profile and combined pipeline screen decide whether the change matters.

The following remain unchanged: arbitrary-byte equality, per-owner random
keys, nullable values, empty-name identity, full/compact symbol identities,
collision equality checks, growth policy, ownership, and unordered iteration.
The public `SymbolTable = HashMap<JsString, Option<SymbolId>>` construction
facade and every other `RandomState` in the repository stay unchanged. No hash
value is persisted or exposed as a semantic identifier. The private table
iteration order can change; differential graph checks retain their existing
qualifications and sorting boundaries.

## Collision policy and the whole-key boundary

Rust selects its default hash algorithm for HashDoS resistance and initializes
`RandomState` with random keys; its documented algorithm is currently
SipHash-1-3, subject to change. This candidate continues to obtain the hasher
from that same owner state. It does not replace the secret with a fixed seed,
derive it from names, or infer protection merely from randomness.
([HashMap documentation](https://doc.rust-lang.org/std/collections/struct.HashMap.html),
[RandomState documentation](https://doc.rust-lang.org/std/collections/hash_map/struct.RandomState.html))

The omitted prefix separates adjacent fields in composable `Hash` values.
This helper has a narrower domain: exactly one complete byte string, immediately
finalized, with no preceding/following fields and no generic `Hash` impl.
Concatenated tuples such as `(ab, c)` and `(a, bc)` are outside that domain.
Adding a composed key later requires an explicit domain encoding; calling this
helper once per field does not supply one. All paths make the same single
`write` call for equal selected bytes, including borrowed ranges.
([Hash prefix-collision contract](https://doc.rust-lang.org/std/hash/trait.Hash.html#prefix-collisions),
[Hasher contract](https://doc.rust-lang.org/std/hash/trait.Hasher.html))

The pinned standard-library source was read locally, rather than assuming that
the `Hasher` trait promises length-aware padding for every implementation:

- `library/std/src/hash/random.rs`: `RandomState::build_hasher` constructs
  `SipHasher13` from the owner's keys.
- `library/core/src/hash/mod.rs`: `[T]::hash` writes the length prefix, then
  its contents; for byte slices the contents are a byte write.
- `library/core/src/hash/sip.rs`: `write` processes complete blocks and keeps
  the tail and message length; `finish` incorporates the length's low byte
  and tail before the compression/finalization rounds.

Thus the candidate still hashes a full variable-length SipHash message,
including finalization; it does not pad different lengths identically or
replace the message with its length. Retaining SipHash's byte-message behavior
is the security rationale, not the finite tests below. The standard library
may change algorithms, so revisit this narrow raw-byte assumption at a toolchain
upgrade; no exact hash value is a compatibility promise.
([Standard-library SipHash source](https://doc.rust-lang.org/src/core/hash/sip.rs.html))

## Rejected alternatives and dependency accounting

- **Foldhash:** its authors explicitly describe limited DoS resistance and
  exclude interactive attackers learning state through observations. Selecting
  it would change the collision policy for compiler/server input, so it is not
  this candidate. A random per-owner seed does not resolve that difference.
  ([Foldhash security discussion](https://docs.rs/crate/foldhash/0.2.0/source/README.md))
- **aHash:** offers a different keyed algorithm and platform-dependent
  implementations. That requires a separate algorithm/source review and new
  dependency closure; its keyed-hash claim alone does not establish equivalence
  to the current policy. The bounded experiment can remove a known redundant
  block while retaining the existing implementation, so no aHash version or
  dependency is selected here.
  ([aHash author documentation](https://github.com/tkaitchuck/aHash/blob/master/README.md))
- **Fixed/non-keyed integer or byte hashing:** changes the existing protection
  against attacker-chosen source names; rejected without an explicit policy
  change. Caching one extra hash per canonical name is also outside this
  candidate because it adds retained storage instead of removing the selected
  per-hash operation.
- **Vendored SipHash or a bespoke keyed hash:** unnecessary maintenance and
  security-review surface for functionality already in the pinned standard
  library. No implementation is copied into this repository.

There are **no new dependencies, transitive versions, licenses, build scripts,
native target requirements or Cargo-vet audit entries**. `Cargo.toml`,
`Cargo.lock`, `deny.toml` and supply-chain policy remain unchanged. The helper
uses stable standard-library APIs available before the workspace's Rust 1.96
minimum. The implementation continues to forbid local unsafe code. Existing
[ADR 0017](adr/0017-dependency-policy.md), dependency-policy checks and exact
MSRV checks still apply to the combined candidate; this is not a claim of a
new dependency audit or a cryptographic proof.

## Counterexamples and verification

The existing symbol-table suite already checks deliberately colliding names,
deliberately colliding table entries through growth/full promotion, absent
versus null values, ownership/stale identities, and byte-equal selected ranges
after input release. Its growth test detects a missed hashing change in the
rehash callback.

The added regression inserts, looks up, enumerates and removes whole byte keys
at empty, block/tail and 255/256-length boundaries, with embedded/trailing NUL,
`0xff`, malformed UTF-8 and shared-prefix names. It compares the results with
the unchanged public standard-map facade across pool/table growth. These are
correctness counterexamples to boundary loss; neither passing them nor forced
collision equality tests establishes resistance to finding hostile collisions.

After the current-control profiling window, run the affected tests in debug,
release and the existing ownership/instrumentation paths, strict Clippy, Rust
1.96 checks, and the full binder/workload graph obligations. Only then run the
one combined scanner/hash pipeline screen with both CPU modes and both memory
metrics. Do not report a hashing-only gain or add a separate microbenchmark.
