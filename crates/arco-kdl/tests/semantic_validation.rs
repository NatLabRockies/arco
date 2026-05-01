mod common;
use arco_kdl::semantic::validate_program;
use arco_kdl::source::parse_program_text;
use common::temp_root;
use std::fs;

#[test]
fn semantic_validation_rejects_missing_scenario() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-no-scenario")?;
    let path = root.join("input.kdl");
    let text = r#"
model "Dispatch" {
  control "x" {
    index "a"
    index "t"
  }
  minimize "Obj" {
    sum(x[a,t] for a in assets for t in time)
  }
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path).expect_err("expected semantic failure");
    assert!(error.to_string().contains("no scenario is available"));

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_requires_single_scenario() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-multi-scenario")?;
    let path = root.join("input.kdl");
    let text = r#"
model "Dispatch" {
  control "x" {
    index "a"
    index "t"
  }
  minimize "Obj" {
    sum(x[a,t] for a in assets for t in time)
  }
}

scenario "S1" {
  use "Dispatch"
}

scenario "S2" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path).expect_err("expected semantic failure");
    assert!(error.to_string().contains("exactly one scenario"));

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_requires_scenario_data_bindings_to_match_known_params()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-data-param-match")?;
    let path = root.join("input.kdl");
    let text = r#"
model "Dispatch" {
  param "demand" {
    index "t"
  }
  control "x" {
    index "a"
    index "t"
  }
  constraint "bal[a,t]" {
    x[a,t] <= demand[t]
  }
  minimize "Obj" {
    sum(x[a,t] for a in assets for t in time)
  }
}

scenario "S1" {
  use "Dispatch"
  data "unknown_param" source="data.csv"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path).expect_err("expected semantic failure");
    assert!(error.to_string().contains("unknown_param"));
    assert!(error.to_string().contains("parameter"));

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_resolves_reports_and_registry_for_low_level_model()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-reports-registry")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("assets.csv"),
        "asset_id,is_candidate,zone\nA,1,north\nB,0,south\n",
    )?;

    let path = root.join("input.kdl");
    let text = r#"
set "time" { "1"; "2" }

data "generator_data" source="data/assets.csv" {
  set "asset_id"
}

model "Dispatch" {
  control "x" {
    index "a"
    index "t"
  }

  expression "FuelCost" {
    sum(x[a,t] for a in asset_id for t in time)
  }

  constraint "balance[a,t]" {
    x[a,t] = 0
  }

  minimize "TotalCost" {
    FuelCost
  }
}

scenario "S1" {
  use "Dispatch"
  report FuelCost
  report TotalCost
  report dual "balance[a,t]"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let semantic = validate_program(&parsed.program, &path)?;

    assert_eq!(semantic.active_scenario, "S1");
    assert_eq!(semantic.active_objective.name, "TotalCost");
    assert_eq!(semantic.variable_families.len(), 1);
    assert_eq!(semantic.variable_families[0].target, "x");
    assert_eq!(semantic.variable_families[0].indices, vec!["a", "t"]);
    assert_eq!(semantic.active_reports.len(), 2);
    assert_eq!(semantic.active_dual_reports.len(), 1);
    assert!(
        semantic
            .active_expressions
            .iter()
            .any(|e| e.name == "FuelCost")
    );
    assert!(semantic.set_registry.contains_key("time"));
    assert!(semantic.set_registry.contains_key("asset_id"));

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_applies_string_set_filters() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-set-filter-string")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("tech.csv"),
        "tech\nwind\nsolar\nwind\n",
    )?;

    let path = root.join("input.kdl");
    let text = r#"
data "tech_data" source="data/tech.csv" {
  set "tech"
  set "wind_tech" {
    in "tech"
    filter { tech == "wind" }
  }
}

model "Dispatch" {
  set "tech"

  control "x" {
    index "tech"
  }

  minimize "Obj" {
    sum(x[tech] for tech in tech)
  }
}

