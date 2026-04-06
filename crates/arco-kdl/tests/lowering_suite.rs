#![allow(clippy::float_cmp)]

use arco_kdl::lowering::lower_program;
use arco_kdl::semantic::validate_program;
use arco_kdl::source::{parse_program_file, parse_program_text};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn filters_capacity_expansion_constraint_rows_by_asset() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("constraint-row-filter")?;
    let path = root.join("input.kdl");
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("existing_thermal.csv"),
        "asset_name,existing_capacity_mw,variable_cost,apply_dispatch_limit\nEnabled,50,10,1\nDisabled,50,10,0\n",
    )?;
    fs::write(root.join("data").join("demand.csv"), "t,demand\n1,25\n")?;
    fs::write(root.join("data").join("voll.csv"), "t,voll\n1,1000\n")?;
    fs::write(
        &path,
        r#"
technology "GasCT" {
  control "dispatch"
}

operation "ExpansionDispatch" {
  constraint "dispatch_limit" if="apply_dispatch_limit[a] == 1" {
    dispatch[a,t] <= existing_capacity[a]
  }
}

rule "EnergyBalance" {
  constraint "balance" {
    sum(dispatch[a,t] for a in assets) + unserved_energy[t] = demand[t]
  }
}

expression "OperatingCost" {
  sum(variable_cost[a] * dispatch[a,t] for a in assets for t in time)
}

expression "PenaltyCost" {
  sum(voll[t] * unserved_energy[t] for t in time)
}

minimize "ExpansionCost" {
  OperatingCost + PenaltyCost
}

instances "ExistingThermal" from="data/existing_thermal.csv" {
  technology "GasCT"
  operation "ExpansionDispatch"
  map "name" from="asset_name"
  map "existing_capacity" from="existing_capacity_mw"
  map "variable_cost" from="variable_cost"
  map "apply_dispatch_limit" from="apply_dispatch_limit"
}

scenario "ExpansionCase" {
  horizon steps=1 resolution="PT1H"
  technology "GasCT"
  operation "ExpansionDispatch"
  rule "EnergyBalance"
  data "demand" from="data/demand.csv"
  data "voll" from="data/voll.csv"
  instances "ExistingThermal"
  minimize "ExpansionCost"
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic_program = validate_program(&parsed.program, &path)?;
    let lowered_problem = lower_program(&semantic_program, &parsed.program, &path)?;

    assert!(
        lowered_problem
            .algebra
            .constraints
            .iter()
            .any(|constraint| constraint.name == "dispatch_limit[Enabled,1]")
    );
    assert!(
        lowered_problem
            .algebra
            .constraints
            .iter()
            .all(|constraint| constraint.name != "dispatch_limit[Disabled,1]")
    );
    assert!(
        lowered_problem
            .algebra
            .constraints
            .iter()
            .any(|constraint| constraint.name == "balance[1]")
    );

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn filters_market_clearing_bound_constraints_without_emitting_rows()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("constraint-bound-filter")?;
    let path = root.join("input.kdl");
    fs::create_dir_all(root.join("data"))?;
    fs::write(root.join("data").join("prices.csv"), "t,prices\n1,50\n")?;
    fs::write(
        &path,
        r#"
technology "Battery" {
  control "charge"
  control "discharge"
  state "soc"
}

asset "Battery1" {
  technology "Battery"
  operation "PriceTakerBattery"
  power_mw 100
  energy_mwh 400
  charge_efficiency 0.92
  discharge_efficiency 0.92
  initial_soc_mwh 200
  terminal_soc_mwh 200
  apply_charge_limit 0
}

operation "PriceTakerBattery" {
  constraint "soc_balance" {
    soc[a,t] = soc[a,t-1] + charge_efficiency[a] * charge[a,t] - discharge[a,t] / discharge_efficiency[a]
  }
  constraint "charge_limit" if="apply_charge_limit[a] == 1" {
    charge[a,t] <= power_mw[a]
  }
  constraint "discharge_limit" {
    discharge[a,t] <= power_mw[a]
  }
  constraint "soc_bounds" {
    0 <= soc[a,t] <= energy_mwh[a]
  }
}

expression "ArbitrageRevenue" {
  sum(prices[t] * (discharge[a,t] - charge[a,t]) for a in assets for t in time)
}

maximize "ArbitrageProfit" {
  ArbitrageRevenue
}

scenario "BatteryArbitrageDay" {
  horizon steps=1 resolution="PT1H"
  technology "Battery"
  operation "PriceTakerBattery"
  data "prices" from="data/prices.csv"
  asset "Battery1"
  maximize "ArbitrageProfit"
}
"#,
    )?;

    let parsed = parse_program_text(&fs::read_to_string(&path)?, &path)?;
    let semantic_program = validate_program(&parsed.program, &path)?;
    let lowered_problem = lower_program(&semantic_program, &parsed.program, &path)?;
    let charge = lowered_problem
        .algebra
        .variable_instances
        .iter()
        .find(|instance| instance.name == "charge[Battery1,1]")
        .ok_or("missing charge variable")?;

    assert_eq!(charge.lower, 0.0);
    assert_eq!(charge.upper, None);

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn lowers_canonical_model_with_declared_control_family() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("canonical-lowering")?;
    let path = root.join("input.kdl");
    fs::write(
        &path,
        r#"
model "EconomicDispatch" {
  control "dispatch" {
    a
    t
  }

  constraint "dispatch_nonnegative" {
    dispatch[a,t] >= 0
  }

  minimize "TotalCost" {
    sum(dispatch[a,t] for a in assets for t in time)
  }
}

scenario "Case" {
  horizon steps=1 resolution="PT1H"
  use "EconomicDispatch"
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic_program = validate_program(&parsed.program, &path)?;
    let lowered_problem = lower_program(&semantic_program, &parsed.program, &path)?;

    assert!(
        lowered_problem
            .algebra
            .variable_instances
            .iter()
            .any(|instance| instance.name == "dispatch[default,1]")
    );
    assert!(
        lowered_problem
            .algebra
            .constraints
            .iter()
            .any(|constraint| constraint.name == "dispatch_nonnegative[default,1]")
    );

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn canonical_model_binary_control_threads_kind_and_bounds() -> Result<(), Box<dyn std::error::Error>>
{
    let root = temp_test_dir("binary-control")?;
    let path = root.join("input.kdl");
    fs::write(
        &path,
        r#"
model "BinaryIndicatorModel" {
    control "indicator" kind="binary" lower=0 upper=1 {
        a
        t
    }

    constraint "indicator_nonneg" {
        indicator[a,t] >= 0
    }

    minimize "TotalCost" {
        sum(indicator[a,t] for a in assets for t in time)
    }
}

scenario "Case" {
    horizon steps=2 resolution="PT1H"
    use "BinaryIndicatorModel"
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic_program = validate_program(&parsed.program, &path)?;

    // Verify semantic overrides were populated.
    assert!(
        semantic_program
            .variable_overrides
            .contains_key("indicator"),
        "expected variable_overrides to contain 'indicator'"
    );

    let lowered_problem = lower_program(&semantic_program, &parsed.program, &path)?;

    // Every indicator instance should be Binary with lower=0 and upper=Some(1.0).
    let indicator_instances: Vec<_> = lowered_problem
        .algebra
        .variable_instances
        .iter()
        .filter(|instance| instance.family == "indicator[a,t]")
        .collect();
    assert!(
        !indicator_instances.is_empty(),
        "expected at least one indicator variable instance"
    );
    for instance in &indicator_instances {
        assert_eq!(
            instance.kind,
            arco_kdl::lowering::VariableKind::Binary,
            "instance {} should be Binary",
            instance.name
        );
        assert_eq!(
            instance.lower, 0.0,
            "instance {} lower should be 0.0",
            instance.name
        );
        assert_eq!(
            instance.upper,
            Some(1.0),
            "instance {} upper should be Some(1.0)",
            instance.name
        );
    }

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn canonical_model_without_overrides_uses_hardcoded_defaults()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("no-overrides")?;
    let path = root.join("input.kdl");
    fs::write(
        &path,
        r#"
model "SimpleDispatch" {
    control "dispatch" {
        a
        t
    }

    constraint "dispatch_nonneg" {
        dispatch[a,t] >= 0
    }

    minimize "TotalCost" {
        sum(dispatch[a,t] for a in assets for t in time)
    }
}

scenario "Case" {
    horizon steps=2 resolution="PT1H"
    use "SimpleDispatch"
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic_program = validate_program(&parsed.program, &path)?;

    // No overrides when kind/lower/upper are absent.
    assert!(
        semantic_program.variable_overrides.is_empty(),
        "expected no variable overrides for plain controls"
    );

    let lowered_problem = lower_program(&semantic_program, &parsed.program, &path)?;

    // Dispatch should use the hardcoded default: lower=0 (no energy_mwh
    // param on the synthetic asset), upper=None, Continuous.
    let dispatch_instances: Vec<_> = lowered_problem
        .algebra
        .variable_instances
        .iter()
        .filter(|instance| instance.family == "dispatch[a,t]")
        .collect();
    assert!(
        !dispatch_instances.is_empty(),
        "expected at least one dispatch variable instance"
    );
    for instance in &dispatch_instances {
        assert_eq!(
            instance.kind,
            arco_kdl::lowering::VariableKind::Continuous,
            "instance {} should be Continuous",
            instance.name
        );
        assert_eq!(
            instance.lower, 0.0,
            "instance {} lower should be 0.0",
            instance.name
        );
        assert_eq!(
            instance.upper, None,
            "instance {} upper should be None",
            instance.name
        );
    }

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn lowers_reduction_with_if_filter_on_parameter() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("reduction-filter")?;
    let path = root.join("input.kdl");
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("plants.csv"),
        "asset_name,marginal_cost,is_active\nPlantA,10,1\nPlantB,20,0\nPlantC,30,1\n",
    )?;
    fs::write(root.join("data").join("demand.csv"), "t,demand\n1,50\n")?;
    fs::write(root.join("data").join("voll.csv"), "t,voll\n1,1000\n")?;
    fs::write(
        &path,
        r#"
technology "Thermal" {
  control "dispatch"
}

operation "SimpleDispatch" {
  constraint "dispatch_nonneg" {
    dispatch[a,t] >= 0
  }
}

rule "Balance" {
  constraint "balance" {
    sum(dispatch[a,t] for a in assets if is_active[a] == 1) + unserved_energy[t] = demand[t]
  }
}

expression "FuelCost" {
  sum(marginal_cost[a] * dispatch[a,t] for a in assets for t in time if is_active[a] == 1)
}

expression "PenaltyCost" {
  sum(voll[t] * unserved_energy[t] for t in time)
}

minimize "TotalCost" {
  FuelCost + PenaltyCost
}

instances "Plants" from="data/plants.csv" {
  technology "Thermal"
  operation "SimpleDispatch"
  map "name" from="asset_name"
  map "marginal_cost" from="marginal_cost"
  map "is_active" from="is_active"
}

scenario "Case" {
  horizon steps=1 resolution="PT1H"
  technology "Thermal"
  operation "SimpleDispatch"
  rule "Balance"
  data "demand" from="data/demand.csv"
  data "voll" from="data/voll.csv"
  instances "Plants"
  minimize "TotalCost"
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic_program = validate_program(&parsed.program, &path)?;
    let lowered = lower_program(&semantic_program, &parsed.program, &path)?;

    // The balance constraint should only have terms for active assets (PlantA and PlantC),
    // not for PlantB (is_active == 0).
    let balance = lowered
        .algebra
        .constraints
        .iter()
        .find(|c| c.name == "balance[1]")
        .expect("expected balance[1] constraint");

    let term_names: Vec<&str> = balance
        .terms
        .iter()
        .map(|t| t.variable_name.as_str())
        .collect();
    assert!(
        term_names.contains(&"dispatch[PlantA,1]"),
        "balance should contain dispatch[PlantA,1]"
    );
    assert!(
        term_names.contains(&"dispatch[PlantC,1]"),
        "balance should contain dispatch[PlantC,1]"
    );
    assert!(
        !term_names.contains(&"dispatch[PlantB,1]"),
        "balance should NOT contain dispatch[PlantB,1] (filtered out by is_active)"
    );

    // FuelCost expression in the objective should also only reference active assets.
    let objective_term_names: Vec<&str> = lowered
        .algebra
        .objective
        .terms
        .iter()
        .map(|t| t.variable_name.as_str())
        .collect();
    assert!(
        objective_term_names.contains(&"dispatch[PlantA,1]"),
        "objective should reference dispatch[PlantA,1]"
    );
    assert!(
        objective_term_names.contains(&"dispatch[PlantC,1]"),
        "objective should reference dispatch[PlantC,1]"
    );
    assert!(
        !objective_term_names.contains(&"dispatch[PlantB,1]"),
        "objective should NOT reference dispatch[PlantB,1]"
    );

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn lowers_reduction_with_multiple_if_filters() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("reduction-multi-filter")?;
    let path = root.join("input.kdl");
    fs::create_dir_all(root.join("data"))?;
    // PlantA: active=1, region=1 -> passes both filters
    // PlantB: active=0, region=1 -> fails is_active filter
    // PlantC: active=1, region=2 -> fails region filter
    // PlantD: active=1, region=1 -> passes both filters
    fs::write(
        root.join("data").join("plants.csv"),
        "asset_name,marginal_cost,is_active,region_id\nPlantA,10,1,1\nPlantB,20,0,1\nPlantC,30,1,2\nPlantD,40,1,1\n",
    )?;
    fs::write(root.join("data").join("demand.csv"), "t,demand\n1,100\n")?;
    fs::write(root.join("data").join("voll.csv"), "t,voll\n1,1000\n")?;
    fs::write(
        &path,
        r#"
technology "Thermal" {
  control "dispatch"
}

operation "SimpleDispatch" {
  constraint "dispatch_nonneg" {
    dispatch[a,t] >= 0
  }
}

rule "Balance" {
  constraint "balance" {
    sum(dispatch[a,t] for a in assets if is_active[a] == 1 if region_id[a] == 1) + unserved_energy[t] = demand[t]
  }
}

expression "FuelCost" {
  sum(marginal_cost[a] * dispatch[a,t] for a in assets for t in time)
}

expression "PenaltyCost" {
  sum(voll[t] * unserved_energy[t] for t in time)
}

minimize "TotalCost" {
  FuelCost + PenaltyCost
}

instances "Plants" from="data/plants.csv" {
  technology "Thermal"
  operation "SimpleDispatch"
  map "name" from="asset_name"
  map "marginal_cost" from="marginal_cost"
  map "is_active" from="is_active"
  map "region_id" from="region_id"
}

scenario "Case" {
  horizon steps=1 resolution="PT1H"
  technology "Thermal"
  operation "SimpleDispatch"
  rule "Balance"
  data "demand" from="data/demand.csv"
  data "voll" from="data/voll.csv"
  instances "Plants"
  minimize "TotalCost"
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic_program = validate_program(&parsed.program, &path)?;
    let lowered = lower_program(&semantic_program, &parsed.program, &path)?;

    // Only PlantA and PlantD pass both filters (is_active == 1 AND region_id == 1).
    let balance = lowered
        .algebra
        .constraints
        .iter()
        .find(|c| c.name == "balance[1]")
        .expect("expected balance[1] constraint");

    let term_names: Vec<&str> = balance
        .terms
        .iter()
        .map(|t| t.variable_name.as_str())
        .collect();
    assert!(
        term_names.contains(&"dispatch[PlantA,1]"),
        "balance should contain dispatch[PlantA,1]"
    );
    assert!(
        term_names.contains(&"dispatch[PlantD,1]"),
        "balance should contain dispatch[PlantD,1]"
    );
    assert!(
        !term_names.contains(&"dispatch[PlantB,1]"),
        "balance should NOT contain dispatch[PlantB,1] (is_active == 0)"
    );
    assert!(
        !term_names.contains(&"dispatch[PlantC,1]"),
        "balance should NOT contain dispatch[PlantC,1] (region_id == 2)"
    );

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn lowers_constraint_with_over_when_expr_generation() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("over-when-expr")?;
    let path = root.join("input.kdl");
    fs::create_dir_all(root.join("data"))?;
    // PlantA and PlantB are loaded from "CandidatePlants" (starts with "Candidate"),
    // so they are candidate assets. PlantC is from "ExistingPlants", not a candidate.
    // PlantA: candidate, is_active=1 -> passes both candidate domain and filter
    // PlantB: candidate, is_active=0 -> in candidate domain but fails filter
    // PlantC: NOT candidate, is_active=1 -> excluded by `over ... in="candidate_assets"`
    fs::write(
        root.join("data").join("candidates.csv"),
        "asset_name,cap_mw,is_active,max_build\nPlantA,100,1,500\nPlantB,200,0,500\n",
    )?;
    fs::write(
        root.join("data").join("existing.csv"),
        "asset_name,cap_mw,is_active\nPlantC,150,1\n",
    )?;
    fs::write(root.join("data").join("demand.csv"), "t,demand\n1,80\n")?;
    fs::write(root.join("data").join("voll.csv"), "t,voll\n1,1000\n")?;
    fs::write(
        &path,
        r#"
technology "Thermal" {
  control "dispatch"
}

operation "Dispatch" {
  constraint "dispatch_nonneg" {
    dispatch[a,t] >= 0
  }
  constraint "cap_limit" {
    over "a" in="candidate_assets"
    over "t" in="time"
    when "is_active[a] == 1"
    expr {
      dispatch[a,t] <= cap_mw[a]
    }
  }
}

rule "Balance" {
  constraint "balance" {
    sum(dispatch[a,t] for a in assets) + unserved_energy[t] = demand[t]
  }
}

expression "FuelCost" {
  sum(dispatch[a,t] for a in assets for t in time)
}

expression "PenaltyCost" {
  sum(voll[t] * unserved_energy[t] for t in time)
}

minimize "TotalCost" {
  FuelCost + PenaltyCost
}

instances "CandidatePlants" from="data/candidates.csv" {
  technology "Thermal"
  operation "Dispatch"
  map "name" from="asset_name"
  map "cap_mw" from="cap_mw"
  map "is_active" from="is_active"
  map "max_build" from="max_build"
}

instances "ExistingPlants" from="data/existing.csv" {
  technology "Thermal"
  operation "Dispatch"
  map "name" from="asset_name"
  map "cap_mw" from="cap_mw"
  map "is_active" from="is_active"
}

scenario "Case" {
  horizon steps=1 resolution="PT1H"
  technology "Thermal"
  operation "Dispatch"
  rule "Balance"
  data "demand" from="data/demand.csv"
  data "voll" from="data/voll.csv"
  instances "CandidatePlants"
  instances "ExistingPlants"
  minimize "TotalCost"
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic_program = validate_program(&parsed.program, &path)?;
    let lowered = lower_program(&semantic_program, &parsed.program, &path)?;

    let constraint_names: Vec<&str> = lowered
        .algebra
        .constraints
        .iter()
        .map(|c| c.name.as_str())
        .collect();

    // cap_limit should ONLY be emitted for PlantA: it is in candidate_assets AND is_active == 1.
    // PlantB is a candidate but is_active == 0 -> filtered out by `when`.
    // PlantC is NOT a candidate -> excluded by `over ... in="candidate_assets"`.
    assert!(
        constraint_names.contains(&"cap_limit[PlantA,1]"),
        "should emit cap_limit for PlantA (candidate + active): {constraint_names:?}"
    );
    assert!(
        !constraint_names
            .iter()
            .any(|n| n.contains("cap_limit") && n.contains("PlantB")),
        "should NOT emit cap_limit for PlantB (candidate but inactive): {constraint_names:?}"
    );
    assert!(
        !constraint_names
            .iter()
            .any(|n| n.contains("cap_limit") && n.contains("PlantC")),
        "should NOT emit cap_limit for PlantC (not a candidate): {constraint_names:?}"
    );

    let cap_limit_a = lowered
        .algebra
        .constraints
        .iter()
        .find(|c| c.name == "cap_limit[PlantA,1]")
        .expect("expected cap_limit constraint for PlantA at step 1");
    let term_names: Vec<&str> = cap_limit_a
        .terms
        .iter()
        .map(|t| t.variable_name.as_str())
        .collect();
    assert!(
        term_names.contains(&"dispatch[PlantA,1]"),
        "cap_limit for PlantA should reference dispatch[PlantA,1]: {term_names:?}"
    );

    // The balance constraint should still work normally (not affected by generation bindings).
    assert!(
        constraint_names.contains(&"balance[1]"),
        "balance constraint should exist: {constraint_names:?}"
    );

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn custom_set_in_scenario_populates_registry_and_resolves_in_reduction()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("custom-set-reduction")?;
    let path = root.join("input.kdl");
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("gen_cost.csv"),
        "asset_name,t,gen_cost\nUnit1,1,10\nUnit1,2,10\n",
    )?;

    fs::write(
        &path,
        r#"
technology "Thermal" {
  control "dispatch"
}

operation "GenDispatch" {
  constraint "nonneg" {
    dispatch[a,t] >= 0
  }
}

rule "Balance" {
  constraint "balance" {
    sum(dispatch[a,t] for a in generators) = 100
  }
}

expression "GenCost" {
  sum(gen_cost[a,t] * dispatch[a,t] for a in generators for t in time)
}

minimize "TotalCost" {
  GenCost
}

asset "Unit1" {
  technology "Thermal"
  operation "GenDispatch"
}

scenario "S1" {
  horizon steps=2 resolution="PT1H"
  technology "Thermal"
  operation "GenDispatch"
  rule "Balance"
  asset "Unit1"
  data "gen_cost" from="data/gen_cost.csv"
  minimize "TotalCost"
  generators {
    "Unit1"
  }
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic_program = validate_program(&parsed.program, &path)?;

    // The custom set should be in the registry.
    assert!(
        semantic_program.set_registry.contains_key("generators"),
        "registry should contain 'generators': {:?}",
        semantic_program.set_registry.keys().collect::<Vec<_>>()
    );
    assert_eq!(
        semantic_program.set_registry["generators"].values,
        vec!["Unit1".to_string()]
    );

    // Lowering should succeed because the "generators" set is resolvable
    // in reduction_domain_values via the set_registry.
    let lowered = lower_program(&semantic_program, &parsed.program, &path)?;

    // The reduction `for a in generators` should iterate over the custom
    // set's members. Since generators = { "Unit1" }, the balance constraint
    // sums dispatch over that one asset.
    let constraint_names: Vec<&str> = lowered
        .algebra
        .constraints
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    assert!(
        constraint_names.contains(&"balance[1]"),
        "expected balance[1] in constraints: {constraint_names:?}"
    );
    assert!(
        constraint_names.contains(&"balance[2]"),
        "expected balance[2] in constraints: {constraint_names:?}"
    );

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn lowers_sqrt_of_parameter_into_evaluated_coefficient() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("sqrt-param")?;
    let path = root.join("input.kdl");
    fs::create_dir_all(root.join("data"))?;
    fs::write(root.join("data").join("demand.csv"), "t,demand\n1,100\n")?;
    fs::write(root.join("data").join("voll.csv"), "t,voll\n1,1000\n")?;
    // eta = 0.81 so sqrt(eta) = 0.9
    fs::write(
        &path,
        r#"
technology "Battery" {
  control "charge"
}

operation "BatteryOp" {
  constraint "scaled_charge" {
    sqrt(eta[a]) * charge[a,t] <= power_mw[a]
  }
}

expression "PenaltyCost" {
  sum(voll[t] * unserved_energy[t] for t in time)
}

minimize "Cost" {
  PenaltyCost
}

asset "B1" {
  technology "Battery"
  operation "BatteryOp"
  eta 0.81
  power_mw 100
}

scenario "S" {
  horizon steps=1 resolution="PT1H"
  technology "Battery"
  operation "BatteryOp"
  data "demand" from="data/demand.csv"
  data "voll" from="data/voll.csv"
  asset "B1"
  minimize "Cost"
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic_program = validate_program(&parsed.program, &path)?;
    let lowered = lower_program(&semantic_program, &parsed.program, &path)?;

    // sqrt(0.81) = 0.9, so the coefficient on charge[B1,1] should be 0.9.
    let constraint = lowered
        .algebra
        .constraints
        .iter()
        .find(|c| c.name == "scaled_charge[B1,1]")
        .expect("expected scaled_charge constraint");

    let charge_term = constraint
        .terms
        .iter()
        .find(|t| t.variable_name == "charge[B1,1]")
        .expect("expected charge[B1,1] term");

    assert!(
        (charge_term.coefficient - 0.9).abs() < 1e-9,
        "sqrt(0.81) should yield coefficient 0.9, got {}",
        charge_term.coefficient
    );

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn rejects_builtin_function_applied_to_decision_variable() -> Result<(), Box<dyn std::error::Error>>
{
    let root = temp_test_dir("sqrt-var-reject")?;
    let path = root.join("input.kdl");
    fs::create_dir_all(root.join("data"))?;
    fs::write(root.join("data").join("demand.csv"), "t,demand\n1,100\n")?;
    fs::write(root.join("data").join("voll.csv"), "t,voll\n1,1000\n")?;
    fs::write(
        &path,
        r#"
technology "Battery" {
  control "charge"
  control "discharge"
}

operation "BatteryOp" {
  constraint "charge_nonneg" {
    charge[a,t] >= 0
  }
  constraint "bad_nonlinear" {
    sqrt(charge[a,t]) <= 10
  }
}

expression "PenaltyCost" {
  sum(voll[t] * unserved_energy[t] for t in time)
}

minimize "Cost" {
  PenaltyCost
}

asset "B1" {
  technology "Battery"
  operation "BatteryOp"
  power_mw 100
}

scenario "S" {
  horizon steps=1 resolution="PT1H"
  technology "Battery"
  operation "BatteryOp"
  data "demand" from="data/demand.csv"
  data "voll" from="data/voll.csv"
  asset "B1"
  minimize "Cost"
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic_program = validate_program(&parsed.program, &path)?;
    let result = lower_program(&semantic_program, &parsed.program, &path);

    assert!(
        result.is_err(),
        "lowering should reject sqrt() applied to a decision variable"
    );
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("must remain scalar"),
        "error should mention that the argument must remain scalar, got: {err_msg}"
    );

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn auto_generates_technology_sets_for_reduction_scoping() -> Result<(), Box<dyn std::error::Error>>
{
    let root = temp_test_dir("tech-set-reduction")?;
    let path = root.join("input.kdl");
    fs::create_dir_all(root.join("data"))?;

    // Two batteries and one generator. The objective sums dispatch only over
    // the "Battery" technology set, so the lowered balance constraint should
    // only reference bat1 and bat2, not gen1.
    fs::write(
        root.join("data").join("demand.csv"),
        "t,demand\n1,100\n2,100\n",
    )?;
    fs::write(
        root.join("data").join("voll.csv"),
        "t,voll\n1,1000\n2,1000\n",
    )?;

    fs::write(
        &path,
        r#"
technology "Battery" {
  control "dispatch"
}

technology "Generator" {
  control "dispatch"
}

operation "SimpleOp" {
  constraint "nonneg" {
    dispatch[a,t] >= 0
  }
}

rule "BatteryBalance" {
  constraint "bat_balance" {
    sum(dispatch[a,t] for a in Battery) = 50
  }
}

expression "BatteryCost" {
  sum(dispatch[a,t] for a in Battery for t in time)
}

expression "PenaltyCost" {
  sum(voll[t] * unserved_energy[t] for t in time)
}

minimize "TotalCost" {
  BatteryCost + PenaltyCost
}

asset "bat1" {
  technology "Battery"
  operation "SimpleOp"
}

asset "bat2" {
  technology "Battery"
  operation "SimpleOp"
}

asset "gen1" {
  technology "Generator"
  operation "SimpleOp"
}

scenario "TechSetCase" {
  horizon steps=2 resolution="PT1H"
  technology "Battery"
  technology "Generator"
  operation "SimpleOp"
  rule "BatteryBalance"
  data "demand" from="data/demand.csv"
  data "voll" from="data/voll.csv"
  asset "bat1"
  asset "bat2"
  asset "gen1"
  minimize "TotalCost"
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic_program = validate_program(&parsed.program, &path)?;

    // The set registry should contain auto-generated technology sets.
    assert!(
        semantic_program.set_registry.contains_key("Battery"),
        "registry should contain 'Battery': {:?}",
        semantic_program.set_registry.keys().collect::<Vec<_>>()
    );
    assert_eq!(
        semantic_program.set_registry["Battery"].values,
        vec!["bat1".to_string(), "bat2".to_string()]
    );

    assert!(
        semantic_program.set_registry.contains_key("Generator"),
        "registry should contain 'Generator': {:?}",
        semantic_program.set_registry.keys().collect::<Vec<_>>()
    );
    assert_eq!(
        semantic_program.set_registry["Generator"].values,
        vec!["gen1".to_string()]
    );

    // Lowering should succeed because "Battery" is resolvable via the set registry.
    let lowered = lower_program(&semantic_program, &parsed.program, &path)?;

    // The bat_balance constraint uses `for a in Battery`, so only bat1 and bat2
    // should appear in its terms, not gen1.
    let bat_balance_1 = lowered
        .algebra
        .constraints
        .iter()
        .find(|c| c.name == "bat_balance[1]")
        .expect("expected bat_balance[1] constraint");

    let term_names: Vec<&str> = bat_balance_1
        .terms
        .iter()
        .map(|t| t.variable_name.as_str())
        .collect();
    assert!(
        term_names.contains(&"dispatch[bat1,1]"),
        "bat_balance should contain dispatch[bat1,1]: {term_names:?}"
    );
    assert!(
        term_names.contains(&"dispatch[bat2,1]"),
        "bat_balance should contain dispatch[bat2,1]: {term_names:?}"
    );
    assert!(
        !term_names.contains(&"dispatch[gen1,1]"),
        "bat_balance should NOT contain dispatch[gen1,1]: {term_names:?}"
    );

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn invest_variables_produce_asset_only_instances() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("invest-asset-only")?;
    let path = root.join("input.kdl");
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("prices.csv"),
        "t,prices\n1,50\n2,60\n",
    )?;
    fs::write(
        &path,
        r#"
technology "Storage" {
    invest "power_charge" lower=0 upper=500
    invest "energy_capacity" kind="integer" lower=0 upper=1000
    control "charge"
    control "discharge"
    state "soc"
}

asset "Bat1" {
    technology "Storage"
    operation "StorageOp"
    power_mw 100
    energy_mwh 400
    charge_efficiency 0.92
    discharge_efficiency 0.92
    initial_soc_mwh 200
    terminal_soc_mwh 200
}

asset "Bat2" {
    technology "Storage"
    operation "StorageOp"
    power_mw 50
    energy_mwh 200
    charge_efficiency 0.90
    discharge_efficiency 0.90
    initial_soc_mwh 100
    terminal_soc_mwh 100
}

operation "StorageOp" {
    constraint "soc_balance" {
        soc[a,t] = soc[a,t-1] + charge_efficiency[a] * charge[a,t] - discharge[a,t] / discharge_efficiency[a]
    }
    constraint "charge_limit" {
        charge[a,t] <= power_mw[a]
    }
    constraint "discharge_limit" {
        discharge[a,t] <= power_mw[a]
    }
    constraint "soc_bounds" {
        0 <= soc[a,t] <= energy_mwh[a]
    }
    constraint "invest_charge_cap" {
        power_charge[a] <= 500
    }
}

expression "Revenue" {
    sum(prices[t] * (discharge[a,t] - charge[a,t]) for a in assets for t in time)
}

maximize "Profit" {
    Revenue
}

scenario "InvestCase" {
    horizon steps=2 resolution="PT1H"
    technology "Storage"
    operation "StorageOp"
    data "prices" from="data/prices.csv"
    asset "Bat1"
    asset "Bat2"
    maximize "Profit"
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic_program = validate_program(&parsed.program, &path)?;
    let lowered = lower_program(&semantic_program, &parsed.program, &path)?;

    // Invest variables should be [a]-indexed (no time dimension).
    // For 2 assets, we expect 2 instances per invest variable = 4 total invest instances.
    let invest_charge_instances: Vec<_> = lowered
        .algebra
        .variable_instances
        .iter()
        .filter(|i| i.family == "power_charge[a]")
        .collect();
    assert_eq!(
        invest_charge_instances.len(),
        2,
        "expected 2 power_charge instances (one per asset), got: {:?}",
        invest_charge_instances
            .iter()
            .map(|i| &i.name)
            .collect::<Vec<_>>()
    );

    let invest_energy_instances: Vec<_> = lowered
        .algebra
        .variable_instances
        .iter()
        .filter(|i| i.family == "energy_capacity[a]")
        .collect();
    assert_eq!(
        invest_energy_instances.len(),
        2,
        "expected 2 energy_capacity instances (one per asset), got: {:?}",
        invest_energy_instances
            .iter()
            .map(|i| &i.name)
            .collect::<Vec<_>>()
    );

    // Verify instance names have asset-only indexing (no time component).
    assert!(
        lowered
            .algebra
            .variable_instances
            .iter()
            .any(|i| i.name == "power_charge[Bat1]"),
        "expected power_charge[Bat1] in variable instances"
    );
    assert!(
        lowered
            .algebra
            .variable_instances
            .iter()
            .any(|i| i.name == "power_charge[Bat2]"),
        "expected power_charge[Bat2] in variable instances"
    );
    assert!(
        lowered
            .algebra
            .variable_instances
            .iter()
            .any(|i| i.name == "energy_capacity[Bat1]"),
        "expected energy_capacity[Bat1] in variable instances"
    );
    assert!(
        lowered
            .algebra
            .variable_instances
            .iter()
            .any(|i| i.name == "energy_capacity[Bat2]"),
        "expected energy_capacity[Bat2] in variable instances"
    );

    // Verify invest variable bounds and kind are threaded through.
    let pc_bat1 = lowered
        .algebra
        .variable_instances
        .iter()
        .find(|i| i.name == "power_charge[Bat1]")
        .expect("missing power_charge[Bat1]");
    assert_eq!(pc_bat1.lower, 0.0, "power_charge lower should be 0");
    assert_eq!(
        pc_bat1.upper,
        Some(500.0),
        "power_charge upper should be 500"
    );
    assert_eq!(
        pc_bat1.kind,
        arco_kdl::lowering::VariableKind::Continuous,
        "power_charge should be Continuous"
    );

    let ec_bat1 = lowered
        .algebra
        .variable_instances
        .iter()
        .find(|i| i.name == "energy_capacity[Bat1]")
        .expect("missing energy_capacity[Bat1]");
    assert_eq!(
        ec_bat1.kind,
        arco_kdl::lowering::VariableKind::Integer,
        "energy_capacity should be Integer"
    );
    assert_eq!(ec_bat1.lower, 0.0, "energy_capacity lower should be 0");
    assert_eq!(
        ec_bat1.upper,
        Some(1000.0),
        "energy_capacity upper should be 1000"
    );

    // Invest variables should appear in constraint linearization (invest_charge_cap uses power_charge[a]).
    let invest_constraint = lowered
        .algebra
        .constraints
        .iter()
        .find(|c| c.name == "invest_charge_cap[Bat1]")
        .expect("expected invest_charge_cap[Bat1] constraint");
    let term_names: Vec<&str> = invest_constraint
        .terms
        .iter()
        .map(|t| t.variable_name.as_str())
        .collect();
    assert!(
        term_names.contains(&"power_charge[Bat1]"),
        "invest_charge_cap should reference power_charge[Bat1]: {term_names:?}"
    );

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn model_with_explicit_assets_and_custom_index_sets() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("model-explicit-assets")?;
    let path = root.join("input.kdl");

    fs::write(
        &path,
        r#"
model "MultiAsset" {
    control "dispatch" { a; t }
    control "build_cap" { a }

    constraint "dispatch_cap" {
        dispatch[a,t] <= build_cap[a]
    }

    constraint "dispatch_nonneg" {
        dispatch[a,t] >= 0
    }

    minimize "Cost" {
        sum(dispatch[a,t] for a in assets for t in time)
    }
}

scenario "S" {
    horizon steps=2 resolution="PT1H"
    use "MultiAsset"
    asset "Gen1" { }
    asset "Gen2" { }
    generators { "Gen1"; "Gen2" }
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic = validate_program(&parsed.program, &path)?;

    // Custom set should be registered
    assert!(
        semantic.set_registry.contains_key("generators"),
        "registry should contain 'generators': {:?}",
        semantic.set_registry.keys().collect::<Vec<_>>()
    );
    assert_eq!(
        semantic.set_registry["generators"].values,
        vec!["Gen1".to_string(), "Gen2".to_string()]
    );

    let lowered = lower_program(&semantic, &parsed.program, &path)?;

    // dispatch should be instantiated per-asset per-time
    // a domain = assets = {Gen1, Gen2}, t domain = time = {1,2}
    // So 4 dispatch instances
    let dispatch_instances: Vec<_> = lowered
        .algebra
        .variable_instances
        .iter()
        .filter(|i| i.family.starts_with("dispatch"))
        .collect();
    assert_eq!(
        dispatch_instances.len(),
        4,
        "expected 4 dispatch instances (2 assets x 2 time steps), got: {:?}",
        dispatch_instances
            .iter()
            .map(|i| &i.name)
            .collect::<Vec<_>>()
    );

    // build_cap should be per-asset only (no time)
    let build_instances: Vec<_> = lowered
        .algebra
        .variable_instances
        .iter()
        .filter(|i| i.family.starts_with("build_cap"))
        .collect();
    assert_eq!(
        build_instances.len(),
        2,
        "expected 2 build_cap instances (2 assets), got: {:?}",
        build_instances.iter().map(|i| &i.name).collect::<Vec<_>>()
    );

    // Constraint should reference both dispatch and build_cap for Gen1
    let cap_constraint = lowered
        .algebra
        .constraints
        .iter()
        .find(|c| c.name.contains("dispatch_cap") && c.name.contains("Gen1"))
        .expect("expected dispatch_cap constraint for Gen1");
    let term_names: Vec<&str> = cap_constraint
        .terms
        .iter()
        .map(|t| t.variable_name.as_str())
        .collect();
    assert!(
        term_names
            .iter()
            .any(|n| n.contains("dispatch") && n.contains("Gen1")),
        "dispatch_cap should reference dispatch[Gen1,...]: {term_names:?}"
    );
    assert!(
        term_names
            .iter()
            .any(|n| n.contains("build_cap") && n.contains("Gen1")),
        "dispatch_cap should reference build_cap[Gen1]: {term_names:?}"
    );

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn set_from_csv_populates_registry_and_resolves_params_in_constraint()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("set-from-csv")?;
    let path = root.join("input.kdl");
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data/periods.csv"),
        "name,budget_start,budget_end,energy_budget\nDay1,1,2,100\nDay2,3,4,200\n",
    )?;
    fs::write(
        root.join("data/demand.csv"),
        "t,demand\n1,50\n2,50\n3,50\n4,50\n",
    )?;
    fs::write(
        root.join("data/voll.csv"),
        "t,voll\n1,1000\n2,1000\n3,1000\n4,1000\n",
    )?;
    fs::write(
        &path,
        r#"
technology "Hydro" {
    control "dispatch"
}

operation "HydroOp" {
    constraint "hydro_budget" {
        over "b" in="periods"
        expr {
            sum(dispatch[a,t] for a in assets for t in time
                if t >= budget_start[b] if t <= budget_end[b])
            = energy_budget[b]
        }
    }
}

rule "Balance" {
    constraint "balance" {
        sum(dispatch[a,t] for a in assets) + unserved_energy[t] = demand[t]
    }
}

expression "Penalty" {
    sum(voll[t] * unserved_energy[t] for t in time)
}

minimize "Cost" { Penalty }

asset "Hydro1" {
    technology "Hydro"
    operation "HydroOp"
}

scenario "S" {
    horizon steps=4 resolution="PT1H"
    technology "Hydro"
    operation "HydroOp"
    rule "Balance"
    set "periods" from="data/periods.csv"
    data "demand" from="data/demand.csv"
    data "voll" from="data/voll.csv"
    asset "Hydro1"
    minimize "Cost"
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic = validate_program(&parsed.program, &path)?;

    // Set should be in registry
    assert!(
        semantic.set_registry.contains_key("periods"),
        "registry should contain 'periods': {:?}",
        semantic.set_registry.keys().collect::<Vec<_>>()
    );
    assert_eq!(
        semantic.set_registry["periods"].values,
        vec!["Day1".to_string(), "Day2".to_string()]
    );

    // Set params should be populated
    assert!(semantic.set_params.contains_key("Day1"));
    assert_eq!(semantic.set_params["Day1"]["budget_start"], 1.0);
    assert_eq!(semantic.set_params["Day1"]["budget_end"], 2.0);
    assert_eq!(semantic.set_params["Day1"]["energy_budget"], 100.0);
    assert_eq!(semantic.set_params["Day2"]["budget_start"], 3.0);

    let lowered = lower_program(&semantic, &parsed.program, &path)?;

    // Two budget constraints: one per period
    let constraint_names: Vec<&str> = lowered
        .algebra
        .constraints
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    assert!(
        constraint_names.contains(&"hydro_budget[Day1]"),
        "expected hydro_budget[Day1]: {constraint_names:?}"
    );
    assert!(
        constraint_names.contains(&"hydro_budget[Day2]"),
        "expected hydro_budget[Day2]: {constraint_names:?}"
    );

    // Day1 budget covers t=1,2. Only those dispatch terms should appear.
    let day1 = lowered
        .algebra
        .constraints
        .iter()
        .find(|c| c.name == "hydro_budget[Day1]")
        .unwrap();
    let day1_terms: Vec<&str> = day1
        .terms
        .iter()
        .map(|t| t.variable_name.as_str())
        .collect();
    assert!(day1_terms.contains(&"dispatch[Hydro1,1]"));
    assert!(day1_terms.contains(&"dispatch[Hydro1,2]"));
    assert!(!day1_terms.contains(&"dispatch[Hydro1,3]"));
    assert!(!day1_terms.contains(&"dispatch[Hydro1,4]"));

    // Day1 RHS should be 100.0
    assert!(
        (day1.rhs - 100.0).abs() < 1e-9,
        "Day1 rhs should be 100.0, got {}",
        day1.rhs
    );

    // Day2 budget covers t=3,4
    let day2 = lowered
        .algebra
        .constraints
        .iter()
        .find(|c| c.name == "hydro_budget[Day2]")
        .unwrap();
    assert!((day2.rhs - 200.0).abs() < 1e-9);

    fs::remove_dir_all(&root)?;
    Ok(())
}

fn temp_test_dir(prefix: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_nanos()
        .to_string();
    let root = std::env::temp_dir().join(format!("arco-dsl-{prefix}-{unique}"));
    fs::create_dir_all(&root)?;
    Ok(root)
}
