use serde_json::Value;
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

fn nodal_input() -> PathBuf {
    fixture_path(&["tests", "e2e", "generator-allocation", "input.kdl"])
}

fn validated_entrypoint_or_input(path: &std::path::Path) -> PathBuf {
    match arco_kdl::pipeline::validate_file(path) {
        Ok(validated) => validated.entrypoint,
        Err(_) => path.to_path_buf(),
    }
}

fn arco_command() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_arco"));
    cmd.env("ARCO_CONFIG_DIR", env!("CARGO_TARGET_TMPDIR"));
    cmd
}

fn parse_stdout_json(output: std::process::Output) -> Result<Value, Box<dyn std::error::Error>> {
    let stdout = String::from_utf8(output.stdout)?;
    Ok(serde_json::from_str(&stdout)?)
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
    assert!(stderr.contains("inspect"));
    assert!(stderr.contains("debug"));
    assert!(stderr.contains("export"));
    assert!(stderr.contains("solver"));

    Ok(())
}

#[test]
fn cli_validates_a_fixture_file() -> Result<(), Box<dyn std::error::Error>> {
    let input = nodal_input();

    let output = arco_command().arg("validate").arg(&input).output()?;

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout)?;
    let line = stdout.trim();
    let display_path = validated_entrypoint_or_input(&input);
    let prefix = format!("Validated file://{} in ", display_path.display());
    let suffix = format!("ms (arco {})", env!("CARGO_PKG_VERSION"));
    assert!(line.starts_with(&prefix));
    assert!(line.ends_with(&suffix));
    let ms_fragment = &line[prefix.len()..line.len() - suffix.len()];
    assert!(ms_fragment.parse::<u128>().is_ok());

    Ok(())
}

#[test]
fn cli_validate_can_force_colored_summary_output() -> Result<(), Box<dyn std::error::Error>> {
    let input = nodal_input();

    let output = arco_command()
        .env("CLICOLOR_FORCE", "1")
        .arg("validate")
        .arg(&input)
        .output()?;

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout)?;
    let display_path = validated_entrypoint_or_input(&input);
    assert!(stdout.contains("\x1b[38;5;245mValidated "));
    assert!(stdout.contains(&format!("\x1b[1mfile://{}\x1b[22m", display_path.display())));
    assert!(stdout.contains(&format!("\x1b[1marco {}", env!("CARGO_PKG_VERSION"))));

    Ok(())
}

#[test]
fn cli_inspect_without_section_defaults_to_pretty_output() -> Result<(), Box<dyn std::error::Error>>
{
    let input = nodal_input();

    let output = arco_command().arg("inspect").arg(&input).output()?;

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("counts"));
    assert!(stdout.contains("[summary]"));
    assert!(stdout.contains("capacity_limit"));

    Ok(())
}

#[test]
fn cli_inspect_json_without_section_returns_full_model_view()
-> Result<(), Box<dyn std::error::Error>> {
    let input = nodal_input();

    let output = arco_command()
        .arg("inspect")
        .arg(&input)
        .arg("--json")
        .output()?;

    assert!(output.status.success());

    let payload = parse_stdout_json(output)?;
    let object = payload.as_object().expect("full inspect object");
    assert!(object.contains_key("summaries"));
    assert!(object.contains_key("sets"));
    assert!(object.contains_key("constraints"));
    assert!(object.contains_key("variables"));
    assert!(object.contains_key("objectives"));
    assert!(object.contains_key("parameters"));

    let sets = object
        .get("sets")
        .and_then(serde_json::Value::as_object)
        .expect("sets object");
    assert!(sets.contains_key("time"));

    let variables = object
        .get("variables")
        .and_then(serde_json::Value::as_array)
        .expect("variables array");
    assert!(
        variables
            .iter()
            .any(|variable| variable["name"] == "dispatch")
    );

    let parameters = object
        .get("parameters")
        .and_then(serde_json::Value::as_object)
        .expect("parameters object");
    assert!(parameters.is_empty());

    Ok(())
}

#[test]
fn cli_inspect_sets_list() -> Result<(), Box<dyn std::error::Error>> {
    let input = nodal_input();

    let output = arco_command()
        .arg("inspect")
        .arg(&input)
        .arg("--json")
        .arg("--section")
        .arg("sets")
        .output()?;

    assert!(output.status.success());

    let payload = parse_stdout_json(output)?;
    let items = payload.as_array().expect("set array");
    assert!(!items.is_empty());
    assert!(items.iter().any(|item| item["name"] == "time"));
    assert!(!items.iter().any(|item| item["name"] == "assets"));
    assert!(!items.iter().any(|item| item["name"] == "candidate_assets"));

    Ok(())
}

#[test]
fn cli_inspect_sets_detail() -> Result<(), Box<dyn std::error::Error>> {
    let input = nodal_input();

    let output = arco_command()
        .arg("inspect")
        .arg(&input)
        .arg("--section")
        .arg("sets")
        .arg("--name")
        .arg("assets")
        .output()?;

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("not found"));
    assert!(stderr.contains("Available sets:"));

    Ok(())
}