scenario "S1" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let semantic = validate_program(&parsed.program, &path)?;

    let wind_tech = semantic
        .set_registry
        .get("wind_tech")
        .ok_or("missing set wind_tech")?;
    assert_eq!(wind_tech.values, vec!["wind".to_string()]);

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_applies_bare_identifier_set_filters()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-set-filter-bare-identifier")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("tech.csv"),
        "tech\nwind\nsolar\nwind\n",
    )?;

    let path = root.join("input.kdl");
    let text = r#"
data "tech_data" source="data/tech.csv" {
  set "tech"
  set "wind_tech" {
    in "tech"
    // Spec: §9 — bare RHS is categorical literal, not column lookup.
    filter { tech == wind }
  }
}

model "Dispatch" {
  set "tech"

  control "x" {
    index "tech"
  }

  minimize "Obj" {
    sum(x[tech] for tech in tech)
  }
}

scenario "S1" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let semantic = validate_program(&parsed.program, &path)?;

    let wind_tech = semantic
        .set_registry
        .get("wind_tech")
        .ok_or("missing set wind_tech")?;
    assert_eq!(wind_tech.values, vec!["wind".to_string()]);

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_applies_mapped_column_set_filters() -> Result<(), Box<dyn std::error::Error>>
{
    let root = temp_root("semantic-set-filter-mapped-column")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("tech.csv"),
        "technology\nwind\nsolar\nwind\n",
    )?;

    let path = root.join("input.kdl");
    let text = r#"
data "tech_data" source="data/tech.csv" {
  map "tech" from="technology"

  set "tech"
  set "wind_tech" {
    in "tech"
    // Spec: §9 — map applies to lhs, RHS bare token remains literal.
    filter { tech == wind }
  }
}

model "Dispatch" {
  set "tech"

  control "x" {
    index "tech"
  }

  minimize "Obj" {
    sum(x[tech] for tech in tech)
  }
}

scenario "S1" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let semantic = validate_program(&parsed.program, &path)?;

    let wind_tech = semantic
        .set_registry
        .get("wind_tech")
        .ok_or("missing set wind_tech")?;
    assert_eq!(wind_tech.values, vec!["wind".to_string()]);

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_applies_set_filter_with_parent_alias_and_bare_rhs()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-subset-filter-parent-bare-rhs")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("assets.csv"),
        "asset_name,zone_raw\nA,north\nB,south\nC,north\n",
    )?;

    let path = root.join("input.kdl");
    let text = r#"
data "assets_data" source="data/assets.csv" {
  map "asset_id" from="asset_name"
  map "zone" from="zone_raw"

  set "asset_id" alias="a"
  set "south_assets" {
    in "a"
    // Spec: §9 — alias resolution and bare RHS semantics together.
    filter { zone == south }
  }
}

model "Dispatch" {
  set "asset_id" alias="a"

  control "x" {
    index "a"
  }

  minimize "Obj" {
    sum(x[a] for a in asset_id)
  }
}

scenario "S1" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let semantic = validate_program(&parsed.program, &path)?;

    let south_assets = semantic
        .set_registry
        .get("south_assets")
        .ok_or("missing set south_assets")?;
    assert_eq!(south_assets.values, vec!["B".to_string()]);

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_applies_numeric_set_filters() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-set-filter-numeric")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("assets.csv"),
        "asset_id,is_candidate\nA,1\nB,0\nC,2\n",
    )?;

    let path = root.join("input.kdl");
    let text = r#"
data "assets_data" source="data/assets.csv" {
  set "asset_id" alias="a"
  set "candidate_assets" {
    in "asset_id"
    filter { is_candidate > 0 }
  }
}

model "Dispatch" {
  set "asset_id" alias="a"

  control "x" {
    index "a"
  }

  minimize "Obj" {
    sum(x[a] for a in asset_id)
  }
}

scenario "S1" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let semantic = validate_program(&parsed.program, &path)?;

    let candidate_assets = semantic
        .set_registry
        .get("candidate_assets")
        .ok_or("missing set candidate_assets")?;
    assert_eq!(
        candidate_assets.values,
        vec!["A".to_string(), "C".to_string()]
    );

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_applies_subset_filters_using_parent_alias_column()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-subset-filter-parent")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("assets.csv"),
        "asset_name,zone\nA,north\nB,south\nC,north\n",
    )?;

    let path = root.join("input.kdl");
    let text = r#"
