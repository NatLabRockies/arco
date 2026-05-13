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

fn cli_fixture_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(relative)
}

fn run_cli(args: &[&str]) -> std::process::Output {
    run_cli_with_env(args, &[])
}

fn run_cli_with_env(args: &[&str], envs: &[(&str, &str)]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_arco"));
    command.args(args);

    let has_config_dir = envs
        .iter()
        .any(|(key, _)| *key == "ARCO_CONFIG_DIR" || *key == "ARCO_PROJECT_CONFIG_DIR");
    let config_root = if has_config_dir {
        None
    } else {
        let root = unique_temp_dir("cli-config");
        fs::create_dir_all(&root).expect("create temp config dir");
        command.env("ARCO_CONFIG_DIR", &root);
        Some(root)
    };

    for (key, value) in envs {
        command.env(key, value);
    }

    let output = command.output().expect("failed to execute arco binary");
    if let Some(root) = config_root {
        let _ = fs::remove_dir_all(root);
    }
    output
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("arco-cli-{prefix}-{}-{nanos}", std::process::id()))
}

#[cfg(feature = "xpress")]
fn local_xpress_dir() -> Option<String> {
    if let Some(path) = std::env::var("XPRESSDIR")
        .ok()
        .filter(|value| !value.is_empty())
    {
        if PathBuf::from(&path).exists() {
            return Some(path);
        }
    }

    let mut candidates = Vec::new();
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        candidates.push(home.join("User Apps").join("FICO Xpress").join("xpressmp"));
        candidates.push(home.join("opt").join("xpressmp"));
    }
    if let Some(user_profile) = std::env::var_os("USERPROFILE") {
        let user_profile = PathBuf::from(user_profile);
        candidates.push(
            user_profile
                .join("AppData")
                .join("Local")
                .join("FICO Xpress")
                .join("xpressmp"),
        );
    }
    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        let program_files = PathBuf::from(program_files);
        candidates.push(program_files.join("FICO Xpress").join("xpressmp"));
    }
    if let Some(program_files_x86) = std::env::var_os("ProgramFiles(x86)") {
        let program_files_x86 = PathBuf::from(program_files_x86);
        candidates.push(program_files_x86.join("FICO Xpress").join("xpressmp"));
    }
    candidates.extend([
        PathBuf::from("/Applications/FICO Xpress/xpressmp"),
        PathBuf::from("/Volumes/FICO Xpress Installer/FICO Xpress/xpressmp"),
        PathBuf::from("/opt/xpressmp"),
        PathBuf::from("/Library/xpressmp"),
        PathBuf::from("C:\\xpressmp"),
    ]);

    candidates
        .into_iter()
        .find(|path| path.exists())
        .map(|path| path.display().to_string())
}

