//! Type display constants and the flag bridge (`tsc/internal/checker/printer.go`).
//!
//! `TypeToStringEx` builds a type node through the node builder and prints it
//! through `ts_printer`; a direct recursive formatter is not the plan. The
//! functions that do that arrive with the P2 slice. What is fixed now: the
//! serialization depth bound, the truncation lengths, and the mask that turns
//! `TypeFormatFlags` into node-builder flags without translation.

use crate::{type_format_flags, TypeAlias, TypeFormatFlags, TypeId};
use ts_arena::SymbolId;

/// Nested type serialization (diagnostics requesting types requesting members)
/// returns `"?"` beyond this depth (`checker.go`, `maxSerializationLevel`).
pub const MAX_SERIALIZATION_LEVEL: i32 = 2;
/// `nodebuilderimpl.go`, `defaultMaximumTruncationLength`.
pub const DEFAULT_MAXIMUM_TRUNCATION_LENGTH: usize = 160;
/// `nodebuilderimpl.go`, `noTruncationMaximumTruncationLength`.
pub const NO_TRUNCATION_MAXIMUM_TRUNCATION_LENGTH: usize = 1_000_000;

/// The bits of `TypeFormatFlags` that are node-builder flags at the same positions.
// port: tsc/internal/checker/printer.go:toNodeBuilderFlags
pub fn to_node_builder_flags(flags: TypeFormatFlags) -> ts_nodebuilder::Flags {
    flags & type_format_flags::NODE_BUILDER_FLAGS_MASK
}

/// Upstream's nil-tolerant accessor: an absent alias has no symbol.
// port: tsc/internal/checker/types.go:TypeAlias.Symbol
#[allow(dead_code, reason = "alias display callers arrive in P2")]
pub(crate) fn alias_symbol(alias: Option<&TypeAlias>) -> Option<SymbolId> {
    alias.map(|alias| alias.symbol)
}

/// Upstream's nil-tolerant accessor: an absent alias has no type arguments.
// port: tsc/internal/checker/types.go:TypeAlias.TypeArguments
#[allow(dead_code, reason = "alias display callers arrive in P2")]
pub(crate) fn alias_type_arguments(alias: Option<&TypeAlias>) -> &[TypeId] {
    alias.map_or(&[], |alias| &alias.type_arguments)
}
