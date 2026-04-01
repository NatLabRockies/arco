use arco_kdl::source::{
    BoundExpr, ColumnMappingDecl, GenerationBinding, IndexDecl, LiteralValue, NamedVariableDecl,
    VariableKindDecl, parse_program_file, parse_program_text,
};
use std::path::PathBuf;

/// Shorthand to build an `IndexDecl` without a domain binding.
fn idx(name: &str) -> IndexDecl {
    IndexDecl {
        name: name.to_string(),
        domain: None,
    }
}

#[test]
fn parses_price_taker_battery_fixture_into_typed_declarations()
-> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("e2e")
        .join("price-taker-battery")
        .join("input.kdl");

    let parsed = parse_program_file(&path)?;
    let program = parsed.program;

    assert_eq!(program.technologies.len(), 1);
    assert_eq!(program.operations.len(), 1);
    assert_eq!(program.assets.len(), 1);
    assert_eq!(program.scenarios.len(), 1);

    let technology = program.technology("Battery").ok_or("missing technology")?;
    let control_names: Vec<&str> = technology
        .controls
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    assert_eq!(control_names, vec!["charge", "discharge"]);
    assert_eq!(technology.states, vec!["soc".to_string()]);

    let scenario = program
        .scenario("BatteryArbitrageDay")
        .ok_or("missing scenario")?;
    assert_eq!(scenario.technologies, vec!["Battery".to_string()]);
    assert_eq!(scenario.operations, vec!["PriceTakerBattery".to_string()]);
    assert_eq!(
        scenario.reports,
        vec![arco_kdl::source::ReportDecl {
            kind: arco_kdl::source::ReportKind::Scalar,
            target: "ArbitrageRevenue".to_string(),
        }]
    );

    Ok(())
}

#[test]
fn parses_constraint_math_blocks_with_named_and_unnamed_constraints()
-> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("inline-constraint-blocks.kdl");
    let text = r#"
operation "BatteryOperation" {
  constraint name="soc_balance" {
    soc[a,t] = soc[a,t-1] + charge_efficiency[a] * charge[a,t] - discharge[a,t] / discharge_efficiency[a]
  }

  constraint {
    charge[a,t] <= power_mw[a]
  }
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let program = parsed.program;
    let operation = program
        .operation("BatteryOperation")
        .ok_or("missing operation")?;

    assert_eq!(operation.constraints.len(), 2);
    assert_eq!(operation.constraints[0].name, "soc_balance");
    assert_eq!(
        operation.constraints[0].expression,
        "soc[a,t] = soc[a,t-1] + charge_efficiency[a] * charge[a,t] - discharge[a,t] / discharge_efficiency[a]"
    );
    assert_eq!(operation.constraints[1].name, "constraint_2");
    assert_eq!(
        operation.constraints[1].expression,
        "charge[a,t] <= power_mw[a]"
    );

    Ok(())
}

#[test]
fn parses_constraint_filters_into_typed_declarations() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("constraint-filters.kdl");
    let text = r#"
operation "ExpansionDispatch" {
  constraint "dispatch_limit" if="apply_dispatch_limit[a] == 1" {
    dispatch[a,t] <= existing_capacity[a]
  }
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let program = parsed.program;
    let operation = program
        .operation("ExpansionDispatch")
        .ok_or("missing operation")?;

    assert_eq!(operation.constraints.len(), 1);
    assert_eq!(
        operation.constraints[0].generation_filter.as_deref(),
        Some("apply_dispatch_limit[a] == 1")
    );
    assert_eq!(
        operation.constraints[0]
            .parsed_generation_filter
            .as_ref()
            .ok_or("missing parsed filter")?
            .to_string(),
        "apply_dispatch_limit[a] == 1"
    );

    Ok(())
}

