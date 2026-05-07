//! Primitive indexed data: sets, domains, parameter tables, and attributes.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::hash::Hash;

/// Atomic index values supported by primitive documents v1.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IndexValue {
    String(String),
    Integer(i64),
    Decimal(String),
    Bool(bool),
}

/// Ordered key into a domain or table.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

/// Sparse numeric parameter table.
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterTable<S = f64> {
    name: String,
    values: BTreeMap<IndexKey, S>,
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
            values,
        }
    }
}

impl<S> ParameterTable<S> {
    pub fn get(&self, key: &IndexKey) -> Option<&S> {
        self.values.get(key)
    }

    pub fn name(&self) -> &str {
        &self.name
    }
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
    use crate::indexed::{DuplicateReducer, IndexKey, IndexValue, ParameterTable, TupleSet};

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
}
