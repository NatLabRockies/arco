from dataclasses import dataclass
import json
from pathlib import Path

import pytest

import arco
from arco import block


@dataclass(slots=True)
class SupplyIn:
    capacity: float


@dataclass(slots=True)
class SupplyOut:
    level: float


@dataclass(slots=True)
class DemandIn:
    supply_level: float


@dataclass(slots=True)
class DemandOut:
    level: float


@dataclass(slots=True)
class BadDemandIn:
    supply_level: str


@block
def build_supply(model: arco.Model, data: SupplyIn, ctx: dict[str, object]) -> None:
    x = model.add_variable(
        bounds=arco.Bounds(lower=0.0, upper=data.capacity),
        name="supply",
    )
    ctx["level"] = x
    model.minimize(x)


def extract_supply(
    result: arco.SolveResult, data: SupplyIn, ctx: dict[str, object]
) -> SupplyOut:
    return SupplyOut(level=float(result.value(ctx["level"])))


@block
def build_demand(model: arco.Model, data: DemandIn, ctx: dict[str, object]) -> None:
    y = model.add_variable(
        bounds=arco.Bounds(lower=data.supply_level, upper=100.0),
        name="demand",
    )
    ctx["level"] = y
    model.minimize(y)


def extract_demand(
    result: arco.SolveResult, data: DemandIn, ctx: dict[str, object]
) -> DemandOut:
    return DemandOut(level=float(result.value(ctx["level"])))


@block
def build_alt_supply(model: arco.Model, data: SupplyIn, ctx: dict[str, object]) -> None:
    x = model.add_variable(
        bounds=arco.Bounds(lower=0.0, upper=data.capacity),
        name="supply_alt",
    )
    ctx["level"] = x
    model.minimize(2.0 * x)


