//! Type records, payload tables, lists and interning caches (`Type`, its
//! `TypeData` payloads and the `TypeAlias` record in
//! `tsc/internal/checker/types.go`; the type caches are `Checker` fields).
//!
//! Upstream's `Type` is a 56-byte header (flags, object flags, id, symbol, alias,
//! checker back-pointer and a `data` interface pointing at the payload struct)
//! embedded in every payload struct, so one Go type costs its payload struct plus
//! the alias record and any lists and maps it owns. The checker back-pointer and
//! the self-referential interface are Go's cost, not the type system's.
//!
//! Here every type has one common record and one row in the payload table for
//! its kind. The tables follow Go's payload structs one for one, including the
//! embedding chain `TupleType ⊃ InterfaceType ⊃ TypeReference ⊃ ObjectType ⊃
//! StructuredType`, so a per-family census compares like with like. Lists are
//! `Arc<[_]>` (ADR 0008): sharing one allocation where Go shares one slice.
//! Cache keys keep upstream's key byte stream (`key.rs`). P1 measures this
//! layout against the Go census; nothing here promises a byte size.

use crate::key::CacheKey;
use crate::{
    object_flags, AliasId, Error, IndexInfoId, ObjectFlags, SignatureId, TypeFlags, TypeId,
};
use std::hash::RandomState;
use std::sync::Arc;
use ts_arena::{NodeId, SymbolId};
use ts_ast::{JsString, SymbolTableId};
use ts_jsnum::{Number, PseudoBigInt};

/// An immutable, independently owned type list (`[]*Type` shared by reference).
pub type TypeList = Arc<[TypeId]>;
pub type SymbolList = Arc<[SymbolId]>;
pub type SignatureList = Arc<[SignatureId]>;
pub type IndexInfoList = Arc<[IndexInfoId]>;

/// The hash map the checker's caches use. `hashbrown` reports its exact
/// allocation size, which the storage census charges; the hasher is `std`'s.
pub type Map<K, V> = hashbrown::HashMap<K, V, RandomState>;

/// The Go payload struct a type carries. Object kinds are also distinguished by
/// `ObjectFlags` (`ObjectFlagsObjectTypeKindMask`), as upstream does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TypeKind {
    Intrinsic,
    Literal,
    UniqueEsSymbol,
    /// `ObjectType`: anonymous object types.
    Anonymous,
    /// `TypeReference`: instantiations of generic classes, interfaces and tuples.
    Reference,
    /// `InterfaceType`: originating class and interface types.
    Interface,
    /// `TupleType`: synthesized tuple targets.
    Tuple,
    Mapped,
    ReverseMapped,
    EvolvingArray,
    InstantiationExpression,
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

/// The common record every type has. `symbol` may belong to a file or to this
/// checker; `alias` and the payload row are checker-local.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypeRecord {
    pub flags: TypeFlags,
    pub object_flags: ObjectFlags,
    pub symbol: Option<SymbolId>,
    pub alias: Option<AliasId>,
    pub kind: TypeKind,
    pub(crate) payload_row: u32,
}

/// `TypeAlias`: the alias symbol and its type arguments.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeAlias {
    pub symbol: SymbolId,
    pub type_arguments: TypeList,
}

/// `LiteralType.value`: `string | jsnum.Number | bool | PseudoBigInt | nil`.
#[derive(Clone, Debug, PartialEq)]
pub enum LiteralValue {
    String(JsString),
    Number(Number),
    Boolean(bool),
    BigInt(PseudoBigInt),
    /// A computed enum member whose value is not known.
    #[allow(
        dead_code,
        reason = "P4 computed enum literal; retained in the P1 literal schema"
    )]
    ComputedEnum,
}

#[derive(Debug)]
pub struct IntrinsicData {
    pub name: JsString,
}

#[derive(Debug)]
pub struct LiteralData {
    pub value: LiteralValue,
    /// Fresh version of the type.
    pub fresh: Option<TypeId>,
    /// Regular version of the type.
    pub regular: TypeId,
}

#[derive(Debug)]
pub struct UniqueEsSymbolData {
    #[cfg_attr(
        not(any(test, feature = "storage-pilot")),
        allow(
            dead_code,
            reason = "P4 unique-symbol display name; the P1 census charges its owned bytes"
        )
    )]
    pub name: JsString,
}

