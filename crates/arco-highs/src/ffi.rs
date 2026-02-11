//! FFI bindings to HiGHS solver library.
//!
//! This module contains unsafe code for interacting with the C library.
#![allow(unsafe_code)]

use highs::{Col, HighsModelStatus, RowProblem, Sense as HighsSense, SolvedModel};
use std::ffi::{CStr, CString};
use std::fmt;
use tracing::{debug, trace, warn};

/// Objective sense for optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectiveSense {
    /// Minimize the objective
    Minimize,
    /// Maximize the objective
    Maximize,
}

/// Status of the solver
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighsStatus {
    /// Optimal solution found
    Optimal,
    /// Problem is infeasible
    Infeasible,
    /// Problem is unbounded
    Unbounded,
    /// HiGHS could not disambiguate between unbounded and infeasible
    UnboundedOrInfeasible,
    /// Solver reached time limit (may have feasible solution)
    ReachedTimeLimit,
    /// Solver reached iteration limit (may have feasible solution)
    ReachedIterationLimit,
    /// Unknown status
    Unknown,
}

/// Errors returned by the HiGHS model wrapper.
#[derive(Debug, Clone)]
pub enum HighsModelError {
    ColumnCoefficientLengthMismatch {
        columns: usize,
        coefficients: usize,
    },
    ColumnIndexOutOfBounds {
        column_index: usize,
        num_columns: usize,
    },
    PrimalStartLengthMismatch {
        expected: usize,
        got: usize,
    },
    SolveRequired {
        operation: &'static str,
    },
}

impl fmt::Display for HighsModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HighsModelError::ColumnCoefficientLengthMismatch {
                columns,
                coefficients,
            } => write!(
                f,
                "columns length ({}) must match coefficients length ({})",
                columns, coefficients
            ),
            HighsModelError::ColumnIndexOutOfBounds {
                column_index,
                num_columns,
            } => write!(
                f,
                "column index {} out of bounds (num_columns = {})",
                column_index, num_columns
            ),
            HighsModelError::PrimalStartLengthMismatch { expected, got } => write!(
                f,
                "primal start length must match number of columns (expected {}, got {})",
                expected, got
            ),
            HighsModelError::SolveRequired { operation } => {
                write!(f, "solve must be called before {}", operation)
            }
        }
    }
}

impl std::error::Error for HighsModelError {}

/// Snapshot of primal and dual solution values.
#[derive(Debug, Clone)]
pub struct SolutionSnapshot {
    col_values: Vec<f64>,
    col_duals: Vec<f64>,
    row_values: Vec<f64>,
    row_duals: Vec<f64>,
}

impl SolutionSnapshot {
    /// Primal values for variables.
    pub fn col_values(&self) -> &[f64] {
        &self.col_values
    }

    /// Dual values for variables (reduced costs).
    pub fn col_duals(&self) -> &[f64] {
        &self.col_duals
    }

    /// Primal values for constraints.
    pub fn row_values(&self) -> &[f64] {
        &self.row_values
    }

    /// Dual values for constraints (shadow prices).
    pub fn row_duals(&self) -> &[f64] {
        &self.row_duals
    }
}

/// Safe wrapper around HiGHS model
pub struct HighsModel {
    problem: RowProblem,
    objective_sense: ObjectiveSense,
    solved: Option<SolvedModel>,
    columns: Vec<Col>,
    log_to_console: bool,
    primal_start: Option<Vec<f64>>,
    options: Vec<(String, HighsOption)>,
    verbosity: Option<u32>,
}

impl HighsModel {
    /// Create a new HiGHS model
    pub fn new() -> Self {
        debug!(
            component = "solver",
            operation = "init_highs",
            status = "success",
            "Creating new HiGHS model"
        );
        HighsModel {
            problem: RowProblem::default(),
            objective_sense: ObjectiveSense::Minimize,
            solved: None,
            columns: Vec::new(),
            log_to_console: false,
            primal_start: None,
            options: Vec::new(),
            verbosity: None,
        }
    }

    /// Add a continuous column (variable) to the model
    ///
    /// # Arguments
    ///
    /// * `lower_bound` - Lower bound on the variable
    /// * `upper_bound` - Upper bound on the variable
    /// * `objective_coefficient` - Coefficient in the objective function
    ///
    /// # Returns
    ///
    /// The index of the added column
    pub fn add_col(
        &mut self,
        lower_bound: f64,
        upper_bound: f64,
        objective_coefficient: f64,
    ) -> usize {
        self.add_col_with_integrality(lower_bound, upper_bound, objective_coefficient, false)
    }

