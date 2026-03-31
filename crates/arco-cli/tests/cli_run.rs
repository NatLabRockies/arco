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
    fixture_path(&["..", "..", "examples", "nodal", "input.kdl"])
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
    assert!(stdout.trim().is_empty());

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
    assert!(stdout.contains("generation_limit"));

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
    assert_eq!(sets["generators"]["symbol"], "g");
    assert_eq!(sets["nodes"]["symbol"], "n");

    let variables = object
        .get("variables")
        .and_then(serde_json::Value::as_array)
        .expect("variables array");
    let dispatch = variables
        .iter()
        .find(|variable| variable["name"] == "dispatch")
        .expect("dispatch variable");
    assert_eq!(
        dispatch["set"][0],
        serde_json::json!({"$ref": "#/sets/generators"})
    );
    assert_eq!(
        dispatch["set"][1],
        serde_json::json!({"$ref": "#/sets/nodes"})
    );
    assert_eq!(
        dispatch["set"][2],
        serde_json::json!({"$ref": "#/sets/time"})
    );

    let parameters = object
        .get("parameters")
        .and_then(serde_json::Value::as_object)
        .expect("parameters object");
    assert!(parameters.contains_key("distance_km"));
    assert!(parameters.contains_key("cost_spur_usd_per_km_mw"));
    assert!(parameters.contains_key("MWLoad"));
    assert_eq!(
        parameters["distance_km"]["set"],
        serde_json::json!([
            {"$ref": "#/sets/generators"},
            {"$ref": "#/sets/nodes"}
        ])
    );
    assert_eq!(
        parameters["MWLoad"]["set"],
        serde_json::json!([
            {"$ref": "#/sets/nodes"}
        ])
    );

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
    assert!(
        items
            .iter()
            .any(|item| item["name"] == "area" && item["cardinality"] == 73)
    );
    assert!(
        items
            .iter()
            .any(|item| item["name"] == "generators" && item["cardinality"] == 2011)
    );
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
        item["name"] == "generation_limit"
            && item["relation"] == "less_or_equal"
            && item["lhs_terms"]
                .as_array()
                .is_some_and(|terms| !terms.is_empty())
            && item["scope"]
                .as_array()
                .is_some_and(|scope| !scope.is_empty())
            && item["variable_refs"] == serde_json::json!([{"$ref": "#/variables/dispatch"}])
            && item["parameter_refs"]
                == serde_json::json!([
                    {"$ref": "#/parameters/capacity_factor"},
                    {"$ref": "#/parameters/required_capacity"}
                ])
    }));
    assert!(items.iter().any(|item| {
        item["name"] == "meet_nodal_demand"
            && item["relation"] == "equal"
            && item["lhs_terms"]
                .as_array()
                .is_some_and(|terms| !terms.is_empty())
    }));

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
        item["name"] == "new_capacity"
            && item["notation"] == "new_capacity[g, n]"
            && item["set"]
                == serde_json::json!([
                    {"index": "g", "name": "generators", "cardinality": 2011},
                    {"index": "n", "name": "nodes", "cardinality": 73}
                ])
            && item["domain"]
                == serde_json::json!({
                    "kind": "continuous",
                    "lower": 0,
                    "upper": null
                })
    }));
    assert!(items.iter().any(|item| {
        item["name"] == "dispatch"
            && item["notation"] == "dispatch[g, n, t]"
            && item["set"]
                == serde_json::json!([
                    {"index": "g", "name": "generators", "cardinality": 2011},
                    {"index": "n", "name": "nodes", "cardinality": 73},
                    {"index": "t", "name": "time", "cardinality": 2}
                ])
            && item["domain"]
                == serde_json::json!({
                    "kind": "continuous",
                    "lower": 0,
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
    assert!(stdout.contains("notation : new_capacity[g, n]"));
    assert!(stdout.contains("domains  : g ∈ generators, n ∈ nodes"));
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
    assert!(items.iter().any(|item| {
        item["name"] == "distance_km"
            && item["set"]
                == serde_json::json!([
                    {"$ref": "#/sets/generators"},
                    {"$ref": "#/sets/nodes"}
                ])
    }));
    assert!(items.iter().any(|item| {
        item["name"] == "MWLoad" && item["set"] == serde_json::json!([{"$ref": "#/sets/nodes"}])
    }));

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
    assert!(stderr.contains("area"));

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
        serde_json::json!([{"$ref": "#/variables/new_capacity"}])
    );
    assert_eq!(
        items[0]["parameter_refs"],
        serde_json::json!([
            {"$ref": "#/parameters/cost_poi_usd_per_mw"},
            {"$ref": "#/parameters/cost_reinforcement_usd_per_mw"},
            {"$ref": "#/parameters/cost_spur_usd_per_km_mw"},
            {"$ref": "#/parameters/distance_km"},
            {"$ref": "#/parameters/pair_exists"}
        ])
    );
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
