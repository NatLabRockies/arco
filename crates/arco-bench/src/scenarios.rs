use crate::types::{BenchRecord, CaseConfig, CaseExecution, CscMatrix, SCHEMA_VERSION, Scenario};
use arco_core::model::{CscMatrix as CoreCscMatrix, SparseMatrixExport};
use arco_core::types::Bounds;
use arco_core::{Constraint, Model, Objective, Sense, Variable};
use arco_expr::{ConstraintId, VariableId};
use arco_tools::{MeasurementRecorder, StageMeasurement, capture_rss_bytes, rss_delta};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const FAC25_VARIABLES: usize = 67_651;
const DEFAULT_CASES: [usize; 5] = [100, 1_000, 10_000, 100_000, 1_000_000];

pub(crate) fn resolve_cases(
    scenario: Scenario,
    variables: Option<usize>,
    constraints: Option<usize>,
    cases: Option<&[usize]>,
) -> Vec<CaseConfig> {
    match scenario {
        Scenario::ModelBuild => {
            if let Some(variables) = variables {
                return vec![CaseConfig {
                    name: format!("vars_{}", variables),
                    variables,
                    constraints,
                }];
            }

            cases
                .unwrap_or(&DEFAULT_CASES)
                .iter()
                .copied()
                .map(|variables| CaseConfig {
                    name: format!("vars_{}", variables),
                    variables,
                    constraints: None,
                })
                .collect()
        }
        Scenario::Fac25 => vec![CaseConfig {
            name: "fac25".to_string(),
            variables: FAC25_VARIABLES,
            constraints: None,
        }],
    }
}

pub(crate) fn execute_case(
    variable_count: usize,
    constraint_override: Option<usize>,
    constraint_ratio: f64,
    collect_csc: bool,
) -> CaseExecution {
    let constraint_count = constraint_override
        .unwrap_or_else(|| {
            let raw = (variable_count as f64 * constraint_ratio).round() as usize;
            raw.max(1)
        })
        .max(1);

    let mut model = Model::with_capacities(variable_count, constraint_count);
    let mut recorder = MeasurementRecorder::new();

    let total_started = Instant::now();
    let total_rss_before = capture_rss_bytes("bench_total");

    let stage_start = recorder.begin_stage("variables");
    for _ in 0..variable_count {
        if model
            .add_variable(Variable {
                bounds: Bounds::new(0.0, 1_000.0),
                is_integer: false,
                is_active: true,
            })
            .is_err()
        {
            break;
        }
    }
    recorder.end_stage(stage_start);

    let stage_start = recorder.begin_stage("constraints");
    for _ in 0..constraint_count {
        if model
            .add_constraint(Constraint {
                bounds: Bounds::new(0.0, 10_000.0),
            })
            .is_err()
        {
            break;
        }
    }
    recorder.end_stage(stage_start);

    let limit = model.num_constraints().min(model.num_variables());
    let stage_start = recorder.begin_stage("coefficients");
    for idx in 0..limit {
        let var = VariableId::new(idx as u32);
        let con = ConstraintId::new(idx as u32);
        if model.set_coefficient(var, con, 1.0).is_err() {
            break;
        }
    }
    recorder.end_stage(stage_start);

    let stage_start = recorder.begin_stage("objective");
    let objective_terms: Vec<(VariableId, f64)> = (0..limit)
        .map(|idx| (VariableId::new(idx as u32), 1.0))
        .collect();
    let _ = model.set_objective(Objective {
        sense: Some(Sense::Minimize),
        terms: objective_terms,
    });
    recorder.end_stage(stage_start);

    let exported_csc = if collect_csc {
        Some(measure_stage(&mut recorder, "export_csc", || {
            model.export_csc()
        }))
    } else {
        measure_stage_discard(&mut recorder, "export_csc", || model.export_csc());
        None
    };
    measure_stage_discard(&mut recorder, "export_crs", || model.export_crs());
    measure_stage_discard(&mut recorder, "export_coo", || model.export_coo());

    let total_duration = total_started.elapsed();
    let total_rss_after = capture_rss_bytes("bench_total");

    let mut stages = recorder.stages().to_vec();
    stages.push(StageMeasurement {
        stage: "total".to_string(),
        duration: total_duration,
        rss_before_bytes: total_rss_before,
        rss_after_bytes: total_rss_after,
        rss_delta_bytes: rss_delta(total_rss_before, total_rss_after),
    });

    let csc = exported_csc.map(convert_csc_matrix);

    CaseExecution {
        variables: model.num_variables(),
        constraints: model.num_constraints(),
        stage_measurements: stages,
        csc,
    }
}

