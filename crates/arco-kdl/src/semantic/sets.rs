use crate::semantic::error::SemanticError;
use crate::semantic::types::{ResolvedSet, ResolvedSets};
use crate::source::{
    FilterComparators, LiteralValue, ModelDecl, SetDecl, SourceProgram, SubsetDecl,
};
use csv::StringRecord;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(crate) struct SetCsvData {
    pub(crate) members: Vec<String>,
    pub(crate) params: BTreeMap<String, BTreeMap<String, f64>>,
}

pub(crate) fn load_set_csv(path: &Path) -> Result<SetCsvData, SemanticError> {
    let mut reader = csv::Reader::from_path(path).map_err(|source| SemanticError::Csv {
        path: path.to_path_buf(),
        source,
    })?;
    let headers = reader
        .headers()
        .map_err(|source| SemanticError::Csv {
            path: path.to_path_buf(),
            source,
        })?
        .clone();

    let name_index = headers.iter().position(|h| h == "name").unwrap_or(0);

    let param_columns: Vec<(usize, String)> = headers
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != name_index)
        .map(|(i, h)| (i, h.to_string()))
        .collect();

    let mut members = Vec::new();
    let mut member_params = BTreeMap::new();

    for (row_index, record) in reader.records().enumerate() {
        let record = record.map_err(|source| SemanticError::Csv {
            path: path.to_path_buf(),
            source,
        })?;
        let member_name = record
            .get(name_index)
            .filter(|v| !v.is_empty())
            .ok_or_else(|| SemanticError::MissingCell {
                column: "name".to_string(),
                row: row_index + 1,
                path: path.to_path_buf(),
            })?
            .to_string();

        let mut params = BTreeMap::new();
        for (col_index, col_name) in &param_columns {
            if let Some(raw) = record.get(*col_index).filter(|v| !v.is_empty()) {
                if let Ok(value) = raw.parse::<f64>() {
                    params.insert(col_name.clone(), value);
                }
            }
        }
        members.push(member_name.clone());
        member_params.insert(member_name, params);
    }

    Ok(SetCsvData {
        members,
        params: member_params,
    })
}

pub(crate) fn build_set_registry(
    sets: &ResolvedSets,
    custom_sets: &BTreeMap<String, Vec<String>>,
) -> BTreeMap<String, ResolvedSet> {
    let mut registry = BTreeMap::new();
    registry.insert(
        "time".to_string(),
        ResolvedSet {
            values: (1..=sets.time.steps).map(|step| step.to_string()).collect(),
        },
    );
    for (name, members) in custom_sets {
        registry.insert(
            name.clone(),
            ResolvedSet {
                values: members.clone(),
            },
        );
    }
    registry
}

pub(crate) fn collect_set_aliases(
    program: &SourceProgram,
    model: Option<&ModelDecl>,
) -> BTreeMap<String, String> {
    let mut aliases = BTreeMap::new();

    for set_decl in &program.sets {
        if let Some(alias) = &set_decl.alias {
            aliases.insert(alias.clone(), set_decl.name.clone());
        }
    }

    for data_decl in &program.data {
        for set_decl in &data_decl.sets {
            if let Some(alias) = &set_decl.alias {
                aliases.insert(alias.clone(), set_decl.name.clone());
            }
        }
    }

    if let Some(model) = model {
        for set_decl in &model.sets {
            if let Some(alias) = &set_decl.alias {
                aliases.insert(alias.clone(), set_decl.name.clone());
            }
        }
    }

    aliases
}

