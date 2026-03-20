// thiserror's Display derive triggers unused_assignments in edition 2024
// because derive-generated code no longer inherits item-level #[allow].
#![allow(unused_assignments)]

use crate::algebra::{
    ConstraintBody, Expr, collect_named_expression_dependencies, constraint_mentions_previous_time,
};
use crate::source::{
    BoundExpr, ConstraintDecl, DataBindingDecl, GenerationBinding, InstancesDecl, ScenarioDecl,
    SourceProgram, VariableKindDecl,
};
use csv::StringRecord;
use miette::Diagnostic;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::{debug, info};

struct DataBindingContext<'a> {
    expected_steps: usize,
    asset_count: usize,
    series: &'a mut BTreeSet<String>,
    indexed: &'a mut BTreeSet<String>,
    asset: &'a mut BTreeSet<String>,
}

/// A structured variable family declaration: a target name and its index
/// dimensions. For example, `dispatch[a,t]` is represented as
/// `FamilySignature { target: "dispatch", indices: ["a", "t"] }`.
///
/// When an index is explicitly bound to a set domain via `in="set_name"`,
/// the mapping is stored in `index_domains`. This lets the lowering phase
/// iterate over user-defined sets instead of relying solely on the
/// hardcoded "a" = assets, "t" = time convention.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FamilySignature {
    pub target: String,
    pub indices: Vec<String>,
    pub index_domains: BTreeMap<String, String>,
}

impl FamilySignature {
    pub fn new(
        target: impl Into<String>,
        indices: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            target: target.into(),
            indices: indices.into_iter().map(Into::into).collect(),
            index_domains: BTreeMap::new(),
        }
    }

    /// Build a signature from `IndexDecl` entries, preserving explicit
    /// domain bindings from `in="..."` properties.
    pub fn from_index_decls(target: impl Into<String>, decls: &[crate::source::IndexDecl]) -> Self {
        let mut index_domains = BTreeMap::new();
        let indices = decls
            .iter()
            .map(|idx| {
                if let Some(domain) = &idx.domain {
                    index_domains.insert(idx.name.clone(), domain.clone());
                }
                idx.name.clone()
            })
            .collect();
        Self {
            target: target.into(),
            indices,
            index_domains,
        }
    }

    /// Render the canonical string form, e.g. `"dispatch[a,t]"`.
    pub fn render(&self) -> String {
        if self.indices.is_empty() {
            return self.target.clone();
        }
        format!("{}[{}]", self.target, self.indices.join(","))
    }
}