data "assets_data" source="data/assets.csv" {
  map "asset_id" from="asset_name"

  set "asset_id" alias="a"
  set "north_assets" {
    in "a"
    filter { zone == "north" }
  }
}

model "Dispatch" {
  set "asset_id" alias="a"

  control "x" {
    index "a"
  }

  minimize "Obj" {
    sum(x[a] for a in asset_id)
  }
}

scenario "S1" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let semantic = validate_program(&parsed.program, &path)?;

    let north_assets = semantic
        .set_registry
        .get("north_assets")
        .ok_or("missing set north_assets")?;
    assert_eq!(north_assets.values, vec!["A".to_string(), "C".to_string()]);

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_rejects_unresolved_subset_filter_identifier()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-set-filter-unresolved-identifier")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("tech.csv"),
        "tech\nwind\nsolar\nwind\n",
    )?;

    let path = root.join("input.kdl");
    let text = r#"
data "tech_data" source="data/tech.csv" {
  set "tech"
  set "wind_tech" {
    in "tech"
    filter { unknown_col == wind }
  }
}

model "Dispatch" {
  set "tech"

  control "x" {
    index "tech"
  }

  minimize "Obj" {
    sum(x[tech] for tech in tech)
  }
}

scenario "S1" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path)
        .expect_err("unresolved filter identifier should fail semantic validation");

    assert!(error.to_string().contains("unknown_col"));

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_rejects_unresolved_standalone_filter_identifier()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-set-filter-unresolved-standalone-identifier")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("tech.csv"),
        "tech\nwind\nsolar\nwind\n",
    )?;

    let path = root.join("input.kdl");
    let text = r#"
data "tech_data" source="data/tech.csv" {
  set "tech"
  set "wind_tech" {
    in "tech"
    filter { unknown_col }
  }
}

model "Dispatch" {
  set "tech"

  control "x" {
    index "tech"
  }

  minimize "Obj" {
    sum(x[tech] for tech in tech)
  }
}

scenario "S1" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path)
        .expect_err("unresolved standalone filter identifier should fail semantic validation");

    assert!(error.to_string().contains("unknown_col"));

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_rejects_unresolved_param_filter_identifier()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-param-filter-unresolved-identifier")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("load.csv"),
        "period,tech,demand\n1,wind,10\n2,solar,20\n",
    )?;

    let path = root.join("input.kdl");
    let text = r#"
set "time" { "1"; "2" }

data "inputs" source="data/load.csv" {
  map "time" from="period"

  param "demand" from="demand" reduce="sum" {
    index "time"
    filter { unknown_col == wind }
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
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path)
        .expect_err("unresolved param filter identifier should fail semantic validation");

    assert!(error.to_string().contains("unknown_col"));

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_rejects_tuple_set_with_missing_component_column()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-tuple-set-missing-component")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("links.csv"),
        "area,tech,gen,feasible\n1,wind,g1,1\n",
    )?;

    let path = root.join("input.kdl");
    let text = r#"
data "links" source="data/links.csv" {
  alias "generators" column="gen"
  alias "buses" column="bus"

  set "feasible_links" {
    index "a" { in "area" }
    index "i" { in "tech" }
    index "g" { in "generators" }
    index "b" { in "buses" }
    filter { feasible > 0 }
  }
}

model "Dispatch" {
  control "x" {
    index "a" { in "feasible_links" }
    index "i" { in "feasible_links" }
    index "g" { in "feasible_links" }
    index "b" { in "feasible_links" }
  }

  minimize "Obj" { 0 }
}

scenario "S1" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path)
        .expect_err("missing tuple component column should fail semantic validation");

    assert!(error.to_string().contains("missing required column"));
    assert!(error.to_string().contains("bus"));

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_rejects_unresolved_rule_set_filter_identifier()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-rule-set-filter-unresolved-identifier")?;
    let path = root.join("input.kdl");
    let text = r#"
