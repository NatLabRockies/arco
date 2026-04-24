use std::path::PathBuf;
use std::process::Command;
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::Value;

fn example_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

fn run_cli(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_arco"))
        .args(args)
        .output()
        .expect("failed to execute arco binary")
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("arco-cli-{prefix}-{}-{nanos}", std::process::id()))
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

    assert!(payload.get("meta").is_some());
    assert!(payload.get("set").is_some());
    assert!(payload.get("variable").is_some());
    assert!(payload.get("parameter").is_some());
    assert!(payload.get("constraint").is_some());
    assert!(payload.get("objective").is_some());

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

#[test]
fn run_compact_nodal_allocation_tracer_bullet_succeeds() {
    let model_path = example_path("examples/nodal-allocation/input.kdl");
    let model = model_path
        .to_str()
        .expect("example path contains invalid unicode");

    let inspect_output = run_cli(&["inspect", model, "--json"]);
    assert!(
        inspect_output.status.success(),
        "inspect failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&inspect_output.stdout),
        String::from_utf8_lossy(&inspect_output.stderr)
    );

    let inspect_payload: Value =
        serde_json::from_slice(&inspect_output.stdout).expect("valid inspect json");
    let variables = inspect_payload["variable"]
        .as_array()
        .expect("variable array");
    let dispatch = variables
        .iter()
        .find(|record| record["name"] == "dispatch")
        .expect("dispatch variable record");
    let sets = dispatch["set"].as_array().expect("dispatch set array");
    assert_eq!(
        sets.len(),
        4,
        "tuple-domain variable should keep four components"
    );
    for binding in sets {
        assert_eq!(binding["name"], "feasible_links");
    }

    let run_output = run_cli(&["run", model, "--compact"]);
    assert!(
        run_output.status.success(),
        "run failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run_output.stdout),
        String::from_utf8_lossy(&run_output.stderr)
    );

    let summary: Value = serde_json::from_slice(&run_output.stdout).expect("valid run json");
    assert_eq!(summary["active_scenario"], "NodalAllocationDay");
    assert_eq!(summary["objective"]["name"], "tuple_membership_tracer");
    assert_eq!(summary["objective"]["value"], 0.0);
    assert_eq!(summary["solve_status"], "optimal");

    let counts = summary["counts"].as_object().expect("counts object");
    assert_eq!(counts.get("variables"), Some(&Value::from(1)));
    assert_eq!(counts.get("constraints"), Some(&Value::from(2)));
}

#[test]
fn validate_surfaces_empty_filtered_subset_warning() {
    let root = unique_temp_dir("validate-empty-filtered-subset");
    let data_dir = root.join("data");
    fs::create_dir_all(&data_dir).expect("create temp data dir");
    fs::write(data_dir.join("assets.csv"), "asset\nA\nB\n").expect("write csv");

    let model_path = root.join("input.kdl");
    fs::write(
        &model_path,
        r#"
data "assets_data" source="data/assets.csv" {
  set "asset"
  set "missing_assets" {
    in "asset"
    filter { asset == "missing" }
  }
}

model "Dispatch" {
  set "asset"

  control "x" {
    index "asset"
  }

  minimize "Obj" {
    sum(x[asset] for asset in asset)
  }
}

scenario "S1" {
  use "Dispatch"
}
"#,
    )
    .expect("write model");

    let model = model_path
        .to_str()
        .expect("model path contains invalid unicode");
    let output = run_cli(&["validate", model]);

    assert!(
        output.status.success(),
        "command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Validated file://"));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("filtered subset resolved empty"),
        "expected filtered-subset warning in stderr, got:\n{stderr}"
    );

    let _ = fs::remove_dir_all(root);
}
