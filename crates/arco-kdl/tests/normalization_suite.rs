use arco_kdl::normalize::normalize_program;
use arco_kdl::source::parse_program_text;
use std::path::PathBuf;

#[test]
fn normalization_equates_direct_wiring_and_canonical_models()
-> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/normalize-equivalence.kdl");

    let direct_wiring = r#"
technology "Generator" {
  control "dispatch"
}

operation "EconomicDispatch" {
  constraint "capacity_limit[a,t]" {
    dispatch[a,t] <= capacity[a,t]
  }
}

minimize "TotalCost" {
  sum(dispatch[a,t] for a in assets for t in time)
}

scenario "Day" {
  horizon steps=24 resolution="PT1H"
  technology "Generator"
  operation "EconomicDispatch"
  minimize "TotalCost"
}
"#;

    let canonical = r#"
model "DayModel" {
  control "dispatch" {
    a
    t
  }

  constraint "capacity_limit[a,t]" {
    dispatch[a,t] <= capacity[a,t]
  }

  minimize "TotalCost" {
    sum(dispatch[a,t] for a in assets for t in time)
  }
}

scenario "Day" {
  horizon steps=24 resolution="PT1H"
  use "DayModel"
}
"#;

    let direct_program = parse_program_text(direct_wiring, &path)?.program;
    let canonical_program = parse_program_text(canonical, &path)?.program;

    let direct_normalized = normalize_program(&direct_program, &path)?;
    let canonical_normalized = normalize_program(&canonical_program, &path)?;

    assert_eq!(direct_normalized, canonical_normalized);
    Ok(())
}

#[test]
fn normalization_equates_name_property_and_positional_forms()
-> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/normalize-name-forms-equivalence.kdl");

    let positional = r#"
model "Dispatch" {
  control "dispatch" {
    a
    t
  }
  minimize "Cost" {
    dispatch[a,t]
  }
}
"#;

    let property = r#"
model name="Dispatch" {
  control name="dispatch" {
    a
    t
  }
  minimize name="Cost" {
    dispatch[a,t]
  }
}
"#;

    let positional_program = parse_program_text(positional, &path)?.program;
    let property_program = parse_program_text(property, &path)?.program;

    let positional_normalized = normalize_program(&positional_program, &path)?;
    let property_normalized = normalize_program(&property_program, &path)?;

    assert_eq!(positional_normalized, property_normalized);
    Ok(())
}

#[test]
fn normalization_equates_math_blocks_and_explicit_expression_properties()
-> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/normalize-math-blocks.kdl");

    let block = r#"
model "Dispatch" {
  control "dispatch" {
    a
    t
  }
  constraint "cap[a,t]" {
    dispatch[a,t] <= capacity[a,t]
  }
  maximize "Serve" {
    sum(dispatch[a,t] for a in assets for t in time)
  }
}
"#;

    let explicit = r#"
model "Dispatch" {
  control "dispatch" {
    a
    t
  }
  constraint "cap[a,t]" expression="dispatch[a,t] <= capacity[a,t]"
  maximize "Serve" expression="sum(dispatch[a,t] for a in assets for t in time)"
}
"#;

    let block_program = parse_program_text(block, &path)?.program;
    let explicit_program = parse_program_text(explicit, &path)?.program;

    let block_normalized = normalize_program(&block_program, &path)?;
    let explicit_normalized = normalize_program(&explicit_program, &path)?;

    assert_eq!(block_normalized, explicit_normalized);
    Ok(())
}