set "area" { "1"; "2" }
set "tech" { "wind"; "solar" }

set "feasible_links" {
  index "a" { in "area" }
  index "i" { in "tech" }
  filter { unknown_col == "1" }
}

model "Dispatch" {
  control "x" {
    index "a" { in "feasible_links" }
    index "i" { in "feasible_links" }
  }

  minimize "Obj" { 0 }
}

scenario "S1" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path)
        .expect_err("unresolved rule-set filter identifier should fail semantic validation");

    assert!(
        error
            .to_string()
            .contains("unresolved identifier `unknown_col`")
    );
    assert!(error.to_string().contains("top-level set filter"));

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_rejects_tuple_set_schema_mismatch_across_sources()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-tuple-set-schema-mismatch")?;
    let path = root.join("input.kdl");
    let text = r#"
set "area" { "1"; "2" }
set "tech" { "wind"; "solar" }

set "feasible_links" {
  index "a" { in "area" }
  index "i" { in "tech" }
}

set "feasible_links" {
  index "i" { in "tech" }
  index "a" { in "area" }
}

model "Dispatch" {
  control "x" {
    index "a" { in "feasible_links" }
    index "i" { in "feasible_links" }
  }

  minimize "Obj" { 0 }
}

scenario "S1" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path).expect_err(
        "tuple set schema mismatch across tuple sources should fail semantic validation",
    );

    assert!(
        error
            .to_string()
            .contains("tuple component schema mismatch")
    );
    assert!(error.to_string().contains("existing `a,i`"));
    assert!(error.to_string().contains("incoming `i,a`"));

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_rejects_tuple_set_domain_schema_mismatch_across_sources()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-tuple-set-domain-schema-mismatch")?;
    let path = root.join("input.kdl");
    let text = r#"
set "area" { "1"; "2" }
set "region" { "1"; "2" }
set "tech" { "wind"; "solar" }

set "feasible_links" {
  index "a" { in "area" }
  index "i" { in "tech" }
}

set "feasible_links" {
  index "a" { in "region" }
  index "i" { in "tech" }
}

model "Dispatch" {
  control "x" {
    index "a" { in "feasible_links" }
    index "i" { in "feasible_links" }
  }

  minimize "Obj" { 0 }
}

scenario "S1" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path).expect_err(
        "tuple set domain schema mismatch across tuple sources should fail semantic validation",
    );

    assert!(
        error
            .to_string()
            .contains("tuple component schema mismatch")
    );
    assert!(error.to_string().contains("domains: area,tech"));
    assert!(error.to_string().contains("domains: region,tech"));

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_rejects_tuple_subset_with_component_domain_mismatch()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-tuple-subset-domain-mismatch")?;
    let path = root.join("input.kdl");
    let text = r#"
set "area" { "1"; "2" }
set "tech" { "wind"; "solar" }

set "feasible_links" {
  index "a" { in "area" }
  index "i" { in "tech" }
}

set "target_pairs" {
  in "feasible_links"
  index "a" { in "tech" }
}

model "Dispatch" {
  control "x" {
    index "a" { in "feasible_links" }
    index "i" { in "feasible_links" }
  }

  minimize "Obj" { 0 }
}

scenario "S1" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path)
        .expect_err("tuple subset domain mismatch should fail semantic validation");

    assert!(
        error
            .to_string()
            .contains("tuple subset index `a` in `target_pairs` declares domain `tech`")
    );
    assert!(error.to_string().contains("parent tuple domain is `area`"));

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_rejects_duplicate_data_tuple_rows_with_provenance()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-duplicate-data-tuples")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("links.csv"),
        "area,tech,gen,bus,feasible\n1,wind,g1,b1,1\n1,wind,g1,b1,1\n",
    )?;

    let path = root.join("input.kdl");
    let text = r#"
