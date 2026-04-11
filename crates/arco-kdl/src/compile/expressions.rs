fn linearize_value_expr(
    expr: &Expr,
    bindings: &LinearizationBindings,
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    named_expressions: &BTreeMap<String, Expr>,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    instantiated_names: &BTreeSet<String>,
    entrypoint: &Path,
) -> Result<AffineExpr, CompileError> {
    match expr {
        Expr::Number(value) => value.parse::<f64>().map(AffineExpr::constant).map_err(|_| {
            CompileError::InvalidFormulation {
                message: format!("invalid numeric literal `{value}`"),
                path: entrypoint.to_path_buf(),
            }
        }),
        Expr::Identifier(name) => {
            if let Some(binding) = bindings.values.get(name) {
                return Ok(AffineExpr::constant(numeric_filter_value(
                    binding,
                    &synthetic_constraint(name),
                    entrypoint,
                )?));
            }
            if let Some(expression) = named_expressions.get(name) {
                return linearize_value_expr(
                    expression,
                    bindings,
                    program,
                    inputs,
                    named_expressions,
                    variable_signatures,
                    instantiated_names,
                    entrypoint,
                );
            }
            Err(CompileError::InvalidFormulation {
                message: format!("unresolved symbol `{name}` in linear expression"),
                path: entrypoint.to_path_buf(),
            })
        }
        Expr::Indexed { target, indices } => linearize_indexed_expr(
            target,
            indices,
            bindings,
            program,
            inputs,
            variable_signatures,
            instantiated_names,
            entrypoint,
        ),
        Expr::Unary { op, expr } => {
            let value = linearize_value_expr(
                expr,
                bindings,
                program,
                inputs,
                named_expressions,
                variable_signatures,
                instantiated_names,
                entrypoint,
            )?;
            match op {
                UnaryOp::Negate => Ok(value.scale(-1.0)),
            }
        }
        Expr::Binary { op, left, right } => {
            let left = linearize_value_expr(
                left,
                bindings,
                program,
                inputs,
                named_expressions,
                variable_signatures,
                instantiated_names,
                entrypoint,
            )?;
            let right = linearize_value_expr(
                right,
                bindings,
                program,
                inputs,
                named_expressions,
                variable_signatures,
                instantiated_names,
                entrypoint,
            )?;
            match op {
                BinaryOp::Add => {
                    let mut value = left;
                    value.add_assign(right);
                    Ok(value)
                }
                BinaryOp::Subtract => Ok(left.subtract(right)),
                BinaryOp::Multiply => {
                    if left.terms.is_empty() {
                        Ok(right.scale(left.constant))
                    } else if right.terms.is_empty() {
                        Ok(left.scale(right.constant))
                    } else {
                        Err(CompileError::InvalidFormulation {
                            message: "non-linear multiplication is not supported".to_string(),
                            path: entrypoint.to_path_buf(),
                        })
                    }
                }
                BinaryOp::Divide => {
                    let denominator = right.as_scalar(entrypoint, "division denominator")?;
                    Ok(left.scale(1.0 / denominator))
                }
            }
        }
        Expr::FunctionCall { name, args } => {
            let evaluated_args = args
                .iter()
                .map(|arg| {
                    let result = linearize_value_expr(
                        arg,
                        bindings,
                        program,
                        inputs,
                        named_expressions,
                        variable_signatures,
                        instantiated_names,
                        entrypoint,
                    )?;
                    result.as_scalar(entrypoint, &format!("{name}() argument"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let value = evaluate_builtin_function(name, &evaluated_args, entrypoint)?;
            Ok(AffineExpr::constant(value))
        }
        Expr::Reduction(reduction) => linearize_reduction(
            reduction,
            bindings,
            program,
            inputs,
            named_expressions,
            variable_signatures,
            instantiated_names,
            entrypoint,
        ),
        Expr::String(_) | Expr::Boolean(_) | Expr::Comparison { .. } => {
            Err(CompileError::InvalidFormulation {
                message: "boolean and string expressions cannot appear in linear algebra"
                    .to_string(),
                path: entrypoint.to_path_buf(),
            })
        }
    }
}

fn evaluate_builtin_function(
    name: &str,
    args: &[f64],
    entrypoint: &Path,
) -> Result<f64, CompileError> {
    match (name, args.len()) {
        ("sqrt", 1) => Ok(args[0].sqrt()),
        ("abs", 1) => Ok(args[0].abs()),
        ("exp", 1) => Ok(args[0].exp()),
        ("ln", 1) => Ok(args[0].ln()),
        ("pow", 2) => Ok(args[0].powf(args[1])),
        (name, n) => Err(CompileError::InvalidFormulation {
            message: format!(
                "{name}() received {n} argument(s), expected {}",
                if name == "pow" { 2 } else { 1 }
            ),
            path: entrypoint.to_path_buf(),
        }),
    }
}

#[allow(clippy::too_many_arguments)]
fn linearize_reduction(
    reduction: &crate::algebra::ReductionExpr,
    bindings: &LinearizationBindings,
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    named_expressions: &BTreeMap<String, Expr>,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    instantiated_names: &BTreeSet<String>,
    entrypoint: &Path,
) -> Result<AffineExpr, CompileError> {
    let expanded =
        expand_reduction_bindings(&reduction.bindings, bindings, inputs, program, entrypoint)?;
    let mut total = AffineExpr::default();
    'outer: for scope in expanded {
        for filter in &reduction.filters {
            if !evaluate_reduction_filter(
                filter,
                &scope,
                program,
                inputs,
                named_expressions,
                variable_signatures,
                instantiated_names,
                entrypoint,
            )? {
                continue 'outer;
            }
        }
        total.add_assign(linearize_value_expr(
            &reduction.body,
            &scope,
            program,
            inputs,
            named_expressions,
            variable_signatures,
            instantiated_names,
            entrypoint,
        )?);
    }
    Ok(total)
}

#[allow(clippy::too_many_arguments)]
fn evaluate_reduction_filter(
    filter: &Expr,
    bindings: &LinearizationBindings,
    program: &SemanticProgram,
    inputs: &ScenarioInputs,
    named_expressions: &BTreeMap<String, Expr>,
    variable_signatures: &BTreeMap<String, FamilySignature>,
    instantiated_names: &BTreeSet<String>,
    entrypoint: &Path,
) -> Result<bool, CompileError> {
    if let Expr::Comparison { op, left, right } = filter {
        let left_affine = linearize_value_expr(
            left,
            bindings,
            program,
            inputs,
            named_expressions,
            variable_signatures,
            instantiated_names,
            entrypoint,
        )?;
        let right_affine = linearize_value_expr(
            right,
            bindings,
            program,
            inputs,
            named_expressions,
            variable_signatures,
            instantiated_names,
            entrypoint,
        )?;
        let left_value = left_affine.as_scalar(entrypoint, "reduction filter operand")?;
        let right_value = right_affine.as_scalar(entrypoint, "reduction filter operand")?;
        Ok(match op {
            ComparisonOp::Equal | ComparisonOp::DoubleEqual => {
                (left_value - right_value).abs() < 1e-12
            }
            ComparisonOp::NotEqual => (left_value - right_value).abs() >= 1e-12,
            ComparisonOp::Less => left_value < right_value,
            ComparisonOp::LessEqual => left_value <= right_value,
            ComparisonOp::Greater => left_value > right_value,
            ComparisonOp::GreaterEqual => left_value >= right_value,
        })
    } else {
        let value = linearize_value_expr(
            filter,
            bindings,
            program,
            inputs,
            named_expressions,
            variable_signatures,
            instantiated_names,
            entrypoint,
        )?
        .as_scalar(entrypoint, "reduction filter expression")?;
        Ok(value.abs() >= 1e-12)
    }
}

fn expand_reduction_bindings(
    bindings: &[crate::algebra::Binding],
    current: &LinearizationBindings,
    inputs: &ScenarioInputs,
    program: &SemanticProgram,
    entrypoint: &Path,
) -> Result<Vec<LinearizationBindings>, CompileError> {
    let mut scopes = vec![current.clone()];
    for binding in bindings {
        let values = reduction_domain_values(&binding.domain, inputs, program, entrypoint)?;
        let mut next = Vec::new();
        for scope in &scopes {
            match &binding.pattern {
                crate::algebra::BindingPattern::Name(name) => {
                    for value in &values {
                        let mut scope = scope.clone();
                        scope.values.insert(name.clone(), value.clone());
                        next.push(scope);
                    }
                }
                crate::algebra::BindingPattern::Tuple(_) => {
                    return Err(CompileError::InvalidFormulation {
                        message: "tuple reduction bindings are not lowered yet".to_string(),
                        path: entrypoint.to_path_buf(),
                    });
                }
            }
        }
        scopes = next;
    }
    Ok(scopes)
}
