fn coerce_numeric_filter_value(value: FilterValue) -> FilterValue {
    match value {
        FilterValue::String(text) => text
            .parse::<f64>()
            .map(FilterValue::Number)
            .unwrap_or(FilterValue::String(text)),
        other => other,
    }
}

fn integer_time_index(value: &FilterValue, entrypoint: &Path) -> Result<i64, CompileError> {
    match value {
        FilterValue::Number(number) => {
            if number.fract() == 0.0 {
                Ok(*number as i64)
            } else {
                Err(CompileError::InvalidFormulation {
                    message: format!("time index `{number}` must be integral"),
                    path: entrypoint.to_path_buf(),
                })
            }
        }
        FilterValue::String(value) => {
            value
                .parse::<i64>()
                .map_err(|_| CompileError::InvalidFormulation {
                    message: format!("time index `{value}` must be integral"),
                    path: entrypoint.to_path_buf(),
                })
        }
        FilterValue::Boolean(value) => Err(CompileError::InvalidFormulation {
            message: format!("time index `{value}` must be numeric"),
            path: entrypoint.to_path_buf(),
        }),
    }
}

fn resolve_index_expr(
    expr: &Expr,
    bindings: &LinearizationBindings,
    entrypoint: &Path,
) -> Result<FilterValue, CompileError> {
    match expr {
        Expr::Identifier(name) => bindings
            .values
            .get(name)
            .cloned()
            .map(coerce_numeric_filter_value)
            .ok_or_else(|| CompileError::InvalidFormulation {
                message: format!("unbound index identifier `{name}`"),
                path: entrypoint.to_path_buf(),
            }),
        Expr::Number(value) => value.parse::<f64>().map(FilterValue::Number).map_err(|_| {
            CompileError::InvalidFormulation {
                message: format!("invalid numeric index `{value}`"),
                path: entrypoint.to_path_buf(),
            }
        }),
        Expr::String(value) => Ok(FilterValue::String(value.clone())),
        Expr::Unary { op, expr } => match op {
            UnaryOp::Negate => Ok(FilterValue::Number(-numeric_filter_value(
                &resolve_index_expr(expr, bindings, entrypoint)?,
                &synthetic_constraint("index"),
                entrypoint,
            )?)),
        },
        Expr::Binary { op, left, right } => {
            let left = resolve_index_expr(left, bindings, entrypoint)?;
            let right = resolve_index_expr(right, bindings, entrypoint)?;
            let left = numeric_filter_value(&left, &synthetic_constraint("index"), entrypoint)?;
            let right = numeric_filter_value(&right, &synthetic_constraint("index"), entrypoint)?;
            Ok(FilterValue::Number(match op {
                BinaryOp::Add => left + right,
                BinaryOp::Subtract => left - right,
                BinaryOp::Multiply => left * right,
                BinaryOp::Divide => left / right,
            }))
        }
        _ => Err(CompileError::InvalidFormulation {
            message: "unsupported index expression during compilation".to_string(),
            path: entrypoint.to_path_buf(),
        }),
    }
}

fn synthetic_constraint(name: &str) -> ResolvedConstraint {
    ResolvedConstraint {
        name: name.to_string(),
        source_kind: "compile".to_string(),
        source_name: "synthetic".to_string(),
        expression_text: String::new(),
        expression: ConstraintBody::Comparison {
            op: ComparisonOp::Equal,
            left: Expr::Number("0".to_string()),
            right: Expr::Number("0".to_string()),
        },
        generation_bindings: Vec::new(),
        generation_filter_text: None,
        generation_filter: None,
    }
}