/// Declared overrides for a variable family's domain: kind, lower bound,
/// and upper bound. When present these take precedence over the hardcoded
/// name-based defaults in `variable_domain_policy`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VariableDeclOverrides {
    pub kind: Option<VariableKindDecl>,
    pub lower: Option<BoundExpr>,
    pub upper: Option<BoundExpr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticProgram {
    pub active_scenario: String,
    pub sets: ResolvedSets,
    pub set_registry: BTreeMap<String, ResolvedSet>,
    pub set_params: BTreeMap<String, BTreeMap<String, f64>>,
    pub parameters: ResolvedParameters,
    pub variable_families: Vec<FamilySignature>,
    pub variable_overrides: BTreeMap<String, VariableDeclOverrides>,
    pub chronology: ResolvedChronology,
    pub active_constraints: Vec<ResolvedConstraint>,
    pub active_expressions: Vec<ResolvedExpression>,
    pub active_objective: ResolvedObjective,
    pub active_reports: Vec<ResolvedReport>,
    pub lowering_rules: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSets {
    pub assets: Vec<String>,
    pub candidate_assets: Vec<String>,
    pub time: ResolvedTimeSet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTimeSet {
    pub steps: usize,
    pub resolution: String,
}

/// A named set of string values for use in reductions and index domains.
/// Built-in sets (assets, candidate_assets, time) are populated automatically;
/// user-declared sets in scenarios extend the registry alongside them.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedSet {
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedParameters {
    pub series: Vec<String>,
    pub indexed: Vec<String>,
    pub asset: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedChronology {
    pub initial_boundary: Option<String>,
    pub terminal_boundary: Option<String>,
    pub initial_commitment_boundary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedConstraint {
    pub name: String,
    pub source_kind: String,
    pub source_name: String,
    pub expression_text: String,
    pub expression: ConstraintBody,
    pub generation_bindings: Vec<GenerationBinding>,
    pub generation_filter_text: Option<String>,
    pub generation_filter: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedExpression {
    pub name: String,
    pub formula_text: String,
    pub formula: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedObjective {
    pub name: String,
    pub sense: String,
    pub expression_text: String,
    pub expression: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedReport {
    pub name: String,
    pub formula_text: String,
    pub formula: Expr,
}

#[derive(Debug, Error, Diagnostic)]
pub enum SemanticError {
    #[error("no scenario is available for semantic validation in {path}")]
    #[diagnostic(
        code(arco::semantic::missing_scenario),
        help("add a `scenario` declaration")
    )]
    MissingScenario { path: PathBuf },
    #[error("missing declaration `{kind}` named `{name}` in {path}")]
    #[diagnostic(
        code(arco::semantic::missing_declaration),
        help("add the missing declaration or update the reference to an existing one")
    )]
    MissingDeclaration {
        kind: &'static str,
        name: String,
        path: PathBuf,
    },
    #[error("duplicate scenario data binding `{name}` in {path}")]
    #[diagnostic(
        code(arco::semantic::duplicate_data_binding),
        help("rename or remove the duplicate data binding")
    )]
    DuplicateDataBinding { name: String, path: PathBuf },
    #[error("duplicate asset identifier `{asset}` in {path}")]
    #[diagnostic(
        code(arco::semantic::duplicate_asset),
        help("ensure each asset name is unique within the resolved scenario")
    )]
    DuplicateAsset { asset: String, path: PathBuf },
    #[error("missing required column `{column}` in {path}")]
    #[diagnostic(
        code(arco::semantic::missing_column),
        help("add the missing CSV column or update the instances mapping")
    )]
    MissingColumn { column: String, path: PathBuf },
    #[error("missing required value in column `{column}` at row {row} in {path}")]
    #[diagnostic(
        code(arco::semantic::missing_cell),
        help("fill in the missing value in the input table")
    )]
    MissingCell {
        column: String,
        row: usize,
        path: PathBuf,
    },
    #[error("time series `{name}` in {path} has {actual_steps} steps, expected {expected_steps}")]
    #[diagnostic(
        code(arco::semantic::time_series_length),
        help("make the series length match the scenario horizon")
    )]
    TimeSeriesLength {
        name: String,
        path: PathBuf,
        expected_steps: usize,
        actual_steps: usize,
    },
    #[error(
        "asset-scoped data `{name}` in {path} has {actual_assets} rows, expected {expected_assets}"
    )]
    #[diagnostic(
        code(arco::semantic::asset_data_length),
        help("make the asset-scoped data row count match the resolved asset set")
    )]
    AssetDataLength {
        name: String,
        path: PathBuf,
        expected_assets: usize,
        actual_assets: usize,
    },
    #[error("indexed data `{name}` in {path} has {actual_rows} rows, expected {expected_rows}")]
    #[diagnostic(
        code(arco::semantic::indexed_data_length),
        help("make the indexed data row count match assets x time")
    )]
    IndexedDataLength {
        name: String,
        path: PathBuf,
        expected_rows: usize,
        actual_rows: usize,
    },
    #[error("chronology-dependent expressions require an explicit initial boundary in {path}")]
    #[diagnostic(
        code(arco::semantic::missing_initial_boundary),
        help(
            "provide an initial state or commitment boundary for chronology-dependent constraints"
        )
    )]
    MissingInitialBoundary { path: PathBuf },
    #[error("failed to read csv {path}: {source}")]
    #[diagnostic(
        code(arco::semantic::csv),
        help("verify the CSV path exists and is readable")
    )]
    Csv {
        path: PathBuf,
        #[source]
        source: csv::Error,
    },
}

