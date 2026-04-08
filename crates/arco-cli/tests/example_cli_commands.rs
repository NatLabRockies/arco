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
        assert_cli_success(&["inspect", model]);
        assert_cli_success(&["run", model, "--compact"]);
    }
}

#[test]
fn inspect_produces_valid_toml() {
    let model_path = example_path("examples/capacity-expansion/input.kdl");
    let model = model_path
        .to_str()
        .expect("example path contains invalid unicode");

    let output = run_cli(&["inspect", model]);
    assert!(
        output.status.success(),
        "command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: toml::Value = toml::from_str(&stdout).expect("output should be valid TOML");

    // Check top-level sections exist
    assert!(parsed.get("meta").is_some(), "should have meta section");
    assert!(parsed.get("set").is_some(), "should have set section");
    assert!(
        parsed.get("variable").is_some(),
        "should have variable section"
    );
    assert!(
        parsed.get("parameter").is_some(),
        "should have parameter section"
    );
    assert!(
        parsed.get("constraint").is_some(),
        "should have constraint section"
    );
    assert!(
        parsed.get("objective").is_some(),
        "should have objective section"
    );
}

#[test]
fn inspect_json_produces_valid_json() {
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

    // Check structure matches TOML layout
    assert!(payload.get("meta").is_some());
    assert!(payload.get("set").is_some());
    assert!(payload.get("variable").is_some());
    assert!(payload.get("parameter").is_some());
    assert!(payload.get("constraint").is_some());
    assert!(payload.get("objective").is_some());

    // Check counts
    let counts = payload["meta"]["counts"]
        .as_object()
        .expect("counts object");
    assert!(counts.get("set").is_some());
    assert!(counts.get("variable").is_some());
    assert!(counts.get("parameter").is_some());
    assert!(counts.get("constraint").is_some());
}

#[test]
fn inspect_json_constraints_have_structured_refs() {
    let model_path = example_path("examples/capacity-expansion/input.kdl");
    let model = model_path
        .to_str()
        .expect("example path contains invalid unicode");

    let output = run_cli(&["inspect", model, "--json"]);
    assert!(output.status.success());

    let payload: Value = serde_json::from_slice(&output.stdout).expect("valid inspect json");
    let constraints = payload["constraint"].as_array().expect("constraint array");

    assert!(!constraints.is_empty());

    // Each constraint should have structured fields
    for constraint in constraints {
        assert!(constraint.get("name").is_some());
        assert!(constraint.get("relation").is_some());
        assert!(constraint.get("template").is_some());
        assert!(constraint.get("source").is_some());
        assert!(constraint.get("scope").is_some());
        assert!(constraint.get("lhs").is_some());
        assert!(constraint.get("rhs").is_some());
        assert!(constraint.get("instances").is_some());
    }
}

#[test]
fn inspect_json_variables_have_set_bindings() {
    let model_path = example_path("examples/capacity-expansion/input.kdl");
    let model = model_path
        .to_str()
        .expect("example path contains invalid unicode");

    let output = run_cli(&["inspect", model, "--json"]);
    assert!(output.status.success());

    let payload: Value = serde_json::from_slice(&output.stdout).expect("valid inspect json");
    let variables = payload["variable"].as_array().expect("variable array");

    assert!(!variables.is_empty());

    for variable in variables {
        assert!(variable.get("name").is_some());
        assert!(variable.get("kind").is_some());
        assert!(variable.get("set").is_some());

        // Each set binding should have name and size
        let sets = variable["set"].as_array().expect("set array");
        for set_binding in sets {
            assert!(set_binding.get("name").is_some());
            assert!(set_binding.get("size").is_some());
        }
    }
}

#[test]
fn inspect_json_parameters_have_dtype() {
    let model_path = example_path("examples/capacity-expansion/input.kdl");
    let model = model_path
        .to_str()
        .expect("example path contains invalid unicode");

    let output = run_cli(&["inspect", model, "--json"]);
    assert!(output.status.success());

    let payload: Value = serde_json::from_slice(&output.stdout).expect("valid inspect json");
    let parameters = payload["parameter"].as_array().expect("parameter array");

    assert!(!parameters.is_empty());

    for param in parameters {
        assert!(param.get("name").is_some());
        assert!(param.get("kind").is_some());
        assert!(param.get("dtype").is_some());
    }
}
