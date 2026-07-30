#[cfg(all(feature = "scip-bundled", feature = "scip-from-source"))]
compile_error!("features `scip-bundled` and `scip-from-source` are mutually exclusive");

use arco_format::{
    PortableConstraintSense, PortableLinearObjective, PortableLinearReport, PortableObjectiveSense,
    PortableProblem, PortableVariableKind, portable_problem_from_model_view,
};
use arco_model::{ModelView, VariableId};
use arco_solver::{
    ModelViewBackend, ModelViewSolveResult, SolverCapabilityModel, SolverConfig, SolverError,
    SolverFamily, SolverRegistry, SolverStatus, validate_model_view_solve_result,
};
use russcip::{Model, ProblemOrSolving, Solution, Status, VarType, WithSolutions};
use std::collections::BTreeMap;
use std::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind};

const SCIP_INFINITY: f64 = 1.0e20;

pub(crate) const FAMILY_NAME: &str = "scip";
pub const BACKEND_NAME: &str = "arco-rust-scip";

/// Native SCIP backend for primitive model views.
#[derive(Debug, Default, Clone, Copy)]
pub struct ScipModelViewBackend;

impl ModelViewBackend for ScipModelViewBackend {
    fn family(&self) -> &'static str {
        FAMILY_NAME
    }

    fn solve_model_view(
        &self,
        model: &dyn ModelView,
        config: &SolverConfig,
    ) -> Result<ModelViewSolveResult, SolverError> {
        solve_model_view(model, config)
    }
}

/// Solve a primitive model view with bundled native SCIP.
pub(crate) fn solve_model_view(
    model: &(impl ModelView + ?Sized),
    config: &SolverConfig,
) -> Result<ModelViewSolveResult, SolverError> {
    solve_model_view_with_options(model, config, &NativeSolveOptions::default())
}

/// Solve a primitive model view with explicit native SCIP options.
pub(crate) fn solve_model_view_with_options(
    model: &(impl ModelView + ?Sized),
    config: &SolverConfig,
    options: &NativeSolveOptions,
) -> Result<ModelViewSolveResult, SolverError> {
    if model.num_variables() == 0 {
        return Err(SolverError::EmptyModel);
    }
    if model.objective().sense.is_none() && model.objective().terms.is_empty() {
        return Err(SolverError::NoObjective);
    }

    let portable = portable_problem_from_model_view(model);
    let variable_families = portable
        .variable_instances
        .iter()
        .map(|variable| variable.family.clone())
        .collect::<Vec<_>>();
    let native_options = options.with_solver_config(config);
    let output = solve_problem_with_options(
        ScipProblem {
            portable: &portable,
            variable_families: &variable_families,
        },
        true,
        config.log_to_console.unwrap_or(false),
        &native_options,
    )
    .map_err(|error| SolverError::SolverSpecific(error.to_string()))?;

    let solution_values = output
        .variable_values
        .iter()
        .flat_map(|variable| variable.values.iter())
        .map(|value| (value.compiled_name.as_str(), value.value))
        .collect::<BTreeMap<_, _>>();
    let primal_values = (0..model.num_variables())
        .map(|idx| {
            let variable_id = VariableId::new(idx as u32);
            let name = model
                .variable_name(variable_id)
                .map_or_else(|| format!("x{idx}"), str::to_string);
            solution_values.get(name.as_str()).copied().unwrap_or(0.0)
        })
        .collect::<Vec<_>>();
    let mut row_values = vec![0.0; model.num_constraints()];
    for (var_idx, primal_value) in primal_values.iter().copied().enumerate() {
        let variable_id = VariableId::new(var_idx as u32);
        let Some(column) = model.column(variable_id) else {
            continue;
        };
        for (constraint_id, coefficient) in column {
            if let Some(row_value) = row_values.get_mut(constraint_id.inner() as usize) {
                *row_value += coefficient * primal_value;
            }
        }
    }

    let result = ModelViewSolveResult {
        fingerprint: model.fingerprint(),
        status: match output.status {
            SolveStatus::Optimal => SolverStatus::Optimal,
            SolveStatus::Infeasible => SolverStatus::Infeasible,
            SolveStatus::TimeLimit => SolverStatus::TimeLimit,
            SolveStatus::Failed => SolverStatus::Unknown,
        },
        objective_value: output.objective_value,
        primal_values,
        variable_duals: Vec::new(),
        row_values,
        constraint_duals: Vec::new(),
        metadata: Default::default(),
    };
    validate_model_view_solve_result(model, &result)?;
    Ok(result)
}

