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
    __array_priority__ = 1000

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

    def __array__(self) -> object:
        return self._values

    def __getitem__(self, index: object) -> object:
        import numpy as np

        values = np.asarray(self._values)[index]
        axes = _slice_axes(self._axes, index)
        if getattr(values, "ndim", 0) == 0:
            return values.item()
        return ParamArray(_values=values, _axes=axes, _name=self._name)

    def _binary_param(
        self, other: object, op: Callable[[object, object], object]
    ) -> object:
        import numpy as np

        if isinstance(other, ParamArray):
            axes = _union_axes(self._axes, other._axes)
            left = _align_values(np.asarray(self._values), self._axes, axes)
            right = _align_values(np.asarray(other._values), other._axes, axes)
            return ParamArray(_values=op(left, right), _axes=axes, _name=self._name)
        if isinstance(other, (_arco.VariableArray, _arco.ExprArray)):
            return NotImplemented

        result = op(np.asarray(self._values), np.asarray(other))
        if getattr(result, "ndim", 0) == 0:
            return result.item()
        if result.shape != self.shape:
            raise _arco.ArrayShapeMismatchError(
                "raw NumPy operands must preserve the labeled parameter shape"
            )
        return ParamArray(_values=result, _axes=self._axes, _name=self._name)

    def __add__(self, other: object) -> object:
        return self._binary_param(other, lambda left, right: left + right)

    def __radd__(self, other: object) -> object:
        return self.__add__(other)

    def __sub__(self, other: object) -> object:
        return self._binary_param(other, lambda left, right: left - right)

    def __rsub__(self, other: object) -> object:
        return self._binary_param(other, lambda left, right: right - left)

    def __mul__(self, other: object) -> object:
        return self._binary_param(other, lambda left, right: left * right)

    def __rmul__(self, other: object) -> object:
        return self.__mul__(other)

    def __truediv__(self, other: object) -> object:
        return self._binary_param(other, lambda left, right: left / right)

    def __rtruediv__(self, other: object) -> object:
        return self._binary_param(other, lambda left, right: right / left)

    def __and__(self, other: object) -> object:
        return self._binary_param(other, lambda left, right: left & right)

    def __rand__(self, other: object) -> object:
        return self.__and__(other)

    def __or__(self, other: object) -> object:
        return self._binary_param(other, lambda left, right: left | right)

    def __ror__(self, other: object) -> object:
        return self.__or__(other)

    def __invert__(self) -> ParamArray:
        import numpy as np

        return ParamArray(
            _values=~np.asarray(self._values), _axes=self._axes, _name=self._name
        )

    def __neg__(self) -> ParamArray:
        import numpy as np

        return ParamArray(
            _values=-np.asarray(self._values), _axes=self._axes, _name=self._name
        )

    def __ge__(self, other: object) -> object:
        return self._binary_param(other, lambda left, right: left >= right)

    def __gt__(self, other: object) -> object:
        return self._binary_param(other, lambda left, right: left > right)

    def __le__(self, other: object) -> object:
        return self._binary_param(other, lambda left, right: left <= right)

    def __lt__(self, other: object) -> object:
        return self._binary_param(other, lambda left, right: left < right)

    def __eq__(self, other: object) -> object:  # type: ignore[override]
        return self._binary_param(other, lambda left, right: left == right)

    def __matmul__(self, other: object) -> object:
        return self.sum(over=other)

    def __rshift__(self, other: object) -> object:
        return self.sum(over=other)

    def sum(self, *, over: object | None = None) -> object:
        import numpy as np

        if over is None:
            return np.sum(np.asarray(self._values)).item()

        axis_indices = _resolve_axis_selection(self._axes, over)
        reduced = np.sum(np.asarray(self._values), axis=tuple(axis_indices))
        new_axes = tuple(
            axis for idx, axis in enumerate(self._axes) if idx not in set(axis_indices)
        )
        if getattr(reduced, "ndim", 0) == 0:
            return reduced.item()
        return ParamArray(_values=reduced, _axes=new_axes, _name=self._name)

    def cumsum(self, *, over: object) -> ParamArray:
        import numpy as np

        axis_index = _resolve_axis_selection(self._axes, over)
        if len(axis_index) != 1:
            raise _arco.ArrayDimensionError("cumsum requires exactly one IndexSet")
        return ParamArray(
            _values=np.cumsum(np.asarray(self._values), axis=axis_index[0]),
            _axes=self._axes,
            _name=self._name,
        )

    def diff(self, *, over: object) -> ParamArray:
        import numpy as np

        axis_index = _resolve_axis_selection(self._axes, over)
        if len(axis_index) != 1:
            raise _arco.ArrayDimensionError("diff requires exactly one IndexSet")
        idx = axis_index[0]
        new_values = np.diff(np.asarray(self._values), axis=idx)
        new_axes = list(self._axes)
        members = new_axes[idx].members[1:]
        new_axes[idx] = _arco.IndexSet(new_axes[idx].name, members=members)
        return ParamArray(_values=new_values, _axes=tuple(new_axes), _name=self._name)

    def roll(self, *, shift: int, over: object) -> ParamArray:
        import numpy as np

        axis_index = _resolve_axis_selection(self._axes, over)
        if len(axis_index) != 1:
            raise _arco.ArrayDimensionError("roll requires exactly one IndexSet")
        return ParamArray(
            _values=np.roll(np.asarray(self._values), shift, axis=axis_index[0]),
            _axes=self._axes,
            _name=self._name,
        )

    def __array_function__(
        self,
        func: object,
        _types: object,
        args: tuple[object, ...],
        kwargs: dict[str, object] | None,
    ) -> object:
        import numpy as np

        kwargs = kwargs or {}
        name = getattr(func, "__name__", "")
        if name == "sum":
            return self.sum(over=kwargs.get("axis"))
        if name == "cumsum":
            return self.cumsum(over=kwargs.get("axis"))
        if name == "diff":
            return self.diff(over=kwargs.get("axis"))
        if name == "roll":
            return self.roll(
                shift=int(args[1] if len(args) > 1 else kwargs.get("shift", 0)),
                over=kwargs.get("axis"),
            )
        if name == "concatenate":
            arrays = tuple(args[0])
            return _concatenate_params(arrays, kwargs.get("axis"))
        if name == "einsum":
            return _param_einsum(*args, **kwargs)
        return np.asarray(self._values).__array_function__(func, _types, args, kwargs)


