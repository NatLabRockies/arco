use arco_core::types::Bounds;
use arco_core::{Constraint, Model, Objective, Sense, Variable};
use arco_expr::{ConstraintId, VariableId};
use arco_tools::{MeasurementRecorder, StageMeasurement, capture_rss_bytes, rss_delta};
use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{File, create_dir_all};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const FAC25_VARIABLES: usize = 67_651;
const DEFAULT_CASES: [usize; 5] = [100, 1_000, 10_000, 100_000, 1_000_000];
const SCHEMA_VERSION: u32 = 1;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Arco benchmark runner and reporting interface"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Execute benchmark scenarios and save JSONL artifacts
    Run(RunArgs),
    /// Render benchmark artifact summaries
    Report(ReportArgs),
    /// Compare two benchmark artifacts and optionally enforce thresholds
    Compare(CompareArgs),
}

#[derive(Parser, Debug)]
struct RunArgs {
    /// Benchmark scenarios to execute
    #[arg(
        long = "scenario",
        value_enum,
        value_delimiter = ',',
        default_value = "model-build"
    )]
    scenarios: Vec<Scenario>,

    /// Comma-separated list of variable counts for model-build scenario
    #[arg(long, value_delimiter = ',')]
    cases: Option<Vec<usize>>,

    /// Run a single model-build case with this variable count
    #[arg(long)]
    variables: Option<usize>,

    /// Override number of constraints for --variables
    #[arg(long, requires = "variables")]
    constraints: Option<usize>,

    /// Ratio of constraints per variable when explicit constraints are not provided
    #[arg(long, default_value_t = 0.01)]
    constraint_ratio: f64,

    /// Number of repetitions per case
    #[arg(long, default_value_t = 1)]
    repetitions: u32,

    /// JSONL output artifact path
    #[arg(long)]
    output: Option<PathBuf>,

    /// Output format for stdout
    #[arg(long, value_enum, default_value = "table")]
    format: OutputFormat,

    /// Directory to write generated CSC matrix artifacts
    #[arg(long)]
    write_csc: Option<PathBuf>,
}

#[derive(Parser, Debug)]
struct ReportArgs {
    /// Input JSONL benchmark artifact
    #[arg(long)]
    input: PathBuf,

    /// Output format for stdout
    #[arg(long, value_enum, default_value = "table")]
    format: OutputFormat,
}

#[derive(Parser, Debug)]
struct CompareArgs {
    /// Baseline JSONL benchmark artifact
    #[arg(long)]
    baseline: PathBuf,

    /// Candidate JSONL benchmark artifact
    #[arg(long)]
    candidate: PathBuf,

    /// Stage filter for comparison (for example, total)
    #[arg(long, default_value = "total")]
    stage: String,

    /// Fail if duration regression exceeds this percentage
    #[arg(long)]
    duration_threshold_pct: Option<f64>,

    /// Fail if memory regression exceeds this percentage
    #[arg(long)]
    memory_threshold_pct: Option<f64>,

    /// Output format for stdout
    #[arg(long, value_enum, default_value = "table")]
    format: OutputFormat,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, ValueEnum)]
enum OutputFormat {
    Table,
    Json,
    Ndjson,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, ValueEnum)]
enum Scenario {
    ModelBuild,
    Fac25,
}

impl Scenario {
    fn as_str(self) -> &'static str {
        match self {
            Scenario::ModelBuild => "model-build",
            Scenario::Fac25 => "fac25",
        }
    }
}

#[derive(Debug, Clone)]
struct CaseConfig {
    name: String,
    variables: usize,
    constraints: Option<usize>,
}

#[derive(Debug, Clone)]
struct CscMatrix {
    col_ptrs: Vec<u64>,
    row_indices: Vec<u32>,
    values: Vec<f64>,
}