/// `StructuredType` (with its embedded `ConstrainedType`): the member state
/// shared by object, union and intersection types.
#[derive(Debug, Default)]
pub struct StructuredMembers {
    #[cfg_attr(
        not(any(test, feature = "storage-pilot")),
        allow(
            dead_code,
            reason = "P3 constraint resolution link; the P1 census retains and charges its reachability"
        )
    )]
    pub resolved_base_constraint: Option<TypeId>,
    pub members: Option<SymbolTableId>,
    pub properties: Option<SymbolList>,
    /// Call signatures followed by construct signatures.
    pub signatures: Option<SignatureList>,
    pub call_signature_count: u32,
    pub index_infos: Option<IndexInfoList>,
    #[cfg_attr(
        not(any(test, feature = "storage-pilot")),
        allow(
            dead_code,
            reason = "P3 abstract-constructor filtering link; the P1 census tracks its reachability"
        )
    )]
    pub object_type_without_abstract_construct_signatures: Option<TypeId>,
}

/// `ObjectType`. The type mapper arrives with instantiation (P3).
#[derive(Debug, Default)]
pub struct ObjectData {
    pub structured: StructuredMembers,
    /// Target of an instantiated type.
    pub target: Option<TypeId>,
    /// Map of type instantiations.
    pub instantiations: Option<Box<Map<CacheKey, TypeId>>>,
}

/// `TypeReference`.
#[derive(Debug, Default)]
pub struct ReferenceData {
    pub object: ObjectData,
    /// TypeReferenceNode | ArrayTypeNode | TupleTypeNode when deferred.
    pub node: Option<NodeId>,
    pub resolved_type_arguments: Option<TypeList>,
}

/// `InterfaceType`.
#[derive(Debug, Default)]
pub struct InterfaceData {
    pub reference: ReferenceData,
    /// Type parameters (outer + local + thisType).
    pub all_type_parameters: Option<TypeList>,
    #[allow(
        dead_code,
        reason = "P3 outer type-parameter partition; retained in the P1 interface record layout"
    )]
    pub outer_type_parameter_count: u32,
    pub this_type: Option<TypeId>,
    #[allow(
        dead_code,
        reason = "P3 base-type resolution state; retained in the P1 interface record layout"
    )]
    pub base_types_resolved: bool,
    pub declared_members_resolved: bool,
    #[cfg_attr(
        not(any(test, feature = "storage-pilot")),
        allow(
            dead_code,
            reason = "P3 base-constructor resolution link; the P1 census tracks its reachability"
        )
    )]
    pub resolved_base_constructor_type: Option<TypeId>,
    #[cfg_attr(
        not(any(test, feature = "storage-pilot")),
        allow(
            dead_code,
            reason = "P3 interface base-type list; the P1 census charges its allocation and reachability"
        )
    )]
    pub resolved_base_types: Option<TypeList>,
    pub declared_members: Option<SymbolTableId>,
    #[cfg_attr(
        not(any(test, feature = "storage-pilot")),
        allow(
            dead_code,
            reason = "P3 interface call signatures; the P1 census charges the retained list"
        )
    )]
    pub declared_call_signatures: Option<SignatureList>,
    #[cfg_attr(
        not(any(test, feature = "storage-pilot")),
        allow(
            dead_code,
            reason = "P3 interface construct signatures; the P1 census charges the retained list"
        )
    )]
    pub declared_construct_signatures: Option<SignatureList>,
    #[cfg_attr(
        not(any(test, feature = "storage-pilot")),
        allow(
            dead_code,
            reason = "P3 interface index information; the P1 census charges the retained list"
        )
    )]
    pub declared_index_infos: Option<IndexInfoList>,
}

impl InterfaceData {
    // port: tsc/internal/checker/types.go:InterfaceType.TypeParameters
    pub fn type_parameters(&self) -> &[TypeId] {
        match &self.all_type_parameters {
            Some(all) if !all.is_empty() => &all[..all.len() - 1],
            _ => &[],
        }
    }
}

/// `ElementFlags` (`tsc/internal/checker/types.go`).
pub type ElementFlags = u32;

pub mod element_flags {
    use super::ElementFlags;

