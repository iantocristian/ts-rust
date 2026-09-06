//! Unit tests for the ownership invariants. The E3 harness measures the design
//! note's scenarios; these cover the pieces a scenario would only reach
//! indirectly.

use std::sync::Arc;

use ts_jsstring::SourceText;

use crate::checker::StaleId;
use crate::core_arena::CoreArena;
use crate::ids::{allocate_arena_id, ArenaId, IdCounter, Slot, SlotCounter, MAX_ID};
use crate::lazy::{LazyArena, LazyError, TokenKey};
use crate::node::{NodeData, NodeFlags, NodeKind};
use crate::owner::{BundleOwner, FileOwnerBuilder, ScratchOwner};
use crate::scope::{with_scoped_arena, Scope};
use crate::{live_allocations, live_owners, CheckerPool, NodeId, PAGE_SIZE};

/// The live owner and allocation counters are process-global, so the tests that
/// read them run one at a time.
static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn serial() -> std::sync::MutexGuard<'static, ()> {
    SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

const KIND: NodeKind = NodeKind(1);
const OTHER_KIND: NodeKind = NodeKind(2);

fn slot(value: u32) -> Slot {
    Slot::from_raw(value).expect("nonzero")
}

fn file(nodes: u32) -> Arc<crate::FileOwner> {
    let mut builder =
        FileOwnerBuilder::new(SourceText::decode(b"const a = 1;")).expect("arena ids available");
    for index in 0..nodes {
        builder
            .allocate(NodeData::new(KIND, index, index + 1))
            .expect("slot");
    }
    builder.finish().expect("arena ids available")
}

#[test]
fn arena_ids_are_never_reused() {
    let _serial = serial();
    let first = allocate_arena_id().expect("arena");
    let second = allocate_arena_id().expect("arena");
    assert_ne!(first, second);
    assert!(second.get() > first.get());
}

#[test]
fn a_counter_never_wraps() {
    let _serial = serial();
    let counter = IdCounter::starting_at("arena", MAX_ID - 1);
    assert_eq!(counter.allocate(), Ok(MAX_ID - 1));
    assert_eq!(counter.allocate(), Ok(MAX_ID));
    assert!(counter.allocate().is_err());
    assert_eq!(counter.peek(), u64::from(MAX_ID) + 1);
}

#[test]
fn a_core_arena_resolves_only_its_own_ids() {
    let _serial = serial();
    let mut arena =
        CoreArena::with_ids(ArenaId::from_raw(11).expect("nonzero"), SlotCounter::new());
    let id = arena.allocate(NodeData::new(KIND, 0, 1)).expect("slot");
    assert!(arena.resolve(id).is_some());
    let foreign = NodeId::new(ArenaId::from_raw(12).expect("nonzero"), id.slot());
    assert!(arena.resolve(foreign).is_none());
    assert!(arena.get(slot(2)).is_none());
}

#[test]
fn lazy_pages_hold_their_nodes_across_growth() {
    let _serial = serial();
    let arena = LazyArena::new().expect("arena");
    let parent = NodeId::new(ArenaId::from_raw(1).expect("nonzero"), slot(1));
    let mut refs = Vec::new();
    for index in 0..=(PAGE_SIZE as u32 * 2) {
        refs.push(arena.get_or_create_token(
            TokenKey {
                parent,
                pos: index,
                end: index + 1,
            },
            KIND,
            NodeFlags::NONE,
            None,
        ));
    }
    assert_eq!(arena.pages(), 3);
    // Every reference taken before the directory grew is still readable.
    for (index, node) in refs.iter().enumerate() {
        assert_eq!(node.node().pos, index as u32);
    }
    // The cache returns the same id, not a second allocation.
    let again = arena.get_or_create_token(
        TokenKey {
            parent,
            pos: 0,
            end: 1,
        },
        KIND,
        NodeFlags::NONE,
        None,
    );
    assert_eq!(again.id(), refs[0].id());
    assert_eq!(arena.published(), PAGE_SIZE as u32 * 2 + 1);
}

#[test]
fn lazy_refusals_leave_the_arena_usable() {
    let _serial = serial();
    let arena = LazyArena::new().expect("arena");
    let parent = NodeId::new(ArenaId::from_raw(1).expect("nonzero"), slot(1));
    let key = TokenKey {
        parent,
        pos: 0,
        end: 1,
    };
    let token = arena.get_or_create_token(key, KIND, NodeFlags::NONE, None);
    assert!(matches!(
        arena.try_get_or_create_token(key, OTHER_KIND, NodeFlags::NONE, None),
        Err(LazyError::KindMismatch { .. })
    ));
    assert!(matches!(
        arena.try_get_or_create_token(
            TokenKey {
                parent,
                pos: 9,
                end: 10
            },
            KIND,
            NodeFlags::REPARSED,
            None
        ),
        Err(LazyError::ReparsedParent { .. })
    ));
    assert_eq!(arena.published(), 1);
    assert_eq!(
        arena.resolve(token.id()).expect("published").node().kind,
        KIND
    );
}

