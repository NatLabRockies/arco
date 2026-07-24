//! Stable primitive document DTOs.

use crate::indexed::{
    AttributeTable, Domain, IndexKey, IndexValue, IndexedData, ParameterStorage, ParameterTable,
    Set, TupleSet,
};
use crate::{
    Bounds, Constraint, Model, ModelBuilder, ModelError, ModelView, Objective, Sense, Variable,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Primitive document kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentKind {
    Model,
    IndexedData,
    Arco,
}

/// Scalar encoding used by a primitive document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScalarType {
    F32,
    F64,
    Decimal,
}

/// Shared document header.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentHeader {
    pub schema_version: u32,
    pub document_kind: DocumentKind,
    pub scalar_type: ScalarType,
}

impl DocumentHeader {
    pub(crate) fn v1(document_kind: DocumentKind, scalar_type: ScalarType) -> Self {
        Self {
            schema_version: 1,
            document_kind,
            scalar_type,
        }
    }
}

/// Canonical scalar string. NaN is intentionally not representable.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CanonicalScalar(String);

impl CanonicalScalar {
    pub(crate) fn from_f64(value: f64) -> Option<Self> {
        if value.is_finite() {
            Some(Self(value.to_string()))
        } else if value == f64::INFINITY {
            Some(Self("inf".to_string()))
        } else if value == f64::NEG_INFINITY {
            Some(Self("-inf".to_string()))
        } else {
            None
        }
    }

