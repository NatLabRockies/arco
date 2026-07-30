//! Helper functions for buffer extraction and type conversion.

use crate::py_modules::errors::{
    CscContiguityError, CscDimensionError, CscDtypeError, CscNegativeIndexError,
};
use pyo3::buffer::PyBuffer;
use pyo3::prelude::*;

/// Extract indices (i32 -> usize) from a numpy buffer.
pub(crate) fn extract_indices(obj: &Bound<'_, PyAny>, name: &str) -> PyResult<Vec<usize>> {
    let values = if let Ok(buffer) = PyBuffer::<i32>::get(obj) {
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
        slice.iter().map(|cell| cell.get()).collect()
    } else {
        let sequence: Vec<i64> = obj.extract().map_err(|_| {
            CscDtypeError::new_err(format!(
                "{name} must be a 1D int sequence or a numpy array with dtype int32"
            ))
        })?;
        let mut converted = Vec::with_capacity(sequence.len());
        for value in sequence {
            let value = i32::try_from(value).map_err(|_| {
                CscDtypeError::new_err(format!("{name} entries must fit within int32 range"))
            })?;
            converted.push(value);
        }
        converted
    };
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

/// Extract f64 values from a numpy buffer or Python sequence.
pub(crate) fn extract_f64(obj: &Bound<'_, PyAny>, name: &str) -> PyResult<Vec<f64>> {
    if let Ok(buffer) = PyBuffer::<f64>::get(obj) {
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
    } else if let Ok(buffer) = PyBuffer::<f32>::get(obj) {
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
        Ok(slice.iter().map(|cell| f64::from(cell.get())).collect())
    } else {
        let sequence: Vec<f64> = obj.extract().map_err(|_| {
            CscDtypeError::new_err(format!(
                "{name} must be a 1D float sequence or a numpy array with dtype float64 or float32"
            ))
        })?;
        Ok(sequence)
    }
}

/// Extract boolean values from a Python object.
pub(crate) fn extract_bool(obj: &Bound<'_, PyAny>, name: &str) -> PyResult<Vec<bool>> {
    obj.extract()
        .map_err(|_| CscDtypeError::new_err(format!("{name} must be a boolean array")))
}