pub fn validate_program(
    program: &SourceProgram,
    entrypoint: &Path,
) -> Result<SemanticProgram, SemanticError> {
    info!("validating program");
    if !program.models.is_empty() {
        return validate_canonical_model_program(program, entrypoint);
    }

    let scenario = resolve_scenario(program, entrypoint)?;

    let mut seen_data = BTreeSet::new();
    for binding in &scenario.data {
        if !seen_data.insert(binding.name.clone()) {
            return Err(SemanticError::DuplicateDataBinding {
                name: binding.name.clone(),
                path: entrypoint.to_path_buf(),
            });
        }
    }

    let entry_dir = entrypoint
        .parent()
        .ok_or_else(|| SemanticError::MissingScenario {
            path: entrypoint.to_path_buf(),
        })?;

    let mut assets = BTreeSet::new();
    let mut candidate_assets = BTreeSet::new();
    let mut asset_parameters = BTreeSet::new();
    let mut technology_names = BTreeSet::new();
    let mut operation_names = BTreeSet::new();
    // Maps the set key (technology.set_name or technology.name) -> asset names.
    let mut technology_assets: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for asset_name in &scenario.assets {
        let asset = program
            .asset(asset_name)
            .ok_or_else(|| SemanticError::MissingDeclaration {
                kind: "asset",
                name: asset_name.clone(),
                path: entrypoint.to_path_buf(),
            })?;
        if !assets.insert(asset.name.clone()) {
            return Err(SemanticError::DuplicateAsset {
                asset: asset.name.clone(),
                path: entrypoint.to_path_buf(),
            });
        }
        technology_names.insert(asset.technology.clone());
        let set_key = technology_set_key(program, &asset.technology);
        technology_assets
            .entry(set_key)
            .or_default()
            .push(asset.name.clone());
        if let Some(operation) = &asset.operation {
            operation_names.insert(operation.clone());
        }
        for parameter_name in asset.parameters.keys() {
            asset_parameters.insert(format!("{parameter_name}[a]"));
        }
    }

    for instances_name in &scenario.instances {
        let instances =
            program
                .instances(instances_name)
                .ok_or_else(|| SemanticError::MissingDeclaration {
                    kind: "instances",
                    name: instances_name.clone(),
                    path: entrypoint.to_path_buf(),
                })?;
        technology_names.insert(instances.technology.clone());
        if let Some(operation) = &instances.operation {
            operation_names.insert(operation.clone());
        }
        let csv_path = entry_dir.join(&instances.source);
        let rows = read_csv_rows(&csv_path)?;
        let name_column = instances_name_column(instances, &csv_path)?;

        for row in &rows {
            let asset_name = required_cell(row, &name_column, 0, &csv_path)?;
            if !assets.insert(asset_name.clone()) {
                return Err(SemanticError::DuplicateAsset {
                    asset: asset_name,
                    path: csv_path.clone(),
                });
            }
            let set_key = technology_set_key(program, &instances.technology);
            technology_assets
                .entry(set_key)
                .or_default()
                .push(asset_name);
        }

        if instances.columns.is_empty() {
            // Auto-map: every CSV column except the identity column becomes a
            // parameter with the same name.
            if let Some(first_row) = rows.first() {
                for col_name in first_row.keys() {
                    if col_name != &name_column {
                        asset_parameters.insert(format!("{col_name}[a]"));
                    }
                }
            }
        } else {
            for target in instances
                .columns
                .iter()
                .filter(|column| column.target != "name")
                .map(|column| column.target.as_str())
            {
                asset_parameters.insert(format!("{target}[a]"));
            }
        }

        if instances.name.starts_with("Candidate") {
            for row in &rows {
                candidate_assets.insert(required_cell(row, &name_column, 0, &csv_path)?);
            }
        }
    }

    let asset_count = assets.len();
    let mut series_parameters = BTreeSet::new();
    let mut indexed_parameters = BTreeSet::new();

    for binding in &scenario.data {
        let csv_path = entry_dir.join(&binding.source);
        let rows = read_csv_rows(&csv_path)?;
        let mut context = DataBindingContext {
            expected_steps: scenario.horizon.steps,
            asset_count,
            series: &mut series_parameters,
            indexed: &mut indexed_parameters,
            asset: &mut asset_parameters,
        };
        classify_data_binding(binding, &csv_path, &rows, &mut context)?;
    }

    let mut variable_families =
        technology_variable_families(program, &technology_names, entrypoint)?;

    // Derive extra variable families from the problem structure rather than
    // relying on a mode string. If candidate assets exist, the problem needs
    // a build decision variable. If unserved_energy appears in constraints,
    // objectives, or expressions, it needs its own family.
    if !candidate_assets.is_empty() {
        variable_families.insert(FamilySignature::new("build", ["a"]));
    }

    let active_objective_name =
        scenario
            .objective
            .clone()
            .ok_or_else(|| SemanticError::MissingDeclaration {
                kind: "objective",
                name: "active objective".to_string(),
                path: entrypoint.to_path_buf(),
            })?;
    let active_objective_decl = program.objective(&active_objective_name).ok_or_else(|| {
        SemanticError::MissingDeclaration {
            kind: "objective",
            name: active_objective_name.clone(),
            path: entrypoint.to_path_buf(),
        }
    })?;

    // Auto-wire rules: when the scenario doesn't list any rules explicitly,
    // include every declared rule. When it does list rules, use that subset.
    let active_rule_names: Vec<String> = if scenario.rules.is_empty() {
        program.rules.iter().map(|r| r.name.clone()).collect()
    } else {
        scenario.rules.clone()
    };

    // Scan all algebra sources for implicit variable families not covered
    // by declared technologies (e.g. unserved_energy).
    let implicit_families = collect_implicit_variable_families(
        program,
        &operation_names,
        &active_rule_names,
        active_objective_decl,
        &scenario.reports,
        &variable_families,
    );
    variable_families.extend(implicit_families);

    let chronology = ResolvedChronology {
        initial_boundary: contains_parameter(&asset_parameters, "initial_soc_mwh[a]"),
        terminal_boundary: contains_parameter(&asset_parameters, "terminal_soc_mwh[a]"),
        initial_commitment_boundary: contains_parameter(&asset_parameters, "initial_commitment[a]"),
    };

    let active_constraints = resolve_direct_wiring_constraints(
        program,
        &operation_names,
        &active_rule_names,
        entrypoint,
    )?;
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
        name: active_objective_decl.name.clone(),
        sense: active_objective_decl.sense.clone(),
        expression_text: active_objective_decl.expression.clone(),
        expression: active_objective_decl.parsed_expression.clone(),
    };
    let active_reports = resolve_scenario_reports(program, scenario, entrypoint)?;
    let active_expressions =
        resolve_active_expressions(program, &active_objective, &active_reports, entrypoint)?;

    let lowering_rules = if candidate_assets.is_empty() {
        Vec::new()
    } else {
        vec![
            "build[a] is a decision family over candidate_assets".to_string(),
            "build[a] is fixed to zero for non-candidate assets".to_string(),
        ]
    };

    let resolved_sets = ResolvedSets {
        assets: assets.into_iter().collect(),
        candidate_assets: candidate_assets.into_iter().collect(),
        time: ResolvedTimeSet {
            steps: scenario.horizon.steps,
            resolution: scenario.horizon.resolution.clone(),
        },
    };
    let mut set_registry = build_set_registry(&resolved_sets, &scenario.custom_sets);
    for (technology_name, asset_names) in technology_assets {
        set_registry
            .entry(technology_name)
            .or_insert_with(|| ResolvedSet {
                values: asset_names,
            });
    }

    let mut set_params: BTreeMap<String, BTreeMap<String, f64>> = BTreeMap::new();
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

    Ok(SemanticProgram {
        active_scenario: scenario.name.clone(),
        sets: resolved_sets,
        set_registry,
        set_params,
        parameters: ResolvedParameters {
            series: series_parameters.into_iter().collect(),
            indexed: indexed_parameters.into_iter().collect(),
            asset: asset_parameters.into_iter().collect(),
        },
        variable_families: variable_families.into_iter().collect(),
        variable_overrides: collect_technology_overrides(program, &technology_names, entrypoint)?,
        chronology,
        active_constraints,
        active_expressions,
        active_objective,
        active_reports,
        lowering_rules,
    })
    .inspect(|semantic_program| {
        debug!(
            "resolved {} assets, {} constraints, {} variable families",
            semantic_program.sets.assets.len(),
            semantic_program.active_constraints.len(),
            semantic_program.variable_families.len()
        );
    })
}

