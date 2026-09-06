//! The E3 ownership scenarios that S04 implements, from the scenario tables of
//! docs/design/ownership.md and docs/design/symbols.md. Each scenario returns
//! `Ok(())` or a description of the first violated assertion. The harness binary
//! runs them in release mode and reports the metrics `status/experiments.toml`
//! names; the same functions run under `cargo test`, Miri and AddressSanitizer.

use std::num::NonZeroU32;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use ts_arena::{
    snapshot, Arena, ArenaCounter, BundleOwner, Counters, FileOwner, LazyArena, LazyRef, NodeId,
    Scope, ScratchOwner, StaleId,
};

/// A stand-in node: the harness only needs identity, a parent link and a payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub kind: u16,
    pub parent: Option<NodeId>,
    pub payload: u64,
}

/// The lazy-cache key: `(parent, pos, end)`, as upstream's token cache.
pub type Key = (Option<NodeId>, u32, u32);
pub type File = FileOwner<Node, Key>;
pub type Bundle = BundleOwner<Node, Key>;
pub type ProgramScope = Scope<Node, Key>;

/// Workload sizes; Miri runs the reduced configuration.
#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub lazy_keys: u32,
    pub threads: usize,
}

impl Config {
    pub const fn release() -> Self {
        Self {
            lazy_keys: 1000,
            threads: 8,
        }
    }

    pub fn for_tests() -> Self {
        if cfg!(miri) {
            Self {
                lazy_keys: 300,
                threads: 4,
            }
        } else {
            Self::release()
        }
    }
}

type Outcome = Result<(), String>;

fn node(kind: u16) -> Node {
    Node {
        kind,
        parent: None,
        payload: u64::from(kind),
    }
}

fn file_with(counter: &ArenaCounter, kinds: &[u16]) -> Arc<File> {
    let mut core = Arena::with_counter(counter);
    for &k in kinds {
        core.alloc(node(k));
    }
    Arc::new(File::with_lazy(core, LazyArena::with_counter(counter)))
}

fn global_file(kinds: &[u16]) -> Arc<File> {
    file_with(ArenaCounter::global(), kinds)
}

fn id(file: &File, slot: u32) -> NodeId {
    NodeId::new(
        file.file_id(),
        NonZeroU32::new(slot).expect("slot 0 is reserved"),
    )
}

fn check(condition: bool, what: &str) -> Outcome {
    if condition {
        Ok(())
    } else {
        Err(what.to_string())
    }
}

/// Node and symbol arena and slot exhaustion at injected `u32::MAX` boundaries;
/// identity survives the 2^31 boundary.
pub fn id_exhaustion() -> Outcome {
    // Identity across the 2^31 boundary: no owner-kind bit, no aliasing.
    let counter = ArenaCounter::starting_at(0x7FFF_FFFF);
    let below = file_with(&counter, &[1]);
    let above = file_with(&counter, &[2]);
    check(
        below.file_id().get() == 0x7FFF_FFFF,
        "first arena id at 2^31 - 1",
    )?;
    check(
        above.lazy_arena_id().get() == 0x8000_0002,
        "lazy arena ids continue past 2^31",
    )?;
    let mut scope = ProgramScope::new();
    scope.retain_file(&below);
    scope.retain_file(&above);
    check(
        scope.import(id(&below, 1)).map(|n| n.kind) == Ok(1),
        "resolve below the boundary",
    )?;
    check(
        scope.import(id(&above, 1)).map(|n| n.kind) == Ok(2),
        "resolve above the boundary",
    )?;
    check(
        id(&below, 1).raw() != id(&above, 1).raw(),
        "equal slots in different arenas are different ids",
    )?;
    let lazy_id = above.lazy().get_or_init((None, 0, 0), |a| a.alloc(node(3)));
    check(
        scope.import(lazy_id).map(|n| n.kind) == Ok(3),
        "lazy id above the boundary resolves",
    )?;

    // Arena id exhaustion: no wrap, no reuse, checked before publication.
    let counter = ArenaCounter::starting_at(u32::MAX - 1);
    let a = counter.allocate();
    let b = counter.allocate();
    check(
        a.get() == u32::MAX - 1 && b.get() == u32::MAX,
        "the last two arena ids",
    )?;
    check(
        counter.try_allocate().is_err(),
        "allocation beyond u32::MAX is rejected",
    )?;
    check(counter.try_allocate().is_err(), "rejection is permanent")?;
    check(
        counter.peek_next() == u64::from(u32::MAX) + 1,
        "the counter did not wrap",
    )?;

    // Slot exhaustion in a core arena and in a lazy arena.
    let counter = ArenaCounter::starting_at(1);
    let mut core: Arena<Node> =
        Arena::with_first_slot(&counter, NonZeroU32::new(u32::MAX - 1).unwrap());
    let first = core.alloc(node(1));
    let last = core.alloc(node(2));
    check(
        first.slot() == u32::MAX - 1 && last.slot() == u32::MAX,
        "the last two core slots",
    )?;
    check(
        core.try_alloc(node(3)).is_err(),
        "core slot allocation beyond u32::MAX is rejected",
    )?;
    check(
        core.len() == 2 && core.get(0).is_none(),
        "nothing was published for the rejected slot; slot 0 stays reserved",
    )?;
    let lazy: LazyArena<Node, u8> =
        LazyArena::with_first_slot(&counter, NonZeroU32::new(u32::MAX).unwrap());
    let only = lazy.get_or_init(1, |a| a.alloc(node(1)));
    check(only.slot() == u32::MAX, "the last lazy slot")?;
    let overflow = lazy.try_get_or_init(2, |a| a.try_alloc(node(2)));
    check(
        overflow.is_err(),
        "lazy slot allocation beyond u32::MAX is rejected",
    )?;
    check(
        lazy.published() == 1 && lazy.get(only).is_some(),
        "the rejected initialization published nothing",
    )?;
    Ok(())
}

