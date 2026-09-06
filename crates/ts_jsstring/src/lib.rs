//! Source bytes, JavaScript strings and wire positions: the contract of ADR 0013
//! and docs/design/text.md.
//!
//! Positions inside the compiler are byte offsets. Conversion to UTF-16 happens
//! at the edge through three distinct upstream paths (API position map, LSP
//! converters, scanner utilities) that are ported separately and are not
//! interchangeable.

pub mod escape;
pub mod helpers;
mod js_case_generated;
pub mod jsstring;
pub mod line_map;
pub mod lsp;
pub mod position_map;
pub mod scanner_positions;
pub mod source_text;
pub mod wtf8;

pub use jsstring::{JsString, Validity};
pub use source_text::SourceText;