data "links" source="data/links.csv" {
  alias "generators" column="gen"
  alias "buses" column="bus"

  set "feasible_links" {
    index "a" { in "area" }
    index "i" { in "tech" }
    index "g" { in "generators" }
    index "b" { in "buses" }
    filter { feasible > 0 }
  }
}

model "Dispatch" {
  control "x" {
    index "a" { in "feasible_links" }
    index "i" { in "feasible_links" }
    index "g" { in "feasible_links" }
    index "b" { in "feasible_links" }
  }

  minimize "Obj" { 0 }
}

scenario "S1" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path)
        .expect_err("duplicate tuple rows from data should fail semantic validation");

    assert!(error.to_string().contains("duplicate feasible tuples"));
    assert!(error.to_string().contains("1,wind,g1,b1"));
    assert!(error.to_string().contains("data `data/links.csv` row 1"));
    assert!(error.to_string().contains("data `data/links.csv` row 2"));

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_rejects_duplicate_rule_tuple_rows_with_provenance()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-duplicate-rule-tuples")?;
    let path = root.join("input.kdl");
    let text = r#"
set "area" { "1"; "1" }
set "tech" { "wind" }

set "feasible_links" {
  index "a" { in "area" }
  index "i" { in "tech" }
}

model "Dispatch" {
  control "x" {
    index "a" { in "feasible_links" }
    index "i" { in "feasible_links" }
  }

  minimize "Obj" { 0 }
}

scenario "S1" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path)
        .expect_err("duplicate tuple rows from rule expansion should fail semantic validation");

    assert!(error.to_string().contains("duplicate feasible tuples"));
    assert!(error.to_string().contains("1,wind"));
    assert!(
        error
            .to_string()
            .contains("rule `feasible_links.rule_1` candidate #1")
    );
    assert!(
        error
            .to_string()
            .contains("rule `feasible_links.rule_1` candidate #2")
    );

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_reports_deterministic_inferred_rule_ids_in_duplicate_diagnostics()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-duplicate-rule-tuples-inferred-id")?;
    let path = root.join("input.kdl");
    let text = r#"
set "area" { "1"; "1" }
set "tech" { "wind" }

set "feasible_links" {
  index "a" { in "area" }
  index "i" { in "tech" }
}

model "Dispatch" {
  control "x" {
    index "a" { in "feasible_links" }
    index "i" { in "feasible_links" }
  }

  minimize "Obj" { 0 }
}

scenario "S1" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let first_error = validate_program(&parsed.program, &path)
        .expect_err("duplicate tuple rows from rule expansion should fail semantic validation");
    let first_message = first_error.to_string();

    let second_error = validate_program(&parsed.program, &path)
        .expect_err("duplicate tuple rows from rule expansion should fail semantic validation");
    let second_message = second_error.to_string();

    assert_eq!(first_message, second_message);
    assert!(first_message.contains("rule `feasible_links.rule_1` candidate #1"));
    assert!(first_message.contains("rule `feasible_links.rule_1` candidate #2"));

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_prefers_user_rule_ids_in_duplicate_diagnostics()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-duplicate-rule-tuples-user-id")?;
    let path = root.join("input.kdl");
    let text = r#"
set "area" { "1"; "1" }
set "tech" { "wind" }

set "feasible_links" id="LinksRule" {
  index "a" { in "area" }
  index "i" { in "tech" }
}

model "Dispatch" {
  control "x" {
    index "a" { in "feasible_links" }
    index "i" { in "feasible_links" }
  }

  minimize "Obj" { 0 }
}

scenario "S1" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path)
        .expect_err("duplicate tuple rows from rule expansion should fail semantic validation");

    let message = error.to_string();
    assert!(message.contains("rule `LinksRule` candidate #1"));
    assert!(message.contains("rule `LinksRule` candidate #2"));

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_rejects_projection_with_unknown_source_domain()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-projection-unknown-source-domain")?;
    let path = root.join("input.kdl");
    let text = r#"
projection "ai" {
  from "missing_links"
  to "a" "i"
}

model "Dispatch" {
  minimize "Obj" { 0 }
}

