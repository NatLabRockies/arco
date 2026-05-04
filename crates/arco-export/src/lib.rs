// thiserror's Display derive triggers unused_assignments in edition 2024
// because derive-generated code no longer inherits item-level #[allow].
#![allow(unused_assignments)]

use arco_kdl::artifacts::{
    AlgebraicProblem, ConstraintSense, LinearTerm, ObjectiveSense, VariableInstance, VariableKind,
};
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
        .filter(|variable| variable.kind == VariableKind::Integer)
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
        .filter(|variable| variable.kind == VariableKind::Binary)
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
        let is_integer = variable.kind != VariableKind::Continuous;
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

fn build_column_terms(
    problem: &AlgebraicProblem,
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

fn lp_objective_header(sense: ObjectiveSense) -> &'static str {
    match sense {
        ObjectiveSense::Minimize => "Minimize",
        ObjectiveSense::Maximize => "Maximize",
    }
}

fn lp_constraint_sense(sense: ConstraintSense) -> &'static str {
    match sense {
        ConstraintSense::GreaterEqual => ">=",
        ConstraintSense::LessEqual => "<=",
        ConstraintSense::Equal => "=",
    }
}

fn mps_row_type(sense: ConstraintSense) -> char {
    match sense {
        ConstraintSense::GreaterEqual => 'G',
        ConstraintSense::LessEqual => 'L',
        ConstraintSense::Equal => 'E',
    }
}

fn write_lp_bounds_line(
    variable: &VariableInstance,
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
    variable: &VariableInstance,
    writer: &mut dyn Write,
) -> Result<(), ExportError> {
    match variable.kind {
        VariableKind::Binary => {
            writeln!(writer, " BV BND1      {}", variable.name).map_err(io_error)?;
            return Ok(());
        }
        VariableKind::Integer | VariableKind::Continuous => {}
    }

    if variable.lower == f64::NEG_INFINITY && variable.upper.is_none() {
        writeln!(writer, " FR BND1      {}", variable.name).map_err(io_error)?;
        return Ok(());
    }

    let lower_code = if variable.kind == VariableKind::Integer {
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
        let upper_code = if variable.kind == VariableKind::Integer {
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

fn format_linear_expression(terms: &[LinearTerm], constant: f64) -> String {
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
