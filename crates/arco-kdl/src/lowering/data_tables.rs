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
    if parameter.comparators.eq.is_none()
        && parameter.comparators.ge.is_none()
        && parameter.comparators.geq.is_none()
        && parameter.comparators.le.is_none()
        && parameter.comparators.leq.is_none()
    {
        return true;
    }

    let filter_column = parameter
        .filter_by
        .as_ref()
        .or(parameter.from.as_ref())
        .map_or_else(
            || resolve_data_column(data_decl, &parameter.name),
            |logical_name| resolve_data_column(data_decl, logical_name),
        );
    let Some(raw_value) = row.get(&filter_column) else {
        return false;
    };

    compare_data_param_comparators(raw_value, &parameter.comparators)
}

fn compare_data_param_comparators(raw_value: &str, comparators: &FilterComparators) -> bool {
    if let Some(expected) = &comparators.eq {
        if !literal_filter_match(raw_value, expected) {
            return false;
        }
    }
    if let Some(expected) = &comparators.ge {
        if !numeric_filter_compare(raw_value, expected, |left, right| left > right) {
            return false;
        }
    }
    if let Some(expected) = &comparators.geq {
        if !numeric_filter_compare(raw_value, expected, |left, right| left >= right) {
            return false;
        }
    }
    if let Some(expected) = &comparators.le {
        if !numeric_filter_compare(raw_value, expected, |left, right| left < right) {
            return false;
        }
    }
    if let Some(expected) = &comparators.leq {
        if !numeric_filter_compare(raw_value, expected, |left, right| left <= right) {
            return false;
        }
    }
    true
}

fn literal_filter_match(raw_value: &str, literal: &LiteralValue) -> bool {
    match literal {
        LiteralValue::String(value) => raw_value == value,
        LiteralValue::Integer(value) => raw_value.parse::<i128>() == Ok(*value),
        LiteralValue::Decimal(value) => {
            let Ok(expected) = value.parse::<f64>() else {
                return false;
            };
            raw_value
                .parse::<f64>()
                .map(|actual| (actual - expected).abs() < 1e-9)
                .unwrap_or(false)
        }
        LiteralValue::Boolean(value) => {
            let normalized = raw_value.trim().to_ascii_lowercase();
            (*value && (normalized == "true" || normalized == "1"))
                || (!*value && (normalized == "false" || normalized == "0"))
        }
    }
}

fn numeric_filter_compare(
    raw_value: &str,
    literal: &LiteralValue,
    comparator: impl Fn(f64, f64) -> bool,
) -> bool {
    let Ok(actual) = raw_value.parse::<f64>() else {
        return false;
    };
    let expected = match literal {
        LiteralValue::Integer(value) => *value as f64,
        LiteralValue::Decimal(value) | LiteralValue::String(value) => {
            let Ok(parsed) = value.parse::<f64>() else {
                return false;
            };
            parsed
        }
        LiteralValue::Boolean(value) => {
            if *value {
                1.0
            } else {
                0.0
            }
        }
    };
    comparator(actual, expected)
}

fn load_generic_data_table(
    program: &SemanticProgram,
    binding_name: &str,
    rows: &[HashMap<String, String>],
    path: &Path,
) -> Result<GenericDataTable, LoweringError> {
    let Some(first_row) = rows.first() else {
        return Err(LoweringError::MissingData {
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
                    .ok_or_else(|| LoweringError::MissingColumn {
                        column: column.clone(),
                        path: path.to_path_buf(),
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let value = if let Some(column) = &value_column {
            let raw = row
                .get(column)
                .cloned()
                .ok_or_else(|| LoweringError::MissingColumn {
                    column: column.clone(),
                    path: path.to_path_buf(),
                })?;
            raw.parse::<f64>()
                .map_err(|_| LoweringError::InvalidNumber {
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
