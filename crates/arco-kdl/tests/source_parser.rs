mod common;

use arco_kdl::source::{
    parse_program_file, parse_program_text, BoundExpr, LiteralValue, ParsedSource, ReportKind,
    SourceError,
};
use arco_kdl::ObjectiveSense;
use common::{fixture_path, fixture_text};
use std::path::PathBuf;

fn parse_fixture(name: &str) -> Result<ParsedSource, Box<dyn std::error::Error>> {
    let path = PathBuf::from("test.kdl");
    let text = fixture_text(name)?;
    Ok(parse_program_text(&text, &path)?)
}

fn parse_fixture_error(name: &str, context: &str) -> SourceError {
    let path = PathBuf::from("test.kdl");
    let text = fixture_text(name).expect("fixture should load");
    parse_program_text(&text, &path).expect_err(context)
}

fn parse_file_fixture_error(name: &str, context: &str) -> SourceError {
    parse_program_file(&fixture_path(name)).expect_err(context)
}

#[test]
fn parses_top_level_low_level_declarations() -> Result<(), Box<dyn std::error::Error>> {
    let program = parse_fixture("parses_top_level_low_level_declarations.kdl")?.program;

    assert_eq!(program.params.len(), 1);
    assert_eq!(program.sets.len(), 1);
    assert_eq!(program.data.len(), 1);
    assert_eq!(program.models.len(), 1);
    assert_eq!(program.scenarios.len(), 1);

    Ok(())
}

#[test]
fn parses_data_children_including_index_forms() -> Result<(), Box<dyn std::error::Error>> {
    let parsed = parse_fixture("parses_data_children_including_index_forms.kdl")?;
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
    let parsed = parse_fixture("parses_model_children_with_indexing_and_algebra.kdl")?;
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
    let parsed = parse_fixture("parses_scenario_reports_scalar_and_dual.kdl")?;
    let scenario = parsed.program.scenario("Base").ok_or("missing scenario")?;

    assert_eq!(scenario.reports.len(), 2);
    assert_eq!(scenario.reports[0].kind, ReportKind::Scalar);
    assert_eq!(scenario.reports[0].target, "FuelCost");
    assert_eq!(scenario.reports[1].kind, ReportKind::Dual);
    assert_eq!(scenario.reports[1].target, "balance");

    Ok(())
}

#[test]
fn parses_generated_constraint_with_index_if_and_expression_children(
) -> Result<(), Box<dyn std::error::Error>> {
    let parsed =
        parse_fixture("parses_generated_constraint_with_index_if_and_expression_children.kdl")?;
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
    let parsed = parse_fixture("parses_top_level_projection_declaration.kdl")?;
    assert_eq!(parsed.program.projections.len(), 1);

    let projection = &parsed.program.projections[0];
    assert_eq!(projection.name, "ai");
    assert_eq!(projection.from_domain, "feasible_links");
    assert_eq!(projection.to_keys, vec!["a", "i"]);

    Ok(())
}

#[test]
fn parses_top_level_projection_declaration_with_domain_and_key_blocks(
) -> Result<(), Box<dyn std::error::Error>> {
    let parsed =
        parse_fixture("parses_top_level_projection_declaration_with_domain_and_key_blocks.kdl")?;
    assert_eq!(parsed.program.projections.len(), 1);

    let projection = &parsed.program.projections[0];
    assert_eq!(projection.name, "ai");
    assert_eq!(projection.from_domain, "feasible_links");
    assert_eq!(projection.to_keys, vec!["a", "i"]);

    Ok(())
}

#[test]
fn parses_expression_reduce_projection_block_form() -> Result<(), Box<dyn std::error::Error>> {
    let parsed = parse_fixture("parses_expression_reduce_projection_block_form.kdl")?;
    let model = parsed.program.model("Dispatch").ok_or("missing model")?;
    let expression = model.expressions.first().ok_or("missing expression")?;

    assert!(expression.abstraction.is_some());
    assert!(expression.formula.contains("__reduce_projection__"));

    Ok(())
}

#[test]
fn parses_generated_expression_with_index_and_expression_children(
) -> Result<(), Box<dyn std::error::Error>> {
    let parsed =
        parse_fixture("parses_generated_expression_with_index_and_expression_children.kdl")?;
    let model = parsed.program.model("Dispatch").ok_or("missing model")?;
    let expression = model
        .expressions
        .iter()
        .find(|expr| expr.name == "net_injection_by_bus")
        .ok_or("missing net_injection_by_bus expression")?;

    assert_eq!(expression.generation_bindings.len(), 1);
    assert_eq!(expression.generation_bindings[0].variable, "b");
    assert_eq!(expression.generation_bindings[0].domain, "bus");
    assert!(expression.generation_filter.is_none());

    Ok(())
}

#[test]
fn rejects_generated_expression_with_multiple_if_children() {
    let error = parse_fixture_error(
        "parses_generated_expression_with_multiple_if_filters_preserves_grouping.kdl",
        "generated expression should reject multiple if children",
    );

    assert!(error
        .to_string()
        .contains("expression declarations support at most one `if` child"));
}

#[test]
fn rejects_generated_expression_with_conflicting_formula_sources() {
    let error = parse_fixture_error(
        "rejects_generated_expression_with_conflicting_formula_sources.kdl",
        "generated expression should reject conflicting formula sources",
    );

    assert!(error
        .to_string()
        .contains("expression declarations support only one formula source"));
}

