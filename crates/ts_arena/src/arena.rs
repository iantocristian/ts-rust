use crate::{
    counters::Track,
    ids::{allocate_slot, next_arena},
    ArenaId, Counters, Error,
};

/// Exclusive construction storage. Only its owning builder exposes mutation.
pub(crate) struct Arena<T> {
    pub(crate) id: ArenaId,
    values: Vec<T>,
    allocation: Option<Track>,
    counters: Counters,
}

impl<T> Arena<T> {
    pub(crate) fn new(counters: &Counters) -> Self {
        Self {
            id: next_arena(),
            values: Vec::new(),
            allocation: None,
            counters: counters.clone(),
        }
    }

    pub(crate) fn push(&mut self, value: T) -> u32 {
        let slot = allocate_slot(self.values.len());
        if self.values.is_empty() {
            self.allocation = Some(self.counters.allocation());
        }
        self.values.push(value);
        slot
    }

    pub(crate) fn get(&self, arena: ArenaId, slot: u32) -> Result<&T, Error> {
        if arena != self.id {
            return Err(Error::WrongOwner);
        }
        self.get_slot(slot)
    }

    pub(crate) fn get_slot(&self, slot: u32) -> Result<&T, Error> {
        let index = slot.checked_sub(1).ok_or(Error::InvalidSlot)? as usize;
        self.values.get(index).ok_or(Error::InvalidSlot)
    }

    pub(crate) fn values(&self) -> impl Iterator<Item = &T> {
        self.values.iter()
    }

    pub(crate) fn get_mut(&mut self, arena: ArenaId, slot: u32) -> Result<&mut T, Error> {
        if arena != self.id {
            return Err(Error::WrongOwner);
        }
        let index = slot.checked_sub(1).ok_or(Error::InvalidSlot)? as usize;
        self.values.get_mut(index).ok_or(Error::InvalidSlot)
    }
}
