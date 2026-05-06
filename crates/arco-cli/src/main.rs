use arco_cli::cli_io::{
    ColorMode, should_colorize_stdout, should_log_solver_to_console, write_stdout,
    write_stdout_line,
};
use arco_cli::config::{load_solver_config, save_solver_selection};
use arco_cli::debug_shell::launch_ipython;
use arco_cli::driver::{
    RunOptions, inspect_file_report, kdl_check_file_json, print_file_model,
    run_file_json_with_options_and_selection, validate_file_only,
};
use arco_ops::{ArcoOps, OpsExportFormat};
use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use miette::IntoDiagnostic;
use std::fs;
use std::io::IsTerminal;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "arco",
    about = "Algebraic optimization DSL compiler and solver",
    version = env!("CARGO_PKG_VERSION")
)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Increase verbosity (-v, -vv)
    #[arg(short, long, action = ArgAction::Count, global = true)]
    verbose: u8,
}

#[derive(Subcommand)]
enum Command {
    /// Compile and solve a .kdl formulation
    Run {
        path: PathBuf,
        /// Filter output variables by glob pattern
        #[arg(long)]
        filter_variable: Option<String>,
        /// Filter output by asset name glob
        #[arg(long)]
        filter_asset: Option<String>,
        /// Omit full value arrays from the JSON summary
        #[arg(long)]
        compact: bool,
    },
    /// Print the algebraic model sent to the solver
    PrintModel { path: PathBuf },
    /// Validate a .kdl file without solving
    Validate { path: PathBuf },
    /// KDL tooling helpers
    Kdl {
        #[command(subcommand)]
        action: KdlAction,
    },
    /// Inspect semantic model information from a validated .kdl file
    Inspect {
        path: PathBuf,
        /// Emit structured JSON output instead of TOML
        #[arg(long)]
        json: bool,
    },
    /// Open an interactive debug shell in IPython
    Debug { path: PathBuf },
    /// Export the algebraic model in LP or MPS format
    Export {
        path: PathBuf,
        #[arg(long, default_value = "lp")]
        format: ExportFormat,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Configure solver specification
    Solver {
        #[command(subcommand)]
        action: SolverAction,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ExportFormat {
    Lp,
    Mps,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CheckFormat {
    Text,
    Json,
}

#[derive(Subcommand)]
enum KdlAction {
    /// Validate a .kdl file with canonical parser diagnostics
    Check {
        path: PathBuf,
        #[arg(long, default_value = "text")]
        format: CheckFormat,
    },
}

#[derive(Subcommand)]
enum SolverAction {
    /// Show the active solver selection and availability
    Show,
    /// Set the default solver selection token (family or profile)
    Set { selection: String },
}

fn main() -> miette::Result<()> {
    let cli = Cli::parse();
    let _ = miette::set_hook(Box::new(|_| {
        Box::new(miette::MietteHandlerOpts::new().build())
    }));
    let validate_command = matches!(
        &cli.command,
        Command::Validate { .. }
            | Command::Kdl {
                action: KdlAction::Check {
                    format: CheckFormat::Text,
                    ..
                },
            }
    );
    init_tracing(cli.verbose, validate_command);

    match cli.command {
        Command::Run {
            path,
            filter_variable,
            filter_asset,
            compact,
        } => {
            let solver_config = load_solver_config()?;
            let output = run_file_json_with_options_and_selection(
                &path,
                &RunOptions {
                    compact,
                    filter_variable,
                    filter_asset,
                    solver_log: should_log_solver_to_console(
                        cli.verbose,
                        std::io::stdout().is_terminal(),
                    ),
                },
                &solver_config.resolved,
            )?;
            write_stdout(output.as_bytes()).into_diagnostic()?;
        }
        Command::PrintModel { path } => {
            write_stdout_line(&print_file_model(&path)?).into_diagnostic()?;
        }
        Command::Validate { path } => {
            write_stdout_line(&validate_file_only(
                &path,
                ColorMode::from(should_colorize_stdout(std::io::stdout().is_terminal())),
            )?)
            .into_diagnostic()?;
        }
        Command::Inspect { path, json } => {
            write_stdout_line(&inspect_file_report(&path, json)?).into_diagnostic()?;
        }
        Command::Kdl { action } => handle_kdl_action(action)?,
        Command::Debug { path } => {
            launch_ipython(&path)?;
        }
        Command::Export {
            path,
            format,
            output,
        } => export_model(path, format, output)?,
        Command::Solver { action } => handle_solver_action(action)?,
    }

    Ok(())
}

fn handle_kdl_action(action: KdlAction) -> miette::Result<()> {
    match action {
        KdlAction::Check { path, format } => match format {
            CheckFormat::Text => {
                write_stdout_line(&validate_file_only(
                    &path,
                    ColorMode::from(should_colorize_stdout(std::io::stdout().is_terminal())),
                )?)
                .into_diagnostic()?;
            }
            CheckFormat::Json => {
                let outcome = kdl_check_file_json(&path)?;
                write_stdout_line(&outcome.json).into_diagnostic()?;
                if !outcome.valid {
                    std::process::exit(1);
                }
            }
        },
    }

    Ok(())
}

fn export_model(
    path: PathBuf,
    format: ExportFormat,
    output: Option<PathBuf>,
) -> miette::Result<()> {
    let compiled = ArcoOps::compile_file(&path)?;
    let buffer = ArcoOps::export_problem(
        &compiled.compiled_problem.algebra,
        match format {
            ExportFormat::Lp => OpsExportFormat::Lp,
            ExportFormat::Mps => OpsExportFormat::Mps,
        },
    )?;

    if let Some(output_path) = output {
        fs::write(output_path, buffer).into_diagnostic()?;
    } else {
        write_stdout(&buffer).into_diagnostic()?;
    }

    Ok(())
}

fn handle_solver_action(action: SolverAction) -> miette::Result<()> {
    match action {
        SolverAction::Show => {
            let config = load_solver_config()?;
            for line in config.live_status_lines() {
                write_stdout_line(&line).into_diagnostic()?;
            }
        }
        SolverAction::Set { selection } => {
            let path = save_solver_selection(&selection)?;
            write_stdout_line(&format!("selection: {}", selection)).into_diagnostic()?;
            write_stdout_line(&format!("path: {}", path.display())).into_diagnostic()?;
        }
    }

    Ok(())
}

fn init_tracing(verbose: u8, force_warnings: bool) {
    if verbose == 0 && !force_warnings {
        return;
    }
    let level = match verbose {
        0 => tracing::Level::WARN,
        1 => tracing::Level::INFO,
        _ => tracing::Level::DEBUG,
    };

    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_writer(std::io::stderr)
        .with_target(false)
        .without_time()
        .init();
}
