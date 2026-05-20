fn nonlinear_fallback_required(message: &str) -> bool {
    message.contains("non-linear multiplication is not supported")
        || message.contains("must remain scalar")
        || message.contains("cannot appear in linear algebra")
}

fn nonlinear_problem_requires_nlp(problem: &NonlinearProblem) -> bool {
    if nonlinear_expr_requires_nlp(&problem.objective.expression) {
        return true;
    }

    if problem
        .constraints
        .iter()
        .any(|constraint| nonlinear_expr_requires_nlp(&constraint.expression))
    {
        return true;
    }

    problem
        .reports
        .iter()
        .any(|report| nonlinear_expr_requires_nlp(&report.expression))
}

fn nonlinear_expr_requires_nlp(expr: &NonlinearExpr) -> bool {
    matches!(expr_affine_kind(expr), ExprAffineKind::Nonlinear)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExprAffineKind {
    Constant,
    Affine,
    Nonlinear,
}

fn expr_affine_kind(expr: &NonlinearExpr) -> ExprAffineKind {
    match expr {
        NonlinearExpr::Constant(_) => ExprAffineKind::Constant,
        NonlinearExpr::Variable(_) => ExprAffineKind::Affine,
        NonlinearExpr::Unary { op, expr } => match op {
            UnaryOp::Negate => expr_affine_kind(expr),
        },
        NonlinearExpr::Binary { op, left, right } => {
            let left_kind = expr_affine_kind(left);
            let right_kind = expr_affine_kind(right);
            match op {
                BinaryOp::Add | BinaryOp::Subtract => combine_add_sub_kind(left_kind, right_kind),
                BinaryOp::Multiply => combine_multiply_kind(left_kind, right_kind),
                BinaryOp::Divide => combine_divide_kind(left_kind, right_kind),
            }
        }
        NonlinearExpr::FunctionCall { name, args } => {
            if args
                .iter()
                .all(|arg| matches!(expr_affine_kind(arg), ExprAffineKind::Constant))
            {
                return ExprAffineKind::Constant;
            }

            // abs(x), sin(x), cos(x), atan(x), ln(x), sqrt(x), exp(x), pow(x,y)
            // are nonlinear unless all arguments are constants.
            let _ = name;
            ExprAffineKind::Nonlinear
        }
    }
}

fn combine_add_sub_kind(left: ExprAffineKind, right: ExprAffineKind) -> ExprAffineKind {
    if left == ExprAffineKind::Nonlinear || right == ExprAffineKind::Nonlinear {
        return ExprAffineKind::Nonlinear;
    }
    if left == ExprAffineKind::Affine || right == ExprAffineKind::Affine {
        return ExprAffineKind::Affine;
    }
    ExprAffineKind::Constant
}

fn combine_multiply_kind(left: ExprAffineKind, right: ExprAffineKind) -> ExprAffineKind {
    if left == ExprAffineKind::Nonlinear || right == ExprAffineKind::Nonlinear {
        return ExprAffineKind::Nonlinear;
    }
    match (left, right) {
        (ExprAffineKind::Constant, ExprAffineKind::Constant) => ExprAffineKind::Constant,
        (ExprAffineKind::Constant, ExprAffineKind::Affine)
        | (ExprAffineKind::Affine, ExprAffineKind::Constant) => ExprAffineKind::Affine,
        (ExprAffineKind::Affine, ExprAffineKind::Affine) => ExprAffineKind::Nonlinear,
        _ => ExprAffineKind::Nonlinear,
    }
}

fn combine_divide_kind(numerator: ExprAffineKind, denominator: ExprAffineKind) -> ExprAffineKind {
    if numerator == ExprAffineKind::Nonlinear || denominator == ExprAffineKind::Nonlinear {
        return ExprAffineKind::Nonlinear;
    }
    match denominator {
        ExprAffineKind::Constant => numerator,
        ExprAffineKind::Affine => ExprAffineKind::Nonlinear,
        ExprAffineKind::Nonlinear => ExprAffineKind::Nonlinear,
    }
}

#[allow(clippy::too_many_arguments)]
fn compile_nonlinear_problem(
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    named_expressions: &BTreeMap<String, Expr>,
    expression_generation_index: &ExpressionGenerationIndex,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    instantiated_names: &BTreeSet<String>,
    entrypoint: &Path,
) -> Result<NonlinearProblem, CompileError> {
    let objective = NonlinearObjective {
        name: program.active_objective.name.clone(),
        sense: program.active_objective.sense,
        expression: compile_nonlinear_expr(
            &program.active_objective.expression,
            &LinearizationBindings::default(),
            program,
            inputs,
            named_expressions,
            expression_generation_index,
            variable_signatures,
            instantiated_names,
            entrypoint,
        )?,
    };

    let mut constraints = compile_nonlinear_constraint_instances(
        program,
        inputs,
        named_expressions,
        expression_generation_index,
        variable_signatures,
        instantiated_names,
        entrypoint,
    )?;
    constraints.extend(emit_terminal_boundary_nonlinear_constraints(
        program,
        inputs,
        variable_signatures,
        named_expressions,
        expression_generation_index,
        instantiated_names,
        entrypoint,
    )?);

    let reports = program
        .active_reports
        .iter()
        .map(|report| {
            Ok(NonlinearReport {
                name: report.name.clone(),
                expression: compile_nonlinear_expr(
                    &report.formula,
                    &LinearizationBindings::default(),
                    program,
                    inputs,
                    named_expressions,
                    expression_generation_index,
                    variable_signatures,
                    instantiated_names,
                    entrypoint,
                )?,
            })
        })
        .collect::<Result<Vec<_>, CompileError>>()?;

    constraints.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(NonlinearProblem {
        objective,
        constraints,
        reports,
    })
}

#[allow(clippy::too_many_arguments)]
fn compile_nonlinear_constraint_instances(
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    named_expressions: &BTreeMap<String, Expr>,
    expression_generation_index: &ExpressionGenerationIndex,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    instantiated_names: &BTreeSet<String>,
    entrypoint: &Path,
) -> Result<Vec<NonlinearConstraint>, CompileError> {
    let mut constraints = Vec::new();
    for constraint in &program.active_constraints {
        let (binding_order, implicit_scopes, explicit_scopes) =
            if constraint.generation_bindings.is_empty() {
                let (binding_order, scopes) = infer_constraint_generation_bindings(
                    constraint,
                    program,
                    inputs,
                    variable_signatures,
                    entrypoint,
                )?;
                (binding_order, Some(scopes), None)
            } else {
                (
                    constraint
                        .generation_bindings
                        .iter()
                        .map(|binding| binding.variable.clone())
                        .collect::<Vec<_>>(),
                    None,
                    Some(expand_generation_bindings(
                        &constraint.generation_bindings,
                        inputs,
                        program,
                        entrypoint,
                        &constraint.diagnostic_id,
                    )?),
                )
            };
        let mut empty_subset_keys = BTreeSet::new();

        if let Some(implicit) = implicit_scopes {
            for bindings in implicit {
                let asset = bindings_asset(&bindings, inputs);
                let time = bindings_time(&bindings, entrypoint)?;
                if let Some(filter) = &constraint.generation_filter {
                    if !evaluate_constraint_filter(
                        filter,
                        constraint,
                        FilterScope {
                            bindings: &bindings,
                            asset,
                            time,
                        },
                        inputs,
                        entrypoint,
                    )? {
                        continue;
                    }
                }

                match compile_nonlinear_constraint_body(
                    constraint,
                    &bindings,
                    &binding_order,
                    program,
                    inputs,
                    named_expressions,
                    expression_generation_index,
                    variable_signatures,
                    instantiated_names,
                    entrypoint,
                ) {
                    Ok(rows) => constraints.extend(rows),
                    Err(CompileError::EmptyTupleReduction { .. }) => {
                        empty_subset_keys.insert(constraint_scope_key(&bindings, &binding_order));
                    }
                    Err(error) => return Err(error),
                }
            }
        } else {
            for scope in explicit_constraint_scopes(
                explicit_scopes,
                &constraint.diagnostic_id,
                entrypoint,
            )? {
                if let Some(filter) = &constraint.generation_filter {
                    match evaluate_reduction_filter(
                        filter,
                        &scope,
                        program,
                        inputs,
                        named_expressions,
                        expression_generation_index,
                        variable_signatures,
                        instantiated_names,
                        entrypoint,
                    ) {
                        Ok(false) => continue,
                        Ok(true) => {}
                        Err(CompileError::EmptyTupleReduction { .. }) => {
                            empty_subset_keys.insert(constraint_scope_key(&scope, &binding_order));
                            continue;
                        }
                        Err(error) => return Err(error),
                    }
                }

                match compile_nonlinear_constraint_body(
                    constraint,
                    &scope,
                    &binding_order,
                    program,
                    inputs,
                    named_expressions,
                    expression_generation_index,
                    variable_signatures,
                    instantiated_names,
                    entrypoint,
                ) {
                    Ok(rows) => constraints.extend(rows),
                    Err(CompileError::EmptyTupleReduction { .. }) => {
                        empty_subset_keys.insert(constraint_scope_key(&scope, &binding_order));
                    }
                    Err(error) => return Err(error),
                }
            }
        }

        if !empty_subset_keys.is_empty() {
            return Err(CompileError::InvalidFormulation {
                message: format!(
                    "empty constraint-relevant tuple subset for `{}` at keys: {}",
                    constraint.diagnostic_id,
                    empty_subset_keys.into_iter().collect::<Vec<_>>().join("; ")
                ),
                path: entrypoint.to_path_buf(),
            });
        }
    }

    Ok(constraints)
}

#[allow(clippy::too_many_arguments)]
fn compile_nonlinear_constraint_body(
    constraint: &ResolvedConstraint,
    bindings: &LinearizationBindings,
    binding_order: &[String],
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    named_expressions: &BTreeMap<String, Expr>,
    expression_generation_index: &ExpressionGenerationIndex,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    instantiated_names: &BTreeSet<String>,
    entrypoint: &Path,
) -> Result<Vec<NonlinearConstraint>, CompileError> {
    let suffix = constraint_binding_suffix(bindings, binding_order);
    match &constraint.expression {
        ConstraintBody::Comparison { op, left, right } => Ok(vec![compile_nonlinear_comparison(
            format!("{}{}", constraint.name, suffix),
            *op,
            left,
            right,
            bindings,
            program,
            inputs,
            named_expressions,
            expression_generation_index,
            variable_signatures,
            instantiated_names,
            entrypoint,
        )?]),
        ConstraintBody::Range {
            lower,
            lower_op,
            middle,
            upper_op,
            upper,
        } => Ok(vec![
            compile_nonlinear_comparison(
                format!("{}{}_lower", constraint.name, suffix),
                *lower_op,
                lower,
                middle,
                bindings,
                program,
                inputs,
                named_expressions,
                expression_generation_index,
                variable_signatures,
                instantiated_names,
                entrypoint,
            )?,
            compile_nonlinear_comparison(
                format!("{}{}_upper", constraint.name, suffix),
                *upper_op,
                middle,
                upper,
                bindings,
                program,
                inputs,
                named_expressions,
                expression_generation_index,
                variable_signatures,
                instantiated_names,
                entrypoint,
            )?,
        ]),
    }
}

#[allow(clippy::too_many_arguments)]
fn compile_nonlinear_comparison(
    name: String,
    op: ComparisonOp,
    left: &Expr,
    right: &Expr,
    bindings: &LinearizationBindings,
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    named_expressions: &BTreeMap<String, Expr>,
    expression_generation_index: &ExpressionGenerationIndex,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    instantiated_names: &BTreeSet<String>,
    entrypoint: &Path,
) -> Result<NonlinearConstraint, CompileError> {
    let left_expr = compile_nonlinear_expr(
        left,
        bindings,
        program,
        inputs,
        named_expressions,
        expression_generation_index,
        variable_signatures,
        instantiated_names,
        entrypoint,
    )?;
    let right_expr = compile_nonlinear_expr(
        right,
        bindings,
        program,
        inputs,
        named_expressions,
        expression_generation_index,
        variable_signatures,
        instantiated_names,
        entrypoint,
    )?;
    let sense = comparison_to_constraint_sense(op, entrypoint)?;
    Ok(NonlinearConstraint {
        name,
        sense,
        rhs: 0.0,
        expression: NonlinearExpr::Binary {
            op: BinaryOp::Subtract,
            left: Box::new(left_expr),
            right: Box::new(right_expr),
        },
    })
}

#[allow(clippy::too_many_arguments)]
fn compile_nonlinear_expr(
    expr: &Expr,
    bindings: &LinearizationBindings,
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    named_expressions: &BTreeMap<String, Expr>,
    expression_generation_index: &ExpressionGenerationIndex,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    instantiated_names: &BTreeSet<String>,
    entrypoint: &Path,
) -> Result<NonlinearExpr, CompileError> {
    match expr {
        Expr::Number(value) => value
            .parse::<f64>()
            .map(NonlinearExpr::Constant)
            .map_err(|_| CompileError::InvalidFormulation {
                message: format!("invalid numeric literal `{value}`"),
                path: entrypoint.to_path_buf(),
            }),
        Expr::Identifier(name) => {
            if let Some(binding) = bindings.values.get(name) {
                let numeric =
                    numeric_filter_value(binding, &synthetic_constraint(name), entrypoint)?;
                return Ok(NonlinearExpr::Constant(numeric));
            }
            if let Some(expression) = named_expressions.get(name) {
                if let Some(generation_bindings) =
                    expression_generation_bindings(name, program, expression_generation_index)
                {
                    if !generation_bindings.is_empty() {
                        return Err(CompileError::InvalidFormulation {
                            message: format!(
                                "indexed expression `{name}` expects {} index value(s), received 0",
                                generation_bindings.len()
                            ),
                            path: entrypoint.to_path_buf(),
                        });
                    }
                }
                return compile_nonlinear_expr(
                    expression,
                    bindings,
                    program,
                    inputs,
                    named_expressions,
                    expression_generation_index,
                    variable_signatures,
                    instantiated_names,
                    entrypoint,
                );
            }
            if find_variable_family(name, 0, variable_signatures).is_some()
                && instantiated_names.contains(name)
            {
                return Ok(NonlinearExpr::Variable(name.clone()));
            }
            Err(CompileError::InvalidFormulation {
                message: format!("unresolved symbol `{name}` in nonlinear expression"),
                path: entrypoint.to_path_buf(),
            })
        }
        Expr::Indexed { target, indices } => compile_nonlinear_indexed_expr(
            target,
            indices,
            bindings,
            program,
            inputs,
            named_expressions,
            expression_generation_index,
            variable_signatures,
            instantiated_names,
            entrypoint,
        ),
        Expr::Unary { op, expr } => Ok(NonlinearExpr::Unary {
            op: *op,
            expr: Box::new(compile_nonlinear_expr(
                expr,
                bindings,
                program,
                inputs,
                named_expressions,
                expression_generation_index,
                variable_signatures,
                instantiated_names,
                entrypoint,
            )?),
        }),
        Expr::Binary { op, left, right } => Ok(NonlinearExpr::Binary {
            op: *op,
            left: Box::new(compile_nonlinear_expr(
                left,
                bindings,
                program,
                inputs,
                named_expressions,
                expression_generation_index,
                variable_signatures,
                instantiated_names,
                entrypoint,
            )?),
            right: Box::new(compile_nonlinear_expr(
                right,
                bindings,
                program,
                inputs,
                named_expressions,
                expression_generation_index,
                variable_signatures,
                instantiated_names,
                entrypoint,
            )?),
        }),
        Expr::FunctionCall { name, args } => Ok(NonlinearExpr::FunctionCall {
            name: name.clone(),
            args: args
                .iter()
                .map(|arg| {
                    compile_nonlinear_expr(
                        arg,
                        bindings,
                        program,
                        inputs,
                        named_expressions,
                        expression_generation_index,
                        variable_signatures,
                        instantiated_names,
                        entrypoint,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        }),
        Expr::Reduction(reduction) => compile_nonlinear_reduction(
            reduction,
            bindings,
            program,
            inputs,
            named_expressions,
            expression_generation_index,
            variable_signatures,
            instantiated_names,
            entrypoint,
        ),
        Expr::String(_) | Expr::Boolean(_) | Expr::Comparison { .. } => {
            Err(CompileError::InvalidFormulation {
                message: "boolean and string expressions cannot appear in nonlinear algebra"
                    .to_string(),
                path: entrypoint.to_path_buf(),
            })
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn compile_nonlinear_reduction(
    reduction: &arco_kdl::algebra::ReductionExpr,
    bindings: &LinearizationBindings,
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    named_expressions: &BTreeMap<String, Expr>,
    expression_generation_index: &ExpressionGenerationIndex,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    instantiated_names: &BTreeSet<String>,
    entrypoint: &Path,
) -> Result<NonlinearExpr, CompileError> {
    let tuple_reduction_domain = tuple_reduction_domain_name(&reduction.bindings, program);
    let mut total = NonlinearExpr::Constant(0.0);
    let mut matched_scope_count = 0usize;

    for_each_reduction_scope(
        &reduction.bindings,
        bindings,
        inputs,
        program,
        entrypoint,
        |scope| {
            for filter in &reduction.filters {
                if !evaluate_reduction_filter(
                    filter,
                    scope,
                    program,
                    inputs,
                    named_expressions,
                    expression_generation_index,
                    variable_signatures,
                    instantiated_names,
                    entrypoint,
                )? {
                    return Ok(());
                }
            }

            matched_scope_count += 1;
            let value = compile_nonlinear_expr(
                &reduction.body,
                scope,
                program,
                inputs,
                named_expressions,
                expression_generation_index,
                variable_signatures,
                instantiated_names,
                entrypoint,
            )?;
            let previous_total = std::mem::replace(&mut total, NonlinearExpr::Constant(0.0));
            total = NonlinearExpr::Binary {
                op: BinaryOp::Add,
                left: Box::new(previous_total),
                right: Box::new(value),
            };
            Ok(())
        },
    )?;

    if matched_scope_count == 0 {
        if let Some(domain) = tuple_reduction_domain {
            return Err(CompileError::EmptyTupleReduction {
                domain,
                path: entrypoint.to_path_buf(),
            });
        }
    }

    Ok(total)
}

#[allow(clippy::too_many_arguments)]
fn compile_nonlinear_indexed_expr(
    target: &str,
    indices: &[Expr],
    bindings: &LinearizationBindings,
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    named_expressions: &BTreeMap<String, Expr>,
    expression_generation_index: &ExpressionGenerationIndex,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    instantiated_names: &BTreeSet<String>,
    entrypoint: &Path,
) -> Result<NonlinearExpr, CompileError> {
    let resolved = match (
        indices.len(),
        resolve_tuple_key_index(indices, bindings, program),
    ) {
        (1, Some(tuple_key_values)) => tuple_key_values,
        _ => resolve_index_values(indices, bindings, named_expressions, entrypoint)?,
    };

    let candidate = candidate_instance_name(target, &resolved, entrypoint)?;
    if instantiated_names.contains(&candidate) {
        return Ok(NonlinearExpr::Variable(candidate));
    }

    if let Some(expression) = named_expressions.get(target) {
        let generation_bindings =
            expression_generation_bindings(target, program, expression_generation_index);
        let generation_filter =
            expression_generation_filter(target, program, expression_generation_index);
        let requires_scoped_bindings = generation_filter.is_some()
            || generation_bindings.is_some_and(|bindings| !bindings.is_empty());

        if !requires_scoped_bindings {
            return compile_nonlinear_expr(
                expression,
                bindings,
                program,
                inputs,
                named_expressions,
                expression_generation_index,
                variable_signatures,
                instantiated_names,
                entrypoint,
            );
        }

        let mut scoped_bindings = bindings.clone();
        if let Some(generation_bindings) = generation_bindings {
            bind_generated_expression_indices(
                target,
                generation_bindings,
                &resolved,
                &mut scoped_bindings,
                entrypoint,
            )?;
        }

        if let Some(filter) = generation_filter {
            if !evaluate_reduction_filter(
                filter,
                &scoped_bindings,
                program,
                inputs,
                named_expressions,
                expression_generation_index,
                variable_signatures,
                instantiated_names,
                entrypoint,
            )? {
                return Ok(NonlinearExpr::Constant(0.0));
            }
        }

        return compile_nonlinear_expr(
            expression,
            &scoped_bindings,
            program,
            inputs,
            named_expressions,
            expression_generation_index,
            variable_signatures,
            instantiated_names,
            entrypoint,
        );
    }

    if let [FilterValue::String(_), FilterValue::Number(_)] = resolved.as_slice() {
        let synthetic = synthetic_constraint(target);
        let asset_name = string_filter_value(&resolved[0], &synthetic, entrypoint)?;
        let time = integer_time_index(&resolved[1], entrypoint)?;

        if !(1..=program.time_steps() as i64).contains(&time)
            && find_variable_family(target, resolved.len(), variable_signatures).is_some()
        {
            if let Some(value) =
                chronology_boundary_value(target, &asset_name, time, program, inputs, entrypoint)?
            {
                return Ok(NonlinearExpr::Constant(value));
            }
            return Err(CompileError::InvalidFormulation {
                message: format!("time index `{time}` is out of range for `{target}`"),
                path: entrypoint.to_path_buf(),
            });
        }
    }

    if let [FilterValue::Number(_)] = resolved.as_slice() {
        let time = integer_time_index(&resolved[0], entrypoint)?;
        if !(1..=program.time_steps() as i64).contains(&time)
            && find_variable_family(target, resolved.len(), variable_signatures).is_some()
        {
            return Err(CompileError::InvalidFormulation {
                message: format!("time index `{time}` is out of range for `{target}`"),
                path: entrypoint.to_path_buf(),
            });
        }
    }

    if !parameter_name_known(target, program, inputs) {
        return Err(CompileError::MissingDeclaration {
            kind: "parameter",
            name: target.to_string(),
            path: entrypoint.to_path_buf(),
        });
    }

    let parameter_expr = parameter_reference_expr(target, &resolved, inputs, entrypoint)?;
    if parameter_expr.terms.is_empty() {
        Ok(NonlinearExpr::Constant(parameter_expr.constant))
    } else {
        Err(CompileError::InvalidFormulation {
            message: format!(
                "parameter reference `{target}` unexpectedly resolved to non-scalar expression"
            ),
            path: entrypoint.to_path_buf(),
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_terminal_boundary_nonlinear_constraints(
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    named_expressions: &BTreeMap<String, Expr>,
    expression_generation_index: &ExpressionGenerationIndex,
    instantiated_names: &BTreeSet<String>,
    entrypoint: &Path,
) -> Result<Vec<NonlinearConstraint>, CompileError> {
    let linear_rows =
        emit_terminal_boundary_constraints(program, inputs, variable_signatures, entrypoint)?;
    linear_rows
        .into_iter()
        .map(|row| {
            let mut expr = NonlinearExpr::Constant(-row.rhs);
            for term in row.terms {
                let linear_term = NonlinearExpr::Binary {
                    op: BinaryOp::Multiply,
                    left: Box::new(NonlinearExpr::Constant(term.coefficient)),
                    right: Box::new(NonlinearExpr::Variable(term.variable_name)),
                };
                expr = NonlinearExpr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(expr),
                    right: Box::new(linear_term),
                };
            }
            Ok(NonlinearConstraint {
                name: row.name,
                sense: row.sense,
                rhs: 0.0,
                expression: expr,
            })
        })
        .collect::<Result<Vec<_>, CompileError>>()
        .and_then(|rows| {
            for report in &program.active_reports {
                // Touch report formulas so unresolved references in reports are
                // reported consistently even when no linear reports are emitted.
                let _ = compile_nonlinear_expr(
                    &report.formula,
                    &LinearizationBindings::default(),
                    program,
                    inputs,
                    named_expressions,
                    expression_generation_index,
                    variable_signatures,
                    instantiated_names,
                    entrypoint,
                )?;
            }
            Ok(rows)
        })
}