pub fn register_solver_family(registry: &mut SolverRegistry) {
    registry.add_family(SolverFamily::embedded(
        FAMILY_NAME,
        "SCIP",
        SolverCapabilityModel::lp_mip_default(),
    ));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolveStatus {
    Optimal,
    Infeasible,
    TimeLimit,
    Failed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScalarValue {
    pub compiled_name: String,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableValue {
    pub compiled_name: String,
    pub representative_value: f64,
    pub values: Vec<VariableInstanceValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableInstanceValue {
    pub compiled_name: String,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SolveOutput {
    pub status: SolveStatus,
    pub objective_value: f64,
    pub report_values: Vec<ScalarValue>,
    pub variable_values: Vec<VariableValue>,
}

#[derive(Debug)]
#[cfg_attr(feature = "diagnostics", derive(miette::Diagnostic))]
pub enum Error {
    #[cfg_attr(
        feature = "diagnostics",
        diagnostic(
            code(arco::scip::build),
            help("inspect the generated model for invalid bounds or coefficients")
        )
    )]
    Build { message: String },
    #[cfg_attr(
        feature = "diagnostics",
        diagnostic(
            code(arco::scip::solve),
            help("verify SCIP can solve the generated LP/MIP model")
        )
    )]
    Process { message: String },
    #[cfg_attr(
        feature = "diagnostics",
        diagnostic(
            code(arco::scip::no_feasible_solution),
            help(
                "inspect the solution status or solver logs for why SCIP could not return a feasible solution"
            )
        )
    )]
    NoFeasibleSolution { status: String },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Build { message } => write!(formatter, "SCIP model build failed: {message}"),
            Self::Process { message } => write!(formatter, "SCIP invocation failed: {message}"),
            Self::NoFeasibleSolution { status } => {
                write!(
                    formatter,
                    "SCIP did not produce a feasible solution: {status}"
                )
            }
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NativeSolveOptions {
    pub time_limit: Option<f64>,
    pub mip_gap: Option<f64>,
    pub presolve: Option<bool>,
    pub threads: Option<u32>,
    pub tolerance: Option<f64>,
    pub verbosity: Option<u32>,
}

