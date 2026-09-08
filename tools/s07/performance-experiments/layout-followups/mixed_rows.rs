//! Additive layout proof only: no production storage or timing measurements.
#![forbid(unsafe_code)]
#![allow(dead_code)]

use std::mem::{align_of, size_of};
use std::sync::atomic::AtomicU32;

#[repr(C)]
struct PlainRow<const N: usize> {
    words: [u32; N],
}

#[repr(C)]
struct CompositeRow<const N: usize> {
    facts: AtomicU32,
    words: [u32; N],
}

impl<const N: usize> PlainRow<N> {
    fn word(&self, index: usize) -> Option<&u32> {
        self.words.get(index)
    }
}

impl<const N: usize> CompositeRow<N> {
    fn word(&self, index: usize) -> Option<&u32> {
        self.words.get(index)
    }

    fn word_mut(&mut self, index: usize) -> Option<&mut u32> {
        self.words.get_mut(index)
    }
}

// Envelopes already charged by the previous model. Confirm that using mixed
// records does not change pointer, store or owner-ledger layouts.
enum OneOrMany<P> {
    One(P),
    Many(Vec<P>),
}
struct OneOrManyStore<P> {
    pages: OneOrMany<P>,
    used: usize,
}
struct HeaderStore<P> {
    pages: Vec<P>,
    used: usize,
    counters_handle: usize,
    tracked_pages: usize,
}

fn main() {
    macro_rules! row {
        ($width:literal) => {
            println!(
                "plain.{} {} {}",
                $width,
                size_of::<PlainRow<$width>>(),
                align_of::<PlainRow<$width>>()
            );
            println!(
                "composite.{} {} {}",
                $width,
                size_of::<CompositeRow<{ $width - 1 }>>(),
                align_of::<CompositeRow<{ $width - 1 }>>()
            );
            println!(
                "atomic.{} {} {}",
                $width,
                size_of::<[AtomicU32; $width]>(),
                align_of::<[AtomicU32; $width]>()
            );
        };
    }
    println!(
        "plain.0 {} {}",
        size_of::<PlainRow<0>>(),
        align_of::<PlainRow<0>>()
    );
    row!(1);
    row!(2);
    row!(3);
    row!(4);
    row!(5);
    row!(6);
    row!(7);
    row!(8);
    row!(9);
    row!(10);
    row!(11);
    row!(12);
    row!(13);
    row!(14);
    row!(15);
    row!(16);
    println!(
        "ThinPage {} {}",
        size_of::<Box<[CompositeRow<3>; 8]>>(),
        align_of::<Box<[CompositeRow<3>; 8]>>()
    );
    println!(
        "ThinOwnerOneOrManyStore {} {}",
        size_of::<OneOrManyStore<Box<[CompositeRow<3>; 8]>>>(),
        align_of::<OneOrManyStore<Box<[CompositeRow<3>; 8]>>>()
    );
    println!(
        "ThinStore {} {}",
        size_of::<HeaderStore<Box<[u32; 32]>>>(),
        align_of::<HeaderStore<Box<[u32; 32]>>>()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::offset_of;
    use std::sync::atomic::Ordering;

    #[test]
    fn composite_scalar_count_excludes_the_existing_facts_word() {
        assert_eq!(size_of::<CompositeRow<3>>(), size_of::<PlainRow<4>>());
        assert_eq!(size_of::<CompositeRow<0>>(), size_of::<AtomicU32>());
        assert_eq!(offset_of!(CompositeRow<3>, words), size_of::<AtomicU32>());
        assert_eq!(size_of::<PlainRow<0>>(), 0);
        assert_eq!(offset_of!(PlainRow<4>, words), 0);
    }

    #[test]
    fn shared_scalars_are_plain_borrows_and_do_not_alias_facts() {
        let plain = PlainRow {
            words: [u32::MAX, 0],
        };
        let value: &u32 = plain.word(0).unwrap();
        assert_eq!(*value, u32::MAX);
        let mut composite = CompositeRow {
            facts: AtomicU32::new(17),
            words: [u32::MAX, 0],
        };
        let value: &u32 = composite.word(0).unwrap();
        assert_eq!(*value, u32::MAX);
        *composite.word_mut(1).unwrap() = 23;
        assert_eq!(composite.facts.load(Ordering::Relaxed), 17);
        composite.facts.store(u32::MAX, Ordering::Relaxed);
        assert_eq!(composite.word(1), Some(&23));
        assert!(plain.word(usize::MAX).is_none());
        assert!(composite.word(2).is_none());
        assert!(composite.word_mut(usize::MAX).is_none());
    }
}
