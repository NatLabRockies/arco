use arco_kdl::semantic::{FamilySignature, ResolvedSet, validate_program};
use arco_kdl::source::parse_program_text;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn semantic_validation_accepts_canonical_model_use() -> Result<(), Box<dyn std::error::Error>> {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_nanos()
        .to_string();
    let root = std::env::temp_dir().join(format!("arco-dsl-semantic-canonical-{unique}"));
    fs::create_dir_all(&root)?;

    let path = root.join("canonical.kdl");
    let text = r#"
model "EconomicDispatch" {
  control "dispatch" {
    a
    t
  }

  constraint "balance[a,t]" {
    dispatch[a,t] = 0
  }

  minimize "TotalCost" {
    sum(dispatch[a,t] for a in assets for t in time)
  }
}

scenario "Case" {
  horizon steps=2 resolution="PT1H"
  use "EconomicDispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let semantic = validate_program(&parsed.program, &path)?;
    assert_eq!(semantic.active_scenario, "Case");
    assert_eq!(semantic.active_objective.name, "TotalCost");
    assert_eq!(
        semantic.variable_families,
        vec![FamilySignature::new("dispatch", ["a", "t"])]
    );

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_rejects_canonical_scenario_without_use()
-> Result<(), Box<dyn std::error::Error>> {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_nanos()
        .to_string();
    let root = std::env::temp_dir().join(format!("arco-dsl-semantic-canonical-missing-{unique}"));
    fs::create_dir_all(&root)?;

    let path = root.join("canonical_missing_use.kdl");
    let text = r#"
model "EconomicDispatch" {
  control "dispatch" {
    a
    t
  }
  minimize "TotalCost" {
    dispatch[a,t]
  }
}

scenario "Case" {
  horizon steps=1 resolution="PT1H"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path).expect_err("validation should fail");
    assert!(
        error
            .to_string()
            .contains("missing declaration `model` named `active model`")
    );

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn set_registry_contains_built_in_sets_after_semantic_analysis()
-> Result<(), Box<dyn std::error::Error>> {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_nanos()
        .to_string();
    let root = std::env::temp_dir().join(format!("arco-dsl-semantic-set-registry-{unique}"));
    fs::create_dir_all(&root)?;

    let path = root.join("registry.kdl");
    let text = r#"
model "Simple" {
  control "x" {
    a
    t
  }

  constraint "eq[a,t]" {
    x[a,t] = 0
  }

  minimize "Obj" {
    sum(x[a,t] for a in assets for t in time)
  }
}

scenario "S1" {
  horizon steps=3 resolution="PT1H"
  use "Simple"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let semantic = validate_program(&parsed.program, &path)?;

    // The registry should contain entries for assets, candidate_assets, and time.
    assert!(
        semantic.set_registry.contains_key("assets"),
        "registry should contain 'assets'"
    );
    assert!(
        semantic.set_registry.contains_key("candidate_assets"),
        "registry should contain 'candidate_assets'"
    );
    assert!(
        semantic.set_registry.contains_key("time"),
        "registry should contain 'time'"
    );

    // time set should have 3 string entries matching the horizon.
    assert_eq!(
        semantic.set_registry["time"],
        ResolvedSet {
            values: vec!["1".to_string(), "2".to_string(), "3".to_string()]
        }
    );

    // assets set should match the ResolvedSets.assets (which mirrors set_registry).
    assert_eq!(semantic.set_registry["assets"].values, semantic.sets.assets);

    fs::remove_dir_all(&root)?;
    Ok(())
}
