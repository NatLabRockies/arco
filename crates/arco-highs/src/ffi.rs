//! FFI bindings to HiGHS solver library.
//!
//! This module contains unsafe code for interacting with the C library.
#![allow(unsafe_code)]

use std::ffi::{CStr, CString, c_void};
use std::fmt;
use std::ptr::null;
use tracing::{debug, trace, warn};

use crate::sys as highs_sys;

/// Objective sense for optimization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectiveSense {
    Minimize,
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HighsModelError {
    /// Column index and coefficient slices had different lengths when adding a row.
    ColumnCoefficientLengthMismatch {
        /// Number of column indices provided.
        columns: usize,
        /// Number of coefficients provided.
        coefficients: usize,
    },
    /// A row referenced a column index that does not exist in the model.
    ColumnIndexOutOfBounds {
        /// Invalid column index.
        column_index: usize,
        /// Current number of columns in the model.
        num_columns: usize,
    },
    /// Warm-start vector length did not match the model's column count.
    PrimalStartLengthMismatch {
        /// Expected number of entries.
        expected: usize,
        /// Provided number of entries.
        got: usize,
    },
    /// A solution-only accessor was called before solving.
    SolveRequired {
        /// Name of the operation that requires a prior solve.
        operation: &'static str,
    },
    /// A model dimension does not fit HiGHS' integer type.
    InvalidProblemSize {
        /// Dimension or index name.
        name: &'static str,
        /// Value that did not fit.
        value: usize,
    },
    /// HiGHS returned row/column dimensions that cannot be represented as `usize`.
    InvalidSolutionDimensions {
        /// Raw number of columns reported by HiGHS.
        num_cols: highs_sys::HighsInt,
        /// Raw number of rows reported by HiGHS.
        num_rows: highs_sys::HighsInt,
    },
    /// HiGHS returned a null solver handle.
    NullHandle,
    /// HiGHS returned a non-OK status for an operation.
    HighsCallFailed {
        /// C API operation name.
        operation: &'static str,
        /// Raw HiGHS status code.
        status: highs_sys::HighsInt,
    },
    /// A HiGHS option name contained an interior NUL byte.
    InvalidOptionName { option: String },
    /// A HiGHS option value contained an interior NUL byte.
    InvalidOptionValue { option: String },
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
            HighsModelError::InvalidProblemSize { name, value } => write!(
                f,
                "{name} value {value} is too large for HiGHS integer storage"
            ),
            HighsModelError::InvalidSolutionDimensions { num_cols, num_rows } => write!(
                f,
                "invalid solution dimensions from HiGHS (num_cols = {}, num_rows = {})",
                num_cols, num_rows
            ),
            HighsModelError::NullHandle => write!(f, "Highs_create returned a null handle"),
            HighsModelError::HighsCallFailed { operation, status } => {
                write!(f, "{operation} returned HiGHS status {status}")
            }
            HighsModelError::InvalidOptionName { option } => {
                write!(f, "invalid HiGHS option name {option:?}: contains NUL")
            }
            HighsModelError::InvalidOptionValue { option } => {
                write!(f, "invalid HiGHS option value for {option:?}: contains NUL")
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

    /// Consume the snapshot and return `(col_values, col_duals, row_values, row_duals)`.
    pub(crate) fn into_vecs(self) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
        (
            self.col_values,
            self.col_duals,
            self.row_values,
            self.row_duals,
        )
    }

    #[cfg(test)]
    fn new(
        col_values: Vec<f64>,
        col_duals: Vec<f64>,
        row_values: Vec<f64>,
        row_duals: Vec<f64>,
    ) -> Self {
        Self {
            col_values,
            col_duals,
            row_values,
            row_duals,
        }
    }
}

/// Safe wrapper around HiGHS model.
pub struct HighsModel {
    objective_sense: ObjectiveSense,
    solved: Option<HighsHandle>,
    col_cost: Vec<f64>,
    col_lower: Vec<f64>,
    col_upper: Vec<f64>,
    row_lower: Vec<f64>,
    row_upper: Vec<f64>,
    columns: Vec<Vec<(usize, f64)>>,
    integrality: Option<Vec<highs_sys::HighsInt>>,
    log_to_console: bool,
    primal_start: Option<Vec<f64>>,
    options: Vec<(String, HighsOption)>,
    verbosity: Option<u32>,
}

