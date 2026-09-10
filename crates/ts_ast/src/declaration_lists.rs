//! Binding-owned declaration slice headers and shared backing storage.
use crate::{compact::lists::EdgePages, NodeId};
use std::ops::Range;
use ts_arena::{ArenaId, AuxId, Counters, Error, OwnedArena};

/// A copied Go declaration-slice header. Nil, length and capacity are separate;
/// replacing a header does not replace the backing seen through other headers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DeclarationSlice {
    backing: Option<AuxId>,
    start: u32,
    len: u32,
    capacity: u32,
}
impl DeclarationSlice {
    pub(crate) fn from_storage_parts(
        backing: Option<AuxId>,
        start: u32,
        len: u32,
        capacity: u32,
    ) -> Self {
        Self {
            backing,
            start,
            len,
            capacity,
        }
    }
    /// Non-owning backing identity for graph observation; it cannot construct a
    /// slice or grant mutation of a published binding result.
    pub fn backing_id(self) -> Option<AuxId> {
        self.backing
    }
    pub fn start(self) -> u32 {
        self.start
    }
    pub const fn empty() -> Self {
        Self {
            backing: None,
            start: 0,
            len: 0,
            capacity: 0,
        }
    }
    pub fn is_nil(self) -> bool {
        self.backing.is_none()
    }
    pub fn is_empty(self) -> bool {
        self.len == 0
    }
    pub fn len(self) -> usize {
        self.len as usize
    }
    pub fn capacity(self) -> usize {
        self.capacity as usize
    }
    pub fn same(self, other: Self) -> bool {
        self.len == other.len
            && (self.len == 0 || self.backing == other.backing && self.start == other.start)
    }
    /// Go's two-index slice may extend to capacity, including previously hidden
    /// elements written through another copied header.
    pub fn slice(self, range: Range<usize>) -> Result<Self, Error> {
        self.slice_with_capacity(range, self.capacity())
    }
    pub fn slice_with_capacity(self, range: Range<usize>, max: usize) -> Result<Self, Error> {
        if range.start > range.end || range.end > max || max > self.capacity() {
            return Err(Error::InvalidSlot);
        }
        Ok(Self {
            backing: self.backing,
            start: self
                .start
                .checked_add(range.start as u32)
                .ok_or(Error::InvalidSlot)?,
            len: range.len() as u32,
            capacity: (max - range.start) as u32,
        })
    }
}
#[derive(Clone, Copy, Debug)]
struct DeclarationBacking {
    start: usize,
    capacity: u32,
}

/// A scoped semantic read. Backing identity and capacity stay in DeclarationSlice;
/// decoded full IDs are returned by value, never as references into compact words.
#[derive(Clone, Copy)]
pub struct DeclarationRead<'a> {
    pages: &'a EdgePages,
    owner: ArenaId,
    start: usize,
    len: usize,
}
static EMPTY_DECLARATION_PAGES: EdgePages = EdgePages::empty();
impl std::fmt::Debug for DeclarationRead<'_> {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.debug_list().entries(self.iter()).finish()
    }
}
impl DeclarationRead<'_> {
    /// A transient host's nil declarations need no backing allocation or new owner.
    pub fn empty(owner: ArenaId) -> Self {
        Self {
            pages: &EMPTY_DECLARATION_PAGES,
            owner,
            start: 0,
            len: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    #[allow(clippy::option_option)] // Nil cells are distinct from out-of-range indices.
    pub fn get(&self, index: usize) -> Option<Option<NodeId>> {
        if index >= self.len {
            return None;
        }
        self.pages.get(self.owner, self.start + index)
    }
    pub fn at(&self, index: usize) -> Option<NodeId> {
        self.get(index)
            .expect("declaration slice index out of bounds")
    }
    #[allow(clippy::option_option)]
    pub fn first(&self) -> Option<Option<NodeId>> {
        self.get(0)
    }
    #[allow(clippy::option_option)]
    pub fn last(&self) -> Option<Option<NodeId>> {
        self.len.checked_sub(1).and_then(|index| self.get(index))
    }
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = Option<NodeId>> + ExactSizeIterator + '_ {
        self.pages
            .iter(self.owner, self.start..self.start + self.len)
    }
    pub fn contains(&self, node: &Option<NodeId>) -> bool {
        self.iter().any(|value| value == *node)
    }
    pub fn to_vec(&self) -> Vec<Option<NodeId>> {
        self.iter().collect()
    }
}

