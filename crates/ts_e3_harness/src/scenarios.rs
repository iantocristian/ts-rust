//! The E3 ownership scenarios implemented in S04.
//!
//! Each scenario is a row of the table in docs/design/ownership.md, section 4,
//! and settles one criterion of `status/experiments.toml`. Rows this sprint does
//! not build are absent rather than stubbed, so their criteria stay pending
//! instead of being reported as passing.
//!
//! Every scenario runs between a live-owner and live-allocation baseline; the
//! deltas after its final drop are what the two counter criteria measure.

use std::collections::BTreeSet;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};

use ts_arena::checker::StaleId;
use ts_arena::ids::IdCounter;
use ts_arena::{
    live_allocations, live_owners, with_scoped_arena, ArenaId, BundleOwner, CheckerPool, CoreArena,
    FileOwner, FileOwnerBuilder, LazyArena, LazyError, NodeData, NodeFlags, NodeId, NodeKind,
    Scope, ScratchOwner, Slot, SlotCounter, TokenKey, MAX_ID, PAGE_SIZE,
};
use ts_jsstring::{JsString, SourceText};

/// One scenario's outcome, with the assertions it actually checked.
pub struct Outcome {
    pub name: &'static str,
    pub passed: bool,
    pub checks: Vec<(String, bool)>,
    pub owner_delta: i64,
    pub allocation_delta: i64,
}

/// Collects assertions so a failure names the check that failed.
#[derive(Default)]
pub struct Checks {
    entries: Vec<(String, bool)>,
}

impl Checks {
    pub fn require(&mut self, what: &str, ok: bool) {
        self.entries.push((what.to_string(), ok));
    }

    fn passed(&self) -> bool {
        !self.entries.is_empty() && self.entries.iter().all(|(_, ok)| *ok)
    }
}

/// How large the concurrent scenario runs.
///
/// Miri interprets every instruction, so the full workload would take hours
/// there for no extra coverage: the reduced workload still runs several threads
/// through the same miss, recheck, publish and page-growth paths, and the
/// harness asserts that it still crosses a page boundary. Set `TS_E3_SCALE` to
/// `reduced` for that run; every other run uses the full workload.
pub fn reduced_scale() -> bool {
    std::env::var("TS_E3_SCALE").is_ok_and(|value| value == "reduced")
}

