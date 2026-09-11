//! Structural storage census of one checker, following the accounting frozen in
//! `data/s08/type-footprint.json`: every allocation is charged once, to one
//! family, with its full capacity. Type families carge their headers, payload
//! rows, owned lists, owned text and interning caches; the remaining checker
//! storage is reported beside them. A family the adapter cannot measure is
//! listed as unavailable, never charged as zero.
//!
//! The Go adapter (`tools/s08/oracle/storage_families_bridge.go`) reports the
//! same family names over the same live roots, so the two reports compare like
//! with like.

use crate::{CheckerState, LiteralValue, TypeId, TypeKind, TypeList};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;
use ts_ast::JsString;

/// `Arc<[T]>` and `Arc<[u8]>` allocations carry the strong and weak counts.
const ARC_HEADER: usize = 16;

#[derive(Default)]
struct Family {
    count: usize,
    bytes: usize,
}

#[derive(Default)]
pub(crate) struct Census {
    families: BTreeMap<&'static str, Family>,
    /// Allocations already charged, by address.
    seen: HashSet<usize>,
    unavailable: Vec<&'static str>,
}

impl Census {
    fn add(&mut self, family: &'static str, count: usize, bytes: usize) {
        let entry = self.families.entry(family).or_default();
        entry.count += count;
        entry.bytes += bytes;
    }

    fn list<T>(&mut self, family: &'static str, list: &Arc<[T]>) {
        if self.seen.insert(list.as_ptr() as usize) {
            self.add(family, 0, ARC_HEADER + list.len() * size_of::<T>());
        }
    }

    fn text(&mut self, family: &'static str, text: &JsString) {
        if self.seen.insert(text.as_bytes().as_ptr() as usize) {
            self.add(family, 0, ARC_HEADER + text.len());
        }
    }

    fn vec_capacity<T>(&mut self, family: &'static str, vec: &[T], capacity: usize) {
        self.add(family, vec.len(), capacity * size_of::<T>());
    }

    fn finish(self, type_families: &[&str], created: usize, reachable: usize) -> Value {
        let type_storage: usize = self
            .families
            .iter()
            .filter(|(name, _)| type_families.contains(name))
            .map(|(_, family)| family.bytes)
            .sum();
        let total: usize = self.families.values().map(|family| family.bytes).sum();
        json!({
            "families": self.families.iter().map(|(name, family)| {
                (name.to_string(), json!({"count": family.count, "bytes": family.bytes}))
            }).collect::<serde_json::Map<String, Value>>(),
            "type_storage_bytes": type_storage,
            "checker_bytes": total,
            "types": {"created": created, "reachable": reachable, "unreachable_occupied": created - reachable},
            "unavailable": self.unavailable,
            "record_sizes": {
                "TypeRecord": size_of::<crate::TypeRecord>(), "IntrinsicData": size_of::<crate::IntrinsicData>(),
                "LiteralData": size_of::<crate::LiteralData>(), "ObjectData": size_of::<crate::ObjectData>(),
                "ReferenceData": size_of::<crate::ReferenceData>(), "InterfaceData": size_of::<crate::InterfaceData>(),
                "TupleData": size_of::<crate::TupleData>(), "UnionData": size_of::<crate::UnionData>(),
                "TypeParameterData": size_of::<crate::TypeParameterData>(), "TemplateLiteralData": size_of::<crate::TemplateLiteralData>(),
                "TypeAlias": size_of::<crate::TypeAlias>(), "Signature": size_of::<crate::Signature>(),
                "IndexInfo": size_of::<crate::IndexInfo>(), "Symbol": size_of::<ts_ast::Symbol>(),
                "ValueSymbolLinks": size_of::<crate::ValueSymbolLinks>(), "OptionValueSymbolLinks": size_of::<Option<crate::ValueSymbolLinks>>(),
            },
        })
    }
}

