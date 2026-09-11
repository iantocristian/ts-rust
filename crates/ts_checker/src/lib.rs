//! The checker (`tsc/internal/checker`): checker-owned symbols, types and
//! signatures over S07's immutable bound files, with `.types` and `.errors.txt`
//! parity on the frozen subset as S08's acceptance.
//!
//! This is the S08 scaffold plus the P1 storage families. It fixes the contracts
//! the port builds on and ports the leaf mechanisms every later family depends
//! on. What exists:
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
//!   type references, type parameters, unions with literal reduction, template
//!   literal types, synthetic call signatures and synthetic expressions. Every
//!   constructor sorts and interns the way upstream does.
//! - Flags and enums transcribed from `types.go`, checked against values read
//!   out of the pinned Go package (`data/s08/checker-flag-observations.json`).
//! - [`CheckerHost`]: the program surface the checker requires, mapped method by
//!   method to what the loader already provides and what P2 still owes.
//!
//! Relations, program checking, declared types of source symbols, mapped types,
//! instantiation and diagnostics remain pending; asking for one is a named
//! `Error::Unsupported`, never a default.
//!
//! Design notes: `docs/design/symbols.md`, `docs/design/ownership.md`; plan:
//! `docs/S08-implementation-plan.md`.

mod compare;
mod construct;
mod flags;
mod handles;
mod host;
#[allow(
    dead_code,
    reason = "the numeric id readers of alias, index-info and predicate ids are used by the P2 display and diagnostics families"
)]
mod ids;
mod init;
mod key;
mod links;
mod owner;
#[allow(
    dead_code,
    reason = "resolution callers arrive with the first semantic checker slice"
)]
mod resolution;
#[allow(
    dead_code,
    reason = "signature readers arrive with the P3 signatures family"
)]
mod signatures;
#[allow(
    dead_code,
    reason = "state accessors are consumed by the P2 query slice"
)]
mod state;
mod symbols;
mod template;
mod type_display;
#[allow(
    dead_code,
    reason = "payload fields written here are read by the P2/P3 type families"
)]
mod types;
mod union;
#[allow(
    dead_code,
    reason = "value link fields beyond resolvedType are read by the P2 query slice"
)]
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
    RetainedTypeList, SignatureRef, TypeRef,
};
pub use host::CheckerHost;
pub(crate) use ids::{AliasId, IndexInfoId, SignatureId, TypeId, TypePredicateId};
pub(crate) use init::Builtins;
pub use init::BUILTIN_TYPE_NAMES;
pub(crate) use key::CacheKey;
pub use links::{LinkKey, LinkStore};
pub use owner::{CheckerOwner, Operation};
pub use resolution::TypeSystemPropertyName;
#[allow(
    unused_imports,
    reason = "remaining private resolution consumers arrive in P2"
)]
pub(crate) use resolution::{ResolutionStack, TypeResolution, TypeSystemEntity};
pub(crate) use signatures::{IndexInfo, Signature, SignatureStore, TypePredicate};
pub use state::CheckerOptions;
pub(crate) use state::CheckerState;
#[cfg(test)]
use type_display::{alias_symbol, alias_type_arguments};
pub use type_display::{
    to_node_builder_flags, DEFAULT_MAXIMUM_TRUNCATION_LENGTH, MAX_SERIALIZATION_LEVEL,
    NO_TRUNCATION_MAXIMUM_TRUNCATION_LENGTH,
};
pub use types::{element_flags, ElementFlags, TypeKind};
#[allow(
    unused_imports,
    reason = "payload families are consumed as their constructors and the census land"
)]
pub(crate) use types::{
    IndexInfoList, InterfaceData, IntersectionData, IntrinsicData, LiteralData, LiteralValue,
    NumberKey, ObjectData, Payload, ReferenceData, SignatureList, StructuredMembers, SymbolList,
    TemplateLiteralData, TupleData, TupleElementInfo, TypeAlias, TypeCaches, TypeList,
    TypeParameterData, TypeRecord, TypeStore, UnionData, UnionOfUnionKey,
    UnionOrIntersectionMembers, UniqueEsSymbolData,
};
pub use union::UnionReduction;
pub(crate) use value_links::ValueSymbolLinks;

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

impl std::fmt::Display for Error {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Arena(error) => error.fmt(output),
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
