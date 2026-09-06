# Design note: source bytes, JavaScript strings and positions

Backs ADR 0013. Status: reviewed and accepted by the owner on 2026-09-05. Citations are to the pinned checkout, paths relative to `tsc/internal` unless noted.

## 1. What Corsa does

1. **Files are bytes after BOM handling.** `decodeBytes` decodes a UTF-16 LE or BE BOM'd file with `utf16.Decode` (which replaces unpaired surrogates with U+FFFD), strips a UTF-8 BOM, and otherwise returns the bytes unchanged; nothing validates UTF-8 (`vfs/internal/internal.go:170`).
2. **The scanner reads bytes with an ASCII fast path and decodes the rest with Go's decoder.** `char` and `charAt` return single bytes; `charAndSize` returns the byte for ASCII and otherwise `utf8.DecodeRuneInString`, which yields `RuneError` with size 1 for a malformed byte (`scanner/scanner.go:434` to `460`). Consequences that E4 must reproduce:
   - a token that *starts* with a malformed byte reports `File_appears_to_be_binary` at position 0 and turns the rest of the file into `NonTextFileMarkerTrivia` (`scanner.go:932`);
   - a malformed byte inside a comment is skipped byte by byte, because the comment loop advances by `size`;
   - a malformed byte ends an identifier, because `IsIdentifierPart(RuneError)` is false (`scanner.go:1170`);
   - `scanString` copies the raw bytes between the quotes unchanged when the literal contains no backslash, carriage return or line feed, so malformed bytes inside such a literal survive into the literal value; later helpers and printing can replace or escape them as described below (`scanner.go`, `scanString` fast path);
   - `Invalid_character` is reported for a `#` not followed by an identifier and for other rejected characters (`scanner.go:918`).
3. **Lone surrogates are WTF-8 sentinels.** `EncodeJSStringRune` writes a surrogate code point as the three bytes `ED A0..BF 80..BF`; `DecodeJSStringRune` recognizes that pattern and otherwise defers to the standard decoder; `CombineSurrogatePairs` merges an adjacent high and low sentinel into the supplementary code point and must be applied wherever separately scanned values are joined (`stringutil/util.go:300` to `375`). The relater and the JSX transform special-case these sentinels (`checker/relater.go:2477`, `transformers/jsxtransforms/jsx.go:897`); `js_case.go` preserves them verbatim under case mapping.
4. **Positions use byte offsets, UTF-16 offsets and line/character pairs, with distinct conversion rules.**
   - Inside the compiler every position is a byte offset (`tsc/CHANGES.md`, scanner section).
   - The API encoder converts node positions to UTF-16 through `ast.PositionMap` (`api/encoder/encoder.go:539`). `ComputePositionMap` walks the text, decodes each non-ASCII sequence with `DecodeJSStringRune`, and counts one UTF-16 unit per code point below U+10000 and two above; a malformed byte decodes as `RuneError` with size 1 and counts as one unit; a **WTF-8 sentinel counts as one unit** (`ast/positionmap.go`).
   - The LSP converter, when the client negotiated UTF-16 (`lsp/server.go:1538`), counts units with the standard UTF-8 decoder and `utf16.RuneLen`; a malformed byte again counts as one unit, but the standard decoder rejects surrogates, so a **WTF-8 sentinel in source text counts as three units**, one per byte (`ls/lsconv/converters.go:394` to `445`). The direct byte-offset fast path applies when the whole text is ASCII-only or UTF-8 was negotiated (`ls/lsconv/linemap.go:16`).
   - Conversions also disagree at offsets inside a valid astral character. For the single-line text `😀`, UTF-16 offset 1 maps to byte offset **1** in `PositionMap`, **0** in the LSP converter, and **4** in `scanner.ComputePositionOfLineAndUTF16Character`. They respectively apply the preceding cumulative delta, stop before consuming a partial surrogate pair, and consume the character before testing the accumulated count (`ast/positionmap.go:95` to `110`; `ls/lsconv/converters.go:407` to `413`; `scanner/scanner.go:2765` to `2771`).
   - Line starts differ too: ECMAScript maps recognize CR, LF, CRLF, U+2028 and U+2029; LSP maps recognize only CR, LF and CRLF (`core/core.go:435` to `468`; `ls/lsconv/linemap.go:19` to `48`). The port preserves each path, including partial-character and out-of-range behavior; it does not make these conversions interchangeable.
