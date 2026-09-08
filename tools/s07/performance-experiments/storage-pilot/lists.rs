//! Owner-local edges built through one reusable, nested LIFO scratch buffer.
//!
//! This pilot validates the supplied node domain; it does not own AST nodes or
//! implement foreign imports. A production integration must retain that complete
//! owner and preserve its checked/fallback API. No per-list scratch Vec or boxed
//! edge backing is allocated. Published slices may cross physical edge pages.

use crate::pages::Pages;
use std::collections::BTreeMap;
use std::ops::Range;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    WrongOwner,
    InvalidNode,
    InvalidSlice,
    WrongFrame,
    Incomplete,
    Exhausted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeRef {
    pub owner: u64,
    pub slot: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LocalSlice {
    backing: u32,
    start: u32,
    len: u32,
}

/// A copied slice header retains backing identity; it does not own storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slice {
    owner: u64,
    local: LocalSlice,
}
impl Slice {
    pub fn is_nil(self) -> bool {
        self.local.backing == 0 && self.local.start == 0
    }
    pub fn is_missing(self) -> bool {
        self.local.backing == 0 && self.local.start == 1
    }
    pub fn len(self) -> usize {
        self.local.len as usize
    }
    pub fn is_empty(self) -> bool {
        self.local.len == 0
    }
    pub fn same(self, other: Self) -> bool {
        self.local.len == other.local.len
            && (self.is_empty()
                || (self.owner == other.owner
                    && self.local.backing == other.local.backing
                    && self.local.start == other.local.start))
    }
    pub fn slice(self, range: Range<usize>) -> Result<Self, Error> {
        if range.start > range.end || range.end > self.len() {
            return Err(Error::InvalidSlice);
        }
        let start = self
            .local
            .start
            .checked_add(u32::try_from(range.start).map_err(|_| Error::Exhausted)?)
            .ok_or(Error::Exhausted)?;
        Ok(Self {
            local: LocalSlice {
                start,
                len: u32::try_from(range.len()).map_err(|_| Error::Exhausted)?,
                ..self.local
            },
            ..self
        })
    }
}

#[derive(Default)]
struct Backing {
    start: u32,
    len: u32,
}

fn compact_start(start: usize) -> (u32, Option<usize>) {
    match u32::try_from(start) {
        Ok(word) if word != u32::MAX => (word, None),
        _ => (u32::MAX, Some(start)),
    }
}