#[derive(Debug, Clone)]
struct CaseExecution {
    variables: usize,
    constraints: usize,
    stage_measurements: Vec<StageMeasurement>,
    csc: Option<CscMatrix>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BenchRecord {
    schema_version: u32,
    run_id: String,
    scenario: String,
    case_name: String,
    repetition: u32,
    variables: usize,
    constraints: usize,
    stage: String,
    duration_ms: f64,
    rss_before_bytes: Option<u64>,
    rss_after_bytes: Option<u64>,
    rss_delta_bytes: Option<i64>,
}

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
struct SummaryKey {
    scenario: String,
    case_name: String,
    stage: String,
}

#[derive(Debug, Clone, Serialize)]
struct SummaryRow {
    scenario: String,
    case_name: String,
    stage: String,
    samples: usize,
    mean_duration_ms: f64,
    max_duration_ms: f64,
    mean_rss_delta_bytes: Option<f64>,
    max_rss_after_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
struct CompareRow {
    scenario: String,
    case_name: String,
    stage: String,
    baseline_mean_duration_ms: f64,
    candidate_mean_duration_ms: f64,
    duration_change_pct: Option<f64>,
    baseline_mean_rss_delta_bytes: Option<f64>,
    candidate_mean_rss_delta_bytes: Option<f64>,
    rss_change_pct: Option<f64>,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Command::Run(args) => run_command(args),
        Command::Report(args) => report_command(args),
        Command::Compare(args) => compare_command(args),
    }
}

fn run_command(args: RunArgs) -> Result<(), Box<dyn std::error::Error>> {
    if args.repetitions == 0 {
        return Err(boxed_input_error("repetitions must be greater than zero"));
    }
    if args.constraint_ratio <= 0.0 {
        return Err(boxed_input_error(
            "constraint-ratio must be greater than zero",
        ));
    }

    let run_id = build_run_id()?;
    let output_path = args
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from(format!("artifacts/bench/{}.jsonl", run_id.as_str())));

    let mut records = Vec::new();

    for scenario in &args.scenarios {
        let cases = resolve_cases(*scenario, &args);
        for case in cases {
            for rep_idx in 0..args.repetitions {
                let execution = execute_case(
                    case.variables,
                    case.constraints,
                    args.constraint_ratio,
                    args.write_csc.is_some(),
                );
                if let (Some(base_dir), Some(csc)) =
                    (args.write_csc.as_ref(), execution.csc.as_ref())
                {
                    let dir = base_dir
                        .join(scenario.as_str())
                        .join(&case.name)
                        .join(format!("rep_{}", rep_idx + 1));
                    write_csc_matrix(&dir, csc, execution.variables, execution.constraints)?;
                }
                records.extend(case_records(
                    &run_id,
                    *scenario,
                    &case.name,
                    rep_idx + 1,
                    &execution,
                ));
            }
        }
    }

    write_records_jsonl(&output_path, &records)?;
    render_output(args.format, &records)?;
    println!("artifact: {}", output_path.display());

    Ok(())
}

fn report_command(args: ReportArgs) -> Result<(), Box<dyn std::error::Error>> {
    let records = load_records_jsonl(&args.input)?;
    render_output(args.format, &records)?;
    Ok(())
}

fn compare_command(args: CompareArgs) -> Result<(), Box<dyn std::error::Error>> {
    let baseline_records = load_records_jsonl(&args.baseline)?;
    let candidate_records = load_records_jsonl(&args.candidate)?;

    let baseline_summary = summarize_records(&baseline_records);
    let candidate_summary = summarize_records(&candidate_records);
    let rows = build_comparison_rows(&baseline_summary, &candidate_summary, &args.stage);

    if rows.is_empty() {
        return Err(boxed_input_error(
            "no overlapping scenario/case/stage rows to compare",
        ));
    }

    render_compare_output(args.format, &rows)?;
    if has_regressions(
        &rows,
        args.duration_threshold_pct,
        args.memory_threshold_pct,
    ) {
        return Err(boxed_input_error(
            "regression threshold violated (see compare output)",
        ));
    }

    Ok(())
}

