# Handle Errors

Arco raises typed exceptions for model-building mistakes and returns status codes
for solver outcomes. This separation lets you use standard Python exception
handling for programming errors while inspecting the solve result for
optimization-level outcomes like infeasibility or unboundedness.

## Model-building errors

Errors in model construction are raised immediately when the offending call is
made. Each error type corresponds to a specific kind of mistake, so you can
catch exactly the problem you expect.

Passing bounds where the lower exceeds the upper raises `BoundsInvalidError`.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> try:
...     model.add_variable(bounds=arco.Bounds(lower=10.0, upper=0.0))
... except arco.BoundsInvalidError:
...     print("lower bound exceeds upper bound")
lower bound exceeds upper bound
```

Calling `solve()` on a model that has no variables raises `ModelEmptyError`.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> try:
...     model.solve(log_to_console=False)
... except arco.ModelEmptyError:
...     print("model has no variables")
model has no variables
```

These exceptions are raised before the solver is ever invoked, so you get fast
feedback during development.

Solver configuration is validated the same way. For example, `threads=0`
raises `SolverInvalidSettingError` before any backend is selected.

```python doctest
>>> import arco
>>> try:
...     arco.Solver(threads=0)
... except arco.SolverInvalidSettingError as e:
...     print(arco.error_code(e))
arco::solver::invalid_setting
```

Unavailable solver backends raise `SolverNotAvailableError`, not
`SolverInternalError`. This lets automation distinguish "install or enable a
backend" from a backend crash.

Logging setup follows the same rule. Bad filters or invalid `ARCO_LOG_FORMAT`
values raise `LoggingConfigError`, while `ARCO_LOG_FILE` open failures raise
`LoggingIoError`.

Missing Python-side dependencies raise `DependencyMissingError`. For example,
`arco.param(...)` requires NumPy because labeled parameters are normalized
through NumPy arrays before shape validation.

Invalid `axes=` values for indexed variables raise `ArrayTypeError`, so array
modeling mistakes stay in the same `arco::array::*` diagnostic namespace as
shape, dimension, and mask errors.

Invalid array comparison right-hand sides also raise `ArrayTypeError`. This
keeps expression authoring mistakes such as comparing a `VariableArray` to an
unsupported object in the array diagnostic namespace.

Out-of-range indexing on `VariableArray`, `ExprArray`, and `ConstraintArray`
raises `ArrayIndexError` with `arco::array::index`.

Constraint sense parsing is also typed. Invalid `sense=` values for
`add_constraints(...)` raise `ConstraintSenseError` with
`arco::constraint::sense`.

## Solver outcomes

