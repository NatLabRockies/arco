use arco_ops::ArcoOps;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn issue_262_fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/issue-262/input.kdl")
}

fn write_temp_model(contents: &str) -> Result<(TempDir, PathBuf), Box<dyn std::error::Error>> {
    write_temp_workspace(contents, &[])
}

fn write_temp_workspace(
    contents: &str,
    extra_files: &[(&str, &str)],
) -> Result<(TempDir, PathBuf), Box<dyn std::error::Error>> {
    let workspace = tempfile::Builder::new()
        .prefix("arco-ops-indexed-expression-")
        .tempdir()?;
    let root = workspace.path();
    fs::create_dir_all(&root)?;
    for (relative_path, file_contents) in extra_files {
        let file_path = root.join(relative_path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(file_path, file_contents)?;
    }
    let path = root.join("input.kdl");
    fs::write(&path, contents)?;
    Ok((workspace, path))
}

#[test]
fn compiles_generated_indexed_expression_with_positional_index_binding()
-> Result<(), Box<dyn std::error::Error>> {
    let model = r#"
set "bus" {
  "b1"
  "b2"
}

model "Dispatch" {
  control "dispatch_new" lower=0 {
    index "b" in="bus"
  }

  control "dispatch_existing" lower=0 {
    index "b" in="bus"
  }

  expression "net_injection_by_bus" {
    index "b" in="bus"
    expression {
      dispatch_new[b] + dispatch_existing[b]
    }
  }

  constraint "balance" {
    expression {
      sum(net_injection_by_bus[bb] for bb in bus) = 0
    }
  }

  minimize "TotalCost" {
    0
  }
}

scenario "Base" {
  use "Dispatch"
}
"#;

    let (_workspace, path) = write_temp_model(model)?;
    let compiled = ArcoOps::compile_file(&path)?;

    assert!(!compiled.compiled_problem.algebra.constraints.is_empty());

    Ok(())
}

#[test]
fn rejects_generated_indexed_expression_arity_mismatch() -> Result<(), Box<dyn std::error::Error>> {
    let model = r#"
set "bus" {
  "b1"
}

set "period" {
  "p1"
}

model "Dispatch" {
  control "dispatch_new" lower=0 {
    index "b" in="bus"
  }

  expression "net_injection_by_bus" {
    index "b" in="bus"
    expression {
      dispatch_new[b]
    }
  }

  constraint "balance" {
    expression {
      sum(net_injection_by_bus[bb, pp] for bb in bus for pp in period) = 0
    }
  }

  minimize "TotalCost" {
    0
  }
}

scenario "Base" {
  use "Dispatch"
}
"#;

    let (_workspace, path) = write_temp_model(model)?;
    let error = ArcoOps::compile_file(&path)
        .expect_err("indexed expression arity mismatch should fail compilation");
    let error_message = error.to_string();

    assert!(error_message.contains(
        "indexed expression `net_injection_by_bus` expects 1 index value(s), received 2"
    ));

    Ok(())
}

#[test]
fn compiles_generated_indexed_expression_with_if_filter() -> Result<(), Box<dyn std::error::Error>>
{
    let model = r#"
data "bus_data" source="data/bus.csv" {
  set "bus"
  set "dispatchable_bus" {
    in "bus"
    filter { active == 1 }
  }
  param "active" index="bus"
}

model "Dispatch" {
  control "dispatch_new" lower=0 {
    index "b" in="dispatchable_bus"
  }

  expression "net_injection_by_bus" {
    index "b" in="bus"
    if { active[b] == 1 }
    expression {
      dispatch_new[b]
    }
  }

  constraint "balance" {
    expression {
      sum(net_injection_by_bus[bb] for bb in bus) = 0
    }
  }

  minimize "TotalCost" {
    0
  }
}

scenario "Base" {
  use "Dispatch"
}
"#;

    let (_workspace, path) =
        write_temp_workspace(model, &[("data/bus.csv", "bus,active\nb1,1\nb2,0\n")])?;
    let compiled = ArcoOps::compile_file(&path)?;

    assert!(!compiled.compiled_problem.algebra.constraints.is_empty());

    Ok(())
}

#[test]
fn compiles_issue_262_net_injection_ptdf_fixture() -> Result<(), Box<dyn std::error::Error>> {
    let compiled = ArcoOps::compile_file(&issue_262_fixture_path())?;

    assert!(!compiled.compiled_problem.algebra.constraints.is_empty());

    Ok(())
}

#[test]
fn rejects_generated_expression_with_unresolved_identifier()
-> Result<(), Box<dyn std::error::Error>> {
    let model = r#"
set "bus" {
  "b1"
}

model "Dispatch" {
  control "dispatch_new" lower=0 {
    index "b" in="bus"
  }

  expression "net_injection_by_bus" {
    index "b" in="bus"
    expression {
      dispatch_new[b] - unknown_load[b]
    }
  }

  constraint "balance" {
    expression {
      sum(net_injection_by_bus[bb] for bb in bus) = 0
    }
  }

  minimize "TotalCost" {
    0
  }
}

scenario "Base" {
  use "Dispatch"
}
"#;

    let (_workspace, path) = write_temp_model(model)?;
    let error = ArcoOps::compile_file(&path)
        .expect_err("unresolved identifier in generated expression should fail compilation");
    let message = error.to_string();

    assert!(message.contains("missing declaration `parameter` named `unknown_load`"));

    Ok(())
}

#[test]
fn compiles_filtered_generated_expression_in_nonlinear_objective()
-> Result<(), Box<dyn std::error::Error>> {
    let model = r#"
data "bus_data" source="data/bus.csv" {
  set "bus"
  set "dispatchable_bus" {
    in "bus"
    filter { active == 1 }
  }
  param "active" index="bus"
}

model "Dispatch" {
  control "dispatch_new" lower=0 {
    index "b" in="dispatchable_bus"
  }

  expression "net_injection_by_bus" {
    index "b" in="bus"
    if { active[b] == 1 }
    expression {
      dispatch_new[b]
    }
  }

  minimize "TotalCost" {
    abs(sum(net_injection_by_bus[bb] for bb in bus))
  }
}

scenario "Base" {
  use "Dispatch"
}
"#;

    let (_workspace, path) =
        write_temp_workspace(model, &[("data/bus.csv", "bus,active\nb1,1\nb2,0\n")])?;
    let compiled = ArcoOps::compile_file(&path)?;

    assert!(compiled.compiled_problem.algebra.nonlinear.is_some());

    Ok(())
}
