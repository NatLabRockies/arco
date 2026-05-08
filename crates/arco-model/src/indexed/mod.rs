//! Primitive indexed data: sets, domains, parameter tables, and attributes.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::hash::Hash;

/// Atomic index values supported by primitive documents v1.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexValue {
    String(String),
    Integer(i64),
    Decimal(String),
    Bool(bool),
}

/// Ordered key into a domain or table.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IndexKey(pub Vec<IndexValue>);

/// Ordered unique scalar set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Set {
    name: String,
    values: BTreeSet<IndexValue>,
}

impl Set {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            values: BTreeSet::new(),
        }
    }

    pub fn insert(&mut self, value: IndexValue) -> bool {
        self.values.insert(value)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn values(&self) -> impl Iterator<Item = &IndexValue> {
        self.values.iter()
    }
}

/// Ordered unique tuple set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TupleSet {
    name: String,
    arity: usize,
    keys: BTreeSet<IndexKey>,
}

impl TupleSet {
    pub fn new(name: impl Into<String>, arity: usize) -> Self {
        Self {
            name: name.into(),
            arity,
            keys: BTreeSet::new(),
        }
    }

    pub fn insert(&mut self, key: IndexKey) -> bool {
        if key.0.len() == self.arity {
            self.keys.insert(key)
        } else {
            false
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn arity(&self) -> usize {
        self.arity
    }

    pub fn keys(&self) -> impl Iterator<Item = &IndexKey> {
        self.keys.iter()
    }
}

/// Domain backed by ordered index keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Domain {
    name: String,
    keys: Vec<IndexKey>,
}

impl Domain {
    pub fn new(name: impl Into<String>, keys: Vec<IndexKey>) -> Self {
        Self {
            name: name.into(),
            keys,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn keys(&self) -> &[IndexKey] {
        &self.keys
    }
}

/// Duplicate-key reducer for numeric parameter construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplicateReducer {
    Sum,
    Min,
    Max,
    Count,
    Mean,
}

/// Parameter-table construction failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterTableError {
    ShapeValueCountMismatch { expected: usize, actual: usize },
}

impl std::fmt::Display for ParameterTableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ShapeValueCountMismatch { expected, actual } => write!(
                f,
                "dense parameter shape expects {expected} values, received {actual}"
            ),
        }
    }
}

impl std::error::Error for ParameterTableError {}

/// Numeric parameter table storage.
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterStorage<S = f64> {
    Sparse(BTreeMap<IndexKey, S>),
    Dense { shape: Vec<usize>, values: Vec<S> },
}

/// Numeric parameter table.
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterTable<S = f64> {
    name: String,
    storage: ParameterStorage<S>,
}

impl<S: Copy + PartialOrd + From<u32> + std::ops::Add<Output = S> + std::ops::Div<Output = S>>
    ParameterTable<S>
{
    pub fn from_rows(
        name: impl Into<String>,
        rows: impl IntoIterator<Item = (IndexKey, S)>,
        reducer: DuplicateReducer,
    ) -> Self {
        let mut values = BTreeMap::new();
        let mut counts: HashMap<IndexKey, u32> = HashMap::new();
        for (key, value) in rows {
            counts
                .entry(key.clone())
                .and_modify(|count| *count += 1)
                .or_insert(1);
            values
                .entry(key)
                .and_modify(|current| {
                    *current = match reducer {
                        DuplicateReducer::Sum
                        | DuplicateReducer::Mean
                        | DuplicateReducer::Count => *current + value,
                        DuplicateReducer::Min => {
                            if value < *current {
                                value
                            } else {
                                *current
                            }
                        }
                        DuplicateReducer::Max => {
                            if value > *current {
                                value
                            } else {
                                *current
                            }
                        }
                    };
                })
                .or_insert(value);
        }
        if reducer == DuplicateReducer::Count {
            for (key, value) in &mut values {
                *value = S::from(*counts.get(key).unwrap_or(&1));
            }
        } else if reducer == DuplicateReducer::Mean {
            for (key, value) in &mut values {
                *value = *value / S::from(*counts.get(key).unwrap_or(&1));
            }
        }
        Self {
            name: name.into(),
            storage: ParameterStorage::Sparse(values),
        }
    }
}

impl<S> ParameterTable<S> {
    pub fn from_dense(
        name: impl Into<String>,
        shape: Vec<usize>,
        values: Vec<S>,
    ) -> Result<Self, ParameterTableError> {
        let expected = shape.iter().product();
        let actual = values.len();
        if expected != actual {
            return Err(ParameterTableError::ShapeValueCountMismatch { expected, actual });
        }
        Ok(Self {
            name: name.into(),
            storage: ParameterStorage::Dense { shape, values },
        })
    }

    pub fn get(&self, key: &IndexKey) -> Option<&S> {
        match &self.storage {
            ParameterStorage::Sparse(values) => values.get(key),
            ParameterStorage::Dense { shape, values } => {
                let offset = dense_offset(shape, key)?;
                values.get(offset)
            }
        }
    }

    pub fn rows(&self) -> Vec<(IndexKey, &S)> {
        match &self.storage {
            ParameterStorage::Sparse(values) => values
                .iter()
                .map(|(key, value)| (key.clone(), value))
                .collect(),
            ParameterStorage::Dense { shape, values } => {
                dense_keys(shape).into_iter().zip(values.iter()).collect()
            }
        }
    }