fn resolve_cases(scenario: Scenario, args: &RunArgs) -> Vec<CaseConfig> {
    match scenario {
        Scenario::ModelBuild => {
            if let Some(variables) = args.variables {
                return vec![CaseConfig {
                    name: format!("vars_{}", variables),
                    variables,
                    constraints: args.constraints,
                }];
            }

            args.cases
                .clone()
                .unwrap_or_else(|| DEFAULT_CASES.to_vec())
                .into_iter()
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

fn execute_case(
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

    let csc = if collect_csc {
        Some(extract_csc_matrix(&model))
    } else {
        None
    };

    CaseExecution {
        variables: model.num_variables(),
        constraints: model.num_constraints(),
        stage_measurements: stages,
        csc,
    }
}

fn extract_csc_matrix(model: &Model) -> CscMatrix {
    let mut col_ptrs = Vec::with_capacity(model.num_variables() + 1);
    let mut row_indices = Vec::new();
    let mut values = Vec::new();
    col_ptrs.push(0);

    for (_, column) in model.columns() {
        for (constraint_id, coeff) in column {
            row_indices.push(constraint_id.inner());
            values.push(*coeff);
        }
        col_ptrs.push(row_indices.len() as u64);
    }

    CscMatrix {
        col_ptrs,
        row_indices,
        values,
    }
}

fn case_records(
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

fn render_output(
    format: OutputFormat,
    records: &[BenchRecord],
) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Table => {
            let rows = summarize_records(records);
            print_summary_table(&rows);
            Ok(())
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(records)?);
            Ok(())
        }
        OutputFormat::Ndjson => {
            for record in records {
                println!("{}", serde_json::to_string(record)?);
            }
            Ok(())
        }
    }
}

fn render_compare_output(
    format: OutputFormat,
    rows: &[CompareRow],
) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Table => {
            print_compare_table(rows);
            Ok(())
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(rows)?);
            Ok(())
        }
        OutputFormat::Ndjson => {
            for row in rows {
                println!("{}", serde_json::to_string(row)?);
            }
            Ok(())
        }
    }
}

fn summarize_records(records: &[BenchRecord]) -> Vec<SummaryRow> {
    #[derive(Default)]
    struct Acc {
        samples: usize,
        duration_sum: f64,
        duration_max: f64,
        rss_delta_sum: f64,
        rss_delta_count: usize,
        rss_after_max: Option<u64>,
    }

    let mut groups: BTreeMap<SummaryKey, Acc> = BTreeMap::new();
    for record in records {
        let key = SummaryKey {
            scenario: record.scenario.clone(),
            case_name: record.case_name.clone(),
            stage: record.stage.clone(),
        };
        let entry = groups.entry(key).or_default();
        entry.samples += 1;
        entry.duration_sum += record.duration_ms;
        if record.duration_ms > entry.duration_max {
            entry.duration_max = record.duration_ms;
        }
        if let Some(delta) = record.rss_delta_bytes {
            entry.rss_delta_sum += delta as f64;
            entry.rss_delta_count += 1;
        }
        entry.rss_after_max = match (entry.rss_after_max, record.rss_after_bytes) {
            (Some(current), Some(next)) => Some(current.max(next)),
            (None, Some(next)) => Some(next),
            (current, None) => current,
        };
    }

    groups
        .into_iter()
        .map(|(key, acc)| SummaryRow {
            scenario: key.scenario,
            case_name: key.case_name,
            stage: key.stage,
            samples: acc.samples,
            mean_duration_ms: if acc.samples == 0 {
                0.0
            } else {
                acc.duration_sum / acc.samples as f64
            },
            max_duration_ms: acc.duration_max,
            mean_rss_delta_bytes: if acc.rss_delta_count == 0 {
                None
            } else {
                Some(acc.rss_delta_sum / acc.rss_delta_count as f64)
            },
            max_rss_after_bytes: acc.rss_after_max,
        })
        .collect()
}

fn build_comparison_rows(
    baseline_summary: &[SummaryRow],
    candidate_summary: &[SummaryRow],
    stage_filter: &str,
) -> Vec<CompareRow> {
    let mut baseline_map: BTreeMap<SummaryKey, &SummaryRow> = BTreeMap::new();
    for row in baseline_summary {
        if row.stage == stage_filter {
            let key = SummaryKey {
                scenario: row.scenario.clone(),
                case_name: row.case_name.clone(),
                stage: row.stage.clone(),
            };
            baseline_map.insert(key, row);
        }
    }

    let mut rows = Vec::new();
    for candidate in candidate_summary {
        if candidate.stage != stage_filter {
            continue;
        }
        let key = SummaryKey {
            scenario: candidate.scenario.clone(),
            case_name: candidate.case_name.clone(),
            stage: candidate.stage.clone(),
        };
        let Some(baseline) = baseline_map.get(&key) else {
            continue;
        };
        rows.push(CompareRow {
            scenario: key.scenario,
            case_name: key.case_name,
            stage: key.stage,
            baseline_mean_duration_ms: baseline.mean_duration_ms,
            candidate_mean_duration_ms: candidate.mean_duration_ms,
            duration_change_pct: percent_change(
                baseline.mean_duration_ms,
                candidate.mean_duration_ms,
            ),
            baseline_mean_rss_delta_bytes: baseline.mean_rss_delta_bytes,
            candidate_mean_rss_delta_bytes: candidate.mean_rss_delta_bytes,
            rss_change_pct: match (
                baseline.mean_rss_delta_bytes,
                candidate.mean_rss_delta_bytes,
            ) {
                (Some(base), Some(next)) => percent_change(base, next),
                _ => None,
            },
        });
    }

    rows
}

