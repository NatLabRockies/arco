use crate::algebra::constraint_mentions_previous_time;
use crate::semantic::error::SemanticError;
use crate::semantic::resolution::{
    resolve_active_model_expressions, resolve_model_scenario_reports,
};
use crate::semantic::sets::{
    build_set_registry, collect_set_aliases, extend_set_registry_from_low_level_declarations,
    load_set_csv,
};
use crate::semantic::types::{
    FamilySignature, ResolvedChronology, ResolvedConstraint, ResolvedObjective, ResolvedParameters,
    ResolvedSet, ResolvedSets, ResolvedTimeSet, SemanticProgram, VariableDeclOverrides,
};
use crate::source::{BoundExpr, ModelDecl, ScenarioDecl, SourceProgram, VariableKindDecl};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use tracing::info;

pub fn validate_program(
    program: &SourceProgram,
    entrypoint: &Path,
) -> Result<SemanticProgram, SemanticError> {
    info!(status = "ok", "validating program");

    if program.models.is_empty() {
        return Err(SemanticError::MissingModel {
            path: entrypoint.to_path_buf(),
        });
    }

    let scenario = resolve_scenario(program, entrypoint)?;
    let model_name = scenario
        .model_use
        .clone()
        .ok_or_else(|| SemanticError::MissingModelUse {
            scenario: scenario.name.clone(),
            path: entrypoint.to_path_buf(),
        })?;
    let model = program
        .model(&model_name)
        .ok_or_else(|| SemanticError::MissingDeclaration {
            kind: "model",
            name: model_name,
            path: entrypoint.to_path_buf(),
        })?;

    validate_scenario_data_bindings_match_model_params(scenario, model, entrypoint)?;

    let mut seen_data_bindings = BTreeSet::new();
    for binding in &scenario.data {
        if !seen_data_bindings.insert(binding.name.clone()) {
            return Err(SemanticError::DuplicateDataBinding {
                name: binding.name.clone(),
                path: entrypoint.to_path_buf(),
            });
        }
    }

    let mut series_parameters = BTreeSet::new();
    let mut indexed_parameters = BTreeSet::new();
    let mut asset_parameters = BTreeSet::new();
    for parameter in &model.parameters {
        classify_parameter_indices(
            &parameter.name,
            &parameter.indices,
            &mut series_parameters,
            &mut indexed_parameters,
            &mut asset_parameters,
        );
    }

    let active_constraints = model
        .constraints
        .iter()
        .map(|constraint| ResolvedConstraint {
            name: constraint.name.clone(),
            source_kind: "model".to_string(),
            source_name: model.name.clone(),
            expression_text: constraint.expression.clone(),
            expression: constraint.parsed_expression.clone(),
            generation_bindings: constraint.generation_bindings.clone(),
            generation_filter_text: constraint.generation_filter.clone(),
            generation_filter: constraint.parsed_generation_filter.clone(),
        })
        .collect::<Vec<_>>();

    let chronology = detect_model_chronology(&asset_parameters, scenario);
    if active_constraints
        .iter()
        .any(|constraint| constraint_mentions_previous_time(&constraint.expression))
        && chronology.initial_boundary.is_none()
        && chronology.initial_commitment_boundary.is_none()
    {
        return Err(SemanticError::MissingInitialBoundary {
            path: entrypoint.to_path_buf(),
        });
    }

    let active_objective = ResolvedObjective {
        name: model.optimize.name.clone(),
        sense: model.optimize.sense.clone(),
        expression_text: model.optimize.expression.clone(),
        expression: model.optimize.parsed_expression.clone(),
    };

    let (active_reports, active_dual_reports) =
        resolve_model_scenario_reports(model, scenario, &active_constraints, entrypoint)?;
    let active_expressions =
        resolve_active_model_expressions(model, &active_objective, &active_reports, entrypoint)?;

    let resolved_sets = ResolvedSets {
        time: ResolvedTimeSet {
            steps: scenario.horizon.steps,
            resolution: scenario.horizon.resolution.clone(),
        },
    };

    let mut set_registry = build_set_registry(&resolved_sets, &scenario.custom_sets);
    if let Some(entry_dir) = entrypoint.parent() {
        extend_set_registry_from_low_level_declarations(program, entry_dir, &mut set_registry)?;
    }

    let mut set_params: BTreeMap<String, BTreeMap<String, f64>> = BTreeMap::new();
    if let Some(entry_dir) = entrypoint.parent() {
        for set_binding in &scenario.set_bindings {
            let csv_path = entry_dir.join(&set_binding.source);
            let set_csv = load_set_csv(&csv_path)?;
            set_registry.insert(
                set_binding.name.clone(),
                ResolvedSet {
                    values: set_csv.members,
                },
            );
            set_params.extend(set_csv.params);
        }
    }

    Ok(SemanticProgram {
        active_scenario: scenario.name.clone(),
        sets: resolved_sets,
        set_registry,
        set_aliases: collect_set_aliases(program, Some(model)),
        set_params,
        parameters: ResolvedParameters {
            series: series_parameters.into_iter().collect(),
            indexed: indexed_parameters.into_iter().collect(),
            asset: asset_parameters.into_iter().collect(),
        },
        variable_families: model
            .controls
            .iter()
            .map(|control| FamilySignature::from_index_decls(&control.name, &control.indices))
            .collect(),
        variable_overrides: collect_control_overrides(
            model
                .controls
                .iter()
                .map(|c| (c.name.as_str(), c.kind, c.lower.as_ref(), c.upper.as_ref())),
        ),
        chronology,
        active_constraints,
        active_expressions,
        active_objective,
        active_reports,
        active_dual_reports,
    })
}

