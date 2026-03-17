from pathlib import Path


def _class_block(*, source: str, class_name: str) -> str:
    marker = f"class {class_name}:"
    start = source.find(marker)
    assert start >= 0, f"missing class {class_name!r} in arco.pyi"

    tail = source[start + len(marker) :]
    next_class = tail.find("\nclass ")
    if next_class < 0:
        return source[start:]
    return source[start : start + len(marker) + next_class]


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
    for signature in expected_signatures:
        assert signature in expr_block


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
    for signature in expected_signatures:
        assert signature in variable_block