#[test]
fn jsdoc_is_parsed_once_per_node() {
    let _serial = serial();
    let arena = LazyArena::new().expect("arena");
    let node = NodeId::new(ArenaId::from_raw(1).expect("nonzero"), slot(1));
    let parses = std::cell::Cell::new(0);
    let parse = || {
        parses.set(parses.get() + 1);
        vec![NodeData::new(KIND, 0, 1), NodeData::new(KIND, 1, 2)]
    };
    let first = arena.resolve_jsdoc(node, &parse).expect("slots");
    let second = arena.resolve_jsdoc(node, &parse).expect("slots");
    assert_eq!(parses.get(), 1);
    assert_eq!(
        first
            .iter()
            .map(crate::lazy::LazyRef::id)
            .collect::<Vec<_>>(),
        second
            .iter()
            .map(crate::lazy::LazyRef::id)
            .collect::<Vec<_>>()
    );
    assert_eq!(arena.published(), 2);
}

#[test]
fn bind_once_binds_once() {
    let _serial = serial();
    let owner = file(2);
    let binds = std::cell::Cell::new(0);
    owner.bind_once(|| binds.set(binds.get() + 1));
    owner.bind_once(|| binds.set(binds.get() + 1));
    assert_eq!(binds.get(), 1);
    assert!(owner.is_bound());
}

#[test]
fn dropping_an_owner_returns_the_counters_to_their_baseline() {
    let _serial = serial();
    let owners = live_owners();
    let allocations = live_allocations();
    {
        let owner = file(5);
        assert_eq!(live_allocations(), allocations + 5);
        assert!(live_owners() > owners);
        let _bundle = BundleOwner::new(owner, vec![file(3)]);
        assert_eq!(live_allocations(), allocations + 8);
    }
    assert_eq!(live_owners(), owners);
    assert_eq!(live_allocations(), allocations);
}

#[test]
fn scratch_is_its_own_owner() {
    let _serial = serial();
    let mut scratch = ScratchOwner::new().expect("arena");
    let id = scratch.allocate(NodeData::new(KIND, 0, 1)).expect("slot");
    let scratch = scratch.into_shared();
    let mut request = Scope::new();
    request.add_scratch(&scratch);
    assert!(request.import(id).is_ok());
    assert_eq!(Scope::new().import(id).err(), Some(StaleId::UnknownArena));
}

#[test]
fn a_checker_node_needs_that_exact_checker_permit() {
    let _serial = serial();
    let pool = CheckerPool::new();
    let left = pool.checker().expect("arena");
    let right = pool.checker().expect("arena");
    let node = left.allocate(NodeData::new(KIND, 0, 1)).expect("slot");
    let mut scope = Scope::new();
    scope.add_checker(&left);
    scope.add_checker(&right);
    assert_eq!(scope.import(node).err(), Some(StaleId::WrongChecker));
    let permit = right.permit();
    assert_eq!(
        scope.import_with_permit(node, &permit).err(),
        Some(StaleId::WrongChecker)
    );
    drop(permit);
    let permit = left.permit();
    assert!(scope.import_with_permit(node, &permit).is_ok());
}

#[test]
fn a_bundle_resolves_its_siblings_and_a_file_alone_does_not() {
    let _serial = serial();
    let canonical = file(2);
    let supplemental = file(2);
    let canonical_id = canonical.id();
    let supplemental_id = supplemental.id();
    let bundle = BundleOwner::new(canonical, vec![supplemental]);
    assert_eq!(
        bundle.canonical().mapper_info().expect("info").supplemental,
        vec![supplemental_id]
    );
    assert_eq!(
        bundle.supplemental()[0]
            .mapper_info()
            .expect("info")
            .canonical,
        Some(canonical_id)
    );
    let mut scope = Scope::new();
    scope.add_bundle(&bundle);
    let arena = bundle.canonical().core().id();
    assert!(scope.bundle(arena).is_some());

    // The same file retained on its own cannot follow the sibling link.
    let mut file_only = Scope::new();
    file_only.add_file(bundle.canonical());
    assert!(file_only.bundle(arena).is_none());
}

#[test]
fn a_local_handle_belongs_to_one_scope() {
    let _serial = serial();
    let owner = file(3);
    let id = NodeId::new(owner.core().id(), slot(1));
    with_scoped_arena(owner.core(), |scope| {
        let local = scope.check(id).expect("checks");
        assert_eq!(scope.id(local), id);
        assert_eq!(scope.get(local).pos, 0);
        assert!(scope
            .check(NodeId::new(owner.core().id(), slot(99)))
            .is_none());
    });
}
