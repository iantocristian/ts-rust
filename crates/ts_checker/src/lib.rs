//! The checker (`tsc/internal/checker`): checker-owned symbols, types and
//! signatures over S07's immutable bound files, with `.types` and `.errors.txt`
//! parity on the frozen subset as S08's acceptance.
//!
//! This is the S08 scaffold. It fixes the contracts the port builds on and ports
//! the leaf mechanisms that every later family depends on; it contains no type
//! construction, relation or checking algorithm yet. What exists:
//!
//! - [`CheckerOwner`]: one owner, one exclusive operation, the reserved checker
//!   identity adopted as the symbol arena exactly once (ADR 0007, plan §4.1).
//! - [`ResolutionStack`]: the `pushTypeResolution` cycle guard, keyed on entity
//!   and property, ported exactly (ADR 0008).
//! - [`LinkStore`]: checker-local side tables keyed by arena identity and slot,
//!   paged on first use (`links.go`).
//! - [`TypeStore`]: the common type record and alias store. Payload storage is
//!   a P1 decision made from the Go census; nothing here promises a byte size.
//! - Flags and enums transcribed from `types.go`, checked against values read
//!   out of the pinned Go package (`data/s08/checker-flag-observations.json`).
//! - [`CheckerHost`]: the program surface the checker requires, mapped method by
//!   method to what the loader already provides and what P2 still owes.
//!
//! Design notes: `docs/design/symbols.md`, `docs/design/ownership.md`; plan:
//! `docs/S08-implementation-plan.md`.

mod flags;
mod host;
mod ids;
mod links;
mod owner;
mod resolution;
mod type_display;
mod types;

pub use flags::*;
pub use host::CheckerHost;
pub use ids::{SignatureId, TypeId};
pub use links::{LinkKey, LinkStore};
pub use owner::{CheckerOwner, CheckerState, Operation};
pub use resolution::{ResolutionStack, TypeResolution, TypeSystemEntity, TypeSystemPropertyName};
pub use type_display::{
    alias_symbol, alias_type_arguments, to_node_builder_flags, DEFAULT_MAXIMUM_TRUNCATION_LENGTH,
    MAX_SERIALIZATION_LEVEL, NO_TRUNCATION_MAXIMUM_TRUNCATION_LENGTH,
};
pub use types::{AliasId, TypeAlias, TypeKind, TypeRecord, TypeStore};

/// Checker-boundary failures. Arena identity, generation and bounds failures pass
/// through unchanged; the checker adds the failures only it can observe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Arena(ts_arena::Error),
    /// The same thread asked for a second operation on an owner whose operation
    /// it already holds. Detected before waiting; ordinary contention waits.
    Reentry,
    /// A checker-local identity space is exhausted; IDs never wrap or recycle.
    IdExhausted,
    /// A required operation the port does not implement yet, by upstream name.
    /// A frozen case that reaches one is a named failure, never a pass.
    Unsupported(&'static str),
}

impl From<ts_arena::Error> for Error {
    fn from(error: ts_arena::Error) -> Self {
        Self::Arena(error)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Arena(error) => error.fmt(output),
            Self::Reentry => {
                output.write_str("the checker operation is already held by this thread")
            }
            Self::IdExhausted => output.write_str("checker-local identity space exhausted"),
            Self::Unsupported(name) => write!(output, "unsupported checker operation: {name}"),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod flag_tests;
#[cfg(test)]
mod tests;
