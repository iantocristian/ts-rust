use crate::{counters::Track, ids::next_arena, ArenaId, Counters, Error};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, MutexGuard,
};

struct GenerationState {
    id: ArenaId,
    active: AtomicBool,
    _owner: Track,
}

/// A pool generation's permanent invalidation state. Retirement does not dispose
/// storage retained by owners/leases. The server publication gate is a later
/// integration requirement; this primitive never claims to commit responses.
#[derive(Clone)]
pub struct Generation(Arc<GenerationState>);
impl Generation {
    pub fn new(counters: &Counters) -> Self {
        Self(Arc::new(GenerationState {
            id: next_arena(),
            active: AtomicBool::new(true),
            _owner: counters.owner(),
        }))
    }
    pub fn id(&self) -> ArenaId {
        self.0.id
    }
    pub fn retire(&self) {
        self.0.active.store(false, Ordering::Release);
    }
    pub fn validate(&self) -> Result<(), Error> {
        if self.0.active.load(Ordering::Acquire) {
            Ok(())
        } else {
            Err(Error::Retired)
        }
    }
}

/// Exact-checker identity and exclusive operation permit, independent of checker
/// algorithms. The id reserves this checker's future symbol-arena identity.
pub struct CheckerIdentity {
    id: ArenaId,
    generation: Generation,
    operation: Mutex<()>,
    _owner: Track,
}
impl CheckerIdentity {
    pub fn new(generation: Generation, counters: &Counters) -> Arc<Self> {
        Arc::new(Self {
            id: next_arena(),
            generation,
            operation: Mutex::new(()),
            _owner: counters.owner(),
        })
    }
    pub fn id(&self) -> ArenaId {
        self.id
    }
    pub fn generation(&self) -> &Generation {
        &self.generation
    }
    pub fn lease(&self) -> Result<CheckerLease<'_>, Error> {
        self.generation.validate()?;
        let permit = self.operation.lock().map_err(|_| {
            self.generation.retire();
            Error::Retired
        })?;
        self.generation.validate()?;
        Ok(CheckerLease {
            owner: self,
            _permit: permit,
        })
    }
}

/// A non-transferable operation scope. Reentry must drop it and acquire another;
/// every import validates exact identity, active generation and published bounds.
pub struct CheckerLease<'a> {
    owner: &'a CheckerIdentity,
    _permit: MutexGuard<'a, ()>,
}

impl Drop for CheckerLease<'_> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            // Retire before releasing the permit, including sibling checkers
            // sharing this generation. Poison is only a defensive backstop.
            self.owner.generation.retire();
        }
    }
}

impl CheckerLease<'_> {
    pub fn validate_identity(&self, identity: ArenaId) -> Result<(), Error> {
        self.owner.generation.validate()?;
        if identity == self.owner.id {
            Ok(())
        } else {
            Err(Error::WrongOwner)
        }
    }
    /// Checker implementations supply the provenance stored with their retained
    /// handle and their actual published arena length; this returns a checked
    /// index and never creates an owning or freely transferable typed handle.
    pub fn check_slot(
        &self,
        identity: ArenaId,
        slot: u32,
        published: usize,
    ) -> Result<usize, Error> {
        self.validate_identity(identity)?;
        let index = slot.checked_sub(1).ok_or(Error::InvalidSlot)? as usize;
        if index < published {
            Ok(index)
        } else {
            Err(Error::InvalidSlot)
        }
    }
}