#[test]
fn parses_expression_math_blocks_without_formula_wrapper() -> Result<(), Box<dyn std::error::Error>>
{
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("inline-expression-blocks.kdl");
    let text = r#"
expression "ArbitrageRevenue" {
  sum(prices[t] * (discharge[a,t] - charge[a,t]) for a in assets for t in time)
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let program = parsed.program;
    let expression = program
        .expression("ArbitrageRevenue")
        .ok_or("missing expression")?;

    assert_eq!(
        expression.formula,
        "sum(prices[t] * (discharge[a,t] - charge[a,t]) for a in assets for t in time)"
    );

    Ok(())
}

#[test]
fn parses_verbose_name_properties_for_common_declarations() -> Result<(), Box<dyn std::error::Error>>
{
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("verbose-name-properties.kdl");
    let text = r#"
technology name="Battery" {
  control name="charge"
  state name="soc"
}

operation name="BatteryDispatch" {
  constraint name="charge_limit" {
    charge[a,t] <= power_mw[a]
  }
}

asset name="Battery1" {
  technology name="Battery"
  operation name="BatteryDispatch"
  power_mw 100
}

expression name="Revenue" {
  charge[a,t]
}

maximize name="Profit" {
  Revenue
}

scenario name="Case" {
  horizon steps=1 resolution="PT1H"
  asset name="Battery1"
  maximize name="Profit"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let program = parsed.program;

    let technology = program.technology("Battery").ok_or("missing technology")?;
    assert_eq!(technology.controls.len(), 1);
    assert_eq!(technology.controls[0].name, "charge");
    assert_eq!(technology.states, vec!["soc".to_string()]);

    let operation = program
        .operation("BatteryDispatch")
        .ok_or("missing operation")?;
    assert_eq!(operation.constraints.len(), 1);
    assert_eq!(operation.constraints[0].name, "charge_limit");
    assert_eq!(
        operation.constraints[0].expression,
        "charge[a,t] <= power_mw[a]"
    );

    let asset = program.asset("Battery1").ok_or("missing asset")?;
    assert_eq!(asset.technology, "Battery");
    assert_eq!(asset.operation.as_deref(), Some("BatteryDispatch"));
    assert_eq!(
        asset.parameters.get("power_mw"),
        Some(&LiteralValue::Integer(100))
    );

    let expression = program.expression("Revenue").ok_or("missing expression")?;
    assert_eq!(expression.formula, "charge[a,t]");

    let objective = program.objective("Profit").ok_or("missing objective")?;
    assert_eq!(objective.sense, "maximize");
    assert_eq!(objective.expression, "Revenue");

    let scenario = program.scenario("Case").ok_or("missing scenario")?;
    assert_eq!(scenario.horizon.steps, 1);
    assert_eq!(scenario.assets, vec!["Battery1".to_string()]);
    assert_eq!(scenario.objective.as_deref(), Some("Profit"));

    Ok(())
}

#[test]
fn parses_generalized_network_fixture_into_ast_declarations()
-> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("e2e")
        .join("generalized-two-node-network")
        .join("input.kdl");

    let parsed = parse_program_file(&path)?;
    let program = parsed.program;

    let operation = program
        .operation("Transmission")
        .ok_or("missing transmission operation")?;
    let rule = program.rule("NodeBalance").ok_or("missing balance rule")?;
    let objective = program.objective("TotalCost").ok_or("missing objective")?;

    assert_eq!(
        operation.constraints[0].parsed_expression.to_string(),
        "-thermal_limit_mw[l] <= flow[l,t] <= thermal_limit_mw[l]"
    );
    assert_eq!(
        rule.constraints[0].parsed_expression.to_string(),
        "sum(dispatch[g,t] for g in generators if generator_node[g] == n) + sum(flow[l,t] for l in incoming_lines if line_to[l] == n) - sum(flow[l,t] for l in outgoing_lines if line_from[l] == n) = demand[n,t]"
    );
    assert_eq!(
        objective.parsed_expression.to_string(),
        "GenerationCost + LossPenalty"
    );

    Ok(())
}

#[test]
fn parses_canonical_model_with_balanced_syntax() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("canonical-model.kdl");
    let text = r#"
model "EconomicDispatch" {
  set "generators"
  set "time" from="horizon"

  param "capacity_mw" {
    g
  }
  param "availability" {
    g
    t
  }
  param "marginal_cost" {
    g
  }
  param "demand" {
    t
  }

  control "dispatch" lower=0 {
    a
    t
  }

  constraint "capacity_limit[g in generators, t in time]" {
    dispatch[g,t] <= capacity_mw[g] * availability[g,t]
  }

  constraint "balance[t in time]" {
    sum(dispatch[g,t] for g in generators) = demand[t]
  }

  minimize "TotalCost" {
    sum(marginal_cost[g] * dispatch[g,t] for g in generators for t in time)
  }
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let program = parsed.program;
    let model = program.model("EconomicDispatch").ok_or("missing model")?;

    assert_eq!(model.sets[0].name, "generators");
    assert_eq!(model.sets[0].source, None);
    assert_eq!(model.sets[1].name, "time");
    assert_eq!(model.sets[1].source.as_deref(), Some("horizon"));

    let param_names: Vec<&str> = model.parameters.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(
        param_names,
        &["capacity_mw", "availability", "marginal_cost", "demand"]
    );

    assert_eq!(model.controls[0].name, "dispatch");
    assert_eq!(
        model.controls[0].lower,
        Some(BoundExpr::Literal(LiteralValue::Integer(0)))
    );
    assert_eq!(model.controls[0].indices, vec![idx("a"), idx("t")]);
    let constraint_names: Vec<&str> = model.constraints.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        constraint_names,
        &[
            "capacity_limit[g in generators, t in time]",
            "balance[t in time]",
        ]
    );

    assert_eq!(model.optimize.name, "TotalCost");
    assert_eq!(model.optimize.sense, "minimize");
    assert!(
        model
            .optimize
            .expression
            .contains("marginal_cost[g] * dispatch[g,t]")
    );

    Ok(())
}

