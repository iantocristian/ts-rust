use crate::scenarios;
#[test]
fn e3_id_exhaustion() {
    scenarios::id_exhaustion();
}
#[test]
fn e3_wrong_owner_rejected() {
    scenarios::wrong_owner_rejected();
}
#[test]
fn e3_stale_and_recycled_ids_rejected() {
    scenarios::stale_and_recycled_ids_rejected();
}
#[test]
fn e3_concurrent_lazy_storage() {
    scenarios::concurrent_lazy_storage();
}
#[test]
fn e3_mapper_bundle_disposal() {
    scenarios::mapper_bundle_disposal();
}
#[test]
fn e3_owners_return_to_baseline() {
    scenarios::owners_return_to_baseline();
}
#[test]
fn e3_allocations_return_to_baseline() {
    scenarios::allocations_return_to_baseline();
}
