//! Type records and aliases (`Type`, `TypeAlias` in `tsc/internal/checker/types.go`).
//!
//! Upstream's `Type` is a 56-byte header (flags, object flags, id, symbol, alias,
//! checker back-pointer and a `data` interface pointing at the payload struct)
//! embedded in every payload struct, so one Go type costs its payload struct plus
//! the alias record and any lists it owns. The checker back-pointer and the
//! self-referential interface are Go's cost, not the type system's.
//!
//! This store keeps a small common record per type and leaves payload storage to
//! plan P1, which measures real intrinsic, literal, object, union and tuple
//! families against the Go census before choosing rows, pages and list backing.
//! Nothing here promises a byte size; `TypeRecord` may grow a payload reference
//! or shrink its fields once that census exists.

use crate::{Error, ObjectFlags, TypeFlags, TypeId};
use std::num::NonZeroU32;
use std::sync::Arc;
use ts_arena::SymbolId;

/// The kind of payload a type carries. Object kinds are further distinguished by
/// `ObjectFlags` (`ObjectFlagsObjectTypeKindMask`), as upstream does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TypeKind {
    Intrinsic,
    Literal,
    UniqueEsSymbol,
    Object,
    Union,
    Intersection,
    TypeParameter,
    Index,
    IndexedAccess,
    TemplateLiteral,
    StringMapping,
    Substitution,
    Conditional,
}

/// An alias record in this checker's alias store.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct AliasId(NonZeroU32);

impl AliasId {
    fn index(self) -> usize {
        self.0.get() as usize - 1
    }
}

/// The common record every type has. `symbol` may belong to a file or to this
/// checker; `alias` and `payload_row` are checker-local.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypeRecord {
    pub flags: TypeFlags,
    pub object_flags: ObjectFlags,
    pub symbol: Option<SymbolId>,
    pub alias: Option<AliasId>,
    pub kind: TypeKind,
    /// Row in the payload store for `kind`; provisional until P1.
    pub payload_row: u32,
}

/// `TypeAlias`: the alias symbol and its type arguments. The arguments are an
/// independently owned immutable list (ADR 0008).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeAlias {
    pub symbol: SymbolId,
    pub type_arguments: Arc<[TypeId]>,
}

#[derive(Debug, Default)]
pub struct TypeStore {
    records: Vec<TypeRecord>,
    aliases: Vec<TypeAlias>,
}

impl TypeStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Types created so far (`Checker.TypeCount`).
    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Appends a record and numbers it from 1 in creation order, as upstream's
    /// `TypeCount` does. Exhaustion is an error, never a wrap.
    pub fn push(&mut self, record: TypeRecord) -> Result<TypeId, Error> {
        let id = u32::try_from(self.records.len() + 1)
            .ok()
            .and_then(TypeId::new)
            .ok_or(Error::IdExhausted)?;
        self.records.push(record);
        Ok(id)
    }

    /// `None` for an id this store never issued. A same-numbered id from another
    /// checker is not distinguishable here; the operation scope keeps foreign ids
    /// out, and retained handles carry the exact owner (plan §4.1).
    pub fn get(&self, id: TypeId) -> Option<&TypeRecord> {
        self.records.get(id.index())
    }

    pub fn get_mut(&mut self, id: TypeId) -> Option<&mut TypeRecord> {
        self.records.get_mut(id.index())
    }

    pub fn push_alias(&mut self, alias: TypeAlias) -> Result<AliasId, Error> {
        let id = u32::try_from(self.aliases.len() + 1)
            .ok()
            .and_then(NonZeroU32::new)
            .map(AliasId)
            .ok_or(Error::IdExhausted)?;
        self.aliases.push(alias);
        Ok(id)
    }

    pub fn alias(&self, id: AliasId) -> Option<&TypeAlias> {
        self.aliases.get(id.index())
    }
}
