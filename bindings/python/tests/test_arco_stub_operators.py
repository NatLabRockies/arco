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
        "def __iter__(self) -> Iterator[Variable | Expr]: ...",
        "def __getitem__(self, index: int | slice | tuple[object, ...] | object) -> Variable | Expr | VariableArray: ...",
        "def dense_count(self) -> int: ...",
        "def active_count(self) -> int: ...",
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
        "def __iter__(self) -> Iterator[Expr]: ...",
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
        expected_signatures=[
            "def __init__(self, *, name: str, size: int | None = None, members: Sequence[IndexMember] | None = None, ) -> None: ...",
            "def alias(self, name: str) -> IndexSet: ...",
        ],
    )


def test_param_stub_exposes_function_signature() -> None:
    source = (Path(__file__).resolve().parents[1] / "arco" / "arco.pyi").read_text()
    normalized = _normalize_whitespace(source)
    for fragment in [
        "def param(",
        "axes: tuple[IndexSet, ...]",
        "name: str | None = None",
        ") -> ParamArray: ...",
    ]:
        assert _normalize_whitespace(fragment) in normalized


def test_param_array_stub_exposes_labeled_operator_signatures() -> None:
    source = (Path(__file__).resolve().parents[1] / "arco" / "arco.pyi").read_text()
    param_block = _class_block(source=source, class_name="ParamArray")
    expected_signatures = [
        "def name(self) -> str | None: ...",
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
    assert (
        _normalize_whitespace("def add_variable( self, *, bounds:") in normalized_model
    )
    assert _normalize_whitespace("def add_variables(") in normalized_model
    assert _normalize_whitespace("axes: tuple[IndexSet, ...]") in normalized_model
    assert _normalize_whitespace("active: object | None = None") in normalized_model
    assert _normalize_whitespace("def add_constraints(") in normalized_model


def test_model_stub_exposes_expert_sparse_apis_without_internal_columns() -> None:
    source = (Path(__file__).resolve().parents[1] / "arco" / "arco.pyi").read_text()
    model_block = _class_block(source=source, class_name="Model")
    normalized_model = _normalize_whitespace(model_block)

    expected_signatures = [
        "def set_coefficient( self, *, var_idx: int, constraint_idx: int, coeff: float ) -> None: ...",
        "def set_variable_name(self, *, index: int, name: str) -> None: ...",
        "def set_constraint_name(self, *, index: int, name: str) -> None: ...",
        "def set_objective( self, *, sense: Sense, terms: Sequence[tuple[int, float]], name: str | None = None, ) -> None: ...",
        "def export_csc(self) -> CscExport: ...",
        "def export_crs(self) -> CrsExport: ...",
        "def export_coo(self) -> CooExport: ...",
    ]
    _assert_signatures_present(
        block=model_block, expected_signatures=expected_signatures
    )
    assert _normalize_whitespace("def get_columns(") not in normalized_model
    assert _normalize_whitespace("def export_arrow(") not in normalized_model


def test_solve_result_stub_exposes_ladder_accessors() -> None:
    source = (Path(__file__).resolve().parents[1] / "arco" / "arco.pyi").read_text()
    result_block = _class_block(source=source, class_name="SolveResult")
    expected_signatures = [
        "def value(self, variable: Variable | VariableArray) -> float | object: ...",
        "def dual(self, constraint: Constraint) -> float: ...",
        "def reduced_cost(self, variable: Variable) -> float: ...",
        "def slack(self, constraint: Constraint) -> float: ...",
    ]
    _assert_signatures_present(
        block=result_block, expected_signatures=expected_signatures
    )
    normalized_result = _normalize_whitespace(result_block)
    assert _normalize_whitespace("def get_value(") not in normalized_result
    assert _normalize_whitespace("def get_dual(") not in normalized_result
    assert _normalize_whitespace("def get_slack(") not in normalized_result
    assert _normalize_whitespace("def get_reduced_cost(") not in normalized_result


def test_stub_exposes_ladder_diagnostic_helpers_and_errors() -> None:
    source = (Path(__file__).resolve().parents[1] / "arco" / "arco.pyi").read_text()

    expected_top_level = [
        "def error_code(exc: BaseException) -> str | None: ...",
        "def diagnostic_codes() -> dict[str, str]: ...",
        "class MetadataConversionError(ArcoError): ...",
        "class BlockArtifactError(ArcoError): ...",
        "class SolverNotAvailableError(ArcoError): ...",
    ]
    _assert_signatures_present(block=source, expected_signatures=expected_top_level)


def test_model_iterator_stubs_expose_stable_protocol() -> None:
    source = (Path(__file__).resolve().parents[1] / "arco" / "arco.pyi").read_text()
    variable_iterator = _class_block(source=source, class_name="VariableIterator")
    constraint_iterator = _class_block(source=source, class_name="ConstraintIterator")

    _assert_signatures_present(
        block=variable_iterator,
        expected_signatures=[
            "def __iter__(self) -> Iterator[Variable]: ...",
            "def __next__(self) -> Variable: ...",
            "def __len__(self) -> int: ...",
        ],
    )
    _assert_signatures_present(
        block=constraint_iterator,
        expected_signatures=[
            "def __iter__(self) -> Iterator[Constraint]: ...",
            "def __next__(self) -> Constraint: ...",
            "def __len__(self) -> int: ...",
        ],
    )


def test_block_results_stub_exposes_stage_diagnostics_accessors() -> None:
    source = (Path(__file__).resolve().parents[1] / "arco" / "arco.pyi").read_text()
    block_results = _class_block(source=source, class_name="BlockResults")
    expected_signatures = [
        "def __len__(self) -> int: ...",
        "def __contains__(self, key: str) -> bool: ...",
        "def __iter__(self) -> Iterator[str]: ...",
        "def get(self, key: str, default: object = None) -> SolveResult | object: ...",
        "def diagnostics(self) -> list[dict[str, object]]: ...",
        "def diagnostics_json(self) -> str: ...",
    ]
    _assert_signatures_present(
        block=block_results, expected_signatures=expected_signatures
    )


def test_blocks_module_stub_matches_stable_import_surface() -> None:
    source = (Path(__file__).resolve().parents[1] / "arco" / "blocks.pyi").read_text()
    normalized = _normalize_whitespace(source)

    assert "defblock(" in normalized
    assert "classBlockSpec" not in normalized
    assert "defbuild_model_from_spec" not in normalized
    assert "allow_slacks" not in source