/// `import` rejects ids of arenas the scope does not hold, in release builds.
pub fn wrong_owner_rejected() -> Outcome {
    let a = global_file(&[1, 2]);
    let b = global_file(&[3]);
    let mut program_a = ProgramScope::new();
    program_a.retain_file(&a);
    let mut program_b = ProgramScope::new();
    program_b.retain_file(&b);
    let a_lazy = a
        .lazy()
        .get_or_init((Some(id(&a, 1)), 0, 1), |alloc| alloc.alloc(node(9)));
    check(
        program_a.import(id(&a, 2)).is_ok() && program_a.import(a_lazy).is_ok(),
        "a program resolves its own file's core and lazy ids",
    )?;
    check(
        matches!(program_b.import(id(&a, 1)), Err(StaleId::WrongOwner { .. })),
        "a core id of a file the program does not hold is rejected",
    )?;
    check(
        matches!(program_b.import(a_lazy), Err(StaleId::WrongOwner { .. })),
        "a lazy id of a file the program does not hold is rejected",
    )?;
    check(
        matches!(program_a.import(id(&b, 1)), Err(StaleId::WrongOwner { .. })),
        "symmetric rejection",
    )?;

    let mut scratch = ScratchOwner::new();
    let scratch_id = scratch.alloc(node(7));
    check(
        matches!(
            program_a.import(scratch_id),
            Err(StaleId::WrongOwner { .. })
        ),
        "scratch storage is never resolvable from a program scope",
    )?;
    let scratch = Arc::new(scratch);
    program_a.retain_scratch(&scratch);
    check(
        program_a.import(scratch_id).map(|n| n.kind) == Ok(7),
        "a retained scratch owner resolves",
    )?;

    let canonical = global_file(&[10]);
    let supplemental = global_file(&[11]);
    let bundle = Bundle::new(canonical, vec![supplemental]);
    let other = Bundle::new(global_file(&[12]), vec![global_file(&[13])]);
    let mut holder = ProgramScope::new();
    holder.retain_bundle(&other);
    let member_id = id(bundle.supplementals()[0].as_ref(), 1);
    check(
        matches!(holder.import(member_id), Err(StaleId::WrongOwner { .. })),
        "a member of a different bundle is rejected",
    )?;

    let branded = program_a.with_core_arena(a.file_id(), |local| local.check(id(&b, 1)).is_none());
    check(
        branded == Some(true),
        "the branded check refuses a foreign id",
    )?;
    Ok(())
}