def _axis_key(axis: object) -> tuple[str, int]:
    if not isinstance(axis, _arco.IndexSet):
        raise _arco.ArrayTypeError(f"expected IndexSet, got {type(axis).__name__}")
    return (axis.name, axis.size)


def _resolve_axis_index(axes: tuple[object, ...], axis: object) -> int:
    target = _axis_key(axis)
    for idx, candidate in enumerate(axes):
        if candidate is axis or _axis_key(candidate) == target:
            return idx
    for idx, candidate in enumerate(axes):
        if isinstance(candidate, _arco.IndexSet) and candidate.name == axis.name:
            return idx
    raise _arco.ArrayIndexError(
        f"IndexSet {target[0]!r} is not a dimension of this array"
    )


def _resolve_axis_selection(axes: tuple[object, ...], selection: object) -> list[int]:
    if isinstance(selection, _arco.IndexSet):
        return [_resolve_axis_index(axes, selection)]
    if selection is None:
        raise _arco.ArrayDimensionError("axis=IndexSet is required")

    try:
        items = list(selection)
    except TypeError as exc:
        raise _arco.ArrayTypeError(
            "axis must be an IndexSet or tuple of IndexSets"
        ) from exc

    indices = [_resolve_axis_index(axes, item) for item in items]
    if len(set(indices)) != len(indices):
        raise _arco.ArrayDimensionError("duplicate axes are not allowed in a reduction")
    return indices


