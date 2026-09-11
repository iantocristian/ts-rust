//! The checker (`tsc/internal/checker`): checker-owned symbols, types and
//! signatures over S07's immutable bound files, with `.types` and `.errors.txt`
//! parity on the frozen subset as S08's acceptance.
//!
//! S08 P1 storage and the P2/P3 production semantic slices. What exists:
//!
//! - [`CheckerOwner`]: one owner, one exclusive operation, the reserved checker
//!   identity adopted as the symbol arena exactly once (ADR 0007, plan §4.1).
//! - [`ResolutionStack`]: the `pushTypeResolution` cycle guard, keyed on entity
//!   and property, ported exactly (ADR 0008).
//! - [`LinkStore`]: checker-local side tables keyed by arena identity and slot,
//!   paged on first use (`links.go`).
//! - [`TypeStore`]: the common type record, per-kind payload tables, aliases,
//!   `Arc` lists and the interning caches; [`SignatureStore`] for signatures,
//!   index infos and type predicates.
//! - The type-creating prefix of `NewChecker`, and the constructors it and the
//!   frozen storage trace need: intrinsics, string/number/bigint/boolean
//!   literals with fresh/regular links, anonymous object types, tuple targets,
//!   type references, type parameters, unions and ordered intersections, template
//!   literal types, synthetic call signatures and synthetic expressions. Every
//!   constructor sorts and interns the way upstream does.
//! - Flags and enums transcribed from `types.go`, checked against values read
//!   out of the pinned Go package (`data/s08/checker-flag-observations.json`).
//! - [`CheckerHost`]: the program surface the checker requires, mapped method by
//!   method to the retained compiler program.
//!
//! - Program initialization, checker-local source-symbol merges, primitive and
//!   anonymous-object source queries, and bounded source checking with structured
//!   diagnostics. Type display builds syntax through `ts_nodebuilder` and prints
//!   it through `ts_printer`.
//!
//! Source arrays/tuples, full relations/body checking, loaded generic library
//! operations and declaration emit remain pending. Unported branches return
//! named `Error::Unsupported` failures. The named P2 programs do not certify the
//! frozen E2 denominator; see `docs/S08-P2.md` and `docs/S08-P3.md`.
//!
//! Design notes: `docs/design/symbols.md`, `docs/design/ownership.md`; plan:
//! `docs/S08-implementation-plan.md`.

mod check;
mod compare;
mod construct;
mod diagnostics;
mod flags;
mod handles;
mod host;
mod ids;
mod init;
mod intersection;
mod key;
mod links;
mod members;
mod merge;
mod name_resolution;
mod node_builder;
mod owner;
mod program;
mod program_init;
mod query;
mod reduction;
mod resolution;
mod signatures;
mod state;
mod symbols;
mod template;
mod type_display;
mod types;
mod union;
mod value_links;

#[cfg(any(test, feature = "storage-pilot"))]
mod census;
/// The P1 storage-families trace and census (`docs/S08-P1.md`).
#[cfg(any(test, feature = "storage-pilot"))]
pub mod storage_families;
/// A bounded constructor diagnostic; it does not expose checker-local identities.
#[cfg(feature = "storage-pilot")]
pub mod storage_pilot;

pub use flags::*;
pub use handles::{
    MemberSpec, NodeRef, RetainedNode, RetainedSignature, RetainedSymbol, RetainedType,
    RetainedTypeList, SignatureRef, SymbolRef, TypeRef,
};
pub use host::CheckerHost;
pub(crate) use ids::{AliasId, IndexInfoId, SignatureId, TypeId, TypePredicateId};
pub(crate) use init::Builtins;
pub use init::BUILTIN_TYPE_NAMES;
pub(crate) use key::CacheKey;
pub use links::{LinkKey, LinkStore};
pub use owner::{CheckerOwner, Operation};
#[cfg(test)]
use resolution::TypeResolution;
pub use resolution::TypeSystemPropertyName;
pub(crate) use resolution::{ResolutionStack, TypeSystemEntity};
#[cfg(any(test, feature = "storage-pilot"))]
pub(crate) use signatures::{IndexInfo, Signature};
pub(crate) use signatures::{SignatureStore, TypePredicate};
pub use state::CheckerOptions;
pub(crate) use state::CheckerState;
#[cfg(test)]
use type_display::{alias_symbol, alias_type_arguments};
pub use type_display::{
    to_node_builder_flags, DEFAULT_MAXIMUM_TRUNCATION_LENGTH, MAX_SERIALIZATION_LEVEL,
    NO_TRUNCATION_MAXIMUM_TRUNCATION_LENGTH,
};
pub use types::{element_flags, ElementFlags, TypeKind};
pub(crate) use types::{
    InterfaceData, IntrinsicData, LiteralData, LiteralValue, NumberKey, ObjectData, Payload,
    ReferenceData, SymbolList, TemplateLiteralData, TupleData, TupleElementInfo, TypeAlias,
    TypeList, TypeParameterData, TypeStore, UnionData, UnionOfUnionKey, UnionOrIntersectionMembers,
};
#[cfg(any(test, feature = "storage-pilot"))]
pub(crate) use types::{StructuredMembers, TypeRecord};
pub use union::UnionReduction;
pub(crate) use value_links::ValueSymbolLinks;

/// Checker-boundary failures. Arena identity, generation and bounds failures pass
/// through unchanged; the checker adds the failures only it can observe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Arena(ts_arena::Error),
    Printer(ts_printer::Error),
    /// The same thread asked for a second operation on an owner whose operation
    /// it already holds. Detected before waiting; ordinary contention waits.
    Reentry,
    /// A checker-local identity space is exhausted; IDs never wrap or recycle.
    IdExhausted,
    /// A required operation the port does not implement yet, by upstream name.
    /// A frozen case that reaches one is a named failure, never a pass.
    Unsupported(&'static str),
    /// A type of a kind this operation cannot take (upstream's `As*` panics).
    UnexpectedType {
        context: &'static str,
        kind: TypeKind,
    },
    /// A link upstream would have set before this read (a nil dereference there).
    MissingLink(&'static str),
}

impl From<ts_arena::Error> for Error {
    fn from(error: ts_arena::Error) -> Self {
        match error {
            ts_arena::Error::Reentry => Self::Reentry,
            error => Self::Arena(error),
        }
    }
}

impl From<ts_printer::Error> for Error {
    fn from(error: ts_printer::Error) -> Self {
        Self::Printer(error)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Arena(error) => error.fmt(output),
            Self::Printer(error) => error.fmt(output),
            Self::Reentry => {
                output.write_str("the checker operation is already held by this thread")
            }
            Self::IdExhausted => output.write_str("checker-local identity space exhausted"),
            Self::Unsupported(name) => write!(output, "unsupported checker operation: {name}"),
            Self::UnexpectedType { context, kind } => {
                write!(output, "unexpected {kind:?} type in {context}")
            }
            Self::MissingLink(what) => write!(output, "missing {what}"),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod families_tests;
#[cfg(test)]
mod flag_tests;
#[cfg(test)]
mod tests;