#[test]
fn cli_inspect_constraints_list() -> Result<(), Box<dyn std::error::Error>> {
    let input = nodal_input();

    let output = arco_command()
        .arg("inspect")
        .arg(&input)
        .arg("--json")
        .arg("--section")
        .arg("constraints")
        .output()?;

    assert!(output.status.success());

    let payload = parse_stdout_json(output)?;
    let items = payload.as_array().expect("constraint array");
    assert!(items.iter().any(|item| {
        item["name"] == "capacity_limit"
            && item["relation"] == "less_or_equal"
            && item["lhs_terms"]
                .as_array()
                .is_some_and(|terms| !terms.is_empty())
            && item["variable_refs"] == serde_json::json!([{"$ref": "#/variables/dispatch"}])
            && item["parameter_refs"] == serde_json::json!([])
    }));
    assert_eq!(items.len(), 1);

    Ok(())
}

#[test]
fn cli_inspect_variables_list() -> Result<(), Box<dyn std::error::Error>> {
    let input = nodal_input();

    let output = arco_command()
        .arg("inspect")
        .arg(&input)
        .arg("--json")
        .arg("--section")
        .arg("variables")
        .output()?;

    assert!(output.status.success());

    let payload = parse_stdout_json(output)?;
    let items = payload.as_array().expect("variable array");
    assert!(items.iter().any(|item| {
        item["name"] == "dispatch"
            && item["notation"] == "dispatch[a, t]"
            && item["set"]
                == serde_json::json!([
                    {"index": "a", "name": "a", "cardinality": 0},
                    {"index": "t", "name": "t", "cardinality": 0}
                ])
            && item["domain"]
                == serde_json::json!({
                    "kind": "continuous",
                    "lower": "asset-dependent",
                    "upper": null
                })
    }));

    Ok(())
}

#[test]
fn cli_inspect_variables_list_defaults_to_pretty_cards() -> Result<(), Box<dyn std::error::Error>> {
    let input = nodal_input();

    let output = arco_command()
        .arg("inspect")
        .arg(&input)
        .arg("--section")
        .arg("variables")
        .output()?;

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("[variable]"));
    assert!(stdout.contains("notation : dispatch[a, t]"));
    assert!(stdout.contains("domains  : a ∈ a, t ∈ t"));
    assert!(!stdout.contains("\"$ref\""));

    Ok(())
}

#[test]
fn cli_inspect_parameters_list() -> Result<(), Box<dyn std::error::Error>> {
    let input = nodal_input();

    let output = arco_command()
        .arg("inspect")
        .arg(&input)
        .arg("--json")
        .arg("--section")
        .arg("parameters")
        .output()?;

    assert!(output.status.success());

    let payload = parse_stdout_json(output)?;
    let items = payload.as_array().expect("parameter array");
    assert!(items.is_empty());

    Ok(())
}

#[test]
fn cli_inspect_rejects_name_without_section() -> Result<(), Box<dyn std::error::Error>> {
    let input = nodal_input();

    let output = arco_command()
        .arg("inspect")
        .arg(&input)
        .arg("--name")
        .arg("area")
        .output()?;

    assert_eq!(output.status.code(), Some(2));

    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("--section"));

    Ok(())
}

#[test]
fn cli_inspect_sets_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let input = nodal_input();

    let output = arco_command()
        .arg("inspect")
        .arg(&input)
        .arg("--section")
        .arg("sets")
        .arg("--name")
        .arg("nonexistent")
        .output()?;

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("not found"));
    assert!(stderr.contains("Available sets:"));
    assert!(!stderr.contains("assets"));
    assert!(stderr.contains("time"));

    Ok(())
}

#[test]
fn cli_inspect_objective() -> Result<(), Box<dyn std::error::Error>> {
    let input = nodal_input();

    let output = arco_command()
        .arg("inspect")
        .arg(&input)
        .arg("--json")
        .arg("--section")
        .arg("objective")
        .output()?;

    assert!(output.status.success());

    let payload = parse_stdout_json(output)?;
    let items = payload.as_array().expect("objective array");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["name"], "TotalCost");
    assert_eq!(items[0]["sense"], "minimize");
    assert_eq!(items[0]["aggregation"], "sum");
    assert!(
        items[0]["scope"]
            .as_array()
            .is_some_and(|scope| !scope.is_empty())
    );
    assert_eq!(
        items[0]["variable_refs"],
        serde_json::json!([{"$ref": "#/variables/dispatch"}])
    );
    assert_eq!(items[0]["parameter_refs"], serde_json::json!([]));
    assert!(items[0].get("expression").is_none());
    assert!(
        items[0]["terms"]
            .as_array()
            .is_some_and(|terms| !terms.is_empty())
    );

    Ok(())
}

#[test]
fn cli_inspect_objective_pretty_aligns_field_colons() -> Result<(), Box<dyn std::error::Error>> {
    let input = nodal_input();

    let output = arco_command()
        .arg("inspect")
        .arg(&input)
        .arg("--section")
        .arg("objective")
        .output()?;

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("[objective]"));

    let positions = stdout
        .lines()
        .filter(|line| line.starts_with("  ") && line.contains(':'))
        .map(|line| line.find(':').expect("line has colon"))
        .collect::<Vec<_>>();
    assert!(!positions.is_empty());

    let first = positions[0];
    assert!(
        positions.iter().all(|position| *position == first),
        "field colons should align in pretty output: {positions:?}"
    );

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