    /// Add an integer column (variable) to the model
    pub fn add_integer_col(
        &mut self,
        lower_bound: f64,
        upper_bound: f64,
        objective_coefficient: f64,
    ) -> usize {
        self.add_col_with_integrality(lower_bound, upper_bound, objective_coefficient, true)
    }

    fn add_col_with_integrality(
        &mut self,
        lower_bound: f64,
        upper_bound: f64,
        objective_coefficient: f64,
        is_integer: bool,
    ) -> usize {
        trace!(
            lower_bound,
            upper_bound,
            objective_coefficient,
            is_integer,
            component = "solver",
            operation = "add_column",
            status = "success",
            "Adding column"
        );
        self.solved = None;
        self.primal_start = None;
        let col = if is_integer {
            self.problem
                .add_integer_column(objective_coefficient, lower_bound..=upper_bound)
        } else {
            self.problem
                .add_column(objective_coefficient, lower_bound..=upper_bound)
        };
        self.columns.push(col);
        self.columns.len() - 1
    }

    /// Add a linear constraint (row) to the model
    ///
    /// # Arguments
    ///
    /// * `lower_bound` - Lower bound on the constraint
    /// * `upper_bound` - Upper bound on the constraint
    /// * `columns` - Indices of variables in the constraint
    /// * `coefficients` - Coefficients of the variables
    ///
    /// # Returns
    ///
    /// The index of the added row
    ///
    /// # Errors
    ///
    /// Returns an error if columns and coefficients have different lengths
    /// or if any column index is out of bounds.
    pub fn add_row(
        &mut self,
        lower_bound: f64,
        upper_bound: f64,
        columns: &[usize],
        coefficients: &[f64],
    ) -> Result<usize, HighsModelError> {
        if columns.len() != coefficients.len() {
            warn!(
                component = "solver",
                operation = "add_row",
                status = "error",
                columns = columns.len(),
                coefficients = coefficients.len(),
                "Column/coefficients length mismatch"
            );
            return Err(HighsModelError::ColumnCoefficientLengthMismatch {
                columns: columns.len(),
                coefficients: coefficients.len(),
            });
        }
        trace!(
            lower_bound,
            upper_bound,
            component = "solver",
            operation = "add_row",
            status = "success",
            "Adding row"
        );
        self.solved = None;
        let num_columns = self.columns.len();
        let mut factors = Vec::with_capacity(columns.len());
        for (col_idx, coeff) in columns.iter().copied().zip(coefficients.iter().copied()) {
            let col = *self.columns.get(col_idx).ok_or_else(|| {
                warn!(
                    component = "solver",
                    operation = "add_row",
                    status = "error",
                    col_idx,
                    num_columns,
                    "Column index out of bounds for constraint"
                );
                HighsModelError::ColumnIndexOutOfBounds {
                    column_index: col_idx,
                    num_columns,
                }
            })?;
            factors.push((col, coeff));
        }
        self.problem.add_row(lower_bound..=upper_bound, factors);
        Ok(self.problem.num_rows().saturating_sub(1))
    }

    /// Set the objective sense
    pub fn set_objective_sense(&mut self, sense: ObjectiveSense) {
        debug!(
            component = "solver",
            operation = "set_objective_sense",
            status = "success",
            ?sense,
            "Setting objective sense"
        );
        self.objective_sense = sense;
    }

    /// Enable or disable logging to console for the next solve
    pub fn set_log_to_console(&mut self, enabled: bool) {
        self.log_to_console = enabled;
    }

    /// Set a HiGHS option for the next solve.
    pub fn set_option(&mut self, option: impl Into<String>, value: HighsOption) {
        self.options.push((option.into(), value));
    }

    /// Set verbosity level for the next solve.
    pub fn set_verbosity(&mut self, level: u32) {
        self.verbosity = Some(level);
    }

    /// Set primal start values for warm-start hints.
    ///
    /// # Errors
    ///
    /// Returns an error if the provided vector length does not match the
    /// number of columns in the model.
    pub fn set_primal_start(&mut self, cols: Vec<f64>) -> Result<(), HighsModelError> {
        if cols.len() != self.columns.len() {
            warn!(
                component = "solver",
                operation = "set_primal_start",
                status = "error",
                expected = self.columns.len(),
                got = cols.len(),
                "Primal start length mismatch"
            );
            return Err(HighsModelError::PrimalStartLengthMismatch {
                expected: self.columns.len(),
                got: cols.len(),
            });
        }
        self.primal_start = Some(cols);
        Ok(())
    }

