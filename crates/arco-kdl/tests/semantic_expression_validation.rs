use arco_kdl::semantic::validate_program;
use arco_kdl::source::parse_program_text;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root(prefix: &str) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_nanos()
        .to_string();
    let root = std::env::temp_dir().join(format!("arco-kdl-{prefix}-{unique}"));
    fs::create_dir_all(&root)?;
    Ok(root)
}

#[test]
fn semantic_validation_rejects_missing_report_expression_target()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-expr-missing")?;
    let path = root.join("input.kdl");
    let text = r#"
model "Dispatch" {
  control "x" {
    index "a"
    index "t"
  }

  expression "BaseCost" {
    x[a,t]
  }

  minimize "TotalCost" {
    BaseCost
  }
}

scenario "S1" {
  horizon steps=1 resolution="PT1H"
  use "Dispatch"
  report MissingExpr
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path).expect_err("expected semantic failure");
    assert!(
        error
            .to_string()
            .contains("missing declaration `expression` named `MissingExpr`")
    );

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_rejects_expression_cycles() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-expr-cycle")?;
    let path = root.join("input.kdl");
    let text = r#"
model "Dispatch" {
  control "x" {
    index "a"
    index "t"
  }

  expression "E1" {
    E2 + x[a,t]
  }

  expression "E2" {
    E1
  }

  minimize "TotalCost" {
    E1
  }
}

scenario "S1" {
  horizon steps=1 resolution="PT1H"
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path).expect_err("expected semantic failure");
    assert!(error.to_string().contains("expression cycle"));
    assert!(error.to_string().contains("E1"));
    assert!(error.to_string().contains("E2"));

    fs::remove_dir_all(&root)?;
    Ok(())
}
