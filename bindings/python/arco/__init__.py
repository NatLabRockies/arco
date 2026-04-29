from __future__ import annotations

from dataclasses import fields as dataclass_fields
from dataclasses import is_dataclass
import inspect
from typing import Callable, TypeVar, overload

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
    }
    for key, value in overrides.items():
        if value is not None:
            solve_kwargs[key] = value

    return self.solve(**solve_kwargs)


setattr(_arco.Model, "control", _model_control)
setattr(_arco.Model, "scenario", _model_scenario)
setattr(_arco.Model, "run_scenario", _model_run_scenario)


if "block" not in __all__:
    __all__.append("block")