fn validate_canonical_model_program(
    program: &SourceProgram,
    entrypoint: &Path,
) -> Result<SemanticProgram, SemanticError> {
    let scenario = resolve_scenario(program, entrypoint)?;
    let model_name =
        scenario
            .model_use
            .clone()
            .ok_or_else(|| SemanticError::MissingDeclaration {
                kind: "model",
                name: "active model".to_string(),
                path: entrypoint.to_path_buf(),
            })?;
    let model = program
        .model(&model_name)
        .ok_or_else(|| SemanticError::MissingDeclaration {
            kind: "model",
            name: model_name,
            path: entrypoint.to_path_buf(),
        })?;

    let mut asset_names = scenario.assets.iter().cloned().collect::<BTreeSet<_>>();
    if asset_names.is_empty() {
        for set_decl in &model.sets {
            if set_decl.name == "assets" {
                asset_names.insert("assets".to_string());
            }
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
    let active_reports = resolve_scenario_reports(program, scenario, entrypoint)?;
    let active_expressions =
        resolve_active_expressions(program, &active_objective, &active_reports, entrypoint)?;

    let resolved_sets = ResolvedSets {
        assets: asset_names.into_iter().collect(),
        candidate_assets: Vec::new(),
        time: ResolvedTimeSet {
            steps: scenario.horizon.steps,
            resolution: scenario.horizon.resolution.clone(),
        },
    };
    let mut set_registry = build_set_registry(&resolved_sets, &scenario.custom_sets);

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
        lowering_rules: Vec::new(),
    })
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
        let index = &normalized_indices[0];
        if index.contains('t') || index.contains("time") {
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

fn resolve_scenario<'a>(
    program: &'a SourceProgram,
    entrypoint: &Path,
) -> Result<&'a ScenarioDecl, SemanticError> {
    program
        .first_scenario()
        .ok_or_else(|| SemanticError::MissingScenario {
            path: entrypoint.to_path_buf(),
        })
}

fn resolve_direct_wiring_constraints(
    program: &SourceProgram,
    operation_names: &BTreeSet<String>,
    rule_names: &[String],
    entrypoint: &Path,
) -> Result<Vec<ResolvedConstraint>, SemanticError> {
    let mut constraints = Vec::new();

    for operation_name in operation_names {
        let operation =
            program
                .operation(operation_name)
                .ok_or_else(|| SemanticError::MissingDeclaration {
                    kind: "operation",
                    name: operation_name.clone(),
                    path: entrypoint.to_path_buf(),
                })?;
        append_constraints(
            &mut constraints,
            "operation",
            &operation.name,
            &operation.constraints,
        );
    }

    for rule_name in rule_names {
        let rule = program
            .rule(rule_name)
            .ok_or_else(|| SemanticError::MissingDeclaration {
                kind: "rule",
                name: rule_name.clone(),
                path: entrypoint.to_path_buf(),
            })?;
        append_constraints(&mut constraints, "rule", &rule.name, &rule.constraints);
    }

    constraints.sort_by_key(|constraint| {
        (
            constraint.source_kind.clone(),
            constraint.source_name.clone(),
            constraint.name.clone(),
        )
    });
    Ok(constraints)
}

fn append_constraints(
    target: &mut Vec<ResolvedConstraint>,
    source_kind: &str,
    source_name: &str,
    constraints: &[ConstraintDecl],
) {
    for constraint in constraints {
        target.push(ResolvedConstraint {
            name: constraint.name.clone(),
            source_kind: source_kind.to_string(),
            source_name: source_name.to_string(),
            expression_text: constraint.expression.clone(),
            expression: constraint.parsed_expression.clone(),
            generation_bindings: constraint.generation_bindings.clone(),
            generation_filter_text: constraint.generation_filter.clone(),
            generation_filter: constraint.parsed_generation_filter.clone(),
        });
    }
}

