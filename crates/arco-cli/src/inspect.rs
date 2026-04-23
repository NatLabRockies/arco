use arco_kdl::ObjectiveSense;
use arco_kdl::algebra::{self, ComparisonOp, ConstraintBody, Expr, ReductionOp};
use arco_kdl::semantic::{
    FamilySignature, ResolvedConstraint, SemanticProgram, VariableDeclOverrides,
};
use arco_kdl::source::VariableKindDecl;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

// ─── Top-level inspect payload ───────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectPayload {
    pub meta: Meta,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub set: Vec<SetRecord>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub variable: Vec<VariableRecord>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parameter: Vec<ParameterRecord>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub expression: Vec<ExpressionRecord>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub constraint: Vec<ConstraintRecord>,
    pub objective: ObjectiveRecord,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub report: Vec<ReportRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chronology: Option<ChronologyRecord>,
}

// ─── Records ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub entrypoint: String,
    pub scenario: String,
    pub counts: Counts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Counts {
    pub set: usize,
    pub variable: usize,
    pub parameter: usize,
    pub constraint: usize,
    pub expression: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetRecord {
    pub id: usize,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    pub size: usize,
    pub dtype: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub subset_of: Vec<SetRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetRef {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetBinding {
    pub name: String,
    #[serde(rename = "as", skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    pub size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableRecord {
    pub id: usize,
    pub name: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lower: Option<BoundValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upper: Option<BoundValue>,
    pub set: Vec<SetBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BoundValue {
    Numeric(f64),
    Symbolic(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterRecord {
    pub id: usize,
    pub name: String,
    pub kind: String,
    pub dtype: String,
    pub set: Vec<SetBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpressionRecord {
    pub id: usize,
    pub name: String,
    pub formula: String,
    pub uses: Vec<UseRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseRef {
    pub name: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintRecord {
    pub id: usize,
    pub name: String,
    pub relation: String,
    pub template: String,
    pub source: SourceRef,
    pub scope: Vec<SetBinding>,
    pub lhs: Vec<TermRef>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rhs: Vec<TermRef>,
    pub instances: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRef {
    pub kind: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermRef {
    pub name: String,
    pub kind: String,
    pub over: Vec<SetBinding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduction: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub reduce_over: Vec<SetRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveRecord {
    pub name: String,
    pub sense: ObjectiveSense,
    pub term: Vec<ObjectiveTermRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveTermRef {
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRecord {
    pub id: usize,
    pub name: String,
    pub formula: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronologyRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_boundary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_boundary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_commitment_boundary: Option<String>,
}

// ─── Builder ─────────────────────────────────────────────────────

pub fn build_inspect_payload(entrypoint: &Path, program: &SemanticProgram) -> InspectPayload {
    let filtered_set_names: &[&str] = &["assets", "candidate_assets"];

    let variable_targets = collect_variable_targets(program);
    let parameter_targets = collect_parameter_targets(program, &variable_targets);

    // Build set records
    let set_records = build_set_records(program, filtered_set_names);

    // Build a lookup from set name → size for bindings
    let mut set_sizes: BTreeMap<&str, usize> = program
        .set_registry
        .iter()
        .map(|(name, resolved)| (name.as_str(), resolved_set_cardinality(resolved)))
        .collect();
    for (alias, canonical) in &program.set_aliases {
        if let Some(size) = set_sizes.get(canonical.as_str()).copied() {
            set_sizes.entry(alias.as_str()).or_insert(size);
        }
    }

    // Build variable records
    let variable_records = build_variable_records(program, &set_sizes);

    // Build parameter records
    let parameter_records =
        build_parameter_records(program, &parameter_targets, &variable_targets, &set_sizes);

    // Build expression records
    let expression_records =
        build_expression_records(program, &variable_targets, &parameter_targets);

    // Build constraint records
    let constraint_records =
        build_constraint_records(program, &variable_targets, &parameter_targets, &set_sizes);

    // Build objective record
    let objective_record = build_objective_record(program, &variable_targets, &parameter_targets);

    // Build report records
    let report_records = build_report_records(program);

    // Build chronology
    let chronology = build_chronology(program);

    InspectPayload {
        meta: Meta {
            entrypoint: entrypoint.display().to_string(),
            scenario: program.active_scenario.clone(),
            counts: Counts {
                set: set_records.len(),
                variable: variable_records.len(),
                parameter: parameter_records.len(),
                constraint: constraint_records.len(),
                expression: expression_records.len(),
            },
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

// ─── Set builder ─────────────────────────────────────────────────

fn build_set_records(program: &SemanticProgram, filtered_set_names: &[&str]) -> Vec<SetRecord> {
    let mut records = Vec::new();

    for (id, (name, resolved)) in program
        .set_registry
        .iter()
        .filter(|(name, _)| !filtered_set_names.contains(&name.as_str()))
        .enumerate()
    {
        let alias = find_set_alias(program, name);
        let dtype = infer_set_dtype(resolved);
        let subset_of = find_subset_relations(program, name);

        records.push(SetRecord {
            id,
            name: name.clone(),
            alias,
            size: resolved_set_cardinality(resolved),
            dtype,
            subset_of,
        });
    }

    records
}

fn find_set_alias(program: &SemanticProgram, set_name: &str) -> Option<String> {
    program
        .set_aliases
        .iter()
        .find_map(|(alias, canonical)| (canonical == set_name).then(|| alias.clone()))
}

fn resolved_set_cardinality(resolved: &arco_kdl::semantic::ResolvedSet) -> usize {
    resolved
        .tuple_rows
        .as_ref()
        .map_or(resolved.values.len(), Vec::len)
}

fn infer_set_dtype(resolved: &arco_kdl::semantic::ResolvedSet) -> String {
    if resolved.values.is_empty() {
        return "string".to_string();
    }
    // Check if all values are integers
    if resolved.values.iter().all(|v| v.parse::<i64>().is_ok()) {
        return "int".to_string();
    }
    // Check if all values are floats
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
                lower,
                upper,
                set: build_family_set_bindings(family, set_sizes),
            }
        })
        .collect()
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

fn render_bound(bound: &arco_kdl::source::BoundExpr) -> BoundValue {
    match bound {
        arco_kdl::source::BoundExpr::Literal(arco_kdl::source::LiteralValue::Integer(v)) => {
            BoundValue::Numeric(*v as f64)
        }
        arco_kdl::source::BoundExpr::Literal(arco_kdl::source::LiteralValue::Decimal(text)) => text
            .parse::<f64>()
            .map_or_else(|_| BoundValue::Symbolic(text.clone()), BoundValue::Numeric),
        arco_kdl::source::BoundExpr::Literal(other) => BoundValue::Symbolic(format!("{other:?}")),
        arco_kdl::source::BoundExpr::Formula(expr) => BoundValue::Symbolic(format!("{expr:?}")),
    }
}

fn build_family_set_bindings(
    family: &FamilySignature,
    set_sizes: &BTreeMap<&str, usize>,
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
            let size = set_sizes.get(set_name.as_str()).copied().unwrap_or(0);
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

// ─── Parameter builder ───────────────────────────────────────────

fn build_parameter_records(
    program: &SemanticProgram,
    parameter_targets: &BTreeSet<String>,
    _variable_targets: &BTreeSet<String>,
    set_sizes: &BTreeMap<&str, usize>,
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
                    size: set_sizes.get(set_name).copied().unwrap_or(0),
                })
                .collect()
        } else {
            infer_parameter_sets(program, name, set_sizes)
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
                set: infer_parameter_sets(program, name, set_sizes),
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
                let size = set_sizes.get(set_name.as_str()).copied().unwrap_or(0);
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
        if let Some(&size) = set_sizes.get("asset_id") {
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
        if let Some(&size) = set_sizes.get("time") {
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
    bindings: &[arco_kdl::source::GenerationBinding],
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

// ─── Constraint builder ──────────────────────────────────────────

fn build_constraint_records(
    program: &SemanticProgram,
    variable_targets: &BTreeSet<String>,
    parameter_targets: &BTreeSet<String>,
    set_sizes: &BTreeMap<&str, usize>,
) -> Vec<ConstraintRecord> {
    program
        .active_constraints
        .iter()
        .enumerate()
        .map(|(id, constraint)| {
            build_constraint_record(
                id,
                constraint,
                program,
                variable_targets,
                parameter_targets,
                set_sizes,
            )
        })
        .collect()
}

fn build_constraint_record(
    id: usize,
    constraint: &ResolvedConstraint,
    program: &SemanticProgram,
    variable_targets: &BTreeSet<String>,
    parameter_targets: &BTreeSet<String>,
    set_sizes: &BTreeMap<&str, usize>,
) -> ConstraintRecord {
    let scope = build_constraint_scope(constraint, set_sizes);
    let instances = estimate_constraint_instances(program, constraint, set_sizes);

    let symbol_to_set: BTreeMap<&str, &str> = constraint
        .generation_bindings
        .iter()
        .map(|b| (b.variable.as_str(), b.domain.as_str()))
        .collect();

    let (relation, lhs_terms, rhs_terms) = match &constraint.expression {
        ConstraintBody::Comparison { op, left, right } => {
            let relation = relation_name(*op);
            let lhs = build_term_refs(
                left,
                variable_targets,
                parameter_targets,
                &symbol_to_set,
                set_sizes,
            );
            let rhs = build_term_refs(
                right,
                variable_targets,
                parameter_targets,
                &symbol_to_set,
                set_sizes,
            );
            (relation, lhs, rhs)
        }
        ConstraintBody::Range {
            lower_op, middle, ..
        } => {
            // For range constraints, represent as the primary comparison
            let relation = relation_name(*lower_op);
            let lhs = build_term_refs(
                middle,
                variable_targets,
                parameter_targets,
                &symbol_to_set,
                set_sizes,
            );
            (relation, lhs, Vec::new())
        }
    };

    ConstraintRecord {
        id,
        name: constraint.name.clone(),
        relation,
        template: constraint.expression_text.clone(),
        source: SourceRef {
            kind: constraint.source_kind.clone(),
            name: constraint.source_name.clone(),
        },
        scope,
        lhs: lhs_terms,
        rhs: rhs_terms,
        instances,
    }
}

fn build_constraint_scope(
    constraint: &ResolvedConstraint,
    set_sizes: &BTreeMap<&str, usize>,
) -> Vec<SetBinding> {
    constraint
        .generation_bindings
        .iter()
        .map(|binding| {
            let size = set_sizes.get(binding.domain.as_str()).copied().unwrap_or(0);
            let alias = if binding.variable == binding.domain {
                None
            } else {
                Some(binding.variable.clone())
            };
            SetBinding {
                name: binding.domain.clone(),
                alias,
                size,
            }
        })
        .collect()
}

fn estimate_constraint_instances(
    program: &SemanticProgram,
    constraint: &ResolvedConstraint,
    set_sizes: &BTreeMap<&str, usize>,
) -> usize {
    let mut instances = 1usize;
    let mut seen_tuple_domains = BTreeSet::new();

    for binding in &constraint.generation_bindings {
        let canonical_domain = program
            .set_aliases
            .get(binding.domain.as_str())
            .map_or(binding.domain.as_str(), String::as_str);

        if let Some(resolved_set) = program.set_registry.get(canonical_domain) {
            if resolved_set.tuple_rows.is_some() {
                if seen_tuple_domains.insert(canonical_domain) {
                    instances = instances.saturating_mul(resolved_set_cardinality(resolved_set));
                }
            } else {
                instances = instances.saturating_mul(resolved_set_cardinality(resolved_set));
            }
            continue;
        }

        let size = set_sizes
            .get(canonical_domain)
            .copied()
            .or_else(|| set_sizes.get(binding.domain.as_str()).copied())
            .unwrap_or(0);
        instances = instances.saturating_mul(size);
    }

    instances.max(1)
}

fn build_term_refs(
    expr: &Expr,
    variable_targets: &BTreeSet<String>,
    parameter_targets: &BTreeSet<String>,
    symbol_to_set: &BTreeMap<&str, &str>,
    set_sizes: &BTreeMap<&str, usize>,
) -> Vec<TermRef> {
    let additive_terms = split_additive_terms(expr);
    let mut refs = Vec::new();

    for term in additive_terms {
        collect_term_refs_from_expr(
            &term,
            variable_targets,
            parameter_targets,
            symbol_to_set,
            set_sizes,
            &mut refs,
        );
    }

    refs
}

fn split_additive_terms(expr: &Expr) -> Vec<Expr> {
    match expr {
        Expr::Binary { op, left, right }
            if *op == algebra::BinaryOp::Add || *op == algebra::BinaryOp::Subtract =>
        {
            let mut terms = split_additive_terms(left);
            terms.extend(split_additive_terms(right));
            terms
        }
        _ => vec![expr.clone()],
    }
}

fn collect_term_refs_from_expr(
    expr: &Expr,
    variable_targets: &BTreeSet<String>,
    parameter_targets: &BTreeSet<String>,
    symbol_to_set: &BTreeMap<&str, &str>,
    set_sizes: &BTreeMap<&str, usize>,
    out: &mut Vec<TermRef>,
) {
    match expr {
        Expr::Indexed { target, indices } => {
            let kind = if variable_targets.contains(target) {
                "variable"
            } else if parameter_targets.contains(target) {
                "parameter"
            } else {
                "unknown"
            };

            let over = indices
                .iter()
                .filter_map(|idx| {
                    if let Expr::Identifier(symbol) = idx {
                        let set_name = symbol_to_set
                            .get(symbol.as_str())
                            .map_or(symbol.clone(), |&s| s.to_string());
                        let size = set_sizes.get(set_name.as_str()).copied().unwrap_or(0);
                        let alias = if *symbol == set_name {
                            None
                        } else {
                            Some(symbol.clone())
                        };
                        Some(SetBinding {
                            name: set_name,
                            alias,
                            size,
                        })
                    } else {
                        None
                    }
                })
                .collect();

            // Check if this target is already in the output
            if !out.iter().any(|r| r.name == *target) {
                out.push(TermRef {
                    name: target.clone(),
                    kind: kind.to_string(),
                    over,
                    reduction: None,
                    reduce_over: Vec::new(),
                });
            }
        }
        Expr::Reduction(reduction) => {
            let reduction_op = match reduction.op {
                ReductionOp::Sum => "sum",
            };

            // Extend symbol_to_set with reduction bindings
            let mut extended = symbol_to_set.clone();
            let mut reduce_over_sets = Vec::new();
            for binding in &reduction.bindings {
                if let algebra::BindingPattern::Name(name) = &binding.pattern {
                    extended.insert(name.as_str(), binding.domain.as_str());
                    reduce_over_sets.push(SetRef {
                        name: binding.domain.clone(),
                    });
                }
            }

            // Extract indexed terms from the reduction body
            let body_terms = split_additive_terms(&reduction.body);
            for body_term in body_terms {
                let mut inner_refs = Vec::new();
                collect_term_refs_from_expr(
                    &body_term,
                    variable_targets,
                    parameter_targets,
                    &extended,
                    set_sizes,
                    &mut inner_refs,
                );
                for mut inner_ref in inner_refs {
                    inner_ref.reduction = Some(reduction_op.to_string());
                    inner_ref.reduce_over.clone_from(&reduce_over_sets);
                    if !out.iter().any(|r| r.name == inner_ref.name) {
                        out.push(inner_ref);
                    }
                }
            }
        }
        Expr::Binary { left, right, .. } => {
            // For multiplication etc, descend into both sides
            collect_term_refs_from_expr(
                left,
                variable_targets,
                parameter_targets,
                symbol_to_set,
                set_sizes,
                out,
            );
            collect_term_refs_from_expr(
                right,
                variable_targets,
                parameter_targets,
                symbol_to_set,
                set_sizes,
                out,
            );
        }
        Expr::Unary { expr, .. } => {
            collect_term_refs_from_expr(
                expr,
                variable_targets,
                parameter_targets,
                symbol_to_set,
                set_sizes,
                out,
            );
        }
        Expr::FunctionCall { args, .. } => {
            for arg in args {
                collect_term_refs_from_expr(
                    arg,
                    variable_targets,
                    parameter_targets,
                    symbol_to_set,
                    set_sizes,
                    out,
                );
            }
        }
        Expr::Identifier(_) | Expr::Number(_) | Expr::String(_) | Expr::Boolean(_) => {}
        Expr::Comparison { left, right, .. } => {
            collect_term_refs_from_expr(
                left,
                variable_targets,
                parameter_targets,
                symbol_to_set,
                set_sizes,
                out,
            );
            collect_term_refs_from_expr(
                right,
                variable_targets,
                parameter_targets,
                symbol_to_set,
                set_sizes,
                out,
            );
        }
    }
}

// ─── Objective builder ───────────────────────────────────────────

fn build_objective_record(
    program: &SemanticProgram,
    variable_targets: &BTreeSet<String>,
    parameter_targets: &BTreeSet<String>,
) -> ObjectiveRecord {
    let objective = &program.active_objective;
    let expression_names: BTreeSet<String> = program
        .active_expressions
        .iter()
        .map(|e| e.name.clone())
        .collect();

    let terms = build_objective_terms(
        &objective.expression,
        &expression_names,
        variable_targets,
        parameter_targets,
    );

    ObjectiveRecord {
        name: objective.name.clone(),
        sense: objective.sense,
        term: terms,
    }
}

fn build_objective_terms(
    expr: &Expr,
    expression_names: &BTreeSet<String>,
    variable_targets: &BTreeSet<String>,
    parameter_targets: &BTreeSet<String>,
) -> Vec<ObjectiveTermRef> {
    // First, try to decompose into additive terms at the top level
    let top_terms = split_additive_terms(expr);
    let mut result = Vec::new();

    for term in &top_terms {
        match term {
            Expr::Identifier(name) if expression_names.contains(name) => {
                result.push(ObjectiveTermRef {
                    name: name.clone(),
                    kind: "expression".to_string(),
                });
            }
            Expr::Indexed { target, .. } if variable_targets.contains(target) => {
                result.push(ObjectiveTermRef {
                    name: target.clone(),
                    kind: "variable".to_string(),
                });
            }
            Expr::Indexed { target, .. } if parameter_targets.contains(target) => {
                result.push(ObjectiveTermRef {
                    name: target.clone(),
                    kind: "parameter".to_string(),
                });
            }
            Expr::Reduction(reduction) => {
                // Check if the body references named expressions
                let body_terms = split_additive_terms(&reduction.body);
                let mut found_expressions = false;
                for body_term in &body_terms {
                    if let Expr::Identifier(name) = body_term {
                        if expression_names.contains(name) {
                            result.push(ObjectiveTermRef {
                                name: name.clone(),
                                kind: "expression".to_string(),
                            });
                            found_expressions = true;
                        }
                    }
                }
                if !found_expressions {
                    // Fall back to extracting indexed targets
                    let mut targets = BTreeSet::new();
                    collect_indexed_targets(term, &mut targets);
                    for target in targets {
                        let kind = if variable_targets.contains(&target) {
                            "variable"
                        } else if parameter_targets.contains(&target) {
                            "parameter"
                        } else {
                            continue;
                        };
                        result.push(ObjectiveTermRef {
                            name: target,
                            kind: kind.to_string(),
                        });
                    }
                }
            }
            _ => {
                // Extract named references from complex expressions
                let mut targets = BTreeSet::new();
                collect_indexed_targets(term, &mut targets);
                for target in targets {
                    if expression_names.contains(&target) {
                        result.push(ObjectiveTermRef {
                            name: target,
                            kind: "expression".to_string(),
                        });
                    } else if variable_targets.contains(&target) {
                        result.push(ObjectiveTermRef {
                            name: target,
                            kind: "variable".to_string(),
                        });
                    } else if parameter_targets.contains(&target) {
                        result.push(ObjectiveTermRef {
                            name: target,
                            kind: "parameter".to_string(),
                        });
                    }
                }
            }
        }
    }

    result
}

// ─── Report builder ──────────────────────────────────────────────

fn build_report_records(program: &SemanticProgram) -> Vec<ReportRecord> {
    program
        .active_reports
        .iter()
        .enumerate()
        .map(|(id, report)| ReportRecord {
            id,
            name: report.name.clone(),
            formula: report.formula_text.clone(),
        })
        .collect()
}

// ─── Chronology builder ──────────────────────────────────────────

fn build_chronology(program: &SemanticProgram) -> Option<ChronologyRecord> {
    let c = &program.chronology;
    if c.initial_boundary.is_none()
        && c.terminal_boundary.is_none()
        && c.initial_commitment_boundary.is_none()
    {
        return None;
    }

    Some(ChronologyRecord {
        initial_boundary: c.initial_boundary.clone(),
        terminal_boundary: c.terminal_boundary.clone(),
        initial_commitment_boundary: c.initial_commitment_boundary.clone(),
    })
}

// ─── Shared helpers ──────────────────────────────────────────────

fn relation_name(op: ComparisonOp) -> String {
    match op {
        ComparisonOp::Equal | ComparisonOp::DoubleEqual => "eq",
        ComparisonOp::LessEqual => "le",
        ComparisonOp::GreaterEqual => "ge",
        ComparisonOp::Less => "lt",
        ComparisonOp::Greater => "gt",
        ComparisonOp::NotEqual => "ne",
    }
    .to_string()
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
];

pub fn render_toml(payload: &InspectPayload) -> Result<String, toml::ser::Error> {
    let raw = toml::to_string_pretty(payload)?;
    // Parse into a document so we can mark nested tables as inline
    let mut doc: toml_edit::DocumentMut = raw
        .parse()
        .expect("toml::to_string_pretty should produce valid TOML");

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
