//! Property discovery over source object types, unions and intersections. All synthesized
//! symbols, declaration lists and cache entries belong to the checker.

use crate::{object_flags as of, type_flags as tf, CheckerState, Error, TypeId};
use std::collections::HashSet;
use ts_arena::SymbolId;
use ts_ast::{
    check_flags as cf, modifier_flags as mf, symbol_flags as sf, JsString, SymbolTable,
    SyntaxKind as K,
};

impl CheckerState {
    // port: tsc/internal/checker/checker.go:Checker.getPropertiesOfType
    pub(crate) fn get_properties_of_type(&mut self, ty: TypeId) -> Result<Vec<SymbolId>, Error> {
        // Go uses getReducedApparentType here. This slice resolves primitive
        // wrappers per constituent instead of constructing an apparent
        // intersection. That is sufficient only while interface `this`, type
        // parameters and heritage are rejected by get_declared_type_of_interface.
        // Port the apparent intersection and this-argument substitution before
        // relaxing those guards; constituent lookup alone cannot substitute `this`.
        let ty = self.get_reduced_type(ty)?;
        let flags = self.types.flags(ty)?;
        if flags & tf::UNION_OR_INTERSECTION != 0 && flags & tf::BOOLEAN == 0 {
            return self.get_properties_of_union_or_intersection_type(ty);
        }
        if flags & tf::OBJECT == 0 {
            return self.properties_of_primitive_type(ty);
        }
        self.resolve_type_members(ty)?;
        Ok(self
            .types
            .structured(ty)?
            .properties
            .as_deref()
            .unwrap_or_default()
            .to_vec())
    }

    // port: tsc/internal/checker/checker.go:Checker.getPropertiesOfUnionOrIntersectionType
    pub(crate) fn get_properties_of_union_or_intersection_type(
        &mut self,
        ty: TypeId,
    ) -> Result<Vec<SymbolId>, Error> {
        if let Some(properties) = &self.types.compound_members(ty)?.resolved_properties {
            return Ok(properties.to_vec());
        }
        let is_union = self.types.flags(ty)? & tf::UNION != 0;
        let types = self.types.compound_types(ty)?.clone();
        let mut checked = HashSet::new();
        let mut properties = Vec::new();
        for &current in types.iter() {
            self.require_plain_members(current)?;
            for prop in self.get_properties_of_type(current)? {
                let name = self.symbol(prop)?.name_to_owned();
                if checked.insert(name.clone()) {
                    if let Some(property) = self.get_compound_property(ty, name, !is_union)? {
                        properties.push(property);
                    }
                }
            }
            // The supported constituents have no index signatures, so unions
            // enumerate only the first; intersections enumerate all of them.
            if is_union {
                break;
            }
        }
        self.types.compound_members_mut(ty)?.resolved_properties = Some(properties.clone().into());
        Ok(properties)
    }

    fn require_plain_members(&mut self, ty: TypeId) -> Result<(), Error> {
        let flags = self.types.flags(ty)?;
        if flags & tf::UNION_OR_INTERSECTION != 0 && flags & tf::BOOLEAN == 0 {
            let types = self.types.compound_types(ty)?.clone();
            for &ty in types.iter() {
                self.require_plain_members(ty)?;
            }
        } else if flags & tf::OBJECT != 0 {
            self.resolve_type_members(ty)?;
            let members = self.types.structured(ty)?;
            if members.signatures.as_ref().is_some_and(|s| !s.is_empty())
                || members.index_infos.as_ref().is_some_and(|s| !s.is_empty())
            {
                return Err(Error::Unsupported("union property: signatures/index infos"));
            }
        } else if let Some(apparent) = self.apparent_primitive_type(ty)? {
            self.require_plain_members(apparent)?;
        }
        Ok(())
    }

