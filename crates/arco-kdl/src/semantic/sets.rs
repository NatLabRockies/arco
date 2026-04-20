use crate::algebra::{BinaryOp, ComparisonOp, Expr, UnaryOp};
use crate::semantic::error::SemanticError;
use crate::semantic::types::ResolvedSet;
use crate::source::{DataDecl, LiteralValue, ModelDecl, SetDecl, SourceProgram};
use csv::StringRecord;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use tracing::warn;

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
    let set_aliases = collect_set_aliases(program, None);
    for data_decl in &program.data {
        let csv_path = entry_dir.join(&data_decl.source);
        let rows = read_csv_rows(&csv_path)?;
        validate_data_filter_identifiers(data_decl, &rows, &csv_path)?;
        for set_decl in &data_decl.sets {
            let resolved_set = resolved_set_for_data_set(data_decl, set_decl, &rows, &csv_path)?;
            merge_resolved_set_into_registry(&set_decl.name, resolved_set, registry, entry_dir)?;
        }
    }
    for set_decl in &program.sets {
        let resolved_set =
            resolved_set_for_program_set(set_decl, registry, &set_aliases, entry_dir)?;
        merge_resolved_set_into_registry(&set_decl.name, resolved_set, registry, entry_dir)?;
    }
    Ok(())
}
fn merge_resolved_set_into_registry(
    set_name: &str,
    next: ResolvedSet,
    registry: &mut BTreeMap<String, ResolvedSet>,
    merge_path: &Path,
) -> Result<(), SemanticError> {
    if let Some(existing) = registry.get(set_name) {
        if let Some(intersection) =
            intersect_tuple_set_sources(set_name, existing, &next, merge_path)?
        {
            registry.insert(set_name.to_string(), intersection);
            return Ok(());
        }
    }
    registry.insert(set_name.to_string(), next);
    Ok(())
}

fn intersect_tuple_set_sources(
    set_name: &str,
    existing: &ResolvedSet,
    next: &ResolvedSet,
    merge_path: &Path,
) -> Result<Option<ResolvedSet>, SemanticError> {
    let (Some(existing_components), Some(existing_rows), Some(next_components), Some(next_rows)) = (
        existing.tuple_components.as_ref(),
        existing.tuple_rows.as_ref(),
        next.tuple_components.as_ref(),
        next.tuple_rows.as_ref(),
    ) else {
        return Ok(None);
    };
    if existing_components != next_components {
        return Err(SemanticError::TupleSetSchemaMismatch {
            set: set_name.to_string(),
            existing_components: existing_components.join(","),
            incoming_components: next_components.join(","),
            path: merge_path.to_path_buf(),
        });
    }
    let existing_rows = existing_rows.iter().cloned().collect::<BTreeSet<_>>();
    let next_rows = next_rows.iter().cloned().collect::<BTreeSet<_>>();
    let intersected_rows = existing_rows
        .intersection(&next_rows)
        .cloned()
        .collect::<Vec<_>>();
    Ok(Some(ResolvedSet {
        values: Vec::new(),
        tuple_components: Some(existing_components.clone()),
        tuple_rows: Some(intersected_rows),
    }))
}

fn resolved_set_for_program_set(
    set_decl: &SetDecl,
    registry: &BTreeMap<String, ResolvedSet>,
    set_aliases: &BTreeMap<String, String>,
    path: &Path,
) -> Result<ResolvedSet, SemanticError> {
    if set_decl.tuple_indices.is_empty() {
        let values = set_decl
            .members
            .iter()
            .map(literal_to_string)
            .collect::<Vec<_>>();
        return Ok(ResolvedSet {
            values,
            tuple_components: None,
            tuple_rows: None,
        });
    }

    let tuple_rows = tuple_rows_for_rule_set(set_decl, registry, set_aliases, path)?;
    Ok(ResolvedSet {
        values: Vec::new(),
        tuple_components: Some(
            set_decl
                .tuple_indices
                .iter()
                .map(|tuple_index| tuple_index.name.clone())
                .collect(),
        ),
        tuple_rows: Some(tuple_rows),
    })
}

