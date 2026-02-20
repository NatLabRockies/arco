mod comparison;
mod reporting;
mod scenarios;
mod types;

use clap::{Parser, Subcommand};
use comparison::{build_comparison_rows, has_regressions, summarize_records};
use reporting::{
    load_records_jsonl, render_compare_output, render_output, write_csc_matrix, write_records_jsonl,
};
use scenarios::{build_run_id, case_records, execute_case, resolve_cases};
use std::path::PathBuf;
use types::{OutputFormat, Scenario};

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
pub(crate) struct RunArgs {
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
    let RunArgs {
        scenarios,
        cases,
        variables,
        constraints,
        constraint_ratio,
        repetitions,
        output,
        format,
        write_csc,
    } = args;

    if repetitions == 0 {
        return Err(boxed_input_error("repetitions must be greater than zero"));
    }
    if constraint_ratio <= 0.0 {
        return Err(boxed_input_error(
            "constraint-ratio must be greater than zero",
        ));
    }

    let run_id = build_run_id()?;
    let output_path = output
        .unwrap_or_else(|| PathBuf::from(format!("artifacts/bench/{}.jsonl", run_id.as_str())));

    let mut records = Vec::new();

    for scenario in &scenarios {
        let cases = resolve_cases(*scenario, variables, constraints, cases.as_deref());
        for case in cases {
            for rep_idx in 0..repetitions {
                let execution = execute_case(
                    case.variables,
                    case.constraints,
                    constraint_ratio,
                    write_csc.is_some(),
                );
                if let (Some(base_dir), Some(csc)) = (write_csc.as_ref(), execution.csc.as_ref()) {
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
    render_output(format, &records)?;
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

fn boxed_input_error(message: &str) -> Box<dyn std::error::Error> {
    Box::new(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        message.to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use crate::comparison::{build_comparison_rows, has_regressions, summarize_records};
    use crate::scenarios::execute_case;
    use crate::types::{BenchRecord, CompareRow, SummaryRow};

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

        let execution = execute_case(32, Some(8), 0.25, false);
        let stages: Vec<&str> = execution
            .stage_measurements
            .iter()
            .map(|measurement| measurement.stage.as_str())
            .collect();
        assert!(stages.contains(&"export_csc"));
        assert!(stages.contains(&"export_crs"));
        assert!(stages.contains(&"export_coo"));
    }
}
