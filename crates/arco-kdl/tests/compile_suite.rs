#![allow(clippy::float_cmp)]

mod common;

use arco_kdl::compile::{CompileError, compile_program};
use arco_kdl::semantic::validate_program;
use arco_kdl::source::parse_program_file;
use common::write_fixture_to_path;
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

fn repo_example_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
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
    write_fixture_to_path("lowering_loads_top_level_data_block_params.kdl", &path)?;

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
    write_fixture_to_path(
        "lowering_prefers_scenario_data_bindings_over_top_level_data_params.kdl",
        &path,
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
    write_fixture_to_path(
        "lowering_reports_missing_data_point_for_sparse_generic_data_table.kdl",
        &path,
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
    write_fixture_to_path("lowering_applies_data_param_filters.kdl", &path)?;

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
    write_fixture_to_path(
        "lowering_applies_data_param_filters_with_bare_identifier_rhs.kdl",
        &path,
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
fn lowering_applies_data_param_filters_with_mapped_identifier_rhs()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("data-param-filter-mapped-identifier")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("load.csv"),
        "period,technology,demand\n1,wind,10\n1,solar,90\n2,wind,20\n2,solar,80\n",
    )?;

    let path = root.join("input.kdl");
    write_fixture_to_path(
        "lowering_applies_data_param_filters_with_mapped_identifier_rhs.kdl",
        &path,
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
    write_fixture_to_path(
        "lowering_instantiates_tuple_domain_variables_from_data_rows.kdl",
        &path,
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
    write_fixture_to_path(
        "lowering_tuple_domain_instantiation_handles_alias_and_canonical_set_names.kdl",
        &path,
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
    write_fixture_to_path(
        "lowering_intersects_data_and_rule_tuple_sources_for_domain.kdl",
        &path,
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
    write_fixture_to_path(
        "lowering_instantiates_constraint_bindings_from_tuple_subset_rows.kdl",
        &path,
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
    write_fixture_to_path(
        "lowering_rejects_constraint_auto_projection_from_high_dim_tuple_domain.kdl",
        &path,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic = validate_program(&parsed.program, &path)?;
    let error = compile_program(&semantic, &parsed.program, &path)
        .expect_err("auto projection from tuple domains should fail in V1");

    match error {
        CompileError::InvalidFormulation { message, .. } => {
            assert!(message.contains("index order mismatch for `bad_projection`"));
            assert!(message.contains("tuple domain `feasible_links`"));
            assert!(message.contains("expected `a,i,g,b`"));
            assert!(message.contains("received `a`"));
        }
        other => panic!("expected InvalidFormulation, got {other:?}"),
    }

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn lowering_reports_tuple_domain_provenance_for_variable_index_order_mismatches()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("tuple-domain-variable-order")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("links.csv"),
        "area,tech,gen,bus,feasible\n1,wind,g1,b1,1\n",
    )?;

    let path = root.join("input.kdl");
    write_fixture_to_path(
        "lowering_reports_tuple_domain_provenance_for_variable_index_order_mismatches.kdl",
        &path,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic = validate_program(&parsed.program, &path)?;
    let error = compile_program(&semantic, &parsed.program, &path)
        .expect_err("tuple-domain variable order mismatches should fail compilation");

    match error {
        CompileError::InvalidFormulation { message, .. } => {
            assert!(message.contains("index order mismatch for `x[a,g,i,b]`"));
            assert!(message.contains("tuple domain `feasible_links`"));
            assert!(message.contains("expected `a,i,g,b`"));
            assert!(message.contains("received `a,g,i,b`"));
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
    write_fixture_to_path(
        "lowering_reports_all_empty_constraint_relevant_tuple_subset_keys.kdl",
        &path,
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
            assert!(message.contains("1,solar"));
            assert!(message.contains("2,wind"));
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
    write_fixture_to_path("lowering_exports_scoped_inferred_constraint_ids.kdl", &path)?;

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
fn lowering_example_nodal_allocation_preserves_sparse_tuple_membership()
-> Result<(), Box<dyn std::error::Error>> {
    let path = repo_example_path("examples/nodal-allocation/input.kdl");

    let parsed = parse_program_file(&path)?;
    let semantic = validate_program(&parsed.program, &path)?;
    let compiled = compile_program(&semantic, &parsed.program, &path)?;

    let investment_instances = compiled
        .algebra
        .variable_instances
        .iter()
        .map(|instance| instance.name.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        investment_instances,
        vec![
            "investment[north,wind,g1,b1]".to_string(),
            "investment[north,wind,g2,b2]".to_string(),
            "investment[south,gas,g4,b3]".to_string(),
            "investment[south,solar,g3,b3]".to_string(),
        ]
    );

    let priority_constraints = compiled
        .algebra
        .constraints
        .iter()
        .filter(|constraint| constraint.name.starts_with("priority_floor"))
        .map(|constraint| constraint.name.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        priority_constraints,
        vec![
            "priority_floor[south,b3,g3,solar]".to_string(),
            "priority_floor[south,b3,g4,gas]".to_string(),
        ]
    );

    Ok(())
}

#[test]
fn lowering_preserves_numeric_tuple_index_labels_in_expression_lookups()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("numeric-tuple-index-labels")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("links.csv"),
        "area,tech,bus,gen,mw_target\n1.0,solar,bus_101,solar_1,50\n",
    )?;

    let path = root.join("input.kdl");
    write_fixture_to_path(
        "lowering_preserves_numeric_tuple_index_labels_in_expression_lookups.kdl",
        &path,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic = validate_program(&parsed.program, &path)?;
    let compiled = compile_program(&semantic, &parsed.program, &path)?;

    assert!(
        compiled
            .algebra
            .variable_instances
            .iter()
            .any(|instance| instance.name
                == "nodal_site_capacity_variable[1.0,solar,bus_101,solar_1]")
    );

    let capacity_constraints = compiled
        .algebra
        .constraints
        .iter()
        .filter(|constraint| constraint.name.starts_with("capacity_target"))
        .count();
    assert_eq!(capacity_constraints, 1);

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
    write_fixture_to_path(
        "lowering_reports_scoped_inferred_constraint_ids_in_tuple_projection_errors.kdl",
        &path,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic = validate_program(&parsed.program, &path)?;
    let error = compile_program(&semantic, &parsed.program, &path)
        .expect_err("auto projection from tuple domains should fail in V1");

    match error {
        CompileError::InvalidFormulation { message, .. } => {
            assert!(message.contains("index order mismatch for `S1.TupleDispatch.constraint_1`"));
            assert!(message.contains("tuple domain `feasible_links`"));
            assert!(message.contains("expected `a,i,g,b`"));
            assert!(message.contains("received `a`"));
        }
        other => panic!("expected InvalidFormulation, got {other:?}"),
    }

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn lowering_unpacks_tuple_set_shorthand_for_control_and_constraint_bindings()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("tuple-shorthand-unpack")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("links.csv"),
        "area,tech,gen,bus,feasible\n1,wind,g1,b1,1\n2,solar,g2,b3,1\n",
    )?;

    let path = root.join("input.kdl");
    write_fixture_to_path(
        "lowering_unpacks_tuple_set_shorthand_for_control_and_constraint_bindings.kdl",
        &path,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic = validate_program(&parsed.program, &path)?;
    let compiled = compile_program(&semantic, &parsed.program, &path)?;

    let variable_names = compiled
        .algebra
        .variable_instances
        .iter()
        .map(|instance| instance.name.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        variable_names,
        vec![
            "x[1,wind,g1,b1]".to_string(),
            "x[2,solar,g2,b3]".to_string()
        ]
    );

    let constraint_names = compiled
        .algebra
        .constraints
        .iter()
        .map(|constraint| constraint.name.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        constraint_names,
        vec![
            "cap[1,b1,g1,wind]".to_string(),
            "cap[2,b3,g2,solar]".to_string()
        ]
    );

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn lowering_allows_tuple_key_indexing_with_tuple_set_name() -> Result<(), Box<dyn std::error::Error>>
{
    let root = temp_test_dir("tuple-key-indexing")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("links.csv"),
        "area,tech,gen,bus,feasible\n1,wind,g1,b1,1\n2,solar,g2,b3,1\n",
    )?;

    let path = root.join("input.kdl");
    write_fixture_to_path(
        "lowering_allows_tuple_key_indexing_with_tuple_set_name.kdl",
        &path,
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
        vec![
            "cap[1,b1,g1,wind]".to_string(),
            "cap[2,b3,g2,solar]".to_string()
        ]
    );

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn lowering_allows_parent_indexed_symbol_lookup_with_subset_tuple_key()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("tuple-key-subset")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("links.csv"),
        "area,tech,gen,bus,feasible\n1,wind,g1,b1,1\n1,solar,g2,b2,1\n2,solar,g3,b3,1\n",
    )?;

    let path = root.join("input.kdl");
    write_fixture_to_path(
        "lowering_allows_parent_indexed_symbol_lookup_with_subset_tuple_key.kdl",
        &path,
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
        vec![
            "cap[1,b1,g1,wind]".to_string(),
            "cap[1,b2,g2,solar]".to_string()
        ]
    );

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn lowering_accepts_data_param_tuple_shorthand_indexing() -> Result<(), Box<dyn std::error::Error>>
{
    let root = temp_test_dir("tuple-param-shorthand")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("links.csv"),
        "area,tech,gen,bus,cap,feasible\n1,wind,g1,b1,10,1\n2,solar,g2,b3,20,1\n",
    )?;

    let path = root.join("input.kdl");
    write_fixture_to_path(
        "lowering_accepts_data_param_tuple_shorthand_indexing.kdl",
        &path,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic = validate_program(&parsed.program, &path)?;
    let compiled = compile_program(&semantic, &parsed.program, &path)?;

    let rhs_values = compiled
        .algebra
        .constraints
        .iter()
        .map(|constraint| constraint.rhs)
        .collect::<Vec<_>>();
    assert_eq!(rhs_values, vec![10.0, 20.0]);

    fs::remove_dir_all(&root)?;
    Ok(())
}
