use arco_kdl::artifacts::{AlgebraicProblem, ConstraintSense, ObjectiveSense, VariableKind};
use arco_kdl::pipeline::compile_file;
use miette::{IntoDiagnostic, Result, miette};
use std::collections::BTreeMap;
use std::fs::{File, remove_file};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const ARCO_PYTHON_BINDINGS_SPEC: &str = "arco";

pub fn launch_ipython(path: &Path) -> Result<()> {
    let compiled = compile_file(path)?;
    let model_data = build_python_model_data(&compiled.compiled_problem.algebra)?;
    let script = build_ipython_script(path, &model_data);
    let bootstrap = DebugBootstrapScript::create(&script)?;

    let status = Command::new("uvx")
        .arg("--with")
        .arg(ARCO_PYTHON_BINDINGS_SPEC)
        .arg("ipython")
        .arg("--no-banner")
        .arg("-i")
        .arg(bootstrap.path())
        .status();

    let status = match status {
        Ok(status) => status,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(miette!(
                "failed to start uvx (uv is not installed). Install uv first: https://docs.astral.sh/uv/getting-started/installation/"
            ));
        }
        Err(error) => return Err(error).into_diagnostic(),
    };

    if status.success() {
        Ok(())
    } else {
        Err(miette!(
            "uvx ipython exited with status {}",
            status
                .code()
                .map_or_else(|| "signal".to_string(), |code| code.to_string())
        ))
    }
}

#[derive(Debug, PartialEq)]
struct PythonModelData {
    col_ptrs: Vec<usize>,
    row_indices: Vec<usize>,
    values: Vec<f64>,
    var_lower: Vec<f64>,
    var_upper: Vec<f64>,
    con_lower: Vec<f64>,
    con_upper: Vec<f64>,
    is_integer: Vec<bool>,
    variable_names: Vec<String>,
    constraint_names: Vec<String>,
    objective_sense: ObjectiveSense,
    objective_name: String,
    objective_constant: f64,
    objective_terms: Vec<(usize, f64)>,
}