fn resolve_scenario_reports(
    program: &SourceProgram,
    scenario: &ScenarioDecl,
    entrypoint: &Path,
) -> Result<Vec<ResolvedReport>, SemanticError> {
    let mut reports = Vec::new();
    for report_name in &scenario.reports {
        let expression =
            program
                .expression(report_name)
                .ok_or_else(|| SemanticError::MissingDeclaration {
                    kind: "expression",
                    name: report_name.clone(),
                    path: entrypoint.to_path_buf(),
                })?;
        reports.push(ResolvedReport {
            name: expression.name.clone(),
            formula_text: expression.formula.clone(),
            formula: expression.parsed_formula.clone(),
        });
    }
    Ok(reports)
}

fn resolve_active_expressions(
    program: &SourceProgram,
    objective: &ResolvedObjective,
    reports: &[ResolvedReport],
    entrypoint: &Path,
) -> Result<Vec<ResolvedExpression>, SemanticError> {
    let mut names = BTreeSet::new();
    let mut expressions = Vec::new();

    resolve_expression_dependencies(program, &objective.expression, &mut names, entrypoint)?;
    for report in reports {
        names.insert(report.name.clone());
        resolve_expression_dependencies(program, &report.formula, &mut names, entrypoint)?;
    }

    for name in names {
        let expression =
            program
                .expression(&name)
                .ok_or_else(|| SemanticError::MissingDeclaration {
                    kind: "expression",
                    name: name.clone(),
                    path: entrypoint.to_path_buf(),
                })?;
        expressions.push(ResolvedExpression {
            name: expression.name.clone(),
            formula_text: expression.formula.clone(),
            formula: expression.parsed_formula.clone(),
        });
    }
    expressions.sort_by_key(|expression| expression.name.clone());
    Ok(expressions)
}

fn resolve_expression_dependencies(
    program: &SourceProgram,
    expression: &Expr,
    names: &mut BTreeSet<String>,
    entrypoint: &Path,
) -> Result<(), SemanticError> {
    for dependency in collect_named_expression_dependencies(expression) {
        let declaration =
            program
                .expression(&dependency)
                .ok_or_else(|| SemanticError::MissingDeclaration {
                    kind: "expression",
                    name: dependency.clone(),
                    path: entrypoint.to_path_buf(),
                })?;
        if names.insert(dependency) {
            resolve_expression_dependencies(
                program,
                &declaration.parsed_formula,
                names,
                entrypoint,
            )?;
        }
    }
    Ok(())
}

fn contains_parameter(parameters: &BTreeSet<String>, name: &str) -> Option<String> {
    parameters.contains(name).then(|| name.to_string())
}

