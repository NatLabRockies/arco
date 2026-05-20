from __future__ import annotations

from typing import Callable, TypeVar, overload

_BlockFnT = TypeVar("_BlockFnT", bound=Callable[..., object])

__all__: list[str]

@overload
def block(func: _BlockFnT, *, name: str | None = None) -> _BlockFnT: ...
@overload
def block(
    func: None = None,
    *,
    name: str | None = None,
) -> Callable[[_BlockFnT], _BlockFnT]: ...
