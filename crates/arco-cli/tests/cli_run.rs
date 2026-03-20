use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn fixture_path(parts: &[&str]) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for part in parts {
        path.push(part);
    }
    path
}

fn price_taker_battery_input() -> PathBuf {
    fixture_path(&["tests", "e2e", "price-taker-battery", "input.kdl"])
}

fn simple_market_storage_input() -> PathBuf {
    fixture_path(&[
        "tests",
        "e2e",
        "simple-electricity-market-storage",
        "input.kdl",
    ])
}

fn arco_command() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_arco"));
    cmd.env("ARCO_CONFIG_DIR", env!("CARGO_TARGET_TMPDIR"));
    cmd
}

#[test]
fn cli_runs_a_fixture_file() -> Result<(), Box<dyn std::error::Error>> {
    let input = price_taker_battery_input();

    let output = arco_command().arg("run").arg(&input).output()?;

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout)?;
    assert!(!stdout.contains('\n'));
    assert!(stdout.contains("\"active_scenario\":\"BatteryArbitrageDay\""));
    assert!(stdout.contains("\"backend\":\"arco-rust-highs\""));
    assert!(stdout.contains("\"solve_status\":\"optimal\""));
    assert!(stdout.contains("\"name\":\"ArbitrageProfit\""));
    assert!(stdout.contains("\"timing\""));
    assert!(stdout.contains("\"peak_memory_bytes\""));

    Ok(())
}

#[test]
fn cli_run_handles_broken_pipe_without_panicking() -> Result<(), Box<dyn std::error::Error>> {
    let input = price_taker_battery_input();

    let command = format!(
        "'{}' run '{}' | head -c 1 >/dev/null",
        env!("CARGO_BIN_EXE_arco"),
        input.display()
    );
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .env("ARCO_CONFIG_DIR", env!("CARGO_TARGET_TMPDIR"))
        .output()?;

    assert!(output.status.success());

    let stderr = String::from_utf8(output.stderr)?;
    assert!(!stderr.contains("panicked at"));
    assert!(!stderr.contains("Broken pipe"));

    Ok(())
}

#[test]
fn cli_run_keeps_json_pipeable_at_double_verbose() -> Result<(), Box<dyn std::error::Error>> {
    let input = price_taker_battery_input();

    let command = format!(
        "'{}' run '{}' -vv | jq -r '.active_scenario'",
        env!("CARGO_BIN_EXE_arco"),
        input.display()
    );
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .env("ARCO_CONFIG_DIR", env!("CARGO_TARGET_TMPDIR"))
        .output()?;

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout)?, "BatteryArbitrageDay\n");

    Ok(())
}

#[test]
fn cli_emits_debug_tracing_at_double_verbose() -> Result<(), Box<dyn std::error::Error>> {
    let input = price_taker_battery_input();

    let output = arco_command().arg("-vv").arg("run").arg(&input).output()?;

    assert!(output.status.success());

    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("lowering program"));
    assert!(stderr.contains("HiGHS solve completed"));

    Ok(())
}

#[test]
fn cli_prints_the_model_sent_to_highs() -> Result<(), Box<dyn std::error::Error>> {
    let input = price_taker_battery_input();

    let output = arco_command().arg("print-model").arg(&input).output()?;

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("Max ArbitrageProfit:"));
    assert!(stdout.contains("s.t."));
    assert!(stdout.contains("soc_balance[Battery1,1]:"));
    assert!(stdout.contains("charge[Battery1,1]"));

    Ok(())
}

#[test]
fn cli_rejects_missing_arguments() -> Result<(), Box<dyn std::error::Error>> {
    let output = arco_command().output()?;

    assert_eq!(output.status.code(), Some(2));

    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("Usage: arco"));
    assert!(stderr.contains("Commands:"));
    assert!(stderr.contains("validate"));
    assert!(stderr.contains("debug"));
    assert!(stderr.contains("export"));
    assert!(stderr.contains("solver"));

    Ok(())
}

#[test]
fn cli_validates_a_fixture_file() -> Result<(), Box<dyn std::error::Error>> {
    let input = price_taker_battery_input();

    let output = arco_command().arg("validate").arg(&input).output()?;

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("Validation succeeded"));
    assert!(stdout.contains("scenario: BatteryArbitrageDay"));

    Ok(())
}