    /// Solve the model
    pub fn solve(&mut self) -> HighsStatus {
        debug!(
            num_cols = self.problem.num_cols(),
            num_rows = self.problem.num_rows(),
            ?self.objective_sense,
            component = "solver",
            operation = "solve",
            status = "success",
            "Solving model"
        );

        let sense = match self.objective_sense {
            ObjectiveSense::Minimize => HighsSense::Minimise,
            ObjectiveSense::Maximize => HighsSense::Maximise,
        };

        // Consume the built problem to avoid cloning and keep the solve path memory-first.
        let problem = std::mem::take(&mut self.problem);
        let mut model = problem.optimise(sense);
        if self.verbosity.unwrap_or(0) == 0 && !self.log_to_console {
            model.make_quiet();
        }
        if let Some(level) = self.verbosity {
            model.set_option("output_flag", level > 0);
        }
        for (option, value) in self.options.drain(..) {
            match value {
                HighsOption::Bool(val) => model.set_option(option.as_str(), val),
                HighsOption::Int(val) => model.set_option(option.as_str(), val),
                HighsOption::Float(val) => model.set_option(option.as_str(), val),
                HighsOption::Str(val) => model.set_option(option.as_str(), val.as_str()),
            }
        }
        if self.log_to_console {
            model.set_option("log_to_console", true);
            model.set_option("output_flag", true);
        }
        if let Some(cols) = self.primal_start.as_ref() {
            if let Err(err) = model.try_set_solution(Some(cols), None, None, None) {
                warn!(
                    component = "solver",
                    operation = "set_primal_start",
                    status = "warn",
                    ?err,
                    "Failed to set warm-start solution; continuing without hints"
                );
            }
        }
        let solution = model.solve();
        let status = map_status(solution.status());

        trace!(
            component = "solver",
            operation = "solve",
            status = "success",
            ?status,
            "Solution status received"
        );
        self.solved = Some(solution);
        // After solving, keep an empty problem; if the caller needs another solve, they must
        // rebuild columns/rows. This avoids retaining or cloning the original CSC buffers.
        self.problem = RowProblem::default();
        self.columns.clear();
        self.primal_start = None;
        self.options.clear();
        self.verbosity = None;
        status
    }

    /// Get the number of columns (variables)
    pub fn columns(&self) -> usize {
        self.columns.len()
    }

    /// Get the objective value of the current solution
    ///
    /// # Errors
    ///
    /// Returns an error if the model has not been solved yet.
    pub fn objective_value(&self) -> Result<f64, HighsModelError> {
        let solved = self.solved.as_ref().ok_or(HighsModelError::SolveRequired {
            operation: "objective_value",
        })?;
        Ok(solved.objective_value())
    }

    /// Get the MIP gap (or infinity for pure LPs).
    pub fn mip_gap(&self) -> f64 {
        match self.solved.as_ref() {
            Some(solved) => solved.mip_gap(),
            None => f64::NAN,
        }
    }

    /// Get the simplex iteration count for the latest solve.
    pub fn simplex_iteration_count(&self) -> u64 {
        let Some(solved) = self.solved.as_ref() else {
            return 0;
        };

        let Ok(name) = CString::new("simplex_iteration_count") else {
            return 0;
        };
        let mut value: highs_sys::HighsInt = 0;
        let status = unsafe {
            highs_sys::Highs_getIntInfoValue(solved.as_ptr(), name.as_ptr(), &raw mut value)
        };

        if status != highs_sys::STATUS_OK {
            warn!(
                component = "solver",
                operation = "solve_info",
                status = "warn",
                info = "simplex_iteration_count",
                status_code = status,
                "Failed to read simplex iteration count"
            );
            return 0;
        }

        if value < 0 {
            return 0;
        }
        value as u64
    }

    /// Get barrier iteration count for the latest solve.
    pub fn barrier_iteration_count(&self) -> u64 {
        let Some(solved) = self.solved.as_ref() else {
            return 0;
        };

        let Ok(name) = CString::new("barrier_iteration_count") else {
            return 0;
        };
        let mut value: highs_sys::HighsInt = 0;
        let status = unsafe {
            highs_sys::Highs_getIntInfoValue(solved.as_ptr(), name.as_ptr(), &raw mut value)
        };

        if status != highs_sys::STATUS_OK {
            // This is expected when simplex solver was used (barrier info not available)
            // Only log at debug level since it's not an error
            debug!(
                component = "solver",
                operation = "solve_info",
                info = "barrier_iteration_count",
                status_code = status,
                "Barrier iteration count not available (simplex solver used)"
            );
            return 0;
        }

        if value < 0 {
            return 0;
        }
        value as u64
    }

