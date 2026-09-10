//! Checker-independent printing (`tsc/internal/printer`).
//!
//! This crate starts with the two text writers the checker's type display uses
//! (`TypeToStringEx` writes through a `TextWriter`, `SymbolToStringEx` through a
//! `SingleLineStringWriter`). The printer, emit context, factory helpers and
//! emit metadata follow with the S08 type-display slice; JavaScript and
//! declaration emit are Phase 3. The dependency direction is fixed: the checker
//! depends on this crate, never the reverse.
//!
//! Output is bytes. Upstream strings may hold arbitrary bytes and the text
//! contract (`docs/design/text.md`) forbids lossy conversion, so writers accept
//! and return `[u8]` and count columns in UTF-16 units the way upstream does.

mod emit_text_writer;
mod single_line_string_writer;
mod text_writer;

pub use emit_text_writer::EmitTextWriter;
pub use single_line_string_writer::SingleLineStringWriter;
pub use text_writer::{get_default_indent_size, TextWriter};

#[cfg(test)]
mod tests;
