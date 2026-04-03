use crate::model::Model;

#[derive(Debug, Clone, PartialEq)]
pub struct CscMatrix {
    pub col_ptrs: Vec<usize>,
    pub row_indices: Vec<u32>,
    pub values: Vec<f64>,
    pub shape: (usize, usize),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CrsMatrix {
    pub row_ptrs: Vec<usize>,
    pub col_indices: Vec<u32>,
    pub values: Vec<f64>,
    pub shape: (usize, usize),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CooMatrix {
    pub rows: Vec<u32>,
    pub cols: Vec<u32>,
    pub values: Vec<f64>,
    pub shape: (usize, usize),
}

pub trait SparseMatrixExport {
    fn export_csc(&self) -> CscMatrix;
    fn export_crs(&self) -> CrsMatrix;
    fn export_coo(&self) -> CooMatrix;
}

impl Model {
    #[inline]
    fn sparse_shape(&self) -> (usize, usize) {
        (self.num_constraints(), self.num_variables())
    }

    #[inline]
    fn sparse_nnz(&self) -> usize {
        self.num_coefficients()
    }
}

impl SparseMatrixExport for Model {
    fn export_csc(&self) -> CscMatrix {
        let shape = self.sparse_shape();
        let nnz = self.sparse_nnz();

        let mut col_ptrs = Vec::with_capacity(shape.1 + 1);
        let mut row_indices = Vec::with_capacity(nnz);
        let mut values = Vec::with_capacity(nnz);

        col_ptrs.push(0);
        for (_var_id, column) in self.columns() {
            for (constraint_id, value) in column {
                row_indices.push(constraint_id.inner());
                values.push(*value);
            }
            col_ptrs.push(row_indices.len());
        }

        CscMatrix {
            col_ptrs,
            row_indices,
            values,
            shape,
        }
    }

    /// Export to CRS format with single-pass algorithm.
    /// Reduces passes from 3 to 1 and eliminates zeroed allocations.
    fn export_crs(&self) -> CrsMatrix {
        let shape = self.sparse_shape();
        let nnz = self.sparse_nnz();

        // Pre-allocate row storage without zeroing
        let mut row_entries: Vec<Vec<(u32, f64)>> = (0..shape.0)
            .map(|_| Vec::with_capacity(nnz / shape.0 + 1))
            .collect();

        // SINGLE PASS: Accumulate entries by row
        for (var_id, column) in self.columns() {
            for (constraint_id, value) in column {
                let row = constraint_id.inner() as usize;
                row_entries[row].push((var_id.inner(), *value));
            }
        }

        // Flatten to CRS format
        let mut row_ptrs = Vec::with_capacity(shape.0 + 1);
        let mut col_indices = Vec::with_capacity(nnz);
        let mut values = Vec::with_capacity(nnz);

        row_ptrs.push(0);
        for row in row_entries {
            col_indices.extend(row.iter().map(|(c, _)| *c));
            values.extend(row.iter().map(|(_, v)| *v));
            row_ptrs.push(col_indices.len());
        }

        CrsMatrix {
            row_ptrs,
            col_indices,
            values,
            shape,
        }
    }

    fn export_coo(&self) -> CooMatrix {
        let shape = self.sparse_shape();
        let nnz = self.sparse_nnz();

        let mut rows = Vec::with_capacity(nnz);
        let mut cols = Vec::with_capacity(nnz);
        let mut values = Vec::with_capacity(nnz);

        for (var_id, column) in self.columns() {
            for (constraint_id, value) in column {
                rows.push(constraint_id.inner());
                cols.push(var_id.inner());
                values.push(*value);
            }
        }

        CooMatrix {
            rows,
            cols,
            values,
            shape,
        }
    }
}