#[test]
fn parses_scenario_use_of_model() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("canonical-scenario-use.kdl");
    let text = r#"
model "EconomicDispatch" {
  set "generators"
  set "time" from="horizon"

  param "demand" {
    t
  }
  control "dispatch" lower=0 {
    a
    t
  }

  constraint "balance[t in time]" {
    sum(dispatch[g,t] for g in generators) = demand[t]
  }

  maximize "ServeDemand" {
    sum(dispatch[g,t] for g in generators for t in time)
  }
}

scenario "Day" {
  horizon steps=24 resolution="PT1H"
  use "EconomicDispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let scenario = parsed.program.scenario("Day").ok_or("missing scenario")?;

    assert_eq!(scenario.model_use.as_deref(), Some("EconomicDispatch"));

    Ok(())
}

#[test]
fn parses_data_binding_with_positional_name_and_from_property()
-> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data-binding-positional.kdl");
    let text = r#"
scenario "Base" {
  horizon steps=24 resolution="PT1H"
  data "demand" from="data/demand.csv"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let scenario = parsed.program.scenario("Base").ok_or("missing scenario")?;

    assert_eq!(scenario.data.len(), 1);
    assert_eq!(scenario.data[0].name, "demand");
    assert_eq!(scenario.data[0].source, "data/demand.csv");

    Ok(())
}

#[test]
fn parses_report_positional_syntax_in_scenario() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("report-positional-syntax.kdl");
    let text = r#"
model "EconomicDispatch" {
  set "generators"
  set "time" from="horizon"

  param "demand" {
    t
  }
  control "dispatch" lower=0 {
    a
    t
  }

  constraint "balance[t in time]" {
    sum(dispatch[g,t] for g in generators) = demand[t]
  }

  maximize "ServeDemand" {
    sum(dispatch[g,t] for g in generators for t in time)
  }
}

scenario "Day" {
  horizon steps=24 resolution="PT1H"
  use "EconomicDispatch"
  report FuelCost
  report StartupCost
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let scenario = parsed.program.scenario("Day").ok_or("missing scenario")?;

    assert_eq!(
        scenario.reports,
        vec![
            arco_kdl::source::ReportDecl {
                kind: arco_kdl::source::ReportKind::Scalar,
                target: "FuelCost".to_string(),
            },
            arco_kdl::source::ReportDecl {
                kind: arco_kdl::source::ReportKind::Scalar,
                target: "StartupCost".to_string(),
            },
        ]
    );

    Ok(())
}

#[test]
fn parses_instances_map_syntax_for_column_mappings() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("instances-map-syntax.kdl");
    let text = r#"
technology "GasCT" {
  control "dispatch"
}

operation "SimpleDispatch" {
  constraint "dispatch_limit" {
    dispatch[a,t] <= capacity[a]
  }
}

instances "Fleet" from="data/fleet.csv" {
  technology "GasCT"
  operation "SimpleDispatch"
  map "name" from="asset_name"
  map "capacity" from="capacity_mw"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let instances = parsed
        .program
        .instances
        .first()
        .ok_or("missing instances")?;

    assert_eq!(instances.name, "Fleet");
    assert_eq!(instances.source, "data/fleet.csv");
    assert_eq!(
        instances.columns,
        vec![
            ColumnMappingDecl {
                source: "asset_name".to_string(),
                target: "name".to_string(),
            },
            ColumnMappingDecl {
                source: "capacity_mw".to_string(),
                target: "capacity".to_string(),
            },
        ]
    );

    Ok(())
}