    pub const NONE: ElementFlags = 0;
    /// `T`
    pub const REQUIRED: ElementFlags = 1 << 0;
    /// `T?`
    pub const OPTIONAL: ElementFlags = 1 << 1;
    /// `...T[]`
    pub const REST: ElementFlags = 1 << 2;
    /// `...T`
    pub const VARIADIC: ElementFlags = 1 << 3;
    pub const FIXED: ElementFlags = REQUIRED | OPTIONAL;
    pub const VARIABLE: ElementFlags = REST | VARIADIC;
    pub const NON_REQUIRED: ElementFlags = OPTIONAL | REST | VARIADIC;
    pub const NON_REST: ElementFlags = REQUIRED | OPTIONAL | VARIADIC;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TupleElementInfo {
    pub flags: ElementFlags,
    /// NamedTupleMember | ParameterDeclaration.
    pub labeled_declaration: Option<NodeId>,
}

/// `TupleType`.
#[derive(Debug)]
pub struct TupleData {
    pub interface: InterfaceData,
    pub element_infos: Arc<[TupleElementInfo]>,
    /// Number of required or variadic elements.
    pub min_length: u32,
    /// Number of initial required or optional elements.
    pub fixed_length: u32,
    pub combined_flags: ElementFlags,
    pub readonly: bool,
}

/// `UnionOrIntersectionType`: the member state shared by unions and intersections.
#[derive(Debug, Default)]
pub struct UnionOrIntersectionMembers {
    pub structured: StructuredMembers,
    #[allow(
        dead_code,
        reason = "P3 union/intersection property cache; retained in the P1 record layout"
    )]
    pub property_cache: Option<SymbolTableId>,
    #[allow(
        dead_code,
        reason = "P3 property-augmentation cache; retained in the P1 record layout"
    )]
    pub property_cache_without_function_property_augment: Option<SymbolTableId>,
    #[cfg_attr(
        not(any(test, feature = "storage-pilot")),
        allow(
            dead_code,
            reason = "P3 union/intersection property list; the P1 census charges the retained list"
        )
    )]
    pub resolved_properties: Option<SymbolList>,
}

/// `UnionType`.
#[derive(Debug)]
pub struct UnionData {
    pub common: UnionOrIntersectionMembers,
    pub types: TypeList,
    #[cfg_attr(
        not(any(test, feature = "storage-pilot")),
        allow(
            dead_code,
            reason = "P3 union reduction link; the P1 census retains and charges its reachability"
        )
    )]
    pub resolved_reduced_type: Option<TypeId>,
    pub regular_type: Option<TypeId>,
    /// Denormalized union, intersection or index type in which the union originates.
    pub origin: Option<TypeId>,
    /// Property with unique unit type that exists in every object/intersection in the union.
    #[cfg_attr(
        not(any(test, feature = "storage-pilot")),
        allow(
            dead_code,
            reason = "P3 discriminant-property cache; the P1 census charges its owned bytes"
        )
    )]
    pub key_property_name: Option<JsString>,
    /// Constituents keyed by unit type discriminants.
    #[cfg_attr(
        not(any(test, feature = "storage-pilot")),
        allow(
            dead_code,
            reason = "P3 discriminant-constituent cache; the P1 census charges its allocation and reachability"
        )
    )]
    pub constituent_map: Option<Box<Map<TypeId, TypeId>>>,
}

/// `IntersectionType`.
#[derive(Debug)]
pub struct IntersectionData {
    pub common: UnionOrIntersectionMembers,
    pub types: TypeList,
    #[cfg_attr(
        not(any(test, feature = "storage-pilot")),
        allow(
            dead_code,
            reason = "P3 intersection apparent-type link; the P1 census tracks its reachability"
        )
    )]
    pub resolved_apparent_type: Option<TypeId>,
    /// Instantiation with type parameters mapped to the never type.
    #[cfg_attr(
        not(any(test, feature = "storage-pilot")),
        allow(
            dead_code,
            reason = "P3 intersection instantiation link; the P1 census tracks its reachability"
        )
    )]
    pub unique_literal_filled_instantiation: Option<TypeId>,
}

/// `TypeParameter`. The type mapper arrives with instantiation (P3).
#[derive(Debug, Default)]
pub struct TypeParameterData {
    #[cfg_attr(
        not(any(test, feature = "storage-pilot")),
        allow(
            dead_code,
            reason = "P3 constraint resolution link; the P1 census retains and charges its reachability"
        )
    )]
    pub resolved_base_constraint: Option<TypeId>,
    pub constraint: Option<TypeId>,
    pub target: Option<TypeId>,
    pub is_this_type: bool,
    #[cfg_attr(
        not(any(test, feature = "storage-pilot")),
        allow(
            dead_code,
            reason = "P3 generic default-type link; the P1 census tracks its reachability"
        )
    )]
    pub resolved_default_type: Option<TypeId>,
}

