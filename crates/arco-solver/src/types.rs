use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SolverStatus {
    Optimal,
    Infeasible,
    Unbounded,
    TimeLimit,
    IterationLimit,
    Unknown,
}

pub trait SolverStatusMapping {
    fn to_solver_status(self) -> SolverStatus;

    fn has_solution(self) -> bool
    where
        Self: Sized,
    {
        self.to_solver_status().is_feasible()
    }
}

impl SolverStatus {
    pub const fn is_optimal(self) -> bool {
        matches!(self, SolverStatus::Optimal)
    }

    pub const fn is_feasible(self) -> bool {
        matches!(
            self,
            SolverStatus::Optimal | SolverStatus::TimeLimit | SolverStatus::IterationLimit
        )
    }

    pub const fn is_infeasible(self) -> bool {
        matches!(self, SolverStatus::Infeasible)
    }

    pub const fn is_unbounded(self) -> bool {
        matches!(self, SolverStatus::Unbounded)
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            SolverStatus::Optimal => "optimal",
            SolverStatus::Infeasible => "infeasible",
            SolverStatus::Unbounded => "unbounded",
            SolverStatus::TimeLimit => "time_limit",
            SolverStatus::IterationLimit => "iteration_limit",
            SolverStatus::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for SolverStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SolverModelStats {
    pub variables: usize,
    pub constraints: usize,
    pub coefficients: usize,
}

impl SolverModelStats {
    pub const fn rows_plus_columns(&self) -> usize {
        self.variables + self.constraints
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SolverDiagnostic {
    ModelSizeLimit {
        solver: String,
        operation: String,
        return_code: i32,
        limit: usize,
        model: SolverModelStats,
    },
}

impl std::fmt::Display for SolverDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SolverDiagnostic::ModelSizeLimit {
                solver,
                operation,
                return_code,
                limit,
                model,
            } => write!(
                f,
                "{solver} cannot solve this model because it exceeds the configured size limit.\n\nModel size:\n  - rows: {}\n  - columns: {}\n  - nonzeros: {}\n  - rows + columns: {}\n  - limit: {limit}\n\nTry one of:\n  1. reduce the number of variables or constraints in the model\n  2. switch to another solver, for example: `arco solver set highs`\n  3. use a solver license/profile with a higher size limit\n\nDetails: solver={solver}, operation={operation}, rc={return_code}",
                model.constraints,
                model.variables,
                model.coefficients,
                model.rows_plus_columns(),
            ),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SolverError {
    EmptyModel,
    NoObjective,
    InvalidObjectiveSense,
    InvalidVariableId(u32),
    InvalidResultShape(String),
    InvalidSettings(String),
    SolverNotAvailable(String),
    SolveFailure { status: SolverStatus },
    Diagnostic(SolverDiagnostic),
    SolverSpecific(String),
}

impl SolverError {
    pub const fn code(&self) -> &'static str {
        match self {
            SolverError::EmptyModel => "SOLVER_EMPTY_MODEL",
            SolverError::NoObjective => "SOLVER_NO_OBJECTIVE",
            SolverError::InvalidObjectiveSense => "SOLVER_INVALID_OBJECTIVE_SENSE",
            SolverError::InvalidVariableId(_) => "SOLVER_INVALID_VARIABLE_ID",
            SolverError::InvalidResultShape(_) => "SOLVER_INVALID_RESULT_SHAPE",
            SolverError::InvalidSettings(_) => "SOLVER_INVALID_SETTINGS",
            SolverError::SolverNotAvailable(_) => "SOLVER_NOT_AVAILABLE",
            SolverError::SolveFailure { .. } => "SOLVER_SOLVE_FAILURE",
            SolverError::Diagnostic(_) => "SOLVER_DIAGNOSTIC",
            SolverError::SolverSpecific(_) => "SOLVER_SPECIFIC",
        }
    }
}

impl std::fmt::Display for SolverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SolverError::EmptyModel => write!(f, "[{}] Model has no variables", self.code()),
            SolverError::NoObjective => write!(f, "[{}] Model has no objective", self.code()),
            SolverError::InvalidObjectiveSense => {
                write!(f, "[{}] Invalid objective sense", self.code())
            }
            SolverError::InvalidVariableId(id) => {
                write!(f, "[{}] Variable ID {} does not exist", self.code(), id)
            }
            SolverError::InvalidResultShape(msg) => {
                write!(f, "[{}] Invalid solver result shape: {}", self.code(), msg)
            }
            SolverError::InvalidSettings(msg) => {
                write!(f, "[{}] Invalid solver settings: {}", self.code(), msg)
            }
            SolverError::SolverNotAvailable(msg) => {
                write!(f, "[{}] Solver not available: {}", self.code(), msg)
            }
            SolverError::SolveFailure { status } => {
                write!(f, "[{}] Solve failed with status: {}", self.code(), status)
            }
            SolverError::Diagnostic(diagnostic) => write!(f, "{}", diagnostic),
            SolverError::SolverSpecific(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for SolverError {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Solution {
    pub primal_values: Vec<f64>,
    pub variable_duals: Vec<f64>,
    pub constraint_duals: Vec<f64>,
    pub row_values: Vec<f64>,
    pub objective_value: f64,
    pub status: SolverStatus,
    pub solve_time_seconds: f64,
    pub metadata: BTreeMap<String, f64>,
}

impl Solution {
    pub fn get_primal(&self, index: usize) -> Option<f64> {
        self.primal_values.get(index).copied()
    }

    pub fn get_variable_dual(&self, index: usize) -> Option<f64> {
        self.variable_duals.get(index).copied()
    }

    pub fn get_constraint_dual(&self, index: usize) -> Option<f64> {
        self.constraint_duals.get(index).copied()
    }

    pub fn get_row_value(&self, index: usize) -> Option<f64> {
        self.row_values.get(index).copied()
    }

    pub fn is_optimal(&self) -> bool {
        self.status.is_optimal()
    }

    pub fn is_feasible(&self) -> bool {
        self.status.is_feasible()
    }

    pub fn is_infeasible(&self) -> bool {
        self.status.is_infeasible()
    }

    pub fn is_unbounded(&self) -> bool {
        self.status.is_unbounded()
    }

    pub fn status_string(&self) -> &'static str {
        self.status.as_str()
    }
}
