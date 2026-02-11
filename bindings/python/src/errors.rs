//! Structured ArcoError exception hierarchy.
//!
//! Provides `ArcoError` as the base exception and subclass exceptions for each
//! error code defined in the PRD. All exceptions are created via PyO3's
//! `create_exception!` macro and registered as both module-level classes and
//! as class attributes on `ArcoError` itself (so `except ArcoError.MODEL_EMPTY`
//! works).

use pyo3::exceptions::PyException;
use pyo3::prelude::*;

// Base exception
pyo3::create_exception!(
    arco,
    ArcoError,
    PyException,
    "Base exception for all Arco errors."
);

// Model errors
pyo3::create_exception!(arco, ModelEmptyError, ArcoError, "Model has no variables.");

// Variable errors
pyo3::create_exception!(
    arco,
    VariableInvalidIdError,
    ArcoError,
    "Variable ID does not exist."
);
pyo3::create_exception!(
    arco,
    VariableInvalidBoundsError,
    ArcoError,
    "Variable bounds are invalid."
);

// Constraint errors
pyo3::create_exception!(
    arco,
    ConstraintInvalidIdError,
    ArcoError,
    "Constraint ID does not exist."
);
pyo3::create_exception!(
    arco,
    ConstraintInvalidBoundsError,
    ArcoError,
    "Constraint bounds are invalid."
);

// Objective errors
pyo3::create_exception!(
    arco,
    ObjectiveMissingError,
    ArcoError,
    "No objective function has been set."
);
pyo3::create_exception!(
    arco,
    ObjectiveAlreadySetError,
    ArcoError,
    "Objective has already been set."
);

// Slack errors
pyo3::create_exception!(
    arco,
    SlackInvalidPenaltyError,
    ArcoError,
    "Slack penalty is invalid."
);

// Solver errors
pyo3::create_exception!(
    arco,
    SolverInfeasibleError,
    ArcoError,
    "Problem is infeasible."
);
pyo3::create_exception!(
    arco,
    SolverUnboundedError,
    ArcoError,
    "Problem is unbounded."
);
pyo3::create_exception!(
    arco,
    SolverTimeLimitError,
    ArcoError,
    "Solver reached time limit."
);
pyo3::create_exception!(
    arco,
    SolverIterationLimitError,
    ArcoError,
    "Solver reached iteration limit."
);
pyo3::create_exception!(
    arco,
    SolverInternalError,
    ArcoError,
    "Internal solver error."
);

// CSC errors
pyo3::create_exception!(arco, CscInvalidDataError, ArcoError, "CSC data is invalid.");

// ── New structured exceptions (replacing ARCO_* inline codes) ─────────

// Expression errors
pyo3::create_exception!(
    arco,
    ExprDivisionByZeroError,
    ArcoError,
    "Division by zero in expression."
);
pyo3::create_exception!(
    arco,
    ExprNotSingleVariableError,
    ArcoError,
    "Expression does not represent a single variable."
);
pyo3::create_exception!(
    arco,
    ExprCoefficientError,
    ArcoError,
    "Expression coefficient must be 1.0 for conversion."
);
pyo3::create_exception!(
    arco,
    ExprConstantOffsetError,
    ArcoError,
    "Expression has constant offset."
);
pyo3::create_exception!(
    arco,
    ExprTypeError,
    ArcoError,
    "Expected an Expr, Variable, or numeric constant."
);

// Array errors
pyo3::create_exception!(
    arco,
    ArrayShapeMismatchError,
    ArcoError,
    "Array shapes do not match."
);
pyo3::create_exception!(
    arco,
    ArrayIndexError,
    ArcoError,
    "Array index out of bounds."
);
pyo3::create_exception!(
    arco,
    ArrayTypeError,
    ArcoError,
    "Invalid array type or argument."
);
pyo3::create_exception!(
    arco,
    ArrayDimensionError,
    ArcoError,
    "Array dimension mismatch or invalid."
);
pyo3::create_exception!(arco, ArrayOverflowError, ArcoError, "Array size overflow.");

