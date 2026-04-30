fn load_inputs(
    program: &SemanticProgram,
    source_program: &SourceProgram,
    scenario: &ScenarioDecl,
    entrypoint: &Path,
) -> Result<ScenarioInputs, CompileError> {
    let entry_dir = entrypoint
        .parent()
        .ok_or_else(|| CompileError::MissingScenario {
            name: program.active_scenario.clone(),
            path: entrypoint.to_path_buf(),
        })?;

    let mut assets = if let Some(asset_set) = program.set_registry.get("assets") {
        let candidate_assets = program
            .set_registry
            .get("candidate_assets")
            .map(|set| set.values.iter().cloned().collect::<BTreeSet<_>>())
            .unwrap_or_default();
        asset_set
            .values
            .iter()
            .map(|asset_name| AssetInputs {
                name: asset_name.clone(),
                families: BTreeSet::new(),
                parameters: BTreeMap::new(),
                candidate: candidate_assets.contains(asset_name),
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    if let Some(model_name) = &scenario.model_use {
        let model =
            source_program
                .model(model_name)
                .ok_or_else(|| CompileError::MissingDeclaration {
                    kind: "model",
                    name: model_name.clone(),
                    path: entrypoint.to_path_buf(),
                })?;

        let model_families: BTreeSet<String> = model
            .controls
            .iter()
            .map(|control| control.name.clone())
            .collect();

        if assets.is_empty() {
            // No explicit assets: create a synthetic default asset.
            if !model_families.is_empty() {
                assets.push(AssetInputs {
                    name: "default".to_string(),
                    families: model_families,
                    parameters: BTreeMap::new(),
                    candidate: false,
                });
            }
        } else {
            // Explicit assets declared: assign model control families to each.
            for asset in &mut assets {
                asset.families.clone_from(&model_families);
            }
        }
    }

    assets.sort_by(|a, b| a.name.cmp(&b.name));

    let mut series = BTreeMap::new();
    let mut indexed = BTreeMap::new();
    let mut asset_data = BTreeMap::new();
    let mut generic_data = BTreeMap::new();
    let mut set_params = BTreeMap::new();

    for data_decl in &source_program.data {
        load_data_decl_params(
            source_program,
            data_decl,
            entry_dir,
            &mut generic_data,
            &mut set_params,
        )?;
    }

    for binding in &scenario.data {
        generic_data.remove(&binding.name);

        let csv_path = entry_dir.join(&binding.source);
        let rows = read_csv_rows(&csv_path)?;
        let Some(first_row) = rows.first() else {
            return Err(CompileError::MissingData {
                name: binding.name.clone(),
                path: csv_path,
            });
        };
        let headers = first_row.keys().cloned().collect::<BTreeSet<_>>();
        if headers.contains("asset_name") && headers.contains("t") {
            let mut values = BTreeMap::new();
            for row in &rows {
                let asset_name =
                    row.get("asset_name")
                        .cloned()
                        .ok_or_else(|| CompileError::MissingColumn {
                            column: "asset_name".to_string(),
                            path: csv_path.clone(),
                        })?;
                let time = parse_usize_field(row, "t", &csv_path)?;
                let value = parse_data_value(row, &binding.name, &csv_path)?;
                values.insert((asset_name, time), value);
            }
            indexed.insert(binding.name.clone(), values);
        } else if headers.contains("asset_name") {
            let mut values = BTreeMap::new();
            for row in &rows {
                let asset_name =
                    row.get("asset_name")
                        .cloned()
                        .ok_or_else(|| CompileError::MissingColumn {
                            column: "asset_name".to_string(),
                            path: csv_path.clone(),
                        })?;
                let value = parse_data_value(row, &binding.name, &csv_path)?;
                values.insert(asset_name, value);
            }
            asset_data.insert(binding.name.clone(), values);
        } else if headers.contains("t") {
            let mut values = BTreeMap::new();
            for row in &rows {
                let time = parse_usize_field(row, "t", &csv_path)?;
                let value = parse_data_value(row, &binding.name, &csv_path)?;
                values.insert(time, value);
            }
            series.insert(binding.name.clone(), values);
        } else {
            let table = load_generic_data_table(program, &binding.name, &rows, &csv_path)?;
            generic_data.insert(binding.name.clone(), table);
        }
    }

    Ok(ScenarioInputs {
        assets,
        series,
        indexed,
        asset_data,
        generic_data,
        set_params,
    })
}

#[derive(Debug, Clone, Default)]
struct AffineExpr {
    constant: f64,
    terms: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Default)]
struct LinearizationBindings {
    values: BTreeMap<String, FilterValue>,
}

fn read_csv_rows(path: &Path) -> Result<Vec<HashMap<String, String>>, CompileError> {
    let mut reader = csv::Reader::from_path(path).map_err(|source| CompileError::Csv {
        path: path.to_path_buf(),
        source,
    })?;
    let headers = reader
        .headers()
        .map_err(|source| CompileError::Csv {
            path: path.to_path_buf(),
            source,
        })?
        .clone();

    reader
        .records()
        .map(|record| {
            let record = record.map_err(|source| CompileError::Csv {
                path: path.to_path_buf(),
                source,
            })?;
            record_to_map(path, &headers, record)
        })
        .collect()
}

fn record_to_map(
    path: &Path,
    headers: &StringRecord,
    record: StringRecord,
) -> Result<HashMap<String, String>, CompileError> {
    let mut row = HashMap::with_capacity(headers.len());
    for i in 0..headers.len() {
        if let Some(value) = record.get(i) {
            row.insert(headers[i].to_string(), value.to_string());
        }
    }
    if row.is_empty() {
        return Err(CompileError::MissingData {
            name: "csv".to_string(),
            path: path.to_path_buf(),
        });
    }
    Ok(row)
}

fn parse_usize_field(
    row: &HashMap<String, String>,
    field: &str,
    path: &Path,
) -> Result<usize, CompileError> {
    let raw = row
        .get(field)
        .cloned()
        .ok_or_else(|| CompileError::MissingColumn {
            column: field.to_string(),
            path: path.to_path_buf(),
        })?;
    raw.parse::<usize>()
        .map_err(|_| CompileError::InvalidNumber {
            value: raw,
            field: field.to_string(),
            path: path.to_path_buf(),
        })
}

fn parse_data_value(
    row: &HashMap<String, String>,
    name: &str,
    path: &Path,
) -> Result<f64, CompileError> {
    let raw = row
        .get(name)
        .cloned()
        .or_else(|| row.get("value").cloned())
        .ok_or_else(|| CompileError::MissingColumn {
            column: name.to_string(),
            path: path.to_path_buf(),
        })?;
    raw.parse::<f64>()
        .map_err(|_| CompileError::InvalidNumber {
            value: raw,
            field: name.to_string(),
            path: path.to_path_buf(),
        })
}

fn load_data_decl_params(
    source_program: &SourceProgram,
    data_decl: &DataDecl,
    entry_dir: &Path,
    generic_data: &mut BTreeMap<String, GenericDataTable>,
    set_params: &mut BTreeMap<String, BTreeMap<String, f64>>,
) -> Result<(), CompileError> {
    let csv_path = entry_dir.join(&data_decl.source);
    let rows = read_csv_rows(&csv_path)?;
    if rows.is_empty() {
        return Ok(());
    }

    for parameter in &data_decl.parameters {
        let value_column = parameter
            .from
            .clone()
            .unwrap_or_else(|| parameter.name.clone());
        let key_columns = resolve_param_key_columns(source_program, data_decl, parameter);
        let mut values: BTreeMap<Vec<String>, f64> = BTreeMap::new();
        let mut counts: BTreeMap<Vec<String>, usize> = BTreeMap::new();

        for row in &rows {
            if !matches_data_param_filter(row, data_decl, parameter) {
                continue;
            }

            let value_raw = row
                .get(&value_column)
                .cloned()
                .or_else(|| row.get("value").cloned())
                .ok_or_else(|| CompileError::MissingColumn {
                    column: value_column.clone(),
                    path: csv_path.clone(),
                })?;
            let value = value_raw
                .parse::<f64>()
                .map_err(|_| CompileError::InvalidNumber {
                    value: value_raw,
                    field: value_column.clone(),
                    path: csv_path.clone(),
                })?;

            let key = key_columns
                .iter()
                .map(|column| {
                    row.get(column)
                        .cloned()
                        .ok_or_else(|| CompileError::MissingColumn {
                            column: column.clone(),
                            path: csv_path.clone(),
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;

            let reducer = parameter.reduce.as_deref().unwrap_or("replace");
            match reducer {
                "sum" => {
                    *values.entry(key.clone()).or_insert(0.0) += value;
                }
                "min" => {
                    values
                        .entry(key.clone())
                        .and_modify(|existing| *existing = existing.min(value))
                        .or_insert(value);
                }
                "max" => {
                    values
                        .entry(key.clone())
                        .and_modify(|existing| *existing = existing.max(value))
                        .or_insert(value);
                }
                "avg" | "mean" => {
                    *values.entry(key.clone()).or_insert(0.0) += value;
                    *counts.entry(key).or_insert(0) += 1;
                }
                _ => {
                    values.insert(key, value);
                }
            }
        }

        if matches!(parameter.reduce.as_deref(), Some("avg" | "mean")) {
            for (key, sum) in &mut values {
                if let Some(count) = counts.get(key) {
                    *sum /= *count as f64;
                }
            }
        }

        if key_columns.len() == 1 {
            let members = set_params.entry(parameter.name.clone()).or_default();
            for (key, value) in &values {
                if let Some(member) = key.first() {
                    members.insert(member.clone(), *value);
                }
            }
        }

        generic_data.insert(
            parameter.name.clone(),
            GenericDataTable {
                values,
                default_missing: None,
            },
        );
    }

    Ok(())
}

fn resolve_param_key_columns(
    source_program: &SourceProgram,
    data_decl: &DataDecl,
    parameter: &ParamDecl,
) -> Vec<String> {
    if !parameter.indices.is_empty() {
        let indices = if parameter.indices.len() == 1 {
            expand_tuple_param_index_shorthand(source_program, data_decl, &parameter.indices[0])
                .unwrap_or_else(|| parameter.indices.clone())
        } else {
            parameter.indices.clone()
        };

        return indices
            .iter()
            .map(|index| canonical_data_set_name(source_program, data_decl, index))
            .map(|index| resolve_data_column(data_decl, index.as_str()))
            .collect();
    }

    if let Some(index) = &parameter.index {
        let canonical = canonical_data_set_name(source_program, data_decl, index);
        return vec![resolve_data_column(data_decl, &canonical)];
    }

    if let Some(index_decl) = data_decl.indices.first() {
        return index_decl
            .columns
            .iter()
            .map(|column| canonical_data_set_name(source_program, data_decl, column))
            .map(|column| resolve_data_column(data_decl, column.as_str()))
            .collect();
    }

    Vec::new()
}

fn expand_tuple_param_index_shorthand(
    source_program: &SourceProgram,
    data_decl: &DataDecl,
    symbol: &str,
) -> Option<Vec<String>> {
    let canonical = canonical_data_set_name(source_program, data_decl, symbol);

    data_decl
        .sets
        .iter()
        .chain(source_program.sets.iter())
        .find(|set| set.name == canonical)
        .and_then(|set| {
            if set.tuple_indices.is_empty() {
                None
            } else {
                Some(
                    set.tuple_indices
                        .iter()
                        .map(|index| {
                            index
                                .domain
                                .clone()
                                .unwrap_or_else(|| index.name.clone())
                        })
                        .collect::<Vec<_>>(),
                )
            }
        })
}

fn canonical_data_set_name(
    source_program: &SourceProgram,
    data_decl: &DataDecl,
    symbol: &str,
) -> String {
    if let Some(local) = data_decl
        .sets
        .iter()
        .find(|set| set.alias.as_deref() == Some(symbol))
        .map(|set| set.name.clone())
    {
        return local;
    }

    if let Some(global) = source_program
        .data
        .iter()
        .flat_map(|decl| decl.sets.iter())
        .chain(source_program.sets.iter())
        .find(|set| set.alias.as_deref() == Some(symbol))
        .map(|set| set.name.clone())
    {
        return global;
    }

    symbol.to_string()
}
