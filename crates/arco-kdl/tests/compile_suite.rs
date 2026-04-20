#![allow(clippy::float_cmp)]

use arco_kdl::compile::{CompileError, compile_program};
use arco_kdl::semantic::validate_program;
use arco_kdl::source::parse_program_file;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_test_dir(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_nanos()
        .to_string();
    let root = std::env::temp_dir().join(format!("arco-kdl-lowering-{name}-{unique}"));
    fs::create_dir_all(&root)?;
    Ok(root)
}

#[test]
fn lowering_loads_top_level_data_block_params() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("top-level-data")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("inputs.csv"),
        "time,cap,demand\n1,10,10\n2,20,5\n",
    )?;

    let path = root.join("input.kdl");
    fs::write(
        &path,
        r#"
set "time" { "1"; "2" }

data "inputs" source="data/inputs.csv" {
  map "time" from="time"

  param "capacity" index="time" from="cap" reduce="sum"
  param "demand" index="time" from="demand"
}

model "Dispatch" {
  set time alias="t"

  param "capacity" {
    index "t"
  }
  param "demand" {
    index "t"
  }

  control "x" {
    index "t"
  }

  constraint "cap_limit[t]" {
    x[t] <= capacity[t]
  }

  constraint "balance[t]" {
    x[t] = demand[t]
  }

  minimize "Obj" {
    sum(x[t] for t in time)
  }
}

scenario "S1" {
  use "Dispatch"
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic = validate_program(&parsed.program, &path)?;
    let compiled = compile_program(&semantic, &parsed.program, &path)?;

    let cap_1 = compiled
        .algebra
        .constraints
        .iter()
        .find(|c| c.name == "cap_limit[t][1]")
        .ok_or("missing cap constraint at t=1")?;
    let bal_1 = compiled
        .algebra
        .constraints
        .iter()
        .find(|c| c.name == "balance[t][1]")
        .ok_or("missing balance constraint at t=1")?;

    assert_eq!(cap_1.rhs, 10.0);
    assert_eq!(bal_1.rhs, 10.0);

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn lowering_prefers_scenario_data_bindings_over_top_level_data_params()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("scenario-override")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(root.join("data").join("top.csv"), "time,cap\n1,10\n")?;
    fs::write(root.join("data").join("override.csv"), "t,capacity\n1,55\n")?;

    let path = root.join("input.kdl");
    fs::write(
        &path,
        r#"
set "time" { "1" }

data "defaults" source="data/top.csv" {
  map "time" from="time"
  param "capacity" index="time" from="cap"
}

model "Dispatch" {
  set time alias="t"

  param "capacity" {
    index "t"
  }

  control "x" {
    index "t"
  }

  constraint "cap_limit[t]" {
    x[t] <= capacity[t]
  }

  minimize "Obj" {
    sum(x[t] for t in time)
  }
}

scenario "S1" {
  use "Dispatch"
  data "capacity" source="data/override.csv"
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic = validate_program(&parsed.program, &path)?;
    let compiled = compile_program(&semantic, &parsed.program, &path)?;

    let cap = compiled
        .algebra
        .constraints
        .iter()
        .find(|c| c.name == "cap_limit[t][1]")
        .ok_or("missing capacity constraint")?;
    assert_eq!(cap.rhs, 55.0);

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn lowering_reports_missing_data_point_for_sparse_generic_data_table()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("sparse-generic-data")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("dist.csv"),
        "g,b,distance_km\ng1,b1,10\ng1,b2,20\ng2,b1,30\n",
    )?;

    let path = root.join("input.kdl");
    fs::write(
        &path,
        r#"
set "time" { "1" }

data "distance" source="data/dist.csv" {
  set "g"
  set "b"
  param "distance_km" {
    index "g"
    index "b"
  }
}

model "SparseDistance" {
  set "g"
  set "b"

  param "distance_km" {
    index "g"
    index "b"
  }

  control "flow" lower=0 {
    index "g"
    index "b"
  }

  minimize "TotalCost" {
    sum(distance_km[g,b] * flow[g,b] for g in g for b in b)
  }
}

scenario "SparseDistanceCase" {
  use "SparseDistance"
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic = validate_program(&parsed.program, &path)?;
    let err = compile_program(&semantic, &parsed.program, &path)
        .expect_err("sparse data table should fail lowering with a missing key");

    match err {
        CompileError::MissingDataPoint { name, key, .. } => {
            assert_eq!(name, "distance_km");
            assert_eq!(key, "g2,b2");
        }
        other => panic!("expected MissingDataPoint, got {other:?}"),
    }

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn lowering_applies_data_param_filters() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("data-param-filter")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("load.csv"),
        "period,is_candidate,demand\n1,1,10\n1,0,90\n2,1,20\n2,0,80\n",
    )?;

    let path = root.join("input.kdl");
    fs::write(
        &path,
        r#"
set "time" { "1"; "2" }

data "inputs" source="data/load.csv" {
  map "time" from="period"

  param "demand" from="demand" reduce="sum" {
    index "time"
    filter { is_candidate > 0 }
  }
}

model "Dispatch" {
  set "time" alias="t"

  param "demand" {
    index "t"
  }

  control "x" {
    index "t"
  }

  constraint "balance[t]" {
    x[t] = demand[t]
  }

  minimize "Obj" {
    sum(x[t] for t in time)
  }
}

scenario "S1" {
  use "Dispatch"
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic = validate_program(&parsed.program, &path)?;
    let compiled = compile_program(&semantic, &parsed.program, &path)?;

    let bal_1 = compiled
        .algebra
        .constraints
        .iter()
        .find(|c| c.name == "balance[t][1]")
        .ok_or("missing balance constraint at t=1")?;
    let bal_2 = compiled
        .algebra
        .constraints
        .iter()
        .find(|c| c.name == "balance[t][2]")
        .ok_or("missing balance constraint at t=2")?;

    assert_eq!(bal_1.rhs, 10.0);
    assert_eq!(bal_2.rhs, 20.0);

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn lowering_applies_data_param_filters_with_bare_identifier_rhs()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("data-param-filter-bare-identifier")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("load.csv"),
        "period,tech,demand\n1,wind,10\n1,solar,90\n2,wind,20\n2,solar,80\n",
    )?;

    let path = root.join("input.kdl");
    fs::write(
        &path,
        r#"
set "time" { "1"; "2" }

data "inputs" source="data/load.csv" {
  map "time" from="period"

  param "demand" from="demand" reduce="sum" {
    index "time"
    filter { tech == wind }
  }
}

model "Dispatch" {
  set "time" alias="t"

  param "demand" {
    index "t"
  }

  control "x" {
    index "t"
  }

  constraint "balance[t]" {
    x[t] = demand[t]
  }

  minimize "Obj" {
    sum(x[t] for t in time)
  }
}

scenario "S1" {
  use "Dispatch"
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic = validate_program(&parsed.program, &path)?;
    let compiled = compile_program(&semantic, &parsed.program, &path)?;

    let bal_1 = compiled
        .algebra
        .constraints
        .iter()
        .find(|c| c.name == "balance[t][1]")
        .ok_or("missing balance constraint at t=1")?;
    let bal_2 = compiled
        .algebra
        .constraints
        .iter()
        .find(|c| c.name == "balance[t][2]")
        .ok_or("missing balance constraint at t=2")?;

    assert_eq!(bal_1.rhs, 10.0);
    assert_eq!(bal_2.rhs, 20.0);

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn lowering_instantiates_tuple_domain_variables_from_data_rows()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("tuple-domain-from-data")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("links.csv"),
        "area,tech,gen,bus,feasible\n1,wind,g1,b1,1\n1,wind,g1,b2,1\n1,solar,g9,b9,0\n2,solar,g2,b3,1\n",
    )?;

    let path = root.join("input.kdl");
    fs::write(
        &path,
        r#"
data "links" source="data/links.csv" {
  alias "generators" column="gen"
  alias "buses" column="bus"

  set "feasible_links" {
    index "a" { in "area" }
    index "i" { in "tech" }
    index "g" { in "generators" }
    index "b" { in "buses" }
    where { feasible > 0 }
  }
}

model "TupleDispatch" {
  control "x" lower=0 {
    index "a" { in "feasible_links" }
    index "i" { in "feasible_links" }
    index "g" { in "feasible_links" }
    index "b" { in "feasible_links" }
  }

  minimize "Obj" { 0 }
}

scenario "S1" {
  use "TupleDispatch"
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic = validate_program(&parsed.program, &path)?;
    let compiled = compile_program(&semantic, &parsed.program, &path)?;

    let x_instances = compiled
        .algebra
        .variable_instances
        .iter()
        .map(|instance| instance.name.clone())
        .collect::<Vec<_>>();

    assert_eq!(
        x_instances,
        vec![
            "x[1,wind,g1,b1]".to_string(),
            "x[1,wind,g1,b2]".to_string(),
            "x[2,solar,g2,b3]".to_string(),
        ]
    );

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn lowering_tuple_domain_instantiation_handles_alias_and_canonical_set_names()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("tuple-domain-alias-canonical")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("links.csv"),
        "area,tech,gen,bus,feasible\n1,wind,g1,b1,1\n1,wind,g1,b2,1\n2,solar,g2,b3,1\n",
    )?;

    let path = root.join("input.kdl");
    fs::write(
        &path,
        r#"
data "links" source="data/links.csv" {
  alias "generators" column="gen"
  alias "buses" column="bus"

  set "feasible_links" as="fl" {
    index "a" { in "area" }
    index "i" { in "tech" }
    index "g" { in "generators" }
    index "b" { in "buses" }
    where { feasible > 0 }
  }
}

model "TupleDispatch" {
  control "x" lower=0 {
    index "a" { in "fl" }
    index "i" { in "feasible_links" }
    index "g" { in "fl" }
    index "b" { in "feasible_links" }
  }

  minimize "Obj" { 0 }
}

scenario "S1" {
  use "TupleDispatch"
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic = validate_program(&parsed.program, &path)?;
    let compiled = compile_program(&semantic, &parsed.program, &path)?;

    let x_instances = compiled
        .algebra
        .variable_instances
        .iter()
        .map(|instance| instance.name.clone())
        .collect::<Vec<_>>();

    assert_eq!(
        x_instances,
        vec![
            "x[1,wind,g1,b1]".to_string(),
            "x[1,wind,g1,b2]".to_string(),
            "x[2,solar,g2,b3]".to_string(),
        ]
    );

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn lowering_intersects_data_and_rule_tuple_sources_for_domain()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("tuple-domain-intersection")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("links.csv"),
        "area,tech,gen,bus,feasible\n1,wind,g1,b1,1\n1,wind,g1,b2,1\n2,solar,g2,b3,1\n2,wind,g2,b4,0\n",
    )?;

    let path = root.join("input.kdl");
    fs::write(
        &path,
        r#"
data "links" source="data/links.csv" {
  alias "generators" column="gen"
  alias "buses" column="bus"

  set "area"
  set "tech"
  set "generators"
  set "buses"

  set "feasible_links" {
    index "a" { in "area" }
    index "i" { in "tech" }
    index "g" { in "generators" }
    index "b" { in "buses" }
    where { feasible > 0 }
  }
}

set "feasible_links" {
  index "a" { in "area" }
  index "i" { in "tech" }
  index "g" { in "generators" }
  index "b" { in "buses" }
  where { a == "1" }
}

model "TupleDispatch" {
  control "x" lower=0 {
    index "a" { in "feasible_links" }
    index "i" { in "feasible_links" }
    index "g" { in "feasible_links" }
    index "b" { in "feasible_links" }
  }

  minimize "Obj" { 0 }
}

scenario "S1" {
  use "TupleDispatch"
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic = validate_program(&parsed.program, &path)?;
    let compiled = compile_program(&semantic, &parsed.program, &path)?;

    let x_instances = compiled
        .algebra
        .variable_instances
        .iter()
        .map(|instance| instance.name.clone())
        .collect::<Vec<_>>();

    assert_eq!(
        x_instances,
        vec!["x[1,wind,g1,b1]".to_string(), "x[1,wind,g1,b2]".to_string(),]
    );

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn lowering_instantiates_constraint_bindings_from_tuple_subset_rows()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("tuple-subset-constraint-bindings")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("links.csv"),
        "area,tech,gen,bus,feasible\n1,wind,g1,b1,1\n1,wind,g1,b2,1\n1,solar,g2,b3,1\n2,solar,g3,b4,1\n",
    )?;

    let path = root.join("input.kdl");
    fs::write(
        &path,
        r#"
data "links" source="data/links.csv" {
  alias "generators" column="gen"
  alias "buses" column="bus"

  set "area" as="a"
  set "tech" as="i"
  set "generators" as="g"
  set "buses" as="b"

  set "feasible_links" {
    index "a" { in "area" }
    index "i" { in "tech" }
    index "g" { in "generators" }
    index "b" { in "buses" }
    where { feasible > 0 }
  }
}

set "target_pairs" {
  in "feasible_links"
  index "a" { in "area" }
  index "i" { in "tech" }
  where { generators == "g1" }
}

model "TupleDispatch" {
  control "x" lower=0 {
    index "a" { in "feasible_links" }
    index "i" { in "feasible_links" }
    index "g" { in "feasible_links" }
    index "b" { in "feasible_links" }
  }

  constraint "capacity_target" {
    index "a" { in "target_pairs" }
    index "i" { in "target_pairs" }
    expression {
      0 == 0
    }
  }

  minimize "Obj" { 0 }
}

scenario "S1" {
  use "TupleDispatch"
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic = validate_program(&parsed.program, &path)?;
    let compiled = compile_program(&semantic, &parsed.program, &path)?;

    let constraint_names = compiled
        .algebra
        .constraints
        .iter()
        .map(|constraint| constraint.name.clone())
        .collect::<Vec<_>>();

    assert_eq!(
        constraint_names,
        vec!["capacity_target[1,wind]".to_string()]
    );

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn lowering_rejects_constraint_auto_projection_from_high_dim_tuple_domain()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("tuple-subset-auto-projection")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("links.csv"),
        "area,tech,gen,bus,feasible\n1,wind,g1,b1,1\n",
    )?;

    let path = root.join("input.kdl");
    fs::write(
        &path,
        r#"
data "links" source="data/links.csv" {
  alias "generators" column="gen"
  alias "buses" column="bus"

  set "area" as="a"
  set "tech" as="i"
  set "generators" as="g"
  set "buses" as="b"

  set "feasible_links" {
    index "a" { in "area" }
    index "i" { in "tech" }
    index "g" { in "generators" }
    index "b" { in "buses" }
    where { feasible > 0 }
  }
}

model "TupleDispatch" {
  control "x" lower=0 {
    index "a" { in "feasible_links" }
    index "i" { in "feasible_links" }
    index "g" { in "feasible_links" }
    index "b" { in "feasible_links" }
  }

  constraint "bad_projection" {
    index "a" { in "feasible_links" }
    expression { 0 == 0 }
  }

  minimize "Obj" { 0 }
}

scenario "S1" {
  use "TupleDispatch"
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic = validate_program(&parsed.program, &path)?;
    let error = compile_program(&semantic, &parsed.program, &path)
        .expect_err("auto projection from tuple domains should fail in V1");

    match error {
        CompileError::InvalidFormulation { message, .. } => {
            assert!(message.contains("index order mismatch for `bad_projection`"));
            assert!(message.contains("expected `a,i,g,b`"));
            assert!(message.contains("received `a`"));
        }
        other => panic!("expected InvalidFormulation, got {other:?}"),
    }

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn lowering_reports_all_empty_constraint_relevant_tuple_subset_keys()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("tuple-subset-empty-keys")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("links.csv"),
        "area,tech,gen,bus,feasible\n1,wind,g1,b1,1\n",
    )?;

    let path = root.join("input.kdl");
    fs::write(
        &path,
        r#"
set "area" { "2"; "1" }
set "tech" { "wind"; "solar" }

data "links" source="data/links.csv" {
  alias "generators" column="gen"
  alias "buses" column="bus"

  set "feasible_links" {
    index "a" { in "area" }
    index "i" { in "tech" }
    index "g" { in "generators" }
    index "b" { in "buses" }
    where { feasible > 0 }
  }
}

model "TupleDispatch" {
  control "x" lower=0 {
    index "a" { in "feasible_links" }
    index "i" { in "feasible_links" }
    index "g" { in "feasible_links" }
    index "b" { in "feasible_links" }
  }

  constraint "capacity_target" {
    index "a" { in "area" }
    index "i" { in "tech" }
    expression {
      sum(1 for g in feasible_links for b in feasible_links) == 1
    }
  }

  minimize "Obj" { 0 }
}

scenario "S1" {
  use "TupleDispatch"
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic = validate_program(&parsed.program, &path)?;
    let error = compile_program(&semantic, &parsed.program, &path)
        .expect_err("empty tuple-domain reductions should fail with all offending keys");

    match error {
        CompileError::InvalidFormulation { message, .. } => {
            assert!(
                message.contains("empty constraint-relevant tuple subset for `capacity_target`")
            );
            assert!(message.contains("1,solar; 1,wind; 2,solar; 2,wind"));
        }
        other => panic!("expected InvalidFormulation, got {other:?}"),
    }

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn lowering_exports_scoped_inferred_constraint_ids() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("scoped-inferred-constraint-ids")?;
    let path = root.join("input.kdl");
    fs::write(
        &path,
        r#"
model "Dispatch" {
  constraint {
    expression { 0 == 0 }
  }

  minimize "Obj" { 0 }
}

scenario "S1" {
  use "Dispatch"
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic = validate_program(&parsed.program, &path)?;
    let compiled = compile_program(&semantic, &parsed.program, &path)?;

    assert_eq!(semantic.active_constraints.len(), 1);
    assert_eq!(semantic.active_constraints[0].name, "constraint_1");
    assert_eq!(
        semantic.active_constraints[0].diagnostic_id,
        "S1.Dispatch.constraint_1"
    );

    assert_eq!(compiled.constraints.len(), 1);
    assert_eq!(compiled.constraints[0].name, "constraint_1");
    assert_eq!(
        compiled.constraints[0].diagnostic_id,
        "S1.Dispatch.constraint_1"
    );

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn lowering_reports_scoped_inferred_constraint_ids_in_tuple_projection_errors()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("tuple-projection-scoped-constraint-id")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("links.csv"),
        "area,tech,gen,bus,feasible\n1,wind,g1,b1,1\n",
    )?;

    let path = root.join("input.kdl");
    fs::write(
        &path,
        r#"
data "links" source="data/links.csv" {
  alias "generators" column="gen"
  alias "buses" column="bus"

  set "feasible_links" {
    index "a" { in "area" }
    index "i" { in "tech" }
    index "g" { in "generators" }
    index "b" { in "buses" }
    where { feasible > 0 }
  }
}

model "TupleDispatch" {
  control "x" lower=0 {
    index "a" { in "feasible_links" }
    index "i" { in "feasible_links" }
    index "g" { in "feasible_links" }
    index "b" { in "feasible_links" }
  }

  constraint {
    index "a" { in "feasible_links" }
    expression { 0 == 0 }
  }

  minimize "Obj" { 0 }
}

scenario "S1" {
  use "TupleDispatch"
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic = validate_program(&parsed.program, &path)?;
    let error = compile_program(&semantic, &parsed.program, &path)
        .expect_err("auto projection from tuple domains should fail in V1");

    match error {
        CompileError::InvalidFormulation { message, .. } => {
            assert!(message.contains("index order mismatch for `S1.TupleDispatch.constraint_1`"));
            assert!(message.contains("expected `a,i,g,b`"));
            assert!(message.contains("received `a`"));
        }
        other => panic!("expected InvalidFormulation, got {other:?}"),
    }

    fs::remove_dir_all(&root)?;
    Ok(())
}
