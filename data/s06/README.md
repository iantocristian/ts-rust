# S06 frozen direct parser inputs

These files define requests and native Go observations. The registered E1
producer reconstructs them before comparing the production Rust parser/runtime;
the files alone do not establish parity.

`python3 scripts/s06.py freeze` reconstructs the corpus from a fresh export of
the pinned Git tree and verifies it without writing. `--write-manifest` is an
explicit reviewed regeneration operation. The canonical submodule is never
patched. The Go test harness's repository locator alone retains its source path;
other packages build with `-trimpath`. Caller Go caches are retained outside the
export and are not evidence.

- `cases.json` has exactly 12,829 primary IDs: 12,721 physical compiler/conformance
  cases and 108 bundled libraries. Expanded variants never add primary rows.
- `corpus.json` records physical raw/loaded hashes, every extracted unit and its
  eligibility, original directives, configurations, source-loading routes,
  effective parser options and virtual filesystem metadata reads. Each physical
  case occupies one JSON line so a change is reviewable as a complete case.
- `requests.json` records each finalized parser request's metadata and SHA-256.
  Source bytes are reconstructed by the authoritative Go preprocessing bridge;
  their hash and the complete request hash must match. No Rust parser output
  selects membership, options or source bytes.
- `probes.json` binds the complete ordered request sequence, counts, largest
  request and the independently inspected pre-expansion totals.

Physical files pass through Go's BOM decoder before directive splitting.
Ordinary virtual files pass through the test filesystem decoder; initial
tsconfig text is supplied directly. Bundled libraries use embedded bytes at
`bundled:///libs/<name>`, without file decoding or directive splitting. Final
parser text is always passed as bytes without another decoding step.

Seven retained cases contain the removed `module: none` setting. Their original
configuration helper fails. The reviewed `legacy-module-none-parser-input-v1`
policy applies only to those exact seven paths and that exact value: it retains
the rejection, expands the remaining valid variants through the unchanged
helper, and uses the actual error-bearing `ParseCommandLine` result for effective
parse options. Each affected variant retains diagnostic 6046. There are 13 such
variants and 23 requests. This is not compiler-option validation or an E2
diagnostic divergence allowance. An unreviewed path or changed value fails.

All other settings remain recorded. The parser-relevant option projection is
explicit in `probes.json`; package metadata comes solely from each frozen
virtual filesystem. Program/project-reference loading, checker/emit behavior,
external scripts and content-mapper execution are outside this direct parser
corpus. Unsupported auxiliary assets remain recorded without kind coercion.

The supplemental inventory in `fixtures.json` covers constructed factory,
JSDoc, codec, malformed-input and recursion traces without adding primary rows.
`watchdogs.json` separately measures the pinned cyclic decoder nontermination;
neither timeout contributes a returned-error parity row. Utility/accessor
observations are independently regenerated from access-only Go probes and then
checked against the Rust APIs. Generated method scope is provenance, not credit
in the handwritten source-function denominator.

Two named keyword-split diagnostics admit only the alternatives observed in
fresh pinned-Go processes; Rust must keep its reviewed deterministic choice.
The report preserves both raw arguments, exactness bits and stream hashes. No
encoder output or other diagnostic receives this qualification.
