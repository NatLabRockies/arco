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

    /// Export to CRS format with optimized 2-pass algorithm.
    /// Pass 1: count entries per row
    /// Pass 2: fill flat arrays directly (no per-row Vec allocations)
    fn export_crs(&self) -> CrsMatrix {
        let shape = self.sparse_shape();
        let nnz = self.sparse_nnz();
        let num_rows = shape.0;

        // PASS 1: Count entries per row
        let mut row_counts = vec![0usize; num_rows];
        for (_var_id, column) in self.columns() {
            for (constraint_id, _value) in column {
                let row = constraint_id.inner() as usize;
                row_counts[row] += 1;
            }
        }

        // Prefix sum: convert counts to row pointers
        let mut row_ptrs = Vec::with_capacity(num_rows + 1);
        row_ptrs.push(0);
        let mut offset = 0;
        for count in row_counts {
            offset += count;
            row_ptrs.push(offset);
        }

        // PASS 2: Fill col_indices and values directly
        // Track write positions per row (reuse row_counts as write positions)
        let mut row_write_pos: Vec<usize> = row_ptrs[..num_rows].to_vec();
        let mut col_indices = vec![0u32; nnz];
        let mut values = vec![0.0f64; nnz];

        for (var_id, column) in self.columns() {
            for (constraint_id, value) in column {
                let row = constraint_id.inner() as usize;
                let write_idx = row_write_pos[row];
                col_indices[write_idx] = var_id.inner();
                values[write_idx] = *value;
                row_write_pos[row] += 1;
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