5. **The encoder's string section.** The string data section is the file text once, plus each node string that differs from its source slice, "UTF-8 encoded string data, with WTF-8 used for JS strings containing lone UTF-16 surrogates" (`api/encoder/encoder.go:115`). The JS client decodes it with its own WTF-8 decoder (`packages/typescript/src/api/node/wtf8.ts`).
6. **Small semantics.** `jsnum.Number.String` implements JavaScript `Number.prototype.toString` and `FromString` implements `ToNumber` (`jsnum/string.go:17`, `41`); `ParsePseudoBigInt` handles bigint literals (`jsnum/pseudobigint.go`). Identifier tables are generated from Unicode 15.1.0 (`stringutil/_scripts/generate-unicode-data.mts:11`). The organize-imports comparer normalizes with `x/text/unicode/norm`, builds natural-number keys and applies its own case-first and accent rules; the locale preference is accepted and ignored (`ls/lsutil/organizeimports.go:89`, `ls/lsutil/userpreferences.go:110`).

## 2. Contract

### 2.1 `SourceText`

`SourceText` owns the file's bytes after the same BOM handling as `decodeBytes`: UTF-16 with a BOM is decoded to UTF-8 with U+FFFD for unpaired surrogates, exactly as Go's `utf16.Decode` does; a UTF-8 BOM is dropped; everything else is kept byte for byte. It is immutable and shared (`Arc<[u8]>`), because programs, snapshots and the encoder all hold the same text.

On construction it records whether the bytes are valid UTF-8. If they are, `as_str()` returns `Some(&str)`; otherwise only byte views are available. A valid whole file does not guarantee that a compiler or converted wire offset lies on a UTF-8 character boundary. Slicing by those offsets operates on bytes; a sliced view or new string gets its own validity classification, and an `&str` slice requires both endpoints to be character boundaries. Standard string operations are used only where their behavior matches the corresponding Go operation.

Positions remain byte offsets in both cases. ECMAScript line starts use the line-break set in `stringutil.IsLineBreak`; the distinct LSP line map uses only CR, LF and CRLF. Both maps retain the source byte offsets and port their respective decoding loops.

The scanner's reading primitives are ported as written: `char` returns a byte, `char_and_size` returns the ASCII byte or the standard decoding with size 1 for a malformed byte. The behaviors in section 1, item 2 follow from that without special cases.

### 2.2 `JsString`

`JsString` is the type of every JavaScript string value: literal values, template text, identifier and symbol names, string-literal types, property keys, and the strings the encoder writes. It owns cheaply shared immutable bytes (`Arc<[u8]>` with an inline small-string optimization to be measured) and a validity tag computed once at construction:

| Tag | Meaning | Views available |
|---|---|---|
| `Utf8` | valid UTF-8, the common case | `&str`, bytes |
| `Wtf8` | valid UTF-8 except for WTF-8 sentinels for lone surrogates | bytes; code-point iteration through the sentinel-aware decoder |
| `Raw` | contains bytes that are neither, including scanner raw-copy results and fragments produced by byte slicing | bytes only |

Rules:

- No implicit conversion to `String`, `&str` or `Cow<str>`. Code that needs `&str` asks for it and handles `None`.
- Equality, ordering and hashing are byte-wise, as Go's string comparisons are; sorting at observation points (ADR 0010) uses the ported comparators, which compare bytes unless they say otherwise.
- `encode_rune`, `decode_rune` and `combine_surrogate_pairs` port `EncodeJSStringRune`, `DecodeJSStringRune` and `CombineSurrogatePairs`, and the latter is applied at the same join points upstream applies it.
- Decoder choice and output behavior are ported per helper, not selected globally from the validity tag. `ToLowerJS` and `ToUpperJS` use `DecodeJSStringRune` and preserve complete surrogate sentinels, but write U+FFFD for other malformed bytes (`js_case.go:20` to `35`, `51` to `61`). `TruncateByRunes` uses Go's standard `range` behavior, which counts each malformed byte separately and can split a sentinel: truncating bytes `ED A0 80` to one rune returns just `ED` (`stringutil/util.go:256` to `267`). `LowerFirstChar` likewise uses the standard decoder, then re-encodes the first decoded rune and copies the suffix (`stringutil/util.go:248` to `253`). These replacements and fragments are preserved as behavior, rather than repaired by a shared helper policy.
- Slices use byte offsets with checked bounds. They do not inherit the parent's validity tag without proving that the sliced bytes satisfy it; in particular, slicing a `Utf8` or `Wtf8` string can produce `Raw` bytes.
- Diagnostics text, baselines, emitted JavaScript and declaration files use byte writers to preserve the output of the corresponding upstream formatting path. This does not bypass literal escaping: when `getLiteralText` can reuse eligible original source text, it copies those bytes; otherwise its string-literal path calls `escapeStringWorker`, which emits `\uD800` for a high-surrogate sentinel and `\uFFFD` for a stray malformed byte, with the same quote and escape flags (`printer/utilities.go:77` to `174`, `227` to `253`). The final writer adds no further normalization. E4 compares both source reuse and regenerated literal output to Go.