fn tuple_rows_for_rule_set(
    set_decl: &SetDecl,
    registry: &BTreeMap<String, ResolvedSet>,
    set_aliases: &BTreeMap<String, String>,
    path: &Path,
) -> Result<Vec<Vec<String>>, SemanticError> {
    let mut domain_values = Vec::with_capacity(set_decl.tuple_indices.len());
    for tuple_index in &set_decl.tuple_indices {
        let domain_name = tuple_index
            .domain
            .as_deref()
            .unwrap_or(tuple_index.name.as_str());
        let domain_set = resolve_set_for_rule_domain(domain_name, registry, set_aliases)
            .ok_or_else(|| SemanticError::MissingDeclaration {
                kind: "set",
                name: domain_name.to_string(),
                path: path.to_path_buf(),
            })?;
        if domain_set.values.is_empty() {
            return Err(SemanticError::MissingDeclaration {
                kind: "set",
                name: domain_name.to_string(),
                path: path.to_path_buf(),
            });
        }
        domain_values.push(domain_set.values.clone());
    }

    let allowed_identifiers = tuple_rule_filter_identifiers(set_decl, set_aliases);
    validate_rule_set_filter_identifiers(set_decl, &allowed_identifiers, path)?;

    let mut combinations = vec![Vec::new()];
    for values in &domain_values {
        let mut next = Vec::new();
        for combo in &combinations {
            for value in values {
                let mut extended = combo.clone();
                extended.push(value.clone());
                next.push(extended);
            }
        }
        combinations = next;
    }

    let mut tuples = BTreeSet::new();
    for combo in combinations {
        let mut binding_values = BTreeMap::new();
        for (position, tuple_index) in set_decl.tuple_indices.iter().enumerate() {
            let value = combo[position].clone();
            binding_values.insert(tuple_index.name.clone(), value.clone());
            let domain_name = tuple_index
                .domain
                .as_deref()
                .unwrap_or(tuple_index.name.as_str());
            add_tuple_rule_binding_aliases(&mut binding_values, domain_name, value, set_aliases);
        }

        if matches_rule_set_filter(&binding_values, set_decl) {
            tuples.insert(combo);
        }
    }

    Ok(tuples.into_iter().collect())
}

fn resolve_set_for_rule_domain<'a>(
    name: &str,
    registry: &'a BTreeMap<String, ResolvedSet>,
    set_aliases: &BTreeMap<String, String>,
) -> Option<&'a ResolvedSet> {
    if let Some(set) = registry.get(name) {
        return Some(set);
    }
    if let Some(canonical) = set_aliases.get(name) {
        if let Some(set) = registry.get(canonical.as_str()) {
            return Some(set);
        }
    }
    for (alias, canonical) in set_aliases {
        if canonical == name {
            if let Some(set) = registry.get(alias.as_str()) {
                return Some(set);
            }
        }
    }

    None
}

fn add_tuple_rule_binding_aliases(
    binding_values: &mut BTreeMap<String, String>,
    domain_name: &str,
    value: String,
    set_aliases: &BTreeMap<String, String>,
) {
    binding_values.insert(domain_name.to_string(), value.clone());
    if let Some(canonical) = set_aliases.get(domain_name) {
        binding_values.insert(canonical.clone(), value.clone());
    }
    for (alias, canonical) in set_aliases {
        if canonical == domain_name {
            binding_values.insert(alias.clone(), value.clone());
        }
    }
}

fn tuple_rule_filter_identifiers(
    set_decl: &SetDecl,
    set_aliases: &BTreeMap<String, String>,
) -> BTreeSet<String> {
    let mut identifiers = BTreeSet::new();
    for tuple_index in &set_decl.tuple_indices {
        identifiers.insert(tuple_index.name.clone());
        let domain_name = tuple_index
            .domain
            .as_deref()
            .unwrap_or(tuple_index.name.as_str());
        identifiers.insert(domain_name.to_string());
        collect_domain_aliases_for_rule_filter(&mut identifiers, domain_name, set_aliases);
    }

    identifiers
}

fn collect_domain_aliases_for_rule_filter(
    identifiers: &mut BTreeSet<String>,
    domain_name: &str,
    set_aliases: &BTreeMap<String, String>,
) {
    if let Some(canonical) = set_aliases.get(domain_name) {
        identifiers.insert(canonical.clone());
    }
    for (alias, canonical) in set_aliases {
        if canonical == domain_name {
            identifiers.insert(alias.clone());
        }
    }
}

