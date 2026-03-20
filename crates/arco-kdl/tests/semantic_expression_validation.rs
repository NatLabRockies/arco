use arco_kdl::semantic::validate_program;
use arco_kdl::source::parse_program_text;
use miette::Diagnostic;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn semantic_validation_rejects_missing_named_expression_dependency()
-> Result<(), Box<dyn std::error::Error>> {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_nanos()
        .to_string();
    let root = std::env::temp_dir().join(format!("arco-dsl-semantic-expr-{unique}"));
    fs::create_dir_all(&root)?;

    let path = root.join("missing_expression_dependency.kdl");
    let text = r#"
technology "Battery" {
  control "charge"
}

expression "BatteryValue" {
  0
}

maximize "BatteryObjective" {
  BatteryValue + MissingValue
}

asset "Battery1" {
  technology "Battery"
  power_mw 10
}

scenario "Case" {
  horizon steps=1 resolution="PT1H"
  technology "Battery"
  asset "Battery1"
  maximize "BatteryObjective"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path).expect_err("validation should fail");
    assert!(
        error
            .to_string()
            .contains("missing declaration `expression` named `MissingValue`")
    );
    assert_eq!(
        error.code().map(|code| code.to_string()),
        Some("arco::semantic::missing_declaration".to_string())
    );

    fs::remove_dir_all(&root)?;
    Ok(())
}
