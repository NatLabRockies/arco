mod common;
use arco_compile::semantic::validate_program;
use arco_kdl::source::parse_program_text;
use common::{fixture_text, temp_root};
use std::fs;

#[test]
fn semantic_validation_rejects_missing_scenario() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-no-scenario")?;
    let path = root.join("input.kdl");
    let text = fixture_text("semantic_validation_rejects_missing_scenario.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
    let error = validate_program(&parsed.program, &path).expect_err("expected semantic failure");
    assert!(error.to_string().contains("no scenario is available"));

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn semantic_validation_requires_single_scenario() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("semantic-multi-scenario")?;
    let path = root.join("input.kdl");
    let text = fixture_text("semantic_validation_requires_single_scenario.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text = fixture_text(
        "semantic_validation_requires_scenario_data_bindings_to_match_known_params.kdl",
    )?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text =
        fixture_text("semantic_validation_resolves_reports_and_registry_for_low_level_model.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text = fixture_text("semantic_validation_applies_string_set_filters.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text = fixture_text("semantic_validation_applies_bare_identifier_set_filters.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text = fixture_text("semantic_validation_applies_mapped_column_set_filters.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text =
        fixture_text("semantic_validation_applies_set_filter_with_parent_alias_and_bare_rhs.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text = fixture_text("semantic_validation_applies_numeric_set_filters.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text =
        fixture_text("semantic_validation_applies_subset_filters_using_parent_alias_column.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text = fixture_text("semantic_validation_rejects_unresolved_subset_filter_identifier.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text =
        fixture_text("semantic_validation_rejects_unresolved_standalone_filter_identifier.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text = fixture_text("semantic_validation_rejects_unresolved_param_filter_identifier.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text =
        fixture_text("semantic_validation_rejects_tuple_set_with_missing_component_column.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text =
        fixture_text("semantic_validation_rejects_unresolved_rule_set_filter_identifier.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text =
        fixture_text("semantic_validation_rejects_tuple_set_schema_mismatch_across_sources.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text = fixture_text(
        "semantic_validation_rejects_tuple_set_domain_schema_mismatch_across_sources.kdl",
    )?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text = fixture_text(
        "semantic_validation_rejects_tuple_subset_with_component_domain_mismatch.kdl",
    )?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text =
        fixture_text("semantic_validation_rejects_duplicate_data_tuple_rows_with_provenance.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text =
        fixture_text("semantic_validation_rejects_duplicate_rule_tuple_rows_with_provenance.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text = fixture_text(
        "semantic_validation_reports_deterministic_inferred_rule_ids_in_duplicate_diagnostics.kdl",
    )?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text =
        fixture_text("semantic_validation_prefers_user_rule_ids_in_duplicate_diagnostics.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text =
        fixture_text("semantic_validation_rejects_projection_with_unknown_source_domain.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text = fixture_text("semantic_validation_rejects_projection_with_unknown_target_key.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text =
        fixture_text("semantic_validation_rejects_projection_with_non_tuple_source_domain.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text =
        fixture_text("semantic_validation_rejects_projection_without_dimensional_reduction.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text = fixture_text(
        "semantic_validation_rejects_reduce_projection_with_incompatible_target_signature.kdl",
    )?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text = fixture_text(
        "semantic_validation_accepts_reduce_projection_with_matching_target_signature.kdl",
    )?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text = fixture_text("semantic_validation_rejects_reduce_projection_non_sum_operator.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
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
    let text =
        fixture_text("semantic_validation_assigns_stable_scoped_inferred_constraint_ids.kdl")?;

    let parsed = parse_program_text(&text, &path)?;
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
    assert_duplicate_model_declaration_fails(
        "semantic-duplicate-expression-name",
        "semantic_validation_rejects_duplicate_expression_names_in_model.kdl",
        "expression",
        "cost",
    )
}

#[test]
fn semantic_validation_rejects_duplicate_constraint_names_in_model()
-> Result<(), Box<dyn std::error::Error>> {
    assert_duplicate_model_declaration_fails(
        "semantic-duplicate-constraint-name",
        "semantic_validation_rejects_duplicate_constraint_names_in_model.kdl",
        "constraint",
        "bal",
    )
}

#[test]
fn semantic_validation_rejects_duplicate_control_names_in_model()
-> Result<(), Box<dyn std::error::Error>> {
    assert_duplicate_model_declaration_fails(
        "semantic-duplicate-control-name",
        "semantic_validation_rejects_duplicate_control_names_in_model.kdl",
        "control",
        "x",
    )
}

fn assert_duplicate_model_declaration_fails(
    test_name: &str,
    fixture: &str,
    kind: &str,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root(test_name)?;
    let path = root.join("input.kdl");
    let text = fixture_text(fixture)?;

    let parsed = parse_program_text(&text, &path)?;
    let error = validate_program(&parsed.program, &path)
        .expect_err("duplicate model declaration names should fail semantic validation");

    assert!(error.to_string().contains("duplicate"));
    assert!(error.to_string().contains(kind));
    assert!(error.to_string().contains(name));

    fs::remove_dir_all(&root)?;
    Ok(())
}
