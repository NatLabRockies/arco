use arco_cli::benchmark::evaluate_manifest;
use std::path::PathBuf;
use std::time::Instant;

#[test]
fn csv_reading_performance_benchmark() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("e2e")
        .join("manifest.json");

    let start = Instant::now();
    let outcomes = evaluate_manifest(&manifest_path)?;
    let elapsed = start.elapsed();

    let total_us = elapsed.as_micros();
    println!("METRIC csv_read_us={}", total_us);

    // Verify all cases passed
    for outcome in &outcomes {
        assert!(
            outcome.actual_e2e_summary.expect_parse_success,
            "case {} failed parse",
            outcome.case_id
        );
        assert!(
            outcome
                .actual_e2e_summary
                .expect_semantic_validation_success,
            "case {} failed semantic validation",
            outcome.case_id
        );
        assert!(
            outcome.actual_e2e_summary.expect_lowering_success,
            "case {} failed lowering",
            outcome.case_id
        );
        assert!(
            outcome.actual_e2e_summary.expect_solve_success,
            "case {} failed solve",
            outcome.case_id
        );
    }

    println!("✓ All {} benchmark cases passed", outcomes.len());
    Ok(())
}