/// Several probes are specified to fail loudly. Their messages are not the
/// result, so the default hook is silenced while the scenarios run.
type PanicHook = Box<dyn Fn(&std::panic::PanicHookInfo<'_>) + Sync + Send>;

pub struct QuietPanics(Option<PanicHook>);

impl QuietPanics {
    pub fn install() -> Self {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        Self(Some(previous))
    }
}

impl Drop for QuietPanics {
    fn drop(&mut self) {
        if let Some(previous) = self.0.take() {
            std::panic::set_hook(previous);
        }
    }
}

fn run(name: &'static str, body: impl FnOnce(&mut Checks)) -> Outcome {
    let owners_before = live_owners();
    let allocations_before = live_allocations();
    let mut checks = Checks::default();
    let result = catch_unwind(AssertUnwindSafe(|| body(&mut checks)));
    if result.is_err() {
        checks.require("the scenario itself did not panic", false);
    }
    Outcome {
        name,
        passed: checks.passed(),
        checks: checks.entries,
        owner_delta: live_owners() - owners_before,
        allocation_delta: live_allocations() - allocations_before,
    }
}

/// Whether a checked access was refused for the expected reason. `NodeRef` and
/// `Lease` are values, not comparable results, so the reason is compared alone.
fn refused<T>(result: Result<T, StaleId>, expected: StaleId) -> bool {
    result.err() == Some(expected)
}

const KIND_SOURCE_FILE: NodeKind = NodeKind(1);
const KIND_IDENTIFIER: NodeKind = NodeKind(2);
const KIND_STRING_LITERAL: NodeKind = NodeKind(3);
const KIND_JSDOC: NodeKind = NodeKind(4);

/// Slot 1, the first slot of any arena.
fn first_slot() -> Slot {
    Slot::from_raw(1).expect("slot 1 is nonzero")
}

fn file(text: &str, nodes: u32) -> Arc<FileOwner> {
    let mut builder =
        FileOwnerBuilder::new(SourceText::decode(text.as_bytes())).expect("arena ids available");
    builder
        .allocate(NodeData::new(KIND_SOURCE_FILE, 0, text.len() as u32))
        .expect("slot");
    for index in 1..nodes {
        builder
            .allocate(
                NodeData::new(KIND_IDENTIFIER, index, index + 1)
                    .with_text(JsString::from_text("x")),
            )
            .expect("slot");
    }
    builder.finish().expect("arena ids available")
}

/// Node and symbol arena and slot exhaustion, at injected boundaries.
pub fn id_exhaustion() -> Outcome {
    run("id_exhaustion", |checks| {
        // Arena 0 and slot 0 are reserved and unrepresentable, so no allocation
        // path can publish a zero slot or a zero arena.
        checks.require("arena 0 is unrepresentable", ArenaId::from_raw(0).is_none());
        checks.require("slot 0 is unrepresentable", Slot::from_raw(0).is_none());

        // Identity survives the whole 32-bit space, including the 31-bit
        // boundary a signed counter or a stolen bit would corrupt.
        let mut seen = BTreeSet::new();
        let boundaries = [1u32, 0x7FFF_FFFF, 0x8000_0000, MAX_ID];
        let mut round_trips = true;
        for arena in boundaries {
            for slot in boundaries {
                let id = NodeId::new(
                    ArenaId::from_raw(arena).expect("nonzero"),
                    Slot::from_raw(slot).expect("nonzero"),
                );
                round_trips &= id.arena().get() == arena && id.slot().get() == slot;
                seen.insert(id.raw());
            }
        }
        checks.require("ids round-trip across the full 32-bit range", round_trips);
        checks.require(
            "distinct arena and slot pairs stay distinct",
            seen.len() == boundaries.len() * boundaries.len(),
        );

        // The arena counter is the production one, exercised through an injected
        // start rather than an assumed session lifetime.
        let arenas = IdCounter::starting_at("arena", MAX_ID);
        checks.require(
            "the last arena id is issued",
            arenas.allocate() == Ok(MAX_ID),
        );
        checks.require("the next arena id is refused", arenas.allocate().is_err());
        checks.require(
            "a refused arena counter stays refused",
            arenas.allocate().is_err(),
        );
        checks.require(
            "the arena counter saturates one past the limit instead of wrapping",
            arenas.peek() == u64::from(MAX_ID) + 1,
        );

        // Slot allocation applies the same full 32-bit limit.
        let slots = SlotCounter::starting_at(MAX_ID);
        checks.require("the last slot is issued", slots.allocate().is_ok());
        checks.require("the next slot is refused", slots.allocate().is_err());
        checks.require(
            "a refused slot counter stays refused",
            slots.allocate().is_err(),
        );
        checks.require(
            "the slot counter saturates instead of wrapping",
            slots.issued() == u64::from(MAX_ID),
        );

        // A slot counter that has run past the arena's contents refuses
        // publication rather than truncating or reusing: a node published under
        // a slot no lookup can index would be reachable only by accident.
        let boundary_arena = ArenaId::from_raw(0x8000_0000).expect("nonzero");
        let mut core = CoreArena::with_ids(boundary_arena, SlotCounter::starting_at(MAX_ID));
        checks.require(
            "the core arena refuses a slot no lookup could index",
            core.allocate(NodeData::new(KIND_IDENTIFIER, 0, 1)).is_err(),
        );
        let mut core = CoreArena::with_ids(boundary_arena, SlotCounter::new());
        let node = core
            .allocate(NodeData::new(KIND_IDENTIFIER, 0, 1))
            .expect("slot");
        checks.require(
            "an arena above the 31-bit boundary still publishes a resolvable node",
            core.resolve(node).is_some() && node.arena() == boundary_arena,
        );

        let lazy = LazyArena::with_ids(
            ArenaId::from_raw(0x7FFF_FFFF).expect("nonzero"),
            SlotCounter::starting_at(MAX_ID),
        );
        let refused = lazy.try_get_or_create_token(
            TokenKey {
                parent: NodeId::new(boundary_arena, first_slot()),
                pos: 0,
                end: 1,
            },
            KIND_IDENTIFIER,
            NodeFlags::NONE,
            None,
        );
        checks.require(
            "lazy publication is refused before an unindexable slot is used",
            matches!(refused, Err(LazyError::Exhausted(_))),
        );
        checks.require("nothing was published", lazy.published() == 0);

        // Owner kind is resolver metadata, not a stolen arena bit: it stays
        // distinct for arenas anywhere in the range.
        let owner = file("const a = 1;", 2);
        let scratch = ScratchOwner::new().expect("arena").into_shared();
        let mut scope = Scope::new();
        scope.add_file(&owner);
        scope.add_scratch(&scratch);
        checks.require(
            "each retained arena reports its own owner kind",
            scope.owner_kind(owner.core().id()) == Some(ts_arena::OwnerKind::FileCore)
                && scope.owner_kind(owner.lazy().id()) == Some(ts_arena::OwnerKind::FileLazy)
                && scope.owner_kind(scratch.id()) == Some(ts_arena::OwnerKind::Scratch),
        );
        checks.require(
            "an arena at the top of the range is not silently held",
            scope
                .owner_kind(ArenaId::from_raw(MAX_ID).expect("nonzero"))
                .is_none(),
        );
    })
}

/// Wrong-file and wrong-checker handles, including equal numeric slots in two
/// active checkers of one pool generation.
pub fn wrong_owner_rejected() -> Outcome {
    run("wrong_owner_rejected", |checks| {
        let one = file("const a = 1;", 4);
        let two = file("const b = 2;", 4);
        let mut scope = Scope::new();
        scope.add_file(&one);

        let own = NodeId::new(one.core().id(), first_slot());
        let other = NodeId::new(two.core().id(), first_slot());
        checks.require(
            "the two files hold equal numeric slots in different arenas",
            own.slot() == other.slot() && own.arena() != other.arena(),
        );
        checks.require(
            "a file scope resolves its own id",
            scope.import(own).is_ok(),
        );
        checks.require(
            "a file scope rejects another file's id",
            refused(scope.import(other), StaleId::UnknownArena),
        );
        checks.require(
            "the lazy arena of a held file is held as well",
            scope.owner_kind(one.lazy().id()).is_some()
                && scope.owner_kind(two.lazy().id()).is_none(),
        );

        // A slot past the published bounds is rejected, not aliased.
        let past_end = NodeId::new(
            one.core().id(),
            Slot::from_raw(one.core().published() + 1).expect("nonzero"),
        );
        checks.require(
            "a slot past the published bounds is rejected",
            refused(scope.import(past_end), StaleId::OutOfBounds),
        );

        // Two active checkers in one pool generation, with equal numeric slots.
        let pool = CheckerPool::new();
        let left = pool.checker().expect("arena");
        let right = pool.checker().expect("arena");
        let left_node = left
            .allocate(NodeData::new(KIND_STRING_LITERAL, 0, 1))
            .expect("slot");
        let right_node = right
            .allocate(NodeData::new(KIND_STRING_LITERAL, 0, 1))
            .expect("slot");
        checks.require(
            "the two checkers hold equal numeric slots",
            left_node.slot() == right_node.slot() && left_node.arena() != right_node.arena(),
        );
        checks.require(
            "both checkers are in one pool generation",
            left.generation() == right.generation() && left.id() != right.id(),
        );

        let mut checker_scope = Scope::new();
        checker_scope.add_checker(&left);
        checker_scope.add_checker(&right);
        {
            let permit = left.permit();
            checks.require(
                "a checker resolves its own node under its own permit",
                checker_scope.import_with_permit(left_node, &permit).is_ok(),
            );
            checks.require(
                "that permit cannot reach the other checker's equal slot",
                refused(
                    checker_scope.import_with_permit(right_node, &permit),
                    StaleId::WrongChecker,
                ),
            );
        }
        checks.require(
            "checker storage is unreachable without a permit",
            refused(checker_scope.import(left_node), StaleId::WrongChecker),
        );
        checks.require(
            "a file-only scope cannot resolve a checker-created node",
            refused(scope.import(left_node), StaleId::UnknownArena),
        );

        // Request scratch is its own owner and is not reachable from the file
        // scope that did not retain it.
        let mut scratch = ScratchOwner::new().expect("arena");
        let scratch_node = scratch
            .allocate(NodeData::new(KIND_IDENTIFIER, 0, 1))
            .expect("slot");
        let scratch = scratch.into_shared();
        checks.require(
            "scratch ids are rejected by a scope that does not hold them",
            refused(scope.import(scratch_node), StaleId::UnknownArena),
        );
        let mut request = Scope::new();
        request.add_scratch(&scratch);
        checks.require(
            "the request scope resolves its own scratch",
            request.import(scratch_node).is_ok(),
        );
    })
}

/// Stale and recycled owner and generation identities.
pub fn stale_and_recycled_ids_rejected() -> Outcome {
    run("stale_and_recycled_ids_rejected", |checks| {
        let dropped = file("const a = 1;", 3);
        let dropped_arena = dropped.core().id();
        let stale = NodeId::new(dropped_arena, first_slot());
        let mut scope = Scope::new();
        scope.add_file(&dropped);
        checks.require(
            "the id resolves while its owner lives",
            scope.import(stale).is_ok(),
        );
        drop(scope);
        drop(dropped);

        // Because arena ids are never reused, nothing can occupy the dropped
        // arena's numbers, so there is no ABA window to test against.
        let replacement = file("const a = 1;", 3);
        checks.require(
            "a replacement file takes a fresh arena id",
            replacement.core().id() != dropped_arena,
        );
        let mut scope = Scope::new();
        scope.add_file(&replacement);
        checks.require(
            "the stale id is rejected by a scope holding the replacement",
            refused(scope.import(stale), StaleId::UnknownArena),
        );
        checks.require(
            "the stale id is rejected by an empty scope too",
            refused(Scope::new().import(stale), StaleId::UnknownArena),
        );
        checks.require(
            "the replacement's own id resolves",
            scope
                .import(NodeId::new(replacement.core().id(), first_slot()))
                .is_ok(),
        );

        // A retired pool generation invalidates its leases at the next boundary
        // while its storage is still held.
        let pool = CheckerPool::new();
        let checker = pool.checker().expect("arena");
        let node = checker
            .allocate(NodeData::new(KIND_STRING_LITERAL, 0, 1))
            .expect("slot");
        let lease = pool.lease(&checker).expect("a live generation leases");
        checks.require("a live lease validates", lease.validate(&pool).is_ok());
        pool.retire();
        checks.require(
            "the retired generation's lease fails at the next boundary",
            refused(lease.validate(&pool), StaleId::RetiredGeneration),
        );
        checks.require(
            "a retired generation supplies no new leases",
            refused(pool.lease(&checker), StaleId::RetiredGeneration),
        );
        checks.require(
            "retirement did not dispose the storage the lease still holds",
            checker.node(node).is_some(),
        );

        // A checker of a retired generation cannot be used through a scope that
        // still retains it, even with its own permit.
        let mut checker_scope = Scope::new();
        checker_scope.add_checker(&checker);
        let replacement_pool = CheckerPool::new();
        let fresh = replacement_pool.checker().expect("arena");
        checks.require(
            "a replacement checker takes a different identity and generation",
            fresh.id() != checker.id() && fresh.generation() != checker.generation(),
        );
        checks.require(
            "an old registry cannot rebind to the replacement checker",
            checker_scope.owner_kind(fresh.ast_arena_id()).is_none(),
        );
    })
}

/// Concurrent first-use JSDoc and token requests, including page growth.
pub fn concurrent_lazy_storage() -> Outcome {
    run("concurrent_lazy_storage", |checks| {
        // Enough keys to grow the page directory several times over.
        const JSDOC_OWNERS: u32 = 2;
        let (keys, threads): (u32, usize) = if reduced_scale() { (300, 3) } else { (900, 8) };
        let keys_usize = keys as usize;

        let owner = file("const a = 1;", 3);
        let parent = NodeId::new(owner.core().id(), first_slot());
        let parses = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(Barrier::new(threads));

        let observed: Vec<Vec<(u32, NodeId)>> = std::thread::scope(|scope| {
            let mut handles = Vec::new();
            for _ in 0..threads {
                let owner = Arc::clone(&owner);
                let parses = Arc::clone(&parses);
                let barrier = Arc::clone(&barrier);
                handles.push(scope.spawn(move || {
                    barrier.wait();
                    let mut seen = Vec::with_capacity(keys_usize);
                    for index in 0..keys {
                        let token = owner.lazy().get_or_create_token(
                            TokenKey {
                                parent,
                                pos: index,
                                end: index + 1,
                            },
                            KIND_IDENTIFIER,
                            NodeFlags::NONE,
                            Some(JsString::from_text("t")),
                        );
                        // A reader must see a fully initialized node with the
                        // supplied parent, whichever thread published it.
                        assert_eq!(token.node().kind, KIND_IDENTIFIER);
                        assert_eq!(token.node().parent, Some(parent));
                        assert_eq!(token.node().pos, index);
                        seen.push((index, token.id()));

                        let jsdoc_owner = NodeId::new(
                            owner.core().id(),
                            Slot::from_raw(index % JSDOC_OWNERS + 1).expect("nonzero"),
                        );
                        let parses = Arc::clone(&parses);
                        let docs = owner
                            .lazy()
                            .resolve_jsdoc(jsdoc_owner, &move || {
                                parses.fetch_add(1, Ordering::SeqCst);
                                vec![NodeData::new(KIND_JSDOC, 0, 1)]
                            })
                            .expect("slots available");
                        assert_eq!(docs.len(), 1);
                        assert_eq!(docs[0].node().kind, KIND_JSDOC);
                    }
                    seen
                }));
            }
            handles
                .into_iter()
                .map(|handle| handle.join().expect("thread"))
                .collect()
        });

        // One published allocation per key, whatever the interleaving.
        let first = &observed[0];
        checks.require(
            "every thread observes the same id for each key",
            observed.iter().all(|seen| seen == first),
        );
        let unique: BTreeSet<_> = first.iter().map(|(_, id)| *id).collect();
        checks.require(
            "each token key published exactly one node",
            unique.len() == keys_usize,
        );
        checks.require(
            "JSDoc is parsed exactly once per owning node",
            parses.load(Ordering::SeqCst) == JSDOC_OWNERS as usize,
        );
        let published = u64::from(owner.lazy().published());
        checks.require(
            "the published count is the tokens plus the JSDoc nodes",
            published == u64::from(keys) + u64::from(JSDOC_OWNERS),
        );
        checks.require(
            "the page directory grew and holds exactly the published pages",
            owner.lazy().pages() > 1
                && owner.lazy().pages() == (published as usize).div_ceil(PAGE_SIZE),
        );

        // A reference into the first page is still readable after growth.
        let early = owner
            .lazy()
            .resolve(first[0].1)
            .expect("a published slot resolves");
        checks.require(
            "a reference taken across directory growth stays readable",
            early.node().kind == KIND_IDENTIFIER && early.node().pos == 0,
        );

        // A cached token whose kind disagrees is refused, as upstream panics.
        checks.require(
            "a cached kind mismatch is refused",
            matches!(
                owner.lazy().try_get_or_create_token(
                    TokenKey {
                        parent,
                        pos: 0,
                        end: 1
                    },
                    KIND_STRING_LITERAL,
                    NodeFlags::NONE,
                    None,
                ),
                Err(LazyError::KindMismatch { .. })
            ),
        );
        let panicked = catch_unwind(AssertUnwindSafe(|| {
            owner.lazy().get_or_create_token(
                TokenKey {
                    parent,
                    pos: 0,
                    end: 1,
                },
                KIND_STRING_LITERAL,
                NodeFlags::NONE,
                None,
            )
        }));
        checks.require("the panicking entry point still panics", panicked.is_err());

        // A reparsed parent can never own a lazily created token, and the
        // refusal leaves the arena usable.
        checks.require(
            "a reparsed parent is refused",
            matches!(
                owner.lazy().try_get_or_create_token(
                    TokenKey {
                        parent,
                        pos: 5000,
                        end: 5001
                    },
                    KIND_IDENTIFIER,
                    NodeFlags::REPARSED,
                    None,
                ),
                Err(LazyError::ReparsedParent { .. })
            ),
        );
        checks.require(
            "a refusal published nothing and left the arena usable",
            u64::from(owner.lazy().published()) == published
                && owner
                    .lazy()
                    .resolve(first[0].1)
                    .is_some_and(|node| node.node().kind == KIND_IDENTIFIER),
        );

        // The file owner is what keeps lazy storage resolvable.
        let mut scope = Scope::new();
        scope.add_file(&owner);
        checks.require(
            "lazy ids resolve through a scope that holds the file",
            scope.import(first[0].1).is_ok(),
        );
        let stray = NodeId::new(
            owner.lazy().id(),
            Slot::from_raw(owner.lazy().published() + 1).expect("nonzero"),
        );
        checks.require(
            "an unpublished lazy slot is rejected",
            refused(scope.import(stray), StaleId::OutOfBounds),
        );
    })
}

/// A mapper with three supplemental files, released in every order.
pub fn mapper_bundle_disposal() -> Outcome {
    run("mapper_bundle_disposal", |checks| {
        const HOLDERS: usize = 4;
        let mut orders = Vec::new();
        permutations(&mut (0..HOLDERS).collect::<Vec<_>>(), 0, &mut orders);
        checks.require("every release order is exercised", orders.len() == 24);

        let mut links_both_ways = true;
        let mut resolvable_while_held = true;
        let mut siblings_reachable = true;
        let mut held_until_last_drop = true;
        let mut freed_after_last_drop = true;

        for order in &orders {
            let allocations_before = live_allocations();
            let owners_before = live_owners();

            let canonical = file("<canonical>", 3);
            let supplemental: Vec<_> = (0..3).map(|_| file("<supplemental>", 2)).collect();
            let canonical_id = canonical.id();
            let supplemental_ids: Vec<_> = supplemental.iter().map(|file| file.id()).collect();
            let bundle = BundleOwner::new(canonical, supplemental);

            // Both link directions are ids, so the cycle owns nothing.
            let info = bundle.canonical().mapper_info().expect("canonical info");
            links_both_ways &= info.supplemental == supplemental_ids && info.canonical.is_none();
            for member in bundle.supplemental() {
                let info = member.mapper_info().expect("supplemental info");
                links_both_ways &=
                    info.canonical == Some(canonical_id) && info.supplemental.is_empty();
            }

            // Four independent holders, each retaining the bundle as one unit.
            let mut holders: Vec<Option<Scope>> = (0..HOLDERS)
                .map(|_| {
                    let mut scope = Scope::new();
                    scope.add_bundle(&bundle);
                    Some(scope)
                })
                .collect();
            let ids: Vec<NodeId> = bundle
                .members()
                .map(|member| NodeId::new(member.core().id(), first_slot()))
                .collect();
            let live = live_allocations();
            drop(bundle);

            for holder in order {
                resolvable_while_held &= holders
                    .iter()
                    .flatten()
                    .all(|scope| ids.iter().all(|id| scope.import(*id).is_ok()));
                siblings_reachable &= holders.iter().flatten().all(|scope| {
                    scope.bundle(ids[0].arena()).is_some_and(|bundle| {
                        supplemental_ids.iter().all(|id| bundle.file(*id).is_some())
                            && bundle.file(canonical_id).is_some()
                    })
                });
                held_until_last_drop &= live_allocations() == live;
                holders[*holder] = None;
            }

            freed_after_last_drop &=
                live_allocations() == allocations_before && live_owners() == owners_before;
        }

        checks.require(
            "canonical and supplemental links point both ways",
            links_both_ways,
        );
        checks.require(
            "every member id resolves while any holder lives",
            resolvable_while_held,
        );
        checks.require(
            "sibling links resolve through the bundle, not a file reference",
            siblings_reachable,
        );
        checks.require(
            "the bundle's storage is held until the last holder drops",
            held_until_last_drop,
        );
        checks.require(
            "the bundle's storage is freed after the last holder drops",
            freed_after_last_drop,
        );
    })
}

/// Scoped access: a branded local handle cannot cross arenas.
pub fn scoped_access() -> Outcome {
    run("scoped_access", |checks| {
        let owner = file("const a = 1;", 5);
        let other = file("const b = 2;", 5);
        let id = NodeId::new(owner.core().id(), first_slot());
        with_scoped_arena(owner.core(), |scope| {
            let local = scope.check(id).expect("an id of this arena checks");
            checks.require(
                "a local handle resolves without a second owner check",
                scope.get(local).kind == KIND_SOURCE_FILE,
            );
            checks.require("a local handle maps back to its id", scope.id(local) == id);
            let past_end = NodeId::new(
                owner.core().id(),
                Slot::from_raw(owner.core().published() + 1).expect("nonzero"),
            );
            checks.require(
                "an out-of-bounds id does not mint a local handle",
                scope.check(past_end).is_none(),
            );
        });
        with_scoped_arena(other.core(), |scope| {
            checks.require(
                "an id of another arena does not check",
                scope.check(id).is_none(),
            );
        });
    })
}

fn permutations(values: &mut Vec<usize>, index: usize, out: &mut Vec<Vec<usize>>) {
    if index == values.len() {
        out.push(values.clone());
        return;
    }
    for swap in index..values.len() {
        values.swap(index, swap);
        permutations(values, index + 1, out);
        values.swap(index, swap);
    }
}

/// Every scenario S04 implements, in a stable order.
pub fn all() -> Vec<Outcome> {
    vec![
        id_exhaustion(),
        wrong_owner_rejected(),
        stale_and_recycled_ids_rejected(),
        concurrent_lazy_storage(),
        mapper_bundle_disposal(),
        scoped_access(),
    ]
}