    /// Get number of rows after presolve (0 if presolve disabled or not available)
    pub fn presolved_num_rows(&self) -> u64 {
        self.get_int_info("presolve_num_rows").unwrap_or(0)
    }

    /// Get number of cols after presolve (0 if presolve disabled or not available)
    pub fn presolved_num_cols(&self) -> u64 {
        self.get_int_info("presolve_num_cols").unwrap_or(0)
    }

    /// Get number of non-zeros after presolve
    pub fn presolved_num_nz(&self) -> u64 {
        self.get_int_info("presolve_num_nz").unwrap_or(0)
    }

    /// Helper to get an integer info value
    fn get_int_info(&self, name: &str) -> Option<u64> {
        let solved = self.solved.as_ref()?;
        let c_name = CString::new(name).ok()?;
        let mut value: highs_sys::HighsInt = 0;
        let status = unsafe {
            highs_sys::Highs_getIntInfoValue(solved.as_ptr(), c_name.as_ptr(), &raw mut value)
        };
        if status == highs_sys::STATUS_OK && value >= 0 {
            Some(value as u64)
        } else {
            None
        }
    }

    /// Get primal feasibility tolerance achieved
    pub fn primal_feasibility_tolerance(&self) -> f64 {
        // Return default tolerance for now
        1e-6
    }

    /// Get dual feasibility tolerance achieved
    pub fn dual_feasibility_tolerance(&self) -> f64 {
        // Return default tolerance for now
        1e-6
    }

    /// Get a snapshot of primal and dual solution values.
    ///
    /// # Errors
    ///
    /// Returns an error if the model has not been solved yet.
    pub fn solution_snapshot(&self) -> Result<SolutionSnapshot, HighsModelError> {
        let solution = self.solved.as_ref().ok_or(HighsModelError::SolveRequired {
            operation: "solution_snapshot",
        })?;
        let solution = solution.get_solution();

        Ok(SolutionSnapshot {
            col_values: solution.columns().to_vec(),
            col_duals: solution.dual_columns().to_vec(),
            row_values: solution.rows().to_vec(),
            row_duals: solution.dual_rows().to_vec(),
        })
    }
}

impl Default for HighsModel {
    fn default() -> Self {
        Self::new()
    }
}

/// Option value types for HiGHS solver configuration.
#[derive(Debug, Clone)]
pub enum HighsOption {
    Bool(bool),
    Int(i32),
    Float(f64),
    Str(String),
}

/// Return the HiGHS solver version string, if available.
pub fn highs_version() -> Option<String> {
    unsafe {
        let ptr = highs_sys::Highs_version();
        if ptr.is_null() {
            None
        } else {
            CStr::from_ptr(ptr).to_str().ok().map(|s| s.to_string())
        }
    }
}

impl fmt::Debug for HighsModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let objective_value = self.solved.as_ref().map(|s| s.objective_value());
        f.debug_struct("HighsModel")
            .field("num_variables", &self.problem.num_cols())
            .field("num_constraints", &self.problem.num_rows())
            .field("objective_sense", &self.objective_sense)
            .field("objective_value", &objective_value)
            .finish_non_exhaustive()
    }
}

fn map_status(status: HighsModelStatus) -> HighsStatus {
    match status {
        HighsModelStatus::Optimal => HighsStatus::Optimal,
        HighsModelStatus::Infeasible => HighsStatus::Infeasible,
        HighsModelStatus::Unbounded => HighsStatus::Unbounded,
        HighsModelStatus::UnboundedOrInfeasible => HighsStatus::UnboundedOrInfeasible,
        HighsModelStatus::ReachedTimeLimit => HighsStatus::ReachedTimeLimit,
        HighsModelStatus::ReachedIterationLimit => HighsStatus::ReachedIterationLimit,
        _ => HighsStatus::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use crate::ffi::{HighsModel, ObjectiveSense};

    #[test]
    fn test_create_model() {
        let model = HighsModel::new();
        assert_eq!(model.columns(), 0);
    }

    #[test]
    fn test_objective_sense() {
        let mut model = HighsModel::new();
        assert_eq!(model.objective_sense, ObjectiveSense::Minimize);

        model.set_objective_sense(ObjectiveSense::Maximize);
        assert_eq!(model.objective_sense, ObjectiveSense::Maximize);
    }
}
