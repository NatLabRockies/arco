fn generic_data_value(
    generic_data: &BTreeMap<String, GenericDataTable>,
    target: &str,
    resolved: &[FilterValue],
    entrypoint: &Path,
) -> Result<Option<f64>, CompileError> {
    let Some(table) = generic_data.get(target) else {
        return Ok(None);
    };

    let key = resolved
        .iter()
        .map(|value| filter_value_to_key_component(value, entrypoint))
        .collect::<Result<Vec<_>, _>>()?;

    if let Some(value) = table.values.get(&key).copied() {
        return Ok(Some(value));
    }

    for prefix_len in (1..key.len()).rev() {
        if let Some(value) = table.values.get(&key[..prefix_len]).copied() {
            return Ok(Some(value));
        }
    }

    Ok(table.default_missing)
}

fn format_filter_lookup_key(
    resolved: &[FilterValue],
    entrypoint: &Path,
) -> Result<String, CompileError> {
    let parts = resolved
        .iter()
        .map(|value| filter_value_to_key_component(value, entrypoint))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(parts.join(","))
}

fn filter_value_to_key_component(
    value: &FilterValue,
    entrypoint: &Path,
) -> Result<String, CompileError> {
    match value {
        FilterValue::String(value) => Ok(value.clone()),
        FilterValue::Number(value) => {
            if value.fract() == 0.0 {
                Ok((*value as i64).to_string())
            } else {
                Ok(value.to_string())
            }
        }
        FilterValue::Boolean(_) => Err(CompileError::InvalidFormulation {
            message: "boolean indices are not supported for data lookups".to_string(),
            path: entrypoint.to_path_buf(),
        }),
    }
}

/// Convert a `BoundExpr` to a constant `f64` when possible. Returns `None`
/// for `BoundExpr::Formula` expressions that depend on identifiers, indexed
/// parameters, or reductions (which need an instance binding context that is
/// not available here).
fn literal_bound_to_f64(bound: &BoundExpr, path: &Path) -> Result<Option<f64>, CompileError> {
    match bound {
        BoundExpr::Literal(literal) => literal_to_f64("bound", literal, path).map(Some),
        BoundExpr::Formula(expr) => Ok(evaluate_constant_expr(expr)),
    }
}

/// Evaluate an `Expr` as a pure numeric constant (no free identifiers,
/// indexed lookups, reductions, or comparisons). Returns `None` if any
/// non-constant subexpression is encountered.
fn evaluate_constant_expr(expr: &crate::algebra::Expr) -> Option<f64> {
    use crate::algebra::{BinaryOp, Expr, UnaryOp};
    match expr {
        Expr::Number(value) => value.parse::<f64>().ok(),
        Expr::Boolean(value) => Some(if *value { 1.0 } else { 0.0 }),
        Expr::Unary { op, expr } => {
            let value = evaluate_constant_expr(expr)?;
            match op {
                UnaryOp::Negate => Some(-value),
            }
        }
        Expr::Binary { op, left, right } => {
            let left_value = evaluate_constant_expr(left)?;
            let right_value = evaluate_constant_expr(right)?;
            match op {
                BinaryOp::Add => Some(left_value + right_value),
                BinaryOp::Subtract => Some(left_value - right_value),
                BinaryOp::Multiply => Some(left_value * right_value),
                BinaryOp::Divide => {
                    if right_value == 0.0 {
                        None
                    } else {
                        Some(left_value / right_value)
                    }
                }
            }
        }
        Expr::FunctionCall { name, args } => {
            let evaluated = args
                .iter()
                .map(evaluate_constant_expr)
                .collect::<Option<Vec<_>>>()?;
            match (name.as_str(), evaluated.as_slice()) {
                ("sqrt", [x]) => Some(x.sqrt()),
                ("abs", [x]) => Some(x.abs()),
                ("exp", [x]) => Some(x.exp()),
                ("ln", [x]) => Some(x.ln()),
                ("sin", [x]) => Some(x.sin()),
                ("cos", [x]) => Some(x.cos()),
                ("atan", [x]) => Some(x.atan()),
                ("pow", [base, exponent]) => Some(base.powf(*exponent)),
                _ => None,
            }
        }
        Expr::String(_)
        | Expr::Identifier(_)
        | Expr::Indexed { .. }
        | Expr::Comparison { .. }
        | Expr::Reduction(_) => None,
    }
}

fn literal_to_f64(name: &str, value: &LiteralValue, path: &Path) -> Result<f64, CompileError> {
    match value {
        LiteralValue::Integer(value) => Ok(*value as f64),
        LiteralValue::Decimal(value) | LiteralValue::String(value) => {
            value
                .parse::<f64>()
                .map_err(|_| CompileError::InvalidNumber {
                    value: value.clone(),
                    field: name.to_string(),
                    path: path.to_path_buf(),
                })
        }
        LiteralValue::Boolean(value) => Ok(if *value { 1.0 } else { 0.0 }),
    }
}

fn asset_parameter(asset: &AssetInputs, name: &str, path: &Path) -> Result<f64, CompileError> {
    asset
        .parameters
        .get(name)
        .copied()
        .ok_or_else(|| CompileError::MissingParameter {
            name: name.to_string(),
            asset: asset.name.clone(),
            path: path.to_path_buf(),
        })
}

fn has_asset_parameter(asset: &AssetInputs, name: &str) -> bool {
    asset.parameters.contains_key(name)
}

fn series_value(
    series: &BTreeMap<String, BTreeMap<usize, f64>>,
    name: &str,
    time: usize,
    path: &Path,
) -> Result<f64, CompileError> {
    series
        .get(name)
        .ok_or_else(|| CompileError::MissingData {
            name: name.to_string(),
            path: path.to_path_buf(),
        })?
        .get(&time)
        .copied()
        .ok_or_else(|| CompileError::MissingDataPoint {
            name: name.to_string(),
            key: time.to_string(),
            path: path.to_path_buf(),
        })
}

fn indexed_value(
    indexed: &BTreeMap<String, BTreeMap<(String, usize), f64>>,
    name: &str,
    asset_name: &str,
    time: usize,
    path: &Path,
) -> Result<f64, CompileError> {
    indexed
        .get(name)
        .ok_or_else(|| CompileError::MissingData {
            name: name.to_string(),
            path: path.to_path_buf(),
        })?
        .get(&(asset_name.to_string(), time))
        .copied()
        .ok_or_else(|| CompileError::MissingDataPoint {
            name: name.to_string(),
            key: format!("{asset_name},{time}"),
            path: path.to_path_buf(),
        })
}

fn asset_data_value(
    asset_data: &BTreeMap<String, BTreeMap<String, f64>>,
    name: &str,
    asset_name: &str,
    path: &Path,
) -> Result<f64, CompileError> {
    asset_data
        .get(name)
        .ok_or_else(|| CompileError::MissingData {
            name: name.to_string(),
            path: path.to_path_buf(),
        })?
        .get(asset_name)
        .copied()
        .ok_or_else(|| CompileError::MissingDataPoint {
            name: name.to_string(),
            key: asset_name.to_string(),
            path: path.to_path_buf(),
        })
}
