# ADR 0013: Source bytes, JavaScript strings and wire positions

Status: Accepted (2026-09-05)
Design note: [docs/design/text.md](../design/text.md), reviewed and accepted by the owner on 2026-09-05
Plan: section 6, decision 8

## Context

Corsa's filesystem decoder removes BOMs and decodes UTF-16 but otherwise preserves arbitrary bytes; the scanner diagnoses malformed bytes and its string fast path can preserve malformed literal bytes. JavaScript strings can contain lone surrogates, stored as WTF-8 sentinels; the API encoder documents its string section as UTF-8 with WTF-8 for such values and converts positions to UTF-16; LSP negotiates UTF-16 or UTF-8. A Rust `String` cannot hold either malformed input or `"\uD800"`.

## Decision

`SourceText` owns source bytes after the same BOM handling as Go, with a validated `str` fast path for ordinary UTF-8 and byte offsets that match the scanner's. `JsString` owns cheaply shared immutable bytes: UTF-8 or WTF-8 for ordinary values, raw malformed bytes where Go accepts them, with explicit validation and conversion boundaries and no implicit lossy conversion. Positions are converted at the edge through per-file position maps that preserve the oracle's handling of malformed sequences. `jsnum` is ported with a shortest-round-trip printer and V8 golden tests; identifier tables are regenerated from Unicode 15.1; the organize-imports comparer is ported as written (NFD, natural keys, case and accent rules, locale ignored, no collation library).

## Consequences

All text output that must be byte-identical to baselines goes through byte writers after the corresponding upstream formatting and escaping operations. Byte-backed storage does not bypass Go's explicit replacements or printer escapes. E4 pins scanner, literal, helper, printer and encoder behavior for valid, WTF-8 and malformed input, plus the separate API/LSP/scanner position and line-map rules. Accepted on 2026-09-05 after the owner reviewed the design note and its Go evidence; the E3 and E4 assertions remain required verification, not completed verification.

## Evidence

`tsc/internal/vfs/internal/internal.go` (`decodeBytes`), `tsc/internal/stringutil/util.go` (`EncodeJSStringRune`), `tsc/internal/scanner/scanner.go` (invalid-character diagnostics), `tsc/internal/api/encoder/encoder.go` (string section, UTF-16 positions), `tsc/internal/ls/lsutil/organizeimports.go` (comparer), `packages/typescript/src/api/node/wtf8.ts`.

## Amendments

Draft 1 said UTF-8 everywhere; draft 2 added WTF-8 values; draft 3.1 removed ICU collation; draft 3.2 added byte-backed source text after the third review.

The design-note correction pass preserves helper-specific replacement/truncation and original-source versus regenerated-literal printing. Converted offsets can cut valid UTF-8 sequences, so slices remain byte-backed and require their own validity classification. E4 records partial-character rounding, clamping and the separate ECMAScript/LSP line maps as well as malformed-byte and surrogate counting.
