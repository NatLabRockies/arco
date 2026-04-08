use std::path::PathBuf;
use std::process::Command;

use serde_json::Value;

fn example_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

fn assert_cli_success(args: &[&str]) {
    let output = run_cli(args);

    assert!(
        output.status.success(),
        "command `arco {}` failed\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_cli(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_arco"))
        .args(args)
        .output()
        .expect("failed to execute arco binary")
}

#[test]
fn examples_support_validate_print_model_inspect_and_run() {
    let examples = [
        "examples/generator-allocation/input.kdl",
        "examples/price-taker-battery/input.kdl",
        "examples/simple-electricity-market-storage/input.kdl",
        "examples/capacity-expansion/input.kdl",
    ];

    for example in examples {
        let model_path = example_path(example);
        let model = model_path
            .to_str()
            .expect("example path contains invalid unicode");

        assert_cli_success(&["validate", model]);
        assert_cli_success(&["print-model", model]);
        assert_cli_success(&["inspect", model, "--section", "constraints"]);
        assert_cli_success(&["run", model, "--compact"]);
    }
}

#[test]
fn inspect_objective_expands_refs_from_named_expressions() {
    let model_path = example_path("examples/capacity-expansion/input.kdl");
    let model = model_path
        .to_str()
        .expect("example path contains invalid unicode");

    let output = run_cli(&["inspect", model, "--section", "objective", "--json"]);
    assert!(
        output.status.success(),
        "command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value = serde_json::from_slice(&output.stdout).expect("valid objective json");
    let objective = payload
        .as_array()
        .and_then(|items| items.first())
        .expect("objective section returns one objective item");

    let variable_refs = objective
        .get("variable_refs")
        .and_then(Value::as_array)
        .expect("objective includes variable refs");
    let parameter_refs = objective
        .get("parameter_refs")
        .and_then(Value::as_array)
        .expect("objective includes parameter refs");

    let variable_ref_values = variable_refs
        .iter()
        .filter_map(|value| value.get("$ref"))
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    let parameter_ref_values = parameter_refs
        .iter()
        .filter_map(|value| value.get("$ref"))
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();

    assert!(variable_ref_values.contains(&"#/variables/dispatch"));
    assert!(variable_ref_values.contains(&"#/variables/expansion"));
    assert!(variable_ref_values.contains(&"#/variables/unserved_energy"));

    assert!(parameter_ref_values.contains(&"#/parameters/build_cost"));
    assert!(parameter_ref_values.contains(&"#/parameters/variable_cost"));
    assert!(parameter_ref_values.contains(&"#/parameters/voll"));
}

#[test]
fn inspect_full_json_omits_empty_chronologies() {
    let model_path = example_path("examples/capacity-expansion/input.kdl");
    let model = model_path
        .to_str()
        .expect("example path contains invalid unicode");

    let output = run_cli(&["inspect", model, "--json"]);
    assert!(
        output.status.success(),
        "command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value = serde_json::from_slice(&output.stdout).expect("valid full inspect json");
    assert!(
        payload.get("chronologies").is_none(),
        "full inspect should omit empty chronology section"
    );
}

#[test]
fn inspect_parameters_section_uses_set_dimensions() {
    let model_path = example_path("examples/capacity-expansion/input.kdl");
    let model = model_path
        .to_str()
        .expect("example path contains invalid unicode");

    let output = run_cli(&["inspect", model, "--section", "parameters", "--json"]);
    assert!(
        output.status.success(),
        "command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value = serde_json::from_slice(&output.stdout).expect("valid parameter json");
    let sets = payload
        .get("sets")
        .and_then(Value::as_object)
        .expect("parameter section includes set catalog");
    assert!(sets.contains_key("asset_id"));

    let parameters = payload
        .get("parameters")
        .and_then(Value::as_object)
        .expect("parameter section includes parameter table");

    assert!(
        parameters.get("build_cost").is_none(),
        "parameter table should not include repeated inferred base entries"
    );

    let build_cost = parameters
        .get("build_cost[asset_id]")
        .expect("build_cost indexed parameter present");

    let set_refs = build_cost
        .get("set")
        .and_then(Value::as_array)
        .expect("indexed parameter includes set refs");

    let ref_values = set_refs
        .iter()
        .filter_map(|value| value.get("$ref"))
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();

    assert!(ref_values.contains(&"#/sets/asset_id"));
    assert!(build_cost.get("type").is_none());
}

#[test]
fn inspect_variables_section_includes_set_catalog() {
    let model_path = example_path("examples/capacity-expansion/input.kdl");
    let model = model_path
        .to_str()
        .expect("example path contains invalid unicode");

    let output = run_cli(&["inspect", model, "--section", "variables", "--json"]);
    assert!(
        output.status.success(),
        "command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value = serde_json::from_slice(&output.stdout).expect("valid variable json");
    let sets = payload
        .get("sets")
        .and_then(Value::as_object)
        .expect("variable section includes set catalog");
    assert!(sets.contains_key("time"));

    let variables = payload
        .get("variables")
        .and_then(Value::as_array)
        .expect("variable section includes variable cards");
    assert!(!variables.is_empty());
}

#[test]
fn inspect_full_json_parameters_table_omits_repeated_bases() {
    let model_path = example_path("examples/capacity-expansion/input.kdl");
    let model = model_path
        .to_str()
        .expect("example path contains invalid unicode");

    let output = run_cli(&["inspect", model, "--json"]);
    assert!(
        output.status.success(),
        "command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value = serde_json::from_slice(&output.stdout).expect("valid inspect json");
    let parameters = payload
        .get("parameters")
        .and_then(Value::as_object)
        .expect("full inspect includes parameter table");

    assert!(parameters.get("build_cost").is_none());
    assert!(parameters.get("build_cost[asset_id]").is_some());
}
