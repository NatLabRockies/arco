from __future__ import annotations

from dataclasses import dataclass
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


@dataclass(frozen=True)
class ParamArray:
    _values: object
    _axes: tuple[object, ...]
    _name: str | None = None

    @property
    def axes(self) -> tuple[object, ...]:
        return self._axes

    @property
    def shape(self) -> tuple[int, ...]:
        shape = getattr(self._values, "shape", None)
        if shape is None:
            return ()
        return tuple(int(v) for v in shape)

    @property
    def values(self) -> object:
        return self._values


def param(values: object, *axes: object, name: str | None = None) -> ParamArray:
    try:
        import numpy as np
    except ModuleNotFoundError as exc:  # pragma: no cover
        raise RuntimeError("arco.param requires numpy") from exc

    np_values = np.asarray(values)

    if np_values.ndim != len(axes):
        raise _arco.ArrayDimensionError(
            f"values.ndim ({np_values.ndim}) must equal len(axes) ({len(axes)})"
        )

    for idx, (axis, dim_size) in enumerate(zip(axes, np_values.shape, strict=True)):
        if not isinstance(axis, _arco.IndexSet):
            raise _arco.ArrayTypeError(
                f"axis {idx} must be IndexSet, got {type(axis).__name__}"
            )
        if axis.size != dim_size:
            raise _arco.ArrayShapeMismatchError(
                f"axis {axis.name!r} size ({axis.size}) does not match dimension size ({dim_size})"
            )

    return ParamArray(_values=np_values, _axes=tuple(axes), _name=name)


if "block" not in __all__:
    __all__.append("block")
if "ParamArray" not in __all__:
    __all__.append("ParamArray")
if "param" not in __all__:
    __all__.append("param")