fn has_regressions(
    rows: &[CompareRow],
    duration_threshold_pct: Option<f64>,
    memory_threshold_pct: Option<f64>,
) -> bool {
    rows.iter().any(|row| {
        let duration_failed = duration_threshold_pct
            .is_some_and(|threshold| row.duration_change_pct.is_some_and(|pct| pct > threshold));
        let memory_failed = memory_threshold_pct
            .is_some_and(|threshold| row.rss_change_pct.is_some_and(|pct| pct > threshold));
        duration_failed || memory_failed
    })
}

fn percent_change(baseline: f64, candidate: f64) -> Option<f64> {
    if baseline.abs() <= f64::EPSILON {
        return None;
    }
    Some(((candidate - baseline) / baseline.abs()) * 100.0)
}

fn print_summary_table(rows: &[SummaryRow]) {
    println!(
        "{:<12} {:<16} {:<12} {:>7} {:>12} {:>12} {:>14} {:>14}",
        "scenario", "case", "stage", "samples", "mean_ms", "max_ms", "mean_rss_mb", "max_rss_mb"
    );
    for row in rows {
        println!(
            "{:<12} {:<16} {:<12} {:>7} {:>12.3} {:>12.3} {:>14} {:>14}",
            row.scenario,
            row.case_name,
            row.stage,
            row.samples,
            row.mean_duration_ms,
            row.max_duration_ms,
            format_option_mb_f64(row.mean_rss_delta_bytes),
            format_option_mb_u64(row.max_rss_after_bytes),
        );
    }
}

fn print_compare_table(rows: &[CompareRow]) {
    println!(
        "{:<12} {:<16} {:<12} {:>12} {:>12} {:>10} {:>12} {:>12} {:>10}",
        "scenario",
        "case",
        "stage",
        "base_ms",
        "cand_ms",
        "dur_%",
        "base_rss_mb",
        "cand_rss_mb",
        "rss_%"
    );
    for row in rows {
        println!(
            "{:<12} {:<16} {:<12} {:>12.3} {:>12.3} {:>10} {:>12} {:>12} {:>10}",
            row.scenario,
            row.case_name,
            row.stage,
            row.baseline_mean_duration_ms,
            row.candidate_mean_duration_ms,
            format_option_pct(row.duration_change_pct),
            format_option_mb_f64(row.baseline_mean_rss_delta_bytes),
            format_option_mb_f64(row.candidate_mean_rss_delta_bytes),
            format_option_pct(row.rss_change_pct),
        );
    }
}

fn format_option_mb_f64(value: Option<f64>) -> String {
    value.map_or_else(
        || "-".to_string(),
        |bytes| format!("{:.3}", bytes / (1024.0 * 1024.0)),
    )
}

fn format_option_mb_u64(value: Option<u64>) -> String {
    value.map_or_else(
        || "-".to_string(),
        |bytes| format!("{:.3}", bytes as f64 / (1024.0 * 1024.0)),
    )
}

fn format_option_pct(value: Option<f64>) -> String {
    value.map_or_else(|| "-".to_string(), |pct| format!("{:.2}", pct))
}

fn write_records_jsonl(
    path: &Path,
    records: &[BenchRecord],
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    for record in records {
        serde_json::to_writer(&mut writer, record)?;
        writer.write_all(b"\n")?;
    }
    writer.flush()?;
    Ok(())
}

fn load_records_jsonl(path: &Path) -> Result<Vec<BenchRecord>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        records.push(serde_json::from_str::<BenchRecord>(&line)?);
    }
    Ok(records)
}

