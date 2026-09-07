use crate::{counters::Track, FileBuilder, FileId, FileOwner};
use std::{ops::Deref, sync::Arc};

/// One retention root owns every file in a content-mapped parse result.
pub struct BundleOwner<N, S = ()> {
    files: Vec<Arc<FileOwner<N, S>>>,
    _owner: Track,
}

impl<N, S> std::fmt::Debug for BundleOwner<N, S> {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        output
            .debug_struct("BundleOwner")
            .field("files", &self.files)
            .finish_non_exhaustive()
    }
}

impl<N, S> BundleOwner<N, S> {
    /// Consumes builders so a mapped member never escapes as a standalone owner.
    pub fn new(canonical: FileBuilder<N, S>, supplemental: Vec<FileBuilder<N, S>>) -> Arc<Self> {
        let counters = canonical.counters.clone();
        let canonical_id = canonical.id();
        let ids = supplemental.iter().map(FileBuilder::id).collect();
        let mut files = vec![Arc::new(canonical.publish(None, ids))];
        files.extend(
            supplemental
                .into_iter()
                .map(|file| Arc::new(file.publish(Some(canonical_id), Vec::new()))),
        );
        Arc::new(Self {
            files,
            _owner: counters.owner(),
        })
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    pub fn file(self: &Arc<Self>, index: usize) -> Option<FileHandle<N, S>> {
        (index < self.files.len()).then(|| FileHandle {
            root: Root::Bundle(self.clone(), index),
        })
    }
}

pub(crate) enum Root<N, S> {
    File(Arc<FileOwner<N, S>>),
    Bundle(Arc<BundleOwner<N, S>>, usize),
}

/// An owning file reference. Every mapped handle keeps all sibling files alive.
pub struct FileHandle<N, S = ()> {
    pub(crate) root: Root<N, S>,
}

impl<N, S> std::fmt::Debug for FileHandle<N, S> {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        output
            .debug_struct("FileHandle")
            .field("id", &self.id())
            .field("bundled", &matches!(self.root, Root::Bundle(..)))
            .finish()
    }
}

impl<N, S> Clone for FileHandle<N, S> {
    fn clone(&self) -> Self {
        Self {
            root: match &self.root {
                Root::File(file) => Root::File(file.clone()),
                Root::Bundle(bundle, index) => Root::Bundle(bundle.clone(), *index),
            },
        }
    }
}

impl<N, S> Deref for FileHandle<N, S> {
    type Target = FileOwner<N, S>;
    fn deref(&self) -> &Self::Target {
        match &self.root {
            Root::File(file) => file,
            Root::Bundle(bundle, index) => &bundle.files[*index],
        }
    }
}

impl<N, S> FileHandle<N, S> {
    /// Follow a mapper link through its complete retention group.
    pub fn file(&self, id: FileId) -> Option<Self> {
        match &self.root {
            Root::File(file) => (file.id() == id).then(|| self.clone()),
            Root::Bundle(bundle, _) => bundle
                .files
                .iter()
                .position(|file| file.id() == id)
                .and_then(|index| bundle.file(index)),
        }
    }

    pub(crate) fn into_members(self) -> Vec<Self> {
        match self.root {
            Root::File(file) => vec![Self {
                root: Root::File(file),
            }],
            Root::Bundle(bundle, _) => (0..bundle.len())
                .map(|index| Self {
                    root: Root::Bundle(bundle.clone(), index),
                })
                .collect(),
        }
    }
}
