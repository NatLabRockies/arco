mod common;

use arco_kdl::ObjectiveSense;
use arco_kdl::source::{BoundExpr, LiteralValue, ReportKind, SourceError, parse_program_text};
use common::fixture_text;
use std::path::PathBuf;

#[test]
fn parses_top_level_low_level_declarations() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("test.kdl");
    let text = fixture_text("parses_top_level_low_level_declarations.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text = fixture_text("parses_data_children_including_index_forms.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text = fixture_text("parses_model_children_with_indexing_and_algebra.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
    let model = parsed.program.model("Dispatch").ok_or("missing model")?;

    assert_eq!(model.parameters[0].indices, vec!["time"]);
    assert_eq!(model.parameters[1].indices, vec!["assets"]);
    assert_eq!(model.controls[0].indices.len(), 1);
    assert_eq!(model.controls[0].indices[0].name, "assets");
    assert_eq!(model.controls[1].indices[0].name, "assets");
    assert_eq!(model.controls[1].indices[1].name, "time");
    assert_eq!(model.expressions[0].name, "FuelCost");
    assert_eq!(model.constraints[0].name, "balance");
    assert_eq!(model.optimize.sense, ObjectiveSense::Maximize);
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
    let text = fixture_text("parses_scenario_reports_scalar_and_dual.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text =
        fixture_text("parses_generated_constraint_with_index_if_and_expression_children.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
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
fn parses_top_level_projection_declaration() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("test.kdl");
    let text = fixture_text("parses_top_level_projection_declaration.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
    assert_eq!(parsed.program.projections.len(), 1);

    let projection = &parsed.program.projections[0];
    assert_eq!(projection.name, "ai");
    assert_eq!(projection.from_domain, "feasible_links");
    assert_eq!(projection.to_keys, vec!["a", "i"]);

    Ok(())
}

#[test]
fn parses_top_level_projection_declaration_with_domain_and_key_blocks()
-> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("test.kdl");
    let text =
        fixture_text("parses_top_level_projection_declaration_with_domain_and_key_blocks.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
    assert_eq!(parsed.program.projections.len(), 1);

    let projection = &parsed.program.projections[0];
    assert_eq!(projection.name, "ai");
    assert_eq!(projection.from_domain, "feasible_links");
    assert_eq!(projection.to_keys, vec!["a", "i"]);

    Ok(())
}

#[test]
fn parses_expression_reduce_projection_block_form() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("test.kdl");
    let text = fixture_text("parses_expression_reduce_projection_block_form.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
    let model = parsed.program.model("Dispatch").ok_or("missing model")?;
    let expression = model.expressions.first().ok_or("missing expression")?;

    assert!(expression.abstraction.is_some());
    assert!(expression.formula.contains("__reduce_projection__"));

    Ok(())
}

#[test]
fn rejects_expression_reduce_with_mixed_sibling_math() {
    let path = PathBuf::from("test.kdl");
    let text = fixture_text("rejects_expression_reduce_with_mixed_sibling_math.kdl")
        .expect("fixture should load");

    let error = parse_program_text(&text, &path)
        .expect_err("mixed reduce and sibling math should fail parse");
    assert!(error.to_string().contains("expression"));
}

#[test]
fn rejects_expression_with_prefixed_math_before_reduce() {
    let path = PathBuf::from("test.kdl");
    let text = fixture_text("rejects_expression_with_prefixed_math_before_reduce.kdl")
        .expect("fixture should load");

    let error = parse_program_text(&text, &path)
        .expect_err("prefixed math before reduce should fail parse");
    assert!(error.to_string().contains("expression"));
}

#[test]
fn rejects_unsupported_top_level_declarations() {
    let path = PathBuf::from("test.kdl");
    let cases = [
        (
            "technology",
            "rejects_unsupported_top_level_declarations_technology.kdl",
        ),
        (
            "operation",
            "rejects_unsupported_top_level_declarations_operation.kdl",
        ),
        (
            "asset",
            "rejects_unsupported_top_level_declarations_asset.kdl",
        ),
        (
            "instances",
            "rejects_unsupported_top_level_declarations_instances.kdl",
        ),
        (
            "rule",
            "rejects_unsupported_top_level_declarations_rule.kdl",
        ),
        (
            "expression",
            "rejects_unsupported_top_level_declarations_expression.kdl",
        ),
        (
            "minimize",
            "rejects_unsupported_top_level_declarations_minimize.kdl",
        ),
        (
            "maximize",
            "rejects_unsupported_top_level_declarations_maximize.kdl",
        ),
        (
            "subset",
            "rejects_unsupported_top_level_declarations_subset.kdl",
        ),
    ];

    for (decl, fixture) in cases {
        let text = fixture_text(fixture).expect("fixture should load");
        let error = parse_program_text(&text, &path)
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
    let text = fixture_text("rejects_legacy_index_by_property.kdl").expect("fixture should load");

    let error = parse_program_text(&text, &path)
        .expect_err("legacy index_by should be rejected at parse time");
    assert!(error.to_string().contains("index_by"));
}

#[test]
fn rejects_unsupported_scenario_horizon_and_set_binding() {
    let path = PathBuf::from("test.kdl");
    let text = fixture_text("rejects_unsupported_scenario_horizon_and_set_binding.kdl")
        .expect("fixture should load");

    let error = parse_program_text(&text, &path)
        .expect_err("scenario-level horizon should be rejected at parse time");

    match error {
        SourceError::UnsupportedDeclaration { name, .. } => assert_eq!(name, "horizon"),
        other => panic!("expected UnsupportedDeclaration, got {other:?}"),
    }
}