/// Declaration backings retain stable identities while their cells share edge
/// pages. The first nonnil identity selects the local namespace; earlier nil
/// cells need no namespace. Foreign and full-slot IDs remain non-owning escapes.
pub struct DeclarationLists {
    backings: OwnedArena<DeclarationBacking>,
    pages: EdgePages,
    nodes: Option<ArenaId>,
}
impl std::fmt::Debug for DeclarationLists {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.debug_struct("DeclarationLists")
            .field("backings", &self.backings)
            .field("nodes", &self.nodes)
            .finish_non_exhaustive()
    }
}
impl DeclarationLists {
    pub fn new(counters: &Counters) -> Self {
        Self {
            backings: OwnedArena::new(counters),
            pages: EdgePages::default(),
            nodes: None,
        }
    }
    pub fn id(&self) -> ArenaId {
        self.backings.id()
    }
    fn node_arena(&self) -> ArenaId {
        // Without a nonnil node all stored words are zero, so this temporary
        // namespace cannot affect a decoded identity.
        self.nodes.unwrap_or_else(|| self.id())
    }
    fn observe_node(&mut self, node: Option<NodeId>) {
        if self.nodes.is_none() {
            self.nodes = node.map(NodeId::arena);
        }
    }
    fn read_range(&self, range: Range<usize>) -> DeclarationRead<'_> {
        DeclarationRead {
            pages: &self.pages,
            owner: self.node_arena(),
            start: range.start,
            len: range.len(),
        }
    }
    fn range(&self, slice: DeclarationSlice) -> Result<Range<usize>, Error> {
        let Some(id) = slice.backing else {
            return Ok(0..0);
        };
        let backing = self.backings.get(id)?;
        let end = slice
            .start
            .checked_add(slice.len)
            .ok_or(Error::InvalidSlot)?;
        if end > backing.capacity {
            return Err(Error::InvalidSlot);
        }
        let start = backing
            .start
            .checked_add(slice.start as usize)
            .ok_or(Error::InvalidSlot)?;
        let end = start.checked_add(slice.len()).ok_or(Error::InvalidSlot)?;
        Ok(start..end)
    }
    pub fn alloc(&mut self, values: Vec<Option<NodeId>>) -> Result<DeclarationSlice, Error> {
        let len = values.len();
        self.alloc_with_capacity(values, len)
    }
    /// The common singleton path writes directly into the shared page.
    pub fn alloc_one(&mut self, node: Option<NodeId>) -> Result<DeclarationSlice, Error> {
        self.observe_node(node);
        let backing = self.pages.append(self.node_arena(), &[node])?;
        Ok(self.allocate_header(backing.start, 1, 1))
    }
    pub fn alloc_with_capacity(
        &mut self,
        values: Vec<Option<NodeId>>,
        capacity: usize,
    ) -> Result<DeclarationSlice, Error> {
        let len = u32::try_from(values.len()).map_err(|_| Error::InvalidSlot)?;
        let capacity = u32::try_from(capacity).map_err(|_| Error::InvalidSlot)?;
        if len > capacity {
            return Err(Error::InvalidSlot);
        }
        if self.nodes.is_none() {
            self.observe_node(values.iter().flatten().next().copied());
        }
        let mut values = values.into_iter();
        let backing = self.pages.append_iter(
            self.node_arena(),
            (0..capacity).map(|_| values.next().flatten()),
        )?;
        Ok(self.allocate_header(backing.start, len, capacity))
    }
    fn allocate_header(&mut self, start: usize, len: u32, capacity: u32) -> DeclarationSlice {
        DeclarationSlice {
            backing: Some(self.backings.push(DeclarationBacking { start, capacity })),
            start: 0,
            len,
            capacity,
        }
    }
    pub fn get(&self, slice: DeclarationSlice) -> Result<DeclarationRead<'_>, Error> {
        self.range(slice).map(|range| self.read_range(range))
    }
    /// Change a semantic cell without replacing the backing shared by copied
    /// headers. This replaces unrestricted mutable slices; compact words stay private.
    pub fn set(
        &mut self,
        slice: DeclarationSlice,
        index: usize,
        node: Option<NodeId>,
    ) -> Result<(), Error> {
        let range = self.range(slice)?;
        if index >= range.len() {
            return Err(Error::InvalidSlot);
        }
        self.observe_node(node);
        self.pages.set(self.node_arena(), range.start + index, node)
    }
    pub fn iter(&self) -> impl Iterator<Item = (AuxId, DeclarationRead<'_>)> {
        self.backings.iter().map(|(id, backing)| {
            (
                id,
                self.read_range(backing.start..backing.start + backing.capacity as usize),
            )
        })
    }
    pub fn append(
        &mut self,
        slice: DeclarationSlice,
        node: Option<NodeId>,
    ) -> Result<DeclarationSlice, Error> {
        let old = self.range(slice)?;
        let len = slice.len.checked_add(1).ok_or(Error::InvalidSlot)?;
        if slice.len == 0 && slice.capacity == 0 {
            // A valid zero-capacity header has no elements. Go's first pointer
            // slice allocation has capacity one, whether its old header was nil.
            return self.alloc_one(node);
        }
        if len <= slice.capacity {
            let id = slice.backing.expect("nonzero capacity has backing");
            let backing = self.backings.get(id)?;
            let index = slice.start as usize + slice.len();
            if index >= backing.capacity as usize {
                return Err(Error::InvalidSlot);
            }
            let index = backing.start + index;
            self.observe_node(node);
            self.pages.set(self.node_arena(), index, node)?;
            return Ok(DeclarationSlice { len, ..slice });
        }
        let capacity = declaration_growth_capacity(len as usize, slice.capacity())?;
        self.observe_node(node);
        let arena = self.node_arena();
        let copied_len = u32::try_from(old.len() + 1).map_err(|_| Error::InvalidSlot)?;
        if old.is_empty() {
            let mut first = Some(node);
            let backing = self
                .pages
                .append_iter(arena, (0..capacity).map(|_| first.take().flatten()))?;
            return Ok(self.allocate_header(backing.start, copied_len, capacity as u32));
        }
        // Keep old backing cells observable. Reserve a new full-capacity range,
        // then copy semantic values directly without a transient Vec or box.
        let backing = self.pages.append_iter(arena, (0..capacity).map(|_| None))?;
        for index in 0..old.len() {
            let value = self
                .pages
                .get(arena, old.start + index)
                .expect("validated declaration backing cell");
            self.pages.set(arena, backing.start + index, value)?;
        }
        self.pages.set(arena, backing.start + old.len(), node)?;
        Ok(self.allocate_header(backing.start, copied_len, capacity as u32))
    }
    pub fn append_if_unique(
        &mut self,
        slice: DeclarationSlice,
        node: Option<NodeId>,
    ) -> Result<DeclarationSlice, Error> {
        if self.get(slice)?.contains(&node) {
            return Ok(slice);
        }
        self.append(slice, node)
    }
}

