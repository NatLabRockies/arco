use arco_cli::benchmark::evaluate_manifest;
use std::path::PathBuf;

#[test]
fn benchmark_manifest_cases_match_golden_semantic_programs()
-> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("e2e")
        .join("manifest.json");

    let outcomes = evaluate_manifest(&manifest_path)?;

    let case_ids: Vec<&str> = outcomes.iter().map(|c| c.case_id.as_str()).collect();
    assert_eq!(
        case_ids,
        &[
            "price-taker-battery",
            "capacity-expansion",
            "pcm",
            "simple-electricity-market-storage",
            "generator-allocation",
            "sdom",
            "sdom-canonical",
        ]
    );

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

    Ok(())
}