Interning is section 13, item 8 and is deferred; the `Arc`-backed representation is what makes copying a `JsString` as cheap as copying a Go string header, which is the requirement the second review set.

### 2.3 Positions

Byte offsets everywhere inside the compiler, converted at the edge:

- **API.** A per-file `PositionMap` ported from `ast/positionmap.go`: an ASCII-only flag, else a sorted table of `(byte offset after the sequence, cumulative delta)` built with the sentinel-aware decoder, so a sentinel is one unit, a malformed byte is one unit, and an astral character is two. Each direction retains its upstream binary-search calculation, including results inside byte sequences; it does not round to character boundaries or clamp offsets.
- **LSP.** `ComputeLSPLineStarts` recognizes CR, LF and CRLF and records whether the whole text is ASCII-only. Byte offsets apply when UTF-8 was negotiated or that flag is set. Otherwise conversion from UTF-16 scans with the standard decoder and counts UTF-16 units, stopping before a character that would exceed the requested count. Conversion to UTF-16 uses Go's standard `range` over the byte prefix. A sentinel counts as three units and a malformed byte as one. Preserve the existing line/character and absolute-position clamping, and the byte prefix even when its end cuts a valid sequence (`ls/lsconv/converters.go:368` to `445`).
- **Scanner utilities.** `ComputePositionOfLineAndUTF16Character` consumes a whole character before checking whether the accumulated UTF-16 count reached the requested offset. It therefore rounds an interior surrogate-pair offset forward, unlike LSP. Preserve its `allowEdits` clamping versus panic behavior. Its ECMAScript wrappers use the CR/LF/CRLF/U+2028/U+2029 line map; `GetECMALineAndUTF16CharacterOfPosition` counts the byte prefix with `core.UTF16Len` (`scanner/scanner.go:2684` to `2718`, `2741` to `2797`; `core/core.go:486` to `499`).

For `😀` at UTF-16 offset 1, E4 requires the API/LSP/scanner results 1/0/4 respectively. Sentinel counting, partial-character offsets, line-map differences and range handling are separate fixtures. Byte slices remain representable even when one of these conversions produces an offset that is not a Rust `str` boundary.

### 2.4 Encoder strings

The encoder writes the file text once and appends only strings that differ from their source slice, in the order upstream appends them; string offsets are byte offsets into that section, and node positions are converted through the file's `PositionMap`. Because `JsString` and `SourceText` are already bytes, no re-encoding happens at this binary string-table boundary: a `Raw` string is written as-is, matching `api/encoder/stringtable.go:25` to `59`. This byte-copy rule is distinct from the printer's literal-escaping rules above.

### 2.5 Small semantics

`jsnum` is ported with a shortest-round-trip printer for `Number.prototype.toString`, `ToNumber` parsing with the same grammar upstream accepts, and the pseudo-bigint routines; golden tests compare against V8 output for the corpus's numeric literals plus edge cases (subnormals, `-0`, exponent thresholds at 1e21 and 1e-7). Identifier tables are regenerated from Unicode 15.1.0 data, not from whatever version a crate ships. The organize-imports comparer is ported as written with a Unicode normalization crate for NFD; the locale preference is parsed and ignored, as upstream.

## 3. What E4 asserts for this note