/// `TemplateLiteralType`: `texts` is always one longer than `types`.
#[derive(Debug)]
pub struct TemplateLiteralData {
    #[cfg_attr(
        not(any(test, feature = "storage-pilot")),
        allow(
            dead_code,
            reason = "P3 constraint resolution link; the P1 census retains and charges its reachability"
        )
    )]
    pub resolved_base_constraint: Option<TypeId>,
    pub texts: Arc<[JsString]>,
    pub types: TypeList,
}

/// The payload a new type carries; each variant fills one table.
#[derive(Debug)]
pub(crate) enum Payload {
    Intrinsic(IntrinsicData),
    Literal(LiteralData),
    #[allow(
        dead_code,
        reason = "P4 unique-symbol constructor; the P1 census retains this payload family"
    )]
    UniqueEsSymbol(UniqueEsSymbolData),
    Anonymous(ObjectData),
    Reference(ReferenceData),
    Interface(InterfaceData),
    Tuple(TupleData),
    Union(UnionData),
    #[allow(
        dead_code,
        reason = "P3 intersection constructor; the P1 census retains this payload family"
    )]
    Intersection(IntersectionData),
    TypeParameter(TypeParameterData),
    TemplateLiteral(TemplateLiteralData),
}

/// A `jsnum.Number` map key with Go's float-key semantics: `-0` and `+0` are one
/// key; NaN is never a key (`getNumberLiteralType` caches it separately).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NumberKey(u64);

impl NumberKey {
    pub fn new(value: Number) -> Option<Self> {
        let value = value.value();
        if value.is_nan() {
            return None;
        }
        Some(Self(if value == 0.0 { 0.0f64 } else { value }.to_bits()))
    }
}

/// `UnionOfUnionKey`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UnionOfUnionKey {
    pub id1: TypeId,
    pub id2: TypeId,
    pub reduction: crate::UnionReduction,
    pub alias: CacheKey,
}

/// The checker's type interning caches (`Checker.*Types` maps).
#[derive(Debug, Default)]
pub struct TypeCaches {
    pub string_literal_types: Map<JsString, TypeId>,
    pub number_literal_types: Map<NumberKey, TypeId>,
    pub nan_type: Option<TypeId>,
    pub bigint_literal_types: Map<PseudoBigInt, TypeId>,
    pub union_types: Map<CacheKey, TypeId>,
    pub union_of_union_types: Map<UnionOfUnionKey, TypeId>,
    pub tuple_types: Map<CacheKey, TypeId>,
    #[cfg_attr(
        not(any(test, feature = "storage-pilot")),
        allow(
            dead_code,
            reason = "P3 intersection interning cache; the P1 census charges its allocation and reachability"
        )
    )]
    pub intersection_types: Map<CacheKey, TypeId>,
    pub template_literal_types: Map<CacheKey, TypeId>,
}

#[derive(Debug, Default)]
pub struct TypeStore {
    /// Ids start above this; only tests move it, to reach exhaustion.
    base: u32,
    records: Vec<TypeRecord>,
    aliases: Vec<TypeAlias>,
    intrinsics: Vec<IntrinsicData>,
    literals: Vec<LiteralData>,
    unique_symbols: Vec<UniqueEsSymbolData>,
    anonymous: Vec<ObjectData>,
    references: Vec<ReferenceData>,
    interfaces: Vec<InterfaceData>,
    tuples: Vec<TupleData>,
    unions: Vec<UnionData>,
    intersections: Vec<IntersectionData>,
    type_parameters: Vec<TypeParameterData>,
    template_literals: Vec<TemplateLiteralData>,
    pub(crate) caches: TypeCaches,
}

fn invalid() -> Error {
    Error::Arena(ts_arena::Error::InvalidSlot)
}

fn row(row: u32) -> usize {
    row as usize
}

