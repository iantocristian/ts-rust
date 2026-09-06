//! Byte-preserving source text and JavaScript strings for the Phase 0 text slice.
//!
//! See `SLICE.md` for supported operations and the integration work outside this crate.

pub mod helpers;
pub mod positions;
pub mod strings;

mod case_tables;

pub use helpers::{LiteralEscapeFlags, QuoteChar};
pub use strings::{JsString, SourceText, Validity};
