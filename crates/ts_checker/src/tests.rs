use crate::{
    alias_symbol, alias_type_arguments, object_flags, to_node_builder_flags, type_flags,
    type_format_flags, CheckerOwner, Error, LinkStore, ResolutionStack, TypeAlias, TypeId,
    TypeKind, TypeRecord, TypeResolution, TypeStore, TypeSystemEntity, TypeSystemPropertyName,
};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{mpsc, Arc};
use ts_arena::{CheckerIdentity, Counters, Generation, NodeId, SymbolArena, SymbolId};
use ts_ast::{symbol_flags, JsString, Symbol};

fn node(arena: &SymbolArena<u32>, slot: u32) -> NodeId {
    NodeId::from_parts(arena.id(), slot).unwrap()
}

fn symbol(arena: &SymbolArena<u32>, slot: u32) -> SymbolId {
    SymbolId::from_parts(arena.id(), slot).unwrap()
}

fn owner() -> (Counters, Generation, Arc<CheckerIdentity>, CheckerOwner) {
    let counters = Counters::new();
    let generation = Generation::new(&counters);
    let identity = CheckerIdentity::new(generation.clone(), &counters);
    let owner = CheckerOwner::new(identity.clone(), &counters).unwrap();
    (counters, generation, identity, owner)
}

#[test]
fn link_store_pages_on_first_use_per_arena_and_distinguishes_absent_entries() {
    let counters = Counters::new();
    let first = SymbolArena::<u32>::new(&counters);
    let second = SymbolArena::<u32>::new(&counters);
    let mut links = LinkStore::<NodeId, u32>::new();
    assert!(links.is_empty());
    assert_eq!(links.pages(), 0);
    assert!(!links.has(node(&first, 1)));
    assert_eq!(links.try_get(node(&first, 1)), None);
    *links.get_or_default(node(&first, 1)) = 5;
    *links.get_or_default(node(&first, 300)) = 6;
    *links.get_or_default(node(&second, 1)) = 7;
    assert_eq!(links.len(), 3);
    assert_eq!(
        links.pages(),
        3,
        "slot 300 opens a second page; the second arena its own"
    );
    assert_eq!(links.try_get(node(&first, 1)), Some(&5));
    assert_eq!(links.try_get(node(&first, 300)), Some(&6));
    assert_eq!(links.try_get(node(&second, 1)), Some(&7));
    assert!(
        !links.has(node(&first, 2)),
        "a page does not make its other slots present"
    );
    assert!(!links.has(node(&first, 299)));
    assert_eq!(
        *links.get_or_default(node(&first, 1)),
        5,
        "Get returns the existing entry"
    );
    assert_eq!(links.len(), 3);
    *links.try_get_mut(node(&second, 1)).unwrap() = 8;
    assert_eq!(links.try_get(node(&second, 1)), Some(&8));
    let mut by_symbol = LinkStore::<SymbolId, Vec<u8>>::new();
    by_symbol.get_or_default(symbol(&first, 1)).push(1);
    assert_eq!(by_symbol.try_get(symbol(&first, 1)), Some(&vec![1]));
}

#[test]
fn resolution_cycle_invalidates_the_suffix_and_pushes_nothing() {
    let counters = Counters::new();
    let arena = SymbolArena::<u32>::new(&counters);
    let a = TypeSystemEntity::Symbol(symbol(&arena, 1));
    let b = TypeSystemEntity::Symbol(symbol(&arena, 2));
    let mut stack = ResolutionStack::new();
    let never = |_: &TypeResolution| false;
    assert!(stack.push(a, TypeSystemPropertyName::Type, never));
    assert!(stack.push(b, TypeSystemPropertyName::DeclaredType, never));
    assert!(
        stack.push(a, TypeSystemPropertyName::DeclaredType, never),
        "same entity, other property"
    );
    assert!(!stack.push(a, TypeSystemPropertyName::Type, never));
    assert_eq!(stack.depth(), 3, "a failed push adds nothing");
    assert!(
        !stack.pop(),
        "everything from the cycle start is marked failed"
    );
    assert!(!stack.pop());
    assert!(!stack.pop());
    assert_eq!(stack.depth(), 0);
}

#[test]
fn resolution_search_stops_at_a_produced_property_and_at_the_floor() {
    let counters = Counters::new();
    let arena = SymbolArena::<u32>::new(&counters);
    let a = TypeSystemEntity::Symbol(symbol(&arena, 1));
    let b = TypeSystemEntity::Type(TypeId::new(1).unwrap());
    let mut stack = ResolutionStack::new();
    assert!(stack.push(a, TypeSystemPropertyName::Type, |_| false));
    assert!(stack.push(b, TypeSystemPropertyName::ResolvedBaseTypes, |_| false));
    // b has produced its property meanwhile: the search stops there and a is not a cycle.
    let produced = |r: &TypeResolution| r.target == b;
    assert_eq!(
        stack.find_resolution_cycle_start_index(a, TypeSystemPropertyName::Type, produced),
        None
    );
    assert!(stack.push(a, TypeSystemPropertyName::Type, produced));
    assert!(stack.pop());
    assert_eq!(
        stack.find_resolution_cycle_start_index(a, TypeSystemPropertyName::Type, |_| false),
        Some(0)
    );
    assert_eq!(stack.set_resolution_start(2), 0);
    assert!(
        stack.push(a, TypeSystemPropertyName::Type, |_| false),
        "below the floor is not searched"
    );
    assert_eq!(stack.set_resolution_start(0), 2);
    assert!(stack.pop());
    assert!(stack.pop());
    assert!(stack.pop());
}

