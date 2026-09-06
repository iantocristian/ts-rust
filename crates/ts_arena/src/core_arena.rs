//! The core arena: a file's parsed nodes, immutable after binding and readable
//! without locks (docs/design/ownership.md, section 2.4).

use crate::counters::AllocationGuard;
use crate::ids::{allocate_arena_id, ArenaId, Exhausted, NodeId, Slot, SlotCounter};
use crate::node::NodeData;

/// An append-only arena of parsed nodes. Slots are never reused.
#[derive(Debug)]
pub struct CoreArena {
    id: ArenaId,
    slots: SlotCounter,
    nodes: Vec<NodeData>,
    allocations: AllocationGuard,
}

impl CoreArena {
    /// # Errors
    /// [`Exhausted`] when the process-global arena counter is spent.
    pub fn new() -> Result<Self, Exhausted> {
        Ok(Self::with_ids(allocate_arena_id()?, SlotCounter::new()))
    }

    /// An arena with an injected identity and slot counter, for the E3
    /// exhaustion boundaries.
    pub fn with_ids(id: ArenaId, slots: SlotCounter) -> Self {
        Self {
            id,
            slots,
            nodes: Vec::new(),
            allocations: AllocationGuard::new(),
        }
    }

    pub fn id(&self) -> ArenaId {
        self.id
    }

    /// The number of published slots. Ids of slots beyond this are out of
    /// bounds for every access path.
    pub fn published(&self) -> u32 {
        self.nodes.len() as u32
    }

    /// # Errors
    /// [`Exhausted`] beyond `u32::MAX` slots, before truncation or reuse.
    pub fn allocate(&mut self, node: NodeData) -> Result<NodeId, Exhausted> {
        let slot = self.slots.allocate()?;
        // A slot counter that has been advanced past the arena's contents would
        // publish a node under an id no lookup can index; refuse instead.
        if slot.index() != self.nodes.len() {
            return Err(Exhausted { what: "slot" });
        }
        self.nodes.push(node);
        self.allocations.record(1);
        Ok(NodeId::new(self.id, slot))
    }

    /// Resolve a slot of *this* arena. The caller has already established that
    /// the id belongs here.
    pub fn get(&self, slot: Slot) -> Option<&NodeData> {
        self.nodes.get(slot.index())
    }

    pub fn get_mut(&mut self, slot: Slot) -> Option<&mut NodeData> {
        self.nodes.get_mut(slot.index())
    }

    /// A checked resolution of a raw id against this arena and its published
    /// slot bounds.
    pub fn resolve(&self, id: NodeId) -> Option<&NodeData> {
        (id.arena() == self.id)
            .then(|| self.get(id.slot()))
            .flatten()
    }

    pub fn live_nodes(&self) -> i64 {
        self.allocations.live()
    }
}