| Fixture | Assertion | E4 criteria |
|---|---|---|
| Every string literal in the corpus | literal value bytes, literal types and encoder strings byte-identical to the oracle | `token_value_bytes`, `token_literal_bytes`, `encoder_output_bytes` |
| Lone surrogates in escapes (`"\uD800"`, `"\uDC00"`, split pairs joined by concatenation and template cooking) | sentinel bytes identical; `combine_surrogate_pairs` produces the same code points at the same join points | `token_value_bytes`, `token_literal_bytes` |
| Malformed byte at token start, inside a comment, inside an identifier, inside a raw-copied literal | same diagnostics (`File_appears_to_be_binary` at 0, identifier end, raw bytes preserved) and same token boundaries | `diagnostics`, `token_value_bytes`, `token_literal_bytes` |
| `ToLowerJS` and `ToUpperJS` on sentinel bytes and on malformed byte `FF` | complete sentinels survive; `FF` becomes U+FFFD bytes `EF BF BD`, matching Go | `helper_semantics`, `helper_printer_semantics` |
| `TruncateByRunes` on `ED A0 80` with maximum length 1; `LowerFirstChar` on sentinel/malformed prefixes | truncation returns raw byte `ED`; helper-specific decoding, replacement and suffix copying match Go | `helper_semantics`, `helper_printer_semantics` |
| Literal escape helpers on sentinels, malformed bytes and quote/escape flags | escaped bytes match the oracle before integration with the printer | `helper_semantics`, `helper_printer_semantics` |
| Original-source and synthesized string literals containing sentinels or malformed bytes | eligible source reuse preserves its bytes; regenerated literals apply Go's escapes, including `\uD800` and `\uFFFD`, with matching quote/escape flags | `helper_printer_semantics` |
| UTF-8 BOM, UTF-16 LE BOM, UTF-16 BE BOM, unpaired surrogate under a UTF-16 BOM | same decoded bytes and offsets, U+FFFD where Go produces it | `source_decoding` |
| Sentinel bytes in source text | API positions count one unit, LSP positions count three, both matching the oracle | `utf16_positions` |
| Astral and multi-byte characters, including UTF-16 offset 1 inside single-line `😀` | each conversion matches its own oracle path; the interior offset maps to bytes 1/0/4 for API/LSP/scanner respectively | `utf16_positions` |
| Byte slices ending or starting inside valid UTF-8 or a sentinel | bytes remain representable, slice validity is reclassified, and no invalid `&str` view is exposed | `slice_validity` |
| CR, LF, CRLF, U+2028 and U+2029 | ECMAScript and LSP line maps each match Go; for `a<U+2028>b` or `a<U+2029>b`, their line starts are `[0, 4]` and `[0]` respectively | `utf8_positions` |
| Out-of-range lines, characters and absolute positions | preserve each API/LSP/scanner path's clamping, arithmetic or panic behavior, including both `allowEdits` settings | `utf16_positions`, `utf8_positions` |
| Encoder success and failure | same success or error outcome per file; byte-identical output wherever the oracle encodes | `encoder_success_error`, `encoder_output_bytes` |

Criterion ids refer to `status/experiments.toml`. S04 gates leaf helpers with `helper_semantics`; S05 gates scanner values and rescans with `token_value_bytes`; S06 gates encoder behavior. S08 completes E4 through production literal-type construction and original-source/regenerated printing, retaining the full `token_literal_bytes` and `helper_printer_semantics` requirements.

## 4. Choices deliberately left to measurement

The inline small-string threshold of `JsString`; whether `Utf8`-tagged strings keep a cached `&str` view or recompute it; and interning (section 13, item 8).

## 5. Evidence index

`vfs/internal/internal.go:170`; `scanner/scanner.go` 434 to 460, 918, 932, 1170, `scanString`, 2684 to 2718, 2741 to 2797; `stringutil/util.go` 248 to 270, 280 to 375; `stringutil/js_case.go` 20 to 35, 51 to 61; `stringutil/_scripts/generate-unicode-data.mts:11`; `printer/utilities.go` 77 to 174, 227 to 253; `ast/positionmap.go`; `api/encoder/encoder.go` 115, 539; `api/encoder/stringtable.go` 25 to 59; `core/core.go` 435 to 468, 486 to 499; `ls/lsconv/linemap.go` 14 to 52; `ls/lsconv/converters.go` 368 to 445; `lsp/server.go:1538`; `checker/relater.go:2477`; `transformers/jsxtransforms/jsx.go:897`; `jsnum/string.go` 17, 41; `jsnum/pseudobigint.go`; `ls/lsutil/organizeimports.go:89`; `ls/lsutil/userpreferences.go:110`; `packages/typescript/src/api/node/wtf8.ts`.