#[test]
fn parses_minimize_directive_in_scenario_as_objective() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("scenario-minimize.kdl");
    let text = r#"
model "EconomicDispatch" {
  set "generators"
  set "time" from="horizon"

  param "demand" {
    t
  }
  control "dispatch" lower=0 {
    a
    t
  }

  constraint "balance[t in time]" {
    sum(dispatch[g,t] for g in generators) = demand[t]
  }

  minimize "C" {
    sum(dispatch[g,t] for g in generators for t in time)
  }
}

scenario "Day" {
  horizon steps=24 resolution="PT1H"
  use "EconomicDispatch"
  minimize "C"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let scenario = parsed.program.scenario("Day").ok_or("missing scenario")?;

    assert_eq!(scenario.objective.as_deref(), Some("C"));

    Ok(())
}

#[test]
fn parses_top_level_minimize_into_objective() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("test.kdl");
    let text = r#"
minimize "SystemCost" { FuelCost }
"#;

    let parsed = parse_program_text(text, &path)?;
    let program = parsed.program;

    assert_eq!(program.objectives.len(), 1);

    let obj = &program.objectives[0];
    assert_eq!(obj.name, "SystemCost");
    assert_eq!(obj.sense, "minimize");
    assert_eq!(obj.expression, "FuelCost");

    Ok(())
}

#[test]
fn parses_top_level_maximize_into_objective() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("test.kdl");
    let text = r#"
maximize "TotalProfit" { Revenue - Cost }
"#;

    let parsed = parse_program_text(text, &path)?;
    let program = parsed.program;

    assert_eq!(program.objectives.len(), 1);

    let obj = &program.objectives[0];
    assert_eq!(obj.name, "TotalProfit");
    assert_eq!(obj.sense, "maximize");
    assert_eq!(obj.expression, "Revenue - Cost");

    Ok(())
}

#[test]
fn parses_scenario_with_direct_technology_operation_rule_wiring()
-> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("test.kdl");
    let text = r#"
scenario "Wired" {
  horizon steps=24 resolution="PT1H"
  technology "Generator"
  operation "Dispatch"
  rule "Balance"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let scenario = parsed.program.scenario("Wired").ok_or("missing scenario")?;

    assert_eq!(scenario.technologies, vec!["Generator".to_string()]);
    assert_eq!(scenario.operations, vec!["Dispatch".to_string()]);
    assert_eq!(scenario.rules, vec!["Balance".to_string()]);

    Ok(())
}

#[test]
fn parses_top_level_set_declarations() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("test.kdl");
    let text = r#"
set "generators"
set "time" from="horizon"
set "nodes"
"#;

    let parsed = parse_program_text(text, &path)?;
    let program = parsed.program;

    assert_eq!(program.sets.len(), 3);

    assert_eq!(program.sets[0].name, "generators");
    assert_eq!(program.sets[0].source, None);

    assert_eq!(program.sets[1].name, "time");
    assert_eq!(program.sets[1].source.as_deref(), Some("horizon"));

    assert_eq!(program.sets[2].name, "nodes");
    assert_eq!(program.sets[2].source, None);

    Ok(())
}

#[test]
fn parses_constraint_with_over_when_expr_generation_bindings()
-> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("test.kdl");
    let text = r#"
rule "NodeBalance" {
  constraint "balance" {
    over "n" in="nodes"
    over "t" in="time"
    when "active_node[n]"
    expr {
      sum(dispatch[g,t] for g in generators if generator_node[g] == n) = demand[n,t]
    }
  }
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let program = parsed.program;
    let rule = program.rule("NodeBalance").ok_or("missing rule")?;

    assert_eq!(rule.constraints.len(), 1);
    let constraint = &rule.constraints[0];

    assert_eq!(constraint.name, "balance");
    assert_eq!(
        constraint.generation_bindings,
        vec![
            GenerationBinding {
                variable: "n".to_string(),
                domain: "nodes".to_string(),
            },
            GenerationBinding {
                variable: "t".to_string(),
                domain: "time".to_string(),
            },
        ]
    );
    assert_eq!(
        constraint.generation_filter.as_deref(),
        Some("active_node[n]")
    );
    assert!(constraint.expression.contains(
        "sum(dispatch[g,t] for g in generators if generator_node[g] == n) = demand[n,t]"
    ));

    Ok(())
}

