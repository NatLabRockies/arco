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
fn semantic_validation_rejects_missing_scenario() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-no-scenario")?;
    let path = root.join("input.kdl");
    let text = r#"
model "Dispatch" {
  control "x" {
    index "a"
    index "t"
  }
  minimize "Obj" {
    sum(x[a,t] for a in assets for t in time)
  }
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path).expect_err("expected semantic failure");
    assert!(error.to_string().contains("no scenario is available"));

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_requires_single_scenario() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-multi-scenario")?;
    let path = root.join("input.kdl");
    let text = r#"
model "Dispatch" {
  control "x" {
    index "a"
    index "t"
  }
  minimize "Obj" {
    sum(x[a,t] for a in assets for t in time)
  }
}

scenario "S1" {
  use "Dispatch"
}

scenario "S2" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path).expect_err("expected semantic failure");
    assert!(error.to_string().contains("exactly one scenario"));

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_requires_scenario_data_bindings_to_match_known_params()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-data-param-match")?;
    let path = root.join("input.kdl");
    let text = r#"
model "Dispatch" {
  param "demand" {
    index "t"
  }
  control "x" {
    index "a"
    index "t"
  }
  constraint "bal[a,t]" {
    x[a,t] <= demand[t]
  }
  minimize "Obj" {
    sum(x[a,t] for a in assets for t in time)
  }
}

scenario "S1" {
  use "Dispatch"
  data "unknown_param" from="data.csv"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path).expect_err("expected semantic failure");
    assert!(error.to_string().contains("unknown_param"));
    assert!(error.to_string().contains("parameter"));

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_resolves_reports_and_registry_for_low_level_model()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-reports-registry")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("assets.csv"),
        "asset_id,is_candidate,zone\nA,1,north\nB,0,south\n",
    )?;

    let path = root.join("input.kdl");
    let text = r#"
set "time" { "1"; "2" }

data "generator_data" from="data/assets.csv" {
  set "asset_id"
}

model "Dispatch" {
  control "x" {
    index "a"
    index "t"
  }

  expression "FuelCost" {
    sum(x[a,t] for a in asset_id for t in time)
  }

  constraint "balance[a,t]" {
    x[a,t] = 0
  }

  minimize "TotalCost" {
    FuelCost
  }
}

scenario "S1" {
  use "Dispatch"
  report FuelCost
  report TotalCost
  report dual "balance[a,t]"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let semantic = validate_program(&parsed.program, &path)?;

    assert_eq!(semantic.active_scenario, "S1");
    assert_eq!(semantic.active_objective.name, "TotalCost");
    assert_eq!(semantic.variable_families.len(), 1);
    assert_eq!(semantic.variable_families[0].target, "x");
    assert_eq!(semantic.variable_families[0].indices, vec!["a", "t"]);
    assert_eq!(semantic.active_reports.len(), 2);
    assert_eq!(semantic.active_dual_reports.len(), 1);
    assert!(
        semantic
            .active_expressions
            .iter()
            .any(|e| e.name == "FuelCost")
    );
    assert!(semantic.set_registry.contains_key("time"));
    assert!(semantic.set_registry.contains_key("asset_id"));

    fs::remove_dir_all(&root)?;
    Ok(())
}
