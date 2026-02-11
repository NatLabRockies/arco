//! Async CRS (Compressed Row Storage) matrix building for coefficient matrices.
//!
//! This module provides async-aware functions for building sparse constraint matrices
//! more efficiently than the sequential path. It uses optional rayon for
//! parallelization of CPU-bound work.
//!
//! ## Architecture
//!
//! The async builder partitions the work across three phases:
//! 1. **Partition**: Split columns into chunks (work items)
//! 2. **Process**: Each chunk is processed independently to aggregate coefficients
//! 3. **Merge**: Aggregated results are merged into the final CRS structure
//!
//! ```text
//! Columns [A, B, C, D, E, F, G, H]
//!    │
//!    ├─ Chunk 0: [A, B] ──┐
//!    ├─ Chunk 1: [C, D] ──┤
//!    ├─ Chunk 2: [E, F] ──┼─> Process (parallel or async)
//!    └─ Chunk 3: [G, H] ──┘
//!         │
//!         ▼
//! Constraint aggregates:
//! {C0: [(x0, 1.0), (x1, 2.0)], C1: [(x2, 3.0)], ...}
//!         │
//!         ▼
//! Final CRS (stored by constraint)
//! ```

use arco_core::Model;
use arco_expr::{ConstraintId, VariableId};
use std::collections::BTreeMap;
use std::time::Instant;
use tracing::{debug, trace};

/// Type alias for a variable-id chunk used as a work unit.
type VariableChunk = Vec<VariableId>;

/// Type alias for processed constraint entries: (column indices, coefficients)
pub(crate) type ConstraintEntries = BTreeMap<ConstraintId, (Vec<usize>, Vec<f64>)>;

/// Result of CRS matrix construction
#[derive(Clone, Debug)]
pub struct CrsMatrixResult {
    /// Mapping from constraint ID to (column indices, coefficients)
    pub constraint_entries: ConstraintEntries,
    /// Duration of the build operation in milliseconds
    pub duration_ms: f64,
}

/// Async CRS matrix builder
pub struct AsyncCrsBuilder {
    /// Number of work chunks to create
    chunk_count: usize,
    /// Whether to use parallel processing (if rayon is available)
    use_parallel: bool,
}

impl AsyncCrsBuilder {
    /// Create a new async CRS builder
    pub fn new() -> Self {
        Self {
            chunk_count: num_cpus::get(),
            use_parallel: true,
        }
    }

    /// Set the number of chunks for partitioning
    pub fn with_chunk_count(mut self, count: usize) -> Self {
        self.chunk_count = count.max(1);
        self
    }

    /// Enable or disable parallel processing
    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.use_parallel = parallel;
        self
    }

    /// Build CRS matrix from model columns (blocking async wrapper)
    ///
    /// This is the main entry point. It spawns async work to build the matrix
    /// and blocks on completion.
    pub fn build_blocking(
        &self,
        model: &Model,
        var_id_to_col: &BTreeMap<VariableId, usize>,
    ) -> CrsMatrixResult {
        let started = Instant::now();

        // Collect variable IDs into a vector for partitioning. Coefficients stay
        // in the model so large CSC buffers are not cloned.
        let variable_ids: Vec<VariableId> = model.columns().map(|(var_id, _)| var_id).collect();

        debug!(
            component = "async_matrix",
            operation = "build_crs",
            status = "start",
            num_columns = variable_ids.len(),
            chunk_count = self.chunk_count,
            use_parallel = self.use_parallel,
            "Starting async CRS matrix build"
        );

        // Partition variable IDs into chunks
        let chunks = self.partition_columns(&variable_ids);

        // Process chunks (sequentially or in parallel)
        let chunk_results = if self.use_parallel && cfg!(feature = "parallel") {
            #[cfg(feature = "parallel")]
            {
                self.process_chunks_parallel(&chunks, var_id_to_col, model)
            }
            #[cfg(not(feature = "parallel"))]
            {
                self.process_chunks_sequential(&chunks, var_id_to_col, model)
            }
        } else {
            self.process_chunks_sequential(&chunks, var_id_to_col, model)
        };

        // Merge results from all chunks
        let constraint_entries = self.merge_chunk_results(chunk_results);

        let duration_ms = started.elapsed().as_secs_f64() * 1000.0;

        debug!(
            component = "async_matrix",
            operation = "build_crs",
            status = "complete",
            num_constraints = constraint_entries.len(),
            duration_ms = duration_ms,
            "Completed async CRS matrix build"
        );

        CrsMatrixResult {
            constraint_entries,
            duration_ms,
        }
    }

    /// Partition variable IDs into work chunks
    fn partition_columns(&self, variable_ids: &[VariableId]) -> Vec<VariableChunk> {
        let chunk_size = (variable_ids.len() / self.chunk_count).max(1);
        variable_ids
            .chunks(chunk_size)
            .map(|chunk| chunk.to_vec())
            .collect()
    }

    /// Process chunks sequentially
    fn process_chunks_sequential(
        &self,
        chunks: &[VariableChunk],
        var_id_to_col: &BTreeMap<VariableId, usize>,
        model: &Model,
    ) -> Vec<ConstraintEntries> {
        chunks
            .iter()
            .enumerate()
            .map(|(idx, chunk)| {
                trace!(
                    component = "async_matrix",
                    operation = "process_chunk",
                    chunk_id = idx,
                    chunk_size = chunk.len(),
                    "Processing chunk sequentially"
                );
                self.process_single_chunk(chunk, var_id_to_col, model)
            })
            .collect()
    }

    /// Process chunks in parallel using rayon
    #[cfg(feature = "parallel")]
    fn process_chunks_parallel(
        &self,
        chunks: &[VariableChunk],
        var_id_to_col: &BTreeMap<VariableId, usize>,
        model: &Model,
    ) -> Vec<ConstraintEntries> {
        use rayon::prelude::*;

        chunks
            .par_iter()
            .enumerate()
            .map(|(idx, chunk)| {
                trace!(
                    component = "async_matrix",
                    operation = "process_chunk",
                    chunk_id = idx,
                    chunk_size = chunk.len(),
                    "Processing chunk in parallel"
                );
                self.process_single_chunk(chunk, var_id_to_col, model)
            })
            .collect()
    }

    /// Process a single chunk of variable IDs
    #[allow(clippy::unused_self)]
    fn process_single_chunk(
        &self,
        chunk: &[VariableId],
        var_id_to_col: &BTreeMap<VariableId, usize>,
        model: &Model,
    ) -> ConstraintEntries {
        let mut result: ConstraintEntries = BTreeMap::new();

        for var_id in chunk {
            // Check if variable is active
            let Ok(var) = model.get_variable(*var_id) else {
                continue;
            };
            if !var.is_active {
                continue;
            }

            // Get HiGHS column index for this variable
            if let Some(&col_idx) = var_id_to_col.get(var_id) {
                let Some(column) = model.get_column(*var_id) else {
                    continue;
                };
                // Add all coefficients from this column to their constraints
                for (constraint_id, coeff) in column.iter().copied() {
                    let entry = result
                        .entry(constraint_id)
                        .or_insert_with(|| (Vec::new(), Vec::new()));
                    entry.0.push(col_idx);
                    entry.1.push(coeff);
                }
            }
        }

        result
    }

    /// Merge chunk results into a single CRS structure
    #[allow(clippy::unused_self)]
    fn merge_chunk_results(&self, chunk_results: Vec<ConstraintEntries>) -> ConstraintEntries {
        let mut merged: ConstraintEntries = BTreeMap::new();

        for chunk_result in chunk_results {
            for (constraint_id, (mut col_indices, mut coefficients)) in chunk_result {
                let entry = merged
                    .entry(constraint_id)
                    .or_insert_with(|| (Vec::new(), Vec::new()));
                entry.0.append(&mut col_indices);
                entry.1.append(&mut coefficients);
            }
        }

        merged
    }
}

