fn generic_data_value(
    generic_data: &BTreeMap<String, GenericDataTable>,
    target: &str,
    resolved: &[FilterValue],
    entrypoint: &Path,
) -> Result<Option<f64>, LoweringError> {
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
) -> Result<String, LoweringError> {
    let parts = resolved
        .iter()
        .map(|value| filter_value_to_key_component(value, entrypoint))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(parts.join(","))
}

fn filter_value_to_key_component(
    value: &FilterValue,
    entrypoint: &Path,
) -> Result<String, LoweringError> {
    match value {
        FilterValue::String(value) => Ok(value.clone()),
        FilterValue::Number(value) => {
            if value.fract() == 0.0 {
                Ok((*value as i64).to_string())
            } else {
                Ok(value.to_string())
            }
        }
        FilterValue::Boolean(_) => Err(LoweringError::InvalidFormulation {
            message: "boolean indices are not supported for data lookups".to_string(),
            path: entrypoint.to_path_buf(),
        }),
    }
}

/// Convert a `BoundExpr::Literal` to `f64`. Returns `None` for
/// `BoundExpr::Formula` (parameter-based bounds need a linearization context
/// that isn't available yet).
fn literal_bound_to_f64(bound: &BoundExpr, path: &Path) -> Result<Option<f64>, LoweringError> {
    match bound {
        BoundExpr::Literal(literal) => literal_to_f64("bound", literal, path).map(Some),
        BoundExpr::Formula(_) => Ok(None),
    }
}

fn literal_to_f64(name: &str, value: &LiteralValue, path: &Path) -> Result<f64, LoweringError> {
    match value {
        LiteralValue::Integer(value) => Ok(*value as f64),
        LiteralValue::Decimal(value) | LiteralValue::String(value) => {
            value
                .parse::<f64>()
                .map_err(|_| LoweringError::InvalidNumber {
                    value: value.clone(),
                    field: name.to_string(),
                    path: path.to_path_buf(),
                })
        }
        LiteralValue::Boolean(value) => Ok(if *value { 1.0 } else { 0.0 }),
    }
}

fn asset_parameter(asset: &AssetInputs, name: &str, path: &Path) -> Result<f64, LoweringError> {
    asset
        .parameters
        .get(name)
        .copied()
        .ok_or_else(|| LoweringError::MissingParameter {
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
) -> Result<f64, LoweringError> {
    series
        .get(name)
        .ok_or_else(|| LoweringError::MissingData {
            name: name.to_string(),
            path: path.to_path_buf(),
        })?
        .get(&time)
        .copied()
        .ok_or_else(|| LoweringError::MissingDataPoint {
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
) -> Result<f64, LoweringError> {
    indexed
        .get(name)
        .ok_or_else(|| LoweringError::MissingData {
            name: name.to_string(),
            path: path.to_path_buf(),
        })?
        .get(&(asset_name.to_string(), time))
        .copied()
        .ok_or_else(|| LoweringError::MissingDataPoint {
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
) -> Result<f64, LoweringError> {
    asset_data
        .get(name)
        .ok_or_else(|| LoweringError::MissingData {
            name: name.to_string(),
            path: path.to_path_buf(),
        })?
        .get(asset_name)
        .copied()
        .ok_or_else(|| LoweringError::MissingDataPoint {
            name: name.to_string(),
            key: asset_name.to_string(),
            path: path.to_path_buf(),
        })
}
