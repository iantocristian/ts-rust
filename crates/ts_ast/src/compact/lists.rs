//! Physical syntax edges: local core slots share fixed 256-word pages.
//!
//! Backing descriptors retain their auxiliary identities. Exceptional edges
//! retain full non-owning IDs; the enclosing owner retains their dependencies.

use crate::NodeId;
use std::{collections::BTreeMap, ops::Range};
use ts_arena::{ArenaId, Error};

const PAGE_WORDS: usize = 256;
const ESCAPE: u32 = u32::MAX;

/// A physical backing in one owner's edge pages. The public syntax slice adds
/// its own offset and keeps its stable auxiliary identity separately.
#[derive(Clone, Copy, Debug)]
pub struct CompactNodes {
    pub(crate) start: usize,
    pub(crate) len: u32,
}

#[derive(Default)]
pub(crate) struct EdgePages {
    #[allow(clippy::vec_box)] // Directory growth must not copy existing edge pages.
    pages: Vec<Box<[u32; PAGE_WORDS]>>,
    len: usize,
    escapes: BTreeMap<usize, NodeId>,
}

impl EdgePages {
    pub(crate) fn append(
        &mut self,
        owner: ArenaId,
        nodes: &[Option<NodeId>],
    ) -> Result<CompactNodes, Error> {
        let len = u32::try_from(nodes.len()).map_err(|_| Error::InvalidSlot)?;
        self.len
            .checked_add(nodes.len())
            .ok_or(Error::InvalidSlot)?;
        let start = self.len;
        for &node in nodes {
            let index = self.len;
            if index.is_multiple_of(PAGE_WORDS) {
                self.pages.push(Box::new([0; PAGE_WORDS]));
            }
            let word = self.encode(owner, index, node);
            self.pages[index / PAGE_WORDS][index % PAGE_WORDS] = word;
            self.len += 1;
        }
        Ok(CompactNodes { start, len })
    }

    fn encode(&mut self, owner: ArenaId, index: usize, node: Option<NodeId>) -> u32 {
        match node {
            None => 0,
            Some(node) if node.arena() == owner && node.slot() != ESCAPE => node.slot(),
            Some(node) => {
                self.escapes.insert(index, node);
                ESCAPE
            }
        }
    }

    #[allow(clippy::option_option)] // A present nil edge differs from an out-of-bounds index.
    pub(crate) fn get(&self, owner: ArenaId, index: usize) -> Option<Option<NodeId>> {
        if index >= self.len {
            return None;
        }
        let word = self.pages[index / PAGE_WORDS][index % PAGE_WORDS];
        Some(match word {
            0 => None,
            ESCAPE => Some(
                *self
                    .escapes
                    .get(&index)
                    .expect("escaped edge has a full identity"),
            ),
            slot => {
                Some(NodeId::from_parts(owner, slot).expect("stored local edge slot is nonzero"))
            }
        })
    }

    pub(crate) fn valid_range(&self, range: Range<usize>) -> bool {
        range.start <= range.end && range.end <= self.len
    }

    pub(crate) fn iter(
        &self,
        owner: ArenaId,
        range: Range<usize>,
    ) -> impl DoubleEndedIterator<Item = Option<NodeId>> + ExactSizeIterator + '_ {
        assert!(
            self.valid_range(range.clone()),
            "validated compact edge range"
        );
        range.map(move |index| {
            self.get(owner, index)
                .expect("validated compact edge index")
        })
    }

    /// Change one existing cell without replacing any copied backing header.
    /// Callers must validate the new ID against the owner's retained graph.
    #[allow(dead_code)] // Syntax currently mutates list headers, not existing edge cells.
    pub(crate) fn set(
        &mut self,
        owner: ArenaId,
        index: usize,
        node: Option<NodeId>,
    ) -> Result<(), Error> {
        if index >= self.len {
            return Err(Error::InvalidSlot);
        }
        self.escapes.remove(&index);
        let word = self.encode(owner, index, node);
        self.pages[index / PAGE_WORDS][index % PAGE_WORDS] = word;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owner() -> ArenaId {
        ts_arena::StorageBuilder::<ts_arena::Node<()>>::from_source_text(
            ts_jsstring::SourceText::default(),
            &ts_arena::Counters::new(),
        )
        .id()
        .arena()
    }

    #[test]
    fn pages_preserve_cross_page_nil_full_slot_and_foreign_edges() {
        let owner = owner();
        let foreign = NodeId::from_parts(self::owner(), 1).unwrap();
        let local = NodeId::from_parts(owner, 7).unwrap();
        let full = NodeId::from_parts(owner, u32::MAX).unwrap();
        let mut pages = EdgePages::default();
        let empty = pages.append(owner, &[]).unwrap();
        assert_eq!(empty.len, 0);
        assert!(pages.pages.is_empty());
        let prefix = pages.append(owner, &vec![Some(local); 255]).unwrap();
        let backing = pages
            .append(owner, &[None, Some(foreign), Some(full), Some(local)])
            .unwrap();
        assert_eq!(prefix.start, 0);
        assert_eq!(backing.start, 255);
        assert_eq!(pages.pages.len(), 2);
        assert_eq!(pages.escapes.len(), 2);
        assert_eq!(
            pages.iter(owner, 254..259).collect::<Vec<_>>(),
            vec![Some(local), None, Some(foreign), Some(full), Some(local)]
        );
        assert_eq!(
            pages.iter(owner, 255..259).rev().collect::<Vec<_>>(),
            vec![Some(local), Some(full), Some(foreign), None]
        );
        assert_eq!(pages.get(owner, 259), None);
        assert_eq!(pages.get(owner, usize::MAX), None);
        assert!(!pages.valid_range(259..260));
        assert!(pages.iter(owner, 259..259).next().is_none());
    }

    #[test]
    fn cell_changes_preserve_backing_aliases_and_remove_stale_escapes() {
        let owner = owner();
        let local = NodeId::from_parts(owner, 3).unwrap();
        let foreign = NodeId::from_parts(self::owner(), 3).unwrap();
        let full = NodeId::from_parts(owner, u32::MAX).unwrap();
        let mut pages = EdgePages::default();
        let backing = pages.append(owner, &[Some(foreign)]).unwrap();
        let copied = backing;
        for value in [Some(full), Some(local), None, Some(foreign)] {
            pages.set(owner, backing.start, value).unwrap();
            assert_eq!(pages.get(owner, copied.start), Some(value));
            assert_eq!(
                pages.escapes.len(),
                usize::from(matches!(value, Some(id) if id == full || id == foreign))
            );
        }
        assert_eq!(pages.set(owner, 1, None), Err(Error::InvalidSlot));
        assert_eq!(pages.get(owner, 0), Some(Some(foreign)));
    }
}
