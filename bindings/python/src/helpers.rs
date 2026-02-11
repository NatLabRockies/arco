//! Helper functions for buffer extraction and type conversion.

use crate::errors::{CscContiguityError, CscDimensionError, CscDtypeError, CscNegativeIndexError};
use pyo3::buffer::PyBuffer;
use pyo3::prelude::*;

/// Extract a 1D buffer from a Python object.
pub fn extract_buffer_1d<T>(obj: &Bound<'_, PyAny>, name: &str, dtype: &str) -> PyResult<Vec<T>>
where
    T: pyo3::buffer::Element + Copy,
{
    let buffer = PyBuffer::<T>::get(obj).map_err(|_| {
        CscDtypeError::new_err(format!("{name} must be a numpy array with dtype {dtype}"))
    })?;
    if buffer.dimensions() != 1 {
        return Err(CscDimensionError::new_err(format!(
            "{name} must be a 1D array"
        )));
    }
    if !buffer.is_c_contiguous() {
        return Err(CscContiguityError::new_err(format!(
            "{name} must be a contiguous array"
        )));
    }
    let slice = buffer
        .as_slice(obj.py())
        .ok_or_else(|| CscContiguityError::new_err("array is not contiguous"))?;
    Ok(slice.iter().map(|cell| cell.get()).collect())
}

/// Extract indices (i32 -> usize) from a numpy buffer.
pub fn extract_indices(obj: &Bound<'_, PyAny>, name: &str) -> PyResult<Vec<usize>> {
    let values = extract_buffer_1d::<i32>(obj, name, "int32")?;
    let mut indices = Vec::with_capacity(values.len());
    for value in values {
        if value < 0 {
            return Err(CscNegativeIndexError::new_err(format!(
                "{name} entries must be non-negative"
            )));
        }
        indices.push(value as usize);
    }
    Ok(indices)
}

/// Extract f32 values from a numpy buffer.
pub fn extract_f32(obj: &Bound<'_, PyAny>, name: &str) -> PyResult<Vec<f32>> {
    extract_buffer_1d::<f32>(obj, name, "float32")
}

/// Extract boolean values from a Python object.
pub fn extract_bool(obj: &Bound<'_, PyAny>, name: &str) -> PyResult<Vec<bool>> {
    obj.extract()
        .map_err(|_| CscDtypeError::new_err(format!("{name} must be a boolean array")))
}