/// A dropped arena's ids fail everywhere; a later arena never reuses its id.
pub fn stale_and_recycled_ids_rejected() -> Outcome {
    let old = global_file(&[1]);
    let old_id = id(&old, 1);
    let old_arena = old.file_id();
    let old_lazy = old.lazy().get_or_init((None, 0, 0), |a| a.alloc(node(2)));
    let mut scope = ProgramScope::new();
    scope.retain_file(&old);
    check(
        scope.import(old_id).is_ok() && scope.import(old_lazy).is_ok(),
        "live ids resolve",
    )?;
    scope.release_file(&old);
    drop(old);
    check(
        matches!(scope.import(old_id), Err(StaleId::WrongOwner { .. })),
        "a released file's ids are stale in the scope",
    )?;

    let new = global_file(&[1]);
    check(
        new.file_id() > old_arena && new.lazy_arena_id() > old_arena,
        "arena ids are never reused",
    )?;
    scope.retain_file(&new);
    check(
        matches!(scope.import(old_id), Err(StaleId::WrongOwner { .. })),
        "the stale id is still rejected after a replacement file exists",
    )?;
    check(
        matches!(scope.import(old_lazy), Err(StaleId::WrongOwner { .. })),
        "the stale lazy id is still rejected",
    )?;
    check(
        id(&new, 1).raw() != old_id.raw(),
        "the same numeric slot in the new arena is a different id",
    )?;
    check(
        scope.import(id(&new, 1)).is_ok(),
        "the new file's ids resolve",
    )?;
    let unpublished = NodeId::new(new.file_id(), NonZeroU32::new(5).unwrap());
    check(
        matches!(
            scope.import(unpublished),
            Err(StaleId::UnpublishedSlot { .. })
        ),
        "an unpublished slot of a held arena is rejected",
    )?;
    Ok(())
}

/// Concurrent first-use JSDoc and token requests with page growth.
pub fn concurrent_lazy_storage(cfg: Config) -> Outcome {
    let file = global_file(&[1, 2, 3]);
    let parent = id(&file, 2);
    let keys = cfg.lazy_keys;
    let inits: Arc<Vec<AtomicUsize>> = Arc::new((0..keys).map(|_| AtomicUsize::new(0)).collect());
    let mut handles = Vec::new();
    for t in 0..cfg.threads {
        let file = Arc::clone(&file);
        let inits = Arc::clone(&inits);
        handles.push(std::thread::spawn(move || {
            let mut ids = Vec::with_capacity(keys as usize);
            let mut early: Vec<(u32, LazyRef<Node>, usize)> = Vec::new();
            for step in 0..keys {
                let key = (step + t as u32 * 37) % keys;
                let node_id = file
                    .lazy()
                    .get_or_init((Some(parent), key, key + 1), |alloc| {
                        inits[key as usize].fetch_add(1, Ordering::SeqCst);
                        alloc.alloc(Node {
                            kind: key as u16,
                            parent: Some(parent),
                            payload: u64::from(key),
                        })
                    });
                ids.push((key, node_id));
                if early.len() < 8 {
                    let r = file.lazy().get(node_id).expect("published");
                    let address = r.address();
                    early.push((key, r, address));
                }
            }
            (ids, early)
        }));
    }
    let mut results = Vec::new();
    for h in handles {
        results.push(h.join().map_err(|_| "a worker panicked".to_string())?);
    }
    for (key, count) in inits.iter().enumerate() {
        check(
            count.load(Ordering::SeqCst) == 1,
            &format!(
                "key {key} was initialized {} times",
                count.load(Ordering::SeqCst)
            ),
        )?;
    }
    let mut by_key = vec![None; keys as usize];
    for (ids, early) in &results {
        for &(key, node_id) in ids {
            match by_key[key as usize] {
                None => by_key[key as usize] = Some(node_id),
                Some(seen) => check(seen == node_id, "every thread sees the same id for a key")?,
            }
            let n = file.lazy().get(node_id).ok_or("published id resolves")?;
            check(
                n.kind == key as u16 && n.parent == Some(parent),
                "node content and supplied parent agree with the key",
            )?;
        }
        for (key, r, address) in early {
            check(
                r.kind == *key as u16,
                "an early reference still reads its node after page growth",
            )?;
            check(
                r.address() == *address,
                "the node address is stable across directory growth",
            )?;
        }
    }
    let expected_pages = (keys as usize).div_ceil(ts_arena::PAGE_SIZE);
    check(
        file.lazy().page_count() == expected_pages,
        "page growth matches the key count",
    )?;
    check(
        file.lazy().published() == keys,
        "exactly one published slot per key",
    )?;
    Ok(())
}

