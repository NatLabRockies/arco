from pathlib import Path
import re


def _class_block(*, source: str, class_name: str) -> str:
    marker = f"class {class_name}:"
    start = source.find(marker)
    assert start >= 0, f"missing class {class_name!r} in arco.pyi"

    tail = source[start + len(marker) :]
    next_class = tail.find("\nclass ")
    if next_class < 0:
        return source[start:]
    return source[start : start + len(marker) + next_class]


def _normalize_whitespace(text: str) -> str:
    return re.sub(r"\s+", "", text)


def _assert_signatures_present(*, block: str, expected_signatures: list[str]) -> None:
    normalized_block = _normalize_whitespace(block)
    for signature in expected_signatures:
        assert _normalize_whitespace(signature) in normalized_block


def test_expr_stub_exposes_operator_signatures() -> None:
    source = (Path(__file__).resolve().parents[1] / "arco" / "arco.pyi").read_text()
    expr_block = _class_block(source=source, class_name="Expr")
    expected_signatures = [
        "def __add__(self, other: Expr | Variable | float) -> Expr: ...",
        "def __radd__(self, other: Expr | Variable | float) -> Expr: ...",
        "def __sub__(self, other: Expr | Variable | float) -> Expr: ...",
        "def __rsub__(self, other: Expr | Variable | float) -> Expr: ...",
        "def __mul__(self, other: float) -> Expr: ...",
        "def __rmul__(self, other: float) -> Expr: ...",
        "def __neg__(self) -> Expr: ...",
        "def __truediv__(self, other: float) -> Expr: ...",
        "def __ge__(self, rhs: Expr | Variable | float) -> ConstraintExpr: ...",
        "def __le__(self, rhs: Expr | Variable | float) -> ConstraintExpr: ...",
        "def __eq__(self, rhs: object) -> ConstraintExpr: ...",
        "def __int__(self) -> int: ...",
        "def __index__(self) -> int: ...",
    ]
    _assert_signatures_present(
        block=expr_block, expected_signatures=expected_signatures
    )


def test_variable_stub_exposes_operator_signatures() -> None:
    source = (Path(__file__).resolve().parents[1] / "arco" / "arco.pyi").read_text()
    variable_block = _class_block(source=source, class_name="Variable")
    expected_signatures = [
        "def __add__(self, other: Expr | Variable | float) -> Expr: ...",
        "def __radd__(self, other: Expr | Variable | float) -> Expr: ...",
        "def __sub__(self, other: Expr | Variable | float) -> Expr: ...",
        "def __rsub__(self, other: Expr | Variable | float) -> Expr: ...",
        "def __mul__(self, other: float) -> Expr: ...",
        "def __rmul__(self, other: float) -> Expr: ...",
        "def __neg__(self) -> Expr: ...",
        "def __truediv__(self, other: float) -> Expr: ...",
        "def __ge__(self, rhs: Expr | Variable | float) -> ConstraintExpr: ...",
        "def __le__(self, rhs: Expr | Variable | float) -> ConstraintExpr: ...",
        "def __eq__(self, rhs: object) -> ConstraintExpr: ...",
        "def __int__(self) -> int: ...",
        "def __index__(self) -> int: ...",
    ]
    _assert_signatures_present(
        block=variable_block, expected_signatures=expected_signatures
    )


def test_variable_array_stub_exposes_operator_signatures() -> None:
    source = (Path(__file__).resolve().parents[1] / "arco" / "arco.pyi").read_text()
    variable_array_block = _class_block(source=source, class_name="VariableArray")
    expected_signatures = [
        "def __sub__(self, other: VariableArray | ExprArray | float | Sequence[float]) -> ExprArray: ...",
        "def __ge__(",
        "def __len__(self) -> int: ...",
        "def __getitem__(self, index: int | slice | tuple[object, object] | object) -> Variable | VariableArray: ...",
    ]
    _assert_signatures_present(
        block=variable_array_block, expected_signatures=expected_signatures
    )


def test_expr_array_stub_exposes_operator_signatures() -> None:
    source = (Path(__file__).resolve().parents[1] / "arco" / "arco.pyi").read_text()
    expr_array_block = _class_block(source=source, class_name="ExprArray")
    expected_signatures = [
        "def __add__(self, other: VariableArray | ExprArray | float | Sequence[float]) -> ExprArray: ...",
        "def __le__(",
        "def __len__(self) -> int: ...",
        "def __getitem__(self, index: int | slice | tuple[object, object] | object) -> Expr | ExprArray: ...",
    ]
    _assert_signatures_present(
        block=expr_array_block, expected_signatures=expected_signatures
    )


def test_model_stub_exposes_control_declaration_signature() -> None:
    source = (Path(__file__).resolve().parents[1] / "arco" / "arco.pyi").read_text()
    model_block = _class_block(source=source, class_name="Model")
    expected_signatures = [
        "def control(self, name: str, *index_sets: IndexSet, bounds: Bounds | BoundType, is_integer: bool = False, is_binary: bool = False) -> Variable | VariableArray: ...",
    ]
    _assert_signatures_present(
        block=model_block, expected_signatures=expected_signatures
    )


def test_model_stub_exposes_scenario_declaration_signatures() -> None:
    source = (Path(__file__).resolve().parents[1] / "arco" / "arco.pyi").read_text()
    model_block = _class_block(source=source, class_name="Model")
    expected_signatures = [
        "def scenario(self, name: str, *, solver: Solver | None = None, log_to_console: bool | None = None, primal_start: Sequence[tuple[int, float]] | None = None, time_limit: float | None = None, mip_gap: float | None = None, verbosity: int | None = None) -> None: ...",
        "def run_scenario(self, name: str, *, solver: Solver | None = None, log_to_console: bool | None = None, primal_start: Sequence[tuple[int, float]] | None = None, time_limit: float | None = None, mip_gap: float | None = None, verbosity: int | None = None) -> SolveResult: ...",
    ]
    _assert_signatures_present(
        block=model_block, expected_signatures=expected_signatures
    )

    def test_solve_result_stub_exposes_dataframe_export_signatures() -> None:
        source = (Path(__file__).resolve().parents[1] / "arco" / "arco.pyi").read_text()
        solve_result_block = _class_block(source=source, class_name="SolveResult")
        expected_signatures = [
            'def to_pandas(self, *, table: str = "variables") -> object: ...',
            'def to_polars(self, *, table: str = "variables") -> object: ...',
        ]
        _assert_signatures_present(
            block=solve_result_block,
            expected_signatures=expected_signatures,
        )