/// Every family both runtimes report, so an absent family is a mismatch, not a zero.
pub(crate) const ALL_FAMILIES: [&str; 23] = [
    "type_records",
    "intrinsic",
    "literal",
    "unique_es_symbol",
    "anonymous",
    "reference",
    "interface",
    "tuple",
    "union",
    "intersection",
    "type_parameter",
    "template_literal",
    "alias",
    "type_lists",
    "type_caches",
    "symbols",
    "symbol_tables",
    "signatures",
    "index_infos",
    "type_predicates",
    "value_symbol_links",
    "synthetic_expression_links",
    "checker_ast",
];

/// Type families whose bytes sum to the footprint statistic's numerator.
pub(crate) const TYPE_FAMILIES: [&str; 15] = [
    "type_records",
    "intrinsic",
    "literal",
    "unique_es_symbol",
    "anonymous",
    "reference",
    "interface",
    "tuple",
    "union",
    "intersection",
    "type_parameter",
    "template_literal",
    "alias",
    "type_lists",
    "type_caches",
];

impl CheckerState {
    /// The census over this checker with `roots` as the retained results.
    pub(crate) fn census(&self, roots: &[TypeId]) -> Value {
        let mut census = Census::default();
        for family in ALL_FAMILIES {
            census.add(family, 0, 0);
        }
        let tables = self.types.tables();
        census.vec_capacity("type_records", tables.records, tables.records.capacity());
        census.vec_capacity("intrinsic", tables.intrinsics, tables.intrinsics.capacity());
        for data in tables.intrinsics {
            census.text("intrinsic", &data.name);
        }
        census.vec_capacity("literal", tables.literals, tables.literals.capacity());
        for data in tables.literals {
            match &data.value {
                LiteralValue::String(text) => census.text("literal", text),
                LiteralValue::BigInt(value) => {
                    census.add("literal", 0, value.base10_value.capacity());
                }
                LiteralValue::Number(_) | LiteralValue::Boolean(_) | LiteralValue::ComputedEnum => {
                }
            }
        }
        census.vec_capacity(
            "unique_es_symbol",
            tables.unique_symbols,
            tables.unique_symbols.capacity(),
        );
        for data in tables.unique_symbols {
            census.text("unique_es_symbol", &data.name);
        }
        census.vec_capacity("anonymous", tables.anonymous, tables.anonymous.capacity());
        for data in tables.anonymous {
            Self::census_object(&mut census, "anonymous", data);
        }
        census.vec_capacity("reference", tables.references, tables.references.capacity());
        for data in tables.references {
            Self::census_reference(&mut census, "reference", data);
        }
        census.vec_capacity("interface", tables.interfaces, tables.interfaces.capacity());
        for data in tables.interfaces {
            Self::census_interface(&mut census, "interface", data);
        }
        census.vec_capacity("tuple", tables.tuples, tables.tuples.capacity());
        for data in tables.tuples {
            Self::census_interface(&mut census, "tuple", &data.interface);
            if census.seen.insert(data.element_infos.as_ptr() as usize) {
                census.add(
                    "tuple",
                    0,
                    ARC_HEADER + data.element_infos.len() * size_of::<crate::TupleElementInfo>(),
                );
            }
        }
        census.vec_capacity("union", tables.unions, tables.unions.capacity());
        for data in tables.unions {
            census.list("type_lists", &data.types);
            Self::census_union_common(&mut census, &data.common);
            if let Some(name) = &data.key_property_name {
                census.text("union", name);
            }
            if let Some(map) = &data.constituent_map {
                census.add(
                    "union",
                    0,
                    size_of::<crate::types::Map<TypeId, TypeId>>() + map.allocation_size(),
                );
            }
        }
        census.vec_capacity(
            "intersection",
            tables.intersections,
            tables.intersections.capacity(),
        );
        for data in tables.intersections {
            census.list("type_lists", &data.types);
            Self::census_union_common(&mut census, &data.common);
        }
        census.vec_capacity(
            "type_parameter",
            tables.type_parameters,
            tables.type_parameters.capacity(),
        );
        census.vec_capacity(
            "template_literal",
            tables.template_literals,
            tables.template_literals.capacity(),
        );
        for data in tables.template_literals {
            census.list("type_lists", &data.types);
            if census.seen.insert(data.texts.as_ptr() as usize) {
                census.add(
                    "template_literal",
                    0,
                    ARC_HEADER + data.texts.len() * size_of::<JsString>(),
                );
            }
            for text in data.texts.iter() {
                census.text("template_literal", text);
            }
        }
        census.vec_capacity("alias", tables.aliases, tables.aliases.capacity());
        for alias in tables.aliases {
            census.list("type_lists", &alias.type_arguments);
        }
        self.census_caches(&mut census);

        // Storage beside the type families.
        census.add(
            "symbols",
            self.symbols.len(),
            self.symbols.structural_bytes(),
        );
        for (_, symbol) in self.symbols.iter() {
            census.text("symbols", &symbol.name);
        }
        census.add(
            "symbol_tables",
            self.tables.table_count(),
            self.tables.structural_bytes(),
        );
        let (signature_capacity, index_info_capacity, predicate_capacity) =
            self.signatures.capacities();
        census.add(
            "signatures",
            self.signatures.len(),
            signature_capacity * size_of::<crate::Signature>(),
        );
        for index in 0..self.signatures.len() {
            let id = crate::SignatureId::new(index as u32 + 1).expect("nonzero");
            let signature = self.signatures.get(id).expect("published signature");
            if let Some(list) = &signature.type_parameters {
                census.list("type_lists", list);
            }
            if let Some(list) = &signature.parameters {
                census.list("signatures", list);
            }
        }
        census.add(
            "index_infos",
            self.signatures.index_info_count(),
            index_info_capacity * size_of::<crate::IndexInfo>(),
        );
        census.add(
            "type_predicates",
            self.signatures.predicate_count(),
            predicate_capacity * size_of::<crate::TypePredicate>(),
        );
        for index in 0..self.signatures.predicate_count() {
            let id = crate::TypePredicateId::new(index as u32 + 1).expect("nonzero");
            census.text(
                "type_predicates",
                &self
                    .signatures
                    .predicate(id)
                    .expect("published predicate")
                    .parameter_name,
            );
        }
        census.add(
            "value_symbol_links",
            self.value_symbol_links.len(),
            self.value_symbol_links.structural_bytes(),
        );
        census.add(
            "synthetic_expression_links",
            self.synthetic_expression_types.len(),
            self.synthetic_expression_types.allocation_size(),
        );
        // The checker's synthetic AST arena has no byte accounting yet; its node
        // count is reported and the family is named as unavailable.
        census.add(
            "checker_ast",
            usize::try_from(self.factory.node_count()).unwrap_or(0),
            0,
        );
        census.unavailable.push("checker_ast");

        let created = self.types.len();
        let reachable = self.reachable_types(roots);
        census.finish(&TYPE_FAMILIES, created, reachable)
    }