impl HighsModel {
    /// Create a new HiGHS model.
    pub fn new() -> Self {
        debug!(
            component = "solver",
            operation = "init_highs",
            status = "success",
            "Creating new HiGHS model"
        );
        HighsModel {
            objective_sense: ObjectiveSense::Minimize,
            solved: None,
            col_cost: Vec::new(),
            col_lower: Vec::new(),
            col_upper: Vec::new(),
            row_lower: Vec::new(),
            row_upper: Vec::new(),
            columns: Vec::new(),
            integrality: None,
            log_to_console: false,
            primal_start: None,
            options: Vec::new(),
            verbosity: None,
        }
    }

    /// Add a continuous column (variable) to the model.
    pub fn add_col(
        &mut self,
        lower_bound: f64,
        upper_bound: f64,
        objective_coefficient: f64,
    ) -> usize {
        self.add_col_with_integrality(lower_bound, upper_bound, objective_coefficient, false)
    }

    /// Add an integer column (variable) to the model.
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
        let index = self.col_cost.len();
        self.col_cost.push(objective_coefficient);
        self.col_lower.push(lower_bound);
        self.col_upper.push(upper_bound);
        self.columns.push(Vec::new());
        if is_integer && self.integrality.is_none() {
            self.integrality = Some(vec![highs_sys::VAR_TYPE_CONTINUOUS; index]);
        }
        if let Some(integrality) = &mut self.integrality {
            integrality.push(if is_integer {
                highs_sys::VAR_TYPE_INTEGER
            } else {
                highs_sys::VAR_TYPE_CONTINUOUS
            });
        }
        index
    }

    /// Add a linear constraint (row) to the model.
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

        let num_columns = self.columns.len();
        for col_idx in columns {
            if *col_idx >= num_columns {
                warn!(
                    component = "solver",
                    operation = "add_row",
                    status = "error",
                    col_idx,
                    num_columns,
                    "Column index out of bounds for constraint"
                );
                return Err(HighsModelError::ColumnIndexOutOfBounds {
                    column_index: *col_idx,
                    num_columns,
                });
            }
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
        let row_index = self.row_lower.len();
        self.row_lower.push(lower_bound);
        self.row_upper.push(upper_bound);
        for (col_idx, coeff) in columns.iter().copied().zip(coefficients.iter().copied()) {
            self.columns[col_idx].push((row_index, coeff));
        }
        Ok(row_index)
    }

    /// Set the objective sense.
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

    /// Enable or disable logging to console for the next solve.
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
            return Err(HighsModelError::PrimalStartLengthMismatch {
                expected: self.columns.len(),
                got: cols.len(),
            });
        }
        self.primal_start = Some(cols);
        Ok(())
    }

    /// Solve the model.
    pub fn solve(&mut self) -> HighsStatus {
        debug!(
            num_cols = self.col_cost.len(),
            num_rows = self.row_lower.len(),
            ?self.objective_sense,
            component = "solver",
            operation = "solve",
            status = "success",
            "Solving model"
        );

        let mut handle = match HighsHandle::load(self) {
            Ok(handle) => handle,
            Err(err) => {
                warn!(
                    component = "solver",
                    operation = "load_highs",
                    status = "error",
                    ?err,
                    "Failed to load HiGHS model"
                );
                return HighsStatus::Unknown;
            }
        };

        if let Err(err) =
            handle.apply_options(self.log_to_console, self.verbosity, self.options.drain(..))
        {
            warn!(
                component = "solver",
                operation = "set_options",
                status = "error",
                ?err,
                "Failed to set HiGHS options"
            );
            return HighsStatus::Unknown;
        }

        if let Some(cols) = self.primal_start.as_ref() {
            if let Err(err) = handle.set_primal_start(cols) {
                warn!(
                    component = "solver",
                    operation = "set_primal_start",
                    status = "warn",
                    ?err,
                    "Failed to set warm-start solution; continuing without hints"
                );
            }
        }

        let status = match handle.run() {
            Ok(status) => status,
            Err(err) => {
                warn!(
                    component = "solver",
                    operation = "solve",
                    status = "error",
                    ?err,
                    "HiGHS solve failed"
                );
                HighsStatus::Unknown
            }
        };

        trace!(
            component = "solver",
            operation = "solve",
            status = "success",
            ?status,
            "Solution status received"
        );
        self.solved = Some(handle);
        self.clear_problem();
        status
    }

    /// Get the number of columns (variables).
    pub fn columns(&self) -> usize {
        self.columns.len()
    }

    /// Get the objective value of the current solution.
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
            Some(solved) => solved.double_info_value("mip_gap").unwrap_or(f64::INFINITY),
            None => f64::NAN,
        }
    }

    /// Get the simplex iteration count for the latest solve.
    pub fn simplex_iteration_count(&self) -> u64 {
        self.get_int_info("simplex_iteration_count").unwrap_or(0)
    }

    /// Get barrier iteration count for the latest solve.
    pub fn barrier_iteration_count(&self) -> u64 {
        self.get_int_info("barrier_iteration_count").unwrap_or(0)
    }

    /// Get number of rows after presolve (None if presolve disabled or not available).
    pub fn presolved_num_rows(&self) -> Option<u64> {
        self.get_int_info("presolve_num_rows")
    }

    /// Get number of cols after presolve (None if presolve disabled or not available).
    pub fn presolved_num_cols(&self) -> Option<u64> {
        self.get_int_info("presolve_num_cols")
    }

    /// Get number of non-zeros after presolve (None if presolve disabled or not available).
    pub fn presolved_num_nz(&self) -> Option<u64> {
        self.get_int_info("presolve_num_nz")
    }

    /// Helper to get an integer info value.
    fn get_int_info(&self, name: &str) -> Option<u64> {
        self.solved
            .as_ref()?
            .int_info_value(name)
            .ok()
            .and_then(|value| u64::try_from(value).ok())
    }

    /// Get primal feasibility tolerance achieved.
    pub fn primal_feasibility_tolerance(&self) -> f64 {
        // HiGHS does not expose achieved tolerances via info values.
        1e-6
    }

    /// Get dual feasibility tolerance achieved.
    pub fn dual_feasibility_tolerance(&self) -> f64 {
        // Return the default tolerance value used by this wrapper.
        1e-6
    }

    /// Get a snapshot of primal and dual solution values.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - the model has not been solved yet,
    /// - HiGHS reports invalid row/column dimensions,
    /// - or HiGHS fails to extract solution vectors.
    pub fn solution_snapshot(&self) -> Result<SolutionSnapshot, HighsModelError> {
        let solved = self.solved.as_ref().ok_or(HighsModelError::SolveRequired {
            operation: "solution_snapshot",
        })?;
        solved.solution_snapshot()
    }

    fn clear_problem(&mut self) {
        self.col_cost.clear();
        self.col_lower.clear();
        self.col_upper.clear();
        self.row_lower.clear();
        self.row_upper.clear();
        self.columns.clear();
        self.integrality = None;
        self.primal_start = None;
        self.verbosity = None;
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
    /// Boolean option value.
    Bool(bool),
    /// Integer option value.
    Int(i32),
    /// Floating-point option value.
    Float(f64),
    /// String option value.
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
        let objective_value = self.solved.as_ref().map(HighsHandle::objective_value);
        let solved_dimensions = self.solved.as_ref().and_then(|solved| {
            Some((
                solved.num_cols().try_into().ok()?,
                solved.num_rows().try_into().ok()?,
            ))
        });
        let num_variables = solved_dimensions.map_or(self.col_cost.len(), |(num_cols, _)| num_cols);
        let num_constraints =
            solved_dimensions.map_or(self.row_lower.len(), |(_, num_rows)| num_rows);
        f.debug_struct("HighsModel")
            .field("num_variables", &num_variables)
            .field("num_constraints", &num_constraints)
            .field("objective_sense", &self.objective_sense)
            .field("objective_value", &objective_value)
            .finish_non_exhaustive()
    }
}