impl NativeSolveOptions {
    fn with_solver_config(&self, config: &SolverConfig) -> Self {
        let mut options = self.clone();
        if config.time_limit.is_some() {
            options.time_limit = config.time_limit;
        }
        if config.mip_gap.is_some() {
            options.mip_gap = config.mip_gap;
        }
        if config.presolve.is_some() {
            options.presolve = config.presolve;
        }
        if config.threads.is_some() {
            options.threads = config.threads;
        }
        if config.tolerance.is_some() {
            options.tolerance = config.tolerance;
        }
        if config.verbosity.is_some() {
            options.verbosity = config.verbosity;
        }
        options
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ScipProblem<'a> {
    pub portable: &'a PortableProblem,
    pub variable_families: &'a [String],
}

pub(crate) fn solve_problem(
    problem: ScipProblem<'_>,
    include_variable_values: bool,
    log_to_console: bool,
) -> Result<SolveOutput, Error> {
    solve_problem_with_options(
        problem,
        include_variable_values,
        log_to_console,
        &NativeSolveOptions::default(),
    )
}

pub fn solve_problem_with_options(
    problem: ScipProblem<'_>,
    include_variable_values: bool,
    log_to_console: bool,
    options: &NativeSolveOptions,
) -> Result<SolveOutput, Error> {
    catch_unwind(AssertUnwindSafe(|| {
        solve_problem_native(problem, include_variable_values, log_to_console, options)
    }))
    .map_err(|_| Error::Process {
        message: "SCIP native solve panicked".to_string(),
    })?
}

fn solve_problem_native(
    problem: ScipProblem<'_>,
    include_variable_values: bool,
    log_to_console: bool,
    options: &NativeSolveOptions,
) -> Result<SolveOutput, Error> {
    validate_native_options(options)?;
    let objective_coefficients = objective_coefficients(&problem.portable.objective);
    let mut model = match problem.portable.objective.sense {
        PortableObjectiveSense::Minimize => Model::default().minimize(),
        PortableObjectiveSense::Maximize => Model::default().maximize(),
    };
    if !log_to_console {
        model = model.hide_output();
    }
    if let Some(time_limit) = options.time_limit {
        model = model
            .set_real_param("limits/time", time_limit)
            .map_err(|source| Error::Process {
                message: format!("failed to set SCIP time limit: {source:?}"),
            })?;
    }
    if let Some(mip_gap) = options.mip_gap {
        model = model
            .set_real_param("limits/gap", mip_gap)
            .map_err(|source| Error::Process {
                message: format!("failed to set SCIP MIP gap: {source:?}"),
            })?;
    }
    if options.presolve == Some(false) {
        model = model
            .set_int_param("presolving/maxrounds", 0)
            .map_err(|source| Error::Process {
                message: format!("failed to disable SCIP presolve: {source:?}"),
            })?;
    }
    if let Some(threads) = options.threads {
        model = model
            .set_int_param("parallel/maxnthreads", threads as i32)
            .map_err(|source| Error::Process {
                message: format!("failed to set SCIP thread limit: {source:?}"),
            })?;
    }
    if let Some(tolerance) = options.tolerance {
        model = model
            .set_real_param("numerics/feastol", tolerance)
            .map_err(|source| Error::Process {
                message: format!("failed to set SCIP feasibility tolerance: {source:?}"),
            })?;
    }
    if let Some(verbosity) = options.verbosity {
        let level = verbosity.min(i32::MAX as u32) as i32;
        model = model
            .set_int_param("display/verblevel", level)
            .map_err(|source| Error::Process {
                message: format!("failed to set SCIP verbosity: {source:?}"),
            })?;
    }

    let mut variables = Vec::with_capacity(problem.portable.variable_instances.len());
    let mut variable_indices = BTreeMap::new();
    for variable in &problem.portable.variable_instances {
        let objective = objective_coefficients
            .get(&variable.name)
            .copied()
            .unwrap_or(0.0);
        let scip_variable = model.add_var(
            variable.lower,
            variable.upper.unwrap_or(f64::INFINITY),
            objective,
            &variable.name,
            scip_var_type(variable.kind),
        );
        variable_indices.insert(variable.name.clone(), variables.len());
        variables.push(scip_variable);
    }

    for constraint in &problem.portable.constraints {
        let mut constraint_variables = Vec::with_capacity(constraint.terms.len());
        let mut coefficients = Vec::with_capacity(constraint.terms.len());
        for term in &constraint.terms {
            let Some(index) = variable_indices.get(&term.variable_name).copied() else {
                return Err(Error::Build {
                    message: format!(
                        "constraint '{}' references unknown variable '{}'",
                        constraint.name, term.variable_name
                    ),
                });
            };
            constraint_variables.push(&variables[index]);
            coefficients.push(term.coefficient);
        }
        let (lhs, rhs) = scip_constraint_bounds(constraint.sense, constraint.rhs);
        model.add_cons(
            constraint_variables,
            &coefficients,
            lhs,
            rhs,
            &constraint.name,
        );
    }

    let solved_model = model.solve();
    let status = map_native_status(solved_model.status());
    let best_solution = solved_model.best_sol();
    let native_values = best_solution.as_ref().map(|solution: &Solution| {
        variables
            .iter()
            .map(|variable| (variable.name(), solution.val(variable)))
            .collect::<BTreeMap<_, _>>()
    });
    let variable_values =
        build_variable_values(problem, include_variable_values, native_values.as_ref());
    let report_values = problem
        .portable
        .reports
        .iter()
        .map(|report| ScalarValue {
            compiled_name: report.name.clone(),
            value: evaluate_scip_linear_report(report, native_values.as_ref()),
        })
        .collect::<Vec<_>>();

    Ok(SolveOutput {
        status,
        objective_value: problem.portable.objective.constant
            + best_solution
                .as_ref()
                .map_or(0.0, |solution| solution.obj_val()),
        report_values,
        variable_values,
    })
}

fn validate_native_options(options: &NativeSolveOptions) -> Result<(), Error> {
    if let Some(time_limit) = options.time_limit {
        if !time_limit.is_finite() || time_limit < 0.0 {
            return Err(Error::Process {
                message: "SCIP time_limit must be finite and non-negative".to_string(),
            });
        }
    }
    if let Some(mip_gap) = options.mip_gap {
        if !mip_gap.is_finite() || mip_gap < 0.0 {
            return Err(Error::Process {
                message: "SCIP mip_gap must be finite and non-negative".to_string(),
            });
        }
    }
    if let Some(threads) = options.threads {
        if threads == 0 {
            return Err(Error::Process {
                message: "SCIP threads must be >= 1".to_string(),
            });
        }
    }
    if let Some(tolerance) = options.tolerance {
        if !tolerance.is_finite() || tolerance < 0.0 {
            return Err(Error::Process {
                message: "SCIP tolerance must be finite and non-negative".to_string(),
            });
        }
    }
    Ok(())
}

fn objective_coefficients(objective: &PortableLinearObjective) -> BTreeMap<String, f64> {
    let mut coefficients = BTreeMap::new();
    for term in &objective.terms {
        *coefficients
            .entry(term.variable_name.clone())
            .or_insert(0.0) += term.coefficient;
    }
    coefficients
}

fn scip_var_type(kind: PortableVariableKind) -> VarType {
    match kind {
        PortableVariableKind::Continuous => VarType::Continuous,
        PortableVariableKind::Integer => VarType::Integer,
        PortableVariableKind::Binary => VarType::Binary,
    }
}

fn scip_constraint_bounds(sense: PortableConstraintSense, rhs: f64) -> (f64, f64) {
    match sense {
        PortableConstraintSense::GreaterEqual => (rhs, SCIP_INFINITY),
        PortableConstraintSense::LessEqual => (-SCIP_INFINITY, rhs),
        PortableConstraintSense::Equal => (rhs, rhs),
    }
}

fn map_native_status(status: Status) -> SolveStatus {
    match status {
        Status::Optimal | Status::GapLimit => SolveStatus::Optimal,
        Status::Infeasible => SolveStatus::Infeasible,
        Status::TimeLimit => SolveStatus::TimeLimit,
        Status::Unbounded
        | Status::Inforunbd
        | Status::Unknown
        | Status::UserInterrupt
        | Status::NodeLimit
        | Status::TotalNodeLimit
        | Status::StallNodeLimit
        | Status::MemoryLimit
        | Status::SolutionLimit
        | Status::BestSolutionLimit
        | Status::RestartLimit
        | Status::Terminate => SolveStatus::Failed,
    }
}

fn build_variable_values(
    problem: ScipProblem<'_>,
    include_variable_values: bool,
    native_values: Option<&BTreeMap<String, f64>>,
) -> Vec<VariableValue> {
    problem
        .variable_families
        .iter()
        .map(|family| {
            let representative_value = problem
                .portable
                .variable_instances
                .iter()
                .find(|instance| instance.family == *family)
                .map_or(0.0, |instance| native_value(native_values, &instance.name));

            let values = if include_variable_values {
                problem
                    .portable
                    .variable_instances
                    .iter()
                    .filter(|instance| instance.family == *family)
                    .map(|instance| VariableInstanceValue {
                        compiled_name: instance.name.clone(),
                        value: native_value(native_values, &instance.name),
                    })
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };

            VariableValue {
                compiled_name: family.clone(),
                representative_value,
                values,
            }
        })
        .collect::<Vec<_>>()
}

fn native_value(native_values: Option<&BTreeMap<String, f64>>, variable_name: &str) -> f64 {
    native_values
        .and_then(|values| values.get(variable_name).copied())
        .unwrap_or(0.0)
}

fn evaluate_scip_linear_report(
    report: &PortableLinearReport,
    variable_values: Option<&BTreeMap<String, f64>>,
) -> f64 {
    let terms_value: f64 = report
        .terms
        .iter()
        .map(|term| term.coefficient * native_value(variable_values, &term.variable_name))
        .sum();
    report.constant + terms_value
}

#[cfg(test)]
mod tests {
    use super::*;
    use arco_format::{PortableLinearTerm, PortableVariableInstance};
    use arco_solver::{
        SolverRegistry, SolverTransport, check_empty_model_rejected, check_no_objective_rejected,
        check_small_lp, check_small_milp,
    };