fn build_python_model_data(problem: &AlgebraicProblem) -> Result<PythonModelData> {
    let variable_indices = problem
        .variable_instances
        .iter()
        .enumerate()
        .map(|(index, variable)| (variable.name.clone(), index))
        .collect::<BTreeMap<_, _>>();

    let mut columns = vec![Vec::new(); problem.variable_instances.len()];
    for (row_index, constraint) in problem.constraints.iter().enumerate() {
        for term in &constraint.terms {
            let column_index = lookup_variable_index(
                &variable_indices,
                &term.variable_name,
                "debug bootstrap references unknown lowered variable",
            )?;
            columns[column_index].push((row_index, term.coefficient));
        }
    }

    let mut col_ptrs = Vec::with_capacity(problem.variable_instances.len() + 1);
    let mut row_indices = Vec::new();
    let mut values = Vec::new();
    col_ptrs.push(0);
    for column in columns {
        for (row_index, value) in column {
            row_indices.push(row_index);
            values.push(value);
        }
        col_ptrs.push(row_indices.len());
    }

    let var_lower = problem
        .variable_instances
        .iter()
        .map(|variable| variable.lower)
        .collect();
    let var_upper = problem
        .variable_instances
        .iter()
        .map(|variable| variable.upper.unwrap_or(f64::INFINITY))
        .collect();
    let con_lower = problem
        .constraints
        .iter()
        .map(|constraint| match constraint.sense {
            ConstraintSense::GreaterEqual | ConstraintSense::Equal => constraint.rhs,
            ConstraintSense::LessEqual => f64::NEG_INFINITY,
        })
        .collect();
    let con_upper = problem
        .constraints
        .iter()
        .map(|constraint| match constraint.sense {
            ConstraintSense::LessEqual | ConstraintSense::Equal => constraint.rhs,
            ConstraintSense::GreaterEqual => f64::INFINITY,
        })
        .collect();
    let is_integer = problem
        .variable_instances
        .iter()
        .map(|variable| matches!(variable.kind, VariableKind::Integer | VariableKind::Binary))
        .collect();
    let variable_names = problem
        .variable_instances
        .iter()
        .map(|variable| variable.name.clone())
        .collect();
    let constraint_names = problem
        .constraints
        .iter()
        .map(|constraint| constraint.name.clone())
        .collect();
    let objective_terms = problem
        .objective
        .terms
        .iter()
        .map(|term| {
            let index = lookup_variable_index(
                &variable_indices,
                &term.variable_name,
                "debug bootstrap objective references unknown lowered variable",
            )?;
            Ok((index, term.coefficient))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(PythonModelData {
        col_ptrs,
        row_indices,
        values,
        var_lower,
        var_upper,
        con_lower,
        con_upper,
        is_integer,
        variable_names,
        constraint_names,
        objective_sense: problem.objective.sense,
        objective_name: problem.objective.name.clone(),
        objective_constant: problem.objective.constant,
        objective_terms,
    })
}

fn lookup_variable_index(
    variable_indices: &BTreeMap<String, usize>,
    variable_name: &str,
    context: &str,
) -> Result<usize> {
    variable_indices
        .get(variable_name)
        .copied()
        .ok_or_else(|| miette!("{context} `{variable_name}`"))
}

fn build_ipython_script(path: &Path, model: &PythonModelData) -> String {
    format!(
        "from pathlib import Path\n\nimport arco\n\nmodel_path = Path(r{path})\nmodel = arco.Model.from_csc(\n    num_constraints={num_constraints},\n    num_variables={num_variables},\n    col_ptrs={col_ptrs},\n    row_indices={row_indices},\n    values={values},\n    var_lower={var_lower},\n    var_upper={var_upper},\n    con_lower={con_lower},\n    con_upper={con_upper},\n    is_integer={is_integer},\n)\nfor index, name in enumerate({variable_names}):\n    model.set_variable_name(index, name=name)\nfor index, name in enumerate({constraint_names}):\n    model.set_constraint_name(index, name=name)\nmodel.set_objective(\n    arco.Sense.{objective_sense},\n    {objective_terms},\n    name={objective_name},\n)\nobjective_constant = {objective_constant}\n",
        path = format_python_string(&path.display().to_string()),
        num_constraints = model.constraint_names.len(),
        num_variables = model.variable_names.len(),
        col_ptrs = format_python_usize_list(&model.col_ptrs),
        row_indices = format_python_usize_list(&model.row_indices),
        values = format_python_f64_list(&model.values),
        var_lower = format_python_f64_list(&model.var_lower),
        var_upper = format_python_f64_list(&model.var_upper),
        con_lower = format_python_f64_list(&model.con_lower),
        con_upper = format_python_f64_list(&model.con_upper),
        is_integer = format_python_bool_list(&model.is_integer),
        variable_names = format_python_string_list(&model.variable_names),
        constraint_names = format_python_string_list(&model.constraint_names),
        objective_sense = match model.objective_sense {
            ObjectiveSense::Minimize => "MINIMIZE",
            ObjectiveSense::Maximize => "MAXIMIZE",
        },
        objective_terms = format_python_tuple_list(&model.objective_terms),
        objective_name = format_python_string(&model.objective_name),
        objective_constant = format_python_f64(model.objective_constant),
    )
}

fn format_python_string(value: &str) -> String {
    // Hand-rolled to avoid expect/unwrap on serde_json::to_string.
    let mut encoded = String::with_capacity(value.len() + 2);
    encoded.push('"');
    for character in value.chars() {
        match character {
            '\\' => encoded.push_str("\\\\"),
            '"' => encoded.push_str("\\\""),
            '\n' => encoded.push_str("\\n"),
            '\r' => encoded.push_str("\\r"),
            '\t' => encoded.push_str("\\t"),
            _ => encoded.push(character),
        }
    }
    encoded.push('"');
    encoded
}

fn format_python_string_list(values: &[String]) -> String {
    format_python_list(values, |value| format_python_string(value))
}

fn format_python_usize_list(values: &[usize]) -> String {
    format_python_list(values, |value| value.to_string())
}

fn format_python_f64_list(values: &[f64]) -> String {
    format_python_list(values, |value| format_python_f64(*value))
}

fn format_python_bool_list(values: &[bool]) -> String {
    format_python_list(values, |value| {
        (if *value { "True" } else { "False" }).to_string()
    })
}

fn format_python_tuple_list(values: &[(usize, f64)]) -> String {
    format_python_list(values, |(index, coefficient)| {
        format!("({}, {})", index, format_python_f64(*coefficient))
    })
}

fn format_python_list<T>(values: &[T], format_item: impl Fn(&T) -> String) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(format_item)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn format_python_f64(value: f64) -> String {
    if value == f64::INFINITY {
        "float(\"inf\")".to_string()
    } else if value == f64::NEG_INFINITY {
        "float(\"-inf\")".to_string()
    } else {
        value.to_string()
    }
}

struct DebugBootstrapScript {
    path: PathBuf,
}

impl DebugBootstrapScript {
    fn create(contents: &str) -> Result<Self> {
        let base_name = format!(
            "arco-debug-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .into_diagnostic()?
                .as_nanos()
        );

        for attempt in 0..16 {
            let path = std::env::temp_dir().join(format!("{base_name}-{attempt}.py"));
            match File::options().write(true).create_new(true).open(&path) {
                Ok(mut file) => {
                    file.write_all(contents.as_bytes()).into_diagnostic()?;
                    return Ok(Self { path });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error).into_diagnostic(),
            }
        }

        Err(miette!("failed to create temporary debug bootstrap script"))
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for DebugBootstrapScript {
    fn drop(&mut self) {
        let _ = remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use crate::debug_shell::{PythonModelData, build_ipython_script, build_python_model_data};
    use arco_kdl::artifacts::{
        AlgebraicProblem, ConstraintSense, LinearConstraint, LinearObjective, LinearTerm,
        ObjectiveSense, VariableInstance, VariableKind,
    };
    use std::path::Path;

    #[test]
    fn python_model_data_uses_csc_layout_and_bounds() {
        let problem = AlgebraicProblem {
            variable_instances: vec![
                VariableInstance {
                    name: "x".to_string(),
                    family: "x".to_string(),
                    lower: 0.0,
                    upper: Some(4.0),
                    kind: VariableKind::Continuous,
                },
                VariableInstance {
                    name: "y".to_string(),
                    family: "y".to_string(),
                    lower: 0.0,
                    upper: Some(1.0),
                    kind: VariableKind::Binary,
                },
            ],
            constraints: vec![
                LinearConstraint {
                    name: "c1".to_string(),
                    sense: ConstraintSense::LessEqual,
                    rhs: 5.0,
                    terms: vec![
                        LinearTerm {
                            variable_name: "x".to_string(),
                            coefficient: 2.0,
                        },
                        LinearTerm {
                            variable_name: "y".to_string(),
                            coefficient: 3.0,
                        },
                    ],
                },
                LinearConstraint {
                    name: "c2".to_string(),
                    sense: ConstraintSense::GreaterEqual,
                    rhs: 1.0,
                    terms: vec![LinearTerm {
                        variable_name: "y".to_string(),
                        coefficient: 4.0,
                    }],
                },
            ],
            objective: LinearObjective {
                name: "profit".to_string(),
                sense: ObjectiveSense::Maximize,
                constant: 7.5,
                terms: vec![
                    LinearTerm {
                        variable_name: "x".to_string(),
                        coefficient: 1.0,
                    },
                    LinearTerm {
                        variable_name: "y".to_string(),
                        coefficient: 9.0,
                    },
                ],
            },
            reports: Vec::new(),
        };

        let model = build_python_model_data(&problem).expect("model data should build");

        assert_eq!(
            model,
            PythonModelData {
                col_ptrs: vec![0, 1, 3],
                row_indices: vec![0, 0, 1],
                values: vec![2.0, 3.0, 4.0],
                var_lower: vec![0.0, 0.0],
                var_upper: vec![4.0, 1.0],
                con_lower: vec![f64::NEG_INFINITY, 1.0],
                con_upper: vec![5.0, f64::INFINITY],
                is_integer: vec![false, true],
                variable_names: vec!["x".to_string(), "y".to_string()],
                constraint_names: vec!["c1".to_string(), "c2".to_string()],
                objective_sense: ObjectiveSense::Maximize,
                objective_name: "profit".to_string(),
                objective_constant: 7.5,
                objective_terms: vec![(0, 1.0), (1, 9.0)],
            }
        );
    }

    #[test]
    fn ipython_script_preloads_arco_and_model() {
        let model = PythonModelData {
            col_ptrs: vec![0, 1],
            row_indices: vec![0],
            values: vec![1.0],
            var_lower: vec![0.0],
            var_upper: vec![f64::INFINITY],
            con_lower: vec![f64::NEG_INFINITY],
            con_upper: vec![2.0],
            is_integer: vec![false],
            variable_names: vec!["dispatch[Battery1,1]".to_string()],
            constraint_names: vec!["limit[Battery1,1]".to_string()],
            objective_sense: ObjectiveSense::Minimize,
            objective_name: "cost".to_string(),
            objective_constant: 0.0,
            objective_terms: vec![(0, 1.0)],
        };

        let model_path = "examples/price-taker-battery/input.kdl";
        let script = build_ipython_script(Path::new(model_path), &model);

        assert!(script.contains("import arco"));
        // The model path is embedded as a raw string literal.
        assert!(script.contains(&format!(r#"model_path = Path(r"{model_path}")"#)));
        // CSC matrix data: col_ptrs, row_indices, values from the test model.
        // Note: Rust's f64::to_string() omits the trailing .0 for whole numbers.
        assert!(script.contains("col_ptrs=[0, 1]"));
        assert!(script.contains("row_indices=[0]"));
        assert!(script.contains("values=[1]"));
        // Variable bounds: lower=0.0 (serialized as "0"), upper=inf.
        assert!(script.contains("var_lower=[0]"));
        assert!(script.contains("var_upper=[float(\"inf\")]"));
        // Constraint bounds: lower=-inf (no lower bound), upper=2.0 (serialized as "2").
        assert!(script.contains("con_lower=[float(\"-inf\")]"));
        assert!(script.contains("con_upper=[2]"));
        // Integer flags: the single variable is continuous.
        assert!(script.contains("is_integer=[False]"));
        // Variable name is embedded in the enumerate call.
        assert!(script.contains(r#"["dispatch[Battery1,1]"]"#));
        // Constraint name is embedded in the enumerate call.
        assert!(script.contains(r#"["limit[Battery1,1]"]"#));
        // Objective sense and term: MINIMIZE with coefficient 1.0 on variable 0.
        // Rust serializes 1.0f64 as "1" via to_string().
        assert!(script.contains("arco.Sense.MINIMIZE"));
        assert!(script.contains("[(0, 1)]"));
        assert!(script.contains(r#"name="cost""#));
        assert!(script.contains("objective_constant = 0"));
    }
}
