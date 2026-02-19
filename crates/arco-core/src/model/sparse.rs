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

    fn export_crs(&self) -> CrsMatrix {
        let shape = self.sparse_shape();
        let nnz = self.sparse_nnz();

        let mut row_counts = vec![0usize; shape.0];
        for (_var_id, column) in self.columns() {
            for (constraint_id, _value) in column {
                row_counts[constraint_id.inner() as usize] += 1;
            }
        }

        let mut row_ptrs = Vec::with_capacity(shape.0 + 1);
        row_ptrs.push(0);
        for count in row_counts {
            row_ptrs.push(row_ptrs.last().copied().unwrap_or(0) + count);
        }

        let mut col_indices = vec![0u32; nnz];
        let mut values = vec![0.0; nnz];
        let mut row_cursor = row_ptrs[..shape.0].to_vec();

        for (var_id, column) in self.columns() {
            for (constraint_id, value) in column {
                let row = constraint_id.inner() as usize;
                let idx = row_cursor[row];
                col_indices[idx] = var_id.inner();
                values[idx] = *value;
                row_cursor[row] += 1;
            }
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
