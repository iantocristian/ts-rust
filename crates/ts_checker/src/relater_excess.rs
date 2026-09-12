use crate::{
    object_flags as of,
    relater::{Relater, RelationKind},
    type_flags as tf, CheckerState, Error, TypeId,
};

impl CheckerState {
    // port: tsc/internal/checker/relater.go:isExcessPropertyCheckTarget
    fn excess_check_target(&self, ty: TypeId) -> Result<bool, Error> {
        let record = self.types.get(ty)?;
        if record.flags & tf::OBJECT != 0
            && record.object_flags & of::OBJECT_LITERAL_PATTERN_WITH_COMPUTED_PROPERTIES == 0
            || record.flags & tf::NON_PRIMITIVE != 0
        {
            return Ok(true);
        }
        if record.flags & tf::SUBSTITUTION != 0 {
            return self.excess_check_target(self.types.substitution(ty)?.base);
        }
        if record.flags & tf::UNION_OR_INTERSECTION != 0 {
            let union = record.flags & tf::UNION != 0;
            for &part in self.types.types_of(ty)? {
                if self.excess_check_target(part)? == union {
                    return Ok(union);
                }
            }
            return Ok(!union);
        }
        Ok(false)
    }

    // port: tsc/internal/checker/relater.go:Checker.isKnownProperty
    pub(crate) fn known_property(
        &mut self,
        ty: TypeId,
        name: &[u8],
        jsx: bool,
    ) -> Result<bool, Error> {
        let flags = self.types.flags(ty)?;
        if flags & tf::OBJECT != 0 {
            if self.constituent_property(ty, name, true)?.is_some() {
                return Ok(true);
            }
            let late = name.starts_with(b"\xfe@");
            let key = if late {
                self.builtins.es_symbol_type
            } else {
                self.get_string_literal_type(ts_ast::JsString::from_bytes(name))?
            };
            if self.applicable_index_info(ty, key)?.is_some()
                || late
                    && self
                        .index_info_of_type(ty, self.builtins.string_type)?
                        .is_some()
                || jsx && name.contains(&b'-')
            {
                return Ok(true);
            }
        }
        if flags & tf::SUBSTITUTION != 0 {
            return self.known_property(self.types.substitution(ty)?.base, name, jsx);
        }
        if flags & tf::UNION_OR_INTERSECTION != 0 && self.excess_check_target(ty)? {
            let parts = self.types.compound_types(ty)?.clone();
            for &part in parts.iter() {
                if self.known_property(part, name, jsx)? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}

impl Relater<'_> {
    // port: tsc/internal/checker/relater.go:Relater.isRelatedToEx
    pub(crate) fn no_common_properties(
        &mut self,
        source: TypeId,
        target: TypeId,
    ) -> Result<bool, Error> {
        let s = self.checker.types.flags(source)?;
        let t = self.checker.types.flags(target)?;
        if self.kind == RelationKind::Comparable && s & tf::UNIT == 0
            || s & (tf::PRIMITIVE | tf::OBJECT | tf::INTERSECTION) == 0
            || self.checker.query.global_types.get("Object") == Some(&source)
            || t & (tf::OBJECT | tf::INTERSECTION) == 0
            || !self.checker.is_weak_type(target)?
        {
            return Ok(false);
        }
        let properties = self.checker.get_properties_of_type(source)?;
        if properties.is_empty()
            && self.checker.signatures_of_type(source, false)?.is_empty()
            && self.checker.signatures_of_type(source, true)?.is_empty()
        {
            return Ok(false);
        }
        let jsx = self.checker.types.get(source)?.object_flags & of::JSX_ATTRIBUTES != 0;
        for property in properties {
            let name = self.checker.symbol(property)?.name_to_owned();
            if self.checker.known_property(target, name.as_bytes(), jsx)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    // port: tsc/internal/checker/relater.go:Relater.hasExcessProperties
    pub(crate) fn excess_properties(
        &mut self,
        source: TypeId,
        target: TypeId,
    ) -> Result<bool, Error> {
        let record = *self.checker.types.get(source)?;
        if record.object_flags & (of::OBJECT_LITERAL | of::FRESH_LITERAL)
            != (of::OBJECT_LITERAL | of::FRESH_LITERAL)
            || !self.checker.excess_check_target(target)?
        {
            return Ok(false);
        }
        if self.checker.types.get(target)?.object_flags & of::JS_LITERAL != 0 {
            let options = self.checker.program()?.host.options();
            if !options.strict_option_value(options.no_implicit_any) {
                return Ok(false);
            }
        }
        let jsx = record.object_flags & of::JSX_ATTRIBUTES != 0;
        if matches!(
            self.kind,
            RelationKind::Assignable | RelationKind::Comparable
        ) {
            let global_object = self.checker.query.global_types.get("Object").copied();
            let subset = global_object == Some(target)
                || self.checker.types.flags(target)? & tf::UNION != 0
                    && global_object.is_some_and(|global| {
                        self.checker
                            .types
                            .types_of(target)
                            .is_ok_and(|parts| parts.contains(&global))
                    });
            if subset || !jsx && self.checker.empty_object_type(target)? {
                return Ok(false);
            }
        }
        let mut reduced = target;
        let union = self.checker.types.flags(target)? & tf::UNION != 0;
        if union {
            if let Some(matching) = self.matching_discriminant_type(source, target)? {
                reduced = matching;
            } else {
                let mut non_primitive = false;
                for &part in self.checker.types.types_of(target)? {
                    non_primitive |= self.checker.types.flags(part)?
                        & (tf::INSTANTIABLE_NON_PRIMITIVE | tf::OBJECT | tf::INTERSECTION)
                        != 0;
                }
                if non_primitive {
                    reduced = self.checker.filter_type_flags(target, !tf::PRIMITIVE)?;
                }
            }
        }
        let container = record
            .symbol
            .ok_or(Error::MissingLink("fresh literal symbol"))?;
        let container_declaration = self.checker.symbol(container)?.value_declaration();
        for property in self.checker.get_properties_of_type(source)? {
            let read = self.checker.symbol(property)?;
            let name = read.name_to_owned();
            let Some(declaration) = read.value_declaration() else {
                continue;
            };
            if container_declaration.is_none()
                || self.checker.ast(declaration)?.node(declaration)?.parent()
                    != container_declaration
                || jsx && name.as_bytes().contains(&b'-')
            {
                continue;
            }
            if !self.checker.known_property(reduced, name.as_bytes(), jsx)? {
                return Ok(true);
            }
            if union {
                let parts = if self.checker.types.flags(reduced)? & tf::UNION != 0 {
                    self.checker.types.compound_types(reduced)?.clone()
                } else {
                    vec![reduced].into()
                };
                let mut types = Vec::new();
                for &part in parts.iter() {
                    if let Some(property) =
                        self.checker
                            .constituent_property(part, name.as_bytes(), true)?
                    {
                        types.push(self.checker.get_type_of_symbol(property)?);
                    } else {
                        let key = self.checker.get_string_literal_type(name.clone())?;
                        types.push(
                            if let Some(info) = self.checker.applicable_index_info(part, key)? {
                                self.checker.signatures.index_info(info)?.value_type
                            } else {
                                self.checker.builtins.undefined_type
                            },
                        );
                    }
                }
                let target = self.checker.get_union_type(&types)?;
                let source = self.checker.get_type_of_symbol(property)?;
                if self.related(source, target, crate::relater::BOTH, 0)? == crate::ternary::FALSE {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}