#[test]
fn parses_control_with_kind_and_upper_bound() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from("test.kdl");
    let text = r#"
model "BinaryModel" {
  set "assets"
  set "time" from="horizon"

  control "indicator" kind="binary" lower=0 upper=1 {
    a
    t
  }

  constraint "limit[a in assets, t in time]" {
    indicator[a,t] <= 1
  }

  minimize "Cost" {
    sum(indicator[a,t] for a in assets for t in time)
  }
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let model = parsed.program.model("BinaryModel").ok_or("missing model")?;

    assert_eq!(model.controls.len(), 1);
    let control = &model.controls[0];
    assert_eq!(control.name, "indicator");
    assert_eq!(control.kind, Some(VariableKindDecl::Binary));
    assert_eq!(
        control.lower,
        Some(BoundExpr::Literal(LiteralValue::Integer(0)))
    );
    assert_eq!(
        control.upper,
        Some(BoundExpr::Literal(LiteralValue::Integer(1)))
    );
    assert_eq!(control.indices, vec![idx("a"), idx("t")]);

    Ok(())
}

#[test]
fn parses_control_with_formula_bounds_via_child_nodes() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from("test.kdl");
    let text = r#"
model "FormulaModel" {
  set "assets"
  set "time" from="horizon"

  param "max_cap" {
    a
  }

  control "output" lower=0 {
    a
    t
    upper { max_cap[a] }
  }

  constraint "limit[a in assets, t in time]" {
    output[a,t] <= max_cap[a]
  }

  minimize "Cost" {
    sum(output[a,t] for a in assets for t in time)
  }
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let model = parsed
        .program
        .model("FormulaModel")
        .ok_or("missing model")?;

    let control = &model.controls[0];
    assert_eq!(control.name, "output");
    // Literal lower from property
    assert_eq!(
        control.lower,
        Some(BoundExpr::Literal(LiteralValue::Integer(0)))
    );
    // Formula upper from child node
    assert!(matches!(control.upper, Some(BoundExpr::Formula(_))));
    if let Some(BoundExpr::Formula(ref expr)) = control.upper {
        assert_eq!(expr.to_string(), "max_cap[a]");
    }
    // Indices should NOT include "upper" (that's a bound child, not an index)
    assert_eq!(control.indices, vec![idx("a"), idx("t")]);

    Ok(())
}

#[test]
fn parses_technology_with_control_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from("test.kdl");
    let text = r#"
technology "Generator" {
  control "dispatch" kind="continuous" lower=0
  control "commit" kind="binary" lower=0 upper=1
  state "on_hours"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let technology = parsed
        .program
        .technology("Generator")
        .ok_or("missing technology")?;

    assert_eq!(technology.controls.len(), 2);

    let dispatch = &technology.controls[0];
    assert_eq!(dispatch.name, "dispatch");
    assert_eq!(dispatch.kind, Some(VariableKindDecl::Continuous));
    assert_eq!(
        dispatch.lower,
        Some(BoundExpr::Literal(LiteralValue::Integer(0)))
    );
    assert_eq!(dispatch.upper, None);

    let commit = &technology.controls[1];
    assert_eq!(commit.name, "commit");
    assert_eq!(commit.kind, Some(VariableKindDecl::Binary));
    assert_eq!(
        commit.lower,
        Some(BoundExpr::Literal(LiteralValue::Integer(0)))
    );
    assert_eq!(
        commit.upper,
        Some(BoundExpr::Literal(LiteralValue::Integer(1)))
    );

    assert_eq!(technology.states, vec!["on_hours".to_string()]);

    Ok(())
}

#[test]
fn parses_control_with_integer_kind() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from("test.kdl");
    let text = r#"
model "IntModel" {
  set "assets"
  set "time" from="horizon"

  control "units" kind="integer" lower=0 upper=10 {
    a
    t
  }

  constraint "limit[a in assets, t in time]" {
    units[a,t] <= 10
  }

  minimize "Cost" {
    sum(units[a,t] for a in assets for t in time)
  }
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let model = parsed.program.model("IntModel").ok_or("missing model")?;

    let control = &model.controls[0];
    assert_eq!(control.kind, Some(VariableKindDecl::Integer));
    assert_eq!(
        control.lower,
        Some(BoundExpr::Literal(LiteralValue::Integer(0)))
    );
    assert_eq!(
        control.upper,
        Some(BoundExpr::Literal(LiteralValue::Integer(10)))
    );

    Ok(())
}