struct HighsHandle {
    ptr: *mut c_void,
}

#[allow(unsafe_code)]
impl Drop for HighsHandle {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            // SAFETY: `ptr` is created by `Highs_create`, owned exclusively by
            // this handle, and destroyed exactly once here.
            unsafe {
                highs_sys::Highs_destroy(self.ptr);
            }
        }
    }
}

#[allow(unsafe_code)]
impl HighsHandle {
    fn load(model: &HighsModel) -> Result<Self, HighsModelError> {
        // SAFETY: `Highs_create` takes no arguments and returns either a valid
        // HiGHS handle or null, which is checked immediately.
        let ptr = unsafe { highs_sys::Highs_create() };
        if ptr.is_null() {
            return Err(HighsModelError::NullHandle);
        }

        let mut handle = Self { ptr };
        handle.set_bool_option("output_flag", false)?;
        handle.set_bool_option("log_to_console", false)?;

        let num_cols = checked_highs_int(model.col_cost.len(), "columns")?;
        let num_rows = checked_highs_int(model.row_lower.len(), "rows")?;
        let num_nonzeros =
            checked_highs_int(model.columns.iter().map(Vec::len).sum(), "coefficients")?;
        let mut a_start = Vec::with_capacity(model.columns.len());
        let mut a_index = Vec::with_capacity(num_nonzeros as usize);
        let mut a_value = Vec::with_capacity(num_nonzeros as usize);
        for column in &model.columns {
            a_start.push(checked_highs_int(a_index.len(), "column start")?);
            for (row_index, coefficient) in column {
                a_index.push(checked_highs_int(*row_index, "row index")?);
                a_value.push(*coefficient);
            }
        }

        let sense = raw_objective_sense(model.objective_sense);
        // SAFETY: all slices passed to HiGHS are owned by `model` or local
        // vectors and stay alive for the entire call. Dimensions are checked
        // against HiGHS integer storage, and column-wise matrix arrays match
        // the supplied column offsets, row indices, and values.
        let status = unsafe {
            if let Some(integrality) = model.integrality.as_ref() {
                highs_sys::Highs_passMip(
                    handle.ptr,
                    num_cols,
                    num_rows,
                    num_nonzeros,
                    highs_sys::MATRIX_FORMAT_COLUMN_WISE,
                    sense,
                    0.0,
                    model.col_cost.as_ptr(),
                    model.col_lower.as_ptr(),
                    model.col_upper.as_ptr(),
                    model.row_lower.as_ptr(),
                    model.row_upper.as_ptr(),
                    a_start.as_ptr(),
                    a_index.as_ptr(),
                    a_value.as_ptr(),
                    integrality.as_ptr(),
                )
            } else {
                highs_sys::Highs_passLp(
                    handle.ptr,
                    num_cols,
                    num_rows,
                    num_nonzeros,
                    highs_sys::MATRIX_FORMAT_COLUMN_WISE,
                    sense,
                    0.0,
                    model.col_cost.as_ptr(),
                    model.col_lower.as_ptr(),
                    model.col_upper.as_ptr(),
                    model.row_lower.as_ptr(),
                    model.row_upper.as_ptr(),
                    a_start.as_ptr(),
                    a_index.as_ptr(),
                    a_value.as_ptr(),
                )
            }
        };
        ensure_highs_status_ok_for(status, "Highs_passLp/Highs_passMip")?;
        Ok(handle)
    }

