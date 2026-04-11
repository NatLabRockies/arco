use arco_kdl::source::{BoundExpr, LiteralValue, ReportKind, SourceError, parse_program_text};
use std::path::PathBuf;

#[test]
fn parses_top_level_low_level_declarations() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("test.kdl");
    let text = r#"
param "voll" 9000 units="$/MWh"

set "time" { "1"; "2"; "3" }

data "generator_data" from="data/generator.csv" {
  map "asset_id" from="asset"
}

model "Dispatch" {
  control "dispatch" index="asset_id" lower=0
  constraint "limit" {
    dispatch[a] <= 100
  }
  minimize "SystemCost" {
    dispatch[a]
  }
}

scenario "Base" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let program = parsed.program;

    assert_eq!(program.params.len(), 1);
    assert_eq!(program.sets.len(), 1);
    assert_eq!(program.data.len(), 1);
    assert_eq!(program.models.len(), 1);
    assert_eq!(program.scenarios.len(), 1);

    Ok(())
}

#[test]
fn parses_data_children_including_index_forms() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("test.kdl");
    let text = r#"
data "generator_data" from="data/generator.csv" {
  map "asset_id" from="asset"
  set "asset_id"
  index "asset_id" "zone_id"

  param "capacity_mw" index="asset_id" from="capacity_col" units="MW"
  param "availability" {
    index "asset_id"
    index "time"
    reduce "sum"
  }
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let data = &parsed.program.data[0];

    assert_eq!(data.maps[0].name, "asset_id");
    assert_eq!(data.maps[0].source.as_deref(), Some("asset"));
    assert_eq!(data.sets[0].name, "asset_id");
    assert_eq!(data.indices[0].columns, vec!["asset_id", "zone_id"]);

    assert_eq!(data.parameters[0].name, "capacity_mw");
    assert_eq!(data.parameters[0].from.as_deref(), Some("capacity_col"));
    assert_eq!(data.parameters[0].units.as_deref(), Some("MW"));
    assert_eq!(data.parameters[0].indices, vec!["asset_id"]);

    assert_eq!(data.parameters[1].name, "availability");
    assert_eq!(data.parameters[1].indices, vec!["asset_id", "time"]);
    assert_eq!(data.parameters[1].reduce.as_deref(), Some("sum"));

    Ok(())
}

#[test]
fn parses_model_children_with_indexing_and_algebra() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("test.kdl");
    let text = r#"
model "Dispatch" {
  set "assets" alias="a"
  set "time" alias="t"

  param "demand" {
    index "time"
  }
  param "capacity_mw" index="assets"

  control "dispatch" index="assets" lower=0
  control "on" {
    index "assets"
    index "time"
  }

  expression "FuelCost" {
    sum(dispatch[a] for a in assets)
  }

  constraint "balance" {
    sum(dispatch[a] for a in assets) = demand[t]
  }

  maximize "Revenue" {
    FuelCost
  }
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let model = parsed.program.model("Dispatch").ok_or("missing model")?;

    assert_eq!(model.parameters[0].indices, vec!["time"]);
    assert_eq!(model.parameters[1].indices, vec!["assets"]);
    assert_eq!(model.controls[0].indices.len(), 1);
    assert_eq!(model.controls[0].indices[0].name, "assets");
    assert_eq!(model.controls[1].indices[0].name, "assets");
    assert_eq!(model.controls[1].indices[1].name, "time");
    assert_eq!(model.expressions[0].name, "FuelCost");
    assert_eq!(model.constraints[0].name, "balance");
    assert_eq!(model.optimize.sense, "maximize");
    assert_eq!(model.optimize.expression, "FuelCost");
    assert_eq!(
        model.controls[0].lower,
        Some(BoundExpr::Literal(LiteralValue::Integer(0)))
    );

    Ok(())
}

#[test]
fn parses_scenario_reports_scalar_and_dual() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("test.kdl");
    let text = r#"
