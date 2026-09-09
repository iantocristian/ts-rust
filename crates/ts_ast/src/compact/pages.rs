//! Small typed pages keep payload addresses stable through exclusive growth.

enum Directory<T> {
    Empty,
    One(Box<[T; 4]>),
    Many(Vec<Box<[T; 4]>>),
}

pub(crate) struct RowPages<T> {
    directory: Directory<T>,
    len: u32,
}

impl<T> Default for RowPages<T> {
    fn default() -> Self {
        Self {
            directory: Directory::Empty,
            len: 0,
        }
    }
}

impl<T> RowPages<T> {
    pub(crate) fn len(&self) -> u32 {
        self.len
    }

    pub(crate) fn get(&self, ordinal: u32) -> Option<&T> {
        if ordinal >= self.len {
            return None;
        }
        let page = match &self.directory {
            Directory::Empty => return None,
            Directory::One(page) => page,
            Directory::Many(pages) => pages.get(ordinal as usize / 4)?,
        };
        page.get(ordinal as usize % 4)
    }

    pub(crate) fn get_mut(&mut self, ordinal: u32) -> Option<&mut T> {
        if ordinal >= self.len {
            return None;
        }
        let page = match &mut self.directory {
            Directory::Empty => return None,
            Directory::One(page) => page,
            Directory::Many(pages) => pages.get_mut(ordinal as usize / 4)?,
        };
        page.get_mut(ordinal as usize % 4)
    }
}

impl<T: Default> RowPages<T> {
    pub(crate) fn push(&mut self, value: T) -> u32 {
        let ordinal = self.len;
        let next = ordinal
            .checked_add(1)
            .expect("typed payload ordinal space exhausted");
        if ordinal.is_multiple_of(4) {
            let page = Box::new(std::array::from_fn(|_| T::default()));
            self.directory = match std::mem::replace(&mut self.directory, Directory::Empty) {
                Directory::Empty => Directory::One(page),
                Directory::One(first) => Directory::Many(vec![first, page]),
                Directory::Many(mut pages) => {
                    pages.push(page);
                    Directory::Many(pages)
                }
            };
        }
        self.len = next;
        *self.get_mut(ordinal).expect("newly allocated payload row") = value;
        ordinal
    }
}