#[test]
fn parses_control_without_kind_defaults_to_none() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from("test.kdl");
    let text = r#"
model "SimpleModel" {
  set "assets"
  set "time" from="horizon"

  control "dispatch" lower=0 {
    a
    t
  }

  constraint "limit[a in assets, t in time]" {
    dispatch[a,t] <= 100
  }

  minimize "Cost" {
    sum(dispatch[a,t] for a in assets for t in time)
  }
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let model = parsed.program.model("SimpleModel").ok_or("missing model")?;

    let control = &model.controls[0];
    assert_eq!(control.kind, None);
    assert_eq!(control.upper, None);

    Ok(())
}

#[test]
fn parses_control_index_with_domain_binding() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from("test.kdl");
    let text = r#"
model "GenModel" {
  set "generators"
  set "time" from="horizon"

  control "gen_output" {
    g in="generators"
    t
  }

  constraint "limit[g in generators, t in time]" {
    gen_output[g,t] <= 100
  }

  minimize "Cost" {
    sum(gen_output[g,t] for g in generators for t in time)
  }
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let model = parsed.program.model("GenModel").ok_or("missing model")?;

    let control = &model.controls[0];
    assert_eq!(control.name, "gen_output");
    assert_eq!(
        control.indices,
        vec![
            IndexDecl {
                name: "g".to_string(),
                domain: Some("generators".to_string()),
            },
            idx("t"),
        ]
    );

    Ok(())
}

#[test]
fn parses_scenario_with_custom_set_declarations() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from("test.kdl");
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
  horizon steps=2 resolution="PT1H"
  use "Simple"
  generators {
    "gen1"
    "gen2"
    "gen3"
  }
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let scenario = parsed.program.first_scenario().ok_or("missing scenario")?;

    assert_eq!(
        scenario.custom_sets.get("generators"),
        Some(&vec![
            "gen1".to_string(),
            "gen2".to_string(),
            "gen3".to_string()
        ])
    );

    Ok(())
}

#[test]
fn parses_technology_with_invest_declarations() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from("test.kdl");
    let text = r#"
technology "CoupledStorage" {
    invest "power_charge"
    invest "power_discharge"
    invest "energy_capacity" kind="integer"
    control "charge"
    control "discharge"
    state "soc"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let technology = parsed
        .program
        .technology("CoupledStorage")
        .ok_or("missing technology")?;

    assert_eq!(technology.investments.len(), 3);

    assert_eq!(
        technology.investments[0],
        NamedVariableDecl {
            name: "power_charge".to_string(),
            kind: None,
            lower: None,
            upper: None,
        }
    );
    assert_eq!(
        technology.investments[1],
        NamedVariableDecl {
            name: "power_discharge".to_string(),
            kind: None,
            lower: None,
            upper: None,
        }
    );
    assert_eq!(
        technology.investments[2],
        NamedVariableDecl {
            name: "energy_capacity".to_string(),
            kind: Some(VariableKindDecl::Integer),
            lower: None,
            upper: None,
        }
    );

    assert_eq!(technology.controls.len(), 2);
    assert_eq!(technology.controls[0].name, "charge");
    assert_eq!(technology.controls[1].name, "discharge");
    assert_eq!(technology.states, vec!["soc".to_string()]);

    Ok(())
}

#[test]
fn parses_invest_with_bounds() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from("test.kdl");
    let text = r#"
technology "Storage" {
    invest "energy_capacity" kind="continuous" lower=0 upper=1000
    control "charge"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let technology = parsed
        .program
        .technology("Storage")
        .ok_or("missing technology")?;

    assert_eq!(technology.investments.len(), 1);
    let invest = &technology.investments[0];
    assert_eq!(invest.name, "energy_capacity");
    assert_eq!(invest.kind, Some(VariableKindDecl::Continuous));
    assert_eq!(
        invest.lower,
        Some(BoundExpr::Literal(LiteralValue::Integer(0)))
    );
    assert_eq!(
        invest.upper,
        Some(BoundExpr::Literal(LiteralValue::Integer(1000)))
    );

    Ok(())
}