fn resolve_scenario<'a>(
    program: &'a SourceProgram,
    entrypoint: &Path,
) -> Result<&'a ScenarioDecl, SemanticError> {
    match program.scenarios.len() {
        0 => Err(SemanticError::MissingScenario {
            path: entrypoint.to_path_buf(),
        }),
        1 => Ok(&program.scenarios[0]),
        count => Err(SemanticError::ScenarioCount {
            count,
            path: entrypoint.to_path_buf(),
        }),
    }
}

fn validate_scenario_data_bindings_match_model_params(
    scenario: &ScenarioDecl,
    model: &ModelDecl,
    entrypoint: &Path,
) -> Result<(), SemanticError> {
    let parameter_names = model
        .parameters
        .iter()
        .map(|parameter| parameter.name.as_str())
        .collect::<BTreeSet<_>>();
    for binding in &scenario.data {
        if !parameter_names.contains(binding.name.as_str()) {
            return Err(SemanticError::UnknownScenarioDataBinding {
                scenario: scenario.name.clone(),
                binding: binding.name.clone(),
                model: model.name.clone(),
                path: entrypoint.to_path_buf(),
            });
        }
    }
    Ok(())
}

fn classify_parameter_indices(
    name: &str,
    indices: &[String],
    series_parameters: &mut BTreeSet<String>,
    indexed_parameters: &mut BTreeSet<String>,
    asset_parameters: &mut BTreeSet<String>,
) {
    let normalized_indices = indices
        .iter()
        .map(|index| index.trim().replace(' ', ""))
        .collect::<Vec<_>>();

    if normalized_indices.is_empty() {
        let _ = asset_parameters.insert(render_signature(name, &normalized_indices));
        return;
    }

    let signature = render_signature(name, &normalized_indices);

    if normalized_indices.len() == 1 {
        let index = normalized_indices[0].to_ascii_lowercase();
        if index == "t" || index == "time" {
            let _ = series_parameters.insert(signature);
        } else {
            let _ = asset_parameters.insert(signature);
        }
    } else {
        let _ = indexed_parameters.insert(signature);
    }
}

fn render_signature(name: &str, indices: &[String]) -> String {
    if indices.is_empty() {
        return name.to_string();
    }

    let normalized = indices.join(",");
    format!("{name}[{normalized}]")
}

fn detect_model_chronology(
    asset_parameters: &BTreeSet<String>,
    scenario: &ScenarioDecl,
) -> ResolvedChronology {
    let has_param = |name: &str| -> Option<String> {
        if asset_parameters
            .iter()
            .any(|p| p == name || p.starts_with(&format!("{name}[")))
        {
            return Some(name.to_string());
        }
        if scenario.data.iter().any(|d| d.name == name) {
            return Some(name.to_string());
        }
        None
    };

    ResolvedChronology {
        initial_boundary: has_param("initial_soc_mwh"),
        terminal_boundary: has_param("terminal_soc_mwh"),
        initial_commitment_boundary: has_param("initial_commitment"),
    }
}

pub(crate) fn collect_control_overrides<'a>(
    controls: impl Iterator<
        Item = (
            &'a str,
            Option<VariableKindDecl>,
            Option<&'a BoundExpr>,
            Option<&'a BoundExpr>,
        ),
    >,
) -> BTreeMap<String, VariableDeclOverrides> {
    let mut overrides = BTreeMap::new();
    for (name, kind, lower, upper) in controls {
        if kind.is_some() || lower.is_some() || upper.is_some() {
            overrides.insert(
                name.to_string(),
                VariableDeclOverrides {
                    kind,
                    lower: lower.cloned(),
                    upper: upper.cloned(),
                },
            );
        }
    }
    overrides
}
