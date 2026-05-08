use crate::source::{LiteralValue, ParsedSource, SetDecl};
use arco_model::document::{ArcoDocument, IndexedDataDocument, ModelDocument};
use arco_model::indexed::{
    DuplicateReducer, IndexKey, IndexValue, IndexedData, ParameterTable, Set,
};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum PrimitiveBuildError {
    #[error("unsupported numeric literal in {context}: `{literal}`")]
    UnsupportedNumericLiteral { context: String, literal: String },
}

pub fn build_indexed_data(parsed: &ParsedSource) -> Result<IndexedData<f64>, PrimitiveBuildError> {
    let mut indexed = IndexedData::<f64>::default();

    for set_decl in &parsed.program.sets {
        insert_set_decl(&mut indexed, set_decl);
    }

    for model in &parsed.program.models {
        for set_decl in &model.sets {
            insert_set_decl(&mut indexed, set_decl);
        }
    }

    for param in &parsed.program.params {
        if let Some(value) = param.value.as_ref() {
            let scalar_value = parse_numeric_literal(value, &format!("param `{}`", param.name))?;
            let table = ParameterTable::from_rows(
                &param.name,
                [(IndexKey(Vec::new()), scalar_value)],
                DuplicateReducer::Sum,
            );
            indexed.parameters.insert(param.name.clone(), table);
        }
    }

    Ok(indexed)
}

pub fn build_model_document(parsed: &ParsedSource) -> ModelDocument {
    let _ = parsed;
    ModelDocument::new_f64()
}

pub fn build_arco_document(parsed: &ParsedSource) -> Result<ArcoDocument, PrimitiveBuildError> {
    let model_document = if parsed.program.models.is_empty() {
        None
    } else {
        Some(build_model_document(parsed))
    };

    let indexed_data_document =
        if parsed.program.sets.is_empty() && parsed.program.params.is_empty() {
            None
        } else {
            let _ = build_indexed_data(parsed)?;
            Some(IndexedDataDocument::new_f64())
        };

    let mut document = ArcoDocument::new_f64();
    document.model = model_document;
    document.indexed_data = indexed_data_document;
    Ok(document)
}

fn insert_set_decl(indexed: &mut IndexedData<f64>, set_decl: &SetDecl) {
    let mut set = Set::new(&set_decl.name);
    for member in &set_decl.members {
        set.insert(literal_to_index_value(member));
    }
    indexed.sets.insert(set_decl.name.clone(), set);
}

fn parse_numeric_literal(
    literal: &LiteralValue,
    context: &str,
) -> Result<f64, PrimitiveBuildError> {
    match literal {
        LiteralValue::Integer(value) => Ok(*value as f64),
        LiteralValue::Decimal(value) => {
            value
                .parse::<f64>()
                .map_err(|_| PrimitiveBuildError::UnsupportedNumericLiteral {
                    context: context.to_string(),
                    literal: value.clone(),
                })
        }
        LiteralValue::String(value) => Err(PrimitiveBuildError::UnsupportedNumericLiteral {
            context: context.to_string(),
            literal: value.clone(),
        }),
        LiteralValue::Boolean(value) => Err(PrimitiveBuildError::UnsupportedNumericLiteral {
            context: context.to_string(),
            literal: value.to_string(),
        }),
    }
}

fn literal_to_index_value(literal: &LiteralValue) -> IndexValue {
    match literal {
        LiteralValue::String(value) => IndexValue::String(value.clone()),
        LiteralValue::Integer(value) => IndexValue::Integer(*value as i64),
        LiteralValue::Decimal(value) => IndexValue::Decimal(value.clone()),
        LiteralValue::Boolean(value) => IndexValue::Bool(*value),
    }
}