    #[test]
    fn register_solver_family_registers_embedded_family() {
        let mut registry = SolverRegistry::new();
        register_solver_family(&mut registry);

        let family = registry
            .family(FAMILY_NAME)
            .unwrap_or_else(|| panic!("missing registered family: {FAMILY_NAME}"));
        assert_eq!(family.name, FAMILY_NAME);
        assert!(family.transports.contains(&SolverTransport::Embedded));
    }

    #[test]
    fn native_options_reject_invalid_limits() {
        let options = NativeSolveOptions {
            time_limit: Some(f64::NAN),
            ..NativeSolveOptions::default()
        };

        let error = validate_native_options(&options).expect_err("NaN time limit should fail");
        assert!(error.to_string().contains("time_limit"));
    }

    #[test]
    fn native_status_mapping_handles_optimal_infeasible_and_time_limit() {
        assert_eq!(map_native_status(Status::Optimal), SolveStatus::Optimal);
        assert_eq!(map_native_status(Status::GapLimit), SolveStatus::Optimal);
        assert_eq!(
            map_native_status(Status::Infeasible),
            SolveStatus::Infeasible
        );
        assert_eq!(map_native_status(Status::TimeLimit), SolveStatus::TimeLimit);
        assert_eq!(map_native_status(Status::Unbounded), SolveStatus::Failed);
    }

