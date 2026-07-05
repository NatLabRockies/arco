from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess


ROOT = Path(__file__).resolve().parents[3]
SCIP_LIBRARY_PATTERNS = (
    "*/out/scip_install/lib/libscip.so*",
    "*/out/scip_install/lib/libscip*.dylib",
)
LOADER_PATH_ENV_VARS = ("LD_LIBRARY_PATH", "DYLD_LIBRARY_PATH")


def _prepend_env_paths(env: dict[str, str], name: str, paths: list[Path]) -> None:
    if not paths:
        return
    prefix = os.pathsep.join(str(path) for path in paths)
    current = env.get(name)
    env[name] = f"{prefix}{os.pathsep}{current}" if current else prefix


def _bundled_scip_library_dirs(cli_bin: Path) -> list[Path]:
    build_dir = cli_bin.parent / "build"
    if not build_dir.is_dir():
        return []

    library_dirs: set[Path] = set()
    for pattern in SCIP_LIBRARY_PATTERNS:
        library_dirs.update(path.parent for path in build_dir.glob(pattern))
    return sorted(library_dirs)


def _add_cli_runtime_library_paths(env: dict[str, str], cli_bin: Path) -> None:
    library_dirs = _bundled_scip_library_dirs(cli_bin)
    for name in LOADER_PATH_ENV_VARS:
        _prepend_env_paths(env, name, library_dirs)


def _run_arco_cli(args: list[str], *, config_dir: Path) -> dict[str, object]:
    env = os.environ.copy()
    env["ARCO_CONFIG_DIR"] = str(config_dir)
    env["ARCO_PROJECT_CONFIG_DIR"] = str(config_dir)
    cli_bin = env.get("ARCO_CLI_BIN")
    if cli_bin:
        _add_cli_runtime_library_paths(env, Path(cli_bin))
    command = (
        [cli_bin, *args]
        if cli_bin
        else [
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
        ]
    )
    completed = subprocess.run(
        command,
        cwd=ROOT,
        env=env,
        check=True,
        capture_output=True,
        text=True,
        timeout=180,
    )
    return json.loads(completed.stdout)


def test_cli_runtime_env_includes_bundled_scip_library_dir(tmp_path: Path) -> None:
    cli_bin = tmp_path / "target" / "debug" / "arco"
    library_dir = (
        cli_bin.parent / "build" / "scip-sys-abcd" / "out" / "scip_install" / "lib"
    )
    library_dir.mkdir(parents=True)
    (library_dir / "libscip.so.10.0").touch()

    env = {"LD_LIBRARY_PATH": "/existing/lib"}
    _add_cli_runtime_library_paths(env, cli_bin)

    assert env["LD_LIBRARY_PATH"].split(os.pathsep)[:2] == [
        str(library_dir),
        "/existing/lib",
    ]
    assert env["DYLD_LIBRARY_PATH"] == str(library_dir)


def test_scalar_python_and_kdl_api_contract_models_have_equivalent_snapshot(
    tmp_path: Path,
) -> None:
    import arco

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


def test_indexed_python_and_kdl_api_contract_models_have_equivalent_snapshot(
    tmp_path: Path,
) -> None:
    import arco
    import numpy as np

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