def test_composed_blocks_use_typed_outputs_and_value_extraction(tmp_path: Path) -> None:
    model = arco.Model()
    supply = model.add_block(
        build_supply,
        data=SupplyIn(capacity=50.0),
        extract=extract_supply,
    )
    demand = model.add_block(build_demand, extract=extract_demand)

    model.link(supply.out.level, demand.in_.supply_level)
    result = model.solve(log_to_console=False)

    assert result.is_optimal()
    assert result.blocks is not None
    assert len(result.blocks) == 2
    assert "build_supply" in result.blocks
    assert "missing" not in result.blocks
    assert result.blocks.keys() == ["build_supply", "build_demand"]
    assert list(result.blocks) == result.blocks.keys()
    assert result.blocks.get("build_supply") is result.blocks["build_supply"]
    assert result.blocks.get("missing") is None
    assert result.blocks.get("missing", "fallback") == "fallback"
    with pytest.raises(arco.BlockResultError) as exc:
        _ = result.blocks["missing"]
    assert exc.value.code == arco.diagnostic_codes()["BLOCK_RESULT"]
    assert arco.error_code(exc.value) == arco.diagnostic_codes()["BLOCK_RESULT"]
    assert result.blocks.statuses() == {
        "build_supply": "OPTIMAL",
        "build_demand": "OPTIMAL",
    }
    assert result.blocks.report() == [
        {
            "order": 0,
            "name": "build_supply",
            "status": "OPTIMAL",
            "objective_value": 0.0,
            "variable_count": 1,
            "constraint_count": 0,
        },
        {
            "order": 1,
            "name": "build_demand",
            "status": "OPTIMAL",
            "objective_value": 0.0,
            "variable_count": 1,
            "constraint_count": 0,
        },
    ]
    assert json.loads(result.blocks.report_json()) == result.blocks.report()
    assert result.blocks.diagnostics() == [
        {
            "order": 0,
            "name": "build_supply",
            "status": "OPTIMAL",
            "objective_value": 0.0,
            "result": {
                "variable_count": 1,
                "constraint_count": 0,
            },
            "model": {
                "variables": 1,
                "constraints": 0,
                "coefficients": 0,
                "memory": {
                    "coefficient_value_bytes": 0,
                    "coefficient_index_bytes": 0,
                    "variable_column_pointer_bytes": 16,
                    "sparse_matrix_bytes": 16,
                },
            },
        },
        {
            "order": 1,
            "name": "build_demand",
            "status": "OPTIMAL",
            "objective_value": 0.0,
            "result": {
                "variable_count": 1,
                "constraint_count": 0,
            },
            "model": {
                "variables": 1,
                "constraints": 0,
                "coefficients": 0,
                "memory": {
                    "coefficient_value_bytes": 0,
                    "coefficient_index_bytes": 0,
                    "variable_column_pointer_bytes": 16,
                    "sparse_matrix_bytes": 16,
                },
            },
        },
    ]
    assert json.loads(result.blocks.diagnostics_json()) == result.blocks.diagnostics()
    assert result.blocks.artifact_manifest() == [
        {
            "order": 0,
            "name": "build_supply",
            "artifacts": ["stage_diagnostics", "solution_summary"],
        },
        {
            "order": 1,
            "name": "build_demand",
            "artifacts": ["stage_diagnostics", "solution_summary"],
        },
    ]
    assert result.blocks.artifact_manifest(policy="model") == [
        {
            "order": 0,
            "name": "build_supply",
            "artifacts": ["stage_diagnostics", "model_snapshot", "solution_summary"],
        },
        {
            "order": 1,
            "name": "build_demand",
            "artifacts": ["stage_diagnostics", "model_snapshot", "solution_summary"],
        },
    ]
    assert json.loads(result.blocks.artifact_manifest_json(policy="none")) == [
        {
            "order": 0,
            "name": "build_supply",
            "artifacts": [],
        },
        {
            "order": 1,
            "name": "build_demand",
            "artifacts": [],
        },
    ]
    artifact_rows = result.blocks.write_artifacts(
        tmp_path / "artifacts", policy="model"
    )
    assert artifact_rows == [
        {
            "order": 0,
            "name": "build_supply",
            "files": [
                {
                    "artifact": "stage_diagnostics",
                    "path": "000-build_supply/stage_diagnostics.json",
                },
                {
                    "artifact": "model_snapshot",
                    "path": "000-build_supply/model_snapshot.json",
                },
                {
                    "artifact": "solution_summary",
                    "path": "000-build_supply/solution_summary.json",
                },
            ],
        },
        {
            "order": 1,
            "name": "build_demand",
            "files": [
                {
                    "artifact": "stage_diagnostics",
                    "path": "001-build_demand/stage_diagnostics.json",
                },
                {
                    "artifact": "model_snapshot",
                    "path": "001-build_demand/model_snapshot.json",
                },
                {
                    "artifact": "solution_summary",
                    "path": "001-build_demand/solution_summary.json",
                },
            ],
        },
    ]
    manifest = json.loads((tmp_path / "artifacts" / "manifest.json").read_text())
    assert manifest == {"policy": "model", "blocks": artifact_rows}
    supply_diagnostics = json.loads(
        (
            tmp_path / "artifacts" / "000-build_supply" / "stage_diagnostics.json"
        ).read_text()
    )
    assert supply_diagnostics == result.blocks.diagnostics()[0]
    supply_summary = json.loads(
        (
            tmp_path / "artifacts" / "000-build_supply" / "solution_summary.json"
        ).read_text()
    )
    assert supply_summary["status"] == "OPTIMAL"
    assert supply_summary["objective_value"] == pytest.approx(0.0)
    assert supply_summary["variable_count"] == 1
    supply_snapshot = json.loads(
        (
            tmp_path / "artifacts" / "000-build_supply" / "model_snapshot.json"
        ).read_text()
    )
    assert supply_snapshot["metadata"]["variables"] == 1
    assert supply_snapshot["metadata"]["memory"]["sparse_matrix_bytes"] == 16
    assert supply_snapshot["variables"] == [
        {
            "id": 0,
            "name": "supply",
            "is_integer": False,
            "is_active": True,
        }
    ]
    assert result.blocks["build_supply"].objective_value == pytest.approx(0.0)
    assert result.blocks["build_demand"].objective_value == pytest.approx(0.0)


