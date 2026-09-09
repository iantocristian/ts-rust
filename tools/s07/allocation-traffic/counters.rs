//! Diagnostic backing requests; injected only into a frozen disposable source.
use std::{cell::Cell, sync::atomic::{AtomicU64, Ordering}};

pub const FAMILIES: [&str; 19] = [
    "core_header_pages", "core_header_directories", "auxiliary_slot_pages", "auxiliary_slot_directories",
    "symbol_pages", "symbol_directories", "flow_pages", "flow_directories",
    "flow_list_pages", "flow_list_directories", "other_arena_pages", "other_arena_directories",
    "payload_row_pages", "payload_row_directories", "edge_pages", "edge_directories",
    "text_pool_entries", "text_pool_free_slots", "eager_parser_list_buffers",
];
pub const PHASES: [&str; 3] = ["unscoped", "parse", "bind_publish"];
pub const FIELDS: [&str; 4] = ["requested_bytes", "replaced_bytes", "allocation_calls", "release_bytes"];
const WIDTH: usize = FAMILIES.len() * PHASES.len() * FIELDS.len();
static COUNTS: [AtomicU64; WIDTH] = [const { AtomicU64::new(0) }; WIDTH];
thread_local! { static PHASE: Cell<usize> = const { Cell::new(0) }; }

pub fn set_phase(phase: usize) { assert!(phase < PHASES.len()); PHASE.set(phase); }
pub fn reset() { for value in &COUNTS { value.store(0, Ordering::Relaxed); } }
pub fn snapshot() -> Vec<u64> { COUNTS.iter().map(|v| v.load(Ordering::Relaxed)).collect() }
pub fn family_sum(family: usize) -> [u64; 4] {
    assert!(family < FAMILIES.len());
    std::array::from_fn(|field| (0..PHASES.len()).map(|phase| {
        COUNTS[(phase * FAMILIES.len() + family) * 4 + field].load(Ordering::Relaxed)
    }).sum())
}
pub fn sum() -> [u64; 4] {
    let mut result = [0; 4];
    for (index, value) in COUNTS.iter().enumerate() { result[index % 4] += value.load(Ordering::Relaxed); }
    result
}
pub fn allocation(family: usize, new_bytes: usize, old_bytes: usize) {
    if new_bytes == old_bytes { return; }
    let at = (PHASE.get() * FAMILIES.len() + family) * 4;
    for (field, value) in [(0, new_bytes), (1, old_bytes), (2, usize::from(new_bytes != 0))] {
        COUNTS[at + field].fetch_add(value as u64, Ordering::Relaxed);
    }
}
pub fn release(family: usize, bytes: usize) {
    let at = (PHASE.get() * FAMILIES.len() + family) * 4;
    COUNTS[at + 3].fetch_add(bytes as u64, Ordering::Relaxed);
}
pub fn growth<T>(family: usize, old: usize, values: &Vec<T>) {
    allocation(family, values.capacity() * size_of::<T>(), old * size_of::<T>());
}
pub fn arena_family<T>() -> usize {
    let name = std::any::type_name::<T>();
    if name.ends_with("compact::StoredNode") { 0 }
    else if name.ends_with("auxiliary::StoredAux") { 2 }
    else if name.ends_with("StoredSymbol") { 4 }
    else if name.ends_with("StoredFlowNode") { 6 }
    else if name.ends_with("StoredFlowList") { 8 }
    else { 10 }
}
static WINDOWS: [AtomicU64; 8] = [const { AtomicU64::new(0) }; 8];
pub fn window(phase: usize, before: [usize; 2], after: [usize; 2]) {
    assert!((1..=2).contains(&phase));
    let at = (phase - 1) * 4;
    WINDOWS[at].fetch_add((after[0] - before[0]) as u64, Ordering::Relaxed);
    // Keep positive and negative live changes separate for lossless unsigned counters.
    WINDOWS[at + 1].fetch_add(after[1].saturating_sub(before[1]) as u64, Ordering::Relaxed);
    WINDOWS[at + 2].fetch_add(before[1].saturating_sub(after[1]) as u64, Ordering::Relaxed);
    WINDOWS[at + 3].fetch_add(1, Ordering::Relaxed);
}
pub fn windows() -> [u64; 8] { std::array::from_fn(|i| WINDOWS[i].load(Ordering::Relaxed)) }
