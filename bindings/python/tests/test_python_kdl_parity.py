from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess

import numpy as np

import arco


ROOT = Path(__file__).resolve().parents[3]


def _run_arco_cli(args: list[str], *, config_dir: Path) -> dict[str, object]:
    env = os.environ.copy()
    env["ARCO_CONFIG_DIR"] = str(config_dir)
    env["ARCO_PROJECT_CONFIG_DIR"] = str(config_dir)
    completed = subprocess.run(
        [
            "cargo",
            "run",
            "--quiet",
            "-p",
            "arco-cli",
            "--no-default-features",
            "--bin",
            "arco",
            "--",
            *args,
        ],
        cwd=ROOT,
        env=env,
        check=True,
        capture_output=True,
        text=True,
        timeout=180,
    )
    return json.loads(completed.stdout)


def test_scalar_python_and_kdl_ladder_models_have_equivalent_snapshot(
    tmp_path: Path,
) -> None:
    model = arco.Model()
    x = model.add_variable(
        bounds=arco.Bounds(lower=1.0, upper=float("inf")),
        name="x",
    )
    y = model.add_variable(
        bounds=arco.Bounds(lower=2.0, upper=float("inf")),
        name="y",
    )
    model.add_constraint(x + y >= 5.0, name="demand")
    model.minimize(3.0 * x + 2.0 * y, name="total_cost")

    python_snapshot = model.inspect(include_coeffs=True)
    python_result = model.solve(log_to_console=False)

    kdl_path = tmp_path / "scalar.kdl"
    kdl_path.write_text(
        """
model first_model {
  control x lower=1
  control y lower=2

  constraint demand {
    expression { x + y >= 5 }
  }

  minimize total_cost { (3 * x) + (2 * y) }
}
""".strip()
    )
    config_dir = tmp_path / "config"
    config_dir.mkdir()

    inspect_payload = _run_arco_cli(
        ["inspect", str(kdl_path), "--json"],
        config_dir=config_dir,
    )
    run_payload = _run_arco_cli(
        ["run", str(kdl_path), "--compact"],
        config_dir=config_dir,
    )

    assert (
        python_snapshot.metadata.variables
        == inspect_payload["meta"]["counts"]["variable_instances"]
    )
    assert (
        python_snapshot.metadata.constraints
        == inspect_payload["meta"]["counts"]["constraint_instances"]
    )
    assert (
        python_snapshot.metadata.coefficients
        == inspect_payload["meta"]["counts"]["coefficient_instances"]
    )
    assert (
        python_snapshot.metadata.memory.coefficient_value_bytes
        == inspect_payload["meta"]["memory"]["coefficient_value_bytes"]
    )
    assert python_result.is_optimal()
    assert run_payload["solve_status"] == "optimal"
    assert run_payload["objective"]["name"] == "total_cost"
    assert round(python_result.objective_value, 6) == round(
        float(run_payload["objective"]["value"]),
        6,
    )


def test_indexed_python_and_kdl_ladder_models_have_equivalent_snapshot(
    tmp_path: Path,
) -> None:
    model = arco.Model()
    plant = arco.IndexSet(name="plant", members=["north", "south"])
    demand = arco.param(np.array([3.0, 5.0]), axes=(plant,), name="demand")
    cost = arco.param(np.array([2.0, 1.0]), axes=(plant,), name="cost")
    output = model.add_variables(
        axes=(plant,),
        bounds=arco.NonNegativeFloat,
        name="output",
    )
    model.add_constraints(output >= demand, name="meet_demand")
    model.minimize((cost * output).sum(), name="total_cost")

    python_snapshot = model.inspect(include_coeffs=True)
    python_result = model.solve(log_to_console=False)

    data_dir = tmp_path / "data"
    data_dir.mkdir()
    (data_dir / "plants.csv").write_text("plant,demand,cost\nnorth,3,2\nsouth,5,1\n")
    kdl_path = tmp_path / "indexed.kdl"
    kdl_path.write_text(
        """
data plant_data source="data/plants.csv" {
  set plant

  param demand {
    index plant
  }

  param cost {
    index plant
  }
}

model indexed_allocation {
  control output lower=0 {
    index plant
  }

  constraint meet_demand {
    index p { in plant }
    expression { output[p] >= demand[p] }
  }

  minimize total_cost {
    sum(cost[p] * output[p] for p in plant)
  }
}

scenario indexed_case { use indexed_allocation }
""".strip()
    )
    config_dir = tmp_path / "config"
    config_dir.mkdir()

    inspect_payload = _run_arco_cli(
        ["inspect", str(kdl_path), "--json"],
        config_dir=config_dir,
    )
    run_payload = _run_arco_cli(
        ["run", str(kdl_path), "--compact"],
        config_dir=config_dir,
    )

    assert (
        python_snapshot.metadata.variables
        == inspect_payload["meta"]["counts"]["variable_instances"]
    )
    assert (
        python_snapshot.metadata.constraints
        == inspect_payload["meta"]["counts"]["constraint_instances"]
    )
    assert (
        python_snapshot.metadata.coefficients
        == inspect_payload["meta"]["counts"]["coefficient_instances"]
    )
    assert (
        python_snapshot.metadata.memory.coefficient_value_bytes
        == inspect_payload["meta"]["memory"]["coefficient_value_bytes"]
    )
    assert python_result.is_optimal()
    assert run_payload["solve_status"] == "optimal"
    assert run_payload["objective"]["name"] == "total_cost"
    assert round(python_result.objective_value, 6) == round(
        float(run_payload["objective"]["value"]),
        6,
    )