pub(crate) fn extend_set_registry_from_low_level_declarations(
    program: &SourceProgram,
    entry_dir: &Path,
    registry: &mut BTreeMap<String, ResolvedSet>,
) -> Result<(), SemanticError> {
    let mut data_rows = BTreeMap::new();
    for data_decl in &program.data {
        let csv_path = entry_dir.join(&data_decl.source);
        let rows = read_csv_rows(&csv_path)?;
        data_rows.insert(data_decl.name.clone(), rows);

        for set_decl in &data_decl.sets {
            let values = values_for_data_set(data_decl, set_decl, &data_rows[&data_decl.name]);
            registry.insert(set_decl.name.clone(), ResolvedSet { values });
        }
    }

    for set_decl in &program.sets {
        let values = if set_decl.members.is_empty() {
            values_for_top_level_set(program, set_decl, &data_rows)
        } else {
            set_decl
                .members
                .iter()
                .filter_map(|m| match m {
                    LiteralValue::String(value) => Some(value.clone()),
                    _ => None,
                })
                .collect()
        };
        registry.insert(set_decl.name.clone(), ResolvedSet { values });
    }

    for subset_decl in &program.subsets {
        let values = values_for_subset(program, subset_decl, &data_rows);
        registry.insert(subset_decl.name.clone(), ResolvedSet { values });
    }

    Ok(())
}

fn read_csv_rows(path: &Path) -> Result<Vec<BTreeMap<String, String>>, SemanticError> {
    let mut reader = csv::Reader::from_path(path).map_err(|source| SemanticError::Csv {
        path: path.to_path_buf(),
        source,
    })?;
    let headers = reader
        .headers()
        .map_err(|source| SemanticError::Csv {
            path: path.to_path_buf(),
            source,
        })?
        .clone();

    reader
        .records()
        .enumerate()
        .map(|(index, record)| {
            let record = record.map_err(|source| SemanticError::Csv {
                path: path.to_path_buf(),
                source,
            })?;
            record_to_map(path, &headers, index + 1, record)
        })
        .collect()
}

fn record_to_map(
    path: &Path,
    headers: &StringRecord,
    row_index: usize,
    record: StringRecord,
) -> Result<BTreeMap<String, String>, SemanticError> {
    let mut row = BTreeMap::new();
    for (header, value) in headers.iter().zip(record.iter()) {
        if value.is_empty() {
            return Err(SemanticError::MissingCell {
                column: header.to_string(),
                row: row_index,
                path: path.to_path_buf(),
            });
        }
        row.insert(header.to_string(), value.to_string());
    }
    Ok(row)
}

fn values_for_top_level_set(
    program: &SourceProgram,
    set_decl: &SetDecl,
    data_rows: &BTreeMap<String, Vec<BTreeMap<String, String>>>,
) -> Vec<String> {
    let Some(source) = &set_decl.source else {
        return Vec::new();
    };
    let Some(data_decl) = program.data(source) else {
        return Vec::new();
    };
    let Some(rows) = data_rows.get(source) else {
        return Vec::new();
    };
    values_for_data_set(data_decl, set_decl, rows)
}

fn values_for_subset(
    program: &SourceProgram,
    subset_decl: &SubsetDecl,
    data_rows: &BTreeMap<String, Vec<BTreeMap<String, String>>>,
) -> Vec<String> {
    let Some(data_decl) = program.data(&subset_decl.source) else {
        return Vec::new();
    };
    let Some(rows) = data_rows.get(&subset_decl.source) else {
        return Vec::new();
    };

    let target_column = data_decl.maps.first().map_or_else(
        || "name".to_string(),
        |mapping| {
            mapping
                .source
                .clone()
                .unwrap_or_else(|| mapping.name.clone())
        },
    );

    let mut values = BTreeSet::new();
    for row in rows {
        if !field_filters_match(row, data_decl, &subset_decl.field_filters) {
            continue;
        }
        if !comparators_match(
            row,
            data_decl,
            subset_decl.filter_by.as_deref(),
            Some(target_column.as_str()),
            &subset_decl.comparators,
        ) {
            continue;
        }
        if let Some(value) = row.get(&target_column) {
            values.insert(value.clone());
        }
    }
    values.into_iter().collect()
}