#[test]
fn parse_program_file_expands_top_level_and_model_includes(
) -> Result<(), Box<dyn std::error::Error>> {
    let parsed = parse_program_file(&fixture_path("composition/input.kdl"))?;
    let program = parsed.program;

    assert_eq!(program.sets.len(), 1);
    assert_eq!(program.sets[0].name, "time");
    assert_eq!(program.data.len(), 1);
    assert_eq!(program.models.len(), 1);

    let model = &program.models[0];
    assert!(model.includes.is_empty());
    assert_eq!(model.controls.len(), 1);
    assert_eq!(model.controls[0].name, "x");
    assert_eq!(model.constraints.len(), 1);
    assert_eq!(model.constraints[0].name, "limit");
    assert_eq!(model.optimize.name, "TotalCost");
    assert_eq!(program.scenarios.len(), 1);

    Ok(())
}

#[test]
fn parse_program_text_preserves_include_declarations_without_expanding(
) -> Result<(), Box<dyn std::error::Error>> {
    let text = fixture_text("composition/input.kdl")?;
    let parsed = parse_program_text(&text, &PathBuf::from("input.kdl"))?;

    assert_eq!(parsed.program.includes.len(), 1);
    assert_eq!(parsed.program.includes[0].path, "shared.kdl");
    assert_eq!(parsed.program.models[0].includes.len(), 1);
    assert_eq!(
        parsed.program.models[0].includes[0].path,
        "dispatch-fragment.kdl"
    );

    Ok(())
}

#[test]
fn parse_program_file_rejects_nested_includes() {
    let error = parse_file_fixture_error(
        "composition/nested-entry.kdl",
        "nested includes should be rejected",
    );

    assert!(matches!(error, SourceError::InvalidInclude { .. }));
    assert!(error.to_string().contains("include"));
}

#[test]
fn parse_program_file_rejects_scenarios_from_included_files() {
    let error = parse_file_fixture_error(
        "composition/scenario-entry.kdl",
        "included scenarios should be rejected",
    );

    assert!(matches!(error, SourceError::InvalidInclude { .. }));
    assert!(error.to_string().contains("scenario"));
}

#[test]
fn parse_program_file_rejects_top_level_declarations_in_model_includes() {
    let error = parse_file_fixture_error(
        "composition/model-include-data-entry.kdl",
        "model include should reject top-level data declarations",
    );

    assert!(matches!(error, SourceError::InvalidInclude { .. }));
    assert!(error.to_string().contains("model-scope include"));
}

#[test]
fn parse_program_file_reports_missing_include_io_path() {
    let error = parse_file_fixture_error(
        "composition/missing-include-entry.kdl",
        "missing include should report an io error",
    );

    match error {
        SourceError::Io { path, .. } => assert!(path.ends_with("missing.kdl")),
        other => panic!("expected missing include io error, got {other:?}"),
    }
}

#[test]
fn parse_program_text_rejects_absolute_top_level_include_paths() {
    let text = r#"
include "/tmp/shared.kdl"

model "Dispatch" {
  control "x" lower=0
  minimize "TotalCost" {
    x
  }
}

scenario "Base" {
  use "Dispatch"
}
"#;

    let error = parse_program_text(text, &PathBuf::from("input.kdl"))
        .expect_err("absolute top-level include path should be rejected");

    assert!(matches!(error, SourceError::InvalidInclude { .. }));
    assert!(error.to_string().contains("relative"));
}

#[test]
fn parse_program_text_rejects_absolute_model_include_paths() {
    let text = r#"
model "Dispatch" {
  include "/tmp/fragment.kdl"

  control "x" lower=0
  minimize "TotalCost" {
    x
  }
}

scenario "Base" {
  use "Dispatch"
}
"#;

    let error = parse_program_text(text, &PathBuf::from("input.kdl"))
        .expect_err("absolute model include path should be rejected");

    assert!(matches!(error, SourceError::InvalidInclude { .. }));
    assert!(error.to_string().contains("relative"));
}

#[test]
fn rejects_expression_reduce_with_mixed_sibling_math() {
    let error = parse_fixture_error(
        "rejects_expression_reduce_with_mixed_sibling_math.kdl",
        "mixed reduce and sibling math should fail parse",
    );
    assert!(error.to_string().contains("expression"));
}

#[test]
fn rejects_expression_with_prefixed_math_before_reduce() {
    let error = parse_fixture_error(
        "rejects_expression_with_prefixed_math_before_reduce.kdl",
        "prefixed math before reduce should fail parse",
    );
    assert!(error.to_string().contains("expression"));
}

#[test]
fn rejects_unsupported_top_level_declarations() {
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
        let error = parse_fixture_error(
            fixture,
            "unsupported declaration should be rejected at parse time",
        );
        match error {
            SourceError::UnsupportedDeclaration { name, .. } => assert_eq!(name, decl),
            other => panic!("expected UnsupportedDeclaration for {decl}, got {other:?}"),
        }
    }
}

#[test]
fn rejects_legacy_index_by_property() {
    let error = parse_fixture_error(
        "rejects_legacy_index_by_property.kdl",
        "legacy index_by should be rejected at parse time",
    );
    assert!(error.to_string().contains("index_by"));
}
