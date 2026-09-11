//! The emit context (`tsc/internal/printer/emitcontext.go`), reduced to the
//! per-node emit flags the type-display path reads.
//!
//! Upstream's context also owns the synthetic node factory, original-node links,
//! string-literal text sources, auto-generated names, emit helpers and variable
//! scopes. Those arrive with the node builder and the transforms; each is a
//! named boundary in the printer until then, never a silent default.
//!
//! Flags are a side table keyed by node identity. A synthetic node keeps its
//! flags for as long as the context lives; the context does not retain nodes.

use crate::EmitFlags;
use std::collections::HashMap;
use std::sync::Arc;
use ts_ast::{node_flags, Factory, FactoryHooks, NodeId};

#[derive(Debug, Default)]
pub struct EmitContext {
    emit_flags: HashMap<NodeId, EmitFlags>,
}

impl EmitContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// The factory hooks upstream's context installs on its node factory. Every
    /// node created through them is marked synthesized, which is how the printer
    /// tells a builder-made node from a parse-tree node. Original-node links and
    /// auto-generated name copies, the other two hooks, arrive with the node
    /// builder.
    pub fn factory_hooks() -> Arc<dyn FactoryHooks> {
        Arc::new(SynthesizedNodeHooks)
    }

    /// `EFNone` for a node no one flagged.
    // port: tsc/internal/printer/emitcontext.go:EmitContext.EmitFlags
    pub fn emit_flags(&self, node: NodeId) -> EmitFlags {
        self.emit_flags.get(&node).copied().unwrap_or(0)
    }

    // port: tsc/internal/printer/emitcontext.go:EmitContext.SetEmitFlags
    pub fn set_emit_flags(&mut self, node: NodeId, flags: EmitFlags) {
        self.emit_flags.insert(node, flags);
    }

    // port: tsc/internal/printer/emitcontext.go:EmitContext.AddEmitFlags
    pub fn add_emit_flags(&mut self, node: NodeId, flags: EmitFlags) {
        *self.emit_flags.entry(node).or_insert(0) |= flags;
    }
}

/// `EmitContext.onCreate`: created nodes carry the synthesized flag.
struct SynthesizedNodeHooks;

impl FactoryHooks for SynthesizedNodeHooks {
    // port: tsc/internal/printer/emitcontext.go:EmitContext.onCreate
    fn on_create(&self, factory: &mut dyn Factory, node: NodeId) {
        factory.add_node_flags(node, node_flags::SYNTHESIZED);
    }
}
