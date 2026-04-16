fn resolve_data_column(data_decl: &DataDecl, logical_name: &str) -> String {
    data_decl
        .maps
        .iter()
        .find(|mapping| mapping.name == logical_name)
        .map_or_else(
            || logical_name.to_string(),
            |mapping| {
                mapping
                    .source
                    .clone()
                    .unwrap_or_else(|| mapping.name.clone())
            },
        )
}

fn matches_data_param_filter(
    row: &HashMap<String, String>,
    data_decl: &DataDecl,
    parameter: &ParamDecl,
) -> bool {
    let Some(expr) = parameter.parsed_filter_expression.as_ref() else {
        return true;
    };

    evaluate_data_param_filter_expr(expr, row, data_decl)
        .and_then(|value| truthy_data_param_filter_value(&value))
        .unwrap_or(false)
}

fn evaluate_data_param_filter_expr(
    expr: &Expr,
    row: &HashMap<String, String>,
    data_decl: &DataDecl,
) -> Option<FilterValue> {
    match expr {
        Expr::Number(value) => value.parse::<f64>().ok().map(FilterValue::Number),
        Expr::String(value) => Some(FilterValue::String(value.clone())),
        Expr::Boolean(value) => Some(FilterValue::Boolean(*value)),
        Expr::Identifier(name) => {
            let source_name = resolve_data_column(data_decl, name);
            row.get(name)
                .or_else(|| row.get(&source_name))
                .cloned()
                .map(FilterValue::String)
        }
        Expr::Unary { op, expr } => {
            let value = evaluate_data_param_filter_expr(expr, row, data_decl)?;
            match op {
                UnaryOp::Negate => {
                    data_param_filter_numeric_value(&value).map(|v| FilterValue::Number(-v))
                }
            }
        }
        Expr::Binary { op, left, right } => {
            let left = evaluate_data_param_filter_expr(left, row, data_decl)?;
            let right = evaluate_data_param_filter_expr(right, row, data_decl)?;
            let left = data_param_filter_numeric_value(&left)?;
            let right = data_param_filter_numeric_value(&right)?;
            Some(FilterValue::Number(match op {
                BinaryOp::Add => left + right,
                BinaryOp::Subtract => left - right,
                BinaryOp::Multiply => left * right,
                BinaryOp::Divide => left / right,
            }))
        }
        Expr::Comparison { op, left, right } => {
            let left = evaluate_data_param_filter_expr(left, row, data_decl)?;
            let right = evaluate_data_param_filter_expr(right, row, data_decl)?;
            compare_data_param_filter_values(*op, &left, &right).map(FilterValue::Boolean)
        }
        Expr::Indexed { .. } | Expr::FunctionCall { .. } | Expr::Reduction(_) => None,
    }
}