def _union_axes(
    left: tuple[object, ...], right: tuple[object, ...]
) -> tuple[object, ...]:
    axes = list(left)
    seen = {_axis_key(axis) for axis in left}
    for axis in right:
        key = _axis_key(axis)
        if key not in seen:
            axes.append(axis)
            seen.add(key)
    return tuple(axes)


def _align_values(
    values: object, source_axes: tuple[object, ...], target_axes: tuple[object, ...]
) -> object:
    import numpy as np

    array = np.asarray(values)
    target_keys = [_axis_key(axis) for axis in target_axes]
    source_keys = [_axis_key(axis) for axis in source_axes]

    for key in source_keys:
        if key not in target_keys:
            raise _arco.ArrayDimensionError(
                f"axis {key[0]!r} is not present in the target labeled shape"
            )

    transpose_order = [
        source_keys.index(key) for key in target_keys if key in source_keys
    ]
    aligned = np.transpose(array, axes=transpose_order) if transpose_order else array

    for idx, key in enumerate(target_keys):
        if key not in source_keys:
            aligned = np.expand_dims(aligned, axis=idx)

    return np.broadcast_to(aligned, tuple(axis.size for axis in target_axes))


def _slice_axes(axes: tuple[object, ...], index: object) -> tuple[object, ...]:
    if not isinstance(index, tuple):
        index = (index,)

    padded = list(index) + [slice(None)] * (len(axes) - len(index))
    out: list[object] = []
    for axis, part in zip(axes, padded, strict=True):
        if isinstance(part, int):
            continue
        members = axis.members[part]
        out.append(_arco.IndexSet(axis.name, members=members))
    return tuple(out)


def _concatenate_params(arrays: tuple[object, ...], axis: object | None) -> ParamArray:
    import numpy as np

    params = [array for array in arrays if isinstance(array, ParamArray)]
    if not params:
        raise _arco.ArrayTypeError("np.concatenate requires ParamArray operands")

    first = params[0]
    axis_index = 0 if axis is None else _resolve_axis_index(first.axes, axis)

    for param in params[1:]:
        if len(param.axes) != len(first.axes):
            raise _arco.ArrayDimensionError(
                "all concatenated ParamArrays must have the same rank"
            )
        for idx, (left_axis, right_axis) in enumerate(
            zip(first.axes, param.axes, strict=True)
        ):
            if idx == axis_index:
                continue
            if _axis_key(left_axis) != _axis_key(right_axis):
                raise _arco.ArrayShapeMismatchError(
                    "ParamArray concatenation requires matching non-concatenated axes"
                )

    values = np.concatenate(
        [np.asarray(param.values) for param in params], axis=axis_index
    )
    new_axes = list(first.axes)
    concat_members: list[object] = []
    for param in params:
        concat_members.extend(param.axes[axis_index].members)
    new_axes[axis_index] = _arco.IndexSet(
        first.axes[axis_index].name, members=concat_members
    )
    return ParamArray(_values=values, _axes=tuple(new_axes), _name=first._name)


def _param_einsum(subscripts: object, *operands: object, **kwargs: object) -> object:
    import numpy as np

    dense_operands = [
        np.asarray(operand.values if isinstance(operand, ParamArray) else operand)
        for operand in operands
    ]
    return np.einsum(subscripts, *dense_operands, **kwargs)


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

    seen_axes: set[tuple[str, int]] = set()
    for idx, (axis, dim_size) in enumerate(zip(axes, np_values.shape, strict=True)):
        if not isinstance(axis, _arco.IndexSet):
            raise _arco.ArrayTypeError(
                f"axis {idx} must be IndexSet, got {type(axis).__name__}"
            )
        axis_key = (axis.name, axis.size)
        if axis_key in seen_axes:
            raise _arco.ArrayDimensionError(
                f"duplicate axis {axis.name!r} requires an explicit alias"
            )
        seen_axes.add(axis_key)
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