scenario "S1" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path)
        .expect_err("unknown projection source domain should fail semantic validation");

    assert!(error.to_string().contains("missing_links"));
    assert!(error.to_string().contains("projection source domain"));

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_rejects_projection_with_unknown_target_key()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-projection-unknown-target-key")?;
    let path = root.join("input.kdl");
    let text = r#"
set "area" { "a1" }
set "tech" { "solar" }
set "gen" { "g1" }
set "bus" { "b1" }

set "feasible_links" {
  index "a" { in "area" }
  index "i" { in "tech" }
  index "g" { in "gen" }
  index "b" { in "bus" }
}

projection "ai" {
  from "feasible_links"
  to "a" "z"
}

model "Dispatch" {
  minimize "Obj" { 0 }
}

scenario "S1" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path)
        .expect_err("unknown projection target key should fail semantic validation");

    assert!(error.to_string().contains("projection target key"));
    assert!(error.to_string().contains('z'));

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_rejects_projection_with_non_tuple_source_domain()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-projection-non-tuple-source-domain")?;
    let path = root.join("input.kdl");
    let text = r#"
set "area" { "a1" }

projection "ai" {
  from "area"
  to "a"
}

model "Dispatch" {
  minimize "Obj" { 0 }
}

scenario "S1" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path)
        .expect_err("non-tuple projection source domain should fail semantic validation");

    assert!(
        error
            .to_string()
            .contains("projection source tuple signature")
    );
    assert!(error.to_string().contains("area"));

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_rejects_projection_without_dimensional_reduction()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-projection-no-dim-reduction")?;
    let path = root.join("input.kdl");
    let text = r#"
set "area" { "a1" }
set "tech" { "solar" }

set "feasible_ai" {
  index "a" { in "area" }
  index "i" { in "tech" }
}

projection "ai" {
  from "feasible_ai"
  to "a" "i"
}

model "Dispatch" {
  minimize "Obj" { 0 }
}

scenario "S1" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path)
        .expect_err("identity projection should fail semantic validation");

    assert!(
        error
            .to_string()
            .contains("projection dimensional reduction")
    );
    assert!(error.to_string().contains("ai"));

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_rejects_reduce_projection_with_incompatible_target_signature()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-reduce-projection-signature-mismatch")?;
    let path = root.join("input.kdl");
    let text = r#"
set "area" { "a1" }
set "tech" { "solar" }
set "gen" { "g1" }
set "bus" { "b1" }

set "feasible_links" {
  index "a" { in "area" }
  index "i" { in "tech" }
  index "g" { in "gen" }
  index "b" { in "bus" }
}

projection "ai" {
  from "feasible_links"
  to "a" "i"
}

model "Dispatch" {
  control "investment" {
    index "a"
    index "i"
  }

  expression "investment_by_area_tech[a,i]" {
    reduce "ai" {
      sum "investment"
    }
  }

  minimize "Obj" { 0 }
}

scenario "S1" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path)
        .expect_err("reduce projection target signature mismatch should fail semantic validation");

    assert!(
        error
            .to_string()
            .contains("reduce projection target signature")
    );
    assert!(error.to_string().contains("investment"));

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_accepts_reduce_projection_with_matching_target_signature()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-reduce-projection-signature-match")?;
    let path = root.join("input.kdl");
    let text = r#"
set "area" { "a1" }
set "tech" { "solar" }
set "gen" { "g1" }
set "bus" { "b1" }

set "feasible_links" {
  index "a" { in "area" }
  index "i" { in "tech" }
  index "g" { in "gen" }
  index "b" { in "bus" }
}

projection "ai" {
  from "feasible_links"
  to "a" "i"
}

model "Dispatch" {
  control "investment" {
    index "a"
    index "i"
    index "g"
    index "b"
  }

  expression "investment_by_area_tech[a,i]" {
    reduce "ai" {
      sum "investment"
    }
  }

  minimize "Obj" { 0 }
}