    // The reached data-property branch of getPropertyOfTypeEx. Function
    // augmentation is rejected by require_plain_members, not silently ignored.
    fn constituent_property(
        &mut self,
        ty: TypeId,
        name: &[u8],
        skip_augment: bool,
    ) -> Result<Option<SymbolId>, Error> {
        let ty = self.get_reduced_type(ty)?;
        self.require_plain_members(ty)?;
        let flags = self.types.flags(ty)?;
        if flags & tf::INTERSECTION != 0 {
            if let Some(prop) = self.get_compound_property(ty, JsString::from_bytes(name), true)? {
                return Ok(Some(prop));
            }
            if !skip_augment {
                return self.get_compound_property(ty, JsString::from_bytes(name), false);
            }
            return Ok(None);
        }
        if flags & tf::UNION != 0 && flags & tf::BOOLEAN == 0 {
            return self.get_compound_property(ty, JsString::from_bytes(name), skip_augment);
        }
        let object = if flags & tf::OBJECT != 0 {
            Some(ty)
        } else {
            self.apparent_primitive_type(ty)?
        };
        if let Some(ty) = object {
            if let Some(prop) = self.object_property(ty, name)? {
                return Ok(Some(prop));
            }
            if skip_augment {
                return Ok(None);
            }
            if let Some(&object) = self.query.global_types.get("Object") {
                if object != ty {
                    return self.object_property(object, name);
                }
            }
        }
        Ok(None)
    }

    // port: tsc/internal/checker/checker.go:Checker.getPropertyOfObjectType
    fn object_property(&mut self, ty: TypeId, name: &[u8]) -> Result<Option<SymbolId>, Error> {
        self.resolve_type_members(ty)?;
        let Some(table) = self.types.structured(ty)?.members else {
            return Ok(None);
        };
        let Some(prop) = self.table(table)?.get(name).flatten() else {
            return Ok(None);
        };
        Ok((self.symbol(prop)?.flags() & sf::VALUE != 0).then_some(prop))
    }

    // port: tsc/internal/checker/checker.go:Checker.getPropertyOfUnionOrIntersectionType
    // port: tsc/internal/checker/checker.go:Checker.getUnionOrIntersectionProperty
    fn get_compound_property(
        &mut self,
        ty: TypeId,
        name: JsString,
        skip_augment: bool,
    ) -> Result<Option<SymbolId>, Error> {
        let common = self.types.compound_members(ty)?;
        let cache = if skip_augment {
            common.property_cache_without_function_property_augment
        } else {
            common.property_cache
        };
        if let Some(cache) = cache {
            if let Some(prop) = self.table(cache)?.get(name.as_bytes()).flatten() {
                return Ok(
                    (self.symbol(prop)?.check_flags() & cf::READ_PARTIAL == 0).then_some(prop)
                );
            }
        }
        let Some(prop) = self.create_compound_property(ty, &name, skip_augment)? else {
            return Ok(None);
        };
        let cache = self.compound_property_cache(ty, skip_augment)?;
        self.tables.get_mut(cache)?.insert(name.clone(), Some(prop));
        let flags = self.symbol(prop)?.check_flags();
        if skip_augment && flags & cf::PARTIAL == 0 {
            let augmented = self.compound_property_cache(ty, false)?;
            if self
                .table(augmented)?
                .get(name.as_bytes())
                .flatten()
                .is_none()
            {
                self.tables.get_mut(augmented)?.insert(name, Some(prop));
            }
        }
        Ok((flags & cf::READ_PARTIAL == 0).then_some(prop))
    }

    fn compound_property_cache(
        &mut self,
        ty: TypeId,
        skip_augment: bool,
    ) -> Result<ts_ast::SymbolTableId, Error> {
        let common = self.types.compound_members(ty)?;
        let cache = if skip_augment {
            common.property_cache_without_function_property_augment
        } else {
            common.property_cache
        };
        if let Some(cache) = cache {
            return Ok(cache);
        }
        let cache = self.alloc_symbol_table(SymbolTable::new());
        let common = self.types.compound_members_mut(ty)?;
        if skip_augment {
            common.property_cache_without_function_property_augment = Some(cache);
        } else {
            common.property_cache = Some(cache);
        }
        Ok(cache)
    }

