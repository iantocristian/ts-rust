# S05 differential protocol, version 1

This contract and the request inventory are frozen before scanner behavior is
ported. `scripts/s05_cases.py` defines the exact request shapes and deterministic
fixture construction. `data/s05/corpus.json`, `fixtures.json`, `cases.json` and
`probes.json` record the reviewed expansion. Oracle observations are measured
from the pinned Go implementation; Rust results never select case eligibility.

## Framing and validation

Input and output are UTF-8 NDJSON: exactly one JSON object and one LF per record.
Two persistent processes receive the same requests in frozen order. Each request
constructs a fresh scanner, installs an enabled diagnostic callback, then applies
all source/configuration options. No scanner or checkpoint crosses a case boundary.
Stdout contains protocol records only; stderr is separately drained to disk.

Reject duplicate keys at every nesting level, unknown/missing fields, null where
not explicitly allowed, booleans used as integers, floats/non-finite numbers,
unknown operations, malformed/noncanonical hex and trailing JSON. Validation
precedes behavioral panic recovery in both adapters. Integer fields use signed
64-bit Go-int bounds unless narrower bounds are stated below. Request source
and every replacement source are at most 4 MiB; actions number 1..4096. A wire
record is at most 32 MiB including LF. Case IDs are nonempty strings of at most
1024 characters. Each complete record read or request send has an absolute 120-second deadline; timeout, oversized
records, process exit or malformed framing fail capture and retain stderr.

Every request has exactly these fields:

```json
{"version":1,"id":"case-id","source_hex":"6162","decode_source":false,"target":0,"variant":0,"skip_trivia":true,"actions":[{"op":"scan_all"}]}
```

`source_hex` and all other byte fields are even-length lowercase hex. The source
is passed directly to SetText unless `decode_source` is true, in which case the
pinned BOM/source decoder runs first. Positions, the scan-all token bound and panic input identity then refer to decoded
bytes. The parent models BOM decoding solely for those protocol bounds and
identities; this model does not add a scanner source-decoding parity claim.
`target` is one of 0..12, 99,100 (the pinned Go constants); 0 means Go's default
Latest. `variant` is 0=Standard or 1=JSX. Both booleans are required.

## Actions

An action contains `op` and exactly its documented arguments. A missing Go
variadic slash argument is represented explicitly rather than relying on a
missing JSON field. Only legal LIFO checkpoints are differential inputs; Rust
foreign/out-of-order checkpoint misuse is tested separately.

| Operations | Additional fields |
| --- | --- |
| `scan`, `scan_all`, `snapshot`, `rescan_less_than`, `rescan_greater_than`, `rescan_asterisk_equals`, `rescan_hash`, `rescan_question`, `scan_jsx`, `scan_jsx_identifier`, `scan_jsx_attribute`, `rescan_jsx_attribute`, `scan_jsdoc` | None |
| `rescan_template`, `rescan_jsx`, `scan_jsx_ex`, `scan_jsdoc_text`, `set_skip_trivia`, `set_skip_jsdoc_asterisks`, `set_on_error` | `flag`: boolean, passed to the corresponding upstream operation |
| `rescan_slash` | `report_errors`: `"omitted"`, `"false"`, or `"true"` |
| `reset`, `mark`, `rewind`, `commit`, `can_follow_jsdoc_at` | None; rewind/commit consume the most recent mark |
| `set_text` | `text_hex`: raw replacement bytes; no BOM decoding |
| `reset_pos`, `reset_token_state` | `pos`: signed 64-bit integer; negative positions are behavioral contract probes |
| `set_variant`, `set_target` | `value`: one of the corresponding request enum values |
| `observe` | `getter`: `text`, `token`, `flags`, `full_start`, `start`, `end`, `token_text`, `value`, `range`, `directives`, or `predicates` |
| `identifier_block` | `first`, `count`: 0<=first, 1<=count<=4096, first+count<=0x110000 |
| `identifier_point` | `point`: signed 32-bit Go rune |
| `identifier_text` | `variant`: 0 or 1; observes IsIdentifierText over current text |
| `equal_fold` | `other_hex`: compares current text with these bytes through Go strings.EqualFold |
| `identifier_token`, `valid_identifier`, `intrinsic_jsx_name`, `string_to_token`, `keyword_suggestions`, `shebang`, `normalize_jsdoc` | None; text helpers observe current scanner text |
| `token_to_string` | `kind`: 0..350; all pinned SyntaxKind values |
| `skip_trivia` | `pos`: signed integer; `options`, `stop_after_line_break`, `stop_at_comments`, `in_jsdoc`: required booleans. `options=false` passes nil upstream options |
| `comment_ranges` | `pos`: signed integer; `trailing`: boolean |
| `number_from_string`, `pseudo_bigint` | None; consume current text as arbitrary bytes |
| `number_format` | `bits`: exactly 16 lowercase hex digits containing big-endian IEEE-754 binary64 bits |