scenario "S1" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let semantic = validate_program(&parsed.program, &path)?;
    assert_eq!(semantic.active_scenario, "S1");

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_rejects_reduce_projection_non_sum_operator()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-reduce-projection-non-sum")?;
    let path = root.join("input.kdl");
    let text = r#"
set "area" { "a1" }
set "tech" { "solar" }
set "gen" { "g1" }
set "bus" { "b1" }

set "feasible_links" {
  index "a" { in "area" }
  index "i" { in "tech" }
  index "g" { in "gen" }
  index "b" { in "bus" }
}

projection "ai" {
  from "feasible_links"
  to "a" "i"
}

model "Dispatch" {
  control "investment" {
    index "a"
    index "i"
    index "g"
    index "b"
  }

  expression "investment_by_area_tech[a,i]" {
    reduce "ai" {
      avg "investment"
    }
  }

  minimize "Obj" { 0 }
}

scenario "S1" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path)
        .expect_err("non-sum reduce projection operator should fail semantic validation");
    assert!(error.to_string().contains("reduce projection operation"));
    assert!(error.to_string().contains("avg"));

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_assigns_stable_scoped_inferred_constraint_ids()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-inferred-constraint-id")?;
    let path = root.join("input.kdl");
    let text = r#"
model "Dispatch" {
  constraint {
    expression { 0 == 0 }
  }

  minimize "Obj" { 0 }
}

scenario "S1" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let first_semantic = validate_program(&parsed.program, &path)?;
    let second_semantic = validate_program(&parsed.program, &path)?;

    assert_eq!(first_semantic.active_constraints.len(), 1);
    assert_eq!(second_semantic.active_constraints.len(), 1);

    assert_eq!(first_semantic.active_constraints[0].name, "constraint_1");
    assert_eq!(
        first_semantic.active_constraints[0].diagnostic_id,
        "S1.Dispatch.constraint_1"
    );
    assert_eq!(
        first_semantic.active_constraints[0].diagnostic_id,
        second_semantic.active_constraints[0].diagnostic_id
    );

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_rejects_duplicate_expression_names_in_model()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-duplicate-expression-name")?;
    let path = root.join("input.kdl");
    let text = r#"
model "Dispatch" {
  expression "cost" { 1 }
  expression "cost" { 2 }
  minimize "Obj" { cost }
}

scenario "S1" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path)
        .expect_err("duplicate model expression names should fail semantic validation");

    assert!(error.to_string().contains("duplicate"));
    assert!(error.to_string().contains("expression"));
    assert!(error.to_string().contains("cost"));

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_rejects_duplicate_constraint_names_in_model()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-duplicate-constraint-name")?;
    let path = root.join("input.kdl");
    let text = r#"
set "a" { "n1" }

model "Dispatch" {
  control "x" {
    index "i" { in "a" }
  }

  constraint "bal" {
    index "i" { in "a" }
    expression { x[i] <= 1 }
  }

  constraint "bal" {
    index "i" { in "a" }
    expression { x[i] >= 0 }
  }

  minimize "Obj" { sum(x[i] for i in a) }
}

scenario "S1" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path)
        .expect_err("duplicate model constraint names should fail semantic validation");

    assert!(error.to_string().contains("duplicate"));
    assert!(error.to_string().contains("constraint"));
    assert!(error.to_string().contains("bal"));

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_rejects_duplicate_control_names_in_model()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-duplicate-control-name")?;
    let path = root.join("input.kdl");
    let text = r#"
set "a" { "n1" }

model "Dispatch" {
  control "x" {
    index "i" { in "a" }
  }

  control "x" {
    index "i" { in "a" }
  }

  minimize "Obj" { 0 }
}

scenario "S1" {
  use "Dispatch"
}
"#;

    let parsed = parse_program_text(text, &path)?;
    let error = validate_program(&parsed.program, &path)
        .expect_err("duplicate model control names should fail semantic validation");

    assert!(error.to_string().contains("duplicate"));
    assert!(error.to_string().contains("control"));
    assert!(error.to_string().contains('x'));

    fs::remove_dir_all(&root)?;
    Ok(())
}