macro_rules! payload_accessors {
    (read $(#[$attr:meta])* $get:ident, $table:ident, $kind:ident, $data:ty) => {
        $(#[$attr])*
        pub fn $get(&self, id: TypeId) -> Result<&$data, Error> {
            let record = self.get(id)?;
            if record.kind != TypeKind::$kind {
                return Err(Error::UnexpectedType {
                    context: stringify!($get),
                    kind: record.kind,
                });
            }
            self.$table.get(row(record.payload_row)).ok_or_else(invalid)
        }

    };
    (write $get_mut:ident, $table:ident, $kind:ident, $data:ty) => {
        pub fn $get_mut(&mut self, id: TypeId) -> Result<&mut $data, Error> {
            let record = *self.get(id)?;
            if record.kind != TypeKind::$kind {
                return Err(Error::UnexpectedType {
                    context: stringify!($get_mut),
                    kind: record.kind,
                });
            }
            self.$table
                .get_mut(row(record.payload_row))
                .ok_or_else(invalid)
        }
    };
}

impl TypeStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// A store whose next id is `base + 1`; tests use it to reach exhaustion
    /// without allocating four billion records.
    #[cfg(test)]
    pub(crate) fn with_base_for_test(base: u32) -> Self {
        Self {
            base,
            ..Self::default()
        }
    }

    /// Types created so far (`Checker.TypeCount`).
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// The id the next `new_type` call will issue.
    pub(crate) fn next_id(&self) -> Result<TypeId, Error> {
        TypeId::next(self.base, self.records.len())
    }

    fn index(&self, id: TypeId) -> Result<usize, Error> {
        id.index(self.base)
            .filter(|index| *index < self.records.len())
            .ok_or_else(invalid)
    }

    /// `Err` for an id this store never issued. A same-numbered id from another
    /// checker is not distinguishable here; the operation scope keeps foreign ids
    /// out, and retained handles carry the exact owner (plan §4.1).
    pub fn get(&self, id: TypeId) -> Result<&TypeRecord, Error> {
        let index = self.index(id)?;
        Ok(&self.records[index])
    }

    pub fn get_mut(&mut self, id: TypeId) -> Result<&mut TypeRecord, Error> {
        let index = self.index(id)?;
        Ok(&mut self.records[index])
    }

    pub fn flags(&self, id: TypeId) -> Result<TypeFlags, Error> {
        Ok(self.get(id)?.flags)
    }

    pub fn object_flags(&self, id: TypeId) -> Result<ObjectFlags, Error> {
        Ok(self.get(id)?.object_flags)
    }

    /// Appends the common record and the payload row, numbering the type from 1
    /// in creation order as `TypeCount` does, and clearing the same computed
    /// object-flag bits. Exhaustion is an error, never a wrap.
    // port: tsc/internal/checker/checker.go:Checker.newType
    pub(crate) fn new_type(
        &mut self,
        flags: TypeFlags,
        object_flags: ObjectFlags,
        payload: Payload,
    ) -> Result<TypeId, Error> {
        let id = TypeId::next(self.base, self.records.len())?;
        let flags_to_clear = object_flags::COULD_CONTAIN_TYPE_VARIABLES_COMPUTED
            | object_flags::COULD_CONTAIN_TYPE_VARIABLES
            | object_flags::MEMBERS_RESOLVED;
        let (kind, payload_row) = self.push_payload(payload)?;
        self.records.push(TypeRecord {
            flags,
            object_flags: object_flags & !flags_to_clear,
            symbol: None,
            alias: None,
            kind,
            payload_row,
        });
        Ok(id)
    }

    fn push_payload(&mut self, payload: Payload) -> Result<(TypeKind, u32), Error> {
        fn push<T>(table: &mut Vec<T>, value: T) -> Result<u32, Error> {
            let row = u32::try_from(table.len()).map_err(|_| Error::IdExhausted)?;
            table.push(value);
            Ok(row)
        }
        Ok(match payload {
            Payload::Intrinsic(data) => (TypeKind::Intrinsic, push(&mut self.intrinsics, data)?),
            Payload::Literal(data) => (TypeKind::Literal, push(&mut self.literals, data)?),
            Payload::UniqueEsSymbol(data) => (
                TypeKind::UniqueEsSymbol,
                push(&mut self.unique_symbols, data)?,
            ),
            Payload::Anonymous(data) => (TypeKind::Anonymous, push(&mut self.anonymous, data)?),
            Payload::Reference(data) => (TypeKind::Reference, push(&mut self.references, data)?),
            Payload::Interface(data) => (TypeKind::Interface, push(&mut self.interfaces, data)?),
            Payload::Tuple(data) => (TypeKind::Tuple, push(&mut self.tuples, data)?),
            Payload::Union(data) => (TypeKind::Union, push(&mut self.unions, data)?),
            Payload::Intersection(data) => {
                (TypeKind::Intersection, push(&mut self.intersections, data)?)
            }
            Payload::TypeParameter(data) => (
                TypeKind::TypeParameter,
                push(&mut self.type_parameters, data)?,
            ),
            Payload::TemplateLiteral(data) => (
                TypeKind::TemplateLiteral,
                push(&mut self.template_literals, data)?,
            ),
        })
    }

    pub fn push_alias(&mut self, alias: TypeAlias) -> Result<AliasId, Error> {
        let id = AliasId::next(0, self.aliases.len())?;
        self.aliases.push(alias);
        Ok(id)
    }

    pub fn alias(&self, id: AliasId) -> Result<&TypeAlias, Error> {
        id.index(0)
            .and_then(|index| self.aliases.get(index))
            .ok_or_else(invalid)
    }

    /// The type's alias record, if it has one (`Type.Alias()`).
    pub fn alias_of(&self, id: TypeId) -> Result<Option<&TypeAlias>, Error> {
        self.get(id)?
            .alias
            .map(|alias| self.alias(alias))
            .transpose()
    }

    payload_accessors!(read intrinsic, intrinsics, Intrinsic, IntrinsicData);
    payload_accessors!(read literal, literals, Literal, LiteralData);
    payload_accessors!(write literal_mut, literals, Literal, LiteralData);
    payload_accessors!(read tuple, tuples, Tuple, TupleData);
    payload_accessors!(write tuple_mut, tuples, Tuple, TupleData);
    payload_accessors!(read union, unions, Union, UnionData);
    payload_accessors!(write union_mut, unions, Union, UnionData);
    payload_accessors!(
        read #[cfg(any(test, feature = "storage-pilot"))]
        intersection, intersections, Intersection, IntersectionData
    );
    payload_accessors!(
        read #[cfg(any(test, feature = "storage-pilot"))]
        type_parameter, type_parameters, TypeParameter, TypeParameterData
    );
    payload_accessors!(write type_parameter_mut, type_parameters, TypeParameter, TypeParameterData);
    payload_accessors!(read template_literal, template_literals, TemplateLiteral, TemplateLiteralData);

    /// `Type.AsObjectType()`: the `ObjectType` part of any object kind.
    pub fn object(&self, id: TypeId) -> Result<&ObjectData, Error> {
        let record = self.get(id)?;
        let index = row(record.payload_row);
        match record.kind {
            TypeKind::Anonymous => self.anonymous.get(index),
            TypeKind::Reference => self.references.get(index).map(|data| &data.object),
            TypeKind::Interface => self
                .interfaces
                .get(index)
                .map(|data| &data.reference.object),
            TypeKind::Tuple => self
                .tuples
                .get(index)
                .map(|data| &data.interface.reference.object),
            kind => {
                return Err(Error::UnexpectedType {
                    context: "AsObjectType",
                    kind,
                });
            }
        }
        .ok_or_else(invalid)
    }

    pub fn object_mut(&mut self, id: TypeId) -> Result<&mut ObjectData, Error> {
        let record = *self.get(id)?;
        let index = row(record.payload_row);
        match record.kind {
            TypeKind::Anonymous => self.anonymous.get_mut(index),
            TypeKind::Reference => self.references.get_mut(index).map(|data| &mut data.object),
            TypeKind::Interface => self
                .interfaces
                .get_mut(index)
                .map(|data| &mut data.reference.object),
            TypeKind::Tuple => self
                .tuples
                .get_mut(index)
                .map(|data| &mut data.interface.reference.object),
            kind => {
                return Err(Error::UnexpectedType {
                    context: "AsObjectType",
                    kind,
                });
            }
        }
        .ok_or_else(invalid)
    }

    /// `Type.AsTypeReference()`: the `TypeReference` part of a reference,
    /// interface or tuple type.
    pub fn type_reference(&self, id: TypeId) -> Result<&ReferenceData, Error> {
        let record = self.get(id)?;
        let index = row(record.payload_row);
        match record.kind {
            TypeKind::Reference => self.references.get(index),
            TypeKind::Interface => self.interfaces.get(index).map(|data| &data.reference),
            TypeKind::Tuple => self.tuples.get(index).map(|data| &data.interface.reference),
            kind => {
                return Err(Error::UnexpectedType {
                    context: "AsTypeReference",
                    kind,
                });
            }
        }
        .ok_or_else(invalid)
    }

    pub fn type_reference_mut(&mut self, id: TypeId) -> Result<&mut ReferenceData, Error> {
        let record = *self.get(id)?;
        let index = row(record.payload_row);
        match record.kind {
            TypeKind::Reference => self.references.get_mut(index),
            TypeKind::Interface => self
                .interfaces
                .get_mut(index)
                .map(|data| &mut data.reference),
            TypeKind::Tuple => self
                .tuples
                .get_mut(index)
                .map(|data| &mut data.interface.reference),
            kind => {
                return Err(Error::UnexpectedType {
                    context: "AsTypeReference",
                    kind,
                });
            }
        }
        .ok_or_else(invalid)
    }

    /// `Type.AsInterfaceType()`: the `InterfaceType` part of an interface or tuple.
    pub fn interface(&self, id: TypeId) -> Result<&InterfaceData, Error> {
        let record = self.get(id)?;
        let index = row(record.payload_row);
        match record.kind {
            TypeKind::Interface => self.interfaces.get(index),
            TypeKind::Tuple => self.tuples.get(index).map(|data| &data.interface),
            kind => {
                return Err(Error::UnexpectedType {
                    context: "AsInterfaceType",
                    kind,
                });
            }
        }
        .ok_or_else(invalid)
    }

    pub fn interface_mut(&mut self, id: TypeId) -> Result<&mut InterfaceData, Error> {
        let record = *self.get(id)?;
        let index = row(record.payload_row);
        match record.kind {
            TypeKind::Interface => self.interfaces.get_mut(index),
            TypeKind::Tuple => self.tuples.get_mut(index).map(|data| &mut data.interface),
            kind => {
                return Err(Error::UnexpectedType {
                    context: "AsInterfaceType",
                    kind,
                });
            }
        }
        .ok_or_else(invalid)
    }

    /// `Type.AsStructuredType()`: member state of object, union and intersection types.
    pub fn structured(&self, id: TypeId) -> Result<&StructuredMembers, Error> {
        let record = self.get(id)?;
        let index = row(record.payload_row);
        match record.kind {
            TypeKind::Union => self.unions.get(index).map(|data| &data.common.structured),
            TypeKind::Intersection => self
                .intersections
                .get(index)
                .map(|data| &data.common.structured),
            _ => return self.object(id).map(|object| &object.structured),
        }
        .ok_or_else(invalid)
    }

    pub fn structured_mut(&mut self, id: TypeId) -> Result<&mut StructuredMembers, Error> {
        let record = *self.get(id)?;
        let index = row(record.payload_row);
        match record.kind {
            TypeKind::Union => self
                .unions
                .get_mut(index)
                .map(|data| &mut data.common.structured),
            TypeKind::Intersection => self
                .intersections
                .get_mut(index)
                .map(|data| &mut data.common.structured),
            _ => return self.object_mut(id).map(|object| &mut object.structured),
        }
        .ok_or_else(invalid)
    }

    /// `Type.Types()`: the constituents of a union or intersection, else empty.
    // port: tsc/internal/checker/types.go:Type.Types
    pub fn types_of(&self, id: TypeId) -> Result<&[TypeId], Error> {
        let record = self.get(id)?;
        let index = row(record.payload_row);
        Ok(match record.kind {
            TypeKind::Union => &self.unions.get(index).ok_or_else(invalid)?.types,
            TypeKind::Intersection => &self.intersections.get(index).ok_or_else(invalid)?.types,
            _ => &[],
        })
    }

    /// `Type.Target()`: the target of a reference, interface, tuple or type
    /// parameter, else the type itself.
    // port: tsc/internal/checker/types.go:Type.Target
    pub fn target(&self, id: TypeId) -> Result<TypeId, Error> {
        let record = self.get(id)?;
        let index = row(record.payload_row);
        let target = match record.kind {
            TypeKind::Reference | TypeKind::Interface | TypeKind::Tuple => {
                self.type_reference(id)?.object.target
            }
            TypeKind::TypeParameter => self.type_parameters.get(index).ok_or_else(invalid)?.target,
            TypeKind::Index | TypeKind::StringMapping => {
                return Err(Error::Unsupported(
                    "Type.Target for index and string mapping types",
                ));
            }
            _ => return Ok(id),
        };
        target.ok_or(Error::MissingLink("Type.Target"))
    }

    #[cfg(any(test, feature = "storage-pilot"))]
    pub(crate) fn tables(&self) -> TableView<'_> {
        TableView {
            records: &self.records,
            aliases: &self.aliases,
            intrinsics: &self.intrinsics,
            literals: &self.literals,
            unique_symbols: &self.unique_symbols,
            anonymous: &self.anonymous,
            references: &self.references,
            interfaces: &self.interfaces,
            tuples: &self.tuples,
            unions: &self.unions,
            intersections: &self.intersections,
            type_parameters: &self.type_parameters,
            template_literals: &self.template_literals,
        }
    }
}

