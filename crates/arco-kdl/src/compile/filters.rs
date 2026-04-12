fn evaluate_constraint_filter(
    expr: &Expr,
    constraint: &ResolvedConstraint,
    scope: FilterScope<'_>,
    inputs: &ScenarioInputs,
    path: &Path,
) -> Result<bool, CompileError> {
    let value = evaluate_filter_expr(expr, constraint, scope, inputs, path)?;
    truthy_filter_value(&value, constraint, path)
}

fn evaluate_filter_expr(
    expr: &Expr,
    constraint: &ResolvedConstraint,
    scope: FilterScope<'_>,
    inputs: &ScenarioInputs,
    path: &Path,
) -> Result<FilterValue, CompileError> {
    match expr {
        Expr::Number(value) => value
            .parse::<f64>()
            .map(FilterValue::Number)
            .map_err(|_| invalid_constraint_filter(constraint, path, "numeric literal is invalid")),
        Expr::String(value) => Ok(FilterValue::String(value.clone())),
        Expr::Boolean(value) => Ok(FilterValue::Boolean(*value)),
        Expr::Identifier(name) => evaluate_identifier(name, constraint, scope, path),
        Expr::Indexed { target, indices } => {
            evaluate_indexed_value(target, indices, constraint, scope, inputs, path)
        }
        Expr::Unary { op, expr } => {
            let value = evaluate_filter_expr(expr, constraint, scope, inputs, path)?;
            match op {
                UnaryOp::Negate => Ok(FilterValue::Number(-numeric_filter_value(
                    &value, constraint, path,
                )?)),
            }
        }
        Expr::Binary { op, left, right } => {
            let left = evaluate_filter_expr(left, constraint, scope, inputs, path)?;
            let right = evaluate_filter_expr(right, constraint, scope, inputs, path)?;
            let left = numeric_filter_value(&left, constraint, path)?;
            let right = numeric_filter_value(&right, constraint, path)?;
            let value = match op {
                BinaryOp::Add => left + right,
                BinaryOp::Subtract => left - right,
                BinaryOp::Multiply => left * right,
                BinaryOp::Divide => left / right,
            };
            Ok(FilterValue::Number(value))
        }
        Expr::Comparison { op, left, right } => {
            let left = evaluate_filter_expr(left, constraint, scope, inputs, path)?;
            let right = evaluate_filter_expr(right, constraint, scope, inputs, path)?;
            Ok(FilterValue::Boolean(compare_filter_values(
                *op, &left, &right, constraint, path,
            )?))
        }
        Expr::FunctionCall { .. } => Err(invalid_constraint_filter(
            constraint,
            path,
            "function calls are not supported in constraint filters",
        )),
        Expr::Reduction(_) => Err(invalid_constraint_filter(
            constraint,
            path,
            "reductions are not supported in constraint filters",
        )),
    }
}

fn evaluate_identifier(
    name: &str,
    constraint: &ResolvedConstraint,
    scope: FilterScope<'_>,
    path: &Path,
) -> Result<FilterValue, CompileError> {
    match name {
        "a" => scope
            .asset
            .map(|asset| FilterValue::String(asset.name.clone()))
            .ok_or_else(|| invalid_constraint_filter(constraint, path, "`a` is not in scope")),
        "t" => scope
            .time
            .map(|time| FilterValue::Number(time as f64))
            .ok_or_else(|| invalid_constraint_filter(constraint, path, "`t` is not in scope")),
        "candidate" => scope
            .asset
            .map(|asset| FilterValue::Boolean(asset.candidate))
            .ok_or_else(|| {
                invalid_constraint_filter(constraint, path, "`candidate` is not in scope")
            }),
        other => scope
            .asset
            .and_then(|asset| asset.parameters.get(other).copied())
            .map(FilterValue::Number)
            .ok_or_else(|| {
                invalid_constraint_filter(
                    constraint,
                    path,
                    format!("`{other}` is not available in the current filter scope"),
                )
            }),
    }
}

fn evaluate_indexed_value(
    target: &str,
    indices: &[Expr],
    constraint: &ResolvedConstraint,
    scope: FilterScope<'_>,
    inputs: &ScenarioInputs,
    path: &Path,
) -> Result<FilterValue, CompileError> {
    let values = indices
        .iter()
        .map(|index| evaluate_filter_expr(index, constraint, scope, inputs, path))
        .collect::<Result<Vec<_>, _>>()?;

    match values.as_slice() {
        [index] => {
            if let FilterValue::String(asset_name) = index {
                if target == "candidate" {
                    return find_asset(inputs, asset_name)
                        .map(|asset| FilterValue::Boolean(asset.candidate))
                        .ok_or_else(|| {
                            invalid_constraint_filter(
                                constraint,
                                path,
                                format!("asset `{asset_name}` is not available"),
                            )
                        });
                }
                if let Some(value) = asset_parameter_value(inputs, target, asset_name) {
                    return Ok(FilterValue::Number(value));
                }
                if let Some(value) = inputs
                    .asset_data
                    .get(target)
                    .and_then(|values| values.get(asset_name))
                    .copied()
                {
                    return Ok(FilterValue::Number(value));
                }
                Err(invalid_constraint_filter(
                    constraint,
                    path,
                    format!("`{target}[{asset_name}]` is not available"),
                ))
            } else {
                let time = usize_filter_value(index, constraint, path)?;
                inputs
                    .series
                    .get(target)
                    .and_then(|values| values.get(&time))
                    .copied()
                    .map(FilterValue::Number)
                    .ok_or_else(|| {
                        invalid_constraint_filter(
                            constraint,
                            path,
                            format!("`{target}[{time}]` is not available"),
                        )
                    })
            }
        }
        [asset_name, time] => {
            let asset_name = string_filter_value(asset_name, constraint, path)?;
            let time = usize_filter_value(time, constraint, path)?;
            inputs
                .indexed
                .get(target)
                .and_then(|values| values.get(&(asset_name.clone(), time)))
                .copied()
                .map(FilterValue::Number)
                .ok_or_else(|| {
                    invalid_constraint_filter(
                        constraint,
                        path,
                        format!("`{target}[{asset_name},{time}]` is not available"),
                    )
                })
        }
        _ => Err(invalid_constraint_filter(
            constraint,
            path,
            "constraint filters support only one-dimensional asset/time lookups and two-dimensional asset-time lookups",
        )),
    }
}