    fn census_structured(
        census: &mut Census,
        family: &'static str,
        data: &crate::StructuredMembers,
    ) {
        if let Some(list) = &data.properties {
            census.list(family, list);
        }
        if let Some(list) = &data.signatures {
            census.list(family, list);
        }
        if let Some(list) = &data.index_infos {
            census.list(family, list);
        }
    }

    fn census_object(census: &mut Census, family: &'static str, data: &crate::ObjectData) {
        Self::census_structured(census, family, &data.structured);
        if let Some(map) = &data.instantiations {
            let keys: usize = map.keys().map(|key| key.len()).sum();
            census.add(
                "type_caches",
                map.len(),
                size_of::<crate::types::Map<crate::CacheKey, TypeId>>()
                    + map.allocation_size()
                    + keys,
            );
        }
    }

    fn census_reference(census: &mut Census, family: &'static str, data: &crate::ReferenceData) {
        Self::census_object(census, family, &data.object);
        if let Some(list) = &data.resolved_type_arguments {
            census.list("type_lists", list);
        }
    }

    fn census_interface(census: &mut Census, family: &'static str, data: &crate::InterfaceData) {
        Self::census_reference(census, family, &data.reference);
        for list in [&data.all_type_parameters, &data.resolved_base_types]
            .into_iter()
            .flatten()
        {
            census.list("type_lists", list);
        }
        for list in [
            &data.declared_call_signatures,
            &data.declared_construct_signatures,
        ]
        .into_iter()
        .flatten()
        {
            census.list(family, list);
        }
        if let Some(list) = &data.declared_index_infos {
            census.list(family, list);
        }
    }