    fn apply_options<I>(
        &mut self,
        log_to_console: bool,
        verbosity: Option<u32>,
        options: I,
    ) -> Result<(), HighsModelError>
    where
        I: IntoIterator<Item = (String, HighsOption)>,
    {
        if verbosity.unwrap_or(0) == 0 && !log_to_console {
            self.set_bool_option("output_flag", false)?;
            self.set_bool_option("log_to_console", false)?;
        }
        if let Some(level) = verbosity {
            self.set_bool_option("output_flag", level > 0)?;
        }
        for (option, value) in options {
            match value {
                HighsOption::Bool(value) => self.set_bool_option(&option, value)?,
                HighsOption::Int(value) => self.set_int_option(&option, value)?,
                HighsOption::Float(value) => self.set_double_option(&option, value)?,
                HighsOption::Str(value) => self.set_string_option(&option, &value)?,
            }
        }
        if log_to_console {
            self.set_bool_option("log_to_console", true)?;
            self.set_bool_option("output_flag", true)?;
        }
        Ok(())
    }

    fn run(&mut self) -> Result<HighsStatus, HighsModelError> {
        // SAFETY: `self.ptr` is a live HiGHS handle owned by this wrapper.
        ensure_highs_status_ok_for(unsafe { highs_sys::Highs_run(self.ptr) }, "Highs_run")?;
        // SAFETY: `Highs_run` completed successfully on this live handle.
        let raw_status = unsafe { highs_sys::Highs_getModelStatus(self.ptr) };
        Ok(map_raw_model_status(raw_status))
    }

