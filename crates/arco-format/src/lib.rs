//! Format seam for portable Arco format DTOs and primitive model views.

use arco_model::{ConstraintId, ModelView, Sense, VariableId};
use std::collections::BTreeMap;
use std::fmt;
use std::io::Write;

#[derive(Debug)]
pub enum ExportError {
    Io { source: std::io::Error },
}

impl fmt::Display for ExportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { source } => write!(formatter, "failed to write exported model: {source}"),
        }
    }
}

impl std::error::Error for ExportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source } => Some(source),
        }
    }
}

#[cfg(feature = "diagnostics")]
impl miette::Diagnostic for ExportError {}

#[derive(Debug, Clone, PartialEq)]
pub struct PortableProblem {
    pub variable_instances: Vec<PortableVariableInstance>,
    pub constraints: Vec<PortableLinearConstraint>,
    pub objective: PortableLinearObjective,
    pub reports: Vec<PortableLinearReport>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PortableVariableInstance {
    pub name: String,
    pub family: String,
    pub lower: f64,
    pub upper: Option<f64>,
    pub kind: PortableVariableKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortableVariableKind {
    Continuous,
    Integer,
    Binary,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PortableLinearConstraint {
    pub name: String,
    pub sense: PortableConstraintSense,
    pub rhs: f64,
    pub terms: Vec<PortableLinearTerm>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortableConstraintSense {
    GreaterEqual,
    LessEqual,
    Equal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PortableLinearObjective {
    pub name: String,
    pub sense: PortableObjectiveSense,
    pub constant: f64,
    pub terms: Vec<PortableLinearTerm>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortableObjectiveSense {
    Minimize,
    Maximize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PortableLinearReport {
    pub name: String,
    pub constant: f64,
    pub terms: Vec<PortableLinearTerm>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PortableLinearTerm {
    pub variable_name: String,
    pub coefficient: f64,
}

pub fn write_lp(problem: &PortableProblem, writer: &mut dyn Write) -> Result<(), ExportError> {
    writeln!(writer, "\\ Problem name: MODEL").map_err(io_error)?;
    writeln!(writer, "{}", lp_objective_header(problem.objective.sense)).map_err(io_error)?;
    writeln!(
        writer,
        "  {}: {}",
        problem.objective.name,
        format_linear_expression(&problem.objective.terms, problem.objective.constant)
    )
    .map_err(io_error)?;
    writeln!(writer, "Subject To").map_err(io_error)?;
    for constraint in &problem.constraints {
        writeln!(
            writer,
            "  {}: {} {} {}",
            constraint.name,
            format_linear_expression(&constraint.terms, 0.0),
            lp_constraint_sense(constraint.sense),
            format_number(constraint.rhs)
        )
        .map_err(io_error)?;
    }

    writeln!(writer, "Bounds").map_err(io_error)?;
    for variable in &problem.variable_instances {
        write_lp_bounds_line(variable, writer)?;
    }

    let generals = problem
        .variable_instances
        .iter()
        .filter(|variable| variable.kind == PortableVariableKind::Integer)
        .map(|variable| variable.name.as_str())
        .collect::<Vec<_>>();
    if !generals.is_empty() {
        writeln!(writer, "Generals").map_err(io_error)?;
        for name in generals {
            writeln!(writer, "  {name}").map_err(io_error)?;
        }
    }

    let binaries = problem
        .variable_instances
        .iter()
        .filter(|variable| variable.kind == PortableVariableKind::Binary)
        .map(|variable| variable.name.as_str())
        .collect::<Vec<_>>();
    if !binaries.is_empty() {
        writeln!(writer, "Binaries").map_err(io_error)?;
        for name in binaries {
            writeln!(writer, "  {name}").map_err(io_error)?;
        }
    }

    writeln!(writer, "End").map_err(io_error)?;
    Ok(())
}

pub fn write_mps(problem: &PortableProblem, writer: &mut dyn Write) -> Result<(), ExportError> {
    writeln!(writer, "NAME          MODEL").map_err(io_error)?;
    writeln!(writer, "ROWS").map_err(io_error)?;
    writeln!(writer, " N  OBJ").map_err(io_error)?;
    for constraint in &problem.constraints {
        writeln!(
            writer,
            " {}  {}",
            mps_row_type(constraint.sense),
            constraint.name
        )
        .map_err(io_error)?;
    }

    let objective_terms = problem
        .objective
        .terms
        .iter()
        .map(|term| (term.variable_name.clone(), term.coefficient))
        .collect::<BTreeMap<_, _>>();
    let column_terms = build_column_terms(problem, &objective_terms);

    writeln!(writer, "COLUMNS").map_err(io_error)?;
    let mut in_integer_block = false;
    for variable in &problem.variable_instances {
        let is_integer = variable.kind != PortableVariableKind::Continuous;
        if is_integer && !in_integer_block {
            writeln!(writer, "    MARKER    'MARKER'                 'INTORG'")
                .map_err(io_error)?;
            in_integer_block = true;
        } else if !is_integer && in_integer_block {
            writeln!(writer, "    MARKER    'MARKER'                 'INTEND'")
                .map_err(io_error)?;
            in_integer_block = false;
        }

        let entries = column_terms
            .get(&variable.name)
            .cloned()
            .unwrap_or_default();
        for pair in entries.chunks(2) {
            match pair {
                [(row_1, value_1), (row_2, value_2)] => writeln!(
                    writer,
                    "    {:<8}  {:<8}  {:>16}  {:<8}  {:>16}",
                    variable.name,
                    row_1,
                    format_number(*value_1),
                    row_2,
                    format_number(*value_2)
                )
                .map_err(io_error)?,
                [(row_1, value_1)] => writeln!(
                    writer,
                    "    {:<8}  {:<8}  {:>16}",
                    variable.name,
                    row_1,
                    format_number(*value_1)
                )
                .map_err(io_error)?,
                _ => {}
            }
        }
    }
    if in_integer_block {
        writeln!(writer, "    MARKER    'MARKER'                 'INTEND'").map_err(io_error)?;
    }

    writeln!(writer, "RHS").map_err(io_error)?;
    for constraint in &problem.constraints {
        writeln!(
            writer,
            "    RHS1      {:<8}  {:>16}",
            constraint.name,
            format_number(constraint.rhs)
        )
        .map_err(io_error)?;
    }

    writeln!(writer, "BOUNDS").map_err(io_error)?;
    for variable in &problem.variable_instances {
        write_mps_bounds(variable, writer)?;
    }

    writeln!(writer, "ENDATA").map_err(io_error)?;
    Ok(())
}

/// Format request over a primitive model view.
pub struct ModelViewFormatRequest<'a, V: ModelView + ?Sized> {
    pub model: &'a V,
    pub policy: RenderPolicy,
}

/// Rendering policy for model-view exports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderPolicy {
    pub(crate) include_generated_names: bool,
}

impl Default for RenderPolicy {
    fn default() -> Self {
        Self {
            include_generated_names: true,
        }
    }
}

/// Format result bytes and basic metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatResult {
    pub(crate) bytes: Vec<u8>,
    pub(crate) format: &'static str,
}

/// Render a primitive model view as LP text.
pub(crate) fn write_model_view_lp(
    model: &impl ModelView,
    writer: &mut dyn Write,
) -> Result<(), ExportError> {
    let portable = portable_problem_from_model_view(model);
    write_lp(&portable, writer)
}

/// Render a primitive model view as LP bytes.
pub(crate) fn export_model_view_lp(model: &impl ModelView) -> Result<FormatResult, ExportError> {
    let mut bytes = Vec::new();
    write_model_view_lp(model, &mut bytes)?;
    Ok(FormatResult {
        bytes,
        format: "lp",
    })
}

/// Render a primitive model view as MPS text.
pub(crate) fn write_model_view_mps(
    model: &impl ModelView,
    writer: &mut dyn Write,
) -> Result<(), ExportError> {
    let portable = portable_problem_from_model_view(model);
    write_mps(&portable, writer)
}

/// Render a primitive model view as MPS bytes.
pub(crate) fn export_model_view_mps(model: &impl ModelView) -> Result<FormatResult, ExportError> {
    let mut bytes = Vec::new();
    write_model_view_mps(model, &mut bytes)?;
    Ok(FormatResult {
        bytes,
        format: "mps",
    })
}

/// Build the portable format DTO used by concrete text exporters from a model view.
///
/// This is an exporter DTO, not Arco's canonical model serialization. Canonical
/// structural documents remain owned by `arco-model`; this helper only allocates
/// the row-oriented names and terms required by LP/MPS style renderers.
pub fn portable_problem_from_model_view(model: &(impl ModelView + ?Sized)) -> PortableProblem {
    let variable_instances = (0..model.num_variables())
        .filter_map(|idx| {
            let variable_id = VariableId::new(idx as u32);
            let variable = model.variable(variable_id)?;
            let is_binary_bounds = variable.bounds.lower.to_bits() == 0.0f64.to_bits()
                && variable.bounds.upper.to_bits() == 1.0f64.to_bits();
            let kind = if variable.is_integer && is_binary_bounds {
                PortableVariableKind::Binary
            } else if variable.is_integer {
                PortableVariableKind::Integer
            } else {
                PortableVariableKind::Continuous
            };
            Some(PortableVariableInstance {
                name: variable_name(model, variable_id, idx),
                family: variable_name(model, variable_id, idx),
                lower: variable.bounds.lower,
                upper: variable
                    .bounds
                    .upper
                    .is_finite()
                    .then_some(variable.bounds.upper),
                kind,
            })
        })
        .collect::<Vec<_>>();

    let mut terms_by_constraint = vec![Vec::new(); model.num_constraints()];
    for var_idx in 0..model.num_variables() {
        let variable_id = VariableId::new(var_idx as u32);
        let Some(column) = model.column(variable_id) else {
            continue;
        };
        for (constraint_id, coefficient) in column {
            let row_idx = constraint_id.inner() as usize;
            if let Some(terms) = terms_by_constraint.get_mut(row_idx) {
                terms.push(PortableLinearTerm {
                    variable_name: variable_name(model, variable_id, var_idx),
                    coefficient: *coefficient,
                });
            }
        }
    }

    let constraints = (0..model.num_constraints())
        .filter_map(|idx| {
            let constraint = model.constraint(ConstraintId::new(idx as u32))?;
            Some(
                match (
                    constraint.bounds.lower.is_finite(),
                    constraint.bounds.upper.is_finite(),
                ) {
                    (true, true)
                        if constraint.bounds.lower.to_bits()
                            == constraint.bounds.upper.to_bits() =>
                    {
                        PortableLinearConstraint {
                            name: constraint_name(model, ConstraintId::new(idx as u32), idx),
                            sense: PortableConstraintSense::Equal,
                            rhs: constraint.bounds.lower,
                            terms: terms_by_constraint[idx].clone(),
                        }
                    }
                    (_, true) => PortableLinearConstraint {
                        name: constraint_name(model, ConstraintId::new(idx as u32), idx),
                        sense: PortableConstraintSense::LessEqual,
                        rhs: constraint.bounds.upper,
                        terms: terms_by_constraint[idx].clone(),
                    },
                    (true, false) => PortableLinearConstraint {
                        name: constraint_name(model, ConstraintId::new(idx as u32), idx),
                        sense: PortableConstraintSense::GreaterEqual,
                        rhs: constraint.bounds.lower,
                        terms: terms_by_constraint[idx].clone(),
                    },
                    (false, false) => PortableLinearConstraint {
                        name: constraint_name(model, ConstraintId::new(idx as u32), idx),
                        sense: PortableConstraintSense::Equal,
                        rhs: 0.0,
                        terms: Vec::new(),
                    },
                },
            )
        })
        .collect();

    let objective = model.objective();
    PortableProblem {
        variable_instances,
        constraints,
        objective: PortableLinearObjective {
            name: model.objective_name().unwrap_or("obj").to_string(),
            sense: match objective.sense.unwrap_or(Sense::Minimize) {
                Sense::Minimize => PortableObjectiveSense::Minimize,
                Sense::Maximize => PortableObjectiveSense::Maximize,
            },
            constant: 0.0,
            terms: objective
                .terms
                .iter()
                .map(|(variable_id, coefficient)| PortableLinearTerm {
                    variable_name: variable_name(model, *variable_id, variable_id.inner() as usize),
                    coefficient: *coefficient,
                })
                .collect(),
        },
        reports: Vec::new(),
    }
}

fn variable_name(model: &(impl ModelView + ?Sized), id: VariableId, index: usize) -> String {
    model
        .variable_name(id)
        .map_or_else(|| format!("x{index}"), str::to_string)
}

fn constraint_name(model: &(impl ModelView + ?Sized), id: ConstraintId, index: usize) -> String {
    model
        .constraint_name(id)
        .map_or_else(|| format!("c{index}"), str::to_string)
}

fn build_column_terms(
    problem: &PortableProblem,
    objective_terms: &BTreeMap<String, f64>,
) -> BTreeMap<String, Vec<(String, f64)>> {
    let mut terms = BTreeMap::<String, Vec<(String, f64)>>::new();

    for variable in &problem.variable_instances {
        if let Some(value) = objective_terms.get(&variable.name) {
            terms
                .entry(variable.name.clone())
                .or_default()
                .push(("OBJ".to_string(), *value));
        }
    }

    for constraint in &problem.constraints {
        for term in &constraint.terms {
            terms
                .entry(term.variable_name.clone())
                .or_default()
                .push((constraint.name.clone(), term.coefficient));
        }
    }

    terms
}

fn lp_objective_header(sense: PortableObjectiveSense) -> &'static str {
    match sense {
        PortableObjectiveSense::Minimize => "Minimize",
        PortableObjectiveSense::Maximize => "Maximize",
    }
}

fn lp_constraint_sense(sense: PortableConstraintSense) -> &'static str {
    match sense {
        PortableConstraintSense::GreaterEqual => ">=",
        PortableConstraintSense::LessEqual => "<=",
        PortableConstraintSense::Equal => "=",
    }
}

fn mps_row_type(sense: PortableConstraintSense) -> char {
    match sense {
        PortableConstraintSense::GreaterEqual => 'G',
        PortableConstraintSense::LessEqual => 'L',
        PortableConstraintSense::Equal => 'E',
    }
}

fn write_lp_bounds_line(
    variable: &PortableVariableInstance,
    writer: &mut dyn Write,
) -> Result<(), ExportError> {
    match variable.upper {
        Some(upper) if (variable.lower - upper).abs() < f64::EPSILON => writeln!(
            writer,
            "  {} = {}",
            variable.name,
            format_number(variable.lower)
        )
        .map_err(io_error)?,
        Some(upper) => writeln!(
            writer,
            "  {} <= {} <= {}",
            format_number(variable.lower),
            variable.name,
            format_number(upper)
        )
        .map_err(io_error)?,
        None if variable.lower == f64::NEG_INFINITY => {
            writeln!(writer, "  {} free", variable.name).map_err(io_error)?;
        }
        None => writeln!(
            writer,
            "  {} <= {}",
            format_number(variable.lower),
            variable.name
        )
        .map_err(io_error)?,
    }

    Ok(())
}

fn write_mps_bounds(
    variable: &PortableVariableInstance,
    writer: &mut dyn Write,
) -> Result<(), ExportError> {
    match variable.kind {
        PortableVariableKind::Binary => {
            writeln!(writer, " BV BND1      {}", variable.name).map_err(io_error)?;
            return Ok(());
        }
        PortableVariableKind::Integer | PortableVariableKind::Continuous => {}
    }

    if variable.lower == f64::NEG_INFINITY && variable.upper.is_none() {
        writeln!(writer, " FR BND1      {}", variable.name).map_err(io_error)?;
        return Ok(());
    }

    let lower_code = if variable.kind == PortableVariableKind::Integer {
        "LI"
    } else {
        "LO"
    };
    writeln!(
        writer,
        " {} BND1      {:<8}  {}",
        lower_code,
        variable.name,
        format_number(variable.lower)
    )
    .map_err(io_error)?;

    if let Some(upper) = variable.upper {
        let upper_code = if variable.kind == PortableVariableKind::Integer {
            "UI"
        } else {
            "UP"
        };
        writeln!(
            writer,
            " {} BND1      {:<8}  {}",
            upper_code,
            variable.name,
            format_number(upper)
        )
        .map_err(io_error)?;
    }

    Ok(())
}

fn format_linear_expression(terms: &[PortableLinearTerm], constant: f64) -> String {
    let mut parts = Vec::new();
    if constant != 0.0 || terms.is_empty() {
        parts.push(format_number(constant));
    }

    for term in terms {
        let sign = if term.coefficient < 0.0 { "-" } else { "+" };
        let absolute = term.coefficient.abs();
        let body = if approximately_one(absolute) {
            term.variable_name.clone()
        } else {
            format!("{} {}", format_number(absolute), term.variable_name)
        };

        if parts.is_empty() {
            if sign == "-" {
                parts.push(format!("- {body}"));
            } else {
                parts.push(body);
            }
        } else {
            parts.push(format!("{sign} {body}"));
        }
    }

    parts.join(" ")
}

fn approximately_one(value: f64) -> bool {
    (value - 1.0).abs() < 1e-9
}

fn format_number(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value.is_infinite() {
        return if value.is_sign_positive() {
            "+inf".to_string()
        } else {
            "-inf".to_string()
        };
    }
    let mut text = if approximately_one(value.fract()) || value.fract().abs() < 1e-9 {
        format!("{value:.0}")
    } else {
        format!("{value}")
    };
    if text == "-0" {
        text = "0".to_string();
    }
    text
}

fn io_error(source: std::io::Error) -> ExportError {
    ExportError::Io { source }
}

#[cfg(test)]
mod tests {
    use crate::{
        PortableConstraintSense, PortableLinearConstraint, PortableLinearObjective,
        PortableLinearTerm, PortableObjectiveSense, PortableProblem, PortableVariableInstance,
        PortableVariableKind, write_lp,
    };
    use arco_model::{Bounds, Constraint, Model, Objective, Sense, Variable};

    #[test]
    fn write_lp_accepts_portable_problem() {
        let mut output = Vec::new();
        write_lp(&portable_problem(), &mut output).expect("portable LP should render");

        let output = String::from_utf8(output).expect("valid utf8");
        assert!(output.contains("Minimize"));
        assert!(output.contains("  demand: x + 2 y >= 7"));
        assert!(output.contains("Binaries"));
    }

    #[test]
    fn write_lp_accepts_model_view() {
        let model = model_view_problem();
        let output = crate::export_model_view_lp(&model).expect("model view LP should render");

        assert_eq!(output.format, "lp");
        assert!(
            String::from_utf8(output.bytes)
                .expect("valid utf8")
                .contains("Minimize")
        );
    }

    #[test]
    fn write_mps_accepts_model_view() {
        let model = model_view_problem();
        let output = crate::export_model_view_mps(&model).expect("model view MPS should render");

        assert_eq!(output.format, "mps");
        assert!(
            String::from_utf8(output.bytes)
                .expect("valid utf8")
                .contains("NAME          MODEL")
        );
    }

    fn portable_problem() -> PortableProblem {
        PortableProblem {
            variable_instances: vec![
                PortableVariableInstance {
                    name: "x".to_string(),
                    family: "vars".to_string(),
                    lower: 0.0,
                    upper: Some(10.0),
                    kind: PortableVariableKind::Continuous,
                },
                PortableVariableInstance {
                    name: "y".to_string(),
                    family: "vars".to_string(),
                    lower: -2.0,
                    upper: Some(5.0),
                    kind: PortableVariableKind::Integer,
                },
                PortableVariableInstance {
                    name: "z".to_string(),
                    family: "vars".to_string(),
                    lower: 0.0,
                    upper: Some(1.0),
                    kind: PortableVariableKind::Binary,
                },
            ],
            constraints: vec![
                PortableLinearConstraint {
                    name: "demand".to_string(),
                    sense: PortableConstraintSense::GreaterEqual,
                    rhs: 7.0,
                    terms: vec![
                        PortableLinearTerm {
                            variable_name: "x".to_string(),
                            coefficient: 1.0,
                        },
                        PortableLinearTerm {
                            variable_name: "y".to_string(),
                            coefficient: 2.0,
                        },
                    ],
                },
                PortableLinearConstraint {
                    name: "capacity".to_string(),
                    sense: PortableConstraintSense::LessEqual,
                    rhs: 11.5,
                    terms: vec![PortableLinearTerm {
                        variable_name: "z".to_string(),
                        coefficient: -3.0,
                    }],
                },
            ],
            objective: PortableLinearObjective {
                name: "cost".to_string(),
                sense: PortableObjectiveSense::Minimize,
                constant: 4.0,
                terms: vec![
                    PortableLinearTerm {
                        variable_name: "x".to_string(),
                        coefficient: 1.5,
                    },
                    PortableLinearTerm {
                        variable_name: "y".to_string(),
                        coefficient: -1.0,
                    },
                    PortableLinearTerm {
                        variable_name: "z".to_string(),
                        coefficient: 2.0,
                    },
                ],
            },
            reports: Vec::new(),
        }
    }

    fn model_view_problem() -> Model {
        let mut model = Model::new();
        let x = model
            .add_variable(Variable::continuous(Bounds::new(0.0, 10.0)))
            .expect("add x");
        let c = model
            .add_constraint(Constraint {
                bounds: Bounds::new(7.0, f64::INFINITY),
            })
            .expect("add c");
        model.set_coefficient(x, c, 1.0).expect("set coeff");
        model
            .set_objective(Objective {
                sense: Some(Sense::Minimize),
                terms: vec![(x, 1.5)],
            })
            .expect("set objective");
        model
    }
}
