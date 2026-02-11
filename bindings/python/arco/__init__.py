from __future__ import annotations

from dataclasses import fields as dataclass_fields
from dataclasses import is_dataclass
import inspect
from typing import Any, Callable, TypeVar, overload

from .arco import *  # noqa: F401,F403
from . import arco as _arco

__doc__ = _arco.__doc__
__all__ = list(getattr(_arco, "__all__", []))

_BlockFnT = TypeVar("_BlockFnT", bound=Callable[..., Any])

_ARCO_BLOCK_MARKER_ATTR = "__arco_block_marker__"
_ARCO_BLOCK_NAME_ATTR = "__arco_block_name__"
_ARCO_BLOCK_INPUT_SCHEMA_ATTR = "__arco_block_input_schema__"
_ARCO_BLOCK_INPUT_FIELDS_ATTR = "__arco_block_input_fields__"
_ARCO_BLOCK_EXPECTS_CTX_ATTR = "__arco_block_expects_ctx__"


try:
    from pydantic import BaseModel as _PydanticBaseModel
except Exception:  # pragma: no cover - optional dependency
    _PydanticBaseModel = None


def _is_supported_schema_type(schema: Any) -> bool:
    if not inspect.isclass(schema):
        return False
    if is_dataclass(schema):
        return True
    if _PydanticBaseModel is not None and issubclass(schema, _PydanticBaseModel):
        return True
    return False


def _schema_fields(schema: Any) -> dict[str, Any]:
    if is_dataclass(schema):
        return {field.name: field.type for field in dataclass_fields(schema)}
    if _PydanticBaseModel is not None and issubclass(schema, _PydanticBaseModel):
        return {
            name: getattr(field, "annotation", Any)
            for name, field in schema.model_fields.items()
        }
    raise TypeError("block: input schema must be a dataclass or pydantic BaseModel type")


def _decorate_block(*, func: _BlockFnT, name: str | None) -> _BlockFnT:
    if not callable(func):
        raise TypeError("block: expected a callable")

    signature = inspect.signature(func)
    params = list(signature.parameters.values())
    if len(params) not in (2, 3):
        raise TypeError("block: expected signature (model, data) or (model, data, ctx)")

    for param in params:
        if param.kind in (inspect.Parameter.VAR_POSITIONAL, inspect.Parameter.VAR_KEYWORD):
            raise TypeError("block: variadic *args/**kwargs are not supported")
        if param.kind is inspect.Parameter.KEYWORD_ONLY:
            raise TypeError("block: keyword-only parameters are not supported")

    data_annotation = params[1].annotation
    if data_annotation is inspect.Signature.empty:
        raise TypeError("block: data parameter must include a schema annotation")
    if not _is_supported_schema_type(data_annotation):
        raise TypeError("block: input schema must be a dataclass or pydantic BaseModel type")

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


if "block" not in __all__:
    __all__.append("block")