fn write_csc_matrix(
    dir: &Path,
    matrix: &CscMatrix,
    variables: usize,
    constraints: usize,
) -> std::io::Result<()> {
    create_dir_all(dir)?;

    fn write_u64(path: &Path, data: &[u64]) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        for value in data {
            writer.write_all(&value.to_le_bytes())?;
        }
        Ok(())
    }

    fn write_u32(path: &Path, data: &[u32]) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        for value in data {
            writer.write_all(&value.to_le_bytes())?;
        }
        Ok(())
    }

    fn write_f64(path: &Path, data: &[f64]) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        for value in data {
            writer.write_all(&value.to_le_bytes())?;
        }
        Ok(())
    }

    write_u64(&dir.join("col_ptrs.bin"), &matrix.col_ptrs)?;
    write_u32(&dir.join("row_indices.bin"), &matrix.row_indices)?;
    write_f64(&dir.join("values.bin"), &matrix.values)?;

    let mut meta = BufWriter::new(File::create(dir.join("metadata.txt"))?);
    writeln!(meta, "columns (variables): {}", variables)?;
    writeln!(meta, "rows (constraints): {}", constraints)?;
    writeln!(meta, "nonzeros: {}", matrix.values.len())?;
    Ok(())
}

fn build_run_id() -> Result<String, Box<dyn std::error::Error>> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| std::io::Error::other(err.to_string()))?
        .as_millis();
    Ok(format!("bench_{}", millis))
}

fn boxed_input_error(message: &str) -> Box<dyn std::error::Error> {
    Box::new(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        message.to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::{
        BenchRecord, CompareRow, SummaryRow, build_comparison_rows, has_regressions,
        summarize_records,
    };

    fn approx_eq(left: f64, right: f64) {
        assert!((left - right).abs() < 1e-9, "left={left}, right={right}");
    }

    #[test]
    fn summarize_records_groups_and_averages() {
        let records = vec![
            BenchRecord {
                schema_version: 1,
                run_id: "run".to_string(),
                scenario: "model-build".to_string(),
                case_name: "vars_100".to_string(),
                repetition: 1,
                variables: 100,
                constraints: 1,
                stage: "total".to_string(),
                duration_ms: 10.0,
                rss_before_bytes: Some(1_000),
                rss_after_bytes: Some(2_000),
                rss_delta_bytes: Some(1_000),
            },
            BenchRecord {
                schema_version: 1,
                run_id: "run".to_string(),
                scenario: "model-build".to_string(),
                case_name: "vars_100".to_string(),
                repetition: 2,
                variables: 100,
                constraints: 1,
                stage: "total".to_string(),
                duration_ms: 30.0,
                rss_before_bytes: Some(1_500),
                rss_after_bytes: Some(3_000),
                rss_delta_bytes: Some(1_500),
            },
        ];

        let summary = summarize_records(&records);
        assert_eq!(summary.len(), 1);
        let row = &summary[0];
        assert_eq!(row.samples, 2);
        approx_eq(row.mean_duration_ms, 20.0);
        approx_eq(row.max_duration_ms, 30.0);
        match row.mean_rss_delta_bytes {
            Some(mean) => approx_eq(mean, 1_250.0),
            None => panic!("mean RSS delta should be present"),
        }
        assert_eq!(row.max_rss_after_bytes, Some(3_000));
    }

    #[test]
    fn compare_detects_regressions() {
        let baseline = vec![SummaryRow {
            scenario: "model-build".to_string(),
            case_name: "vars_100".to_string(),
            stage: "total".to_string(),
            samples: 2,
            mean_duration_ms: 100.0,
            max_duration_ms: 110.0,
            mean_rss_delta_bytes: Some(1_000.0),
            max_rss_after_bytes: Some(20_000),
        }];
        let candidate = vec![SummaryRow {
            scenario: "model-build".to_string(),
            case_name: "vars_100".to_string(),
            stage: "total".to_string(),
            samples: 2,
            mean_duration_ms: 120.0,
            max_duration_ms: 130.0,
            mean_rss_delta_bytes: Some(1_300.0),
            max_rss_after_bytes: Some(21_000),
        }];

        let rows = build_comparison_rows(&baseline, &candidate, "total");
        assert_eq!(rows.len(), 1);
        let row: &CompareRow = &rows[0];
        match row.duration_change_pct {
            Some(duration_change) => approx_eq(duration_change, 20.0),
            None => panic!("duration change should be present"),
        }

        assert!(has_regressions(&rows, Some(10.0), Some(20.0)));
        assert!(!has_regressions(&rows, Some(25.0), Some(35.0)));
    }
}