    fn census_union_common(census: &mut Census, data: &crate::UnionOrIntersectionMembers) {
        Self::census_structured(census, "union", &data.structured);
        if let Some(list) = &data.resolved_properties {
            census.list("union", list);
        }
    }

    fn census_caches(&self, census: &mut Census) {
        let caches = &self.types.caches;
        let mut charge = |name: &'static str, entries: usize, allocation: usize, keys: usize| {
            census.add("type_caches", entries, allocation + keys);
            let _ = name;
        };
        // String keys share their backing with the literal's value, charged above.
        charge(
            "stringLiteralTypes",
            caches.string_literal_types.len(),
            caches.string_literal_types.allocation_size(),
            0,
        );
        charge(
            "numberLiteralTypes",
            caches.number_literal_types.len(),
            caches.number_literal_types.allocation_size(),
            0,
        );
        charge(
            "bigintLiteralTypes",
            caches.bigint_literal_types.len(),
            caches.bigint_literal_types.allocation_size(),
            caches
                .bigint_literal_types
                .keys()
                .map(|key| key.base10_value.capacity())
                .sum(),
        );
        charge(
            "unionTypes",
            caches.union_types.len(),
            caches.union_types.allocation_size(),
            caches.union_types.keys().map(|key| key.len()).sum(),
        );
        charge(
            "unionOfUnionTypes",
            caches.union_of_union_types.len(),
            caches.union_of_union_types.allocation_size(),
            caches
                .union_of_union_types
                .keys()
                .map(|key| key.alias.len())
                .sum(),
        );
        charge(
            "tupleTypes",
            caches.tuple_types.len(),
            caches.tuple_types.allocation_size(),
            caches.tuple_types.keys().map(|key| key.len()).sum(),
        );
        charge(
            "intersectionTypes",
            caches.intersection_types.len(),
            caches.intersection_types.allocation_size(),
            caches.intersection_types.keys().map(|key| key.len()).sum(),
        );
        charge(
            "templateLiteralTypes",
            caches.template_literal_types.len(),
            caches.template_literal_types.allocation_size(),
            caches
                .template_literal_types
                .keys()
                .map(|key| key.len())
                .sum(),
        );
    }

    /// Types reachable from the checker's own roots (its named types and
    /// interning caches) and the retained result roots, through payload edges.
    pub(crate) fn reachable_types(&self, roots: &[TypeId]) -> usize {
        let mut visited = vec![false; self.types.len()];
        let mut stack: Vec<TypeId> = Vec::new();
        let push = |id: TypeId, stack: &mut Vec<TypeId>| stack.push(id);
        for name in crate::BUILTIN_TYPE_NAMES {
            if let Some(id) = self.builtins.type_by_name(name) {
                push(id, &mut stack);
            }
        }
        for id in roots {
            push(*id, &mut stack);
        }
        let caches = &self.types.caches;
        for id in caches
            .string_literal_types
            .values()
            .chain(caches.number_literal_types.values())
            .chain(caches.nan_type.iter())
            .chain(caches.bigint_literal_types.values())
            .chain(caches.union_types.values())
            .chain(caches.union_of_union_types.values())
            .chain(caches.tuple_types.values())
            .chain(caches.intersection_types.values())
            .chain(caches.template_literal_types.values())
        {
            push(*id, &mut stack);
        }
        let mut count = 0;
        while let Some(id) = stack.pop() {
            let Ok(record) = self.types.get(id) else {
                continue;
            };
            let Some(index) = id.index(0) else { continue };
            if index >= visited.len() || visited[index] {
                continue;
            }
            visited[index] = true;
            count += 1;
            if let Some(alias) = record.alias {
                if let Ok(alias) = self.types.alias(alias) {
                    stack.extend(alias.type_arguments.iter().copied());
                }
            }
            self.type_edges(id, record.kind, &mut stack);
        }
        count
    }

    fn type_edges(&self, id: TypeId, kind: TypeKind, stack: &mut Vec<TypeId>) {
        let list = |list: &Option<TypeList>, stack: &mut Vec<TypeId>| {
            if let Some(list) = list {
                stack.extend(list.iter().copied());
            }
        };
        let structured = |data: &crate::StructuredMembers, stack: &mut Vec<TypeId>| {
            stack.extend(data.resolved_base_constraint);
            stack.extend(data.object_type_without_abstract_construct_signatures);
            if let Some(properties) = &data.properties {
                for property in properties.iter() {
                    if let Some(links) = self.value_symbol_links.try_get(*property) {
                        stack.extend(links.resolved_type);
                        stack.extend(links.write_type);
                        stack.extend(links.name_type);
                        stack.extend(links.containing_type);
                    }
                }
            }
            if let Some(signatures) = &data.signatures {
                for signature in signatures.iter() {
                    if let Ok(signature) = self.signatures.get(*signature) {
                        stack.extend(signature.resolved_return_type);
                        stack.extend(signature.isolated_signature_type);
                        if let Some(parameters) = &signature.type_parameters {
                            stack.extend(parameters.iter().copied());
                        }
                    }
                }
            }
            if let Some(infos) = &data.index_infos {
                for info in infos.iter() {
                    if let Ok(info) = self.signatures.index_info(*info) {
                        stack.push(info.key_type);
                        stack.push(info.value_type);
                    }
                }
            }
        };
        match kind {
            TypeKind::Literal => {
                if let Ok(data) = self.types.literal(id) {
                    stack.extend(data.fresh);
                    stack.push(data.regular);
                }
            }
            TypeKind::Anonymous | TypeKind::Reference | TypeKind::Interface | TypeKind::Tuple => {
                if let Ok(object) = self.types.object(id) {
                    structured(&object.structured, stack);
                    stack.extend(object.target);
                    if let Some(map) = &object.instantiations {
                        stack.extend(map.values().copied());
                    }
                }
                if let Ok(reference) = self.types.type_reference(id) {
                    list(&reference.resolved_type_arguments, stack);
                }
                if let Ok(interface) = self.types.interface(id) {
                    list(&interface.all_type_parameters, stack);
                    list(&interface.resolved_base_types, stack);
                    stack.extend(interface.this_type);
                    stack.extend(interface.resolved_base_constructor_type);
                    if let Some(members) = interface.declared_members {
                        if let Ok(table) = self.tables.get(members) {
                            for (_, symbol) in table {
                                if let Some(links) =
                                    symbol.and_then(|s| self.value_symbol_links.try_get(s))
                                {
                                    stack.extend(links.resolved_type);
                                }
                            }
                        }
                    }
                }
            }
            TypeKind::Union => {
                if let Ok(data) = self.types.union(id) {
                    structured(&data.common.structured, stack);
                    stack.extend(data.types.iter().copied());
                    stack.extend(data.origin);
                    stack.extend(data.regular_type);
                    stack.extend(data.resolved_reduced_type);
                    if let Some(map) = &data.constituent_map {
                        stack.extend(map.values().copied());
                    }
                }
            }
            TypeKind::Intersection => {
                if let Ok(data) = self.types.intersection(id) {
                    structured(&data.common.structured, stack);
                    stack.extend(data.types.iter().copied());
                    stack.extend(data.resolved_apparent_type);
                    stack.extend(data.unique_literal_filled_instantiation);
                }
            }
            TypeKind::TypeParameter => {
                if let Ok(data) = self.types.type_parameter(id) {
                    stack.extend(data.constraint);
                    stack.extend(data.target);
                    stack.extend(data.resolved_default_type);
                    stack.extend(data.resolved_base_constraint);
                }
            }
            TypeKind::TemplateLiteral => {
                if let Ok(data) = self.types.template_literal(id) {
                    stack.extend(data.types.iter().copied());
                    stack.extend(data.resolved_base_constraint);
                }
            }
            _ => {}
        }
    }
}
