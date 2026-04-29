from __future__ import annotations

from dataclasses import fields as dataclass_fields
from dataclasses import is_dataclass
import inspect
from typing import Callable, Mapping, TypeVar, overload

from .arco import *  # noqa: F403
from . import arco as _arco

__doc__ = _arco.__doc__
__all__ = list(getattr(_arco, "__all__", dir(_arco)))

_BlockFnT = TypeVar("_BlockFnT", bound=Callable[..., object])

_ARCO_BLOCK_MARKER_ATTR = "__arco_block_marker__"
_ARCO_BLOCK_NAME_ATTR = "__arco_block_name__"
_ARCO_BLOCK_INPUT_SCHEMA_ATTR = "__arco_block_input_schema__"
_ARCO_BLOCK_INPUT_FIELDS_ATTR = "__arco_block_input_fields__"
_ARCO_BLOCK_EXPECTS_CTX_ATTR = "__arco_block_expects_ctx__"

_MODEL_SCENARIOS: dict[int, dict[str, dict[str, object]]] = {}
_MODEL_SOLVE = _arco.Model.solve


_PydanticBaseModel: type[object] | None
try:
    from pydantic import BaseModel

    _PydanticBaseModel = BaseModel
except ModuleNotFoundError as exc:  # pragma: no cover - optional dependency
    if exc.name != "pydantic":
        raise
    _PydanticBaseModel = None


def _is_supported_schema_type(schema: object) -> bool:
    if not inspect.isclass(schema):
        return False
    if is_dataclass(schema):
        return True
    if _PydanticBaseModel is not None and issubclass(schema, _PydanticBaseModel):
        return True
    return False


def _schema_fields(schema: type[object]) -> dict[str, object]:
    if is_dataclass(schema):
        return {field.name: field.type for field in dataclass_fields(schema)}
    if _PydanticBaseModel is not None and issubclass(schema, _PydanticBaseModel):
        return {
            name: getattr(field, "annotation", object)
            for name, field in schema.model_fields.items()
        }
    raise TypeError(
        "block: input schema must be a dataclass or pydantic BaseModel type"
    )


def _decorate_block(*, func: _BlockFnT, name: str | None) -> _BlockFnT:
    if not callable(func):
        raise TypeError("block: expected a callable")

    signature = inspect.signature(func)
    params = list(signature.parameters.values())
    if len(params) not in (2, 3):
        raise TypeError("block: expected signature (model, data) or (model, data, ctx)")

    for param in params:
        if param.kind in (
            inspect.Parameter.VAR_POSITIONAL,
            inspect.Parameter.VAR_KEYWORD,
        ):
            raise TypeError("block: variadic *args/**kwargs are not supported")
        if param.kind is inspect.Parameter.KEYWORD_ONLY:
            raise TypeError("block: keyword-only parameters are not supported")

    data_annotation = params[1].annotation
    if data_annotation is inspect.Signature.empty:
        raise TypeError("block: data parameter must include a schema annotation")
    if not _is_supported_schema_type(data_annotation):
        raise TypeError(
            "block: input schema must be a dataclass or pydantic BaseModel type"
        )

    setattr(func, _ARCO_BLOCK_MARKER_ATTR, True)
    setattr(func, _ARCO_BLOCK_NAME_ATTR, name or func.__name__)
    setattr(func, _ARCO_BLOCK_INPUT_SCHEMA_ATTR, data_annotation)
    setattr(func, _ARCO_BLOCK_INPUT_FIELDS_ATTR, _schema_fields(data_annotation))
    setattr(func, _ARCO_BLOCK_EXPECTS_CTX_ATTR, len(params) == 3)
    return func


@overload
def block(func: _BlockFnT, *, name: str | None = None) -> _BlockFnT: ...


@overload
def block(*, name: str | None = None) -> Callable[[_BlockFnT], _BlockFnT]: ...


def block(
    func: _BlockFnT | None = None,
    *,
    name: str | None = None,
) -> _BlockFnT | Callable[[_BlockFnT], _BlockFnT]:
    if func is None:

        def decorator(inner: _BlockFnT) -> _BlockFnT:
            return _decorate_block(func=inner, name=name)

        return decorator
    return _decorate_block(func=func, name=name)


def _model_control(
    self: _arco.Model,
    name: str,
    *index_sets: _arco.IndexSet,
    bounds: _arco.Bounds | _arco.BoundType,
    is_integer: bool = False,
    is_binary: bool = False,
) -> _arco.Variable | _arco.VariableArray:
    if not isinstance(name, str) or not name.strip():
        raise ValueError("control: name must be a non-empty string")

    if index_sets:
        return self.add_variables(
            *index_sets,
            bounds=bounds,
            is_integer=is_integer,
            is_binary=is_binary,
            name=name,
        )

    return self.add_variable(
        bounds=bounds,
        is_integer=is_integer,
        is_binary=is_binary,
        name=name,
    )


def _model_scenario(
    self: _arco.Model,
    name: str,
    *,
    solver: _arco.Solver | None = None,
    log_to_console: bool | None = None,
    primal_start: list[tuple[int, float]] | None = None,
    time_limit: float | None = None,
    mip_gap: float | None = None,
    verbosity: int | None = None,
    solver_params: Mapping[str, bool | int | float | str] | None = None,
) -> None:
    if not isinstance(name, str) or not name.strip():
        raise ValueError("scenario: name must be a non-empty string")

    scenario_config = {
        "solver": solver,
        "log_to_console": log_to_console,
        "primal_start": primal_start,
        "time_limit": time_limit,
        "mip_gap": mip_gap,
        "verbosity": verbosity,
        "solver_params": solver_params,
    }
    _MODEL_SCENARIOS.setdefault(id(self), {})[name] = scenario_config


