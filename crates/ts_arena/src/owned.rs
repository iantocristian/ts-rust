//! Exclusive storage embedded in an outer owner, without an owning back-reference.
//! Publishing a concrete result restricts access to immutable borrows; this generic
//! primitive does not validate edges inside arbitrary payloads.

use crate::{arena::Arena, ArenaId, AuxId, Counters, Error, SymbolId};

macro_rules! owned_arena {
    ($name:ident, $id:ident) => {
        pub struct $name<T>(Arena<T>);
        impl<T> $name<T> {
            pub fn new(counters: &Counters) -> Self {
                Self(Arena::new(counters))
            }
            pub fn id(&self) -> ArenaId {
                self.0.id
            }
            pub fn len(&self) -> usize {
                self.0.len()
            }
            pub fn is_empty(&self) -> bool {
                self.len() == 0
            }
            pub fn push(&mut self, value: T) -> $id {
                $id::new(self.0.id, self.0.push(value))
            }
            pub fn get(&self, id: $id) -> Result<&T, Error> {
                self.0.get(id.arena(), id.slot())
            }
            pub fn get_mut(&mut self, id: $id) -> Result<&mut T, Error> {
                self.0.get_mut(id.arena(), id.slot())
            }
            pub fn iter(&self) -> impl Iterator<Item = ($id, &T)> {
                self.0
                    .values()
                    .enumerate()
                    .map(|(index, value)| ($id::new(self.0.id, index as u32 + 1), value))
            }
        }
        impl<T> std::fmt::Debug for $name<T> {
            fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                out.debug_struct(stringify!($name))
                    .field("id", &self.id())
                    .field("len", &self.len())
                    .finish_non_exhaustive()
            }
        }
    };
}
owned_arena!(OwnedArena, AuxId);
owned_arena!(SymbolArena, SymbolId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_arenas_reject_foreign_and_unpublished_slots_and_never_recycle() {
        let counters = Counters::new();
        let before = counters.snapshot();
        let stale = {
            let mut first = SymbolArena::new(&counters);
            let id = first.push(1);
            assert_eq!(*first.get(id).unwrap(), 1);
            *first.get_mut(id).unwrap() = 2;
            assert_eq!(
                first.iter().map(|(_, value)| *value).collect::<Vec<_>>(),
                [2]
            );
            let unpublished = SymbolId::new(first.id(), 2);
            assert!(matches!(first.get(unpublished), Err(Error::InvalidSlot)));
            let mut second = SymbolArena::new(&counters);
            second.push(3);
            assert!(matches!(second.get(id), Err(Error::WrongOwner)));
            assert!(matches!(second.get_mut(id), Err(Error::WrongOwner)));
            id
        };
        assert_eq!(counters.snapshot(), before);
        let mut replacement = SymbolArena::new(&counters);
        assert_ne!(replacement.push(4).arena(), stale.arena());
        assert!(matches!(replacement.get(stale), Err(Error::WrongOwner)));
        let mut aux = OwnedArena::new(&counters);
        let id = aux.push(5);
        assert_eq!(*aux.get(id).unwrap(), 5);
        let foreign = OwnedArena::<i32>::new(&counters);
        assert!(matches!(foreign.get(id), Err(Error::WrongOwner)));
    }
}
