//! The lazy-resolution cycle guard (`Checker.pushTypeResolution` and friends in
//! `tsc/internal/checker/checker.go`), kept as ADR 0008 requires.
//!
//! The guard keys on the entity *and* the property being resolved. On a cycle it
//! marks every resolution from the cycle start to the top as failed and pushes
//! nothing; the search stops early at any intermediate resolution whose property
//! has already been produced. A set of busy entities is not equivalent (plan
//! §4.2). Whether a property has been produced depends on the checker's links,
//! so the caller supplies that predicate; this module owns only the stack.

use crate::{SignatureId, TypeId};
use ts_arena::{NodeId, SymbolId};

/// `TypeSystemEntity` is `any` upstream; these are the four kinds it holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TypeSystemEntity {
    Symbol(SymbolId),
    #[cfg_attr(
        not(test),
        allow(
            dead_code,
            reason = "P3 resolution entity; P1 cycle tests retain the complete entity schema"
        )
    )]
    Type(TypeId),
    #[allow(
        dead_code,
        reason = "P3 signature resolution entity retained by the P1 cycle-guard schema"
    )]
    Signature(SignatureId),
    #[allow(
        dead_code,
        reason = "P3/P4 node resolution entity retained by the P1 cycle-guard schema"
    )]
    Node(NodeId),
}

/// `TypeSystemPropertyName`, in upstream declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum TypeSystemPropertyName {
    Type = 0,
    ResolvedBaseConstructorType,
    DeclaredType,
    ResolvedReturnType,
    ResolvedBaseConstraint,
    ResolvedTypeArguments,
    ResolvedBaseTypes,
    WriteType,
    InitializerIsUndefined,
    AliasTarget,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypeResolution {
    pub target: TypeSystemEntity,
    pub property_name: TypeSystemPropertyName,
    pub result: bool,
}

#[derive(Debug, Default)]
pub struct ResolutionStack {
    resolutions: Vec<TypeResolution>,
    resolution_start: usize,
}

impl ResolutionStack {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    pub fn depth(&self) -> usize {
        self.resolutions.len()
    }

    /// Sets the search floor and returns the previous one for restoration.
    #[cfg(test)]
    pub fn set_resolution_start(&mut self, start: usize) -> usize {
        std::mem::replace(&mut self.resolution_start, start)
    }

    /// Returns `true` and records the resolution when no cycle exists. On a cycle
    /// every resolution from the cycle start upward is marked failed and nothing
    /// is pushed; the caller must not `pop` for a `false` result.
    // port: tsc/internal/checker/checker.go:Checker.pushTypeResolution
    pub fn push(
        &mut self,
        target: TypeSystemEntity,
        property_name: TypeSystemPropertyName,
        has_property: impl FnMut(&TypeResolution) -> bool,
    ) -> bool {
        if let Some(start) =
            self.find_resolution_cycle_start_index(target, property_name, has_property)
        {
            for resolution in &mut self.resolutions[start..] {
                resolution.result = false;
            }
            return false;
        }
        self.resolutions.push(TypeResolution {
            target,
            property_name,
            result: true,
        });
        true
    }

    /// Searches from the top down to the resolution floor. Upstream returns -1 for
    /// no cycle; that is `None` here.
    // port: tsc/internal/checker/checker.go:Checker.findResolutionCycleStartIndex
    pub fn find_resolution_cycle_start_index(
        &self,
        target: TypeSystemEntity,
        property_name: TypeSystemPropertyName,
        mut has_property: impl FnMut(&TypeResolution) -> bool,
    ) -> Option<usize> {
        for index in (self.resolution_start..self.resolutions.len()).rev() {
            let resolution = &self.resolutions[index];
            if has_property(resolution) {
                return None;
            }
            if resolution.target == target && resolution.property_name == property_name {
                return Some(index);
            }
        }
        None
    }

    /// Pops the last resolution and reports whether it completed without a cycle.
    // port: tsc/internal/checker/checker.go:Checker.popTypeResolution
    pub fn pop(&mut self) -> bool {
        self.resolutions
            .pop()
            .expect("popTypeResolution pairs with a successful pushTypeResolution")
            .result
    }
}
