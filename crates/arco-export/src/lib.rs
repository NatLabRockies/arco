// thiserror's Display derive triggers unused_assignments in edition 2024
// because derive-generated code no longer inherits item-level #[allow].
#![allow(unused_assignments)]

use arco_ir::{
    PortableConstraintSense, PortableLinearConstraint, PortableLinearObjective,
    PortableLinearReport, PortableLinearTerm, PortableObjectiveSense, PortableProblem,
    PortableVariableInstance, PortableVariableKind,
};
use arco_targets::{AlgebraicProblem, ConstraintSense, ObjectiveSense, VariableKind};
use miette::Diagnostic;
use std::collections::BTreeMap;
use std::io::Write;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum ExportError {
    #[error("failed to write exported model: {source}")]
    #[diagnostic(
        code(arco::export::io),
        help("verify the destination path is writable")
    )]
    Io {
        #[source]
        source: std::io::Error,
    },
}

pub fn write_lp(problem: &AlgebraicProblem, writer: &mut dyn Write) -> Result<(), ExportError> {
    let portable = portable_problem_from_algebraic(problem);
    write_portable_lp(&portable, writer)
}

pub fn write_portable_lp(
    problem: &PortableProblem,
    writer: &mut dyn Write,
) -> Result<(), ExportError> {
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

pub fn write_mps(problem: &AlgebraicProblem, writer: &mut dyn Write) -> Result<(), ExportError> {
    let portable = portable_problem_from_algebraic(problem);
    write_portable_mps(&portable, writer)
}

pub fn write_portable_mps(
    problem: &PortableProblem,
    writer: &mut dyn Write,
) -> Result<(), ExportError> {
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

fn portable_problem_from_algebraic(problem: &AlgebraicProblem) -> PortableProblem {
    PortableProblem {
        variable_instances: problem
            .variable_instances
            .iter()
            .map(portable_variable_from_algebraic)
            .collect(),
        constraints: problem
            .constraints
            .iter()
            .map(|constraint| PortableLinearConstraint {
                name: constraint.name.clone(),
                sense: portable_constraint_sense(constraint.sense),
                rhs: constraint.rhs,
                terms: portable_terms_from_algebraic(&constraint.terms),
            })
            .collect(),
        objective: PortableLinearObjective {
            name: problem.objective.name.clone(),
            sense: portable_objective_sense(problem.objective.sense),
            constant: problem.objective.constant,
            terms: portable_terms_from_algebraic(&problem.objective.terms),
        },
        reports: problem
            .reports
            .iter()
            .map(|report| PortableLinearReport {
                name: report.name.clone(),
                constant: report.constant,
                terms: portable_terms_from_algebraic(&report.terms),
            })
            .collect(),
    }
}

fn portable_variable_from_algebraic(
    variable: &arco_targets::VariableInstance,
) -> PortableVariableInstance {
    PortableVariableInstance {
        name: variable.name.clone(),
        family: variable.family.clone(),
        lower: variable.lower,
        upper: variable.upper,
        kind: portable_variable_kind(variable.kind),
    }
}

fn portable_terms_from_algebraic(terms: &[arco_targets::LinearTerm]) -> Vec<PortableLinearTerm> {
    terms
        .iter()
        .map(|term| PortableLinearTerm {
            variable_name: term.variable_name.clone(),
            coefficient: term.coefficient,
        })
        .collect()
}

fn portable_variable_kind(kind: VariableKind) -> PortableVariableKind {
    match kind {
        VariableKind::Continuous => PortableVariableKind::Continuous,
        VariableKind::Integer => PortableVariableKind::Integer,
        VariableKind::Binary => PortableVariableKind::Binary,
    }
}

fn portable_constraint_sense(sense: ConstraintSense) -> PortableConstraintSense {
    match sense {
        ConstraintSense::GreaterEqual => PortableConstraintSense::GreaterEqual,
        ConstraintSense::LessEqual => PortableConstraintSense::LessEqual,
        ConstraintSense::Equal => PortableConstraintSense::Equal,
    }
}

fn portable_objective_sense(sense: ObjectiveSense) -> PortableObjectiveSense {
    match sense {
        ObjectiveSense::Minimize => PortableObjectiveSense::Minimize,
        ObjectiveSense::Maximize => PortableObjectiveSense::Maximize,
    }
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
        format!("{:.0}", value)
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
    use super::{write_lp, write_mps, write_portable_lp, write_portable_mps};
    use arco_ir::{
        PortableConstraintSense, PortableLinearConstraint, PortableLinearObjective,
        PortableLinearTerm, PortableObjectiveSense, PortableProblem, PortableVariableInstance,
        PortableVariableKind,
    };
    use arco_targets::{
        AlgebraicProblem, ConstraintSense, LinearConstraint, LinearObjective, LinearTerm,
        ObjectiveSense, VariableInstance, VariableKind,
    };

    #[test]
    fn lp_export_through_portable_ir_preserves_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let algebraic = algebraic_problem();
        let portable = portable_problem();
        let mut algebraic_output = Vec::new();
        let mut portable_output = Vec::new();

        write_lp(&algebraic, &mut algebraic_output)?;
        write_portable_lp(&portable, &mut portable_output)?;

        assert_eq!(algebraic_output, portable_output);
        assert_eq!(String::from_utf8(algebraic_output)?, expected_lp());
        Ok(())
    }

    #[test]
    fn mps_export_through_portable_ir_preserves_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let algebraic = algebraic_problem();
        let portable = portable_problem();
        let mut algebraic_output = Vec::new();
        let mut portable_output = Vec::new();

        write_mps(&algebraic, &mut algebraic_output)?;
        write_portable_mps(&portable, &mut portable_output)?;

        assert_eq!(algebraic_output, portable_output);
        assert_eq!(String::from_utf8(algebraic_output)?, expected_mps());
        Ok(())
    }

    fn algebraic_problem() -> AlgebraicProblem {
        AlgebraicProblem {
            variable_instances: vec![
                VariableInstance {
                    name: "x".to_string(),
                    family: "vars".to_string(),
                    lower: 0.0,
                    upper: Some(10.0),
                    kind: VariableKind::Continuous,
                },
                VariableInstance {
                    name: "y".to_string(),
                    family: "vars".to_string(),
                    lower: -2.0,
                    upper: Some(5.0),
                    kind: VariableKind::Integer,
                },
                VariableInstance {
                    name: "z".to_string(),
                    family: "vars".to_string(),
                    lower: 0.0,
                    upper: Some(1.0),
                    kind: VariableKind::Binary,
                },
            ],
            constraints: vec![
                LinearConstraint {
                    name: "demand".to_string(),
                    sense: ConstraintSense::GreaterEqual,
                    rhs: 7.0,
                    terms: vec![
                        LinearTerm {
                            variable_name: "x".to_string(),
                            coefficient: 1.0,
                        },
                        LinearTerm {
                            variable_name: "y".to_string(),
                            coefficient: 2.0,
                        },
                    ],
                },
                LinearConstraint {
                    name: "capacity".to_string(),
                    sense: ConstraintSense::LessEqual,
                    rhs: 11.5,
                    terms: vec![LinearTerm {
                        variable_name: "z".to_string(),
                        coefficient: -3.0,
                    }],
                },
            ],
            objective: LinearObjective {
                name: "cost".to_string(),
                sense: ObjectiveSense::Minimize,
                constant: 4.0,
                terms: vec![
                    LinearTerm {
                        variable_name: "x".to_string(),
                        coefficient: 1.5,
                    },
                    LinearTerm {
                        variable_name: "y".to_string(),
                        coefficient: -1.0,
                    },
                    LinearTerm {
                        variable_name: "z".to_string(),
                        coefficient: 2.0,
                    },
                ],
            },
            reports: Vec::new(),
        }
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

    fn expected_lp() -> &'static str {
        "\\ Problem name: MODEL\nMinimize\n  cost: 4 + 1.5 x - y + 2 z\nSubject To\n  demand: x + 2 y >= 7\n  capacity: - 3 z <= 11.5\nBounds\n  0 <= x <= 10\n  -2 <= y <= 5\n  0 <= z <= 1\nGenerals\n  y\nBinaries\n  z\nEnd\n"
    }

    fn expected_mps() -> &'static str {
        "NAME          MODEL\nROWS\n N  OBJ\n G  demand\n L  capacity\nCOLUMNS\n    x         OBJ                    1.5  demand                   1\n    MARKER    'MARKER'                 'INTORG'\n    y         OBJ                     -1  demand                   2\n    z         OBJ                      2  capacity                -3\n    MARKER    'MARKER'                 'INTEND'\nRHS\n    RHS1      demand                   7\n    RHS1      capacity              11.5\nBOUNDS\n LO BND1      x         0\n UP BND1      x         10\n LI BND1      y         -2\n UI BND1      y         5\n BV BND1      z\nENDATA\n"
    }
}