def _model_run_scenario(
    self: _arco.Model,
    name: str,
    *,
    solver: _arco.Solver | None = None,
    log_to_console: bool | None = None,
    primal_start: list[tuple[int, float]] | None = None,
    time_limit: float | None = None,
    mip_gap: float | None = None,
    verbosity: int | None = None,
    solver_params: Mapping[str, bool | int | float | str] | None = None,
) -> _arco.SolveResult:
    scenario_map = _MODEL_SCENARIOS.get(id(self), {})
    if name not in scenario_map:
        known = ", ".join(sorted(scenario_map))
        message = f"run_scenario: unknown scenario {name!r}"
        if known:
            message = f"{message}; known scenarios: {known}"
        raise KeyError(message)

    solve_kwargs = dict(scenario_map[name])
    overrides = {
        "solver": solver,
        "log_to_console": log_to_console,
        "primal_start": primal_start,
        "time_limit": time_limit,
        "mip_gap": mip_gap,
        "verbosity": verbosity,
        "solver_params": solver_params,
    }
    for key, value in overrides.items():
        if value is not None:
            solve_kwargs[key] = value

    return self.solve(**solve_kwargs)


def _model_solve(
    self: _arco.Model,
    *,
    solver: _arco.Solver | None = None,
    log_to_console: bool | None = None,
    primal_start: list[tuple[int, float]] | None = None,
    time_limit: float | None = None,
    mip_gap: float | None = None,
    verbosity: int | None = None,
    solver_params: Mapping[str, bool | int | float | str] | None = None,
    progress: Callable[[dict[str, object]], object] | None = None,
) -> _arco.SolveResult:
    if progress is not None and not callable(progress):
        raise TypeError("solve: progress must be a callable or None")

    effective_solver = solver
    if solver_params is not None:
        if not isinstance(solver_params, Mapping):
            raise TypeError("solve: solver_params must be a mapping or None")
        base_solver = solver if solver is not None else _arco.Solver()
        effective_solver = base_solver.copy(
            update={"solver_params": dict(solver_params)}
        )

    if progress is not None:
        progress(
            {
                "stage": "start",
                "num_variables": self.num_variables,
                "num_constraints": self.num_constraints,
            }
        )

    try:
        result = _MODEL_SOLVE(
            self,
            solver=effective_solver,
            log_to_console=log_to_console,
            primal_start=primal_start,
            time_limit=time_limit,
            mip_gap=mip_gap,
            verbosity=verbosity,
        )
    except Exception as exc:
        if progress is not None:
            progress(
                {
                    "stage": "error",
                    "error_type": type(exc).__name__,
                    "error": str(exc),
                }
            )
        raise

    if progress is not None:
        progress(
            {
                "stage": "done",
                "status": result.status_string(),
                "objective_value": result.objective_value,
                "solve_time_seconds": result.solve_time_seconds(),
            }
        )

    return result


def _solve_result_records(
    self: _arco.SolveResult,
    *,
    table: str = "variables",
) -> list[dict[str, object]]:
    if table == "variables":
        return [
            {
                "variable_id": idx,
                "value": primal,
                "reduced_cost": reduced_cost,
            }
            for idx, (primal, reduced_cost) in enumerate(
                zip(self.primal_values, self.variable_duals, strict=True)
            )
        ]

    if table == "constraints":
        return [
            {
                "constraint_id": idx,
                "dual": dual,
            }
            for idx, dual in enumerate(self.constraint_duals)
        ]

    if table == "summary":
        return [
            {
                "status": self.status_string(),
                "objective_value": self.objective_value,
                "solve_time_seconds": self.solve_time_seconds(),
                "is_optimal": self.is_optimal(),
            }
        ]

    raise ValueError("table must be one of {'variables', 'constraints', 'summary'}")


def _solve_result_to_pandas(
    self: _arco.SolveResult,
    *,
    table: str = "variables",
) -> object:
    try:
        import pandas as pd
    except ModuleNotFoundError as exc:  # pragma: no cover - optional dependency
        raise ModuleNotFoundError(
            "to_pandas() requires pandas. Install with `uv add pandas` or `pip install pandas`."
        ) from exc

    return pd.DataFrame.from_records(_solve_result_records(self, table=table))


def _solve_result_to_polars(
    self: _arco.SolveResult,
    *,
    table: str = "variables",
) -> object:
    try:
        import polars as pl
    except ModuleNotFoundError as exc:  # pragma: no cover - optional dependency
        raise ModuleNotFoundError(
            "to_polars() requires polars. Install with `uv add polars` or `pip install polars`."
        ) from exc

    return pl.DataFrame(_solve_result_records(self, table=table))


setattr(_arco.Model, "control", _model_control)
setattr(_arco.Model, "scenario", _model_scenario)
setattr(_arco.Model, "run_scenario", _model_run_scenario)
setattr(_arco.Model, "solve", _model_solve)
setattr(_arco.SolveResult, "to_pandas", _solve_result_to_pandas)
setattr(_arco.SolveResult, "to_polars", _solve_result_to_polars)


if "block" not in __all__:
    __all__.append("block")