pub(crate) fn case_records(
    run_id: &str,
    scenario: Scenario,
    case_name: &str,
    repetition: u32,
    execution: &CaseExecution,
) -> Vec<BenchRecord> {
    execution
        .stage_measurements
        .iter()
        .map(|measurement| BenchRecord {
            schema_version: SCHEMA_VERSION,
            run_id: run_id.to_string(),
            scenario: scenario.as_str().to_string(),
            case_name: case_name.to_string(),
            repetition,
            variables: execution.variables,
            constraints: execution.constraints,
            stage: measurement.stage.clone(),
            duration_ms: measurement.duration.as_secs_f64() * 1000.0,
            rss_before_bytes: measurement.rss_before_bytes,
            rss_after_bytes: measurement.rss_after_bytes,
            rss_delta_bytes: measurement.rss_delta_bytes,
        })
        .collect()
}

pub(crate) fn build_run_id() -> Result<String, Box<dyn std::error::Error>> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| std::io::Error::other(err.to_string()))?
        .as_millis();
    Ok(format!("bench_{}", millis))
}

fn measure_stage<T>(
    recorder: &mut MeasurementRecorder,
    stage: &str,
    operation: impl FnOnce() -> T,
) -> T {
    let stage_start = recorder.begin_stage(stage);
    let output = operation();
    recorder.end_stage(stage_start);
    output
}

fn measure_stage_discard<T>(
    recorder: &mut MeasurementRecorder,
    stage: &str,
    operation: impl FnOnce() -> T,
) {
    let stage_start = recorder.begin_stage(stage);
    let output = operation();
    drop(output);
    recorder.end_stage(stage_start);
}

fn convert_csc_matrix(matrix: CoreCscMatrix) -> CscMatrix {
    CscMatrix {
        col_ptrs: matrix.col_ptrs.into_iter().map(|ptr| ptr as u64).collect(),
        row_indices: matrix.row_indices,
        values: matrix.values,
    }
}

#[cfg(test)]
mod tests {
    use crate::scenarios::resolve_cases;
    use crate::types::Scenario;

    #[test]
    fn resolve_cases_uses_single_variable_override() {
        let cases = resolve_cases(Scenario::ModelBuild, Some(123), Some(9), None);

        assert_eq!(cases.len(), 1);
        assert_eq!(cases[0].name, "vars_123");
        assert_eq!(cases[0].variables, 123);
        assert_eq!(cases[0].constraints, Some(9));
    }

    #[test]
    fn resolve_cases_uses_default_model_build_cases() {
        let cases = resolve_cases(Scenario::ModelBuild, None, None, None);

        assert_eq!(cases.len(), 5);
        assert_eq!(cases[0].name, "vars_100");
        assert_eq!(cases[4].name, "vars_1000000");
        assert!(cases.iter().all(|case| case.constraints.is_none()));
    }

    #[test]
    fn resolve_cases_supports_fac25_scenario() {
        let cases = resolve_cases(Scenario::Fac25, Some(999), Some(1), Some(&[10, 20]));

        assert_eq!(cases.len(), 1);
        assert_eq!(cases[0].name, "fac25");
        assert_eq!(cases[0].variables, 67_651);
        assert_eq!(cases[0].constraints, None);
    }
}
