//! Stable fixed pages. Allocation of spare entries is charged by the native probe.

pub(crate) struct Pages<T, const N: usize> {
    pages: Vec<Box<[T; N]>>,
    len: usize,
}

impl<T, const N: usize> Default for Pages<T, N> {
    fn default() -> Self {
        assert!(N != 0, "page width is nonzero");
        Self {
            pages: Vec::new(),
            len: 0,
        }
    }
}

impl<T: Default, const N: usize> Pages<T, N> {
    pub(crate) fn push(&mut self, value: T) -> usize {
        let index = self.len;
        if index.is_multiple_of(N) {
            self.pages
                .push(Box::new(std::array::from_fn(|_| T::default())));
        }
        self.pages[index / N][index % N] = value;
        self.len = self.len.checked_add(1).expect("page length overflow");
        index
    }
}

impl<T, const N: usize> Pages<T, N> {
    /// Borrow contiguous spans after one logical range check. This does not
    /// manufacture a single slice across discontiguous allocations.
    pub(crate) fn visit_range(
        &self,
        range: std::ops::Range<usize>,
        mut visit: impl FnMut(&T),
    ) -> Option<()> {
        if range.start > range.end || range.end > self.len {
            return None;
        }
        let mut start = range.start;
        while start < range.end {
            let page = &self.pages[start / N];
            let within = start % N;
            let count = (N - within).min(range.end - start);
            for value in &page[within..within + count] {
                visit(value);
            }
            start += count;
        }
        Some(())
    }
    pub(crate) fn get(&self, index: usize) -> Option<&T> {
        (index < self.len).then(|| &self.pages[index / N][index % N])
    }
    pub(crate) fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        (index < self.len).then(|| &mut self.pages[index / N][index % N])
    }
    pub(crate) fn len(&self) -> usize {
        self.len
    }
    pub(crate) fn capacity(&self) -> usize {
        self.pages.len() * N
    }
    pub(crate) fn page_count(&self) -> usize {
        self.pages.len()
    }
    pub(crate) fn directory_capacity(&self) -> usize {
        self.pages.capacity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spans_preserve_order_across_pages_and_never_expose_spare() {
        let mut pages = Pages::<u32, 2>::default();
        for value in 1..=5 {
            pages.push(value);
        }
        let mut observed = Vec::new();
        assert_eq!(
            pages.visit_range(1..5, |value| observed.push(*value)),
            Some(())
        );
        assert_eq!(observed, vec![2, 3, 4, 5]);
        assert_eq!(pages.visit_range(5..5, |_| panic!("empty span")), Some(()));
        assert_eq!(pages.visit_range(4..6, |_| panic!("invalid span")), None);
    }
}
