//! Per-shape rows use ordinary vector growth during exclusive construction.
//!
//! Borrowed payload reads prevent mutable growth. Published core rows never grow;
//! lazy materialization uses its separate fallback storage. Ordinals, rather than
//! pre-publication addresses, identify rows across vector reallocations.

pub(crate) struct TypedRows<T> {
    values: Vec<T>,
}

impl<T> Default for TypedRows<T> {
    fn default() -> Self {
        Self { values: Vec::new() }
    }
}

impl<T> TypedRows<T> {
    pub(crate) fn len(&self) -> u32 {
        u32::try_from(self.values.len()).expect("typed payload ordinal space exhausted")
    }

    pub(crate) fn get(&self, ordinal: u32) -> Option<&T> {
        self.values.get(ordinal as usize)
    }

    pub(crate) fn get_mut(&mut self, ordinal: u32) -> Option<&mut T> {
        self.values.get_mut(ordinal as usize)
    }

    pub(crate) fn push(&mut self, value: T) -> u32 {
        let ordinal = self.len();
        ordinal
            .checked_add(1)
            .expect("typed payload ordinal space exhausted");
        self.values.push(value);
        ordinal
    }
}
