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
        "def __sub__(self, other: VariableArray | ExprArray | ParamArray | float | Sequence[float]) -> ExprArray: ...",
        "def __ge__(",
        "def __len__(self) -> int: ...",
        "def __getitem__(self, index: int | slice | tuple[object, ...] | object) -> Variable | Expr | VariableArray: ...",
        "def cumsum(self, *, over: IndexSet) -> Expr | ExprArray: ...",
    ]
    _assert_signatures_present(
        block=variable_array_block, expected_signatures=expected_signatures
    )


def test_expr_array_stub_exposes_operator_signatures() -> None:
    source = (Path(__file__).resolve().parents[1] / "arco" / "arco.pyi").read_text()
    expr_array_block = _class_block(source=source, class_name="ExprArray")
    expected_signatures = [
        "def __add__(self, other: VariableArray | ExprArray | ParamArray | float | Sequence[float]) -> ExprArray: ...",
        "def __le__(",
        "def __len__(self) -> int: ...",
        "def __getitem__(self, index: int | slice | tuple[object, ...] | object) -> Expr | ExprArray: ...",
        "def roll(self, *, shift: int, over: IndexSet) -> Expr | ExprArray: ...",
    ]
    _assert_signatures_present(
        block=expr_array_block, expected_signatures=expected_signatures
    )


def test_index_set_stub_exposes_alias_signature() -> None:
    source = (Path(__file__).resolve().parents[1] / "arco" / "arco.pyi").read_text()
    index_set_block = _class_block(source=source, class_name="IndexSet")
    _assert_signatures_present(
        block=index_set_block,
        expected_signatures=["def alias(self, name: str) -> IndexSet: ..."],
    )


def test_param_stub_exposes_function_signature() -> None:
    source = (Path(__file__).resolve().parents[1] / "arco" / "arco.pyi").read_text()
    normalized = _normalize_whitespace(source)
    for fragment in [
        "def param(",
        "*axes: IndexSet",
        "name: str | None = None",
        ") -> ParamArray: ...",
    ]:
        assert _normalize_whitespace(fragment) in normalized


def test_param_array_stub_exposes_labeled_operator_signatures() -> None:
    source = (Path(__file__).resolve().parents[1] / "arco" / "arco.pyi").read_text()
    param_block = _class_block(source=source, class_name="ParamArray")
    expected_signatures = [
        "def __mul__(self, other: ParamArray | float | object) -> object: ...",
        "def __matmul__(self, other: IndexSet | Sequence[IndexSet]) -> object: ...",
        "def cumsum(self, *, over: IndexSet) -> ParamArray: ...",
    ]
    _assert_signatures_present(
        block=param_block, expected_signatures=expected_signatures
    )


def test_model_stub_exposes_active_kwargs_for_array_builders() -> None:
    source = (Path(__file__).resolve().parents[1] / "arco" / "arco.pyi").read_text()
    model_block = _class_block(source=source, class_name="Model")
    normalized_model = _normalize_whitespace(model_block)
    assert _normalize_whitespace("def add_variables(") in normalized_model
    assert _normalize_whitespace("active: object | None = None") in normalized_model
    assert _normalize_whitespace("def add_constraints(") in normalized_model