fn values_for_data_set(
    data_decl: &crate::source::DataDecl,
    set_decl: &SetDecl,
    rows: &[BTreeMap<String, String>],
) -> Vec<String> {
    let target_column = source_column_for_logical_name(data_decl, &set_decl.name);
    let mut values = BTreeSet::new();
    for row in rows {
        if !comparators_match(
            row,
            data_decl,
            set_decl.filter_by.as_deref(),
            Some(target_column.as_str()),
            &set_decl.comparators,
        ) {
            continue;
        }
        if let Some(value) = row.get(&target_column) {
            values.insert(value.clone());
        }
    }
    values.into_iter().collect()
}

fn source_column_for_logical_name(data_decl: &crate::source::DataDecl, logical: &str) -> String {
    data_decl
        .maps
        .iter()
        .find(|mapping| mapping.name == logical)
        .map_or_else(
            || logical.to_string(),
            |mapping| {
                mapping
                    .source
                    .clone()
                    .unwrap_or_else(|| mapping.name.clone())
            },
        )
}

fn field_filters_match(
    row: &BTreeMap<String, String>,
    data_decl: &crate::source::DataDecl,
    field_filters: &BTreeMap<String, LiteralValue>,
) -> bool {
    field_filters.iter().all(|(field, expected)| {
        let source_field = source_column_for_logical_name(data_decl, field);
        row.get(&source_field)
            .is_some_and(|actual| literal_matches(actual, expected))
    })
}

fn comparators_match(
    row: &BTreeMap<String, String>,
    data_decl: &crate::source::DataDecl,
    filter_by: Option<&str>,
    default_column: Option<&str>,
    comparators: &FilterComparators,
) -> bool {
    if comparators.eq.is_none()
        && comparators.ge.is_none()
        && comparators.geq.is_none()
        && comparators.le.is_none()
        && comparators.leq.is_none()
    {
        return true;
    }

    let column = filter_by
        .map(|name| source_column_for_logical_name(data_decl, name))
        .or_else(|| default_column.map(ToString::to_string));
    let Some(column) = column else {
        return true;
    };
    let Some(raw_value) = row.get(&column) else {
        return false;
    };

    if let Some(expected) = &comparators.eq {
        if !literal_matches(raw_value, expected) {
            return false;
        }
    }
    if let Some(expected) = &comparators.ge {
        if !literal_numeric_compare(raw_value, expected, |actual, threshold| actual > threshold) {
            return false;
        }
    }
    if let Some(expected) = &comparators.geq {
        if !literal_numeric_compare(raw_value, expected, |actual, threshold| actual >= threshold) {
            return false;
        }
    }
    if let Some(expected) = &comparators.le {
        if !literal_numeric_compare(raw_value, expected, |actual, threshold| actual < threshold) {
            return false;
        }
    }
    if let Some(expected) = &comparators.leq {
        if !literal_numeric_compare(raw_value, expected, |actual, threshold| actual <= threshold) {
            return false;
        }
    }
    true
}

fn literal_matches(actual: &str, expected: &LiteralValue) -> bool {
    match expected {
        LiteralValue::String(value) => actual == value,
        LiteralValue::Integer(value) => actual.parse::<i128>() == Ok(*value),
        LiteralValue::Decimal(value) => {
            let Ok(expected_value) = value.parse::<f64>() else {
                return false;
            };
            actual
                .parse::<f64>()
                .map(|actual_value| (actual_value - expected_value).abs() < 1e-9)
                .unwrap_or(false)
        }
        LiteralValue::Boolean(value) => {
            let normalized = actual.trim().to_ascii_lowercase();
            (*value && (normalized == "true" || normalized == "1"))
                || (!*value && (normalized == "false" || normalized == "0"))
        }
    }
}

fn literal_numeric_compare(
    actual: &str,
    expected: &LiteralValue,
    compare: impl Fn(f64, f64) -> bool,
) -> bool {
    let Ok(actual_value) = actual.parse::<f64>() else {
        return false;
    };
    let expected_value = match expected {
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
    compare(actual_value, expected_value)
}
