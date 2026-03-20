use arco_kdl::pipeline::compile_file;
use std::path::PathBuf;
use std::time::Instant;

#[test]
fn lowering_performance_benchmark() -> Result<(), Box<dyn std::error::Error>> {
    let test_cases = vec![
        ("price-taker-battery", "input.kdl"),
        ("pcm", "input.kdl"),
        ("capacity-expansion", "input.kdl"),
        ("simple-electricity-market-storage", "input.kdl"),
        ("generator-allocation", "input.kdl"),
        ("sdom", "input.kdl"),
        ("sdom-canonical", "input.kdl"),
    ];

    let test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("e2e");

    let mut total_lowering_us = 0u128;

    for (case_name, filename) in &test_cases {
        let kdl_path = test_dir.join(case_name).join(filename);

        // Compile up to lowering phase
        let start = Instant::now();
        let _lowered = compile_file(&kdl_path)?;
        let elapsed = start.elapsed();

        let lowering_us = elapsed.as_micros();
        total_lowering_us += lowering_us;

        println!("  {}: {} us", case_name, lowering_us);
    }

    println!("METRIC lowering_µs={}", total_lowering_us);
    println!(
        "Total lowering time: {} us ({} ms)",
        total_lowering_us,
        total_lowering_us as f64 / 1000.0
    );

    Ok(())
}