#[test]
fn owner_adopts_the_identity_once_and_scopes_state_to_an_operation() {
    let (counters, _generation, identity, owner) = owner();
    assert!(matches!(
        CheckerOwner::new(identity.clone(), &counters),
        Err(Error::Arena(ts_arena::Error::IdentityAdopted))
    ));
    let mut operation = owner.operation().unwrap();
    assert_eq!(operation.state().symbols().id(), identity.id());
    let undefined = operation.state_mut().symbols_mut().push(Symbol::new(
        symbol_flags::PROPERTY,
        JsString::from_bytes(&b"undefined"[..]),
    ));
    assert_eq!(
        operation.lease().validate_identity(undefined.arena()),
        Ok(())
    );
    let record = TypeRecord {
        flags: type_flags::ANY,
        object_flags: object_flags::NONE,
        symbol: None,
        alias: None,
        kind: TypeKind::Intrinsic,
        payload_row: 0,
    };
    let any = operation.state_mut().types_mut().push(record).unwrap();
    assert_eq!(any.get(), 1, "type ids start at 1 like TypeCount");
    assert_eq!(operation.state().types().get(any), Some(&record));
    assert!(operation.state_mut().resolution_mut().push(
        TypeSystemEntity::Type(any),
        TypeSystemPropertyName::ResolvedBaseConstraint,
        |_| false
    ));
    assert!(operation.state_mut().resolution_mut().pop());
    drop(operation);
    assert_eq!(owner.operation().unwrap().state().types().len(), 1);
}

#[test]
fn same_thread_reentry_fails_before_waiting_and_contention_waits() {
    let (_counters, _generation, _identity, owner) = owner();
    let owner = Arc::new(owner);
    let held = owner.operation().unwrap();
    assert!(matches!(owner.operation(), Err(Error::Reentry)));
    let (started, started_rx) = mpsc::channel();
    let (finished, finished_rx) = mpsc::channel();
    let contender = {
        let owner = owner.clone();
        std::thread::spawn(move || {
            started.send(()).unwrap();
            let operation = owner.operation();
            finished.send(operation.is_ok()).unwrap();
        })
    };
    started_rx.recv().unwrap();
    assert!(
        finished_rx.try_recv().is_err(),
        "another thread waits while the operation is held"
    );
    drop(held);
    assert!(finished_rx.recv().unwrap(), "and acquires it once released");
    contender.join().unwrap();
    assert!(
        owner.operation().is_ok(),
        "the reentry record is cleared on drop"
    );
}

#[test]
fn a_panic_inside_an_operation_retires_the_generation() {
    let (_counters, generation, _identity, owner) = owner();
    let result = catch_unwind(AssertUnwindSafe(|| {
        let _operation = owner.operation().unwrap();
        panic!("injected checker failure");
    }));
    assert!(result.is_err());
    assert_eq!(generation.validate(), Err(ts_arena::Error::Retired));
    assert!(matches!(
        owner.operation(),
        Err(Error::Arena(ts_arena::Error::Retired))
    ));
}

#[test]
fn a_direct_identity_lease_cannot_bypass_operation_reentry_detection() {
    let (_counters, _generation, identity, owner) = owner();
    let lease = identity.lease().unwrap();
    assert!(matches!(owner.operation(), Err(Error::Reentry)));
    drop(lease);
    let operation = owner.operation().unwrap();
    assert!(matches!(identity.lease(), Err(ts_arena::Error::Reentry)));
    drop(operation);
    assert!(identity.lease().is_ok());
}

#[test]
fn owner_is_send_and_sync_and_ids_reject_zero() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<CheckerOwner>();
    assert_eq!(TypeId::new(0), None);
    assert_eq!(crate::SignatureId::new(0), None);
}

#[test]
fn type_store_numbers_from_one_and_alias_helpers_tolerate_absence() {
    let counters = Counters::new();
    let arena = SymbolArena::<u32>::new(&counters);
    let mut store = TypeStore::new();
    let record = TypeRecord {
        flags: type_flags::STRING_LITERAL,
        object_flags: object_flags::NONE,
        symbol: None,
        alias: None,
        kind: TypeKind::Literal,
        payload_row: 0,
    };
    let first = store.push(record).unwrap();
    let second = store.push(record).unwrap();
    assert_eq!((first.get(), second.get(), store.len()), (1, 2, 2));
    assert_eq!(store.get(TypeId::new(3).unwrap()), None);
    let alias = store
        .push_alias(TypeAlias {
            symbol: symbol(&arena, 1),
            type_arguments: Arc::from([first, second]),
        })
        .unwrap();
    store.get_mut(second).unwrap().alias = Some(alias);
    let stored = store.alias(alias);
    assert_eq!(alias_symbol(stored), Some(symbol(&arena, 1)));
    assert_eq!(alias_type_arguments(stored), &[first, second]);
    assert_eq!(alias_symbol(None), None);
    assert_eq!(alias_type_arguments(None), &[] as &[TypeId]);
}

#[test]
fn node_builder_flags_are_a_mask_of_type_format_flags() {
    let flags = type_format_flags::NO_TRUNCATION
        | type_format_flags::ADD_UNDEFINED
        | type_format_flags::USE_ALIAS_DEFINED_OUTSIDE_CURRENT_SCOPE
        | type_format_flags::IN_ARRAY_TYPE;
    let converted = to_node_builder_flags(flags);
    assert_eq!(
        converted,
        ts_nodebuilder::flags::NO_TRUNCATION
            | ts_nodebuilder::flags::USE_ALIAS_DEFINED_OUTSIDE_CURRENT_SCOPE
    );
    assert_eq!(
        converted & type_format_flags::ADD_UNDEFINED,
        0,
        "TypeFormatFlags-only bits are masked out"
    );
}
