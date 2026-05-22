use crate::compile::semantic::error::SemanticError;
use crate::compile::semantic::types::{
    ResolvedConstraint, ResolvedDualReport, ResolvedExpression, ResolvedObjective, ResolvedReport,
    ResolvedVariableReport,
};
use arco_kdl::algebra::{ConstraintBody, collect_named_expression_dependencies};
use arco_kdl::source::{ModelDecl, ReportKind, ScenarioDecl};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

type ResolvedReports = (
    Vec<ResolvedReport>,
    Vec<ResolvedDualReport>,
    Vec<ResolvedVariableReport>,
);

pub(crate) fn resolve_model_scenario_reports(
    model: &ModelDecl,
    scenario: &ScenarioDecl,
    active_constraints: &[ResolvedConstraint],
    entrypoint: &Path,
) -> Result<ResolvedReports, SemanticError> {
    let mut reports = Vec::new();
    let mut dual_reports = Vec::new();
    let mut variable_reports = Vec::new();

    for report_decl in &scenario.reports {
        match report_decl.kind {
            ReportKind::Scalar => {
                if report_decl.target == model.optimize.name {
                    reports.push(ResolvedReport {
                        name: model.optimize.name.clone(),
                        formula_text: model.optimize.expression.clone(),
                        formula: model.optimize.parsed_expression.clone(),
                    });
                    continue;
                }

                if let Some(expression) = model
                    .expressions
                    .iter()
                    .find(|expression| expression.name == report_decl.target)
                {
                    reports.push(ResolvedReport {
                        name: expression.name.clone(),
                        formula_text: expression.formula.clone(),
                        formula: expression.parsed_formula.clone(),
                    });
                    continue;
                }

                if let Some(control) = model.controls.iter().find(|c| c.name == report_decl.target)
                {
                    let compiled_family =
                        crate::compile::semantic::FamilySignature::from_index_decls(
                            &control.name,
                            &control.indices,
                        )
                        .render();
                    variable_reports.push(ResolvedVariableReport {
                        control_name: control.name.clone(),
                        indices: control.indices.iter().map(|i| i.name.clone()).collect(),
                        compiled_family,
                        filter: report_decl.parsed_filter_expression.clone(),
                    });
                    continue;
                }

                return Err(SemanticError::MissingDeclaration {
                    kind: "expression or control",
                    name: report_decl.target.clone(),
                    path: entrypoint.to_path_buf(),
                });
            }
            ReportKind::Dual => {
                let target = &report_decl.target;
                let exists = active_constraints
                    .iter()
                    .any(|constraint| constraint.name == *target);
                if !exists {
                    return Err(SemanticError::MissingDeclaration {
                        kind: "constraint",
                        name: target.clone(),
                        path: entrypoint.to_path_buf(),
                    });
                }
                dual_reports.push(ResolvedDualReport {
                    constraint_name: target.clone(),
                });
            }
        }
    }

    Ok((reports, dual_reports, variable_reports))
}