    #[test]
    fn evaluate_scip_linear_report_defaults_missing_variables_to_zero() {
        let report = PortableLinearReport {
            name: "r".to_string(),
            terms: vec![
                PortableLinearTerm {
                    coefficient: 2.0,
                    variable_name: "x".to_string(),
                },
                PortableLinearTerm {
                    coefficient: -1.0,
                    variable_name: "y".to_string(),
                },
            ],
            constant: 5.0,
        };
        let variable_values = BTreeMap::from([("x".to_string(), 3.0)]);

        let value = evaluate_scip_linear_report(&report, Some(&variable_values));
        assert!((value - 11.0).abs() < f64::EPSILON);
    }

    #[test]
    fn model_view_backend_rejects_empty_problem_before_scip_solve() {
        let backend = ScipModelViewBackend;
        check_empty_model_rejected(&backend).expect("SCIP should reject empty model");
    }

    #[test]
    fn model_view_backend_rejects_no_objective_problem_before_scip_solve() {
        let backend = ScipModelViewBackend;
        check_no_objective_rejected(&backend).expect("SCIP should reject missing objective");
    }

    #[test]
    fn model_view_backend_solves_shared_small_lp() {
        let backend = ScipModelViewBackend;
        let report =
            check_small_lp(&backend, &SolverConfig::default()).expect("SCIP should solve small LP");

        assert_eq!(report.family, FAMILY_NAME);
        assert_eq!(report.variables, 1);
        assert_eq!(report.constraints, 1);
        assert_eq!(report.coefficients, 1);
    }