/// A mapper with three supplemental files, holders released in every order.
pub fn mapper_bundle_disposal() -> Outcome {
    let orders = permutations(4);
    for order in &orders {
        let baseline = snapshot();
        let canonical = global_file(&[1]);
        let supplementals = vec![global_file(&[2]), global_file(&[3]), global_file(&[4])];
        let member_ids: Vec<NodeId> = std::iter::once(&canonical)
            .chain(supplementals.iter())
            .map(|f| id(f, 1))
            .collect();
        let bundle = Bundle::new(canonical, supplementals);
        let links = bundle
            .canonical()
            .bundle_links()
            .ok_or("the canonical file records its links")?
            .clone();
        check(
            links.canonical == bundle.canonical().file_id() && links.supplementals.len() == 3,
            "canonical links",
        )?;
        for s in bundle.supplementals() {
            check(
                s.bundle_links().map(|l| l.canonical) == Some(links.canonical),
                "each supplemental links back to the canonical file",
            )?;
        }
        let mut holders: Vec<Holder> = vec![
            Holder::Plain(Arc::clone(&bundle)),
            Holder::Plain(Arc::clone(&bundle)),
        ];
        for _ in 0..2 {
            let mut scope = ProgramScope::new();
            scope.retain_bundle(&bundle);
            holders.push(Holder::Scope(scope));
        }
        drop(bundle);
        let mut remaining: Vec<Option<Holder>> = holders.into_iter().map(Some).collect();
        for (i, &index) in order.iter().enumerate() {
            let live = remaining.iter().flatten().count();
            check(live == 4 - i, "holder count before a drop")?;
            for holder in remaining.iter().flatten() {
                match holder {
                    Holder::Scope(scope) => {
                        for &m in &member_ids {
                            check(
                                scope.import(m).is_ok(),
                                "every member resolves while any holder lives",
                            )?;
                        }
                    }
                    Holder::Plain(b) => check(
                        b.members().all(|f| f.bundle_links().is_some()),
                        "a plain holder still sees the links",
                    )?,
                }
            }
            check(
                snapshot() != baseline,
                "storage is alive while holders remain",
            )?;
            remaining[index] = None;
        }
        check(
            snapshot() == baseline,
            &format!("counters return to baseline after drop order {order:?}"),
        )?;
    }
    Ok(())
}

enum Holder {
    Plain(Arc<Bundle>),
    Scope(ProgramScope),
}

fn permutations(n: usize) -> Vec<Vec<usize>> {
    fn go(prefix: &mut Vec<usize>, used: &mut Vec<bool>, out: &mut Vec<Vec<usize>>) {
        if prefix.len() == used.len() {
            out.push(prefix.clone());
            return;
        }
        for i in 0..used.len() {
            if !used[i] {
                used[i] = true;
                prefix.push(i);
                go(prefix, used, out);
                prefix.pop();
                used[i] = false;
            }
        }
    }
    let mut out = Vec::new();
    go(&mut Vec::new(), &mut vec![false; n], &mut out);
    out
}

/// The outcome of every scenario plus the counter deviations around them.
#[derive(Debug)]
pub struct Report {
    pub id_exhaustion: Outcome,
    pub wrong_owner_rejected: Outcome,
    pub stale_and_recycled_ids_rejected: Outcome,
    pub concurrent_lazy_storage: Outcome,
    pub mapper_bundle_disposal: Outcome,
    pub live_owner_delta: i64,
    pub live_allocation_delta: i64,
}

fn around(deltas: &mut (i64, i64), scenario: impl FnOnce() -> Outcome) -> Outcome {
    let before = snapshot();
    let outcome = scenario();
    let after: Counters = snapshot();
    deltas.0 = deltas.0.max((after.owners - before.owners).abs());
    deltas.1 = deltas.1.max((after.allocations - before.allocations).abs());
    outcome
}

/// Runs every scenario, measuring the counter deviation after each final drop.
pub fn run_all(cfg: Config) -> Report {
    let mut deltas = (0i64, 0i64);
    let id_exhaustion = around(&mut deltas, id_exhaustion);
    let wrong_owner_rejected = around(&mut deltas, wrong_owner_rejected);
    let stale_and_recycled_ids_rejected = around(&mut deltas, stale_and_recycled_ids_rejected);
    let concurrent_lazy_storage = around(&mut deltas, || concurrent_lazy_storage(cfg));
    let mapper_bundle_disposal = around(&mut deltas, mapper_bundle_disposal);
    Report {
        id_exhaustion,
        wrong_owner_rejected,
        stale_and_recycled_ids_rejected,
        concurrent_lazy_storage,
        mapper_bundle_disposal,
        live_owner_delta: deltas.0,
        live_allocation_delta: deltas.1,
    }
}