fn validate_rule_set_filter_identifiers(
    set_decl: &SetDecl,
    allowed_identifiers: &BTreeSet<String>,
    path: &Path,
) -> Result<(), SemanticError> {
    let Some(expr) = set_decl.parsed_filter_expression.as_ref() else {
        return Ok(());
    };

    validate_rule_set_filter_expr_internal(expr, allowed_identifiers, &set_decl.name, path, false)
}

fn validate_rule_set_filter_expr_internal(
    expr: &Expr,
    allowed_identifiers: &BTreeSet<String>,
    set_name: &str,
    path: &Path,
    allow_unresolved_identifier: bool,
) -> Result<(), SemanticError> {
    match expr {
        Expr::Comparison { left, right, .. } => {
            validate_rule_set_filter_column_side_expr(left, allowed_identifiers, set_name, path)?;
            validate_rule_set_filter_expr_internal(right, allowed_identifiers, set_name, path, true)
        }
        Expr::Unary { expr, .. } => validate_rule_set_filter_expr_internal(
            expr,
            allowed_identifiers,
            set_name,
            path,
            allow_unresolved_identifier,
        ),
        Expr::Binary { left, right, .. } => {
            validate_rule_set_filter_expr_internal(
                left,
                allowed_identifiers,
                set_name,
                path,
                allow_unresolved_identifier,
            )?;
            validate_rule_set_filter_expr_internal(
                right,
                allowed_identifiers,
                set_name,
                path,
                allow_unresolved_identifier,
            )
        }
        Expr::FunctionCall { args, .. } => {
            for arg in args {
                validate_rule_set_filter_expr_internal(
                    arg,
                    allowed_identifiers,
                    set_name,
                    path,
                    allow_unresolved_identifier,
                )?;
            }
            Ok(())
        }
        Expr::Indexed { indices, .. } => {
            for index in indices {
                validate_rule_set_filter_expr_internal(
                    index,
                    allowed_identifiers,
                    set_name,
                    path,
                    allow_unresolved_identifier,
                )?;
            }
            Ok(())
        }
        Expr::Reduction(reduction) => {
            validate_rule_set_filter_expr_internal(
                &reduction.body,
                allowed_identifiers,
                set_name,
                path,
                allow_unresolved_identifier,
            )?;
            for filter in &reduction.filters {
                validate_rule_set_filter_expr_internal(
                    filter,
                    allowed_identifiers,
                    set_name,
                    path,
                    allow_unresolved_identifier,
                )?;
            }
            Ok(())
        }
        Expr::Identifier(identifier) => {
            if allow_unresolved_identifier {
                Ok(())
            } else {
                validate_rule_set_filter_identifier(identifier, allowed_identifiers, set_name, path)
            }
        }
        Expr::Number(_) | Expr::String(_) | Expr::Boolean(_) => Ok(()),
    }
}

fn validate_rule_set_filter_column_side_expr(
    expr: &Expr,
    allowed_identifiers: &BTreeSet<String>,
    set_name: &str,
    path: &Path,
) -> Result<(), SemanticError> {
    match expr {
        Expr::Identifier(identifier) => {
            validate_rule_set_filter_identifier(identifier, allowed_identifiers, set_name, path)
        }
        Expr::Unary { expr, .. } => {
            validate_rule_set_filter_column_side_expr(expr, allowed_identifiers, set_name, path)
        }
        Expr::Binary { left, right, .. } | Expr::Comparison { left, right, .. } => {
            validate_rule_set_filter_column_side_expr(left, allowed_identifiers, set_name, path)?;
            validate_rule_set_filter_column_side_expr(right, allowed_identifiers, set_name, path)
        }
        Expr::FunctionCall { args, .. } => {
            for arg in args {
                validate_rule_set_filter_column_side_expr(
                    arg,
                    allowed_identifiers,
                    set_name,
                    path,
                )?;
            }
            Ok(())
        }
        Expr::Indexed { indices, .. } => {
            for index in indices {
                validate_rule_set_filter_column_side_expr(
                    index,
                    allowed_identifiers,
                    set_name,
                    path,
                )?;
            }
            Ok(())
        }
        Expr::Reduction(reduction) => {
            validate_rule_set_filter_column_side_expr(
                &reduction.body,
                allowed_identifiers,
                set_name,
                path,
            )?;
            for filter in &reduction.filters {
                validate_rule_set_filter_column_side_expr(
                    filter,
                    allowed_identifiers,
                    set_name,
                    path,
                )?;
            }
            Ok(())
        }
        Expr::Number(_) | Expr::String(_) | Expr::Boolean(_) => Ok(()),
    }
}

