//! Type records and aliases (`Type`, `TypeAlias` in `tsc/internal/checker/types.go`).
//!
//! Upstream's `Type` is a 56-byte header (flags, object flags, id, symbol, alias,
//! checker back-pointer and a `data` interface pointing at the payload struct)
//! embedded in every payload struct, so one Go type costs its payload struct plus
//! the alias record and any lists it owns. The checker back-pointer and the
//! self-referential interface are Go's cost, not the type system's.
//!
//! This store keeps a common record per type and provisional intrinsic/string
//! rows. Plan P1 measures real intrinsic, literal, object, union and tuple
//! families against the Go census before choosing rows, pages and list backing.
//! Nothing here promises a byte size; `TypeRecord` may grow a payload reference
//! or shrink its fields once that census exists.

use crate::{object_flags, type_flags, Error, ObjectFlags, TypeFlags, TypeId};
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Arc;
use ts_arena::SymbolId;
use ts_ast::JsString;

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
    intrinsics: Vec<IntrinsicData>,
    strings: Vec<StringLiteralData>,
    string_literals: HashMap<JsString, TypeId>,
}

#[derive(Debug)]
struct IntrinsicData {
    name: JsString,
}

#[derive(Debug)]
struct StringLiteralData {
    value: JsString,
    fresh: Option<TypeId>,
    regular: TypeId,
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

    // port: tsc/internal/checker/checker.go:Checker.newIntrinsicTypeEx
    pub(crate) fn new_intrinsic_type(
        &mut self,
        flags: TypeFlags,
        name: JsString,
        object_flags: ObjectFlags,
    ) -> Result<TypeId, Error> {
        let payload_row = u32::try_from(self.intrinsics.len()).map_err(|_| Error::IdExhausted)?;
        let flags_to_clear = object_flags::COULD_CONTAIN_TYPE_VARIABLES_COMPUTED
            | object_flags::COULD_CONTAIN_TYPE_VARIABLES
            | object_flags::MEMBERS_RESOLVED;
        let id = self.push(TypeRecord {
            flags,
            object_flags: object_flags & !flags_to_clear,
            symbol: None,
            alias: None,
            kind: TypeKind::Intrinsic,
            payload_row,
        })?;
        self.intrinsics.push(IntrinsicData { name });
        Ok(id)
    }

    // port: tsc/internal/checker/checker.go:Checker.getStringLiteralType
    pub(crate) fn string_literal_type(&mut self, value: JsString) -> Result<TypeId, Error> {
        if let Some(id) = self.string_literals.get(&value) {
            return Ok(*id);
        }
        let id = self.new_string_literal(value.clone(), None)?;
        self.string_literals.insert(value, id);
        Ok(id)
    }

    // The string branch of newLiteralType; other literal payload families remain pending.
    fn new_string_literal(
        &mut self,
        value: JsString,
        regular: Option<TypeId>,
    ) -> Result<TypeId, Error> {
        let payload_row = u32::try_from(self.strings.len()).map_err(|_| Error::IdExhausted)?;
        let id = self.push(TypeRecord {
            flags: type_flags::STRING_LITERAL,
            object_flags: object_flags::NONE,
            symbol: None,
            alias: None,
            kind: TypeKind::Literal,
            payload_row,
        })?;
        self.strings.push(StringLiteralData {
            value,
            regular: regular.unwrap_or(id),
            fresh: None,
        });
        Ok(id)
    }

    // Preserve getFreshTypeOfLiteralType's string branch, including self-links.
    pub(crate) fn fresh_string_literal_type(&mut self, id: TypeId) -> Result<TypeId, Error> {
        let record = *self
            .get(id)
            .ok_or(Error::Arena(ts_arena::Error::InvalidSlot))?;
        if record.flags != type_flags::STRING_LITERAL || record.kind != TypeKind::Literal {
            return Err(Error::Unsupported("fresh non-string literal type"));
        }
        let row = record.payload_row as usize;
        if let Some(fresh) = self.strings[row].fresh {
            return Ok(fresh);
        }
        let fresh = self.new_string_literal(self.strings[row].value.clone(), Some(id))?;
        self.records[fresh.index()].symbol = record.symbol;
        let fresh_row = self.records[fresh.index()].payload_row as usize;
        self.strings[fresh_row].fresh = Some(fresh);
        self.strings[row].fresh = Some(fresh);
        Ok(fresh)
    }
}