    #[test]
    fn model_view_backend_solves_shared_small_milp() {
        let backend = ScipModelViewBackend;
        let report = check_small_milp(&backend, &SolverConfig::default())
            .expect("SCIP should solve small MILP");

        assert_eq!(report.family, FAMILY_NAME);
        assert_eq!(report.variables, 1);
        assert_eq!(report.constraints, 1);
        assert_eq!(report.coefficients, 1);
    }

    #[test]
    fn solve_problem_solves_basic_lp_with_bundled_scip() {
        let problem = PortableProblem {
            variable_instances: vec![
                PortableVariableInstance {
                    name: "x".to_string(),
                    family: "x".to_string(),
                    lower: 0.0,
                    upper: None,
                    kind: PortableVariableKind::Continuous,
                },
                PortableVariableInstance {
                    name: "y".to_string(),
                    family: "y".to_string(),
                    lower: 0.0,
                    upper: None,
                    kind: PortableVariableKind::Continuous,
                },
            ],
            constraints: vec![arco_format::PortableLinearConstraint {
                name: "demand".to_string(),
                sense: PortableConstraintSense::GreaterEqual,
                rhs: 5.0,
                terms: vec![
                    PortableLinearTerm {
                        variable_name: "x".to_string(),
                        coefficient: 1.0,
                    },
                    PortableLinearTerm {
                        variable_name: "y".to_string(),
                        coefficient: 1.0,
                    },
                ],
            }],
            objective: PortableLinearObjective {
                name: "obj".to_string(),
                sense: PortableObjectiveSense::Minimize,
                constant: 0.0,
                terms: vec![
                    PortableLinearTerm {
                        variable_name: "x".to_string(),
                        coefficient: 1.0,
                    },
                    PortableLinearTerm {
                        variable_name: "y".to_string(),
                        coefficient: 1.0,
                    },
                ],
            },
            reports: Vec::new(),
        };
        let scip_problem = ScipProblem {
            portable: &problem,
            variable_families: &["x".to_string(), "y".to_string()],
        };

        let output = solve_problem(scip_problem, true, false).expect("SCIP should solve basic LP");

        assert_eq!(output.status, SolveStatus::Optimal);
        assert!((output.objective_value - 5.0).abs() <= 1e-6);
    }

    #[test]
    fn solve_output_defaults_missing_variable_values_to_zero() {
        let problem = PortableProblem {
            variable_instances: vec![PortableVariableInstance {
                name: "x[A,1]".to_string(),
                family: "x[a,t]".to_string(),
                lower: 0.0,
                upper: None,
                kind: PortableVariableKind::Continuous,
            }],
            constraints: Vec::new(),
            objective: PortableLinearObjective {
                name: "obj".to_string(),
                sense: PortableObjectiveSense::Maximize,
                constant: 0.0,
                terms: Vec::new(),
            },
            reports: Vec::new(),
        };
        let scip_problem = ScipProblem {
            portable: &problem,
            variable_families: &["x[a,t]".to_string()],
        };

        let output = build_variable_values(scip_problem, true, None);

        assert!((output[0].representative_value).abs() <= f64::EPSILON);
        assert!(output[0].values[0].value.abs() <= f64::EPSILON);
    }
}
