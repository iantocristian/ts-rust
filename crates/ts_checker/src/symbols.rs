//! Checker-created symbols (`Checker.newSymbol` and `newSymbolEx` in
//! `tsc/internal/checker/checker.go`) and the checker's own symbol tables.
//!
//! Every symbol the checker creates is transient and lives in the checker's
//! symbol arena, whose arena identity is the checker identity (ADR 0007). File
//! symbols are never mutated; merges clone them here first (P2). Member tables
//! the checker builds (tuple targets, anonymous types, merged symbols) live in
//! the checker's `SymbolTables`, so their storage is disposed with the owner.

use crate::{CheckerState, Error};
use ts_arena::SymbolId;
use ts_ast::{symbol_flags, CheckFlags, JsString, Symbol, SymbolFlags, SymbolTable, SymbolTableId};

impl CheckerState {
    // port: tsc/internal/checker/checker.go:Checker.newSymbol
    pub(crate) fn new_symbol(
        &mut self,
        flags: SymbolFlags,
        name: JsString,
    ) -> Result<SymbolId, Error> {
        self.symbol_count = self.symbol_count.checked_add(1).ok_or(Error::IdExhausted)?;
        Ok(self
            .symbols
            .push(Symbol::new(flags | symbol_flags::TRANSIENT, name)))
    }

    // port: tsc/internal/checker/checker.go:Checker.newSymbolEx
    pub(crate) fn new_symbol_ex(
        &mut self,
        flags: SymbolFlags,
        name: JsString,
        check_flags: CheckFlags,
    ) -> Result<SymbolId, Error> {
        let result = self.new_symbol(flags, name)?;
        self.symbols.get_mut(result)?.check_flags = check_flags;
        Ok(result)
    }

    /// A symbol this checker can read: its own transient symbols now, file
    /// symbols through the retained program in P2.
    pub(crate) fn symbol(&self, id: SymbolId) -> Result<&Symbol, Error> {
        if id.arena() != self.symbols.id() {
            return Err(Error::Unsupported("file symbol access"));
        }
        Ok(self.symbols.get(id)?)
    }

    pub(crate) fn symbol_mut(&mut self, id: SymbolId) -> Result<&mut Symbol, Error> {
        if id.arena() != self.symbols.id() {
            return Err(Error::Unsupported("file symbol access"));
        }
        Ok(self.symbols.get_mut(id)?)
    }

    /// `ast.GetSymbolId`: the lazily assigned runtime identity cache keys use.
    pub(crate) fn symbol_runtime_id(&self, id: SymbolId) -> Result<u64, Error> {
        Ok(ts_ast::runtime_symbol_id(self.symbol(id)?))
    }

    /// Allocates a member table owned by this checker.
    pub(crate) fn alloc_symbol_table(&mut self, table: SymbolTable) -> SymbolTableId {
        self.tables.alloc(table)
    }
}