/// Detect chronology boundaries for the model path. Unlike the high-level
/// path which checks for exact signatures like `initial_soc_mwh[a]`, the
/// model path uses custom index letters. We check if the parameter name
/// prefix matches regardless of the index letter used. Data bindings
/// from the scenario are also checked as a fallback since model-path
/// parameters commonly arrive via CSV data rather than inline `param`
/// declarations.
fn detect_model_chronology(
    asset_parameters: &BTreeSet<String>,
    scenario: &ScenarioDecl,
) -> ResolvedChronology {
    let has_param = |name: &str| -> Option<String> {
        // Check model param declarations (any index letter)
        if asset_parameters
            .iter()
            .any(|p| p == name || p.starts_with(&format!("{name}[")))
        {
            return Some(name.to_string());
        }
        // Check scenario data bindings (CSV-provided parameters)
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

struct SetCsvData {
    members: Vec<String>,
    params: BTreeMap<String, BTreeMap<String, f64>>,
}

/// Load a CSV file that defines a set. The first column must be `name` and
/// contains the member identifiers. Remaining columns are numeric parameters
/// keyed by member name.
fn load_set_csv(path: &Path) -> Result<SetCsvData, SemanticError> {
    let mut reader = csv::Reader::from_path(path).map_err(|source| SemanticError::Csv {
        path: path.to_path_buf(),
        source,
    })?;
    let headers = reader
        .headers()
        .map_err(|source| SemanticError::Csv {
            path: path.to_path_buf(),
            source,
        })?
        .clone();

    let name_index =
        headers
            .iter()
            .position(|h| h == "name")
            .ok_or_else(|| SemanticError::MissingColumn {
                column: "name".to_string(),
                path: path.to_path_buf(),
            })?;

    let param_columns: Vec<(usize, String)> = headers
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != name_index)
        .map(|(i, h)| (i, h.to_string()))
        .collect();

    let mut members = Vec::new();
    let mut member_params = BTreeMap::new();

    for (row_index, record) in reader.records().enumerate() {
        let record = record.map_err(|source| SemanticError::Csv {
            path: path.to_path_buf(),
            source,
        })?;
        let member_name = record
            .get(name_index)
            .filter(|v| !v.is_empty())
            .ok_or_else(|| SemanticError::MissingCell {
                column: "name".to_string(),
                row: row_index + 1,
                path: path.to_path_buf(),
            })?
            .to_string();

        let mut params = BTreeMap::new();
        for (col_index, col_name) in &param_columns {
            let raw = record
                .get(*col_index)
                .filter(|v| !v.is_empty())
                .ok_or_else(|| SemanticError::MissingCell {
                    column: col_name.clone(),
                    row: row_index + 1,
                    path: path.to_path_buf(),
                })?;
            let value = raw.parse::<f64>().map_err(|_| SemanticError::MissingCell {
                column: col_name.clone(),
                row: row_index + 1,
                path: path.to_path_buf(),
            })?;
            params.insert(col_name.clone(), value);
        }
        members.push(member_name.clone());
        member_params.insert(member_name, params);
    }

    Ok(SetCsvData {
        members,
        params: member_params,
    })
}

fn read_csv_rows(path: &Path) -> Result<Vec<BTreeMap<String, String>>, SemanticError> {
    let mut reader = csv::Reader::from_path(path).map_err(|source| SemanticError::Csv {
        path: path.to_path_buf(),
        source,
    })?;
    let headers = reader
        .headers()
        .map_err(|source| SemanticError::Csv {
            path: path.to_path_buf(),
            source,
        })?
        .clone();

    reader
        .records()
        .enumerate()
        .map(|(index, record)| {
            let record = record.map_err(|source| SemanticError::Csv {
                path: path.to_path_buf(),
                source,
            })?;
            record_to_map(path, &headers, index + 1, record)
        })
        .collect()
}

fn record_to_map(
    path: &Path,
    headers: &StringRecord,
    row_index: usize,
    record: StringRecord,
) -> Result<BTreeMap<String, String>, SemanticError> {
    let mut row = BTreeMap::new();
    for (header, value) in headers.iter().zip(record.iter()) {
        if value.is_empty() {
            return Err(SemanticError::MissingCell {
                column: header.to_string(),
                row: row_index,
                path: path.to_path_buf(),
            });
        }
        row.insert(header.to_string(), value.to_string());
    }
    Ok(row)
}

fn instances_name_column(instances: &InstancesDecl, path: &Path) -> Result<String, SemanticError> {
    if instances.columns.is_empty() {
        // Auto-map: convention is "asset_name" as the identity column.
        return Ok("asset_name".to_string());
    }
    instances
        .columns
        .iter()
        .find(|column| column.target == "name")
        .map(|column| column.source.clone())
        .ok_or_else(|| SemanticError::MissingColumn {
            column: "name".to_string(),
            path: path.to_path_buf(),
        })
}

fn required_cell(
    row: &BTreeMap<String, String>,
    column: &str,
    row_index: usize,
    path: &Path,
) -> Result<String, SemanticError> {
    row.get(column)
        .cloned()
        .ok_or_else(|| SemanticError::MissingCell {
            column: column.to_string(),
            row: row_index,
            path: path.to_path_buf(),
        })
}

fn classify_data_binding(
    binding: &DataBindingDecl,
    path: &Path,
    rows: &[BTreeMap<String, String>],
    context: &mut DataBindingContext<'_>,
) -> Result<(), SemanticError> {
    let Some(first_row) = rows.first() else {
        return Err(SemanticError::MissingCell {
            column: binding.name.clone(),
            row: 0,
            path: path.to_path_buf(),
        });
    };
    let headers = first_row.keys().cloned().collect::<BTreeSet<_>>();

    if headers.contains("asset_name") && headers.contains("t") {
        let actual_steps = unique_count(rows, "t");
        if actual_steps != context.expected_steps {
            return Err(SemanticError::TimeSeriesLength {
                name: binding.name.clone(),
                path: path.to_path_buf(),
                expected_steps: context.expected_steps,
                actual_steps,
            });
        }
        let expected_rows = context.expected_steps.saturating_mul(context.asset_count);
        if rows.len() != expected_rows {
            return Err(SemanticError::IndexedDataLength {
                name: binding.name.clone(),
                path: path.to_path_buf(),
                expected_rows,
                actual_rows: rows.len(),
            });
        }
        asset_name_rows_unique(rows, path)?;
        context.indexed.insert(format!("{}[a,t]", binding.name));
        return Ok(());
    }

    if headers.contains("asset_name") {
        if rows.len() != context.asset_count {
            return Err(SemanticError::AssetDataLength {
                name: binding.name.clone(),
                path: path.to_path_buf(),
                expected_assets: context.asset_count,
                actual_assets: rows.len(),
            });
        }
        asset_name_rows_unique(rows, path)?;
        context.asset.insert(format!("{}[a]", binding.name));
        return Ok(());
    }

    if headers.contains("t") {
        let actual_steps = unique_count(rows, "t");
        if actual_steps != context.expected_steps {
            return Err(SemanticError::TimeSeriesLength {
                name: binding.name.clone(),
                path: path.to_path_buf(),
                expected_steps: context.expected_steps,
                actual_steps,
            });
        }
        context.series.insert(format!("{}[t]", binding.name));
        return Ok(());
    }

    Err(SemanticError::MissingColumn {
        column: "asset_name or t".to_string(),
        path: path.to_path_buf(),
    })
}

fn asset_name_rows_unique(
    rows: &[BTreeMap<String, String>],
    path: &Path,
) -> Result<(), SemanticError> {
    let mut seen = BTreeSet::new();
    for row in rows {
        if let Some(asset_name) = row.get("asset_name")
            && !seen.insert(asset_name.clone())
            && !row.contains_key("t")
        {
            return Err(SemanticError::DuplicateAsset {
                asset: asset_name.clone(),
                path: path.to_path_buf(),
            });
        }
    }
    Ok(())
}

fn unique_count(rows: &[BTreeMap<String, String>], key: &str) -> usize {
    rows.iter()
        .filter_map(|row| row.get(key).cloned())
        .collect::<BTreeSet<_>>()
        .len()
}

/// Scan constraint, objective, and expression ASTs for indexed references that
/// look like variable families but are not covered by the declared technology
/// controls/states. This replaces the old mode-based variable family injection
/// (e.g. capacity_expansion added build[a] and unserved_energy[t]).
///
/// We use a conservative allowlist: only well-known implicit variable families
/// that the old mode system used to inject are recognized. Everything else is
/// assumed to be a parameter.
fn collect_implicit_variable_families(
    program: &SourceProgram,
    operation_names: &BTreeSet<String>,
    rule_names: &[String],
    objective: &crate::source::ObjectiveDecl,
    report_names: &[String],
    known_families: &BTreeSet<FamilySignature>,
) -> BTreeSet<FamilySignature> {
    let known_targets: BTreeSet<String> = known_families
        .iter()
        .map(|family| family.target.clone())
        .collect();

    let mut all_targets = BTreeSet::new();

    // Collect from operation constraints
    for op_name in operation_names {
        if let Some(operation) = program.operation(op_name) {
            for constraint in &operation.constraints {
                collect_indexed_targets_from_constraint_body(
                    &constraint.parsed_expression,
                    &mut all_targets,
                );
            }
        }
    }

    // Collect from rule constraints
    for rule_name in rule_names {
        if let Some(rule) = program.rule(rule_name) {
            for constraint in &rule.constraints {
                collect_indexed_targets_from_constraint_body(
                    &constraint.parsed_expression,
                    &mut all_targets,
                );
            }
        }
    }

    // Collect from objective and expression ASTs (transitively)
    let mut visited = BTreeSet::new();
    collect_all_targets_from_expr(
        &objective.parsed_expression,
        program,
        &mut all_targets,
        &mut visited,
    );
    for report_name in report_names {
        if let Some(expression) = program.expression(report_name) {
            collect_all_targets_from_expr(
                &expression.parsed_formula,
                program,
                &mut all_targets,
                &mut visited,
            );
        }
    }

    // Keep only targets that are:
    // 1. Not already in technology-derived families
    // 2. On the allowlist of known implicit variable targets
    all_targets
        .into_iter()
        .filter(|(target, _)| !known_targets.contains(target))
        .filter(|(target, _)| is_implicit_variable_target(target))
        .map(|(target, index_sig)| {
            FamilySignature::new(target, index_sig.split(',').map(str::trim))
        })
        .collect()
}

/// Returns true if a target name is a known implicit variable family that
/// should be auto-discovered from DSL algebra rather than declared in a
/// technology block. This is the conservative allowlist that replaces the
/// old mode-based injection.
fn is_implicit_variable_target(target: &str) -> bool {
    matches!(target, "unserved_energy" | "build")
}

fn collect_indexed_targets_from_constraint_body(
    body: &crate::algebra::ConstraintBody,
    targets: &mut BTreeSet<(String, String)>,
) {
    match body {
        crate::algebra::ConstraintBody::Comparison { left, right, .. } => {
            collect_indexed_targets_from_expr(left, targets);
            collect_indexed_targets_from_expr(right, targets);
        }
        crate::algebra::ConstraintBody::Range {
            lower,
            middle,
            upper,
            ..
        } => {
            collect_indexed_targets_from_expr(lower, targets);
            collect_indexed_targets_from_expr(middle, targets);
            collect_indexed_targets_from_expr(upper, targets);
        }
    }
}

fn collect_indexed_targets_from_expr(expr: &Expr, targets: &mut BTreeSet<(String, String)>) {
    match expr {
        Expr::Indexed { target, indices } => {
            let index_sig = indices
                .iter()
                .map(infer_index_domain)
                .collect::<Vec<_>>()
                .join(",");
            targets.insert((target.clone(), index_sig));
            for index in indices {
                collect_indexed_targets_from_expr(index, targets);
            }
        }
        Expr::Unary { expr, .. } => collect_indexed_targets_from_expr(expr, targets),
        Expr::Binary { left, right, .. } | Expr::Comparison { left, right, .. } => {
            collect_indexed_targets_from_expr(left, targets);
            collect_indexed_targets_from_expr(right, targets);
        }
        Expr::FunctionCall { args, .. } => {
            for arg in args {
                collect_indexed_targets_from_expr(arg, targets);
            }
        }
        Expr::Reduction(reduction) => {
            collect_indexed_targets_from_expr(&reduction.body, targets);
            for filter in &reduction.filters {
                collect_indexed_targets_from_expr(filter, targets);
            }
        }
        Expr::Number(_) | Expr::String(_) | Expr::Boolean(_) | Expr::Identifier(_) => {}
    }
}

fn infer_index_domain(expr: &Expr) -> &str {
    match expr {
        Expr::Identifier(name) => match name.as_str() {
            "a" | "g" | "l" | "n" => "a",
            "t" => "t",
            _ => "a",
        },
        Expr::Binary { left, .. } => infer_index_domain(left),
        _ => "t",
    }
}

fn collect_all_targets_from_expr(
    expr: &Expr,
    program: &SourceProgram,
    targets: &mut BTreeSet<(String, String)>,
    visited: &mut BTreeSet<String>,
) {
    match expr {
        Expr::Identifier(name) => {
            if visited.insert(name.clone())
                && let Some(expression) = program.expression(name)
            {
                collect_indexed_targets_from_expr(&expression.parsed_formula, targets);
                collect_all_targets_from_expr(
                    &expression.parsed_formula,
                    program,
                    targets,
                    visited,
                );
            }
        }
        Expr::Indexed { indices, .. } => {
            for index in indices {
                collect_all_targets_from_expr(index, program, targets, visited);
            }
        }
        Expr::Unary { expr, .. } => {
            collect_all_targets_from_expr(expr, program, targets, visited);
        }
        Expr::Binary { left, right, .. } | Expr::Comparison { left, right, .. } => {
            collect_all_targets_from_expr(left, program, targets, visited);
            collect_all_targets_from_expr(right, program, targets, visited);
        }
        Expr::FunctionCall { args, .. } => {
            for arg in args {
                collect_all_targets_from_expr(arg, program, targets, visited);
            }
        }
        Expr::Reduction(reduction) => {
            collect_all_targets_from_expr(&reduction.body, program, targets, visited);
        }
        Expr::Number(_) | Expr::String(_) | Expr::Boolean(_) => {}
    }
}

/// Returns the set registry key for a technology: the `as` alias if declared,
/// otherwise the technology name itself.
fn technology_set_key(program: &SourceProgram, technology_name: &str) -> String {
    program
        .technology(technology_name)
        .and_then(|t| t.set_name.clone())
        .unwrap_or_else(|| technology_name.to_string())
}

fn technology_variable_families(
    program: &SourceProgram,
    technology_names: &BTreeSet<String>,
    entrypoint: &Path,
) -> Result<BTreeSet<FamilySignature>, SemanticError> {
    let mut families = BTreeSet::new();
    for technology_name in technology_names {
        let technology = program.technology(technology_name).ok_or_else(|| {
            SemanticError::MissingDeclaration {
                kind: "technology",
                name: technology_name.clone(),
                path: entrypoint.to_path_buf(),
            }
        })?;
        for invest in &technology.investments {
            families.insert(FamilySignature::new(&invest.name, ["a"]));
        }
        for control in &technology.controls {
            families.insert(FamilySignature::new(&control.name, ["a", "t"]));
        }
        for state in &technology.states {
            families.insert(FamilySignature::new(state, ["a", "t"]));
        }
    }
    Ok(families)
}

/// Collect `VariableDeclOverrides` from technology `NamedVariableDecl`
/// declarations (invest + control) across all referenced technologies.
fn collect_technology_overrides(
    program: &SourceProgram,
    technology_names: &BTreeSet<String>,
    entrypoint: &Path,
) -> Result<BTreeMap<String, VariableDeclOverrides>, SemanticError> {
    let mut decls = Vec::new();
    for technology_name in technology_names {
        let technology = program.technology(technology_name).ok_or_else(|| {
            SemanticError::MissingDeclaration {
                kind: "technology",
                name: technology_name.clone(),
                path: entrypoint.to_path_buf(),
            }
        })?;
        for decl in technology.investments.iter().chain(&technology.controls) {
            decls.push((
                decl.name.as_str(),
                decl.kind,
                decl.lower.as_ref(),
                decl.upper.as_ref(),
            ));
        }
    }
    Ok(collect_control_overrides(decls.into_iter()))
}

/// Collect `VariableDeclOverrides` from an iterator of
/// `(name, kind, lower, upper)` tuples (used for canonical model controls).
fn collect_control_overrides<'a>(
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

/// Build the set registry from the resolved built-in sets and any
/// user-declared custom sets from the scenario. The registry mirrors
/// assets, candidate_assets, and time as string vectors so that
/// `reduction_domain_values` can fall back to a single lookup for both
/// built-in and user-defined sets.
fn build_set_registry(
    sets: &ResolvedSets,
    custom_sets: &BTreeMap<String, Vec<String>>,
) -> BTreeMap<String, ResolvedSet> {
    let mut registry = BTreeMap::new();
    registry.insert(
        "assets".to_string(),
        ResolvedSet {
            values: sets.assets.clone(),
        },
    );
    registry.insert(
        "candidate_assets".to_string(),
        ResolvedSet {
            values: sets.candidate_assets.clone(),
        },
    );
    registry.insert(
        "time".to_string(),
        ResolvedSet {
            values: (1..=sets.time.steps).map(|step| step.to_string()).collect(),
        },
    );
    for (name, members) in custom_sets {
        registry.insert(
            name.clone(),
            ResolvedSet {
                values: members.clone(),
            },
        );
    }
    registry
}
