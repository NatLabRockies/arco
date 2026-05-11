//! DataFrame-as-domain variable creation for sparse modeling
//!
//! Implements the pattern: `model.add_variables(dataframe, bounds=...)`
//! where each row of the DataFrame becomes a variable with named indices.

use crate::{PyIndexSet, pym};
use pyo3::prelude::*;

/// Extract column names from a DataFrame (polars or pandas)
pub fn get_dataframe_columns(df: &Bound<'_, PyAny>) -> PyResult<Vec<String>> {
    // Try polars first
    if let Ok(columns) = df.call_method0("columns") {
        return columns.extract::<Vec<String>>();
    }
    // Fall back to pandas
    if let Ok(columns) = df.getattr("columns") {
        return columns.extract::<Vec<String>>();
    }
    Err(pym::errors::IndexSetTypeError::new_err(
        "expected DataFrame with columns attribute (polars or pandas)",
    ))
}

/// Get number of rows in a DataFrame
pub fn get_dataframe_len(df: &Bound<'_, PyAny>) -> PyResult<usize> {
    if let Ok(height) = df.call_method0("height") {
        return height.extract::<usize>();
    }
    if let Ok(len) = df.call_method0("__len__") {
        return len.extract::<usize>();
    }
    Err(pym::errors::IndexSetTypeError::new_err(
        "expected DataFrame with height or __len__ method",
    ))
}

/// Extract a column as a vector of IndexMember values
pub fn extract_column(
    df: &Bound<'_, PyAny>,
    col_name: &str,
) -> PyResult<Vec<crate::py_modules::index_set::IndexMember>> {
    use crate::py_modules::index_set::IndexMember;

    let col = df.call_method1("__getitem__", (col_name,))?;
    let py = df.py();
    let np = py.import("numpy")?;

    // Convert to numpy array then extract
    let arr = np.call_method1("asarray", (&col,))?;
    let flat = arr.call_method0("flatten")?;

    // Try to extract based on dtype
    if let Ok(values) = flat.extract::<Vec<i64>>() {
        return Ok(values.into_iter().map(IndexMember::Int).collect());
    }
    if let Ok(values) = flat.extract::<Vec<f64>>() {
        return Ok(values.into_iter().map(IndexMember::Float).collect());
    }
    if let Ok(values) = flat.extract::<Vec<String>>() {
        return Ok(values.into_iter().map(IndexMember::Str).collect());
    }

    // Fall back to extracting as Python objects
    let len = flat.call_method0("__len__")?.extract::<usize>()?;
    let mut result = Vec::with_capacity(len);
    for item in flat.try_iter()? {
        let item = item?;
        if let Ok(v) = item.extract::<i64>() {
            result.push(IndexMember::Int(v));
        } else if let Ok(v) = item.extract::<f64>() {
            result.push(IndexMember::Float(v));
        } else if let Ok(v) = item.extract::<String>() {
            result.push(IndexMember::Str(v));
        } else {
            return Err(pym::errors::IndexSetTypeError::new_err(format!(
                "column '{}' contains unsupported type",
                col_name
            )));
        }
    }
    Ok(result)
}

/// Build IndexSets from DataFrame columns
pub fn extract_index_sets_from_dataframe(
    df: &Bound<'_, PyAny>,
    py: Python<'_>,
) -> PyResult<(Vec<Py<PyIndexSet>>, usize, Vec<String>)> {
    let columns = get_dataframe_columns(df)?;
    let n_rows = get_dataframe_len(df)?;

    if n_rows == 0 {
        return Err(pym::errors::IndexSetEmptyError::new_err(
            "DataFrame must have at least one row",
        ));
    }

    if columns.is_empty() {
        return Err(pym::errors::IndexSetEmptyError::new_err(
            "DataFrame must have at least one column",
        ));
    }

    let mut index_sets = Vec::with_capacity(columns.len());

    for col_name in &columns {
        let values = extract_column(df, col_name)?;
        let index_set = PyIndexSet {
            name: col_name.clone(),
            members: values,
        };
        index_sets.push(Py::new(py, index_set)?);
    }

    Ok((index_sets, n_rows, columns))
}
