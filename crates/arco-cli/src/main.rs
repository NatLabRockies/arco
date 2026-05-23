use arco_cli::cli_io::{
    ColorMode, should_colorize_stdout, should_log_solver_to_console, write_stderr_line,
    write_stdout, write_stdout_line,
};
use arco_cli::config::{load_solver_config, save_solver_selection};
use arco_cli::debug_shell::launch_ipython;
use arco_cli::driver::{
    KdlCheckMode, RunOptions, inspect_file_report, kdl_check_file_json, print_file_model,
    render_plain_driver_error, run_file_json_with_options_and_config, validate_file_only,
};
use arco_cli::self_update;
use arco_ops::{ArcoOps, OpsExportFormat};
use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use miette::IntoDiagnostic;
use similar::TextDiff;
use std::fs;
use std::io::{IsTerminal, Read};
use std::path::{Path, PathBuf};

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
        /// Force printing solver iterations and diagnostics to stderr
        #[arg(long)]
        solver_log: bool,
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
    /// Manage the arco executable
    #[command(name = "self")]
    This {
        #[command(subcommand)]
        action: SelfAction,
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
        /// Compile far enough to materialize CSV-backed data contracts
        #[arg(long)]
        materialize_data: bool,
    },
    /// Format KDL files (similar to `ruff format`)
    Fmt(KdlFmtArgs),
}

#[derive(Args, Debug)]
struct KdlFmtArgs {
    /// Files or directories to format (defaults to current directory)
    paths: Vec<PathBuf>,
    /// Check if files are formatted; do not write changes
    #[arg(long)]
    check: bool,
    /// Show a unified diff for required changes
    #[arg(long)]
    diff: bool,
    /// Read KDL from stdin
    #[arg(long)]
    stdin: bool,
    /// Optional display name for stdin source (used in diff headers)
    #[arg(long)]
    stdin_filename: Option<String>,
}

#[derive(Subcommand)]
enum SolverAction {
    /// Show the active solver selection and availability
    Show,
    /// Set the default solver selection token (family or profile)
    Set { selection: String },
}

#[derive(Subcommand)]
enum SelfAction {
    /// Update this arco executable when installed by the standalone installer
    Update {
        /// Update to a specific release tag instead of the latest release
        #[arg(long)]
        version: Option<String>,
        /// GitHub token for higher API rate limits
        #[arg(long)]
        token: Option<String>,
    },
}

fn main() -> miette::Result<()> {
    let cli = Cli::parse();
    let _ = miette::set_hook(Box::new(|_| {
        Box::new(miette::MietteHandlerOpts::new().build())
    }));
    let stdout_is_terminal = std::io::stdout().is_terminal();
    let color_mode = ColorMode::from(should_colorize_stdout(stdout_is_terminal));
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
            solver_log,
            compact,
        } => {
            let solver_config = load_solver_config()?;
            let output = run_file_json_with_options_and_config(
                &path,
                &RunOptions {
                    compact,
                    filter_variable,
                    filter_asset,
                    solver_log: solver_log
                        || should_log_solver_to_console(cli.verbose, stdout_is_terminal),
                },
                &solver_config,
            );
            match output {
                Ok(output) => write_stdout(output.as_bytes()).into_diagnostic()?,
                Err(error) => {
                    if let Some(rendered) = render_plain_driver_error(
                        &error,
                        ColorMode::from(should_colorize_stdout(std::io::stderr().is_terminal())),
                    ) {
                        write_stderr_line(&rendered).into_diagnostic()?;
                        std::process::exit(1);
                    }
                    return Err(error.into());
                }
            }
        }
        Command::PrintModel { path } => {
            write_stdout_line(&print_file_model(&path)?).into_diagnostic()?;
        }
        Command::Validate { path } => {
            write_stdout_line(&validate_file_only(&path, color_mode)?).into_diagnostic()?;
        }
        Command::Inspect { path, json } => {
            write_stdout_line(&inspect_file_report(&path, json)?).into_diagnostic()?;
        }
        Command::Kdl { action } => handle_kdl_action_with_color(action, color_mode)?,
        Command::Debug { path } => {
            launch_ipython(&path)?;
        }
        Command::Export {
            path,
            format,
            output,
        } => export_model(path, format, output)?,
        Command::Solver { action } => handle_solver_action(action)?,
        Command::This { action } => handle_self_action(action, cli.verbose)?,
    }

    Ok(())
}