def test_block_artifact_policy_errors_are_typed() -> None:
    model = arco.Model()
    model.add_block(
        build_supply,
        data=SupplyIn(capacity=50.0),
        extract=extract_supply,
    )
    result = model.solve(log_to_console=False)
    assert result.blocks is not None
    codes = arco.diagnostic_codes()

    with pytest.raises(arco.BlockContractError) as exc:
        result.blocks.artifact_manifest(policy="forever")

    assert exc.value.code == codes["BLOCK_CONTRACT"]
    assert arco.error_code(exc.value) == codes["BLOCK_CONTRACT"]
    assert "unknown block artifact policy" in str(exc.value)


def test_block_artifact_writer_io_errors_are_typed(tmp_path: Path) -> None:
    model = arco.Model()
    model.add_block(
        build_supply,
        data=SupplyIn(capacity=50.0),
        extract=extract_supply,
    )
    result = model.solve(log_to_console=False)
    assert result.blocks is not None
    codes = arco.diagnostic_codes()
    target = tmp_path / "artifact-target"
    target.write_text("not a directory")

    with pytest.raises(arco.BlockArtifactError) as exc:
        result.blocks.write_artifacts(target)

    assert exc.value.code == codes["BLOCK_ARTIFACT_IO"]
    assert arco.error_code(exc.value) == codes["BLOCK_ARTIFACT_IO"]
    assert "failed to create block artifact directory" in str(exc.value)


def test_unknown_block_ports_are_contract_errors() -> None:
    model = arco.Model()
    supply = model.add_block(
        build_supply,
        data=SupplyIn(capacity=50.0),
        extract=extract_supply,
    )
    codes = arco.diagnostic_codes()

    port_lookups = [
        lambda: supply.input("missing"),
        lambda: supply.output("missing"),
        lambda: supply.in_.missing,
        lambda: supply.out.missing,
    ]

    for lookup in port_lookups:
        with pytest.raises(arco.BlockContractError) as exc:
            lookup()

        assert exc.value.code == codes["BLOCK_CONTRACT"]
        assert arco.error_code(exc.value) == codes["BLOCK_CONTRACT"]
        assert "Unknown" in str(exc.value)
        assert "missing" in str(exc.value)


def test_cyclic_block_links_are_contract_errors() -> None:
    model = arco.Model()
    supply = model.add_block(
        build_supply,
        data=SupplyIn(capacity=50.0),
        extract=extract_supply,
    )
    demand = model.add_block(build_demand, extract=extract_demand)
    codes = arco.diagnostic_codes()

    model.link(supply.out.level, demand.in_.supply_level)
    model.link(demand.out.level, supply.in_.capacity)

    with pytest.raises(arco.BlockContractError) as exc:
        model.solve(log_to_console=False)

    assert exc.value.code == codes["BLOCK_CONTRACT"]
    assert arco.error_code(exc.value) == codes["BLOCK_CONTRACT"]
    assert "Cycle detected" in str(exc.value)


def test_swappable_block_builders_preserve_same_ports() -> None:
    model = arco.Model()
    original = model.add_block(
        build_supply,
        name="supply_original",
        data=SupplyIn(capacity=50.0),
        extract=extract_supply,
    )
    replacement = model.add_block(
        build_alt_supply,
        name="supply_replacement",
        data=SupplyIn(capacity=50.0),
        extract=extract_supply,
    )

    assert original.in_.keys() == replacement.in_.keys() == ["capacity"]
    assert original.out.keys() == replacement.out.keys() == ["level"]


