use super::{
    ChronologyRecord, ConstraintRecord, ObjectiveRecord, ObjectiveTermRef, ReportRecord,
    SetBinding, SetRef, SourceRef, TermRef, canonical_set_name, collect_indexed_targets,
    lookup_set_size, resolved_set_cardinality,
};
use crate::kdl::algebra::{self, ComparisonOp, ConstraintBody, Expr, ReductionOp};
use crate::kdl::semantic::{ResolvedConstraint, SemanticProgram};
use std::collections::{BTreeMap, BTreeSet};

// ─── Constraint builder ──────────────────────────────────────────

pub(super) fn build_constraint_records(
    program: &SemanticProgram,
    variable_targets: &BTreeSet<String>,
    parameter_targets: &BTreeSet<String>,
    set_sizes: &BTreeMap<&str, usize>,
    set_aliases: &BTreeMap<String, String>,
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
                set_aliases,
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
    set_aliases: &BTreeMap<String, String>,
) -> ConstraintRecord {
    let scope = build_constraint_scope(constraint, set_sizes, set_aliases);
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
                set_aliases,
            );
            let rhs = build_term_refs(
                right,
                variable_targets,
                parameter_targets,
                &symbol_to_set,
                set_sizes,
                set_aliases,
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
                set_aliases,
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
    set_aliases: &BTreeMap<String, String>,
) -> Vec<SetBinding> {
    constraint
        .generation_bindings
        .iter()
        .map(|binding| {
            let size = lookup_set_size(set_sizes, set_aliases, binding.domain.as_str());
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
        let canonical_domain = canonical_set_name(binding.domain.as_str(), &program.set_aliases);

        if let Some(resolved_set) = program.set_registry.get(canonical_domain) {
            // Non-tuple sets are always counted; tuple-domain sets are counted only
            // once per canonical domain to avoid Cartesian overcounting when
            // multiple bindings share the same tuple domain.
            let should_count =
                resolved_set.tuple_rows.is_none() || seen_tuple_domains.insert(canonical_domain);
            if should_count {
                instances = instances.saturating_mul(resolved_set_cardinality(resolved_set));
            }
            continue;
        }

        let size = lookup_set_size(set_sizes, &program.set_aliases, binding.domain.as_str());
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
    set_aliases: &BTreeMap<String, String>,
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
            set_aliases,
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
    set_aliases: &BTreeMap<String, String>,
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
                        let size = lookup_set_size(set_sizes, set_aliases, set_name.as_str());
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
                    set_aliases,
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
                set_aliases,
                out,
            );
            collect_term_refs_from_expr(
                right,
                variable_targets,
                parameter_targets,
                symbol_to_set,
                set_sizes,
                set_aliases,
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
                set_aliases,
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
                    set_aliases,
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
                set_aliases,
                out,
            );
            collect_term_refs_from_expr(
                right,
                variable_targets,
                parameter_targets,
                symbol_to_set,
                set_sizes,
                set_aliases,
                out,
            );
        }
    }
}

// ─── Objective builder ───────────────────────────────────────────

pub(super) fn build_objective_record(
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

pub(super) fn build_report_records(program: &SemanticProgram) -> Vec<ReportRecord> {
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

pub(super) fn build_chronology(program: &SemanticProgram) -> Option<ChronologyRecord> {
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