/// Layout of a owner-local NodeList/ModifierList header, separate from backing.
#[derive(Clone, Copy, Debug)]
struct Header {
    pos: i32,
    end: i32,
    nodes: LocalSlice,
    modifiers: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ListId {
    owner: u64,
    index: u32,
}

/// A successful finish/abort invalidates this serial; errors preserve the token.
#[derive(Debug)]
pub struct Frame {
    owner: u64,
    serial: u64,
    depth: usize,
    start: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub edge_words: usize,
    pub edge_capacity: usize,
    pub edge_pages: usize,
    pub edge_directory_capacity: usize,
    pub backings: usize,
    pub backing_capacity: usize,
    pub list_headers: usize,
    pub list_header_capacity: usize,
    pub scratch_capacity: usize,
    pub scratch_peak_len: usize,
    pub frame_capacity: usize,
    pub wide_backings: usize,
}

pub struct Builder<const PAGE: usize> {
    owner: u64,
    node_owner: u64,
    max_node: u32,
    edges: Pages<u32, PAGE>,
    backings: Vec<Backing>,
    wide_starts: BTreeMap<u32, usize>,
    headers: Vec<Header>,
    scratch: Vec<u32>,
    frames: Vec<u64>,
    next_serial: u64,
    scratch_peak_len: usize,
}

pub struct Published<const PAGE: usize> {
    store: Builder<PAGE>,
}

impl<const PAGE: usize> Builder<PAGE> {
    pub fn new(node_owner: u64, max_node: u32) -> Self {
        assert_ne!(node_owner, 0, "pilot node domain must be nonzero");
        let owner = NEXT_OWNER
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .expect("pilot construction identities exhausted before reuse");
        Self {
            owner,
            node_owner,
            max_node,
            edges: Pages::default(),
            backings: Vec::new(),
            wide_starts: BTreeMap::new(),
            headers: Vec::new(),
            scratch: Vec::new(),
            frames: Vec::new(),
            next_serial: 0,
            scratch_peak_len: 0,
        }
    }
    pub fn nil(&self) -> Slice {
        self.wrap(LocalSlice {
            backing: 0,
            start: 0,
            len: 0,
        })
    }
    pub fn missing(&self) -> Slice {
        self.wrap(LocalSlice {
            backing: 0,
            start: 1,
            len: 0,
        })
    }
    fn wrap(&self, local: LocalSlice) -> Slice {
        Slice {
            owner: if local.backing == 0 { 0 } else { self.owner },
            local,
        }
    }
    fn check_frame(&self, frame: &Frame) -> Result<(), Error> {
        if frame.owner != self.owner {
            return Err(Error::WrongOwner);
        }
        if self.frames.len() != frame.depth || self.frames.last() != Some(&frame.serial) {
            return Err(Error::WrongFrame);
        }
        Ok(())
    }
    pub fn begin(&mut self) -> Result<Frame, Error> {
        self.next_serial = self.next_serial.checked_add(1).ok_or(Error::Exhausted)?;
        self.frames.push(self.next_serial);
        Ok(Frame {
            owner: self.owner,
            serial: self.next_serial,
            depth: self.frames.len(),
            start: self.scratch.len(),
        })
    }
    pub fn push(&mut self, frame: &Frame, node: Option<NodeRef>) -> Result<(), Error> {
        self.check_frame(frame)?;
        let slot = self.check_node(node)?;
        if self.scratch.len() - frame.start >= u32::MAX as usize {
            return Err(Error::Exhausted);
        }
        self.scratch.push(slot);
        self.scratch_peak_len = self.scratch_peak_len.max(self.scratch.len());
        Ok(())
    }
    fn check_node(&self, node: Option<NodeRef>) -> Result<u32, Error> {
        match node {
            None => Ok(0),
            Some(node) if node.owner != self.node_owner => Err(Error::WrongOwner),
            Some(node) if node.slot == 0 || node.slot > self.max_node => Err(Error::InvalidNode),
            Some(node) => Ok(node.slot),
        }
    }
    pub fn finish(&mut self, frame: &Frame) -> Result<Slice, Error> {
        self.check_frame(frame)?;
        let backing = u32::try_from(self.backings.len())
            .ok()
            .and_then(|n| n.checked_add(1))
            .ok_or(Error::Exhausted)?;
        let len = u32::try_from(self.scratch.len() - frame.start).map_err(|_| Error::Exhausted)?;
        let start = self.edges.len();
        for word in &self.scratch[frame.start..] {
            self.edges.push(*word);
        }
        let (start, wide) = compact_start(start);
        if let Some(start) = wide {
            self.wide_starts.insert(backing, start);
        }
        self.backings.push(Backing { start, len });
        self.scratch.truncate(frame.start);
        self.frames.pop();
        Ok(self.wrap(LocalSlice {
            backing,
            start: 0,
            len,
        }))
    }
    pub fn abort(&mut self, frame: &Frame) -> Result<(), Error> {
        self.check_frame(frame)?;
        self.scratch.truncate(frame.start);
        self.frames.pop();
        Ok(())
    }
    fn range(&self, slice: Slice) -> Result<Range<usize>, Error> {
        if slice.local.backing == 0 {
            if slice.owner != 0 || slice.local.len != 0 || slice.local.start > 1 {
                return Err(Error::InvalidSlice);
            }
            return Ok(0..0);
        }
        if slice.owner != self.owner {
            return Err(Error::WrongOwner);
        }
        let backing = self
            .backings
            .get(slice.local.backing as usize - 1)
            .ok_or(Error::InvalidSlice)?;
        let end = slice
            .local
            .start
            .checked_add(slice.local.len)
            .ok_or(Error::InvalidSlice)?;
        if end > backing.len {
            return Err(Error::InvalidSlice);
        }
        let base = if backing.start == u32::MAX {
            *self
                .wide_starts
                .get(&slice.local.backing)
                .ok_or(Error::InvalidSlice)?
        } else {
            backing.start as usize
        };
        let start = base
            .checked_add(slice.local.start as usize)
            .ok_or(Error::InvalidSlice)?;
        let end = start
            .checked_add(slice.local.len as usize)
            .ok_or(Error::InvalidSlice)?;
        if end > self.edges.len() {
            return Err(Error::InvalidSlice);
        }
        Ok(start..end)
    }
    pub fn get(&self, slice: Slice, index: usize) -> Result<Option<NodeRef>, Error> {
        let range = self.range(slice)?;
        if index >= range.len() {
            return Err(Error::InvalidSlice);
        }
        let slot = *self
            .edges
            .get(range.start + index)
            .ok_or(Error::InvalidSlice)?;
        Ok((slot != 0).then_some(NodeRef {
            owner: self.node_owner,
            slot,
        }))
    }
    /// Shared slices observe mutation through their common backing.
    pub fn set(&mut self, slice: Slice, index: usize, value: Option<NodeRef>) -> Result<(), Error> {
        let range = self.range(slice)?;
        if index >= range.len() {
            return Err(Error::InvalidSlice);
        }
        let slot = self.check_node(value)?;
        *self
            .edges
            .get_mut(range.start + index)
            .ok_or(Error::InvalidSlice)? = slot;
        Ok(())
    }
    pub fn new_list(&mut self, pos: i32, end: i32, nodes: Slice) -> Result<ListId, Error> {
        self.range(nodes)?;
        let index = u32::try_from(self.headers.len()).map_err(|_| Error::Exhausted)?;
        self.headers.push(Header {
            pos,
            end,
            nodes: nodes.local,
            modifiers: 0,
        });
        Ok(ListId {
            owner: self.owner,
            index,
        })
    }
    fn header(&self, id: ListId) -> Result<&Header, Error> {
        if id.owner != self.owner {
            return Err(Error::WrongOwner);
        }
        self.headers
            .get(id.index as usize)
            .ok_or(Error::InvalidSlice)
    }
    pub fn clone_list(&mut self, id: ListId) -> Result<ListId, Error> {
        let header = *self.header(id)?;
        let index = u32::try_from(self.headers.len()).map_err(|_| Error::Exhausted)?;
        self.headers.push(header);
        Ok(ListId {
            owner: self.owner,
            index,
        })
    }
    pub fn set_modifiers(&mut self, id: ListId, flags: u32) -> Result<(), Error> {
        self.header(id)?;
        self.headers[id.index as usize].modifiers = flags;
        Ok(())
    }
    pub fn list(&self, id: ListId) -> Result<(i32, i32, Slice, u32), Error> {
        let header = self.header(id)?;
        Ok((
            header.pos,
            header.end,
            self.wrap(header.nodes),
            header.modifiers,
        ))
    }
    pub fn stats(&self) -> Stats {
        Stats {
            edge_words: self.edges.len(),
            edge_capacity: self.edges.capacity(),
            edge_pages: self.edges.page_count(),
            edge_directory_capacity: self.edges.directory_capacity(),
            backings: self.backings.len(),
            backing_capacity: self.backings.capacity(),
            list_headers: self.headers.len(),
            list_header_capacity: self.headers.capacity(),
            scratch_capacity: self.scratch.capacity(),
            scratch_peak_len: self.scratch_peak_len,
            frame_capacity: self.frames.capacity(),
            wide_backings: self.wide_starts.len(),
        }
    }
    pub fn publish(self) -> Result<Published<PAGE>, Error> {
        if !self.frames.is_empty() || !self.scratch.is_empty() {
            return Err(Error::Incomplete);
        }
        // Scratch remains retained and charged at this endpoint. Moving it to a
        // reusable worker or dropping it at publication is a separate experiment.
        Ok(Published { store: self })
    }
}

impl<const PAGE: usize> Published<PAGE> {
    pub fn get(&self, slice: Slice, index: usize) -> Result<Option<NodeRef>, Error> {
        self.store.get(slice, index)
    }
    pub fn list(&self, id: ListId) -> Result<(i32, i32, Slice, u32), Error> {
        self.store.list(id)
    }
    pub fn stats(&self) -> Stats {
        self.store.stats()
    }
    /// Diagnostic sweep of every physical backing, including empty/unused ones.
    pub fn for_each_edge(&self, mut visit: impl FnMut(u32)) {
        for (index, backing) in self.store.backings.iter().enumerate() {
            let slice = self.store.wrap(LocalSlice {
                backing: u32::try_from(index + 1).expect("allocated backing id"),
                start: 0,
                len: backing.len,
            });
            self.store
                .edges
                .visit_range(
                    self.store.range(slice).expect("published backing bounds"),
                    |value| visit(*value),
                )
                .expect("published edge bounds");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn node(slot: u32) -> Option<NodeRef> {
        Some(NodeRef { owner: 1, slot })
    }
    #[test]
    fn nested_lists_reuse_scratch_without_interleaving_backings() {
        let mut b = Builder::<2>::new(1, 5);
        let outer = b.begin().unwrap();
        b.push(&outer, node(1)).unwrap();
        let inner = b.begin().unwrap();
        b.push(&inner, node(2)).unwrap();
        b.push(&inner, None).unwrap();
        assert_eq!(b.push(&outer, node(3)), Err(Error::WrongFrame));
        assert_eq!(b.finish(&outer), Err(Error::WrongFrame));
        let inside = b.finish(&inner).unwrap();
        b.push(&outer, node(3)).unwrap();
        assert_eq!(b.finish(&inner), Err(Error::WrongFrame));
        let outside = b.finish(&outer).unwrap();
        assert_eq!(
            (b.get(inside, 0), b.get(inside, 1)),
            (Ok(node(2)), Ok(None))
        );
        assert_eq!(
            (b.get(outside, 0), b.get(outside, 1)),
            (Ok(node(1)), Ok(node(3)))
        );
        assert_eq!(b.stats().scratch_peak_len, 3);
        assert_eq!(b.stats().edge_words, 4);
        assert!(b.publish().is_ok());
    }
    #[test]
    fn empty_missing_shared_slices_and_header_identity_survive_publication() {
        let mut b = Builder::<2>::new(1, 5);
        let frame = b.begin().unwrap();
        let empty = b.finish(&frame).unwrap();
        assert!(!empty.is_nil());
        assert!(b.nil().is_nil());
        assert!(b.missing().is_missing());
        assert!(empty.same(b.nil()) && empty.same(b.missing()));
        let frame = b.begin().unwrap();
        for n in [1, 2, 3] {
            b.push(&frame, node(n)).unwrap();
        }
        let full = b.finish(&frame).unwrap();
        let tail = full.slice(1..3).unwrap();
        let header = b.new_list(-1, i32::MAX, full).unwrap();
        let clone = b.clone_list(header).unwrap();
        assert_ne!(clone, header);
        b.set_modifiers(header, u32::MAX).unwrap();
        b.set(tail, 0, node(5)).unwrap();
        assert_eq!(b.get(full, 1), Ok(node(5)));
        assert_eq!(b.list(clone).unwrap().3, 0);
        let p = b.publish().unwrap();
        assert_eq!(p.list(header).unwrap(), (-1, i32::MAX, full, u32::MAX));
        assert_eq!(p.get(tail, 1), Ok(node(3)));
        assert_eq!(p.get(tail, 2), Err(Error::InvalidSlice));
    }
    #[test]
    fn release_boundaries_reject_wrong_owner_invalid_slots_and_open_frames() {
        let mut b = Builder::<2>::new(1, u32::MAX);
        let frame = b.begin().unwrap();
        assert_eq!(
            b.push(&frame, Some(NodeRef { owner: 2, slot: 1 })),
            Err(Error::WrongOwner)
        );
        assert_eq!(b.push(&frame, node(0)), Err(Error::InvalidNode));
        b.push(&frame, node(u32::MAX)).unwrap();
        let slice = b.finish(&frame).unwrap();
        assert_eq!(b.get(slice, 0), Ok(node(u32::MAX)));
        let other = Builder::<2>::new(2, 5);
        assert_eq!(other.get(slice, 0), Err(Error::WrongOwner));
        assert_eq!(
            b.get(
                Slice {
                    local: LocalSlice {
                        start: u32::MAX,
                        len: 2,
                        ..slice.local
                    },
                    ..slice
                },
                0
            ),
            Err(Error::InvalidSlice)
        );
        let _frame = b.begin().unwrap();
        assert!(matches!(b.publish(), Err(Error::Incomplete)));
    }
    #[test]
    fn independent_builders_share_sentinels_but_never_frames_or_backings() {
        let mut a = Builder::<2>::new(1, 5);
        let mut b = Builder::<2>::new(1, 5);
        b.new_list(0, 0, a.nil()).unwrap();
        b.new_list(0, 0, a.missing()).unwrap();
        let frame_a = a.begin().unwrap();
        let frame_b = b.begin().unwrap();
        assert_eq!(b.push(&frame_a, node(1)), Err(Error::WrongOwner));
        a.push(&frame_a, node(1)).unwrap();
        let slice_a = a.finish(&frame_a).unwrap();
        assert_eq!(b.get(slice_a, 0), Err(Error::WrongOwner));
        b.abort(&frame_b).unwrap();
        assert!(a.publish().is_ok() && b.publish().is_ok());
    }
    #[test]
    fn abort_discards_staging_but_preserves_completed_children_and_outer_prefix() {
        let mut b = Builder::<2>::new(1, 5);
        let outer = b.begin().unwrap();
        b.push(&outer, node(1)).unwrap();
        let inner = b.begin().unwrap();
        b.push(&inner, node(2)).unwrap();
        b.abort(&inner).unwrap();
        b.push(&outer, node(3)).unwrap();
        let slice = b.finish(&outer).unwrap();
        assert_eq!(b.get(slice, 1), Ok(node(3)));
        assert_eq!(b.stats().edge_words, 2);
    }
    #[test]
    fn backing_offsets_have_a_full_usize_escape_and_compact_ordinary_layout() {
        assert_eq!(std::mem::size_of::<Backing>(), 8);
        assert_eq!(compact_start(u32::MAX as usize - 1), (u32::MAX - 1, None));
        assert_eq!(
            compact_start(u32::MAX as usize),
            (u32::MAX, Some(u32::MAX as usize))
        );
        assert_eq!(compact_start(usize::MAX), (u32::MAX, Some(usize::MAX)));
    }
}