Infeasible and unbounded models do not raise exceptions. The solver runs to
completion and reports the outcome on the returned `SolveResult`. Check the
status to decide what to do next.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=1.0))
>>> _ = model.add_constraint(x >= 5.0)
>>> model.minimize(x)
>>> solution = model.solve(log_to_console=False)
>>> solution.is_infeasible()
True
>>> solution.status
SolutionStatus.INFEASIBLE
```

This design keeps the control flow predictable: exceptions mean something went
wrong building the model, while status codes mean the solver finished but the
problem itself has no feasible or bounded solution.

## Catch all arco errors

Every arco exception inherits from `arco.ArcoError`, which itself inherits from
`Exception`. When you want a single handler for any model-building error, catch
the base class.

```python doctest
>>> import arco
>>> try:
...     model = arco.Model()
...     model.add_variable(bounds=arco.Bounds(lower=10.0, upper=0.0))
... except arco.ArcoError as e:
...     print(type(e).__name__)
BoundsInvalidError
```

This is useful at API boundaries or in batch pipelines where you want to log the
error and continue rather than crash.

## Branch on stable error codes

Typed exceptions also expose a stable `code` string. Use it when logs,
notebooks, or service boundaries need a machine-readable category without
depending on the exact human message.

```python doctest
>>> import arco
>>> plant = arco.IndexSet(name="plant", members=["north", "south"])
>>> try:
...     arco.param([1.0, 2.0, 3.0], axes=(plant,))
... except arco.ArcoError as e:
...     print(arco.error_code(e))
arco::array::shape_mismatch
```

## Diagnostic code namespace

Arco uses the same `arco::<area>::<reason>` code shape for Python exceptions
and CLI/KDL diagnostics. Treat the code as the stable automation key and the
message as human-facing text that may improve over time.

Use `arco.diagnostic_codes()` when Python automation needs the canonical code
strings instead of duplicating literals:

```python doctest
>>> import arco
>>> arco.diagnostic_codes()["ARRAY_SHAPE_MISMATCH"]
'arco::array::shape_mismatch'
>>> arco.diagnostic_codes()["ARRAY_DIMENSION"]
'arco::array::dimension'
>>> arco.diagnostic_codes()["ARRAY_INDEX"]
'arco::array::index'
>>> arco.diagnostic_codes()["ARRAY_OVERFLOW"]
'arco::array::overflow'
>>> arco.diagnostic_codes()["ARRAY_TYPE"]
'arco::array::type'
>>> arco.diagnostic_codes()["BLOCK_CONTRACT"]
'arco::block::contract'
>>> arco.diagnostic_codes()["BLOCK_RESULT"]
'arco::block::result'
>>> arco.diagnostic_codes()["BLOCK_ARTIFACT_IO"]
'arco::block::artifact_io'
>>> arco.diagnostic_codes()["BOUNDS_INVALID"]
'arco::bounds::invalid'
>>> arco.diagnostic_codes()["COMPILE_MISSING_COLUMN"]
'arco::compile::missing_column'
>>> arco.diagnostic_codes()["CONSTRAINT_BOUNDS_MISSING"]
'arco::constraint::bounds_missing'
>>> arco.diagnostic_codes()["CONSTRAINT_INVALID_BOUNDS"]
'arco::constraint::invalid_bounds'
>>> arco.diagnostic_codes()["CONSTRAINT_INVALID_ID"]
'arco::constraint::invalid_id'
>>> arco.diagnostic_codes()["CONSTRAINT_NOT_FOUND"]
'arco::constraint::not_found'
>>> arco.diagnostic_codes()["CONSTRAINT_SENSE"]
'arco::constraint::sense'
>>> arco.diagnostic_codes()["CONSTRAINT_TYPE"]
'arco::constraint::type'
>>> arco.diagnostic_codes()["CSC_INVALID_DATA"]
'arco::csc::invalid_data'
>>> arco.diagnostic_codes()["DEPENDENCY_MISSING"]
'arco::dependency::missing'
>>> arco.diagnostic_codes()["EXPR_COEFFICIENT"]
'arco::expr::coefficient'
>>> arco.diagnostic_codes()["EXPR_CONSTANT_OFFSET"]
'arco::expr::constant_offset'
>>> arco.diagnostic_codes()["EXPR_DIVISION_BY_ZERO"]
'arco::expr::division_by_zero'
>>> arco.diagnostic_codes()["EXPR_NOT_SINGLE_VARIABLE"]
'arco::expr::not_single_variable'
>>> arco.diagnostic_codes()["EXPR_TYPE"]
'arco::expr::type'
>>> arco.diagnostic_codes()["INDEX_SET_ARGUMENT"]
'arco::index_set::argument'
>>> arco.diagnostic_codes()["INDEX_SET_EMPTY"]
'arco::index_set::empty'
>>> arco.diagnostic_codes()["INDEX_SET_INDEX"]
'arco::index_set::index'
>>> arco.diagnostic_codes()["INDEX_SET_TYPE"]
'arco::index_set::type'
>>> arco.diagnostic_codes()["LOGGING_CONFIG"]
'arco::logging::config'
>>> arco.diagnostic_codes()["LOGGING_IO"]
'arco::logging::io'
>>> arco.diagnostic_codes()["MODEL_BINARY_BOUNDS"]
'arco::model::binary_bounds'
>>> arco.diagnostic_codes()["OBJECTIVE_ALREADY_SET"]
'arco::objective::already_set'
>>> arco.diagnostic_codes()["OBJECTIVE_INDEX"]
'arco::objective::index'
>>> arco.diagnostic_codes()["OBJECTIVE_MISSING"]
'arco::objective::missing'
>>> arco.diagnostic_codes()["SLACK_INVALID_PENALTY"]
'arco::slack::invalid_penalty'
>>> arco.diagnostic_codes()["SLACK_VALUE_UNAVAILABLE"]
'arco::slack::value_unavailable'
>>> arco.diagnostic_codes()["SLACK_BOUND"]
'arco::slack::bound'
>>> arco.diagnostic_codes()["SOLVER_INDEX"]
'arco::solver::index'
>>> arco.diagnostic_codes()["SOLVER_NOT_AVAILABLE"]
'arco::solver::not_available'
>>> arco.diagnostic_codes()["SOLVER_TYPE"]
'arco::solver::type'
>>> arco.diagnostic_codes()["VARIABLE_INVALID_ID"]
'arco::variable::invalid_id'
>>> arco.diagnostic_codes()["VARIABLE_NOT_FOUND"]
'arco::variable::not_found'
```

Common namespaces:

| Namespace             | Surface         | Meaning                                                                          |
| --------------------- | --------------- | -------------------------------------------------------------------------------- |
| `arco::array::*`      | Python          | Labeled-axis, parameter, active-mask, and array-shape errors                     |
| `arco::block::*`      | Python          | Block decorator, schema, registration, link-contract, and artifact writer errors |
| `arco::dependency::*` | Python          | Missing Python package dependencies needed by public helpers                     |
| `arco::logging::*`    | Python          | Logging filter, format, and file-output setup errors                             |
| `arco::model::*`      | Python/Rust     | Primitive model construction errors                                              |
| `arco::objective::*`  | Python/Rust     | Missing, duplicate, or invalid objective errors                                  |
| `arco::solver::*`     | Python/CLI/Rust | Solver status, capability, configuration, and backend errors                     |
| `arco::target::*`     | Rust/CLI        | Lowered solve-target validation errors                                           |
| `arco::source::*`     | KDL/CLI         | KDL parse and source-shape errors                                                |
| `arco::semantic::*`   | KDL/CLI         | KDL declaration, data-binding, and scenario validation errors                    |
| `arco::compile::*`    | KDL/CLI         | Lowering, data materialization, and model compilation errors                     |

For CLI automation, prefer JSON output where available. For example,
`arco kdl check --materialize-data --format json model.kdl` reports diagnostic
objects with stable `code` fields such as `arco::compile::csv`,
`arco::semantic::csv`, `arco::compile::missing_column`, and
`arco::compile::invalid_number`. These common CLI/KDL codes are included in
`arco.diagnostic_codes()` so Python automation can compare against the same
registry instead of duplicating string literals.

## Error reference

The table below lists the most common error classes. All inherit from
`arco.ArcoError`.

| Error                          | Description                                          |
| ------------------------------ | ---------------------------------------------------- |
| `ModelEmptyError`              | Model has no variables                               |
| `ObjectiveMissingError`        | No objective set before solving                      |
| `ObjectiveAlreadySetError`     | Objective already defined                            |
| `BoundsInvalidError`           | Lower bound exceeds upper bound                      |
| `VariableInvalidBoundsError`   | Variable bounds are invalid                          |
| `VariableNotFoundError`        | Named variable lookup failed                         |
| `ConstraintInvalidBoundsError` | Constraint bounds are invalid                        |
| `ConstraintNotFoundError`      | Named constraint lookup failed                       |
| `ConstraintSenseError`         | Constraint comparison sense is invalid               |
| `SlackInvalidPenaltyError`     | Slack penalty must be finite and non-negative        |
| `SlackValueUnavailableError`   | Slack value was read before solving                  |
| `ArrayIndexError`              | Array index or axis lookup is invalid                |
| `ArrayShapeMismatchError`      | Array dimensions don't match                         |
| `BlockContractError`           | Block decorator, schema, or link contract is invalid |
| `BlockResultError`             | Block result lookup failed                           |
| `DependencyMissingError`       | Required Python dependency is unavailable            |
| `LoggingConfigError`           | Logging filter or format is invalid                  |
| `LoggingIoError`               | Logging output file could not be opened              |

> [!NOTE]
> Solver outcomes such as infeasible, unbounded, and time limit are not
> exceptions. They are returned as status values on `SolveResult`. Use
> `solution.is_infeasible()`, `solution.status`, and related methods
> to inspect the outcome after calling `model.solve()`.

---

[How-to Guides](./) | [Docs home](../)