// Constraint construction errors
pyo3::create_exception!(
    arco,
    ConstraintTypeError,
    ArcoError,
    "Invalid constraint expression type."
);
pyo3::create_exception!(
    arco,
    ConstraintBoundsMissingError,
    ArcoError,
    "Constraint bounds are required."
);
pyo3::create_exception!(
    arco,
    ConstraintSenseError,
    ArcoError,
    "Invalid constraint sense."
);

// Solver configuration errors
pyo3::create_exception!(
    arco,
    SolverInvalidSettingError,
    ArcoError,
    "Invalid solver setting."
);
pyo3::create_exception!(
    arco,
    SolverIndexError,
    ArcoError,
    "Solver index out of bounds."
);
pyo3::create_exception!(arco, SolverTypeError, ArcoError, "Invalid solver type.");

// IndexSet errors
pyo3::create_exception!(
    arco,
    IndexSetEmptyError,
    ArcoError,
    "Index set must not be empty."
);
pyo3::create_exception!(
    arco,
    IndexSetTypeError,
    ArcoError,
    "Index set member type is invalid."
);
pyo3::create_exception!(
    arco,
    IndexSetArgumentError,
    ArcoError,
    "Invalid IndexSet arguments."
);

// Bounds validation errors
pyo3::create_exception!(
    arco,
    BoundsInvalidError,
    ArcoError,
    "Bounds are invalid: lower > upper."
);

// Slack errors
pyo3::create_exception!(
    arco,
    SlackBoundError,
    ArcoError,
    "Invalid slack bound specification."
);

// CSC matrix errors (more specific than CscInvalidDataError)
pyo3::create_exception!(
    arco,
    CscDtypeError,
    ArcoError,
    "CSC array has invalid dtype."
);
pyo3::create_exception!(
    arco,
    CscDimensionError,
    ArcoError,
    "CSC array has invalid dimensions."
);
pyo3::create_exception!(
    arco,
    CscContiguityError,
    ArcoError,
    "CSC array must be contiguous."
);
pyo3::create_exception!(
    arco,
    CscNegativeIndexError,
    ArcoError,
    "CSC indices must be non-negative."
);

// Model configuration errors
pyo3::create_exception!(
    arco,
    ModelBinaryBoundsError,
    ArcoError,
    "Binary variables must use bounds=[0,1]."
);

// Objective errors
pyo3::create_exception!(
    arco,
    ObjectiveIndexError,
    ArcoError,
    "Variable index out of range for objective."
);

/// Convert a `arco_core::model::ModelError` into the appropriate ArcoError subclass.
pub fn model_error_to_py(e: arco_core::model::ModelError) -> PyErr {
    let msg = e.to_string();
    match e {
        arco_core::model::ModelError::EmptyModel => ModelEmptyError::new_err(msg),
        arco_core::model::ModelError::InvalidVariableId(_) => VariableInvalidIdError::new_err(msg),
        arco_core::model::ModelError::InvalidVariableBounds { .. } => {
            VariableInvalidBoundsError::new_err(msg)
        }
        arco_core::model::ModelError::InvalidConstraintId(_) => {
            ConstraintInvalidIdError::new_err(msg)
        }
        arco_core::model::ModelError::InvalidConstraintBounds { .. } => {
            ConstraintInvalidBoundsError::new_err(msg)
        }
        arco_core::model::ModelError::NoObjective => ObjectiveMissingError::new_err(msg),
        arco_core::model::ModelError::MultipleObjectives => ObjectiveAlreadySetError::new_err(msg),
        arco_core::model::ModelError::InvalidSlackPenalty { .. } => {
            SlackInvalidPenaltyError::new_err(msg)
        }
        arco_core::model::ModelError::InvalidCscData { .. } => CscInvalidDataError::new_err(msg),
        arco_core::model::ModelError::InvalidCoefficient { .. } => {
            ExprCoefficientError::new_err(msg)
        }
    }
}