Token-producing actions, and explicit `snapshot`, emit the complete token
snapshot described below. State setters/checkpoint actions emit null; they do
not implicitly call potentially panicking getters. `scan_all` repeats Scan until
EOF, with at most decoded-source-byte-length+2 observations. Ordinary single
scan, JSX and JSDoc actions execute exactly once. An expected panic is terminal
for that case; no implicit observation or later action runs afterward.

## Responses

The first record is exactly `{"event":"begin","id":"case-id","version":1}`.
It is followed by zero or more observation records and exactly one end record:

```json
{"event":"observation","id":"case-id","action":0,"ordinal":0,"status":"ok","value":null,"diagnostics":[]}
{"event":"end","id":"case-id","observations":1,"completed_actions":1}
```

Action indexes are zero-based. Ordinals are contiguous and case-wide. Every
non-scan_all action produces one observation; each scan_all produces one or
more observations, ending with EOF or a panic. End counts must match the actual
stream and completed action prefix. Missing fixed actions, skipped/reordered
ordinals, extra framing or missing EOF/end are protocol failures. Each stream
is validated independently: different valid token counts or values are measured
parity failures. Drain each stream to its own case end after a mismatch.

An `ok` token value has exactly:

```json
{"kind":1,"token":1,"full_start":0,"start":0,"end":0,"text_hex":"","value_hex":"","flags":0,"range":[0,0],"predicates":[false,false,false,false,false,false,false],"directives":[]}
```

`kind` is the operation return value; `token` separately observes Scanner.Token.
Snapshot uses Scanner.Token for both. `flags` is signed 32-bit; positions and
range elements are signed 64-bit. Predicate order is HasUnicodeEscape,
HasExtendedUnicodeEscape, HasPrecedingLineBreak, HasPrecedingJSDocComment,
HasPrecedingJSDocLeadingAsterisks, HasPrecedingJSDocWithDeprecatedTag,
HasPrecedingJSDocWithSeeOrLink. Directives are ordered objects with exactly
`start`, `end` (signed integers), `kind` (the upstream directive discriminant).
Observe getters return the corresponding scalar, hex string, range, directive
list or predicate list. Token text/value bytes retain upstream stale values;
they are never normalized according to token kind.

Each diagnostic has exactly `code`, `category`, `key`, `start`, `length`, `args`.
Code/category use upstream signed 32-bit values; positions are signed 64-bit.
`key` is the generated localization identity. Arguments are ordered tagged
objects: `{"kind":"string","hex":"..."}` or
`{"kind":"integer","value":0}`. Reject unsupported Go argument types.
`diagnostics` records callback order for this observation only. No sorting,
deduplication, localization or UTF-8 repair is applied.

Other ok values:

- Identifier point: exactly three booleans `[start,part,jsx_part]`. Identifier
  block: `{start_hex,part_hex,jsx_hex}` bitsets, low bit first within each byte,
  exactly ceil(count/8) bytes and zero unused trailing bits. These are measured
  points, not additional source-file or request counts.
- Identifier predicates, EqualFold and CanFollowJSDocAt: boolean. Identifier/token
  lookups: kind integer. Text helpers: hex string. Keyword suggestions: sorted
  hex strings (explicit set-valued contract; Go map iteration order is not stable).