pub(crate) fn resolve_active_model_expressions(
    model: &ModelDecl,
    objective: &ResolvedObjective,
    reports: &[ResolvedReport],
    constraints: &[ResolvedConstraint],
    entrypoint: &Path,
) -> Result<Vec<ResolvedExpression>, SemanticError> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum VisitState {
        Visiting,
        Done,
    }

    let expression_index = model
        .expressions
        .iter()
        .map(|expression| (expression.name.as_str(), expression))
        .collect::<BTreeMap<_, _>>();

    let mut resolved = BTreeSet::new();
    let mut states: BTreeMap<String, VisitState> = BTreeMap::new();
    let mut stack: Vec<String> = Vec::new();

    fn visit_expression_name(
        name: &str,
        expression_index: &BTreeMap<&str, &arco_kdl::source::ExpressionDecl>,
        resolved: &mut BTreeSet<String>,
        states: &mut BTreeMap<String, VisitState>,
        stack: &mut Vec<String>,
        entrypoint: &Path,
    ) -> Result<(), SemanticError> {
        if let Some(state) = states.get(name) {
            if *state == VisitState::Done {
                return Ok(());
            }
            if let Some(start) = stack.iter().position(|item| item == name) {
                let mut cycle = stack[start..].to_vec();
                cycle.push(name.to_string());
                return Err(SemanticError::ExpressionCycle {
                    cycle: cycle.join(" -> "),
                    path: entrypoint.to_path_buf(),
                });
            }
        }

        let expression =
            expression_index
                .get(name)
                .ok_or_else(|| SemanticError::MissingDeclaration {
                    kind: "expression",
                    name: name.to_string(),
                    path: entrypoint.to_path_buf(),
                })?;

        states.insert(name.to_string(), VisitState::Visiting);
        stack.push(name.to_string());

        for dependency in collect_expression_decl_dependencies(expression) {
            if expression_index.contains_key(dependency.as_str()) {
                visit_expression_name(
                    &dependency,
                    expression_index,
                    resolved,
                    states,
                    stack,
                    entrypoint,
                )?;
            }
        }

        stack.pop();
        states.insert(name.to_string(), VisitState::Done);
        resolved.insert(name.to_string());
        Ok(())
    }

    for dependency in collect_named_expression_dependencies(&objective.expression) {
        if expression_index.contains_key(dependency.as_str()) {
            visit_expression_name(
                &dependency,
                &expression_index,
                &mut resolved,
                &mut states,
                &mut stack,
                entrypoint,
            )?;
        }
    }

    for constraint in constraints {
        for dependency in
            collect_named_expression_dependencies_from_constraint(&constraint.expression)
        {
            if expression_index.contains_key(dependency.as_str()) {
                visit_expression_name(
                    &dependency,
                    &expression_index,
                    &mut resolved,
                    &mut states,
                    &mut stack,
                    entrypoint,
                )?;
            }
        }
    }

    for report in reports {
        if expression_index.contains_key(report.name.as_str()) {
            visit_expression_name(
                &report.name,
                &expression_index,
                &mut resolved,
                &mut states,
                &mut stack,
                entrypoint,
            )?;
        }
        for dependency in collect_named_expression_dependencies(&report.formula) {
            if expression_index.contains_key(dependency.as_str()) {
                visit_expression_name(
                    &dependency,
                    &expression_index,
                    &mut resolved,
                    &mut states,
                    &mut stack,
                    entrypoint,
                )?;
            }
        }
    }

    let mut expressions = resolved
        .into_iter()
        .filter_map(|name| {
            expression_index
                .get(name.as_str())
                .map(|expression| ResolvedExpression {
                    name: expression.name.clone(),
                    formula_text: expression.formula.clone(),
                    formula: expression.parsed_formula.clone(),
                    generation_bindings: expression.generation_bindings.clone(),
                    generation_filter_text: expression.generation_filter.clone(),
                    generation_filter: expression.parsed_generation_filter.clone(),
                })
        })
        .collect::<Vec<_>>();
    expressions.sort_by_key(|expression| expression.name.clone());
    Ok(expressions)
}

fn collect_expression_decl_dependencies(
    expression: &arco_kdl::source::ExpressionDecl,
) -> BTreeSet<String> {
    let mut dependencies = collect_named_expression_dependencies(&expression.parsed_formula);
    if let Some(filter) = &expression.parsed_generation_filter {
        dependencies.extend(collect_named_expression_dependencies(filter));
    }
    dependencies
}

fn collect_named_expression_dependencies_from_constraint(
    constraint: &ConstraintBody,
) -> BTreeSet<String> {
    match constraint {
        ConstraintBody::Comparison { left, right, .. } => {
            let mut names = collect_named_expression_dependencies(left);
            names.extend(collect_named_expression_dependencies(right));
            names
        }
        ConstraintBody::Range {
            lower,
            middle,
            upper,
            ..
        } => {
            let mut names = collect_named_expression_dependencies(lower);
            names.extend(collect_named_expression_dependencies(middle));
            names.extend(collect_named_expression_dependencies(upper));
            names
        }
    }
}
