use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;

use ts_arena::{
    live_allocation_count, live_owner_count, ArenaIdAllocator, ArenaLease, BundleLink, BundleOwner,
    Exhausted, FileOwner, LazyArena, NodeArena, NodeId, OwnerKind, ResolveError, SymbolArena,
    ValidatedScope,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Metrics {
    pub id_exhaustion: bool,
    pub wrong_owner_rejected: bool,
    pub stale_and_recycled_ids_rejected: bool,
    pub concurrent_lazy_storage: bool,
    pub mapper_bundle_disposal: bool,
    pub live_owner_delta: usize,
    pub live_allocation_delta: usize,
}

pub fn run_scenarios() -> Metrics {
    let mut live_owner_delta = 0;
    let mut live_allocation_delta = 0;
    for scenario in [
        id_exhaustion as fn(),
        wrong_owner_rejected,
        stale_and_recycled_ids_rejected,
        concurrent_lazy_storage,
        mapper_bundle_disposal,
    ] {
        let (owner_delta, allocation_delta) = measure_final_drop(scenario);
        live_owner_delta = live_owner_delta.max(owner_delta);
        live_allocation_delta = live_allocation_delta.max(allocation_delta);
    }

    Metrics {
        id_exhaustion: true,
        wrong_owner_rejected: true,
        stale_and_recycled_ids_rejected: true,
        concurrent_lazy_storage: true,
        mapper_bundle_disposal: true,
        live_owner_delta,
        live_allocation_delta,
    }
}

fn measure_final_drop(scenario: fn()) -> (usize, usize) {
    let owner_baseline = live_owner_count();
    let allocation_baseline = live_allocation_count();
    scenario();
    (
        live_owner_count().abs_diff(owner_baseline),
        live_allocation_count().abs_diff(allocation_baseline),
    )
}

fn id_exhaustion() {
    assert_eq!(
        std::mem::size_of::<Option<ts_arena::NodeId>>(),
        std::mem::size_of::<u64>()
    );
    assert_eq!(
        std::mem::size_of::<Option<ts_arena::SymbolId>>(),
        std::mem::size_of::<u64>()
    );

    let node_ids = ArenaIdAllocator::with_next_for_test(u64::from(u32::MAX));
    let last = NodeArena::<u8>::new_in(&node_ids, OwnerKind::FileCore).unwrap();
    assert_eq!(last.id().get(), u32::MAX);
    assert!(matches!(
        NodeArena::<u8>::new_in(&node_ids, OwnerKind::FileCore),
        Err(Exhausted::ArenaIds)
    ));

    let symbol_ids = ArenaIdAllocator::with_next_for_test(u64::from(u32::MAX));
    let last = SymbolArena::<u8>::new_in(&symbol_ids, OwnerKind::Symbol).unwrap();
    assert_eq!(last.id().get(), u32::MAX);
    assert!(matches!(
        SymbolArena::<u8>::new_in(&symbol_ids, OwnerKind::Symbol),
        Err(Exhausted::ArenaIds)
    ));

    let slot_ids = ArenaIdAllocator::with_next_for_test(1);
    let nodes =
        NodeArena::with_next_slot_for_test(&slot_ids, OwnerKind::FileCore, u64::from(u32::MAX))
            .unwrap();
    let node = nodes.allocate(1).unwrap();
    assert_eq!(node.slot().get(), u32::MAX);
    assert_eq!(*nodes.resolve(node).unwrap(), 1);
    assert_eq!(
        nodes.allocate(2),
        Err(Exhausted::Slots { arena: nodes.id() })
    );

    let slot_ids = ArenaIdAllocator::with_next_for_test(1);
    let symbols =
        SymbolArena::with_next_slot_for_test(&slot_ids, OwnerKind::Symbol, u64::from(u32::MAX))
            .unwrap();
    let symbol = symbols.allocate(1).unwrap();
    assert_eq!(symbol.slot().get(), u32::MAX);
    assert!(matches!(symbols.allocate(2), Err(Exhausted::Slots { .. })));

    let slot_ids = ArenaIdAllocator::with_next_for_test(1);
    let lazy = LazyArena::with_next_slot_for_test(&slot_ids, u64::from(u32::MAX)).unwrap();
    let (id, value, inserted) = lazy.get_or_insert_with("last", || 42).unwrap();
    assert!(inserted);
    assert_eq!(id.slot().get(), u32::MAX);
    assert_eq!(*value, 42);
    assert_eq!(*lazy.resolve(id).unwrap(), 42);
    assert!(matches!(
        lazy.get_or_insert_with("overflow", || 43),
        Err(ResolveError::Exhausted(Exhausted::Slots { .. }))
    ));
}

fn wrong_owner_rejected() {
    let file = NodeArena::new(OwnerKind::FileCore).unwrap();
    let other_file = NodeArena::new(OwnerKind::FileCore).unwrap();
    let checker = NodeArena::new(OwnerKind::Checker).unwrap();
    let file_node = file.allocate("file").unwrap();
    let other_node = other_file.allocate("other").unwrap();
    let checker_node = checker.allocate("checker").unwrap();
    assert_eq!(file_node.slot(), other_node.slot());
    assert_eq!(file_node.slot(), checker_node.slot());

    let mut scope = ValidatedScope::allowing([OwnerKind::FileCore, OwnerKind::FileLazy]);
    scope.retain(Arc::clone(&file)).unwrap();
    assert_eq!(*scope.import(file_node).unwrap(), "file");
    assert!(matches!(
        scope.import(other_node),
        Err(ResolveError::UnknownArena(_))
    ));
    assert!(matches!(
        scope.retain(checker),
        Err(ResolveError::WrongOwnerKind { .. })
    ));

    file.with_scope(|local| {
        let checked = local.validate(file_node).unwrap();
        assert_eq!(*local.resolve(checked).unwrap(), "file");
    });
}

fn stale_and_recycled_ids_rejected() {
    let allocator = ArenaIdAllocator::with_next_for_test(7);
    let stale_id = {
        let arena = NodeArena::new_in(&allocator, OwnerKind::Scratch).unwrap();
        let id = arena.allocate(42).unwrap();
        let lease = ArenaLease::new(Arc::clone(&arena));
        drop(arena);
        assert_eq!(*lease.resolve(id).unwrap(), 42);
        drop(lease);
        id
    };

    let replacement = NodeArena::new_in(&allocator, OwnerKind::Scratch).unwrap();
    let replacement_id = replacement.allocate(42).unwrap();
    assert_ne!(stale_id.arena(), replacement_id.arena());
    assert_ne!(stale_id, replacement_id);

    let mut replacement_scope = ValidatedScope::<NodeId, i32>::new();
    replacement_scope.retain(replacement).unwrap();
    assert!(matches!(
        replacement_scope.import(stale_id),
        Err(ResolveError::UnknownArena(_))
    ));
}

#[derive(Debug)]
struct LazyNode {
    key: u32,
    parent: u32,
    initialized: bool,
}

fn concurrent_lazy_storage() {
    let arena = LazyArena::<u32, LazyNode>::new().unwrap();
    for key in 0..256 {
        let (_, node, inserted) = arena
            .get_or_insert_with(key, || LazyNode {
                key,
                parent: key.saturating_sub(1),
                initialized: true,
            })
            .unwrap();
        assert!(inserted);
        assert!(node.initialized);
    }
    let retained = arena
        .resolve(arena.get_or_insert_with(0, || unreachable!()).unwrap().0)
        .unwrap();

    let workers = 16;
    let barrier = Arc::new(Barrier::new(workers));
    let creations = Arc::new(AtomicUsize::new(0));
    let mut threads = Vec::new();
    for _ in 0..workers {
        let arena = Arc::clone(&arena);
        let barrier = Arc::clone(&barrier);
        let creations = Arc::clone(&creations);
        threads.push(thread::spawn(move || {
            barrier.wait();
            arena
                .get_or_insert_with(999, || {
                    creations.fetch_add(1, Ordering::AcqRel);
                    LazyNode {
                        key: 999,
                        parent: 41,
                        initialized: true,
                    }
                })
                .unwrap()
        }));
    }
    let results: Vec<_> = threads
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .collect();
    let expected = results[0].0;
    for (id, node, _) in &results {
        assert_eq!(*id, expected);
        assert_eq!(node.key, 999);
        assert_eq!(node.parent, 41);
        assert!(node.initialized);
    }
    assert_eq!(creations.load(Ordering::Acquire), 1);
    assert_eq!(arena.published_len(), 257);
    assert_eq!(arena.page_count(), 2);

    for key in 1_000..1_300 {
        arena
            .get_or_insert_with(key, || LazyNode {
                key,
                parent: 999,
                initialized: true,
            })
            .unwrap();
    }
    assert!(arena.page_count() >= 3);
    assert_eq!(retained.key, 0);
    assert!(retained.initialized);
}

type TestFile = FileOwner<String, u32, String>;

fn mapper_bundle_disposal() {
    for release_order in [[0, 1, 2, 3], [3, 2, 1, 0], [1, 3, 0, 2], [2, 0, 3, 1]] {
        exercise_bundle_order(release_order);
    }
}

fn exercise_bundle_order(release_order: [usize; 4]) {
    let canonical = TestFile::new().unwrap();
    let supplementals = [
        TestFile::new().unwrap(),
        TestFile::new().unwrap(),
        TestFile::new().unwrap(),
    ];
    let mut files = vec![Arc::clone(&canonical)];
    files.extend(supplementals.iter().cloned());
    let ids: Vec<_> = files.iter().map(|file| file.id()).collect();
    let node_ids: Vec<_> = files
        .iter()
        .enumerate()
        .map(|(index, file)| file.core().allocate(format!("file-{index}")).unwrap())
        .collect();
    let weak_files: Vec<_> = files.iter().map(Arc::downgrade).collect();

    let bundle = BundleOwner::new(canonical, supplementals.into_iter().collect());
    let weak_bundle = Arc::downgrade(&bundle);
    assert_eq!(bundle.len(), 4);
    assert_eq!(
        bundle.link(ids[0]),
        Some(&BundleLink::Canonical {
            supplementals: ids[1..].to_vec()
        })
    );
    for id in &ids[1..] {
        assert_eq!(
            bundle.link(*id),
            Some(&BundleLink::Supplemental { canonical: ids[0] })
        );
    }

    let mut handles: Vec<_> = ids
        .iter()
        .map(|id| Some(bundle.handle(*id).unwrap()))
        .collect();
    drop(files);
    drop(bundle);
    for &index in &release_order[..3] {
        drop(handles[index].take());
        assert!(weak_bundle.upgrade().is_some());
        for file in &weak_files {
            assert!(file.upgrade().is_some());
        }
    }

    let survivor_index = release_order[3];
    let survivor = handles[survivor_index].as_ref().unwrap();
    for (index, id) in ids.iter().enumerate() {
        let sibling = survivor.sibling(*id).unwrap();
        assert_eq!(
            *sibling.owner().core().resolve(node_ids[index]).unwrap(),
            format!("file-{index}")
        );
    }
    drop(handles[survivor_index].take());
    assert!(weak_bundle.upgrade().is_none());
    for file in weak_files {
        assert!(file.upgrade().is_none());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_scenarios() {
        let metrics = run_scenarios();
        assert_eq!(metrics.live_owner_delta, 0);
        assert_eq!(metrics.live_allocation_delta, 0);
    }
}
