use crate::compile::semantic::{FamilySignature, SemanticProgram, VariableDeclOverrides};
use crate::kdl::ObjectiveSense;
use crate::kdl::algebra::{self, ConstraintBody, Expr};
use crate::kdl::source::VariableKindDecl;
use arco_model::SnapshotMemoryEstimate;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use thiserror::Error;

mod modeling;

// ─── Top-level inspect payload ───────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectPayload {
    pub(crate) meta: Meta,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) set: Vec<SetRecord>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) variable: Vec<VariableRecord>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) parameter: Vec<ParameterRecord>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) expression: Vec<ExpressionRecord>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) constraint: Vec<ConstraintRecord>,
    pub(crate) objective: ObjectiveRecord,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) report: Vec<ReportRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) chronology: Option<ChronologyRecord>,
}

// ─── Records ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub(crate) entrypoint: String,
    pub(crate) scenario: String,
    pub(crate) counts: Counts,
    pub(crate) memory: SnapshotMemoryEstimate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Counts {
    pub(crate) set: usize,
    pub(crate) variable: usize,
    pub(crate) variable_instances: usize,
    pub(crate) parameter: usize,
    pub(crate) constraint: usize,
    pub(crate) constraint_instances: usize,
    pub(crate) coefficient_instances: usize,
    pub(crate) expression: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetRecord {
    pub(crate) id: usize,
    pub(crate) name: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) aliases: Vec<String>,
    pub(crate) size: usize,
    pub(crate) dtype: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) subset_of: Vec<SetRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetRef {
    pub(crate) name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetBinding {
    pub(crate) name: String,
    #[serde(rename = "as", skip_serializing_if = "Option::is_none")]
    pub(crate) alias: Option<String>,
    pub(crate) size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableRecord {
    pub(crate) id: usize,
    pub(crate) name: String,
    pub(crate) kind: String,
    pub(crate) instances: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) lower: Option<BoundValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) upper: Option<BoundValue>,
    pub(crate) set: Vec<SetBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BoundValue {
    Numeric(f64),
    Symbolic(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterRecord {
    pub(crate) id: usize,
    pub(crate) name: String,
    pub(crate) kind: String,
    pub(crate) dtype: String,
    pub(crate) set: Vec<SetBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpressionRecord {
    pub(crate) id: usize,
    pub(crate) name: String,
    pub(crate) formula: String,
    pub(crate) uses: Vec<UseRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseRef {
    pub(crate) name: String,
    pub(crate) kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintRecord {
    pub(crate) id: usize,
    pub(crate) name: String,
    pub(crate) relation: String,
    pub(crate) template: String,
    pub(crate) source: SourceRef,
    pub(crate) scope: Vec<SetBinding>,
    pub(crate) lhs: Vec<TermRef>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) rhs: Vec<TermRef>,
    pub(crate) instances: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRef {
    pub(crate) kind: String,
    pub(crate) name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermRef {
    pub(crate) name: String,
    pub(crate) kind: String,
    pub(crate) over: Vec<SetBinding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) reduction: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) reduce_over: Vec<SetRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveRecord {
    pub(crate) name: String,
    pub(crate) sense: ObjectiveSense,
    pub(crate) term: Vec<ObjectiveTermRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveTermRef {
    pub(crate) name: String,
    pub(crate) kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRecord {
    pub(crate) id: usize,
    pub(crate) name: String,
    pub(crate) formula: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronologyRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) initial_boundary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) terminal_boundary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) initial_commitment_boundary: Option<String>,
}

// ─── Builder ─────────────────────────────────────────────────────

pub fn build_inspect_payload(entrypoint: &Path, program: &SemanticProgram) -> InspectPayload {
    let filtered_set_names: &[&str] = &["assets", "candidate_assets"];

    let variable_targets = collect_variable_targets(program);
    let parameter_targets = collect_parameter_targets(program, &variable_targets);

    // Build set records
    let set_records = build_set_records(program, filtered_set_names);

    // Build a lookup from set name → size for bindings
    let set_sizes: BTreeMap<&str, usize> = program
        .set_registry
        .iter()
        .map(|(name, resolved)| (name.as_str(), resolved_set_cardinality(resolved)))
        .collect();

    // Build variable records
    let variable_records = build_variable_records(program, &set_sizes);

    // Build parameter records
    let parameter_records = build_parameter_records(
        program,
        &parameter_targets,
        &variable_targets,
        &set_sizes,
        &program.set_aliases,
    );

    // Build expression records
    let expression_records =
        build_expression_records(program, &variable_targets, &parameter_targets);

    // Build constraint records
    let constraint_records = modeling::build_constraint_records(
        program,
        &variable_targets,
        &parameter_targets,
        &set_sizes,
        &program.set_aliases,
    );
    let coefficient_instances = constraint_records
        .iter()
        .map(estimate_constraint_coefficient_instances)
        .sum();
    // Build objective record
    let objective_record =
        modeling::build_objective_record(program, &variable_targets, &parameter_targets);

    // Build report records
    let report_records = modeling::build_report_records(program);

    // Build chronology
    let chronology = modeling::build_chronology(program);

    let variable_instances = variable_records
        .iter()
        .map(variable_record_instance_count)
        .sum();
    let constraint_instances = constraint_records
        .iter()
        .map(|record| record.instances)
        .sum();
    let memory =
        SnapshotMemoryEstimate::for_sparse_matrix(variable_instances, coefficient_instances);

    InspectPayload {
        meta: Meta {
            entrypoint: entrypoint.display().to_string(),
            scenario: program.active_scenario.clone(),
            counts: Counts {
                set: set_records.len(),
                variable: variable_records.len(),
                variable_instances,
                parameter: parameter_records.len(),
                constraint: constraint_records.len(),
                constraint_instances,
                coefficient_instances,
                expression: expression_records.len(),
            },
            memory,
        },
        set: set_records,
        variable: variable_records,
        parameter: parameter_records,
        expression: expression_records,
        constraint: constraint_records,
        objective: objective_record,
        report: report_records,
        chronology,
    }
}

fn variable_record_instance_count(record: &VariableRecord) -> usize {
    record.instances
}

fn estimate_constraint_coefficient_instances(record: &ConstraintRecord) -> usize {
    let variable_terms = record
        .lhs
        .iter()
        .chain(&record.rhs)
        .filter(|term| term.kind == "variable")
        .map(term_coefficient_fanout)
        .sum::<usize>();
    record.instances.saturating_mul(variable_terms)
}

fn term_coefficient_fanout(term: &TermRef) -> usize {
    let reduction_fanout = term
        .reduce_over
        .iter()
        .filter_map(|set_ref| {
            term.over
                .iter()
                .find(|binding| binding.alias.as_deref() == Some(set_ref.name.as_str()))
                .or_else(|| {
                    term.over
                        .iter()
                        .find(|binding| binding.name == set_ref.name)
                })
        })
        .map(|binding| binding.size)
        .product::<usize>()
        .max(1);
    reduction_fanout
}

// ─── Set builder ─────────────────────────────────────────────────

fn build_set_records(program: &SemanticProgram, filtered_set_names: &[&str]) -> Vec<SetRecord> {
    let mut records = Vec::new();

    for (id, (name, resolved)) in program
        .set_registry
        .iter()
        .filter(|(name, _)| !filtered_set_names.contains(&name.as_str()))
        .enumerate()
    {
        let aliases = find_set_aliases(program, name);
        let dtype = infer_set_dtype(resolved);
        let subset_of = find_subset_relations(program, name);

        records.push(SetRecord {
            id,
            name: name.clone(),
            aliases,
            size: resolved_set_cardinality(resolved),
            dtype,
            subset_of,
        });
    }

    records
}

fn find_set_aliases(program: &SemanticProgram, set_name: &str) -> Vec<String> {
    program
        .set_aliases
        .iter()
        .filter(|(_, canonical)| *canonical == set_name)
        .map(|(alias, _)| alias.clone())
        .collect()
}

fn resolved_set_cardinality(resolved: &crate::compile::semantic::ResolvedSet) -> usize {
    resolved
        .tuple_rows
        .as_ref()
        .map_or(resolved.values.len(), Vec::len)
}

fn canonical_set_name<'a>(set_name: &'a str, set_aliases: &'a BTreeMap<String, String>) -> &'a str {
    set_aliases.get(set_name).map_or(set_name, String::as_str)
}

fn lookup_set_size_option(
    set_sizes: &BTreeMap<&str, usize>,
    set_aliases: &BTreeMap<String, String>,
    set_name: &str,
) -> Option<usize> {
    let canonical = canonical_set_name(set_name, set_aliases);
    set_sizes
        .get(canonical)
        .copied()
        .or_else(|| set_sizes.get(set_name).copied())
}

fn lookup_set_size(
    set_sizes: &BTreeMap<&str, usize>,
    set_aliases: &BTreeMap<String, String>,
    set_name: &str,
) -> usize {
    lookup_set_size_option(set_sizes, set_aliases, set_name).unwrap_or(0)
}

fn infer_set_dtype(resolved: &crate::compile::semantic::ResolvedSet) -> String {
    if resolved.values.is_empty() {
        return "string".to_string();
    }
    if resolved.values.iter().all(|v| v.parse::<i64>().is_ok()) {
        return "int".to_string();
    }
    if resolved.values.iter().all(|v| v.parse::<f64>().is_ok()) {
        return "float64".to_string();
    }
    "string".to_string()
}

fn find_subset_relations(_program: &SemanticProgram, _set_name: &str) -> Vec<SetRef> {
    // SemanticProgram does not expose subset relations.
    Vec::new()
}

// ─── Variable builder ────────────────────────────────────────────

fn build_variable_records(
    program: &SemanticProgram,
    set_sizes: &BTreeMap<&str, usize>,
) -> Vec<VariableRecord> {
    program
        .variable_families
        .iter()
        .enumerate()
        .map(|(id, family)| {
            let overrides = program.variable_overrides.get(&family.target);
            let (kind, lower, upper) = variable_domain(family, overrides);

            VariableRecord {
                id,
                name: family.target.clone(),
                kind,
                instances: family_instance_count(family, program, set_sizes, &program.set_aliases),
                lower,
                upper,
                set: build_family_set_bindings(family, program, set_sizes, &program.set_aliases),
            }
        })
        .collect()
}

fn family_instance_count(
    family: &FamilySignature,
    program: &SemanticProgram,
    set_sizes: &BTreeMap<&str, usize>,
    set_aliases: &BTreeMap<String, String>,
) -> usize {
    let mut seen_tuple_domains = BTreeSet::new();
    let mut instances = 1usize;

    for index in &family.indices {
        let set_name = family
            .index_domains
            .get(index)
            .map_or(index.as_str(), String::as_str);
        let canonical = canonical_set_name(set_name, set_aliases);
        let Some(resolved_set) = program.set_registry.get(canonical) else {
            continue;
        };

        if resolved_set.tuple_rows.is_some() {
            if seen_tuple_domains.insert(canonical) {
                instances = instances.saturating_mul(resolved_set_cardinality(resolved_set));
            }
        } else {
            instances = instances.saturating_mul(lookup_set_size(set_sizes, set_aliases, set_name));
        }
    }

    instances
}

fn variable_domain(
    family: &FamilySignature,
    overrides: Option<&VariableDeclOverrides>,
) -> (String, Option<BoundValue>, Option<BoundValue>) {
    let (mut kind, mut lower, mut upper): (String, Option<BoundValue>, Option<BoundValue>) =
        match family.target.as_str() {
            "build" => (
                "continuous".to_string(),
                Some(BoundValue::Numeric(0.0)),
                None,
            ),
            "unserved_energy" | "charge" | "discharge" | "generation" => (
                "continuous".to_string(),
                Some(BoundValue::Numeric(0.0)),
                None,
            ),
            "dispatch" => (
                "continuous".to_string(),
                Some(BoundValue::Symbolic("asset-dependent".to_string())),
                None,
            ),
            "commit" | "start" | "shutdown" => (
                "binary".to_string(),
                Some(BoundValue::Numeric(0.0)),
                Some(BoundValue::Numeric(1.0)),
            ),
            _ => ("continuous".to_string(), None, None),
        };

    if let Some(override_def) = overrides {
        if let Some(kind_override) = &override_def.kind {
            kind = match kind_override {
                VariableKindDecl::Continuous => "continuous",
                VariableKindDecl::Integer => "integer",
                VariableKindDecl::Binary => "binary",
            }
            .to_string();
        }
        if let Some(lower_override) = &override_def.lower {
            lower = Some(render_bound(lower_override));
        }
        if let Some(upper_override) = &override_def.upper {
            upper = Some(render_bound(upper_override));
        }
    }

    (kind, lower, upper)
}

fn render_bound(bound: &crate::kdl::source::BoundExpr) -> BoundValue {
    match bound {
        crate::kdl::source::BoundExpr::Literal(crate::kdl::source::LiteralValue::Integer(v)) => {
            BoundValue::Numeric(*v as f64)
        }
        crate::kdl::source::BoundExpr::Literal(crate::kdl::source::LiteralValue::Decimal(text)) => {
            text.parse::<f64>()
                .map_or_else(|_| BoundValue::Symbolic(text.clone()), BoundValue::Numeric)
        }
        crate::kdl::source::BoundExpr::Literal(other) => BoundValue::Symbolic(format!("{other:?}")),
        crate::kdl::source::BoundExpr::Formula(expr) => BoundValue::Symbolic(format!("{expr:?}")),
    }
}

fn build_family_set_bindings(
    family: &FamilySignature,
    program: &SemanticProgram,
    set_sizes: &BTreeMap<&str, usize>,
    set_aliases: &BTreeMap<String, String>,
) -> Vec<SetBinding> {
    family
        .indices
        .iter()
        .map(|index| {
            let set_name = family
                .index_domains
                .get(index)
                .cloned()
                .unwrap_or_else(|| index.clone());
            let size =
                tuple_component_domain_size(program, set_sizes, set_aliases, &set_name, index)
                    .unwrap_or_else(|| lookup_set_size(set_sizes, set_aliases, set_name.as_str()));
            let alias = if index == &set_name {
                None
            } else {
                Some(index.clone())
            };
            SetBinding {
                name: set_name,
                alias,
                size,
            }
        })
        .collect()
}

fn tuple_component_domain_size(
    program: &SemanticProgram,
    set_sizes: &BTreeMap<&str, usize>,
    set_aliases: &BTreeMap<String, String>,
    set_name: &str,
    component: &str,
) -> Option<usize> {
    let canonical_set = canonical_set_name(set_name, set_aliases);
    let tuple_set = program.set_registry.get(canonical_set)?;
    let tuple_components = tuple_set.tuple_components.as_ref()?;
    let component_position = tuple_components.iter().position(|name| name == component)?;

    let domain_name = tuple_set
        .tuple_component_domains
        .as_ref()
        .and_then(|domains| domains.get(component_position))
        .map_or(component, String::as_str);

    lookup_set_size_option(set_sizes, set_aliases, domain_name)
}

// ─── Parameter builder ───────────────────────────────────────────

fn build_parameter_records(
    program: &SemanticProgram,
    parameter_targets: &BTreeSet<String>,
    _variable_targets: &BTreeSet<String>,
    set_sizes: &BTreeMap<&str, usize>,
    set_aliases: &BTreeMap<String, String>,
) -> Vec<ParameterRecord> {
    let mut records = Vec::new();
    let mut id = 0;

    // Track both the full declared names (e.g. "build_cost[asset_id]") and
    // their base names (e.g. "build_cost") so we can suppress inferred
    // duplicates that match a declared parameter.
    let mut declared_bases = BTreeSet::new();

    let declared_params = program
        .parameters
        .asset
        .iter()
        .map(|n| (n, "asset"))
        .chain(program.parameters.series.iter().map(|n| (n, "series")))
        .chain(program.parameters.indexed.iter().map(|n| (n, "indexed")));

    for (name, kind) in declared_params {
        // Extract base name: "build_cost[asset_id]" → "build_cost"
        let base = name.split('[').next().unwrap_or(name);
        declared_bases.insert(base.to_string());

        // For the set binding, use the index from the declared name if present
        let sets = if let Some(idx_part) = name.strip_prefix(base).and_then(|s| s.strip_prefix('['))
        {
            let idx_part = idx_part.strip_suffix(']').unwrap_or(idx_part);
            idx_part
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|set_name| SetBinding {
                    name: set_name.to_string(),
                    alias: None,
                    size: lookup_set_size(set_sizes, set_aliases, set_name),
                })
                .collect()
        } else {
            infer_parameter_sets(program, name, set_sizes, set_aliases)
        };

        records.push(ParameterRecord {
            id,
            name: base.to_string(),
            kind: kind.to_string(),
            dtype: infer_parameter_dtype(base),
            set: sets,
        });
        id += 1;
    }

    // Add inferred parameters (referenced but not declared)
    for name in parameter_targets {
        if !declared_bases.contains(name) {
            records.push(ParameterRecord {
                id,
                name: name.clone(),
                kind: "inferred".to_string(),
                dtype: infer_parameter_dtype(name),
                set: infer_parameter_sets(program, name, set_sizes, set_aliases),
            });
            id += 1;
        }
    }

    records
}

fn infer_parameter_dtype(name: &str) -> String {
    let lower = name.to_lowercase();
    if lower.starts_with("is_") || lower.starts_with("has_") || lower.ends_with("_flag") {
        return "bool".to_string();
    }
    "float64".to_string()
}

fn infer_parameter_sets(
    program: &SemanticProgram,
    parameter_name: &str,
    set_sizes: &BTreeMap<&str, usize>,
    set_aliases: &BTreeMap<String, String>,
) -> Vec<SetBinding> {
    // Try to find indexing from constraint/expression usage
    let mut set_refs = Vec::new();

    for constraint in &program.active_constraints {
        collect_parameter_sets_from_constraint_body(
            &constraint.expression,
            parameter_name,
            &constraint.generation_bindings,
            &mut set_refs,
        );
    }

    if !set_refs.is_empty() {
        return set_refs
            .into_iter()
            .map(|set_name| {
                let size = lookup_set_size(set_sizes, set_aliases, set_name.as_str());
                SetBinding {
                    name: set_name,
                    alias: None,
                    size,
                }
            })
            .collect();
    }

    // Fallback: check if param kind implies a default set
    if program
        .parameters
        .asset
        .contains(&parameter_name.to_string())
    {
        if let Some(size) = lookup_set_size_option(set_sizes, set_aliases, "asset_id") {
            return vec![SetBinding {
                name: "asset_id".to_string(),
                alias: None,
                size,
            }];
        }
    }

    if program
        .parameters
        .series
        .contains(&parameter_name.to_string())
    {
        if let Some(size) = lookup_set_size_option(set_sizes, set_aliases, "time") {
            return vec![SetBinding {
                name: "time".to_string(),
                alias: None,
                size,
            }];
        }
    }

    Vec::new()
}

fn collect_parameter_sets_from_constraint_body(
    body: &ConstraintBody,
    parameter_name: &str,
    bindings: &[crate::kdl::source::GenerationBinding],
    out: &mut Vec<String>,
) {
    let symbol_to_set: BTreeMap<&str, &str> = bindings
        .iter()
        .map(|b| (b.variable.as_str(), b.domain.as_str()))
        .collect();

    match body {
        ConstraintBody::Comparison { left, right, .. } => {
            collect_parameter_sets_from_expr(left, parameter_name, &symbol_to_set, out);
            collect_parameter_sets_from_expr(right, parameter_name, &symbol_to_set, out);
        }
        ConstraintBody::Range {
            lower,
            middle,
            upper,
            ..
        } => {
            collect_parameter_sets_from_expr(lower, parameter_name, &symbol_to_set, out);
            collect_parameter_sets_from_expr(middle, parameter_name, &symbol_to_set, out);
            collect_parameter_sets_from_expr(upper, parameter_name, &symbol_to_set, out);
        }
    }
}

fn collect_parameter_sets_from_expr(
    expr: &Expr,
    parameter_name: &str,
    symbol_to_set: &BTreeMap<&str, &str>,
    out: &mut Vec<String>,
) {
    match expr {
        Expr::Indexed { target, indices } if target == parameter_name => {
            for index in indices {
                if let Expr::Identifier(symbol) = index {
                    if let Some(&set_name) = symbol_to_set.get(symbol.as_str()) {
                        if !out.iter().any(|s| s == set_name) {
                            out.push(set_name.to_string());
                        }
                    }
                }
            }
        }
        Expr::Indexed { indices, .. } => {
            for index in indices {
                collect_parameter_sets_from_expr(index, parameter_name, symbol_to_set, out);
            }
        }
        Expr::Unary { expr, .. } => {
            collect_parameter_sets_from_expr(expr, parameter_name, symbol_to_set, out);
        }
        Expr::Binary { left, right, .. } | Expr::Comparison { left, right, .. } => {
            collect_parameter_sets_from_expr(left, parameter_name, symbol_to_set, out);
            collect_parameter_sets_from_expr(right, parameter_name, symbol_to_set, out);
        }
        Expr::FunctionCall { args, .. } => {
            for arg in args {
                collect_parameter_sets_from_expr(arg, parameter_name, symbol_to_set, out);
            }
        }
        Expr::Reduction(reduction) => {
            // Extend symbol_to_set with reduction bindings
            let mut extended = symbol_to_set.clone();
            for binding in &reduction.bindings {
                if let algebra::BindingPattern::Name(name) = &binding.pattern {
                    extended.insert(name.as_str(), binding.domain.as_str());
                }
            }
            collect_parameter_sets_from_expr(&reduction.body, parameter_name, &extended, out);
        }
        Expr::Number(_) | Expr::String(_) | Expr::Boolean(_) | Expr::Identifier(_) => {}
    }
}

// ─── Expression builder ──────────────────────────────────────────

fn build_expression_records(
    program: &SemanticProgram,
    variable_targets: &BTreeSet<String>,
    parameter_targets: &BTreeSet<String>,
) -> Vec<ExpressionRecord> {
    program
        .active_expressions
        .iter()
        .enumerate()
        .map(|(id, expr)| {
            let uses = build_uses(&expr.formula, variable_targets, parameter_targets);
            ExpressionRecord {
                id,
                name: expr.name.clone(),
                formula: expr.formula_text.clone(),
                uses,
            }
        })
        .collect()
}

fn build_uses(
    expr: &Expr,
    variable_targets: &BTreeSet<String>,
    parameter_targets: &BTreeSet<String>,
) -> Vec<UseRef> {
    let mut targets = BTreeSet::new();
    collect_indexed_targets(expr, &mut targets);

    let mut uses = Vec::new();
    for target in &targets {
        if variable_targets.contains(target) {
            uses.push(UseRef {
                name: target.clone(),
                kind: "variable".to_string(),
                role: Some("term".to_string()),
            });
        } else if parameter_targets.contains(target) {
            uses.push(UseRef {
                name: target.clone(),
                kind: "parameter".to_string(),
                role: Some("coefficient".to_string()),
            });
        }
    }
    uses
}

fn collect_variable_targets(program: &SemanticProgram) -> BTreeSet<String> {
    program
        .variable_families
        .iter()
        .map(|f| f.target.clone())
        .collect()
}

fn collect_parameter_targets(
    program: &SemanticProgram,
    variable_targets: &BTreeSet<String>,
) -> BTreeSet<String> {
    let set_targets: BTreeSet<String> = program.set_registry.keys().cloned().collect();
    let mut targets = BTreeSet::new();

    for constraint in &program.active_constraints {
        match &constraint.expression {
            ConstraintBody::Comparison { left, right, .. } => {
                collect_indexed_targets(left, &mut targets);
                collect_indexed_targets(right, &mut targets);
            }
            ConstraintBody::Range {
                lower,
                middle,
                upper,
                ..
            } => {
                collect_indexed_targets(lower, &mut targets);
                collect_indexed_targets(middle, &mut targets);
                collect_indexed_targets(upper, &mut targets);
            }
        }
        if let Some(condition) = &constraint.generation_filter {
            collect_indexed_targets(condition, &mut targets);
        }
    }

    collect_indexed_targets(&program.active_objective.expression, &mut targets);
    for report in &program.active_reports {
        collect_indexed_targets(&report.formula, &mut targets);
    }
    for expression in &program.active_expressions {
        collect_indexed_targets(&expression.formula, &mut targets);
    }

    targets
        .into_iter()
        .filter(|t| !variable_targets.contains(t) && !set_targets.contains(t))
        .collect()
}

fn collect_indexed_targets(expr: &Expr, out: &mut BTreeSet<String>) {
    match expr {
        Expr::Indexed { target, indices } => {
            out.insert(target.clone());
            for index in indices {
                collect_indexed_targets(index, out);
            }
        }
        Expr::Unary { expr, .. } => collect_indexed_targets(expr, out),
        Expr::Binary { left, right, .. } | Expr::Comparison { left, right, .. } => {
            collect_indexed_targets(left, out);
            collect_indexed_targets(right, out);
        }
        Expr::FunctionCall { args, .. } => {
            for arg in args {
                collect_indexed_targets(arg, out);
            }
        }
        Expr::Reduction(reduction) => {
            collect_indexed_targets(&reduction.body, out);
            for filter in &reduction.filters {
                collect_indexed_targets(filter, out);
            }
        }
        Expr::Number(_) | Expr::String(_) | Expr::Boolean(_) | Expr::Identifier(_) => {}
    }
}

// ─── Rendering ───────────────────────────────────────────────────

/// Fields inside `[[record]]` entries that should be rendered as inline
/// arrays of inline tables (uv.lock style) rather than as sub-tables.
const INLINE_ARRAY_FIELDS: &[&str] = &[
    "set",
    "over",
    "scope",
    "lhs",
    "rhs",
    "uses",
    "term",
    "reduce_over",
    "subset_of",
    "counts",
    "memory",
];

#[derive(Debug, Error)]
pub enum InspectRenderError {
    #[error("failed to serialize inspect payload as TOML: {0}")]
    Serialize(#[from] toml::ser::Error),
    #[error("failed to parse inspect TOML for inline formatting: {0}")]
    InlineFormat(#[from] toml_edit::TomlError),
}

pub fn render_toml(payload: &InspectPayload) -> Result<String, InspectRenderError> {
    let raw = toml::to_string_pretty(payload)?;
    // Parse into a document so we can mark nested tables as inline
    let mut doc: toml_edit::DocumentMut = raw.parse()?;

    inlinify_document(&mut doc);

    Ok(doc.to_string())
}

pub fn render_json(payload: &InspectPayload) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(payload)
}

/// Walk the document and convert specific nested tables to inline form.
fn inlinify_document(doc: &mut toml_edit::DocumentMut) {
    // Process top-level arrays of tables: [[set]], [[variable]], etc.
    let keys: Vec<String> = doc.as_table().iter().map(|(k, _)| k.to_string()).collect();

    for key in &keys {
        if let Some(item) = doc.get_mut(key) {
            match item {
                toml_edit::Item::ArrayOfTables(aot) => {
                    for table in aot.iter_mut() {
                        inlinify_table_fields(table);
                    }
                }
                toml_edit::Item::Table(table) => {
                    inlinify_table_fields(table);
                }
                _ => {}
            }
        }
    }
}

/// For a given table, convert known fields to inline arrays of inline tables.
fn inlinify_table_fields(table: &mut toml_edit::Table) {
    let field_keys: Vec<String> = table.iter().map(|(k, _)| k.to_string()).collect();

    for field_key in &field_keys {
        if !INLINE_ARRAY_FIELDS.contains(&field_key.as_str()) {
            continue;
        }

        // Handle the case where the field is an array of tables
        if let Some(toml_edit::Item::ArrayOfTables(aot)) = table.get(field_key) {
            let mut inline_array = toml_edit::Array::new();
            for sub_table in aot {
                let inline = table_to_inline_table(sub_table);
                inline_array.push(toml_edit::Value::InlineTable(inline));
            }
            format_multiline_array(&mut inline_array);
            table.insert(field_key, toml_edit::Item::Value(inline_array.into()));
        }
        // Handle the case where the field is a regular table (e.g. [meta.counts])
        else if let Some(toml_edit::Item::Table(sub_table)) = table.get(field_key) {
            let inline = table_to_inline_table(sub_table);
            table.insert(
                field_key,
                toml_edit::Item::Value(toml_edit::Value::InlineTable(inline)),
            );
        }
        // Handle existing inline arrays — recurse into their elements
        else if let Some(toml_edit::Item::Value(toml_edit::Value::Array(arr))) =
            table.get_mut(field_key)
        {
            // Already an array — check if elements need inlining
            for item in arr.iter_mut() {
                if let toml_edit::Value::InlineTable(it) = item {
                    inlinify_inline_table(it);
                }
            }
        }
    }

    // Also handle 'source' as an inline table (not an array)
    if let Some(toml_edit::Item::Table(sub_table)) = table.get("source") {
        let inline = table_to_inline_table(sub_table);
        table.insert(
            "source",
            toml_edit::Item::Value(toml_edit::Value::InlineTable(inline)),
        );
    }
}

fn table_to_inline_table(table: &toml_edit::Table) -> toml_edit::InlineTable {
    let mut inline = toml_edit::InlineTable::new();
    for (key, item) in table {
        if let toml_edit::Item::Value(value) = item {
            inline.insert(key, value.clone());
        } else if let toml_edit::Item::ArrayOfTables(aot) = item {
            // Nested array of tables inside an inline table → keep on one line
            let mut arr = toml_edit::Array::new();
            for sub_table in aot {
                arr.push(toml_edit::Value::InlineTable(table_to_inline_table(
                    sub_table,
                )));
            }
            inline.insert(key, toml_edit::Value::Array(arr));
        } else if let toml_edit::Item::Table(sub_table) = item {
            inline.insert(
                key,
                toml_edit::Value::InlineTable(table_to_inline_table(sub_table)),
            );
        }
    }
    inline
}

fn inlinify_inline_table(it: &mut toml_edit::InlineTable) {
    let keys: Vec<String> = it.iter().map(|(k, _)| k.to_string()).collect();
    for key in keys {
        if INLINE_ARRAY_FIELDS.contains(&key.as_str()) {
            if let Some(toml_edit::Value::Array(arr)) = it.get_mut(&key) {
                for item in arr.iter_mut() {
                    if let toml_edit::Value::InlineTable(sub_it) = item {
                        inlinify_inline_table(sub_it);
                    }
                }
            }
        }
    }
}

/// Format an inline array so each element appears on its own line with
/// consistent indentation, like uv.lock:
///
/// ```toml
/// field = [
///     { name = "a", size = 4 },
///     { name = "b", size = 24 },
/// ]
/// ```
fn format_multiline_array(arr: &mut toml_edit::Array) {
    if arr.is_empty() {
        return;
    }
    arr.set_trailing("\n");
    arr.set_trailing_comma(true);
    for item in arr.iter_mut() {
        item.decor_mut().set_prefix("\n    ");
    }
}
