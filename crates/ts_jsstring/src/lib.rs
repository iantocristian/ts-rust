//! Source bytes, JavaScript strings and wire positions (ADR 0013).
//!
//! This is one of the two Phase 0 contract leaves. It owns three things the AST
//! depends on and nothing else:
//!
//! * [`SourceText`], a file's bytes after the same BOM handling `decodeBytes`
//!   does, with a validity flag but no repair;
//! * [`JsString`], the type of every JavaScript string value, with the `Utf8`,
//!   `Wtf8` and `Raw` classification and byte-offset slicing that reclassifies;
//! * the position conversions, each ported with its own rounding, clamping and
//!   panic behavior rather than unified.
//!
//! Everything works on bytes and Go `rune` values. Nothing here silently
//! repairs a malformed byte or a lone surrogate: upstream's own transformations
//! decide when a byte becomes U+FFFD, and reproducing exactly those points is
//! what E4 measures. The design note is docs/design/text.md.
//!
//! The helper and escaping semantics gated in S04 are the leaf ones; the
//! scanner's token values (S05), the encoder's string section (S06) and the
//! printer's source-reuse decision (S08) complete E4 later.

pub mod case;
pub mod escape;
pub mod helpers;
pub mod jsstring;
pub mod positions;
pub mod rune;
pub mod source_text;
pub mod unicode_case_generated;

pub use case::{to_lower_js, to_upper_js};
pub use escape::{
    escape_jsx_attribute_string, escape_non_ascii_string, escape_string, escape_string_worker,
    LiteralTextFlags, QuoteChar,
};
pub use helpers::{lower_first_char, truncate_by_runes};
pub use jsstring::{classify, JsString, Validity};
pub use positions::{
    compute_ecma_line_starts, compute_lsp_line_starts, compute_position_map, PositionEncoding,
    PositionMap, TextPos,
};
pub use rune::{
    combine_surrogate_pairs, decode_js_string_rune, encode_js_string_rune, Rune, RUNE_ERROR,
};
pub use source_text::{ByteOrderMark, SourceText};

#[cfg(test)]
mod tests;
