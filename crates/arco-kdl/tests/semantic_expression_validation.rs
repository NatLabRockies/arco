mod common;
use arco_kdl::semantic::validate_program;
use arco_kdl::source::parse_program_text;
use common::{fixture_text, temp_root};
use std::fs;

#[test]
fn semantic_validation_rejects_missing_report_expression_target()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-expr-missing")?;
    let path = root.join("input.kdl");
    let text = fixture_text("semantic_validation_rejects_missing_report_expression_target.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
    let error = validate_program(&parsed.program, &path).expect_err("expected semantic failure");
    assert!(
        error
            .to_string()
            .contains("missing declaration `expression or control` named `MissingExpr`")
    );

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_rejects_expression_cycles() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-expr-cycle")?;
    let path = root.join("input.kdl");
    let text = fixture_text("semantic_validation_rejects_expression_cycles.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
    let error = validate_program(&parsed.program, &path).expect_err("expected semantic failure");
    assert!(error.to_string().contains("expression cycle"));
    assert!(error.to_string().contains("E1"));
    assert!(error.to_string().contains("E2"));

    fs::remove_dir_all(&root)?;
    Ok(())
}