    // port: tsc/internal/checker/checker.go:Checker.createUnionOrIntersectionProperty
    fn create_compound_property(
        &mut self,
        containing: TypeId,
        name: &JsString,
        skip_augment: bool,
    ) -> Result<Option<SymbolId>, Error> {
        let types = self.types.compound_types(containing)?.clone();
        let is_union = self.types.flags(containing)? & tf::UNION != 0;
        let mut props = Vec::new();
        let mut optional = if is_union { 0 } else { sf::OPTIONAL };
        let mut flags = if is_union { 0 } else { cf::READONLY };
        for &current in types.iter() {
            if self.is_error_type(current)? || self.types.flags(current)? & tf::NEVER != 0 {
                continue;
            }
            if let Some(prop) = self.constituent_property(current, name.as_bytes(), skip_augment)? {
                let read = self.symbol(prop)?;
                if read.flags() & sf::PROPERTY == 0 || read.flags() & sf::ACCESSOR != 0 {
                    return Err(Error::Unsupported("union property: method/accessor"));
                }
                if read.check_flags() & (cf::INSTANTIATED | cf::MAPPED | cf::REVERSE_MAPPED) != 0 {
                    return Err(Error::Unsupported(
                        "union property: instantiated/mapped symbol",
                    ));
                }
                if is_union {
                    optional |= read.flags() & sf::OPTIONAL;
                } else {
                    optional &= read.flags();
                }
                let mut readonly = read.check_flags() & cf::READONLY != 0;
                if read.check_flags() & cf::SYNTHETIC == 0 {
                    if let Some(declaration) = read.value_declaration() {
                        let view = self.ast(declaration)?;
                        let node = view.node(declaration)?;
                        if node.kind() != K::PropertySignature
                            && node.kind() != K::PropertyAssignment
                        {
                            return Err(Error::Unsupported("union property: non-data declaration"));
                        }
                        let modifiers = node.modifier_flags(view)?;
                        if modifiers & !mf::READONLY != 0 {
                            return Err(Error::Unsupported(
                                "union property: visibility/static modifiers",
                            ));
                        }
                        readonly |= modifiers & mf::READONLY != 0;
                    }
                }
                if is_union && readonly {
                    flags |= cf::READONLY;
                } else if !is_union && !readonly {
                    flags &= !cf::READONLY;
                }
                flags |= cf::CONTAINS_PUBLIC | cf::CONTAINS_WRITE_PUBLIC;
                if !props.contains(&prop) {
                    props.push(prop);
                }
            } else if is_union && self.types.get(current)?.object_flags & of::OBJECT_LITERAL != 0 {
                return Err(Error::Unsupported(
                    "union property: object literal write-partial type",
                ));
            } else if is_union {
                flags |= cf::READ_PARTIAL;
            }
        }
        let Some(&first) = props.first() else {
            return Ok(None);
        };
        if props.len() == 1 && flags & cf::READ_PARTIAL == 0 {
            return Ok(Some(first));
        }
        if optional != 0 && self.options.exact_optional_property_types {
            return Err(Error::Unsupported(
                "union property: exact optional write type",
            ));
        }
        let mut declarations = Vec::new();
        let mut prop_types = Vec::with_capacity(props.len());
        let mut first_value = None;
        let mut nonuniform_value = false;
        let mut name_type = None;
        for &prop in &props {
            let value = self.symbol(prop)?.value_declaration();
            if first_value.is_none() {
                first_value = value;
            } else if value.is_some() && value != first_value {
                nonuniform_value = true;
            }
            for declaration in self.symbol_declarations(prop)?.iter() {
                if !declarations.contains(&declaration) {
                    declarations.push(declaration);
                }
            }
            let ty = self.get_type_of_symbol(prop)?;
            if let Some(&first_type) = prop_types.first() {
                if ty != first_type {
                    flags |= cf::HAS_NON_UNIFORM_TYPE;
                }
            } else {
                name_type = self
                    .value_symbol_links
                    .try_get(prop)
                    .and_then(|l| l.name_type);
            }
            if self.is_literal_type(ty)? || self.is_pattern_literal_type(ty)? {
                flags |= cf::HAS_LITERAL_TYPE;
            }
            if self.types.flags(ty)? & tf::NEVER != 0 && ty != self.builtins.unique_literal_type {
                flags |= cf::HAS_NEVER_TYPE;
            }
            prop_types.push(ty);
        }
        let declaration_list = self.declarations.alloc(declarations)?;
        let result = self.new_symbol_ex(
            sf::PROPERTY | optional,
            name.clone(),
            flags | cf::SYNTHETIC_PROPERTY,
        )?;
        self.symbol_mut(result)?.declarations = declaration_list;
        if !nonuniform_value {
            if let Some(value) = first_value {
                // Go inherits the raw declaration symbol's parent, before
                // checker-local merges or late-bound resolution.
                let source_symbol = self
                    .program()?
                    .bound(value)?
                    .node_binding(value)?
                    .and_then(|binding| binding.symbol)
                    .ok_or(Error::MissingLink("property declaration symbol"))?;
                let parent = self.symbol(source_symbol)?.parent();
                let result = self.symbol_mut(result)?;
                result.value_declaration = Some(value);
                result.parent = parent;
            }
        }
        let links = self.value_symbol_links.get_or_default(result);
        links.containing_type = Some(containing);
        links.name_type = name_type;
        if prop_types.len() > 2 {
            self.symbol_mut(result)?.check_flags |= cf::DEFERRED_TYPE;
            *self.query.deferred_property_types.get_or_default(result) = Some(prop_types.into());
        } else {
            let ty = if is_union {
                self.get_union_type(&prop_types)?
            } else {
                self.get_intersection_type(&prop_types)?
            };
            self.value_symbol_links.get_or_default(result).resolved_type = Some(ty);
        }
        Ok(Some(result))
    }

