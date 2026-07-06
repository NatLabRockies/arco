//! Module-level solution summary helper.

use arco_python_core::PySolveResult;
use arco_solver::SolverStatus;
use pyo3::prelude::*;

fn format_solve_time(seconds: f64) -> String {
    if seconds < 1.0 {
        format!("{:.2}ms", seconds * 1000.0)
    } else {
        format!("{:.2}s", seconds)
    }
}

fn format_sci(val: f64) -> String {
    if val.is_nan() {
        return "NaN".to_string();
    }
    if val.is_infinite() {
        return if val > 0.0 {
            "inf".to_string()
        } else {
            "-inf".to_string()
        };
    }

    let text = format!("{:.5e}", val);
    if let Some(pos) = text.rfind('e') {
        let mantissa = &text[..pos];
        let exp_str = &text[pos + 1..];
        let exp = match exp_str.parse::<i32>() {
            Ok(exp) => exp,
            Err(_) => return text,
        };
        format!("{}e{:+03}", mantissa, exp)
    } else {
        text
    }
}

fn status_str(status: SolverStatus) -> &'static str {
    match status {
        SolverStatus::Optimal => "OPTIMAL",
        SolverStatus::Infeasible => "INFEASIBLE",
        SolverStatus::Unbounded => "UNBOUNDED",
        SolverStatus::TimeLimit => "TIME_LIMIT",
        SolverStatus::IterationLimit => "ITERATION_LIMIT",
        SolverStatus::Unknown => "ERROR",
    }
}

/// Pretty-print a tree-formatted solution summary.
#[pyo3_macros::pyfunction]
#[pyo3(signature = (result, *, verbose=false))]
pub fn solution_summary(
    py: Python<'_>,
    result: PyRef<'_, PySolveResult>,
    verbose: bool,
) -> PyResult<()> {
    let sol = result.inner();
    let mut lines = Vec::new();

    lines.push("Solution Summary".to_string());
    lines.push("\u{251c} solver          : HiGHS".to_string());

    let is_last_section = !verbose;
    let term_prefix = if is_last_section {
        "\u{2514}"
    } else {
        "\u{251c}"
    };
    let term_cont = if is_last_section { " " } else { "\u{2502}" };
    lines.push(format!("{} Termination", term_prefix));
    lines.push(format!(
        "{} \u{251c} status        : {}",
        term_cont,
        status_str(sol.status)
    ));
    lines.push(format!(
        "{} \u{2514} objective     : {}",
        term_cont,
        format_sci(sol.objective_value)
    ));

    if verbose {
        lines.push("\u{251c} Solution".to_string());

        let has_duals = !sol.constraint_duals.is_empty();
        let values_prefix = if has_duals { "\u{251c}" } else { "\u{2514}" };
        lines.push(format!("\u{2502} {} values", values_prefix));

        let val_cont = if has_duals { "\u{2502}" } else { " " };
        let num_vals = sol.primal_values.len();
        for (i, val) in sol.primal_values.iter().enumerate() {
            let is_last = i + 1 == num_vals;
            let branch = if is_last { "\u{2514}" } else { "\u{251c}" };
            lines.push(format!(
                "\u{2502} {}  {} x{:<12}: {}",
                val_cont,
                branch,
                i,
                format_sci(*val)
            ));
        }

        if has_duals {
            lines.push("\u{2502} \u{2514} duals".to_string());
            let num_duals = sol.constraint_duals.len();
            for (i, val) in sol.constraint_duals.iter().enumerate() {
                let is_last = i + 1 == num_duals;
                let branch = if is_last { "\u{2514}" } else { "\u{251c}" };
                lines.push(format!(
                    "\u{2502}   {} c{:<12}: {}",
                    branch,
                    i,
                    format_sci(*val)
                ));
            }
        }

        let iterations = sol
            .metadata
            .get("simplex_iterations")
            .copied()
            .unwrap_or(0.0) as u64
            + sol
                .metadata
                .get("barrier_iterations")
                .copied()
                .unwrap_or(0.0) as u64;
        let nodes = sol.metadata.get("nodes").copied().unwrap_or(0.0) as u64;

        lines.push("\u{2514} Work".to_string());
        lines.push(format!(
            "  \u{251c} solve_time    : {}",
            format_solve_time(sol.solve_time_seconds)
        ));
        lines.push(format!("  \u{251c} iterations    : {}", iterations));
        lines.push(format!("  \u{2514} nodes         : {}", nodes));
    }

    let output = lines.join("\n");
    let builtins = py.import("builtins")?;
    builtins.call_method1("print", (output,))?;
    Ok(())
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(solution_summary, m)?)?;
    Ok(())
}