def test_composed_solve_rejects_primal_start_with_solver_setting_code() -> None:
    model = arco.Model()
    model.add_block(
        build_supply,
        data=SupplyIn(capacity=50.0),
        extract=extract_supply,
    )
    codes = arco.diagnostic_codes()

    with pytest.raises(arco.SolverInvalidSettingError) as exc:
        model.solve(primal_start=[(0, 0.0)], log_to_console=False)

    assert exc.value.code == codes["SOLVER_INVALID_SETTING"]
    assert arco.error_code(exc.value) == codes["SOLVER_INVALID_SETTING"]
    assert "primal_start is not supported" in str(exc.value)


def test_composed_solve_accepts_empty_primal_start_hint() -> None:
    model = arco.Model()
    model.add_block(
        build_supply,
        data=SupplyIn(capacity=50.0),
        extract=extract_supply,
    )

    result = model.solve(primal_start=[], log_to_console=False)

    assert result.is_optimal()


def test_link_rejects_mismatched_schema_fields() -> None:
    @block
    def build_bad_demand(
        model: arco.Model,
        data: BadDemandIn,
        ctx: dict[str, object],
    ) -> None:
        y = model.add_variable(bounds=arco.NonNegativeFloat, name="bad_demand")
        ctx["level"] = y
        model.minimize(y)

    def extract_bad_demand(
        result: arco.SolveResult,
        data: BadDemandIn,
        ctx: dict[str, object],
    ) -> DemandOut:
        return DemandOut(level=float(result.value(ctx["level"])))

    model = arco.Model()
    supply = model.add_block(
        build_supply,
        data=SupplyIn(capacity=50.0),
        extract=extract_supply,
    )
    demand = model.add_block(build_bad_demand, extract=extract_bad_demand)

    codes = arco.diagnostic_codes()

    with pytest.raises(arco.BlockContractError, match="type mismatch") as exc:
        model.link(supply.out.level, demand.in_.supply_level)

    assert exc.value.code == codes["BLOCK_CONTRACT"]
    assert arco.error_code(exc.value) == codes["BLOCK_CONTRACT"]


def test_block_decorator_contract_errors_are_typed() -> None:
    codes = arco.diagnostic_codes()

    with pytest.raises(arco.BlockContractError) as exc:

        @block
        def missing_data_annotation(model: arco.Model, data) -> None:  # type: ignore[no-untyped-def]
            model.minimize(0.0)

    assert exc.value.code == codes["BLOCK_CONTRACT"]
    assert arco.error_code(exc.value) == codes["BLOCK_CONTRACT"]
    assert "data parameter must include a schema annotation" in str(exc.value)


def test_block_build_return_contract_errors_are_typed() -> None:
    @block
    def bad_build(model: arco.Model, data: SupplyIn) -> str:
        model.minimize(0.0)
        return "bad"

    def extract(result: arco.SolveResult, data: SupplyIn) -> SupplyOut:
        return SupplyOut(level=result.objective_value)

    model = arco.Model()
    model.add_block(bad_build, data=SupplyIn(capacity=1.0), extract=extract)
    codes = arco.diagnostic_codes()

    with pytest.raises(arco.BlockContractError) as exc:
        model.solve(log_to_console=False)

    assert exc.value.code == codes["BLOCK_CONTRACT"]
    assert arco.error_code(exc.value) == codes["BLOCK_CONTRACT"]
    assert "block build function must return None" in str(exc.value)


def test_add_block_requires_decorated_function_with_typed_error() -> None:
    def plain_build(model: arco.Model, data: SupplyIn) -> None:
        model.minimize(0.0)

    def plain_extract(result: arco.SolveResult, data: SupplyIn) -> SupplyOut:
        return SupplyOut(level=result.objective_value)

    model = arco.Model()
    codes = arco.diagnostic_codes()

    with pytest.raises(arco.BlockContractError) as exc:
        model.add_block(
            plain_build,
            data=SupplyIn(capacity=1.0),
            extract=plain_extract,
        )

    assert exc.value.code == codes["BLOCK_CONTRACT"]
    assert arco.error_code(exc.value) == codes["BLOCK_CONTRACT"]
    assert "must be decorated with @arco.block" in str(exc.value)