fn validate_rule_set_filter_identifier(
    identifier: &str,
    allowed_identifiers: &BTreeSet<String>,
    set_name: &str,
    path: &Path,
) -> Result<(), SemanticError> {
    if allowed_identifiers.contains(identifier) {
        return Ok(());
    }

    Err(SemanticError::UnresolvedRuleSetFilterIdentifier {
        identifier: identifier.to_string(),
        set: set_name.to_string(),
        path: path.to_path_buf(),
    })
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

fn validate_data_filter_identifiers(
    data_decl: &DataDecl,
    rows: &[BTreeMap<String, String>],
    path: &Path,
) -> Result<(), SemanticError> {
    let Some(first_row) = rows.first() else {
        return Ok(());
    };
    let source_columns = first_row.keys().cloned().collect::<BTreeSet<_>>();

    for set_decl in &data_decl.sets {
        validate_filter_identifiers_for_declaration(
            data_decl,
            set_decl.parsed_filter_expression.as_ref(),
            &source_columns,
            "set",
            &set_decl.name,
            path,
        )?;
    }

    for param_decl in &data_decl.parameters {
        validate_filter_identifiers_for_declaration(
            data_decl,
            param_decl.parsed_filter_expression.as_ref(),
            &source_columns,
            "param",
            &param_decl.name,
            path,
        )?;
    }

    Ok(())
}

fn validate_filter_identifiers_for_declaration(
    data_decl: &DataDecl,
    expr: Option<&Expr>,
    source_columns: &BTreeSet<String>,
    declaration_kind: &'static str,
    declaration_name: &str,
    path: &Path,
) -> Result<(), SemanticError> {
    let Some(expr) = expr else {
        return Ok(());
    };

    validate_filter_expr(
        expr,
        data_decl,
        source_columns,
        declaration_kind,
        declaration_name,
        path,
    )
}

fn validate_filter_expr(
    expr: &Expr,
    data_decl: &DataDecl,
    source_columns: &BTreeSet<String>,
    declaration_kind: &'static str,
    declaration_name: &str,
    path: &Path,
) -> Result<(), SemanticError> {
    validate_filter_expr_internal(
        expr,
        data_decl,
        source_columns,
        declaration_kind,
        declaration_name,
        path,
        false,
    )
}

fn validate_filter_expr_internal(
    expr: &Expr,
    data_decl: &DataDecl,
    source_columns: &BTreeSet<String>,
    declaration_kind: &'static str,
    declaration_name: &str,
    path: &Path,
    allow_unresolved_identifier: bool,
) -> Result<(), SemanticError> {
    match expr {
        Expr::Comparison { left, right, .. } => {
            validate_filter_column_side_expr(
                left,
                data_decl,
                source_columns,
                declaration_kind,
                declaration_name,
                path,
            )?;
            validate_filter_expr_internal(
                right,
                data_decl,
                source_columns,
                declaration_kind,
                declaration_name,
                path,
                true,
            )
        }
        Expr::Unary { expr, .. } => validate_filter_expr_internal(
            expr,
            data_decl,
            source_columns,
            declaration_kind,
            declaration_name,
            path,
            allow_unresolved_identifier,
        ),
        Expr::Binary { left, right, .. } => {
            validate_filter_expr_internal(
                left,
                data_decl,
                source_columns,
                declaration_kind,
                declaration_name,
                path,
                allow_unresolved_identifier,
            )?;
            validate_filter_expr_internal(
                right,
                data_decl,
                source_columns,
                declaration_kind,
                declaration_name,
                path,
                allow_unresolved_identifier,
            )
        }
        Expr::FunctionCall { args, .. } => {
            for arg in args {
                validate_filter_expr_internal(
                    arg,
                    data_decl,
                    source_columns,
                    declaration_kind,
                    declaration_name,
                    path,
                    allow_unresolved_identifier,
                )?;
            }
            Ok(())
        }
        Expr::Indexed { indices, .. } => {
            for index in indices {
                validate_filter_expr_internal(
                    index,
                    data_decl,
                    source_columns,
                    declaration_kind,
                    declaration_name,
                    path,
                    allow_unresolved_identifier,
                )?;
            }
            Ok(())
        }
        Expr::Reduction(reduction) => {
            validate_filter_expr_internal(
                &reduction.body,
                data_decl,
                source_columns,
                declaration_kind,
                declaration_name,
                path,
                allow_unresolved_identifier,
            )?;
            for filter in &reduction.filters {
                validate_filter_expr_internal(
                    filter,
                    data_decl,
                    source_columns,
                    declaration_kind,
                    declaration_name,
                    path,
                    allow_unresolved_identifier,
                )?;
            }
            Ok(())
        }
        Expr::Identifier(identifier) => {
            if allow_unresolved_identifier {
                Ok(())
            } else {
                validate_filter_identifier(
                    identifier,
                    data_decl,
                    source_columns,
                    declaration_kind,
                    declaration_name,
                    path,
                )
            }
        }
        Expr::Number(_) | Expr::String(_) | Expr::Boolean(_) => Ok(()),
    }
}

fn validate_filter_column_side_expr(
    expr: &Expr,
    data_decl: &DataDecl,
    source_columns: &BTreeSet<String>,
    declaration_kind: &'static str,
    declaration_name: &str,
    path: &Path,
) -> Result<(), SemanticError> {
    match expr {
        Expr::Identifier(identifier) => validate_filter_identifier(
            identifier,
            data_decl,
            source_columns,
            declaration_kind,
            declaration_name,
            path,
        ),
        Expr::Unary { expr, .. } => validate_filter_column_side_expr(
            expr,
            data_decl,
            source_columns,
            declaration_kind,
            declaration_name,
            path,
        ),
        Expr::Binary { left, right, .. } | Expr::Comparison { left, right, .. } => {
            validate_filter_column_side_expr(
                left,
                data_decl,
                source_columns,
                declaration_kind,
                declaration_name,
                path,
            )?;
            validate_filter_column_side_expr(
                right,
                data_decl,
                source_columns,
                declaration_kind,
                declaration_name,
                path,
            )
        }
        Expr::FunctionCall { args, .. } => {
            for arg in args {
                validate_filter_column_side_expr(
                    arg,
                    data_decl,
                    source_columns,
                    declaration_kind,
                    declaration_name,
                    path,
                )?;
            }
            Ok(())
        }
        Expr::Indexed { indices, .. } => {
            for index in indices {
                validate_filter_column_side_expr(
                    index,
                    data_decl,
                    source_columns,
                    declaration_kind,
                    declaration_name,
                    path,
                )?;
            }
            Ok(())
        }
        Expr::Reduction(reduction) => {
            validate_filter_column_side_expr(
                &reduction.body,
                data_decl,
                source_columns,
                declaration_kind,
                declaration_name,
                path,
            )?;
            for filter in &reduction.filters {
                validate_filter_column_side_expr(
                    filter,
                    data_decl,
                    source_columns,
                    declaration_kind,
                    declaration_name,
                    path,
                )?;
            }
            Ok(())
        }
        Expr::Number(_) | Expr::String(_) | Expr::Boolean(_) => Ok(()),
    }
}

fn validate_filter_identifier(
    identifier: &str,
    data_decl: &DataDecl,
    source_columns: &BTreeSet<String>,
    declaration_kind: &'static str,
    declaration_name: &str,
    path: &Path,
) -> Result<(), SemanticError> {
    let source_name = source_column_for_logical_name(data_decl, identifier);
    if source_columns.contains(identifier) || source_columns.contains(source_name.as_str()) {
        return Ok(());
    }

    Err(SemanticError::UnresolvedFilterIdentifier {
        identifier: identifier.to_string(),
        declaration_kind,
        declaration: declaration_name.to_string(),
        data: data_decl.name.clone(),
        path: path.to_path_buf(),
    })
}

fn resolved_set_for_data_set(
    data_decl: &DataDecl,
    set_decl: &SetDecl,
    rows: &[BTreeMap<String, String>],
    path: &Path,
) -> Result<ResolvedSet, SemanticError> {
    if !set_decl.tuple_indices.is_empty() {
        let tuple_rows = tuple_rows_for_data_set(data_decl, set_decl, rows, path)?;
        if set_decl.parsed_filter_expression.is_some() && !rows.is_empty() && tuple_rows.is_empty()
        {
            warn!(
                data = %data_decl.name,
                set = %set_decl.name,
                filter = ?set_decl.filter_expression,
                "filtered subset resolved empty"
            );
        }

        return Ok(ResolvedSet {
            values: Vec::new(),
            tuple_components: Some(
                set_decl
                    .tuple_indices
                    .iter()
                    .map(|index| index.name.clone())
                    .collect(),
            ),
            tuple_rows: Some(tuple_rows),
        });
    }
    let source_set_name = source_set_name_for_data_set_values(data_decl, set_decl);
    let target_column = source_column_for_logical_name(data_decl, &source_set_name);
    let mut values = BTreeSet::new();
    for row in rows {
        if !matches_data_set_filter(row, data_decl, set_decl) {
            continue;
        }
        if let Some(value) = row.get(&target_column) {
            values.insert(value.clone());
        }
    }
    let values = values.into_iter().collect::<Vec<_>>();
    if set_decl.parsed_filter_expression.is_some() && !rows.is_empty() && values.is_empty() {
        warn!(
            data = %data_decl.name,
            set = %set_decl.name,
            filter = ?set_decl.filter_expression,
            "filtered subset resolved empty"
        );
    }

    Ok(ResolvedSet {
        values,
        tuple_components: None,
        tuple_rows: None,
    })
}

fn tuple_rows_for_data_set(
    data_decl: &DataDecl,
    set_decl: &SetDecl,
    rows: &[BTreeMap<String, String>],
    path: &Path,
) -> Result<Vec<Vec<String>>, SemanticError> {
    let mut tuples = BTreeSet::new();
    for (row_index, row) in rows.iter().enumerate() {
        if !matches_data_set_filter(row, data_decl, set_decl) {
            continue;
        }

        let mut tuple = Vec::with_capacity(set_decl.tuple_indices.len());
        for tuple_index in &set_decl.tuple_indices {
            let logical_name = tuple_index
                .domain
                .as_deref()
                .unwrap_or(tuple_index.name.as_str());
            let source_column = source_column_for_logical_name(data_decl, logical_name);
            if let Some(value) = row
                .get(&source_column)
                .or_else(|| row.get(logical_name))
                .cloned()
            {
                tuple.push(value);
            } else {
                return Err(SemanticError::MissingColumn {
                    column: source_column,
                    path: path.to_path_buf(),
                });
            }
        }

        if tuple.len() != set_decl.tuple_indices.len() {
            return Err(SemanticError::MissingCell {
                column: set_decl.tuple_indices[tuple.len()].name.clone(),
                row: row_index + 1,
                path: path.to_path_buf(),
            });
        }
        tuples.insert(tuple);
    }

    Ok(tuples.into_iter().collect())
}

fn source_set_name_for_data_set_values(data_decl: &DataDecl, set_decl: &SetDecl) -> String {
    let Some(parent_name) = set_decl.subset_of.as_deref() else {
        return set_decl.name.clone();
    };
    data_decl
        .sets
        .iter()
        .find(|candidate| candidate.alias.as_deref() == Some(parent_name))
        .map_or_else(
            || parent_name.to_string(),
            |candidate| candidate.name.clone(),
        )
}

fn matches_data_set_filter(
    row: &BTreeMap<String, String>,
    data_decl: &DataDecl,
    set_decl: &SetDecl,
) -> bool {
    let Some(expr) = set_decl.parsed_filter_expression.as_ref() else {
        return true;
    };
    evaluate_data_set_filter_expr(expr, row, data_decl)
        .and_then(|value| truthy_data_set_filter_value(&value))
        .unwrap_or(false)
}

fn matches_rule_set_filter(row: &BTreeMap<String, String>, set_decl: &SetDecl) -> bool {
    let Some(expr) = set_decl.parsed_filter_expression.as_ref() else {
        return true;
    };

    evaluate_rule_set_filter_expr(expr, row)
        .and_then(|value| truthy_data_set_filter_value(&value))
        .unwrap_or(false)
}
#[derive(Debug, Clone)]
enum DataSetFilterValue {
    Number(f64),
    String(String),
    Boolean(bool),
}
fn evaluate_data_set_filter_expr(
    expr: &Expr,
    row: &BTreeMap<String, String>,
    data_decl: &DataDecl,
) -> Option<DataSetFilterValue> {
    evaluate_set_filter_expr(expr, &|name| {
        let source_name = source_column_for_logical_name(data_decl, name);
        row.get(name).or_else(|| row.get(&source_name)).cloned()
    })
}

fn evaluate_rule_set_filter_expr(
    expr: &Expr,
    row: &BTreeMap<String, String>,
) -> Option<DataSetFilterValue> {
    evaluate_set_filter_expr(expr, &|name| row.get(name).cloned())
}

fn evaluate_set_filter_expr<F>(expr: &Expr, resolve_identifier: &F) -> Option<DataSetFilterValue>
where
    F: Fn(&str) -> Option<String>,
{
    match expr {
        Expr::Number(value) => value.parse::<f64>().ok().map(DataSetFilterValue::Number),
        Expr::String(value) => Some(DataSetFilterValue::String(value.clone())),
        Expr::Boolean(value) => Some(DataSetFilterValue::Boolean(*value)),
        Expr::Identifier(name) => Some(DataSetFilterValue::String(
            resolve_identifier(name).unwrap_or_else(|| name.clone()),
        )),
        Expr::Unary { op, expr } => {
            let value = evaluate_set_filter_expr(expr, resolve_identifier)?;
            match op {
                UnaryOp::Negate => {
                    data_set_filter_numeric_value(&value).map(|v| DataSetFilterValue::Number(-v))
                }
            }
        }
        Expr::Binary { op, left, right } => {
            let left = evaluate_set_filter_expr(left, resolve_identifier)?;
            let right = evaluate_set_filter_expr(right, resolve_identifier)?;
            let left = data_set_filter_numeric_value(&left)?;
            let right = data_set_filter_numeric_value(&right)?;
            Some(DataSetFilterValue::Number(match op {
                BinaryOp::Add => left + right,
                BinaryOp::Subtract => left - right,
                BinaryOp::Multiply => left * right,
                BinaryOp::Divide => left / right,
            }))
        }
        Expr::Comparison { op, left, right } => {
            let left = evaluate_set_filter_expr(left, resolve_identifier)?;
            let right = evaluate_set_filter_expr(right, resolve_identifier)?;
            compare_data_set_filter_values(*op, &left, &right).map(DataSetFilterValue::Boolean)
        }
        Expr::Indexed { .. } | Expr::FunctionCall { .. } | Expr::Reduction(_) => None,
    }
}

fn compare_data_set_filter_values(
    op: ComparisonOp,
    left: &DataSetFilterValue,
    right: &DataSetFilterValue,
) -> Option<bool> {
    match op {
        ComparisonOp::Equal | ComparisonOp::DoubleEqual => {
            compare_data_set_filter_for_equality(left, right)
        }
        ComparisonOp::NotEqual => {
            compare_data_set_filter_for_equality(left, right).map(|value| !value)
        }
        ComparisonOp::Less
        | ComparisonOp::LessEqual
        | ComparisonOp::Greater
        | ComparisonOp::GreaterEqual => {
            let left = data_set_filter_numeric_value(left)?;
            let right = data_set_filter_numeric_value(right)?;
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

fn compare_data_set_filter_for_equality(
    left: &DataSetFilterValue,
    right: &DataSetFilterValue,
) -> Option<bool> {
    match (left, right) {
        (DataSetFilterValue::String(left), DataSetFilterValue::String(right)) => {
            Some(left == right)
        }
        (DataSetFilterValue::Boolean(left), DataSetFilterValue::Boolean(right)) => {
            Some(left == right)
        }
        _ => {
            let left = data_set_filter_numeric_value(left)?;
            let right = data_set_filter_numeric_value(right)?;
            Some((left - right).abs() < f64::EPSILON)
        }
    }
}

fn truthy_data_set_filter_value(value: &DataSetFilterValue) -> Option<bool> {
    match value {
        DataSetFilterValue::Boolean(value) => Some(*value),
        DataSetFilterValue::Number(value) => Some(*value != 0.0),
        DataSetFilterValue::String(value) => {
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

fn data_set_filter_numeric_value(value: &DataSetFilterValue) -> Option<f64> {
    match value {
        DataSetFilterValue::Number(value) => Some(*value),
        DataSetFilterValue::Boolean(value) => Some(if *value { 1.0 } else { 0.0 }),
        DataSetFilterValue::String(value) => {
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

fn source_column_for_logical_name(data_decl: &DataDecl, logical: &str) -> String {
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