/// Read access to every table, for the census.
#[cfg(any(test, feature = "storage-pilot"))]
pub(crate) struct TableView<'a> {
    pub records: &'a Vec<TypeRecord>,
    pub aliases: &'a Vec<TypeAlias>,
    pub intrinsics: &'a Vec<IntrinsicData>,
    pub literals: &'a Vec<LiteralData>,
    pub unique_symbols: &'a Vec<UniqueEsSymbolData>,
    pub anonymous: &'a Vec<ObjectData>,
    pub references: &'a Vec<ReferenceData>,
    pub interfaces: &'a Vec<InterfaceData>,
    pub tuples: &'a Vec<TupleData>,
    pub unions: &'a Vec<UnionData>,
    pub intersections: &'a Vec<IntersectionData>,
    pub type_parameters: &'a Vec<TypeParameterData>,
    pub template_literals: &'a Vec<TemplateLiteralData>,
}

#[cfg(test)]
mod store_tests {
    use super::*;
    use crate::type_flags;

    #[test]
    fn ids_start_at_one_and_exhaustion_precedes_any_write() {
        let mut store = TypeStore::new();
        let any = store
            .new_type(
                type_flags::ANY,
                object_flags::COULD_CONTAIN_TYPE_VARIABLES | object_flags::NON_INFERRABLE_TYPE,
                Payload::Intrinsic(IntrinsicData {
                    name: JsString::from_bytes(&b"any"[..]),
                }),
            )
            .unwrap();
        assert_eq!(any.get(), 1, "type ids start at 1 like TypeCount");
        assert_eq!(
            store.get(any).unwrap().object_flags,
            object_flags::NON_INFERRABLE_TYPE,
            "newType clears the computed bits only"
        );
        assert_eq!(store.intrinsic(any).unwrap().name.as_bytes(), b"any");
        assert!(
            store.literal(any).is_err(),
            "payload access checks the kind"
        );
        assert!(store.get(TypeId::new(2).unwrap()).is_err());

        let mut near_end = TypeStore::with_base_for_test(u32::MAX - 2);
        let first = near_end
            .new_type(
                type_flags::NEVER,
                0,
                Payload::Intrinsic(IntrinsicData {
                    name: JsString::from_bytes(&b"never"[..]),
                }),
            )
            .unwrap();
        assert_eq!(first.get(), u32::MAX - 1);
        let last = near_end
            .new_type(
                type_flags::NEVER,
                0,
                Payload::Intrinsic(IntrinsicData {
                    name: JsString::from_bytes(&b"never"[..]),
                }),
            )
            .unwrap();
        assert_eq!(last.get(), u32::MAX);
        assert_eq!(
            near_end
                .new_type(
                    type_flags::NEVER,
                    0,
                    Payload::Intrinsic(IntrinsicData {
                        name: JsString::from_bytes(&b"never"[..]),
                    }),
                )
                .err(),
            Some(Error::IdExhausted)
        );
        assert_eq!(near_end.len(), 2, "the failed allocation wrote nothing");
        assert_eq!(near_end.tables().intrinsics.len(), 2);
        assert!(near_end.get(first).is_ok() && near_end.get(last).is_ok());
    }

    #[test]
    fn number_keys_follow_go_float_map_semantics() {
        assert_eq!(
            NumberKey::new(Number::new(0.0)),
            NumberKey::new(Number::new(-0.0)),
            "+0 and -0 are one map key in Go"
        );
        assert_eq!(NumberKey::new(Number::new(f64::NAN)), None);
        assert_ne!(
            NumberKey::new(Number::new(1.0)),
            NumberKey::new(Number::new(2.0))
        );
    }
}
