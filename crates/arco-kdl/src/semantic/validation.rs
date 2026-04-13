use crate::semantic::error::SemanticError;
use crate::semantic::resolution::{
    resolve_active_model_expressions, resolve_model_scenario_reports,
};
use crate::semantic::sets::{
    collect_set_aliases, extend_set_registry_from_low_level_declarations, literal_to_string,
};
use crate::semantic::types::{
    FamilySignature, ResolvedChronology, ResolvedConstraint, ResolvedExpression, ResolvedObjective,
    ResolvedParameters, ResolvedSets, ResolvedTimeSet, SemanticProgram, VariableDeclOverrides,
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

    validate_scenario_data_bindings_match_known_params(scenario, model, program, entrypoint)?;
    validate_model_parameters_resolved(scenario, model, program, entrypoint)?;

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

    let active_objective = ResolvedObjective {
        name: model.optimize.name.clone(),
        sense: model.optimize.sense,
        expression_text: model.optimize.expression.clone(),
        expression: model.optimize.parsed_expression.clone(),
    };

    let (active_reports, active_dual_reports, active_variable_reports) =
        resolve_model_scenario_reports(model, scenario, &active_constraints, entrypoint)?;
    let mut active_expressions =
        resolve_active_model_expressions(model, &active_objective, &active_reports, entrypoint)?;

    // Inject inline scalar params (e.g. `param fcr_vre 0.072649`) as synthetic
    // named expressions so they are resolvable in algebra and index positions.
    let existing_names: BTreeSet<String> =
        active_expressions.iter().map(|e| e.name.clone()).collect();
    for param in &model.parameters {
        if let Some(ref value) = param.value {
            if existing_names.contains(&param.name) {
                return Err(SemanticError::DuplicateDeclaration {
                    kind: "param/expression".to_string(),
                    name: param.name.clone(),
                    path: entrypoint.to_path_buf(),
                });
            }
            let text = literal_to_string(value);
            active_expressions.push(ResolvedExpression {
                name: param.name.clone(),
                formula_text: text.clone(),
                formula: crate::algebra::Expr::Number(text),
            });
        }
    }

    let mut set_registry = BTreeMap::new();
    if let Some(entry_dir) = entrypoint.parent() {
        extend_set_registry_from_low_level_declarations(program, entry_dir, &mut set_registry)?;
    }

    let time_steps = set_registry
        .get("time")
        .or_else(|| set_registry.get("t"))
        .map_or(0, |set| set.values.len());
    let resolved_sets = ResolvedSets {
        time: ResolvedTimeSet {
            steps: time_steps,
            resolution: String::new(),
        },
    };

    Ok(SemanticProgram {
        active_scenario: scenario.name.clone(),
        sets: resolved_sets,
        set_registry,
        set_aliases: collect_set_aliases(program, Some(model)),
        set_params: BTreeMap::new(),
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
        chronology: ResolvedChronology::default(),
        active_constraints,
        active_expressions,
        active_objective,
        active_reports,
        active_variable_reports,
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

fn validate_scenario_data_bindings_match_known_params(
    scenario: &ScenarioDecl,
    model: &ModelDecl,
    program: &SourceProgram,
    entrypoint: &Path,
) -> Result<(), SemanticError> {
    let model_param_names = model
        .parameters
        .iter()
        .map(|parameter| parameter.name.as_str())
        .collect::<BTreeSet<_>>();
    let data_param_names = program
        .data
        .iter()
        .flat_map(|data_decl| {
            data_decl
                .parameters
                .iter()
                .map(|parameter| parameter.name.as_str())
        })
        .collect::<BTreeSet<_>>();

    for binding in &scenario.data {
        if !model_param_names.contains(binding.name.as_str())
            && !data_param_names.contains(binding.name.as_str())
        {
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

fn validate_model_parameters_resolved(
    scenario: &ScenarioDecl,
    model: &ModelDecl,
    program: &SourceProgram,
    entrypoint: &Path,
) -> Result<(), SemanticError> {
    let scenario_bindings = scenario
        .data
        .iter()
        .map(|binding| binding.name.as_str())
        .collect::<BTreeSet<_>>();
    let data_param_names = program
        .data
        .iter()
        .flat_map(|data_decl| {
            data_decl
                .parameters
                .iter()
                .map(|parameter| parameter.name.as_str())
        })
        .collect::<BTreeSet<_>>();

    for parameter in &model.parameters {
        if parameter.value.is_some() {
            continue;
        }

        if !scenario_bindings.contains(parameter.name.as_str())
            && !data_param_names.contains(parameter.name.as_str())
        {
            return Err(SemanticError::MissingDeclaration {
                kind: "param",
                name: parameter.name.clone(),
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
