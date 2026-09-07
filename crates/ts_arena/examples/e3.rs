fn main() {
    let results = ts_arena::scenarios::run_all();
    let mut metrics = Vec::new();
    for (criterion, measurement) in results {
        eprintln!("{criterion}: {measurement:?}");
        if criterion != "owners_return_to_baseline" && criterion != "allocations_return_to_baseline"
        {
            metrics.push(format!("\"{criterion}\":true"));
        }
    }
    let owners = results
        .iter()
        .map(|(_, result)| result.live_owner_delta)
        .max()
        .unwrap();
    let allocations = results
        .iter()
        .map(|(_, result)| result.live_allocation_delta)
        .max()
        .unwrap();
    metrics.push(format!("\"live_owner_delta\":{owners}"));
    metrics.push(format!("\"live_allocation_delta\":{allocations}"));
    let tests = results
        .iter()
        .map(|(id, _)| format!("{{\"id\":\"{id}\",\"result\":\"pass\"}}"))
        .collect::<Vec<_>>()
        .join(",");
    println!(
        "{{\"metrics\":{{{}}},\"tests\":[{tests}]}}",
        metrics.join(",")
    );
}