fn compare_filter_values(
    op: ComparisonOp,
    left: &FilterValue,
    right: &FilterValue,
    constraint: &ResolvedConstraint,
    path: &Path,
) -> Result<bool, CompileError> {
    match op {
        ComparisonOp::Equal | ComparisonOp::DoubleEqual => {
            compare_for_equality(left, right, constraint, path)
        }
        ComparisonOp::NotEqual => {
            compare_for_equality(left, right, constraint, path).map(|value| !value)
        }
        ComparisonOp::Less
        | ComparisonOp::LessEqual
        | ComparisonOp::Greater
        | ComparisonOp::GreaterEqual => {
            let left = numeric_filter_value(left, constraint, path)?;
            let right = numeric_filter_value(right, constraint, path)?;
            Ok(match op {
                ComparisonOp::Less => left < right,
                ComparisonOp::LessEqual => left <= right,
                ComparisonOp::Greater => left > right,
                ComparisonOp::GreaterEqual => left >= right,
                ComparisonOp::Equal | ComparisonOp::DoubleEqual | ComparisonOp::NotEqual => {
                    unreachable!()
                }
            })
        }
    }
}

fn compare_for_equality(
    left: &FilterValue,
    right: &FilterValue,
    constraint: &ResolvedConstraint,
    path: &Path,
) -> Result<bool, CompileError> {
    match (left, right) {
        (FilterValue::String(left), FilterValue::String(right)) => Ok(left == right),
        (FilterValue::Boolean(left), FilterValue::Boolean(right)) => Ok(left == right),
        _ => {
            let left = numeric_filter_value(left, constraint, path)?;
            let right = numeric_filter_value(right, constraint, path)?;
            Ok((left - right).abs() < f64::EPSILON)
        }
    }
}

fn truthy_filter_value(
    value: &FilterValue,
    constraint: &ResolvedConstraint,
    path: &Path,
) -> Result<bool, CompileError> {
    match value {
        FilterValue::Boolean(value) => Ok(*value),
        FilterValue::Number(value) => Ok(*value != 0.0),
        FilterValue::String(_) => Err(invalid_constraint_filter(
            constraint,
            path,
            "string-valued filters must be used inside an explicit comparison",
        )),
    }
}

fn numeric_filter_value(
    value: &FilterValue,
    constraint: &ResolvedConstraint,
    path: &Path,
) -> Result<f64, CompileError> {
    match value {
        FilterValue::Number(value) => Ok(*value),
        FilterValue::Boolean(value) => Ok(if *value { 1.0 } else { 0.0 }),
        FilterValue::String(_) => Err(invalid_constraint_filter(
            constraint,
            path,
            "numeric operations in constraint filters require numeric or boolean values",
        )),
    }
}

fn string_filter_value(
    value: &FilterValue,
    constraint: &ResolvedConstraint,
    path: &Path,
) -> Result<String, CompileError> {
    match value {
        FilterValue::String(value) => Ok(value.clone()),
        _ => Err(invalid_constraint_filter(
            constraint,
            path,
            "asset-indexed lookups require a string asset name",
        )),
    }
}

fn usize_filter_value(
    value: &FilterValue,
    constraint: &ResolvedConstraint,
    path: &Path,
) -> Result<usize, CompileError> {
    let number = numeric_filter_value(value, constraint, path)?;
    if number.fract() == 0.0 && number >= 0.0 {
        Ok(number as usize)
    } else {
        Err(invalid_constraint_filter(
            constraint,
            path,
            "time-indexed lookups require a non-negative integer time index",
        ))
    }
}

fn find_asset<'a>(inputs: &'a ScenarioInputs, name: &str) -> Option<&'a AssetInputs> {
    inputs.assets.iter().find(|asset| asset.name == name)
}

fn asset_parameter_value(
    inputs: &ScenarioInputs,
    parameter: &str,
    asset_name: &str,
) -> Option<f64> {
    find_asset(inputs, asset_name).and_then(|asset| asset.parameters.get(parameter).copied())
}

fn invalid_constraint_filter(
    constraint: &ResolvedConstraint,
    path: &Path,
    message: impl Into<String>,
) -> CompileError {
    CompileError::InvalidConstraintFilter {
        constraint: format!(
            "{}:{}:{}",
            constraint.source_kind, constraint.source_name, constraint.name
        ),
        message: message.into(),
        path: path.to_path_buf(),
    }
}