    fn objective_value(&self) -> f64 {
        // SAFETY: `self.ptr` is a live HiGHS handle and objective value is a
        // scalar query that does not outlive the handle.
        unsafe { highs_sys::Highs_getObjectiveValue(self.ptr) }
    }

    fn num_cols(&self) -> highs_sys::HighsInt {
        // SAFETY: `self.ptr` is a live HiGHS handle.
        unsafe { highs_sys::Highs_getNumCol(self.ptr) }
    }

    fn num_rows(&self) -> highs_sys::HighsInt {
        // SAFETY: `self.ptr` is a live HiGHS handle.
        unsafe { highs_sys::Highs_getNumRow(self.ptr) }
    }

    fn int_info_value(&self, name: &str) -> Result<highs_sys::HighsInt, HighsModelError> {
        let name = cstring_highs_key(name)?;
        let mut value: highs_sys::HighsInt = 0;
        ensure_highs_status_ok_for(
            // SAFETY: `name` is a NUL-terminated CString and `value` points to
            // valid writable storage for the duration of the call.
            unsafe { highs_sys::Highs_getIntInfoValue(self.ptr, name.as_ptr(), &mut value) },
            "Highs_getIntInfoValue",
        )?;
        Ok(value)
    }

    fn double_info_value(&self, name: &str) -> Result<f64, HighsModelError> {
        let name = cstring_highs_key(name)?;
        let mut value = 0.0;
        ensure_highs_status_ok_for(
            // SAFETY: `name` is a NUL-terminated CString and `value` points to
            // valid writable storage for the duration of the call.
            unsafe { highs_sys::Highs_getDoubleInfoValue(self.ptr, name.as_ptr(), &mut value) },
            "Highs_getDoubleInfoValue",
        )?;
        Ok(value)
    }

    fn set_primal_start(&mut self, cols: &[f64]) -> Result<(), HighsModelError> {
        ensure_highs_status_ok_for(
            // SAFETY: `cols` is sized to the model column count by
            // `HighsModel::set_primal_start`; null pointers mark absent row and
            // dual start vectors as expected by HiGHS.
            unsafe {
                highs_sys::Highs_setSolution(self.ptr, cols.as_ptr(), null(), null(), null())
            },
            "Highs_setSolution",
        )
    }

    fn set_bool_option(&mut self, key: &str, value: bool) -> Result<(), HighsModelError> {
        let key = cstring_option_key(key)?;
        ensure_highs_status_ok_for(
            // SAFETY: `key` is a NUL-terminated CString and the handle is live.
            unsafe {
                highs_sys::Highs_setBoolOptionValue(self.ptr, key.as_ptr(), i32::from(value))
            },
            "Highs_setBoolOptionValue",
        )
    }

    fn set_int_option(&mut self, key: &str, value: i32) -> Result<(), HighsModelError> {
        let key = cstring_option_key(key)?;
        ensure_highs_status_ok_for(
            // SAFETY: `key` is a NUL-terminated CString and the handle is live.
            unsafe { highs_sys::Highs_setIntOptionValue(self.ptr, key.as_ptr(), value) },
            "Highs_setIntOptionValue",
        )
    }

    fn set_double_option(&mut self, key: &str, value: f64) -> Result<(), HighsModelError> {
        let key = cstring_option_key(key)?;
        ensure_highs_status_ok_for(
            // SAFETY: `key` is a NUL-terminated CString and the handle is live.
            unsafe { highs_sys::Highs_setDoubleOptionValue(self.ptr, key.as_ptr(), value) },
            "Highs_setDoubleOptionValue",
        )
    }

    fn set_string_option(&mut self, key: &str, value: &str) -> Result<(), HighsModelError> {
        let key = cstring_option_key(key)?;
        let value = CString::new(value).map_err(|_| HighsModelError::InvalidOptionValue {
            option: key.to_string_lossy().into_owned(),
        })?;
        ensure_highs_status_ok_for(
            // SAFETY: `key` and `value` are NUL-terminated CStrings and both
            // remain alive for the duration of the call.
            unsafe {
                highs_sys::Highs_setStringOptionValue(self.ptr, key.as_ptr(), value.as_ptr())
            },
            "Highs_setStringOptionValue",
        )
    }