impl Default for AsyncCrsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arco_core::Variable;
    use arco_core::types::Bounds;
    use arco_expr::Expr;

    #[test]
    fn test_async_crs_builder_creation() {
        let builder = AsyncCrsBuilder::new();
        assert!(builder.chunk_count > 0);
        assert!(builder.use_parallel);
    }

    #[test]
    fn test_builder_with_config() {
        let builder = AsyncCrsBuilder::new()
            .with_chunk_count(4)
            .with_parallel(false);
        assert_eq!(builder.chunk_count, 4);
        assert!(!builder.use_parallel);
    }

    #[test]
    fn test_partition_columns() {
        let builder = AsyncCrsBuilder::new().with_chunk_count(3);
        let variable_ids = vec![
            VariableId::new(0),
            VariableId::new(1),
            VariableId::new(2),
            VariableId::new(3),
            VariableId::new(4),
        ];

        let chunks = builder.partition_columns(&variable_ids);
        assert!(!chunks.is_empty());
        // chunk_size = max(5 / 3, 1) = 1, so each variable is placed into a chunk.
        let total_len: usize = chunks.iter().map(|c| c.len()).sum();
        assert_eq!(total_len, 5);
    }

    #[test]
    fn test_build_empty_model() {
        let model = Model::new();
        let builder = AsyncCrsBuilder::new().with_chunk_count(2);
        let var_id_to_col = BTreeMap::new();

        let result = builder.build_blocking(&model, &var_id_to_col);
        assert_eq!(result.constraint_entries.len(), 0);
    }

    #[test]
    fn test_build_simple_model() {
        let mut model = Model::new();

        // Add two variables
        let var1 = model
            .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
            .unwrap();
        let var2 = model
            .add_variable(Variable::continuous(Bounds::new(0.0, 20.0)))
            .unwrap();

        // Add a constraint: 2*var1 + 3*var2 <= 100
        let constraint_id = model
            .add_expr_constraint(
                Expr::term(var1, 2.0) + Expr::term(var2, 3.0),
                Bounds::new(f64::NEG_INFINITY, 100.0),
            )
            .unwrap();

        let mut var_id_to_col = BTreeMap::new();
        var_id_to_col.insert(var1, 0);
        var_id_to_col.insert(var2, 1);

        let builder = AsyncCrsBuilder::new().with_chunk_count(2);
        let result = builder.build_blocking(&model, &var_id_to_col);

        // Should have one constraint
        assert_eq!(result.constraint_entries.len(), 1);

        // Check that constraint has correct entries
        if let Some((col_indices, coefficients)) = result.constraint_entries.get(&constraint_id) {
            // Should have exactly 2 coefficients
            assert_eq!(col_indices.len(), 2);
            assert_eq!(coefficients.len(), 2);

            // Coefficients should match (order might differ due to iteration order)
            let total_coeff: f64 = coefficients.iter().sum();
            assert!((total_coeff - 5.0).abs() < 1e-9); // 2.0 + 3.0 = 5.0
        }
    }
}