model "Dispatch" {
  set "assets"
  control "dispatch" index="assets"
  constraint "balance" {
    dispatch[a] <= 100
  }
  minimize "Cost" {
    dispatch[a]
  }
}

scenario "Base" {
  use "Dispatch"
  data "demand" from="data/demand.csv"
  report FuelCost
  report dual balance
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let scenario = parsed.program.scenario("Base").ok_or("missing scenario")?;

    assert_eq!(scenario.reports.len(), 2);
    assert_eq!(scenario.reports[0].kind, ReportKind::Scalar);
    assert_eq!(scenario.reports[0].target, "FuelCost");
    assert_eq!(scenario.reports[1].kind, ReportKind::Dual);
    assert_eq!(scenario.reports[1].target, "balance");

    Ok(())
}

#[test]
fn parses_generated_constraint_with_index_if_and_expression_children()
-> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("test.kdl");
    let text = r"
model Dispatch {
  control p {
    index g
    index t
  }

  constraint ramp {
    index g
    index tt { in t }
    if { tt > 1 }
    expression {
      p[g,tt] - p[g,tt-1] <= 10
    }
  }

  minimize Obj {
    sum(p[g,t] for g in g for t in t)
  }
}

scenario Base {
  use Dispatch
}
";

    let parsed = parse_program_text(text, &path)?;
    let model = parsed.program.model("Dispatch").ok_or("missing model")?;
    let constraint = model
        .constraints
        .iter()
        .find(|constraint| constraint.name == "ramp")
        .ok_or("missing ramp constraint")?;

    assert_eq!(constraint.generation_bindings.len(), 2);
    assert_eq!(constraint.generation_bindings[0].variable, "g");
    assert_eq!(constraint.generation_bindings[0].domain, "g");
    assert_eq!(constraint.generation_bindings[1].variable, "tt");
    assert_eq!(constraint.generation_bindings[1].domain, "t");
    assert_eq!(constraint.generation_filter.as_deref(), Some("tt > 1"));

    Ok(())
}

#[test]
fn rejects_unsupported_top_level_declarations() {
    let path = PathBuf::from("test.kdl");
    let cases = [
        ("technology", "technology Battery { control dispatch }"),
        ("operation", "operation Dispatch { }"),
        ("asset", "asset Gen { }"),
        ("instances", "instances Fleet from=\"data.csv\" { }"),
        ("rule", "rule Balance { }"),
        ("expression", "expression Cost { 1 }"),
        ("minimize", "minimize Obj { 1 }"),
        ("maximize", "maximize Obj { 1 }"),
        ("subset", "subset legacy from=\"x\""),
    ];

    for (decl, text) in cases {
        let error = parse_program_text(text, &path)
            .expect_err("unsupported declaration should be rejected at parse time");
        match error {
            SourceError::UnsupportedDeclaration { name, .. } => assert_eq!(name, decl),
            other => panic!("expected UnsupportedDeclaration for {decl}, got {other:?}"),
        }
    }
}

#[test]
fn rejects_legacy_index_by_property() {
    let path = PathBuf::from("test.kdl");
    let text = r#"
model "Dispatch" {
  param "demand" index_by="t"
  control "x" index="t"
  minimize "Obj" { x[t] }
}
scenario "Base" { use "Dispatch" }
"#;

    let error = parse_program_text(text, &path)
        .expect_err("legacy index_by should be rejected at parse time");
    assert!(error.to_string().contains("index_by"));
}

#[test]
fn rejects_unsupported_scenario_horizon_and_set_binding() {
    let path = PathBuf::from("test.kdl");
    let text = r#"
model "Dispatch" {
  control "p" index="g"
  maximize "Obj" { p[g] }
}

scenario "Base" {
  use "Dispatch"
  horizon steps=24 resolution="PT1H"
}
"#;

    let error = parse_program_text(text, &path)
        .expect_err("scenario-level horizon should be rejected at parse time");

    match error {
        SourceError::UnsupportedDeclaration { name, .. } => assert_eq!(name, "horizon"),
        other => panic!("expected UnsupportedDeclaration, got {other:?}"),
    }
}