fn compare_data_param_filter_values(
    op: ComparisonOp,
    left: &FilterValue,
    right: &FilterValue,
) -> Option<bool> {
    match op {
        ComparisonOp::Equal | ComparisonOp::DoubleEqual => {
            compare_data_param_filter_for_equality(left, right)
        }
        ComparisonOp::NotEqual => compare_data_param_filter_for_equality(left, right).map(|v| !v),
        ComparisonOp::Less
        | ComparisonOp::LessEqual
        | ComparisonOp::Greater
        | ComparisonOp::GreaterEqual => {
            let left = data_param_filter_numeric_value(left)?;
            let right = data_param_filter_numeric_value(right)?;
            Some(match op {
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

fn compare_data_param_filter_for_equality(left: &FilterValue, right: &FilterValue) -> Option<bool> {
    match (left, right) {
        (FilterValue::String(left), FilterValue::String(right)) => Some(left == right),
        (FilterValue::Boolean(left), FilterValue::Boolean(right)) => Some(left == right),
        _ => {
            let left = data_param_filter_numeric_value(left)?;
            let right = data_param_filter_numeric_value(right)?;
            Some((left - right).abs() < f64::EPSILON)
        }
    }
}

fn truthy_data_param_filter_value(value: &FilterValue) -> Option<bool> {
    match value {
        FilterValue::Boolean(value) => Some(*value),
        FilterValue::Number(value) => Some(*value != 0.0),
        FilterValue::String(value) => {
            if value.eq_ignore_ascii_case("true") {
                Some(true)
            } else if value.eq_ignore_ascii_case("false") {
                Some(false)
            } else {
                value.parse::<f64>().ok().map(|number| number != 0.0)
            }
        }
    }
}

fn data_param_filter_numeric_value(value: &FilterValue) -> Option<f64> {
    match value {
        FilterValue::Number(value) => Some(*value),
        FilterValue::Boolean(value) => Some(if *value { 1.0 } else { 0.0 }),
        FilterValue::String(value) => {
            if value.eq_ignore_ascii_case("true") {
                Some(1.0)
            } else if value.eq_ignore_ascii_case("false") {
                Some(0.0)
            } else {
                value.parse::<f64>().ok()
            }
        }
    }
}

fn load_generic_data_table(
    program: &SemanticProgram,
    binding_name: &str,
    rows: &[HashMap<String, String>],
    path: &Path,
) -> Result<GenericDataTable, CompileError> {
    let Some(first_row) = rows.first() else {
        return Err(CompileError::MissingData {
            name: binding_name.to_string(),
            path: path.to_path_buf(),
        });
    };
    let mut headers = first_row.keys().cloned().collect::<Vec<_>>();
    headers.sort();
    let value_column = if headers.iter().any(|h| h == binding_name) {
        Some(binding_name.to_string())
    } else if headers.iter().any(|h| h == "value") {
        Some("value".to_string())
    } else {
        None
    };

    let key_columns = infer_data_key_columns(program, rows, &headers, value_column.as_deref());
    let mut values = BTreeMap::new();
    for row in rows {
        let key = key_columns
            .iter()
            .map(|column| {
                row.get(column)
                    .cloned()
                    .ok_or_else(|| CompileError::MissingColumn {
                        column: column.clone(),
                        path: path.to_path_buf(),
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let value = if let Some(column) = &value_column {
            let raw = row
                .get(column)
                .cloned()
                .ok_or_else(|| CompileError::MissingColumn {
                    column: column.clone(),
                    path: path.to_path_buf(),
                })?;
            raw.parse::<f64>()
                .map_err(|_| CompileError::InvalidNumber {
                    value: raw,
                    field: column.clone(),
                    path: path.to_path_buf(),
                })?
        } else {
            1.0
        };
        values.insert(key, value);
    }

    Ok(GenericDataTable {
        values,
        default_missing: value_column.is_none().then_some(0.0),
    })
}

fn infer_data_key_columns(
    program: &SemanticProgram,
    rows: &[HashMap<String, String>],
    headers: &[String],
    value_column: Option<&str>,
) -> Vec<String> {
    let candidate_columns: Vec<String> = headers
        .iter()
        .filter(|column| value_column != Some(column.as_str()))
        .cloned()
        .collect();

    // Use columns whose values match a declared set as key columns.
    let membership_columns: Vec<String> = candidate_columns
        .iter()
        .filter(|column| data_column_matches_any_set(program, rows, column))
        .cloned()
        .collect();
    if !membership_columns.is_empty() {
        return membership_columns;
    }

    // Fallback: treat every non-value column as a key column.
    candidate_columns
}

fn data_column_matches_any_set(
    program: &SemanticProgram,
    rows: &[HashMap<String, String>],
    column: &str,
) -> bool {
    let mut column_values = BTreeSet::new();
    for row in rows {
        let Some(value) = row.get(column) else {
            return false;
        };
        column_values.insert(value.as_str());
    }
    if column_values.is_empty() {
        return false;
    }

    program.set_registry.values().any(|set| {
        column_values
            .iter()
            .all(|v| set.values.iter().any(|s| s == v))
    })
}
