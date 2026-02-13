from __future__ import annotations

from enum import Enum
from typing import Any, Callable, TypeVar

_BlockFnT = TypeVar("_BlockFnT", bound=Callable[..., Any])

class DropPolicy(Enum):
    DROP_ALL: DropPolicy
    KEEP_SUMMARY: DropPolicy
    KEEP_MODEL: DropPolicy

class BlockContext:
    def __init__(self, *, inputs: dict[str, Any]) -> None: ...
    @property
    def inputs(self) -> dict[str, Any]: ...
    @property
    def attachments(self) -> dict[str, Any]: ...
    def attach(self, key: str, value: Any) -> None: ...

class Transform:
    def __init__(self, steps: list[Any] | None = None) -> None: ...
    def __or__(self, other: Transform) -> Transform: ...
    def apply(self, values: Any) -> Any: ...
    def clone_with_py(self) -> Transform: ...
    @staticmethod
    def identity() -> Transform: ...
    @staticmethod
    def scale(factor: Any) -> Transform: ...
    @staticmethod
    def offset(delta: Any) -> Transform: ...
    @staticmethod
    def shift(periods: int) -> Transform: ...
    @staticmethod
    def clip(lower: float, upper: float) -> Transform: ...
    @staticmethod
    def select(indices: list[int]) -> Transform: ...

class BlockSpec:
    def __init__(self) -> None: ...
    def build(self, model: Any, *, data: Any, ctx: Any) -> Any: ...

class BlockPort:
    @property
    def block_name(self) -> str: ...
    @property
    def key(self) -> str: ...
    @property
    def kind(self) -> str: ...

class BlockLink:
    @property
    def source(self) -> BlockPort: ...
    @property
    def target(self) -> BlockPort: ...
    @property
    def transform(self) -> Transform: ...

class BlockDiagnostics:
    @property
    def build_ms(self) -> float: ...
    @property
    def solve_ms(self) -> float: ...
    @property
    def rss_bytes(self) -> int | None: ...
    @property
    def rss_delta_bytes(self) -> int | None: ...

class BlockRun:
    @property
    def name(self) -> str: ...
    @property
    def model(self) -> Any | None: ...
    @property
    def solution(self) -> Any | None: ...
    @property
    def outputs(self) -> dict[str, Any]: ...
    @property
    def attachments(self) -> dict[str, Any]: ...
    @property
    def diagnostics(self) -> BlockDiagnostics: ...
    def inspect(
        self,
        *,
        include_coeffs: bool = False,
        include_slacks: bool = True,
        variable_ids: list[int] | None = None,
        constraint_ids: list[int] | None = None,
    ) -> Any | None: ...

class BuildResult:
    @property
    def model(self) -> Any: ...
    @property
    def outputs(self) -> Any: ...
    @property
    def spec_name(self) -> str: ...
    @property
    def spec_version(self) -> str: ...

class Block:
    def __init__(
        self,
        build: Any,
        *,
        name: str,
        inputs: dict[str, Any] | None = None,
        outputs: dict[str, Any] | None = None,
        extract: Any | None = None,
        cache_scaffolding: bool = False,
        warm_start: bool = False,
        drop_policy: DropPolicy = ...,
    ) -> None: ...
    @property
    def name(self) -> str: ...
    @property
    def inputs(self) -> dict[str, Any]: ...
    @property
    def outputs(self) -> dict[str, Any]: ...
    @property
    def cache_scaffolding(self) -> bool: ...
    @property
    def warm_start(self) -> bool: ...
    @property
    def drop_policy(self) -> DropPolicy: ...
    def input(self, key: str) -> BlockPort: ...
    def output(self, key: str) -> BlockPort: ...
    @staticmethod
    def from_spec(
        spec: Any,
        *,
        drop_policy: DropPolicy = ...,
        warm_start: bool = False,
        allow_slacks: bool = False,
        slack_penalty: float = 1e6,
    ) -> Block: ...

class BlockModel:
    def __init__(self, *, name: str | None = None) -> None: ...
    @property
    def name(self) -> str: ...
    def add_block(
        self,
        block_or_build: Any,
        *,
        name: str | None = None,
        inputs: dict[str, Any] | None = None,
        inputs_schema: dict[str, Any] | None = None,
        outputs: dict[str, Any] | None = None,
        extract: Any | None = None,
        cache_scaffolding: bool = False,
        warm_start: bool = False,
        drop_policy: DropPolicy = ...,
    ) -> Block: ...
    def link(
        self,
        source: BlockPort,
        target: BlockPort,
        transform: Transform | None = None,
    ) -> None: ...
    def validate(self) -> None: ...
    def solve(
        self,
        *,
        solver: Any | None = None,
        log_to_console: bool | None = None,
        time_limit: float | None = None,
        mip_gap: float | None = None,
        verbosity: int | None = None,
    ) -> list[BlockRun]: ...

# Module-level functions

def block(
    func: _BlockFnT | None = None,
    *,
    name: str | None = None,
) -> _BlockFnT | Callable[[_BlockFnT], _BlockFnT]: ...
def block_spec(
    *,
    name: str,
    data_schema: Any,
    outputs_schema: Any,
    build: Callable[..., Any],
    version: str = "0.0.0",
) -> BlockSpec: ...
def build_model_from_spec(
    *,
    spec: Any,
    data: Any,
    allow_slacks: bool = False,
    slack_penalty: float = 1e6,
) -> BuildResult: ...
def inspect_model(
    *,
    model: Any,
    constraints: list[int] | None = None,
    variables: list[int] | None = None,
    include_coeffs: bool = False,
    include_slacks: bool = True,
) -> Any: ...
def schemas_compatible(schema_a: Any, schema_b: Any) -> tuple[bool, str]: ...
def specs_are_swappable(spec_a: Any, spec_b: Any) -> tuple[bool, str]: ...
