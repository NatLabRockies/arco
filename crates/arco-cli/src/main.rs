use arco_cli::cli_io::{should_log_solver_to_console, write_stdout, write_stdout_line};
use arco_cli::config::{SolverBackend, SolverConfig, load_solver_config, save_solver_config};
use arco_cli::debug::launch_ipython;
use arco_cli::driver::{
    InspectCategory, RunOptions, print_file_model, run_file_json_with_options_and_backend,
    validate_file_report,
};
use arco_cli::export::{write_lp, write_mps};
use arco_kdl::pipeline::compile_file;
use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use miette::IntoDiagnostic;
use std::fs;
use std::io::IsTerminal;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "arco",
    about = "Algebraic optimization DSL compiler and solver"
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
    Validate {
        path: PathBuf,
        /// Inspect a specific semantic category
        #[arg(long)]
        inspect: Option<InspectCategory>,
        /// Filter to a specific element by name within the inspected category
        #[arg(long)]
        name: Option<String>,
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

#[derive(Subcommand)]
enum SolverAction {
    /// Show the active solver backend
    Show,
    /// Set the solver backend
    Set { backend: SolverBackendArg },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum SolverBackendArg {
    Highs,
    Xpress,
}

impl From<SolverBackendArg> for SolverBackend {
    fn from(value: SolverBackendArg) -> Self {
        match value {
            SolverBackendArg::Highs => SolverBackend::Highs,
            SolverBackendArg::Xpress => SolverBackend::Xpress,
        }
    }
}

fn main() -> miette::Result<()> {
    let cli = Cli::parse();
    let _ = miette::set_hook(Box::new(|_| {
        Box::new(miette::MietteHandlerOpts::new().build())
    }));
    init_tracing(cli.verbose);

    match cli.command {
        Command::Run {
            path,
            filter_variable,
            filter_asset,
            compact,
        } => {
            let solver_config = load_solver_config()?;
            let output = run_file_json_with_options_and_backend(
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
                solver_config.backend,
            )?;
            write_stdout(output.as_bytes()).into_diagnostic()?;
        }
        Command::PrintModel { path } => {
            write_stdout_line(&print_file_model(&path)?).into_diagnostic()?;
        }
        Command::Validate {
            path,
            inspect,
            name,
        } => {
            let name_ref = name.as_deref();
            write_stdout_line(&validate_file_report(&path, inspect, name_ref)?)
                .into_diagnostic()?;
        }
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

fn export_model(
    path: PathBuf,
    format: ExportFormat,
    output: Option<PathBuf>,
) -> miette::Result<()> {
    let compiled = compile_file(&path)?;
    let mut buffer = Vec::new();
    match format {
        ExportFormat::Lp => write_lp(&compiled.lowered_problem.algebra, &mut buffer)?,
        ExportFormat::Mps => write_mps(&compiled.lowered_problem.algebra, &mut buffer)?,
    }

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
            write_stdout_line(&format!("backend: {}", config.backend.as_str()))
                .into_diagnostic()?;
        }
        SolverAction::Set { backend } => {
            let config = SolverConfig {
                backend: backend.into(),
            };
            let path = save_solver_config(&config)?;
            write_stdout_line(&format!("backend: {}", config.backend.as_str()))
                .into_diagnostic()?;
            write_stdout_line(&format!("path: {}", path.display())).into_diagnostic()?;
        }
    }

    Ok(())
}

fn init_tracing(verbose: u8) {
    if verbose == 0 {
        return;
    }

    let level = match verbose {
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