    pub(crate) fn to_f64(&self) -> Result<f64, DocumentError> {
        match self.0.as_str() {
            "inf" | "+inf" => Ok(f64::INFINITY),
            "-inf" => Ok(f64::NEG_INFINITY),
            value => {
                f64::from_str(value).map_err(|_| DocumentError::InvalidScalar(value.to_string()))
            }
        }
        .and_then(|value| {
            if value.is_nan() {
                Err(DocumentError::InvalidScalar(self.0.clone()))
            } else {
                Ok(value)
            }
        })
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

/// Document conversion failures.
#[derive(Debug)]
pub enum DocumentError {
    UnsupportedSchemaVersion(u32),
    WrongDocumentKind {
        expected: DocumentKind,
        actual: DocumentKind,
    },
    InvalidScalar(String),
    MissingScalar,
    Model(ModelError),
}

impl std::fmt::Display for DocumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion(version) => {
                write!(f, "unsupported primitive schema version {version}")
            }
            Self::WrongDocumentKind { expected, actual } => {
                write!(f, "expected {expected:?} document, got {actual:?}")
            }
            Self::InvalidScalar(value) => write!(f, "invalid canonical scalar {value}"),
            Self::MissingScalar => write!(f, "finite/non-NaN scalar required"),
            Self::Model(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for DocumentError {}
impl From<ModelError> for DocumentError {
    fn from(value: ModelError) -> Self {
        Self::Model(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundsDocument {
    pub(crate) lower: CanonicalScalar,
    pub(crate) upper: CanonicalScalar,
}

impl BoundsDocument {
    fn from_bounds(bounds: Bounds) -> Result<Self, DocumentError> {
        Ok(Self {
            lower: CanonicalScalar::from_f64(bounds.lower).ok_or(DocumentError::MissingScalar)?,
            upper: CanonicalScalar::from_f64(bounds.upper).ok_or(DocumentError::MissingScalar)?,
        })
    }
    fn to_bounds(&self) -> Result<Bounds, DocumentError> {
        Ok(Bounds::new(self.lower.to_f64()?, self.upper.to_f64()?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VariableDocument {
    pub(crate) id: u32,
    pub(crate) bounds: BoundsDocument,
    pub(crate) is_integer: bool,
    pub(crate) is_active: bool,
    pub(crate) name: Option<String>,
    pub(crate) metadata: Option<serde_json::Value>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstraintDocument {
    pub(crate) id: u32,
    pub(crate) bounds: BoundsDocument,
    pub(crate) name: Option<String>,
    pub(crate) metadata: Option<serde_json::Value>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoefficientDocument {
    pub(crate) variable_id: u32,
    pub(crate) constraint_id: u32,
    pub(crate) value: CanonicalScalar,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectiveTermDocument {
    pub(crate) variable_id: u32,
    pub(crate) value: CanonicalScalar,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectiveDocument {
    pub(crate) sense: Option<Sense>,
    pub(crate) terms: Vec<ObjectiveTermDocument>,
    pub(crate) name: Option<String>,
}

/// Stable model document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelDocument {
    #[serde(flatten)]
    pub header: DocumentHeader,
    pub fingerprint: Option<u64>,
    pub variables: Vec<VariableDocument>,
    pub constraints: Vec<ConstraintDocument>,
    pub coefficients: Vec<CoefficientDocument>,
    pub objective: ObjectiveDocument,
}

impl ModelDocument {
    pub fn new_f64() -> Self {
        Self {
            header: DocumentHeader::v1(DocumentKind::Model, ScalarType::F64),
            fingerprint: None,
            variables: Vec::new(),
            constraints: Vec::new(),
            coefficients: Vec::new(),
            objective: ObjectiveDocument {
                sense: None,
                terms: Vec::new(),
                name: None,
            },
        }
    }

    pub(crate) fn from_model(view: &impl ModelView) -> Result<Self, DocumentError> {
        let mut doc = Self::new_f64();
        doc.fingerprint = Some(view.fingerprint().0);
        for idx in 0..view.num_variables() {
            let id = crate::VariableId::new(idx as u32);
            let variable = view.variable(id).ok_or(DocumentError::MissingScalar)?;
            doc.variables.push(VariableDocument {
                id: idx as u32,
                bounds: BoundsDocument::from_bounds(variable.bounds)?,
                is_integer: variable.is_integer,
                is_active: variable.is_active,
                name: view.variable_name(id).map(str::to_string),
                metadata: view.variable_metadata(id).cloned(),
            });
            if let Some(column) = view.column(id) {
                for (row, value) in column {
                    doc.coefficients.push(CoefficientDocument {
                        variable_id: idx as u32,
                        constraint_id: row.inner(),
                        value: CanonicalScalar::from_f64(*value)
                            .ok_or(DocumentError::MissingScalar)?,
                    });
                }
            }
        }
        for idx in 0..view.num_constraints() {
            let id = crate::ConstraintId::new(idx as u32);
            let constraint = view.constraint(id).ok_or(DocumentError::MissingScalar)?;
            doc.constraints.push(ConstraintDocument {
                id: idx as u32,
                bounds: BoundsDocument::from_bounds(constraint.bounds)?,
                name: view.constraint_name(id).map(str::to_string),
                metadata: view.constraint_metadata(id).cloned(),
            });
        }
        doc.objective = ObjectiveDocument {
            sense: view.objective().sense,
            terms: view
                .objective()
                .terms
                .iter()
                .map(|(id, value)| {
                    Ok(ObjectiveTermDocument {
                        variable_id: id.inner(),
                        value: CanonicalScalar::from_f64(*value)
                            .ok_or(DocumentError::MissingScalar)?,
                    })
                })
                .collect::<Result<_, DocumentError>>()?,
            name: view.objective_name().map(str::to_string),
        };
        Ok(doc)
    }

    pub(crate) fn to_model(&self) -> Result<Model, DocumentError> {
        ensure_header(&self.header, DocumentKind::Model)?;
        let mut builder = ModelBuilder::<f64>::new();
        for variable in &self.variables {
            let id = builder.add_variable(Variable {
                bounds: variable.bounds.to_bounds()?,
                is_integer: variable.is_integer,
                is_active: variable.is_active,
            })?;
            if let Some(name) = &variable.name {
                builder.set_variable_name(id, name.clone())?;
            }
        }
        for constraint in &self.constraints {
            let id = builder.add_constraint(Constraint {
                bounds: constraint.bounds.to_bounds()?,
            })?;
            if let Some(name) = &constraint.name {
                builder.set_constraint_name(id, name.clone())?;
            }
        }
        for coefficient in &self.coefficients {
            builder.set_coefficient(
                crate::VariableId::new(coefficient.variable_id),
                crate::ConstraintId::new(coefficient.constraint_id),
                coefficient.value.to_f64()?,
            )?;
        }
        builder.set_objective(Objective {
            sense: self.objective.sense,
            terms: self
                .objective
                .terms
                .iter()
                .map(|term| {
                    Ok((
                        crate::VariableId::new(term.variable_id),
                        term.value.to_f64()?,
                    ))
                })
                .collect::<Result<_, DocumentError>>()?,
        })?;
        builder.set_objective_name(self.objective.name.clone())?;
        let mut model = builder.finish_legacy_model();
        for variable in &self.variables {
            if let Some(metadata) = &variable.metadata {
                model
                    .set_variable_metadata(crate::VariableId::new(variable.id), metadata.clone())?;
            }
        }
        for constraint in &self.constraints {
            if let Some(metadata) = &constraint.metadata {
                model.set_constraint_metadata(
                    crate::ConstraintId::new(constraint.id),
                    metadata.clone(),
                )?;
            }
        }
        Ok(model)
    }
}

fn ensure_header(header: &DocumentHeader, kind: DocumentKind) -> Result<(), DocumentError> {
    if header.schema_version != 1 {
        return Err(DocumentError::UnsupportedSchemaVersion(
            header.schema_version,
        ));
    }
    if header.document_kind != kind {
        return Err(DocumentError::WrongDocumentKind {
            expected: kind,
            actual: header.document_kind,
        });
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetDocument {
    pub(crate) name: String,
    pub(crate) values: Vec<IndexValue>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TupleSetDocument {
    pub(crate) name: String,
    pub(crate) arity: usize,
    pub(crate) keys: Vec<IndexKey>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainDocument {
    pub(crate) name: String,
    pub(crate) keys: Vec<IndexKey>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParameterStorageDocument {
    Sparse {
        rows: Vec<(IndexKey, CanonicalScalar)>,
    },
    Dense {
        shape: Vec<usize>,
        values: Vec<CanonicalScalar>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParameterTableDocument {
    pub(crate) name: String,
    pub(crate) storage: ParameterStorageDocument,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributeTableDocument {
    pub(crate) name: String,
    pub(crate) rows: Vec<(IndexKey, String)>,
}

/// Stable indexed-data document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexedDataDocument {
    #[serde(flatten)]
    pub(crate) header: DocumentHeader,
    pub(crate) sets: Vec<SetDocument>,
    pub(crate) tuple_sets: Vec<TupleSetDocument>,
    pub(crate) domains: Vec<DomainDocument>,
    pub(crate) parameters: Vec<ParameterTableDocument>,
    pub(crate) attributes: Vec<AttributeTableDocument>,
}

impl IndexedDataDocument {
    pub fn new_f64() -> Self {
        Self {
            header: DocumentHeader::v1(DocumentKind::IndexedData, ScalarType::F64),
            sets: Vec::new(),
            tuple_sets: Vec::new(),
            domains: Vec::new(),
            parameters: Vec::new(),
            attributes: Vec::new(),
        }
    }
    pub(crate) fn from_indexed_data(data: &IndexedData<f64>) -> Result<Self, DocumentError> {
        let mut doc = Self::new_f64();
        doc.sets = data
            .sets
            .values()
            .map(|set| SetDocument {
                name: set.name().to_string(),
                values: set.values().cloned().collect(),
            })
            .collect();
        doc.tuple_sets = data
            .tuple_sets
            .values()
            .map(|set| TupleSetDocument {
                name: set.name().to_string(),
                arity: set.arity(),
                keys: set.keys().cloned().collect(),
            })
            .collect();
        doc.domains = data
            .domains
            .values()
            .map(|domain| DomainDocument {
                name: domain.name().to_string(),
                keys: domain.keys().to_vec(),
            })
            .collect();
        doc.parameters = data
            .parameters
            .values()
            .map(|table| {
                let storage = match table.storage() {
                    ParameterStorage::Sparse(rows) => ParameterStorageDocument::Sparse {
                        rows: rows
                            .iter()
                            .map(|(key, value)| {
                                Ok((
                                    key.clone(),
                                    CanonicalScalar::from_f64(*value)
                                        .ok_or(DocumentError::MissingScalar)?,
                                ))
                            })
                            .collect::<Result<_, DocumentError>>()?,
                    },
                    ParameterStorage::Dense { shape, values } => ParameterStorageDocument::Dense {
                        shape: shape.clone(),
                        values: values
                            .iter()
                            .map(|value| {
                                CanonicalScalar::from_f64(*value)
                                    .ok_or(DocumentError::MissingScalar)
                            })
                            .collect::<Result<_, DocumentError>>()?,
                    },
                };
                Ok(ParameterTableDocument {
                    name: table.name().to_string(),
                    storage,
                })
            })
            .collect::<Result<_, DocumentError>>()?;
        doc.attributes = data
            .attributes
            .values()
            .map(|table| AttributeTableDocument {
                name: table.name().to_string(),
                rows: table
                    .rows()
                    .into_iter()
                    .map(|(key, value)| (key, value.clone()))
                    .collect(),
            })
            .collect();
        Ok(doc)
    }
    pub(crate) fn to_indexed_data(&self) -> Result<IndexedData<f64>, DocumentError> {
        ensure_header(&self.header, DocumentKind::IndexedData)?;
        let mut data = IndexedData::default();
        for set_doc in &self.sets {
            let mut set = Set::new(set_doc.name.clone());
            for value in &set_doc.values {
                set.insert(value.clone());
            }
            data.sets.insert(set_doc.name.clone(), set);
        }
        for tuple_doc in &self.tuple_sets {
            let mut set = TupleSet::new(tuple_doc.name.clone(), tuple_doc.arity);
            for key in &tuple_doc.keys {
                set.insert(key.clone());
            }
            data.tuple_sets.insert(tuple_doc.name.clone(), set);
        }
        for domain in &self.domains {
            data.domains.insert(
                domain.name.clone(),
                Domain::new(domain.name.clone(), domain.keys.clone()),
            );
        }
        for parameter in &self.parameters {
            let table = match &parameter.storage {
                ParameterStorageDocument::Sparse { rows } => {
                    let rows = rows
                        .iter()
                        .map(|(key, value)| Ok((key.clone(), value.to_f64()?)))
                        .collect::<Result<Vec<_>, DocumentError>>()?;
                    ParameterTable::from_rows(
                        parameter.name.clone(),
                        rows,
                        crate::indexed::DuplicateReducer::Sum,
                    )
                }
                ParameterStorageDocument::Dense { shape, values } => ParameterTable::from_dense(
                    parameter.name.clone(),
                    shape.clone(),
                    values
                        .iter()
                        .map(CanonicalScalar::to_f64)
                        .collect::<Result<Vec<_>, DocumentError>>()?,
                )
                .map_err(|_| DocumentError::InvalidScalar(parameter.name.clone()))?,
            };
            data.parameters.insert(parameter.name.clone(), table);
        }
        for attribute in &self.attributes {
            let mut table = AttributeTable::new(attribute.name.clone());
            for (key, value) in &attribute.rows {
                table.insert(key.clone(), value.clone());
            }
            data.attributes.insert(attribute.name.clone(), table);
        }
        Ok(data)
    }
}

/// Top-level primitive document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArcoDocument {
    #[serde(flatten)]
    pub header: DocumentHeader,
    pub model: Option<ModelDocument>,
    pub indexed_data: Option<IndexedDataDocument>,
}
impl ArcoDocument {
    pub fn new_f64() -> Self {
        Self {
            header: DocumentHeader::v1(DocumentKind::Arco, ScalarType::F64),
            model: None,
            indexed_data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::document::{
        ArcoDocument, CanonicalScalar, DocumentKind, IndexedDataDocument, ModelDocument,
    };
    use crate::indexed::{
        AttributeTable, Domain, IndexKey, IndexValue, IndexedData, ParameterTable, Set,
    };
    use crate::{Bounds, Constraint, Model, ModelView, Objective, Sense, Variable};

    #[test]
    fn documents_carry_kind_and_schema_version() {
        let doc = ArcoDocument::new_f64();
        assert_eq!(doc.header.schema_version, 1);
        assert_eq!(doc.header.document_kind, DocumentKind::Arco);
    }

    #[test]
    fn scalar_strings_reject_nan() {
        assert!(CanonicalScalar::from_f64(f64::NAN).is_none());
        assert_eq!(
            CanonicalScalar::from_f64(f64::INFINITY).unwrap().as_str(),
            "inf"
        );
    }

    #[test]
    fn model_document_json_roundtrips_model_contract() {
        let mut model = Model::new();
        let x = model
            .add_variable(Variable::integer(Bounds::new(0.0, f64::INFINITY)))
            .unwrap();
        let c = model
            .add_constraint(Constraint {
                bounds: Bounds::new(-f64::INFINITY, 5.0),
            })
            .unwrap();
        model.set_coefficient(x, c, 2.5).unwrap();
        model
            .set_objective(Objective {
                sense: Some(Sense::Maximize),
                terms: vec![(x, 3.0)],
            })
            .unwrap();
        model.set_variable_name(x, "x".to_string()).unwrap();
        model
            .set_constraint_metadata(c, serde_json::json!({"source":"test"}))
            .unwrap();
        let json = serde_json::to_string(&ModelDocument::from_model(&model).unwrap()).unwrap();
        assert!(json.contains("\"document_kind\":\"model\""));
        let restored = serde_json::from_str::<ModelDocument>(&json)
            .unwrap()
            .to_model()
            .unwrap();
        assert_eq!(restored.variable(x), model.variable(x));
        assert_eq!(restored.constraint(c), model.constraint(c));
        assert_eq!(restored.column(x), model.column(x));
        assert_eq!(restored.objective().terms, model.objective().terms);
        assert_eq!(restored.get_variable_name(x), Some("x"));
        assert_eq!(
            restored.get_constraint_metadata(c),
            Some(&serde_json::json!({"source":"test"}))
        );
    }

    #[test]
    fn indexed_data_document_json_roundtrips_indexed_primitives() {
        let mut data = IndexedData::default();
        let mut set = Set::new("zones");
        set.insert(IndexValue::String("north".into()));
        data.sets.insert("zones".into(), set);
        let key = IndexKey(vec![IndexValue::String("north".into())]);
        data.domains.insert(
            "zone_domain".into(),
            Domain::new("zone_domain", vec![key.clone()]),
        );
        data.parameters.insert(
            "demand".into(),
            ParameterTable::from_rows(
                "demand",
                [(key.clone(), 7.0)],
                crate::indexed::DuplicateReducer::Sum,
            ),
        );
        let mut attrs = AttributeTable::new("labels");
        attrs.insert(key.clone(), "North");
        data.attributes.insert("labels".into(), attrs);
        let json =
            serde_json::to_string(&IndexedDataDocument::from_indexed_data(&data).unwrap()).unwrap();
        assert!(json.contains("\"document_kind\":\"indexed_data\""));
        let restored = serde_json::from_str::<IndexedDataDocument>(&json)
            .unwrap()
            .to_indexed_data()
            .unwrap();
        assert_eq!(
            restored.sets["zones"].values().cloned().collect::<Vec<_>>(),
            data.sets["zones"].values().cloned().collect::<Vec<_>>()
        );
        assert_eq!(restored.parameters["demand"].get(&key), Some(&7.0));
        assert_eq!(restored.attributes["labels"].get(&key), Some("North"));
    }
}