#[test]
fn cli_exports_a_fixture_to_lp() -> Result<(), Box<dyn std::error::Error>> {
    let input = price_taker_battery_input();

    let output = arco_command()
        .arg("export")
        .arg(&input)
        .arg("--format")
        .arg("lp")
        .output()?;

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("Maximize"));
    assert!(stdout.contains("Subject To"));
    assert!(stdout.contains("Bounds"));
    assert!(stdout.contains("End"));

    Ok(())
}

#[test]
fn cli_filters_run_output_by_variable_and_omits_full_values_in_compact_mode()
-> Result<(), Box<dyn std::error::Error>> {
    let input = price_taker_battery_input();

    let output = arco_command()
        .arg("run")
        .arg(&input)
        .arg("--filter-variable")
        .arg("charge*")
        .arg("--compact")
        .output()?;

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("\"name\":\"charge[a,t]\""));
    assert!(!stdout.contains("\"name\":\"discharge[a,t]\""));
    assert!(!stdout.contains("\"values\""));

    Ok(())
}

#[test]
fn cli_filters_run_output_by_asset() -> Result<(), Box<dyn std::error::Error>> {
    let input = simple_market_storage_input();

    let output = arco_command()
        .arg("run")
        .arg(&input)
        .arg("--filter-asset")
        .arg("Pumped*")
        .output()?;

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("[PumpedHydro,1]"));
    assert!(!stdout.contains("[Coal,1]"));

    Ok(())
}

#[test]
fn cli_debug_reports_missing_uvx() -> Result<(), Box<dyn std::error::Error>> {
    let input = price_taker_battery_input();

    let output = arco_command()
        .env("PATH", "")
        .arg("debug")
        .arg(&input)
        .output()?;

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("failed to start uvx"));
    assert!(stderr.contains("Install uv first"));

    Ok(())
}

#[cfg(unix)]
#[test]
fn cli_debug_starts_ipython_without_banner_and_preloads_model()
-> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    let root = std::env::temp_dir().join(format!(
        "arco-cli-debug-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir_all(&root)?;

    let fake_bin = root.join("uvx");
    let args_file = root.join("uvx-args.txt");
    let script_file = root.join("bootstrap.py");
    fs::write(
        &fake_bin,
        "#!/bin/sh\nprintf '%s\n' \"$@\" > \"$ARCO_UVX_ARGS_FILE\"\nlast_arg=''\nfor arg in \"$@\"; do\n  last_arg=\"$arg\"\ndone\n/bin/cp \"$last_arg\" \"$ARCO_UVX_SCRIPT_FILE\"\n",
    )?;
    let mut permissions = fs::metadata(&fake_bin)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&fake_bin, permissions)?;

    let input = price_taker_battery_input();

    let output = arco_command()
        .env("PATH", &root)
        .env("ARCO_UVX_ARGS_FILE", &args_file)
        .env("ARCO_UVX_SCRIPT_FILE", &script_file)
        .arg("debug")
        .arg(&input)
        .output()?;

    assert!(output.status.success());

    let args = fs::read_to_string(&args_file)?;
    assert!(args.contains("--with"));
    assert!(args.contains("ipython"));
    assert!(args.contains("--no-banner"));
    assert!(args.contains("arco"));

    let script = fs::read_to_string(&script_file)?;
    assert!(script.contains("import arco"));
    assert!(script.contains("model = arco.Model.from_csc("));
    assert!(script.contains("model.set_variable_name(index, name=name)"));
    assert!(script.contains("model.set_constraint_name(index, name=name)"));
    assert!(script.contains("model.set_objective("));
    assert!(script.contains(&format!("model_path = Path(r\"{}\")", input.display())));

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn cli_persists_solver_configuration() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::temp_dir().join(format!(
        "arco-cli-solver-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    std::fs::create_dir_all(&root)?;

    let set_output = arco_command()
        .env("ARCO_CONFIG_DIR", &root)
        .arg("solver")
        .arg("set")
        .arg("highs")
        .output()?;

    assert!(set_output.status.success());
    let set_stdout = String::from_utf8(set_output.stdout)?;
    assert!(set_stdout.contains("backend: highs"));

    let show_output = arco_command()
        .env("ARCO_CONFIG_DIR", &root)
        .arg("solver")
        .arg("show")
        .output()?;

    assert!(show_output.status.success());
    let show_stdout = String::from_utf8(show_output.stdout)?;
    assert!(show_stdout.contains("backend: highs"));

    std::fs::remove_dir_all(&root)?;
    Ok(())
}
