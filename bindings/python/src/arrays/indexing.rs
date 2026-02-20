use pyo3::prelude::*;
use pyo3::types::PySlice;

use crate::errors::{ArrayIndexError, ArrayShapeMismatchError, ArrayTypeError};
use crate::index_set::{IndexMember, PyIndexSet};

/// Resolved axis index: either a single index or a range of indices.
pub(super) enum AxisIndex {
    Single(usize),
    Range(Vec<usize>),
}

/// Resolve a Python index (int or slice) to an AxisIndex for one dimension.
pub(super) fn resolve_axis_index(index: &Bound<'_, PyAny>, dim_size: usize) -> PyResult<AxisIndex> {
    if let Ok(idx) = index.extract::<isize>() {
        let resolved = if idx < 0 {
            (dim_size as isize + idx) as usize
        } else {
            idx as usize
        };
        if resolved >= dim_size {
            return Err(ArrayIndexError::new_err(format!(
                "index {} out of range for dimension of size {}",
                idx, dim_size
            )));
        }
        return Ok(AxisIndex::Single(resolved));
    }

    if let Ok(slice) = index.cast::<PySlice>() {
        return Ok(AxisIndex::Range(slice_indices(slice, dim_size)?));
    }

    Err(ArrayTypeError::new_err(
        "tuple index components must be integers or slices",
    ))
}

pub(super) fn slice_indices(slice: &Bound<'_, PySlice>, len: usize) -> PyResult<Vec<usize>> {
    let indices = slice.indices(len as isize)?;
    let mut result = Vec::new();
    let mut i = indices.start;
    while (indices.step > 0 && i < indices.stop) || (indices.step < 0 && i > indices.stop) {
        result.push(i as usize);
        i += indices.step;
    }
    Ok(result)
}

pub(super) fn maybe_boolean_mask_indices(
    py: Python<'_>,
    index: &Bound<'_, PyAny>,
    expected_len: usize,
) -> PyResult<Option<Vec<usize>>> {
    let np = py.import("numpy")?;
    let ndarray_type = np.getattr("ndarray")?;
    if !index.is_instance(&ndarray_type)? {
        return Ok(None);
    }

    let dtype = index.getattr("dtype")?;
    let kind: String = dtype.getattr("kind")?.extract()?;
    if kind != "b" {
        return Ok(None);
    }

    let flat_mask: Vec<bool> = index.call_method0("flatten")?.extract()?;
    if flat_mask.len() != expected_len {
        return Err(ArrayShapeMismatchError::new_err(format!(
            "boolean mask length {} does not match array length {}",
            flat_mask.len(),
            expected_len
        )));
    }

    Ok(Some(
        flat_mask
            .into_iter()
            .enumerate()
            .filter_map(|(idx, include)| include.then_some(idx))
            .collect(),
    ))
}

pub(super) fn sliced_2d_index_sets(
    py: Python<'_>,
    index_sets: &[Py<PyIndexSet>],
    nrows: usize,
    ncols: usize,
    row_indices: &[usize],
    col_indices: &[usize],
) -> PyResult<Vec<Py<PyIndexSet>>> {
    if index_sets.len() != 2 {
        return Ok(Vec::new());
    }

    let row_set = if row_indices.len() == nrows {
        index_sets[0].clone_ref(py)
    } else {
        make_slice_index_set(py, row_indices.len())?
    };

    let col_set = if col_indices.len() == ncols {
        index_sets[1].clone_ref(py)
    } else {
        make_slice_index_set(py, col_indices.len())?
    };

    Ok(vec![row_set, col_set])
}

fn make_slice_index_set(py: Python<'_>, size: usize) -> PyResult<Py<PyIndexSet>> {
    Py::new(
        py,
        PyIndexSet {
            name: format!("_slice_{}", size),
            members: (0..size).map(|i| IndexMember::Int(i as i64)).collect(),
        },
    )
}