// Pinned Go 1.27.1 runtime nextslicecap/roundupsize for []*Node on the project's
// 64-bit targets. Pointerful allocation rounding affects observable slice aliasing.
fn declaration_growth_capacity(len: usize, old: usize) -> Result<usize, Error> {
    let double = old.checked_mul(2).ok_or(Error::InvalidSlot)?;
    let mut capacity = if len > double {
        len
    } else if old < 256 {
        double
    } else {
        old
    };
    while capacity < len {
        capacity = capacity
            .checked_add((capacity + 3 * 256) >> 2)
            .ok_or(Error::InvalidSlot)?;
    }
    let size = capacity.checked_mul(8).ok_or(Error::InvalidSlot)?;
    let rounded = if size <= 32768 - 8 {
        const CLASSES: &[usize] = &[
            8, 16, 24, 32, 48, 64, 80, 96, 112, 128, 144, 160, 176, 192, 208, 224, 240, 256, 288,
            320, 352, 384, 416, 448, 480, 512, 576, 640, 704, 768, 896, 1024, 1152, 1280, 1408,
            1536, 1792, 2048, 2304, 2688, 3072, 3200, 3456, 4096, 4864, 5376, 6144, 6528, 6784,
            6912, 8192, 9472, 9728, 10240, 10880, 12288, 13568, 14336, 16384, 18432, 19072, 20480,
            21760, 24576, 27264, 28672, 32768,
        ];
        let header = if size > 512 { 8 } else { 0 };
        CLASSES
            .iter()
            .find(|&&class| class >= size + header)
            .copied()
            .ok_or(Error::InvalidSlot)?
            - header
    } else {
        size.checked_add(8191).ok_or(Error::InvalidSlot)? & !8191
    };
    let capacity = rounded / 8;
    u32::try_from(capacity).map_err(|_| Error::InvalidSlot)?;
    Ok(capacity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ts_arena::{Node, StorageBuilder};
    use ts_jsstring::SourceText;

    fn arena(counters: &Counters) -> ArenaId {
        StorageBuilder::<Node<()>>::from_source_text(SourceText::default(), counters)
            .id()
            .arena()
    }

    #[test]
    fn pooled_backings_preserve_cross_page_aliases_and_full_foreign_ids() {
        let counters = Counters::new();
        let local = NodeId::from_parts(arena(&counters), 7).unwrap();
        let foreign = NodeId::from_parts(arena(&counters), 9).unwrap();
        let full = NodeId::from_parts(local.arena(), u32::MAX).unwrap();
        let mut lists = DeclarationLists::new(&counters);
        let nil = DeclarationSlice::empty();
        let empty = lists.alloc(Vec::new()).unwrap();
        let prefix = lists.alloc_with_capacity(vec![None], 255).unwrap();
        let backing = lists
            .alloc_with_capacity(vec![Some(local), None, Some(foreign), Some(full)], 7)
            .unwrap();
        assert!(nil.is_nil() && !empty.is_nil());
        assert_ne!(empty.backing_id(), prefix.backing_id());
        assert!(lists.get(nil).unwrap().is_empty());
        assert_eq!(
            lists.get(prefix.slice(0..255).unwrap()).unwrap().to_vec(),
            vec![None; 255]
        );
        let hidden = backing.slice(1..7).unwrap();
        let visible = lists.append(backing, Some(foreign)).unwrap();
        assert_eq!(visible.backing_id(), backing.backing_id());
        assert_eq!(
            lists.get(hidden).unwrap().to_vec(),
            [None, Some(foreign), Some(full), Some(foreign), None, None]
        );
        for replacement in [Some(full), Some(local), None, Some(foreign)] {
            lists.set(visible, 2, replacement).unwrap();
            assert_eq!(lists.get(backing).unwrap().at(2), replacement);
            assert_eq!(lists.get(hidden).unwrap().at(1), replacement);
        }
        let read = lists.get(backing).unwrap();
        assert_eq!(read.get(1), Some(None));
        assert_eq!(read.get(read.len()), None);
        assert_eq!(read.get(usize::MAX), None);
        assert_eq!(
            read.iter().rev().collect::<Vec<_>>(),
            [Some(full), Some(foreign), None, Some(local)]
        );
        assert_eq!(read.first(), Some(Some(local)));
        assert_eq!(read.last(), Some(Some(full)));
        let full_backing = lists
            .iter()
            .find(|(id, _)| Some(*id) == backing.backing_id())
            .unwrap()
            .1;
        assert_eq!(full_backing.len(), 7);
        assert_eq!(full_backing.at(4), Some(foreign));
    }

    #[test]
    fn nil_first_growth_and_restricted_capacity_keep_old_backings_visible() {
        let counters = Counters::new();
        let first = NodeId::from_parts(arena(&counters), 1).unwrap();
        let second = NodeId::from_parts(arena(&counters), u32::MAX).unwrap();
        let mut lists = DeclarationLists::new(&counters);
        let empty = lists.alloc_with_capacity(Vec::new(), 4).unwrap();
        let one = lists.append(empty, None).unwrap();
        let copied = one;
        let two = lists.append(one, Some(first)).unwrap();
        assert_eq!(
            lists.get(copied.slice(0..2).unwrap()).unwrap().to_vec(),
            [None, Some(first)]
        );
        let restricted = two.slice_with_capacity(1..2, 2).unwrap();
        let detached = lists.append(restricted, Some(second)).unwrap();
        assert_ne!(detached.backing_id(), restricted.backing_id());
        assert_eq!(detached.start(), 0);
        assert_eq!(
            lists.get(detached).unwrap().to_vec(),
            [Some(first), Some(second)]
        );
        lists.set(detached, 0, None).unwrap();
        assert_eq!(lists.get(restricted).unwrap().to_vec(), [Some(first)]);
        assert_eq!(
            lists.append_if_unique(detached, Some(second)).unwrap(),
            detached
        );
        let singleton = lists.alloc_one(Some(first)).unwrap();
        assert_eq!((singleton.len(), singleton.capacity()), (1, 1));
        assert_eq!(lists.get(singleton).unwrap().at(0), Some(first));
    }

    #[test]
    fn backing_resolution_keeps_error_order_without_validating_node_identities() {
        let counters = Counters::new();
        let mut lists = DeclarationLists::new(&counters);
        let another = DeclarationLists::new(&counters);
        let forged = DeclarationSlice {
            backing: Some(AuxId::from_parts(another.id(), u32::MAX).unwrap()),
            start: u32::MAX,
            len: u32::MAX,
            capacity: u32::MAX,
        };
        assert!(matches!(lists.get(forged), Err(Error::WrongOwner)));
        assert_eq!(lists.append(forged, None), Err(Error::WrongOwner));
        assert_eq!(lists.set(forged, usize::MAX, None), Err(Error::WrongOwner));
        let unpublished = DeclarationSlice {
            backing: Some(AuxId::from_parts(lists.id(), 1).unwrap()),
            ..forged
        };
        assert!(matches!(lists.get(unpublished), Err(Error::InvalidSlot)));
        assert!(lists.alloc_with_capacity(vec![None], 0).is_err());
        let nil = DeclarationSlice::empty();
        assert_eq!(lists.set(nil, 0, None), Err(Error::InvalidSlot));
        // Declaration storage carries IDs; owning-graph validation remains later.
        let node = NodeId::from_parts(arena(&counters), u32::MAX).unwrap();
        let slice = lists.alloc_one(Some(node)).unwrap();
        assert_eq!(lists.get(slice).unwrap().at(0), Some(node));
    }
}
