#![allow(clippy::float_cmp)]

use arco_kdl::compile::{CompileError, compile_program};
use arco_kdl::semantic::validate_program;
use arco_kdl::source::parse_program_file;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_test_dir(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_nanos()
        .to_string();
    let root = std::env::temp_dir().join(format!("arco-kdl-lowering-{name}-{unique}"));
    fs::create_dir_all(&root)?;
    Ok(root)
}

#[test]
fn lowering_loads_top_level_data_block_params() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("top-level-data")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("inputs.csv"),
        "time,cap,demand\n1,10,10\n2,20,5\n",
    )?;

    let path = root.join("input.kdl");
    fs::write(
        &path,
        r#"
set "time" { "1"; "2" }

data "inputs" from="data/inputs.csv" {
  map "time" from="time"

  param "capacity" index="time" from="cap" reduce="sum"
  param "demand" index="time" from="demand"
}

model "Dispatch" {
  set time alias="t"

  param "capacity" {
    index "t"
  }
  param "demand" {
    index "t"
  }

  control "x" {
    index "t"
  }

  constraint "cap_limit[t]" {
    x[t] <= capacity[t]
  }

  constraint "balance[t]" {
    x[t] = demand[t]
  }

  minimize "Obj" {
    sum(x[t] for t in time)
  }
}

scenario "S1" {
  use "Dispatch"
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic = validate_program(&parsed.program, &path)?;
    let compiled = compile_program(&semantic, &parsed.program, &path)?;

    let cap_1 = compiled
        .algebra
        .constraints
        .iter()
        .find(|c| c.name == "cap_limit[t][1]")
        .ok_or("missing cap constraint at t=1")?;
    let bal_1 = compiled
        .algebra
        .constraints
        .iter()
        .find(|c| c.name == "balance[t][1]")
        .ok_or("missing balance constraint at t=1")?;

    assert_eq!(cap_1.rhs, 10.0);
    assert_eq!(bal_1.rhs, 10.0);

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn lowering_prefers_scenario_data_bindings_over_top_level_data_params()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("scenario-override")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(root.join("data").join("top.csv"), "time,cap\n1,10\n")?;
    fs::write(root.join("data").join("override.csv"), "t,capacity\n1,55\n")?;

    let path = root.join("input.kdl");
    fs::write(
        &path,
        r#"
set "time" { "1" }

data "defaults" from="data/top.csv" {
  map "time" from="time"
  param "capacity" index="time" from="cap"
}

model "Dispatch" {
  set time alias="t"

  param "capacity" {
    index "t"
  }

  control "x" {
    index "t"
  }

  constraint "cap_limit[t]" {
    x[t] <= capacity[t]
  }

  minimize "Obj" {
    sum(x[t] for t in time)
  }
}

scenario "S1" {
  use "Dispatch"
  data "capacity" from="data/override.csv"
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic = validate_program(&parsed.program, &path)?;
    let compiled = compile_program(&semantic, &parsed.program, &path)?;

    let cap = compiled
        .algebra
        .constraints
        .iter()
        .find(|c| c.name == "cap_limit[t][1]")
        .ok_or("missing capacity constraint")?;
    assert_eq!(cap.rhs, 55.0);

    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn lowering_reports_missing_data_point_for_sparse_generic_data_table()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_test_dir("sparse-generic-data")?;
    fs::create_dir_all(root.join("data"))?;
    fs::write(
        root.join("data").join("dist.csv"),
        "g,b,distance_km\ng1,b1,10\ng1,b2,20\ng2,b1,30\n",
    )?;

    let path = root.join("input.kdl");
    fs::write(
        &path,
        r#"
set "time" { "1" }

data "distance" from="data/dist.csv" {
  set "g"
  set "b"
  param "distance_km" {
    index "g"
    index "b"
  }
}

model "SparseDistance" {
  set "g"
  set "b"

  param "distance_km" {
    index "g"
    index "b"
  }

  control "flow" lower=0 {
    index "g"
    index "b"
  }

  minimize "TotalCost" {
    sum(distance_km[g,b] * flow[g,b] for g in g for b in b)
  }
}

scenario "SparseDistanceCase" {
  use "SparseDistance"
}
"#,
    )?;

    let parsed = parse_program_file(&path)?;
    let semantic = validate_program(&parsed.program, &path)?;
    let err = compile_program(&semantic, &parsed.program, &path)
        .expect_err("sparse data table should fail lowering with a missing key");

    match err {
        CompileError::MissingDataPoint { name, key, .. } => {
            assert_eq!(name, "distance_km");
            assert_eq!(key, "g2,b2");
        }
        other => panic!("expected MissingDataPoint, got {other:?}"),
    }

    fs::remove_dir_all(&root)?;
    Ok(())
}
