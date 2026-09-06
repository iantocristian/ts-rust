fn main() {
    let metrics = e3_harness::run_scenarios();
    assert_eq!(metrics.live_owner_delta, 0);
    assert_eq!(metrics.live_allocation_delta, 0);
    println!(
        concat!(
            "{{\"metrics\":{{",
            "\"id_exhaustion\":{},",
            "\"wrong_owner_rejected\":{},",
            "\"stale_and_recycled_ids_rejected\":{},",
            "\"concurrent_lazy_storage\":{},",
            "\"mapper_bundle_disposal\":{},",
            "\"live_owner_delta\":{},",
            "\"live_allocation_delta\":{}",
            "}}}}"
        ),
        metrics.id_exhaustion,
        metrics.wrong_owner_rejected,
        metrics.stale_and_recycled_ids_rejected,
        metrics.concurrent_lazy_storage,
        metrics.mapper_bundle_disposal,
        metrics.live_owner_delta,
        metrics.live_allocation_delta,
    );
}