fn handle_kdl_action_with_color(action: KdlAction, color_mode: ColorMode) -> miette::Result<()> {
    match action {
        KdlAction::Check {
            path,
            format,
            materialize_data,
        } => match format {
            CheckFormat::Text => {
                let mode = kdl_check_mode(materialize_data);
                write_stdout_line(&arco_cli::driver::validate_file(&path, color_mode, mode)?)
                    .into_diagnostic()?;
            }
            CheckFormat::Json => {
                let outcome = kdl_check_file_json(&path, kdl_check_mode(materialize_data))?;
                write_stdout_line(&outcome.json).into_diagnostic()?;
                if !outcome.valid {
                    std::process::exit(1);
                }
            }
        },
        KdlAction::Fmt(args) => run_kdl_fmt(args)?,
    }

    Ok(())
}

fn kdl_check_mode(materialize_data: bool) -> KdlCheckMode {
    if materialize_data {
        KdlCheckMode::Materialized
    } else {
        KdlCheckMode::Structural
    }
}

fn export_model(
    path: PathBuf,
    format: ExportFormat,
    output: Option<PathBuf>,
) -> miette::Result<()> {
    let buffer = ArcoOps::export_model_file(
        &path,
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

fn handle_self_action(action: SelfAction, verbose: u8) -> miette::Result<()> {
    match action {
        SelfAction::Update { version, token } => {
            let code = self_update::update(version, token, verbose)?;
            if code != 0 {
                std::process::exit(code);
            }
        }
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

fn format_kdl_text(input: &str, path: Option<&Path>) -> miette::Result<String> {
    ArcoOps::format_kdl_text(input).map_err(|error| {
        if let Some(path) = path {
            miette::miette!(
                "Failed to parse KDL document: {} ({})",
                path.display(),
                error
            )
        } else {
            miette::miette!("Failed to parse KDL document")
        }
    })
}

fn collect_kdl_files(path: &Path, files: &mut Vec<PathBuf>) -> miette::Result<()> {
    let metadata = fs::symlink_metadata(path).into_diagnostic()?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }

    if metadata.is_file() {
        if path.extension().is_some_and(|ext| ext == "kdl") {
            files.push(path.to_path_buf());
        }
        return Ok(());
    }

    if metadata.is_dir() {
        if let Some(name) = path.file_name().and_then(|value| value.to_str()) {
            if matches!(name, ".git" | "target" | "node_modules" | ".uv-cache") {
                return Ok(());
            }
        }

        for entry in fs::read_dir(path).into_diagnostic()? {
            let entry = entry.into_diagnostic()?;
            collect_kdl_files(&entry.path(), files)?;
        }
    }

    Ok(())
}

fn print_unified_diff(label: &str, before: &str, after: &str) -> miette::Result<()> {
    let diff = TextDiff::from_lines(before, after);
    write_stdout_line(&format!("--- {label}")).into_diagnostic()?;
    write_stdout_line(&format!("+++ {label}")).into_diagnostic()?;
    for change in diff.unified_diff().header("", "").iter_hunks() {
        write_stdout(change.to_string().as_bytes()).into_diagnostic()?;
    }
    Ok(())
}

fn run_kdl_fmt(args: KdlFmtArgs) -> miette::Result<()> {
    if args.stdin || args.paths.iter().any(|path| path == Path::new("-")) {
        let mut input = String::new();
        std::io::stdin()
            .read_to_string(&mut input)
            .into_diagnostic()?;
        let formatted = format_kdl_text(&input, None)?;

        if args.check || args.diff {
            if input != formatted {
                if args.diff {
                    let label = args.stdin_filename.as_deref().unwrap_or("stdin.kdl");
                    print_unified_diff(label, &input, &formatted)?;
                }
                std::process::exit(1);
            }
            return Ok(());
        }

        write_stdout(formatted.as_bytes()).into_diagnostic()?;
        return Ok(());
    }

    let roots = if args.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        args.paths
    };

    let mut files = Vec::new();
    for root in roots {
        collect_kdl_files(&root, &mut files)?;
    }

    let mut changed_files = 0usize;

    for file in files {
        let input = fs::read_to_string(&file).into_diagnostic()?;
        let formatted = format_kdl_text(&input, Some(&file))?;
        if input == formatted {
            continue;
        }

        changed_files += 1;

        if args.diff {
            let label = file.to_string_lossy().to_string();
            print_unified_diff(&label, &input, &formatted)?;
        }

        if !args.check && !args.diff {
            fs::write(&file, formatted).into_diagnostic()?;
        }
    }

    if args.check || args.diff {
        if changed_files > 0 {
            write_stderr_line(&format!("{} file(s) would be reformatted", changed_files))
                .into_diagnostic()?;
            std::process::exit(1);
        }
        write_stderr_line("All files already formatted").into_diagnostic()?;
    } else {
        write_stderr_line(&format!("{} file(s) reformatted", changed_files)).into_diagnostic()?;
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
