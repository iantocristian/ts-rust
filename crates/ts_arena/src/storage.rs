use crate::{
    counters::Track,
    ids::{allocate_slot, next_arena},
    ArenaId, Counters, Error, NodeId,
};
use std::sync::{Arc, OnceLock};

/// Metadata shared by all AST payloads. Parser/binder construction can update it
/// through FileBuilder; published file and lazy nodes are immutable.
pub struct Node<T> {
    pub kind: u32,
    pub parent: Option<NodeId>,
    pub reparsed: bool,
    pub data: T,
}
impl<T> Node<T> {
    pub fn new(kind: u32, data: T) -> Self {
        Self {
            kind,
            parent: None,
            reparsed: false,
            data,
        }
    }
}

pub(crate) struct Slab<T> {
    pub id: ArenaId,
    pub values: Vec<T>,
    allocation: Option<Track>,
    counters: Counters,
}
impl<T> Slab<T> {
    pub fn new(counters: &Counters) -> Self {
        Self {
            id: next_arena(),
            values: Vec::new(),
            allocation: None,
            counters: counters.clone(),
        }
    }
    pub fn push(&mut self, value: T) -> u32 {
        let slot = allocate_slot(self.values.len());
        if self.values.is_empty() {
            self.allocation = Some(self.counters.allocation());
        }
        self.values.push(value);
        slot
    }
    pub fn get(&self, arena: ArenaId, slot: u32) -> Result<&T, Error> {
        if arena != self.id {
            return Err(Error::WrongOwner);
        }
        self.values.get(slot as usize - 1).ok_or(Error::InvalidSlot)
    }
}

pub(crate) const PAGE_SIZE: usize = 256;
pub(crate) struct Page<T> {
    slots: [OnceLock<T>; PAGE_SIZE],
    _allocation: Track,
}
impl<T> Page<T> {
    fn new(counters: &Counters) -> Self {
        Self {
            slots: std::array::from_fn(|_| OnceLock::new()),
            _allocation: counters.allocation(),
        }
    }
    pub fn get(&self, offset: usize) -> &T {
        self.slots[offset]
            .get()
            .expect("published slot initialized")
    }
}

pub(crate) struct Pages<T> {
    pub id: ArenaId,
    pub len: usize,
    directory: Vec<Arc<Page<T>>>,
    counters: Counters,
}
impl<T> Pages<T> {
    pub fn new(counters: &Counters) -> Self {
        Self {
            id: next_arena(),
            len: 0,
            directory: Vec::new(),
            counters: counters.clone(),
        }
    }
    pub fn reserve(&mut self) -> u32 {
        let slot = allocate_slot(self.len);
        let page = self.len / PAGE_SIZE;
        if page == self.directory.len() {
            self.directory.push(Arc::new(Page::new(&self.counters)));
        }
        self.len += 1;
        slot
    }
    pub fn initialize(&mut self, slot: u32, value: T) {
        let index = slot as usize - 1;
        assert!(index < self.len, "slot must have been reserved");
        // OnceLock initializes only the unused slot, without borrowing the rest
        // of the page mutably while published readers hold shared references.
        assert!(self.directory[index / PAGE_SIZE].slots[index % PAGE_SIZE]
            .set(value)
            .is_ok());
    }
    pub fn push(&mut self, value: T) -> u32 {
        let slot = self.reserve();
        self.initialize(slot, value);
        slot
    }
    pub fn get(&self, slot: u32) -> Result<(Arc<Page<T>>, usize), Error> {
        let index = slot as usize - 1;
        if index >= self.len
            || self.directory[index / PAGE_SIZE].slots[index % PAGE_SIZE]
                .get()
                .is_none()
        {
            return Err(Error::InvalidSlot);
        }
        Ok((self.directory[index / PAGE_SIZE].clone(), index % PAGE_SIZE))
    }
}
