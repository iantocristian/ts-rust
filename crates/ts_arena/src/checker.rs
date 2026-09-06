//! Checker identity, pool generations and leases (docs/design/ownership.md,
//! sections 2.1 and 2.7).
//!
//! Retirement is distinct from disposal: a pool generation can be retired while
//! leases, retained results and registries keep the checker's storage alive.
//! Every operation therefore carries the *exact* checker identity and its
//! pool-generation token, and lease acquisition and handle resolution validate
//! both. An old registry can never rebind to a replacement checker.
//!
//! This is the identity and validation half of the contract, which is what the
//! S04 E3 scenarios exercise. The generation gate that serializes retirement
//! against result commitment is S09 work and is deliberately absent here rather
//! than half-built.

use std::num::NonZeroU64;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::core_arena::CoreArena;
use crate::counters::OwnerGuard;
use crate::ids::{Exhausted, NodeId};
use crate::node::NodeData;

macro_rules! counted_id {
    ($name:ident, $counter:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        pub struct $name(NonZeroU64);

        static $counter: AtomicU64 = AtomicU64::new(1);

        impl $name {
            fn next() -> Self {
                let value = $counter.fetch_add(1, Ordering::SeqCst);
                Self(NonZeroU64::new(value).expect("counter starts at one"))
            }

            pub fn get(self) -> u64 {
                self.0.get()
            }
        }
    };
}

counted_id!(
    CheckerId,
    CHECKER_IDS,
    "Exact checker identity, distinct from storage identity."
);
counted_id!(
    PoolGeneration,
    POOL_GENERATIONS,
    "A checker pool's generation token. Retiring one is permanent."
);

/// Why an id, lease or handle was refused.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StaleId {
    /// The scope does not hold that arena's owner, so nothing can be resolved
    /// through it. This covers wrong-owner ids and ids of dropped arenas alike.
    UnknownArena,
    /// The arena is held but the slot is past its published bounds.
    OutOfBounds,
    /// The id belongs to a different checker than the one being used, even
    /// though both are active in the same pool generation.
    WrongChecker,
    /// The captured pool generation has been retired.
    RetiredGeneration,
}

impl std::fmt::Display for StaleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            StaleId::UnknownArena => "the scope does not hold that arena's owner",
            StaleId::OutOfBounds => "the slot is outside the arena's published bounds",
            StaleId::WrongChecker => "the id belongs to a different checker",
            StaleId::RetiredGeneration => "the captured pool generation has been retired",
        };
        f.write_str(text)
    }
}

/// A checker's own storage: its AST arena and, in later sprints, its symbols,
/// types and signatures. One exclusive operation permit protects its mutation.
pub struct CheckerOwner {
    id: CheckerId,
    generation: PoolGeneration,
    ast: Mutex<CoreArena>,
    permit: Mutex<()>,
    _owner: OwnerGuard,
}

impl CheckerOwner {
    pub fn id(&self) -> CheckerId {
        self.id
    }

    pub fn generation(&self) -> PoolGeneration {
        self.generation
    }

    pub fn ast_arena_id(&self) -> crate::ids::ArenaId {
        self.ast.lock().expect("checker ast").id()
    }

    /// Allocate into the checker's own AST arena.
    ///
    /// # Errors
    /// [`Exhausted`] when the arena's slots are spent.
    pub fn allocate(&self, node: NodeData) -> Result<NodeId, Exhausted> {
        self.ast.lock().expect("checker ast").allocate(node)
    }

    /// Resolve one of this checker's own nodes.
    pub fn node(&self, id: NodeId) -> Option<NodeData> {
        self.ast.lock().expect("checker ast").resolve(id).cloned()
    }

    pub fn published(&self) -> u32 {
        self.ast.lock().expect("checker ast").published()
    }

    pub fn live_nodes(&self) -> i64 {
        self.ast.lock().expect("checker ast").live_nodes()
    }

    /// Take this checker's exclusive operation permit. Other slots may keep
    /// running and can retire the pool while it is held.
    pub fn permit(&self) -> OperationPermit<'_> {
        OperationPermit {
            _guard: self.permit.lock().expect("checker permit"),
            checker: self.id,
            generation: self.generation,
        }
    }
}

