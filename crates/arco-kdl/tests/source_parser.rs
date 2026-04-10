use arco_kdl::source::{BoundExpr, LiteralValue, ReportKind, SourceError, parse_program_text};
use std::path::PathBuf;

#[test]
fn parses_top_level_low_level_declarations() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("test.kdl");
    let text = r#"
data "generator_data" from="data/generator.csv" {
  map "asset_id" from="asset"
}

subset "solar_assets" from="generator_data"

model "Dispatch" {
  set "time" from="horizon"
  control "dispatch" index_by="asset_id" lower=0
  constraint "limit" {
    dispatch[a] <= 100
  }
  minimize "SystemCost" {
    dispatch[a]
  }
}

scenario "Base" {
  horizon steps=24 resolution="PT1H"
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let program = parsed.program;

    assert_eq!(program.data.len(), 1);
    assert_eq!(program.subsets.len(), 1);
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
  set "asset_id" subset_of="zone" filter_by="is_active" eq=#true
  index "asset_id" "zone_id"

  param "capacity_mw" index_by="asset_id" from="capacity_col" units="MW"
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
    assert_eq!(data.sets[0].subset_of.as_deref(), Some("zone"));
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
  set "time" from="horizon" alias="t"

  param "demand" {
    index "time"
  }
  param "capacity_mw" index_by="assets"

  control "dispatch" index_by="assets" lower=0
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
  control "dispatch" index_by="assets"
  constraint "balance" {
    dispatch[a] <= 100
  }
  minimize "Cost" {
    dispatch[a]
  }
}

scenario "Base" {
  horizon steps=24 resolution="PT1H"
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
fn parses_subset_property_filters_and_comparators() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("test.kdl");
    let text = r#"
subset "solar_north" from="generator_data" class="solar" area="north"
subset "large_units" from="units" filter_by="capacity_mw" geq=200 leq=500
"#;

    let parsed = parse_program_text(text, &path)?;

    assert_eq!(parsed.program.subsets.len(), 2);
    let subset = &parsed.program.subsets[0];
    assert_eq!(subset.source, "generator_data");
    assert_eq!(subset.field_filters.len(), 2);
    assert_eq!(
        subset.field_filters.get("class"),
        Some(&LiteralValue::String("solar".to_string()))
    );

    let bounded = &parsed.program.subsets[1];
    assert_eq!(bounded.filter_by.as_deref(), Some("capacity_mw"));
    assert_eq!(bounded.comparators.geq, Some(LiteralValue::Integer(200)));
    assert_eq!(bounded.comparators.leq, Some(LiteralValue::Integer(500)));

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
fn parses_scenario_without_horizon_block() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("test.kdl");
    let text = r"
model Dispatch {
  control p index_by=g
  maximize Obj {
    p[g]
  }
}

scenario Base {
  use Dispatch
  report Obj
}
";

    let parsed = parse_program_text(text, &path)?;
    let scenario = parsed.program.scenario("Base").ok_or("missing scenario")?;
    assert_eq!(scenario.horizon.steps, 0);
    assert_eq!(scenario.horizon.resolution, "");

    Ok(())
}

#[test]
fn rejects_unsupported_top_level_technology_declaration() {
    let path = PathBuf::from("test.kdl");
    let text = r#"
technology "Battery" {
  control "dispatch"
}
"#;

    let error = parse_program_text(text, &path).expect_err("technology should be unsupported");

    match error {
        SourceError::UnsupportedDeclaration { name, .. } => assert_eq!(name, "technology"),
        _ => panic!("expected UnsupportedDeclaration"),
    }
}
