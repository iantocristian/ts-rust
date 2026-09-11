use crate::{counters::Track, ids::next_arena, ArenaId, Counters, Error, SymbolArena};
use std::cell::RefCell;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, MutexGuard,
};

thread_local! {
    static ACTIVE_LEASES: RefCell<Vec<ArenaId>> = const { RefCell::new(Vec::new()) };
}

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
/// algorithms. The id reserves this checker's symbol-arena identity, which
/// `adopt_symbol_arena` hands out exactly once.
pub struct CheckerIdentity {
    id: ArenaId,
    generation: Generation,
    operation: Mutex<()>,
    adopted: AtomicBool,
    _owner: Track,
}
impl CheckerIdentity {
    pub fn new(generation: Generation, counters: &Counters) -> Arc<Self> {
        Arc::new(Self {
            id: next_arena(),
            generation,
            operation: Mutex::new(()),
            adopted: AtomicBool::new(false),
            _owner: counters.owner(),
        })
    }
    pub fn id(&self) -> ArenaId {
        self.id
    }
    /// The checker's own symbol arena, numbered with this identity. The second
    /// call fails: one identity never backs two arenas, and no public path
    /// accepts an arbitrary arena number.
    pub fn adopt_symbol_arena<T>(&self, counters: &Counters) -> Result<SymbolArena<T>, Error> {
        self.adopted
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| Error::IdentityAdopted)?;
        Ok(SymbolArena::with_id(self.id, counters))
    }
    pub fn generation(&self) -> &Generation {
        &self.generation
    }
    pub fn lease(&self) -> Result<CheckerLease<'_>, Error> {
        self.generation.validate()?;
        if ACTIVE_LEASES.with(|active| active.borrow().contains(&self.id)) {
            return Err(Error::Reentry);
        }
        #[cfg(test)]
        tests::observe_contention(&self.operation);
        let permit = self.operation.lock().map_err(|_| {
            self.generation.retire();
            Error::Retired
        })?;
        self.generation.validate()?;
        ACTIVE_LEASES.with(|active| active.borrow_mut().push(self.id));
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
        // A lease can live in TLS initialized before ACTIVE_LEASES, in which
        // case its reentry record has already gone away during thread teardown.
        let _ =
            ACTIVE_LEASES.try_with(|active| active.borrow_mut().retain(|id| *id != self.owner.id));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::sync::{mpsc, OnceLock, TryLockError};

    thread_local! {
        static CONTENTION_PROBE: RefCell<Option<mpsc::Sender<()>>> = const { RefCell::new(None) };
        static HELD_LEASE: RefCell<Option<TlsLease>> = const { RefCell::new(None) };
    }

    // Installed only on the contender's thread. A failed try_lock establishes
    // that acquisition was actually attempted while another lease held the
    // permit; the lease still uses the production blocking lock afterwards.
    pub(super) fn observe_contention(operation: &Mutex<()>) {
        CONTENTION_PROBE.with(|probe| {
            if let Some(probe) = probe.borrow_mut().take() {
                assert!(matches!(
                    operation.try_lock(),
                    Err(TryLockError::WouldBlock)
                ));
                probe.send(()).unwrap();
            }
        });
    }

    struct TlsLease {
        lease: Option<CheckerLease<'static>>,
        dropped: mpsc::Sender<(bool, bool)>,
    }

    impl Drop for TlsLease {
        fn drop(&mut self) {
            let records_destroyed = ACTIVE_LEASES.try_with(|_| ()).is_err();
            // Catch the old destructor panic so this regression fails an
            // assertion instead of aborting the entire test process.
            let dropped_without_panic =
                catch_unwind(AssertUnwindSafe(|| drop(self.lease.take()))).is_ok();
            self.dropped
                .send((records_destroyed, dropped_without_panic))
                .unwrap();
        }
    }

    #[test]
    fn a_contended_lease_acquires_after_the_held_permit_is_released() {
        let counters = Counters::new();
        let identity = CheckerIdentity::new(Generation::new(&counters), &counters);
        let held = identity.lease().unwrap();
        let (attempted, attempted_rx) = mpsc::channel();
        let (finished, finished_rx) = mpsc::channel();
        let contender = {
            let identity = identity.clone();
            std::thread::spawn(move || {
                CONTENTION_PROBE.with(|probe| *probe.borrow_mut() = Some(attempted));
                let lease = identity.lease();
                finished.send(lease.is_ok()).unwrap();
            })
        };
        attempted_rx.recv().unwrap();
        assert!(matches!(
            finished_rx.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ));
        drop(held);
        assert!(finished_rx.recv().unwrap());
        contender.join().unwrap();
        assert!(identity.lease().is_ok());
    }

    #[test]
    fn a_tls_held_lease_drops_after_the_reentry_records_are_destroyed() {
        static IDENTITY: OnceLock<Arc<CheckerIdentity>> = OnceLock::new();
        let identity = IDENTITY.get_or_init(|| {
            let counters = Counters::new();
            CheckerIdentity::new(Generation::new(&counters), &counters)
        });
        let (dropped, dropped_rx) = mpsc::channel();
        std::thread::spawn(move || {
            // TLS destructors run in reverse initialization order: initialize
            // the holder before acquiring a lease initializes ACTIVE_LEASES.
            HELD_LEASE.with(|holder| assert!(holder.borrow().is_none()));
            let lease = identity.lease().unwrap();
            HELD_LEASE.with(|holder| {
                *holder.borrow_mut() = Some(TlsLease {
                    lease: Some(lease),
                    dropped,
                });
            });
        })
        .join()
        .unwrap();
        let (records_destroyed, dropped_without_panic) = dropped_rx.recv().unwrap();
        assert!(
            records_destroyed,
            "the regression must exercise TLS teardown"
        );
        assert!(
            dropped_without_panic,
            "lease cleanup must tolerate destroyed TLS"
        );
        assert_eq!(identity.generation().validate(), Ok(()));
        assert!(
            identity.lease().is_ok(),
            "thread teardown releases the permit"
        );
    }

    #[test]
    fn identity_adopts_its_symbol_arena_exactly_once() {
        let counters = Counters::new();
        let identity = CheckerIdentity::new(Generation::new(&counters), &counters);
        let mut symbols = identity.adopt_symbol_arena::<u32>(&counters).unwrap();
        assert_eq!(symbols.id(), identity.id());
        let id = symbols.push(7);
        let lease = identity.lease().unwrap();
        assert_eq!(lease.validate_identity(id.arena()), Ok(()));
        assert_eq!(
            lease.check_slot(id.arena(), id.slot(), symbols.len()),
            Ok(0)
        );
        assert_eq!(
            identity.adopt_symbol_arena::<u32>(&counters).err(),
            Some(Error::IdentityAdopted)
        );
        let foreign = SymbolArena::<u32>::new(&counters);
        assert_eq!(
            lease.validate_identity(foreign.id()),
            Err(Error::WrongOwner)
        );
    }

    #[test]
    fn adoption_does_not_outlive_retirement_checks() {
        let counters = Counters::new();
        let generation = Generation::new(&counters);
        let identity = CheckerIdentity::new(generation.clone(), &counters);
        let symbols = identity.adopt_symbol_arena::<u32>(&counters).unwrap();
        generation.retire();
        assert!(matches!(identity.lease(), Err(Error::Retired)));
        assert_eq!(
            symbols.len(),
            0,
            "storage survives retirement; only access is refused"
        );
    }
}