impl std::fmt::Debug for CheckerOwner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CheckerOwner({:?}, {:?})", self.id, self.generation)
    }
}

/// Proof that the holder has the exclusive right to mutate one exact checker in
/// one exact pool generation.
pub struct OperationPermit<'a> {
    _guard: MutexGuard<'a, ()>,
    checker: CheckerId,
    generation: PoolGeneration,
}

impl OperationPermit<'_> {
    pub fn checker(&self) -> CheckerId {
        self.checker
    }

    pub fn generation(&self) -> PoolGeneration {
        self.generation
    }
}

/// A pool of checkers sharing one generation. Retirement is permanent and
/// invalidates operations before storage is disposed.
pub struct CheckerPool {
    state: Mutex<PoolState>,
}

struct PoolState {
    generation: PoolGeneration,
    retired: bool,
}

impl CheckerPool {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(PoolState {
                generation: PoolGeneration::next(),
                retired: false,
            }),
        }
    }

    pub fn generation(&self) -> PoolGeneration {
        self.state.lock().expect("pool").generation
    }

    pub fn is_retired(&self) -> bool {
        self.state.lock().expect("pool").retired
    }

    /// Create a checker in the current generation.
    ///
    /// # Errors
    /// [`Exhausted`] when the process-global arena counter is spent.
    pub fn checker(&self) -> Result<Arc<CheckerOwner>, Exhausted> {
        let generation = {
            let state = self.state.lock().expect("pool");
            assert!(!state.retired, "a retired pool cannot supply new checkers");
            state.generation
        };
        Ok(Arc::new(CheckerOwner {
            id: CheckerId::next(),
            generation,
            ast: Mutex::new(CoreArena::new()?),
            permit: Mutex::new(()),
            _owner: OwnerGuard::new(),
        }))
    }

    /// Retire the current generation. Storage stays alive for as long as leases,
    /// retained results and registries hold it, but nothing from the retired
    /// generation may be used again.
    pub fn retire(&self) {
        self.state.lock().expect("pool").retired = true;
    }

    /// Acquire a lease on a checker of this pool.
    ///
    /// # Errors
    /// [`StaleId::RetiredGeneration`] once the pool has been retired, and
    /// [`StaleId::WrongChecker`] for a checker of another pool generation.
    pub fn lease(&self, checker: &Arc<CheckerOwner>) -> Result<Lease, StaleId> {
        let state = self.state.lock().expect("pool");
        if state.retired {
            return Err(StaleId::RetiredGeneration);
        }
        if checker.generation != state.generation {
            return Err(StaleId::WrongChecker);
        }
        Ok(Lease {
            checker: Arc::clone(checker),
            generation: state.generation,
        })
    }
}

impl Default for CheckerPool {
    fn default() -> Self {
        Self::new()
    }
}

/// A retained right to use one exact checker in one exact pool generation.
///
/// Holding a lease keeps the checker's storage alive; it does not keep the
/// generation valid. Every boundary revalidates.
pub struct Lease {
    checker: Arc<CheckerOwner>,
    generation: PoolGeneration,
}

impl Lease {
    pub fn checker(&self) -> &Arc<CheckerOwner> {
        &self.checker
    }

    /// Revalidate at a boundary: lease use, handle resolution, and resumption
    /// after a callback or reentry.
    ///
    /// # Errors
    /// [`StaleId::RetiredGeneration`] after the captured generation is retired,
    /// and [`StaleId::WrongChecker`] if the pool has moved on.
    pub fn validate(&self, pool: &CheckerPool) -> Result<(), StaleId> {
        let state = pool.state.lock().expect("pool");
        if state.retired || state.generation != self.generation {
            return Err(StaleId::RetiredGeneration);
        }
        if self.checker.generation != self.generation {
            return Err(StaleId::WrongChecker);
        }
        Ok(())
    }
}

impl std::fmt::Debug for Lease {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Lease({:?}, {:?})", self.checker.id, self.generation)
    }
}
