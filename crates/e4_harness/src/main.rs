fn main() {
    let metrics = e4_harness::run_scenarios();
    println!(
        concat!(
            "{{\"metrics\":{{",
            "\"source_decoding\":{},",
            "\"helper_semantics\":{},",
            "\"slice_validity\":{},",
            "\"utf8_positions\":{},",
            "\"utf16_positions\":{}",
            "}}}}"
        ),
        metrics.source_decoding,
        metrics.helper_semantics,
        metrics.slice_validity,
        metrics.utf8_positions,
        metrics.utf16_positions,
    );
}