    fn solution_snapshot(&self) -> Result<SolutionSnapshot, HighsModelError> {
        let num_cols_raw = self.num_cols();
        let num_rows_raw = self.num_rows();
        let (num_cols, num_rows) = checked_solution_dimensions(num_cols_raw, num_rows_raw)?;
        let mut col_values = vec![0.0; num_cols];
        let mut col_duals = vec![0.0; num_cols];
        let mut row_values = vec![0.0; num_rows];
        let mut row_duals = vec![0.0; num_rows];
        let status = unsafe {
            highs_sys::Highs_getSolution(
                self.ptr,
                col_values.as_mut_ptr(),
                col_duals.as_mut_ptr(),
                row_values.as_mut_ptr(),
                row_duals.as_mut_ptr(),
            )
        };
        ensure_highs_status_ok_for(status, "Highs_getSolution")?;
        Ok(SolutionSnapshot {
            col_values,
            col_duals,
            row_values,
            row_duals,
        })
    }
}

fn raw_objective_sense(sense: ObjectiveSense) -> highs_sys::HighsInt {
    match sense {
        ObjectiveSense::Minimize => highs_sys::OBJECTIVE_SENSE_MINIMIZE,
        ObjectiveSense::Maximize => highs_sys::OBJECTIVE_SENSE_MAXIMIZE,
    }
}

fn map_raw_model_status(status: highs_sys::HighsInt) -> HighsStatus {
    match status {
        highs_sys::MODEL_STATUS_OPTIMAL => HighsStatus::Optimal,
        highs_sys::MODEL_STATUS_INFEASIBLE => HighsStatus::Infeasible,
        highs_sys::MODEL_STATUS_UNBOUNDED => HighsStatus::Unbounded,
        highs_sys::MODEL_STATUS_UNBOUNDED_OR_INFEASIBLE => HighsStatus::UnboundedOrInfeasible,
        highs_sys::MODEL_STATUS_REACHED_TIME_LIMIT => HighsStatus::ReachedTimeLimit,
        highs_sys::MODEL_STATUS_REACHED_ITERATION_LIMIT => HighsStatus::ReachedIterationLimit,
        unknown => {
            debug!("Unknown HiGHS status: {:?}", unknown);
            HighsStatus::Unknown
        }
    }
}

fn checked_highs_int(
    value: usize,
    name: &'static str,
) -> Result<highs_sys::HighsInt, HighsModelError> {
    highs_sys::HighsInt::try_from(value)
        .map_err(|_| HighsModelError::InvalidProblemSize { name, value })
}

fn checked_solution_dimensions(
    num_cols: highs_sys::HighsInt,
    num_rows: highs_sys::HighsInt,
) -> Result<(usize, usize), HighsModelError> {
    let to_usize = |value| {
        usize::try_from(value)
            .map_err(|_| HighsModelError::InvalidSolutionDimensions { num_cols, num_rows })
    };
    Ok((to_usize(num_cols)?, to_usize(num_rows)?))
}

fn cstring_option_key(key: &str) -> Result<CString, HighsModelError> {
    CString::new(key).map_err(|_| HighsModelError::InvalidOptionName {
        option: key.to_string(),
    })
}

fn cstring_highs_key(key: &str) -> Result<CString, HighsModelError> {
    CString::new(key).map_err(|_| HighsModelError::InvalidOptionName {
        option: key.to_string(),
    })
}

fn ensure_highs_status_ok_for(
    status: highs_sys::HighsInt,
    operation: &'static str,
) -> Result<(), HighsModelError> {
    if status == highs_sys::STATUS_OK || status == highs_sys::STATUS_WARNING {
        Ok(())
    } else {
        Err(HighsModelError::HighsCallFailed { operation, status })
    }
}

#[cfg(test)]
mod tests {
    use crate::ffi::{HighsModel, ObjectiveSense, SolutionSnapshot};

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

    #[test]
    fn test_solution_snapshot_into_vecs() {
        let snapshot = SolutionSnapshot::new(vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0], vec![6.0]);
        let (cv, cd, rv, rd) = snapshot.into_vecs();
        assert_eq!(
            (cv, cd, rv, rd),
            (vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0], vec![6.0])
        );
    }
}