#[test]
fn version_flag_prints_arco_version() {
    let output = run_cli(&["--version"]);
    assert!(
        output.status.success(),
        "command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("arco {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn self_update_without_dist_receipt_reports_standalone_requirement() {
    let output = run_cli(&["self", "update"]);
    assert!(
        !output.status.success(),
        "self update should fail without a cargo-dist receipt\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("Self-update is only available for arco binaries installed via the standalone installation scripts")
    );
}

#[test]
fn kdl_check_json_succeeds_for_valid_model() {
    let model_path = example_path("examples/capacity-expansion/input.kdl");
    let model = model_path
        .to_str()
        .expect("example path contains invalid unicode");

    let output = run_cli(&["kdl", "check", model, "--format", "json"]);
    assert!(
        output.status.success(),
        "command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value = serde_json::from_slice(&output.stdout).expect("valid check json");
    assert_eq!(payload["valid"], Value::Bool(true));
    assert_eq!(
        payload["diagnostics"]
            .as_array()
            .expect("diagnostics array")
            .len(),
        0
    );
}

#[test]
fn kdl_check_json_reports_invalid_model() {
    let root = unique_temp_dir("kdl-check-invalid");
    fs::create_dir_all(&root).expect("create temp dir");
    let model_path = root.join("input.kdl");
    fs::write(&model_path, "technology \"thermal\" {}\n").expect("write invalid model");

    let model = model_path
        .to_str()
        .expect("model path contains invalid unicode");
    let output = run_cli(&["kdl", "check", model, "--format", "json"]);
    assert!(
        !output.status.success(),
        "invalid model should exit non-zero\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value = serde_json::from_slice(&output.stdout).expect("valid check json");
    assert_eq!(payload["valid"], Value::Bool(false));
    let diagnostics = payload["diagnostics"]
        .as_array()
        .expect("diagnostics array");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0]["severity"], "error");
    assert_eq!(diagnostics[0]["line"], Value::from(1));
    assert!(
        diagnostics[0]["message"]
            .as_str()
            .expect("message string")
            .contains("unsupported declaration `technology`")
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn kdl_check_json_reports_included_file_path() {
    let model_path = cli_fixture_path("composition-invalid/input.kdl");
    let model = model_path
        .to_str()
        .expect("model path contains invalid unicode");

    let output = run_cli(&["kdl", "check", model, "--format", "json"]);
    assert!(
        !output.status.success(),
        "invalid model should exit non-zero\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value = serde_json::from_slice(&output.stdout).expect("valid check json");
    let diagnostics = payload["diagnostics"]
        .as_array()
        .expect("diagnostics array");
    assert_eq!(diagnostics.len(), 1);
    assert!(
        diagnostics[0]["file"]
            .as_str()
            .expect("diagnostic file")
            .ends_with("composition-invalid/bad-fragment.kdl")
    );
    assert_eq!(diagnostics[0]["line"], Value::from(1));
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

#[cfg(feature = "xpress")]
#[test]
fn solver_set_and_run_use_xpress_backend_when_available() {
    let Some(xpress_dir) = local_xpress_dir() else {
        return;
    };

    let root = unique_temp_dir("solver-xpress");
    let config_dir = root.join("config");
    fs::create_dir_all(&config_dir).expect("create config dir");

    let set_output = run_cli_with_env(
        &["solver", "set", "xpress"],
        &[
            ("ARCO_CONFIG_DIR", config_dir.to_str().expect("config dir")),
            ("XPRESSDIR", &xpress_dir),
        ],
    );
    assert!(
        set_output.status.success(),
        "solver set failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&set_output.stdout),
        String::from_utf8_lossy(&set_output.stderr)
    );

    let model = example_path("examples/dense-lp/input.kdl");
    let run_output = run_cli_with_env(
        &["run", model.to_str().expect("model path"), "--compact"],
        &[
            ("ARCO_CONFIG_DIR", config_dir.to_str().expect("config dir")),
            ("XPRESSDIR", &xpress_dir),
        ],
    );
    if String::from_utf8_lossy(&run_output.stderr).contains("Xpress license initialization failed")
    {
        let _ = fs::remove_dir_all(root);
        return;
    }
    assert!(
        run_output.status.success(),
        "xpress run failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run_output.stdout),
        String::from_utf8_lossy(&run_output.stderr)
    );

    let payload: Value = serde_json::from_slice(&run_output.stdout).expect("valid run json");
    assert_eq!(
        payload["backend"],
        Value::String("arco-rust-xpress".to_string())
    );
    assert_eq!(
        payload["solve_status"],
        Value::String("optimal".to_string())
    );

    let _ = fs::remove_dir_all(root);
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

    let inspect_sets = inspect_payload["set"].as_array().expect("set array");
    let feasible_links = inspect_sets
        .iter()
        .find(|record| record["name"] == "feasible_links")
        .expect("feasible_links set record");
    assert_eq!(
        feasible_links["size"],
        Value::from(4),
        "feasible_links should report tuple-row cardinality"
    );

    let priority_links = inspect_sets
        .iter()
        .find(|record| record["name"] == "priority_links")
        .expect("priority_links set record");
    assert_eq!(
        priority_links["size"],
        Value::from(2),
        "priority_links should report filtered tuple-row cardinality"
    );

    let variables = inspect_payload["variable"]
        .as_array()
        .expect("variable array");
    let capacity = variables
        .iter()
        .find(|record| record["name"] == "capacity_nodal_site")
        .expect("capacity_nodal_site variable record");
    let sets = capacity["set"]
        .as_array()
        .expect("capacity_nodal_site set array");
    assert_eq!(
        sets.len(),
        4,
        "tuple-domain variable should keep four components"
    );
    for binding in sets {
        assert_eq!(binding["name"], "feasible_links");

        let expected_size = match binding["as"].as_str().expect("tuple binding alias") {
            "a" => 2,
            "i" => 3,
            "g" => 5,
            "b" => 4,
            other => panic!("unexpected tuple binding alias: {other}"),
        };

        assert_eq!(
            binding["size"],
            Value::from(expected_size),
            "tuple-domain binding should use tuple-component domain size"
        );
    }

    let constraints = inspect_payload["constraint"]
        .as_array()
        .expect("constraint array");
    let investment_capacity = constraints
        .iter()
        .find(|record| record["name"] == "investment_capacity")
        .expect("investment_capacity constraint record");
    assert_eq!(
        investment_capacity["instances"],
        Value::from(4),
        "tuple-domain constraint instances should track tuple rows, not Cartesian powers"
    );

    let priority_floor = constraints
        .iter()
        .find(|record| record["name"] == "priority_floor")
        .expect("priority_floor constraint record");
    assert_eq!(
        priority_floor["instances"],
        Value::from(2),
        "filtered tuple-domain constraint instances should track tuple rows"
    );

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
    assert_eq!(counts.get("constraints"), Some(&Value::from(3)));
}

#[test]
fn run_accepts_solver_log_flag() {
    let model_path = example_path("examples/nodal-allocation/input.kdl");
    let model = model_path
        .to_str()
        .expect("example path contains invalid unicode");

    let output = run_cli(&["run", model, "--compact", "--solver-log"]);
    assert!(
        output.status.success(),
        "run failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // With `--solver-log`, the embedded HiGHS backend interleaves its log
    // lines with the run summary on stdout. Pull the last line that starts
    // with `{` and parse just that as the JSON summary.
    let stdout = String::from_utf8(output.stdout).expect("stdout is valid utf-8");
    let json_line = stdout
        .lines()
        .rev()
        .find(|line| line.trim_start().starts_with('{'))
        .expect("run summary JSON line present in stdout");
    let summary: Value = serde_json::from_str(json_line).expect("valid run json");
    assert_eq!(summary["active_scenario"], "NodalAllocationDay");
    assert_eq!(summary["solve_status"], "optimal");
}

#[test]
fn inspect_uses_canonical_set_size_for_alias_collision_bindings() {
    let root = unique_temp_dir("inspect-alias-collision");
    let data_dir = root.join("data");
    fs::create_dir_all(&data_dir).expect("create temp data dir");
    fs::write(
        data_dir.join("rows.csv"),
        "i,node\n1,a\n1,b\n2,c\n2,d\n3,e\n",
    )
    .expect("write csv");

    let model_path = root.join("input.kdl");
    fs::write(
        &model_path,
        r#"
data collision_data source="data/rows.csv" {
  map nodes from="node"
  set i
  set nodes alias="i"
}

model Collision {
  set i
  set nodes

  control x {
    index nodes
  }

  constraint c {
    index n { in i }
    expression { x[n] <= 1 }
  }

  minimize Obj {
    sum(x[n] for n in nodes)
  }
}

scenario S1 {
  use Collision
}
"#,
    )
    .expect("write model");

    let model = model_path
        .to_str()
        .expect("model path contains invalid unicode");
    let output = run_cli(&["inspect", model, "--json"]);

    assert!(
        output.status.success(),
        "inspect failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value = serde_json::from_slice(&output.stdout).expect("valid inspect json");
    let constraints = payload["constraint"].as_array().expect("constraint array");
    let constraint = constraints
        .iter()
        .find(|record| record["name"] == "c")
        .expect("constraint record");

    assert_eq!(constraint["instances"], Value::from(5));
    assert_eq!(constraint["scope"][0]["size"], Value::from(5));
    assert_eq!(constraint["lhs"][0]["over"][0]["size"], Value::from(5));
}

#[test]
fn inspect_set_records_include_all_aliases() {
    let root = unique_temp_dir("inspect-multi-alias");
    let data_dir = root.join("data");
    fs::create_dir_all(&data_dir).expect("create temp data dir");
    fs::write(data_dir.join("lines.csv"), "lines\nL1\nL2\nL3\n").expect("write csv");

    let model_path = root.join("input.kdl");
    fs::write(
        &model_path,
        r#"
data line_data source="data/lines.csv" {
  set lines alias="i"
}

model AliasModel {
  set lines alias="j"

  control flow {
    index lines
  }

  minimize Obj {
    sum(flow[l] for l in lines)
  }
}

scenario S1 {
  use AliasModel
}
"#,
    )
    .expect("write model");

    let model = model_path
        .to_str()
        .expect("model path contains invalid unicode");
    let output = run_cli(&["inspect", model, "--json"]);

    assert!(
        output.status.success(),
        "inspect failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value = serde_json::from_slice(&output.stdout).expect("valid inspect json");
    let sets = payload["set"].as_array().expect("set array");
    let lines_set = sets
        .iter()
        .find(|record| record["name"] == "lines")
        .expect("lines set record");

    let aliases = lines_set["aliases"].as_array().expect("aliases array");
    let alias_values: Vec<&str> = aliases
        .iter()
        .map(|value| value.as_str().expect("alias string"))
        .collect();

    assert_eq!(alias_values, vec!["i", "j"]);
}

#[test]
fn run_fails_for_unsupported_embedded_family_selection() {
    let model_path = example_path("examples/dense-lp/input.kdl");
    let model = model_path
        .to_str()
        .expect("example path contains invalid unicode");

    let user_config_dir = unique_temp_dir("solver-config-user-xpress");
    let project_config_dir = unique_temp_dir("solver-config-project-xpress");
    fs::create_dir_all(&user_config_dir).expect("create user config dir");
    fs::create_dir_all(&project_config_dir).expect("create project config dir");

    fs::write(
        user_config_dir.join("solver.toml"),
        "version = 1\ndefault_selection = \"xpress\"\n",
    )
    .expect("write user solver config");

    let user_config_dir_str = user_config_dir.to_string_lossy().into_owned();
    let project_config_dir_str = project_config_dir.to_string_lossy().into_owned();

    let output = run_cli_with_env(
        &["run", model, "--compact"],
        &[
            ("ARCO_CONFIG_DIR", user_config_dir_str.as_str()),
            ("ARCO_PROJECT_CONFIG_DIR", project_config_dir_str.as_str()),
        ],
    );

    assert!(
        !output.status.success(),
        "xpress embedded selection should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("embedded solver family 'xpress' is not available"));

    let _ = fs::remove_dir_all(user_config_dir);
    let _ = fs::remove_dir_all(project_config_dir);
}

#[test]
fn run_external_scip_profile_reports_unsupported_transport() {
    let model_path = example_path("examples/dense-lp/input.kdl");
    let model = model_path
        .to_str()
        .expect("example path contains invalid unicode");

    let user_config_dir = unique_temp_dir("solver-config-user-scip");
    let project_config_dir = unique_temp_dir("solver-config-project-scip");
    fs::create_dir_all(&user_config_dir).expect("create user config dir");
    fs::create_dir_all(&project_config_dir).expect("create project config dir");

    fs::write(
        user_config_dir.join("solver.toml"),
        "version = 1\ndefault_selection = \"scip-missing\"\n\n[profiles.scip-missing]\nname = \"scip-missing\"\nfamily = \"scip\"\ntransport = \"external_process\"\nexecutable = \"/definitely/missing/scip\"\n",
    )
    .expect("write user solver config");

    let user_config_dir_str = user_config_dir.to_string_lossy().into_owned();
    let project_config_dir_str = project_config_dir.to_string_lossy().into_owned();

    let output = run_cli_with_env(
        &["run", model, "--compact"],
        &[
            ("ARCO_CONFIG_DIR", user_config_dir_str.as_str()),
            ("ARCO_PROJECT_CONFIG_DIR", project_config_dir_str.as_str()),
        ],
    );

    assert!(
        !output.status.success(),
        "external scip profile should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("external-process solver family 'scip' is not available"),
        "expected unsupported external-process failure\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        stderr
    );

    let _ = fs::remove_dir_all(user_config_dir);
    let _ = fs::remove_dir_all(project_config_dir);
}

#[test]
fn run_reports_parameter_variable_and_expression() {
    let model_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("report-kinds")
        .join("input.kdl");

    let model = model_path
        .to_str()
        .expect("model path contains invalid unicode");
    let output = run_cli(&["run", model]);

    assert!(
        output.status.success(),
        "run failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value = serde_json::from_slice(&output.stdout).expect("valid run json");
    let reports = payload["reports"].as_array().expect("reports array");

    let variable_report = reports
        .iter()
        .find(|report| report["name"] == "x")
        .expect("variable report x");
    assert_eq!(variable_report["index"], Value::from(vec!["t"]));
    assert_eq!(
        variable_report["values"]
            .as_array()
            .expect("x values")
            .len(),
        2
    );

    let expression_report = reports
        .iter()
        .find(|report| report["name"] == "total_x")
        .expect("expression report total_x");
    assert_eq!(expression_report["values"][0]["value"], Value::from(5.0));

    let parameter_report = reports
        .iter()
        .find(|report| report["name"] == "total_cap")
        .expect("parameter-derived report total_cap");
    assert_eq!(parameter_report["values"][0]["value"], Value::from(5.0));
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