- Comment ranges: ordered `{start,end,kind,trailing_newline}` records.
- SkipTrivia: signed integer.
- Numeric operations: `{number_class,bits,text_hex}` where number_class is finite,
  nan, positive_infinity or negative_infinity. Bits is 16 hex digits except null
  for NaN, whose payload is not a promised Go conversion result. text_hex is the
  actual Number.String bytes. Pseudo-bigint returns hex bytes.

For a panic, status is `"panic"` and value is exactly
`{"message":"raw payload","class":"contract","input_hex":null}`.
Classes are contract, bounds, invalid_bigint, upstream_assertion, or unexpected.
The adapter retains the raw payload; the parent independently checks the narrow
classification. Only frozen expected-panic cases can pass. Contract and named
upstream assertion messages compare exactly. Bounds expectations contain only `action` and `class`; the parent must recognize
each runtime's complete bounds syntax independently before matching their classes.
Their raw messages remain in observations and may differ between runtimes. Invalid-bigint requires the operation-specific
panic and exact hex of the input after TrimSuffix("n"); incidental Go `%q` versus
Rust quoting is not compared. Arbitrary assertion/overflow panics never become
a recognized contract merely because both programs panicked.

## Frozen claims and storage

The physical corpus contains 12,854 files, 12,427,814 bytes and 25,708 mode cases.
Compiler directives remain source bytes; JSX selection follows the physical
extension. Supplemental groups include regexp, rescan, identifier and numbers;
group membership, witness expectations and expected panic contracts are part
of the frozen request digest. The digest is SHA-256 of each canonical sorted-key
ASCII JSON case object plus LF in request order. Case objects contain request,
groups, witness and expected_panic. Tables come from access-only Go exports and
the pinned Go SimpleFold runtime; their named source hashes are recorded separately.

The initial freeze had 33,323 requests with digest
`e56795ae01675f3c89873ef407e0baaa46540f2d6f6d9046ecbf25c18ed2f79a`.
Pre-capture source review added seven cases without removing or changing any
existing case: six pseudo-bigint radix/underscore/opaque-input regressions, and
`regexp/deep/sets-partially-closed`. The latter has 20,000 opening brackets and
one closing bracket, allowing slash discovery to reach the nested grammar and
observe 19,999 missing-closure diagnostics. The original fully unclosed case
remains a delimiter-discovery case. That revision has 33,330 requests and
digest `e9e7329db59a8e94a3bbaedeff64572c92d4170e81d787f182e9c40d4fa0731c`.

A subsequent protocol review added two more cases, again preserving every prior
case: `state/bounds-token-text` resets the position to 4 in `abc`, then reads
TokenText. Pinned Go reports `runtime error: slice bounds out of range [:4] with
length 3`; Rust reports its own recognized bounds message. This exercises the
class comparison without treating arbitrary assertions as bounds.
`number/pseudo-decoded-utf16le` decodes BOM-prefixed UTF-16LE `0b2n` before
ParsePseudoBigInt; its expected panic input is the decoded, suffix-trimmed bytes
`306232`, never the original BOM bytes. The current freeze has 33,332 requests and
digest `7045d660c44245b041a50c5530c7e0d871e61c8d2b7bb5a61dccae8b64e74fb7`.

Overall parity is derived by xtask from the complete `cases.json` test map.
Regexp/rescan ratios use their complete nonempty frozen case subsets. Diagnostics
and token_value_bytes compare every designated observation, including modes with
empty diagnostics. Required fixed diagnostic witnesses prevent disabled callbacks
from passing vacuously. Table drift fails capture. No metric here certifies AST
diagnostic construction, checker literal types, encoding or printer integration.

The parent computes canonical observation digests; adapter digests/pass flags
are not accepted as proof. Successful streams need not be retained in full.
Failures retain case/request identity, first mismatching observations and stderr;
complete mismatching traces may be spooled under target. Evidence records the
stream hashes, counts and bounded failure details in the runner-captured report.