/// Convert a `arco_core::SolverError` into the appropriate ArcoError subclass.
pub fn solver_error_to_py(e: arco_core::SolverError) -> PyErr {
    let msg = e.to_string();
    match e {
        arco_core::SolverError::EmptyModel => ModelEmptyError::new_err(msg),
        arco_core::SolverError::NoObjective => ObjectiveMissingError::new_err(msg),
        arco_core::SolverError::InvalidVariableId(_) => VariableInvalidIdError::new_err(msg),
        arco_core::SolverError::SolveFailure { status } => {
            use arco_core::solver::SolverStatus;
            match status {
                SolverStatus::Infeasible => SolverInfeasibleError::new_err(msg),
                SolverStatus::Unbounded => SolverUnboundedError::new_err(msg),
                SolverStatus::TimeLimit => SolverTimeLimitError::new_err(msg),
                SolverStatus::IterationLimit => SolverIterationLimitError::new_err(msg),
                _ => SolverInternalError::new_err(msg),
            }
        }
        arco_core::SolverError::InvalidObjectiveSense
        | arco_core::SolverError::SolverNotAvailable(_)
        | arco_core::SolverError::SolverSpecific(_) => SolverInternalError::new_err(msg),
    }
}

/// Register ArcoError and all subclass exceptions on the module.
/// Also register each subclass as a class attribute on ArcoError itself
/// (e.g., `ArcoError.MODEL_EMPTY`).
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();

    // Add base exception
    m.add("ArcoError", py.get_type::<ArcoError>())?;

    // Add subclass exceptions as module-level types
    m.add("ModelEmptyError", py.get_type::<ModelEmptyError>())?;
    m.add(
        "VariableInvalidIdError",
        py.get_type::<VariableInvalidIdError>(),
    )?;
    m.add(
        "VariableInvalidBoundsError",
        py.get_type::<VariableInvalidBoundsError>(),
    )?;
    m.add(
        "ConstraintInvalidIdError",
        py.get_type::<ConstraintInvalidIdError>(),
    )?;
    m.add(
        "ConstraintInvalidBoundsError",
        py.get_type::<ConstraintInvalidBoundsError>(),
    )?;
    m.add(
        "ObjectiveMissingError",
        py.get_type::<ObjectiveMissingError>(),
    )?;
    m.add(
        "ObjectiveAlreadySetError",
        py.get_type::<ObjectiveAlreadySetError>(),
    )?;
    m.add(
        "SlackInvalidPenaltyError",
        py.get_type::<SlackInvalidPenaltyError>(),
    )?;
    m.add(
        "SolverInfeasibleError",
        py.get_type::<SolverInfeasibleError>(),
    )?;
    m.add(
        "SolverUnboundedError",
        py.get_type::<SolverUnboundedError>(),
    )?;
    m.add(
        "SolverTimeLimitError",
        py.get_type::<SolverTimeLimitError>(),
    )?;
    m.add(
        "SolverIterationLimitError",
        py.get_type::<SolverIterationLimitError>(),
    )?;
    m.add("SolverInternalError", py.get_type::<SolverInternalError>())?;
    m.add("CscInvalidDataError", py.get_type::<CscInvalidDataError>())?;

    // Expression errors
    m.add(
        "ExprDivisionByZeroError",
        py.get_type::<ExprDivisionByZeroError>(),
    )?;
    m.add(
        "ExprNotSingleVariableError",
        py.get_type::<ExprNotSingleVariableError>(),
    )?;
    m.add(
        "ExprCoefficientError",
        py.get_type::<ExprCoefficientError>(),
    )?;
    m.add(
        "ExprConstantOffsetError",
        py.get_type::<ExprConstantOffsetError>(),
    )?;
    m.add("ExprTypeError", py.get_type::<ExprTypeError>())?;

    // Array errors
    m.add(
        "ArrayShapeMismatchError",
        py.get_type::<ArrayShapeMismatchError>(),
    )?;
    m.add("ArrayIndexError", py.get_type::<ArrayIndexError>())?;
    m.add("ArrayTypeError", py.get_type::<ArrayTypeError>())?;
    m.add("ArrayDimensionError", py.get_type::<ArrayDimensionError>())?;
    m.add("ArrayOverflowError", py.get_type::<ArrayOverflowError>())?;

    // Constraint construction errors
    m.add("ConstraintTypeError", py.get_type::<ConstraintTypeError>())?;
    m.add(
        "ConstraintBoundsMissingError",
        py.get_type::<ConstraintBoundsMissingError>(),
    )?;
    m.add(
        "ConstraintSenseError",
        py.get_type::<ConstraintSenseError>(),
    )?;

    // Solver configuration errors
    m.add(
        "SolverInvalidSettingError",
        py.get_type::<SolverInvalidSettingError>(),
    )?;
    m.add("SolverIndexError", py.get_type::<SolverIndexError>())?;
    m.add("SolverTypeError", py.get_type::<SolverTypeError>())?;

    // IndexSet errors
    m.add("IndexSetEmptyError", py.get_type::<IndexSetEmptyError>())?;
    m.add("IndexSetTypeError", py.get_type::<IndexSetTypeError>())?;
    m.add(
        "IndexSetArgumentError",
        py.get_type::<IndexSetArgumentError>(),
    )?;

    // Bounds and slack errors
    m.add("BoundsInvalidError", py.get_type::<BoundsInvalidError>())?;
    m.add("SlackBoundError", py.get_type::<SlackBoundError>())?;

    // CSC errors (more specific)
    m.add("CscDtypeError", py.get_type::<CscDtypeError>())?;
    m.add("CscDimensionError", py.get_type::<CscDimensionError>())?;
    m.add("CscContiguityError", py.get_type::<CscContiguityError>())?;
    m.add(
        "CscNegativeIndexError",
        py.get_type::<CscNegativeIndexError>(),
    )?;

    // Model configuration errors
    m.add(
        "ModelBinaryBoundsError",
        py.get_type::<ModelBinaryBoundsError>(),
    )?;

    // Objective errors
    m.add("ObjectiveIndexError", py.get_type::<ObjectiveIndexError>())?;

    // Register subclasses as class attributes on ArcoError (e.g., ArcoError.MODEL_EMPTY)
    let arco_error_type = py.get_type::<ArcoError>();
    arco_error_type.setattr("MODEL_EMPTY", py.get_type::<ModelEmptyError>())?;
    arco_error_type.setattr(
        "VARIABLE_INVALID_ID",
        py.get_type::<VariableInvalidIdError>(),
    )?;
    arco_error_type.setattr(
        "VARIABLE_INVALID_BOUNDS",
        py.get_type::<VariableInvalidBoundsError>(),
    )?;
    arco_error_type.setattr(
        "CONSTRAINT_INVALID_ID",
        py.get_type::<ConstraintInvalidIdError>(),
    )?;
    arco_error_type.setattr(
        "CONSTRAINT_INVALID_BOUNDS",
        py.get_type::<ConstraintInvalidBoundsError>(),
    )?;
    arco_error_type.setattr("OBJECTIVE_MISSING", py.get_type::<ObjectiveMissingError>())?;
    arco_error_type.setattr(
        "OBJECTIVE_ALREADY_SET",
        py.get_type::<ObjectiveAlreadySetError>(),
    )?;
    arco_error_type.setattr(
        "SLACK_INVALID_PENALTY",
        py.get_type::<SlackInvalidPenaltyError>(),
    )?;
    arco_error_type.setattr("SOLVER_INFEASIBLE", py.get_type::<SolverInfeasibleError>())?;
    arco_error_type.setattr("SOLVER_UNBOUNDED", py.get_type::<SolverUnboundedError>())?;
    arco_error_type.setattr("SOLVER_TIME_LIMIT", py.get_type::<SolverTimeLimitError>())?;
    arco_error_type.setattr(
        "SOLVER_ITERATION_LIMIT",
        py.get_type::<SolverIterationLimitError>(),
    )?;
    arco_error_type.setattr("SOLVER_INTERNAL", py.get_type::<SolverInternalError>())?;
    arco_error_type.setattr("CSC_INVALID_DATA", py.get_type::<CscInvalidDataError>())?;

    // Expression errors
    arco_error_type.setattr(
        "EXPR_DIVISION_BY_ZERO",
        py.get_type::<ExprDivisionByZeroError>(),
    )?;
    arco_error_type.setattr(
        "EXPR_NOT_SINGLE_VARIABLE",
        py.get_type::<ExprNotSingleVariableError>(),
    )?;
    arco_error_type.setattr("EXPR_COEFFICIENT", py.get_type::<ExprCoefficientError>())?;
    arco_error_type.setattr(
        "EXPR_CONSTANT_OFFSET",
        py.get_type::<ExprConstantOffsetError>(),
    )?;
    arco_error_type.setattr("EXPR_TYPE", py.get_type::<ExprTypeError>())?;

    // Array errors
    arco_error_type.setattr(
        "ARRAY_SHAPE_MISMATCH",
        py.get_type::<ArrayShapeMismatchError>(),
    )?;
    arco_error_type.setattr("ARRAY_INDEX", py.get_type::<ArrayIndexError>())?;
    arco_error_type.setattr("ARRAY_TYPE", py.get_type::<ArrayTypeError>())?;
    arco_error_type.setattr("ARRAY_DIMENSION", py.get_type::<ArrayDimensionError>())?;
    arco_error_type.setattr("ARRAY_OVERFLOW", py.get_type::<ArrayOverflowError>())?;

    // Constraint construction errors
    arco_error_type.setattr("CONSTRAINT_TYPE", py.get_type::<ConstraintTypeError>())?;
    arco_error_type.setattr(
        "CONSTRAINT_BOUNDS_MISSING",
        py.get_type::<ConstraintBoundsMissingError>(),
    )?;
    arco_error_type.setattr("CONSTRAINT_SENSE", py.get_type::<ConstraintSenseError>())?;

    // Solver configuration errors
    arco_error_type.setattr(
        "SOLVER_INVALID_SETTING",
        py.get_type::<SolverInvalidSettingError>(),
    )?;
    arco_error_type.setattr("SOLVER_INDEX", py.get_type::<SolverIndexError>())?;
    arco_error_type.setattr("SOLVER_TYPE", py.get_type::<SolverTypeError>())?;

    // IndexSet errors
    arco_error_type.setattr("INDEXSET_EMPTY", py.get_type::<IndexSetEmptyError>())?;
    arco_error_type.setattr("INDEXSET_TYPE", py.get_type::<IndexSetTypeError>())?;
    arco_error_type.setattr("INDEXSET_ARGUMENT", py.get_type::<IndexSetArgumentError>())?;

    // Bounds and slack errors
    arco_error_type.setattr("BOUNDS_INVALID", py.get_type::<BoundsInvalidError>())?;
    arco_error_type.setattr("SLACK_BOUND", py.get_type::<SlackBoundError>())?;

    // CSC errors (more specific)
    arco_error_type.setattr("CSC_DTYPE", py.get_type::<CscDtypeError>())?;
    arco_error_type.setattr("CSC_DIMENSION", py.get_type::<CscDimensionError>())?;
    arco_error_type.setattr("CSC_CONTIGUITY", py.get_type::<CscContiguityError>())?;
    arco_error_type.setattr("CSC_NEGATIVE_INDEX", py.get_type::<CscNegativeIndexError>())?;

    // Model configuration errors
    arco_error_type.setattr(
        "MODEL_BINARY_BOUNDS",
        py.get_type::<ModelBinaryBoundsError>(),
    )?;

    // Objective errors
    arco_error_type.setattr("OBJECTIVE_INDEX", py.get_type::<ObjectiveIndexError>())?;

    Ok(())
}