    pub fn materialize(&self, domain: &Domain) -> Vec<(IndexKey, &S)> {
        domain
            .keys()
            .iter()
            .filter_map(|key| self.get(key).map(|value| (key.clone(), value)))
            .collect()
    }

    pub fn filter_keys(&self, mut keep: impl FnMut(&IndexKey) -> bool) -> Self
    where
        S: Clone,
    {
        let values = self
            .rows()
            .into_iter()
            .filter(|(key, _)| keep(key))
            .map(|(key, value)| (key, value.clone()))
            .collect();
        Self {
            name: self.name.clone(),
            storage: ParameterStorage::Sparse(values),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn storage(&self) -> &ParameterStorage<S> {
        &self.storage
    }
}

fn dense_offset(shape: &[usize], key: &IndexKey) -> Option<usize> {
    if shape.len() != key.0.len() {
        return None;
    }
    let mut offset = 0usize;
    let mut stride = 1usize;
    for (dimension, value) in shape.iter().rev().zip(key.0.iter().rev()) {
        let IndexValue::Integer(index) = value else {
            return None;
        };
        let index = usize::try_from(*index).ok()?;
        if index >= *dimension {
            return None;
        }
        offset += index * stride;
        stride *= *dimension;
    }
    Some(offset)
}

fn dense_keys(shape: &[usize]) -> Vec<IndexKey> {
    let count = shape.iter().product();
    let mut keys = Vec::with_capacity(count);
    for flat_index in 0..count {
        let mut remainder = flat_index;
        let mut values = Vec::with_capacity(shape.len());
        for stride in dense_strides(shape) {
            let index = remainder / stride;
            remainder %= stride;
            values.push(IndexValue::Integer(index as i64));
        }
        keys.push(IndexKey(values));
    }
    keys
}

fn dense_strides(shape: &[usize]) -> Vec<usize> {
    let mut strides = Vec::with_capacity(shape.len());
    let mut stride = 1usize;
    for dimension in shape.iter().rev() {
        strides.push(stride);
        stride *= *dimension;
    }
    strides.reverse();
    strides
}

/// Non-numeric attribute table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeTable {
    name: String,
    values: BTreeMap<IndexKey, String>,
}

impl AttributeTable {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            values: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, key: IndexKey, value: impl Into<String>) {
        self.values.insert(key, value.into());
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get(&self, key: &IndexKey) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    pub fn rows(&self) -> Vec<(IndexKey, &String)> {
        self.values
            .iter()
            .map(|(key, value)| (key.clone(), value))
            .collect()
    }
}

/// Shared indexed-data container with simple value pooling.
#[derive(Debug, Clone, Default)]
pub struct IndexedData<S = f64> {
    pub sets: BTreeMap<String, Set>,
    pub tuple_sets: BTreeMap<String, TupleSet>,
    pub domains: BTreeMap<String, Domain>,
    pub parameters: BTreeMap<String, ParameterTable<S>>,
    pub attributes: BTreeMap<String, AttributeTable>,
    string_pool: BTreeSet<String>,
}

impl<S> IndexedData<S> {
    pub fn intern_string(&mut self, value: impl Into<String>) -> String {
        let value = value.into();
        self.string_pool.insert(value.clone());
        value
    }

    pub fn string_pool_len(&self) -> usize {
        self.string_pool.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::indexed::{
        Domain, DuplicateReducer, IndexKey, IndexValue, ParameterTable, ParameterTableError,
        TupleSet,
    };

    #[test]
    fn duplicate_reducers_apply_to_numeric_rows() {
        let key = IndexKey(vec![IndexValue::String("a".to_string())]);
        let table = ParameterTable::<f64>::from_rows(
            "p",
            [(key.clone(), 1.0), (key.clone(), 3.0)],
            DuplicateReducer::Mean,
        );
        assert_eq!(table.get(&key), Some(&2.0));
    }

    #[test]
    fn tuple_set_rejects_wrong_arity() {
        let mut set = TupleSet::new("pairs", 2);
        assert!(!set.insert(IndexKey(vec![IndexValue::Integer(1)])));
    }

    #[test]
    fn dense_parameter_tables_lookup_materialize_and_filter() {
        let table = ParameterTable::from_dense("p", vec![2, 2], vec![1.0, 2.0, 3.0, 4.0]).unwrap();
        let key = IndexKey(vec![IndexValue::Integer(1), IndexValue::Integer(0)]);
        assert_eq!(table.get(&key), Some(&3.0));

        let domain = Domain::new(
            "subset",
            vec![
                IndexKey(vec![IndexValue::Integer(0), IndexValue::Integer(1)]),
                key.clone(),
            ],
        );
        assert_eq!(
            table.materialize(&domain),
            vec![
                (
                    IndexKey(vec![IndexValue::Integer(0), IndexValue::Integer(1)]),
                    &2.0,
                ),
                (key.clone(), &3.0),
            ]
        );

        let filtered =
            table.filter_keys(|row_key| row_key.0.first() == Some(&IndexValue::Integer(1)));
        assert_eq!(filtered.get(&key), Some(&3.0));
        assert_eq!(
            filtered.get(&IndexKey(vec![
                IndexValue::Integer(0),
                IndexValue::Integer(1),
            ])),
            None
        );
    }

    #[test]
    fn dense_parameter_table_rejects_shape_value_mismatch() {
        assert_eq!(
            ParameterTable::<f64>::from_dense("p", vec![2, 2], vec![1.0]).unwrap_err(),
            ParameterTableError::ShapeValueCountMismatch {
                expected: 4,
                actual: 1,
            }
        );
    }
}
