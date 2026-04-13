use crate::semantic::error::SemanticError;
use crate::semantic::types::ResolvedSet;
use crate::source::{LiteralValue, ModelDecl, SetDecl, SourceProgram};
use csv::StringRecord;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

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
        let values = set_decl
            .members
            .iter()
            .map(literal_to_string)
            .collect::<Vec<_>>();
        registry.insert(set_decl.name.clone(), ResolvedSet { values });
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

fn values_for_data_set(
    data_decl: &crate::source::DataDecl,
    set_decl: &SetDecl,
    rows: &[BTreeMap<String, String>],
) -> Vec<String> {
    let target_column = source_column_for_logical_name(data_decl, &set_decl.name);
    let mut values = BTreeSet::new();

    for row in rows {
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

pub(crate) fn literal_to_string(value: &LiteralValue) -> String {
    match value {
        LiteralValue::String(v) => v.clone(),
        LiteralValue::Integer(v) => v.to_string(),
        LiteralValue::Decimal(v) => v.clone(),
        LiteralValue::Boolean(v) => v.to_string(),
    }
}