    // port: tsc/internal/checker/checker.go:isLiteralType
    fn is_literal_type(&self, ty: TypeId) -> Result<bool, Error> {
        let flags = self.types.flags(ty)?;
        if flags & tf::BOOLEAN != 0 {
            return Ok(true);
        }
        if flags & tf::UNION != 0 {
            if flags & tf::ENUM_LITERAL != 0 {
                return Ok(true);
            }
            for &ty in self.types.union(ty)?.types.iter() {
                if self.types.flags(ty)? & tf::UNIT == 0 {
                    return Ok(false);
                }
            }
            return Ok(true);
        }
        Ok(flags & tf::UNIT != 0)
    }

    // port: tsc/internal/checker/checker.go:Checker.getTypeOfSymbolWithDeferredType
    pub(crate) fn get_type_of_symbol_with_deferred_type(
        &mut self,
        symbol: SymbolId,
    ) -> Result<TypeId, Error> {
        if let Some(ty) = self
            .value_symbol_links
            .try_get(symbol)
            .and_then(|l| l.resolved_type)
        {
            return Ok(ty);
        }
        let types = self
            .query
            .deferred_property_types
            .try_get(symbol)
            .and_then(Option::as_ref)
            .ok_or(Error::Unsupported(
                "deferred property: non-union constituents",
            ))?
            .clone();
        let containing = self
            .value_symbol_links
            .try_get(symbol)
            .and_then(|links| links.containing_type)
            .ok_or(Error::MissingLink("deferred property containing type"))?;
        let ty = if self.types.flags(containing)? & tf::UNION != 0 {
            self.get_union_type(&types)?
        } else {
            self.get_intersection_type(&types)?
        };
        self.value_symbol_links.get_or_default(symbol).resolved_type = Some(ty);
        Ok(ty)
    }
}
