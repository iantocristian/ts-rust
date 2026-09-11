//! Everything a checker mutates: the S08 subset of the `Checker` struct's fields
//! (`tsc/internal/checker/checker.go`). Reachable only through an
//! [`crate::Operation`], so inside it the checker is a plain single-threaded
//! `&mut` state machine (ADR 0008).

use crate::{
    Builtins, Error, LinkStore, ResolutionStack, SignatureStore, TypeId, TypeStore,
    ValueSymbolLinks,
};
use ts_arena::{CheckerIdentity, Counters, NodeId, SymbolArena, SymbolId};
use ts_ast::{AstBuilder, Symbol, SymbolTables};
use ts_jsstring::SourceText;

/// The compiler options the P1 constructors read. `strictNullChecks` and
/// `exactOptionalPropertyTypes` are upstream's effective values
/// (`GetStrictOptionValue`); the compiler host supplies them in P2.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CheckerOptions {
    pub strict_null_checks: bool,
    pub exact_optional_property_types: bool,
}

pub(crate) struct CheckerState {
    pub(crate) options: CheckerOptions,
    /// Checker-owned symbols: merged clones, transient and synthetic symbols.
    /// Its arena identity is the checker identity (`CheckerIdentity::id`).
    pub(crate) symbols: SymbolArena<Symbol>,
    /// `Checker.SymbolCount`.
    pub(crate) symbol_count: u32,
    /// Member tables the checker builds; disposed with the owner.
    pub(crate) tables: SymbolTables,
    pub(crate) types: TypeStore,
    pub(crate) signatures: SignatureStore,
    pub(crate) resolution: ResolutionStack,
    pub(crate) value_symbol_links: LinkStore<SymbolId, ValueSymbolLinks>,
    /// `Checker.factory`: the checker's own synthetic AST arena, distinct from
    /// the node builder's. Synthetic signature declarations and synthetic
    /// expressions live here and share the owner's lifetime (plan §4.3).
    pub(crate) factory: AstBuilder,
    /// The checker type a synthetic expression embeds (`SyntheticExpression.Type`).
    pub(crate) synthetic_expression_types: crate::types::Map<NodeId, TypeId>,
    pub(crate) builtins: Builtins,
}

impl CheckerState {
    /// Adopts the identity's reserved arena number as the checker symbol arena
    /// and runs the type-creating part of `NewChecker`.
    pub(crate) fn new(
        identity: &CheckerIdentity,
        counters: &Counters,
        options: CheckerOptions,
    ) -> Result<Self, Error> {
        let mut state = Self::bare(identity, counters, options)?;
        state.initialize()?;
        Ok(state)
    }

    /// The state before `NewChecker` creates anything: empty stores and sentinel
    /// builtins. Only the storage pilot, which numbers its own types from 1 the
    /// way its Go counterpart does, uses it without `initialize`.
    pub(crate) fn bare(
        identity: &CheckerIdentity,
        counters: &Counters,
        options: CheckerOptions,
    ) -> Result<Self, Error> {
        let symbols = identity.adopt_symbol_arena(counters)?;
        let builtins = Builtins::uninitialized(symbols.id());
        Ok(Self {
            options,
            symbols,
            symbol_count: 0,
            tables: SymbolTables::new(counters),
            types: TypeStore::new(),
            signatures: SignatureStore::new(),
            resolution: ResolutionStack::new(),
            value_symbol_links: LinkStore::new(),
            factory: AstBuilder::new(SourceText::from_bytes(&b""[..]), counters),
            synthetic_expression_types: crate::types::Map::default(),
            builtins,
        })
    }

    pub fn symbols(&self) -> &SymbolArena<Symbol> {
        &self.symbols
    }
    pub fn types(&self) -> &TypeStore {
        &self.types
    }
    pub fn signatures(&self) -> &SignatureStore {
        &self.signatures
    }
    pub fn resolution(&self) -> &ResolutionStack {
        &self.resolution
    }
    pub fn resolution_mut(&mut self) -> &mut ResolutionStack {
        &mut self.resolution
    }
    pub(crate) fn builtins(&self) -> &Builtins {
        &self.builtins
    }
}