#[cfg(feature = "storage-pilot")]
impl TypeStore {
    pub(crate) fn pilot_observation(&self, id: TypeId) -> serde_json::Value {
        let record = &self.records[id.index()];
        let (kind, value, regular, fresh) = match record.kind {
            TypeKind::Intrinsic => (
                "intrinsic",
                &self.intrinsics[record.payload_row as usize].name,
                0,
                0,
            ),
            TypeKind::Literal => {
                let value = &self.strings[record.payload_row as usize];
                (
                    "string",
                    &value.value,
                    value.regular.get(),
                    value.fresh.map_or(0, TypeId::get),
                )
            }
            _ => unreachable!("pilot only constructs intrinsic and string literal payloads"),
        };
        serde_json::json!({"id": id.get(), "flags": record.flags, "kind": kind,
            "text_hex": crate::storage_pilot::hex(value.as_bytes()), "regular": regular, "fresh": fresh})
    }

    pub(crate) fn pilot_census(&self) -> serde_json::Value {
        serde_json::json!({
            "type_records": self.records.len(), "record_bytes": size_of::<TypeRecord>(),
            "record_capacity_bytes": self.records.capacity() * size_of::<TypeRecord>(),
            "intrinsic_records": self.intrinsics.len(), "intrinsic_row_bytes": size_of::<IntrinsicData>(),
            "intrinsic_capacity_bytes": self.intrinsics.capacity() * size_of::<IntrinsicData>(),
            "string_records": self.strings.len(), "string_row_bytes": size_of::<StringLiteralData>(),
            "string_capacity_bytes": self.strings.capacity() * size_of::<StringLiteralData>(),
            "string_cache_entries": self.string_literals.len(), "string_cache_capacity": self.string_literals.capacity(),
        })
    }
}

#[cfg(test)]
mod constructor_tests {
    use super::*;

    #[test]
    fn intrinsic_creation_clears_only_the_source_computed_bits() {
        let mut store = TypeStore::new();
        let id = store
            .new_intrinsic_type(type_flags::ANY, JsString::from_bytes(*b"any"), u32::MAX)
            .unwrap();
        let record = store.get(id).unwrap();
        assert_eq!(
            record.object_flags,
            !(object_flags::COULD_CONTAIN_TYPE_VARIABLES_COMPUTED
                | object_flags::COULD_CONTAIN_TYPE_VARIABLES
                | object_flags::MEMBERS_RESOLVED)
        );
        assert_eq!(record.flags, type_flags::ANY);
        assert_eq!(
            store.fresh_string_literal_type(id),
            Err(Error::Unsupported("fresh non-string literal type"))
        );
    }

    #[test]
    fn literal_interning_and_fresh_links_preserve_arbitrary_bytes_and_symbol() {
        let mut store = TypeStore::new();
        let regular = store
            .string_literal_type(JsString::from_bytes([0xed, 0xa0, 0x80, 0xff]))
            .unwrap();
        let duplicate = store
            .string_literal_type(JsString::from_bytes([0xed, 0xa0, 0x80, 0xff]))
            .unwrap();
        assert_eq!(regular, duplicate);
        let mut symbols = ts_arena::SymbolArena::new(&ts_arena::Counters::new());
        let symbol = symbols.push(());
        store.get_mut(regular).unwrap().symbol = Some(symbol);
        let fresh = store.fresh_string_literal_type(regular).unwrap();
        assert_ne!(regular, fresh);
        assert_eq!(store.fresh_string_literal_type(regular).unwrap(), fresh);
        assert_eq!(store.fresh_string_literal_type(fresh).unwrap(), fresh);
        assert_eq!(store.get(fresh).unwrap().symbol, Some(symbol));
        let data = &store.strings[store.get(fresh).unwrap().payload_row as usize];
        assert_eq!(data.regular, regular);
        assert_eq!(data.value.as_bytes(), &[0xed, 0xa0, 0x80, 0xff]);
        assert_eq!(store.len(), 2);
    }
}
