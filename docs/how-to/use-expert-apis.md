# Use Expert APIs

Most models should use named variables, named constraints, `IndexSet`,
`param(...)`, `model.inspect(...)`, and `result.value(...)`. The APIs below are
for integration code that already owns a sparse matrix or needs raw solver-order
vectors. They are stable escape hatches, not the beginner modeling path.

## Import A CSC Matrix

Use `Model.from_csc(...)` when another system has already produced the
constraint matrix. The arrays use solver order:

- `num_constraints` and `num_variables` define the matrix shape.
- `col_ptrs`, `row_indices`, and `values` define the constraint matrix in CSC
  form.
- variable and constraint bounds are parallel arrays.
- `is_integer` is a parallel boolean array for variables.
- floating-point arrays are imported as `float64`; `float32` buffers are also
  accepted and widened before storage.

```python doctest
>>> import math
>>> import arco
>>> model = arco.Model.from_csc(
...     num_constraints=1,
...     num_variables=1,
...     col_ptrs=[0, 1],
...     row_indices=[0],
...     values=[1.0],
...     var_lower=[0.0],
...     var_upper=[math.inf],
...     con_lower=[1.0],
...     con_upper=[math.inf],
...     is_integer=[False],
... )
>>> model.set_objective(sense=arco.Sense.MINIMIZE, terms=[(0, 1.0)], name="min_x")
>>> snapshot = model.inspect(include_coeffs=True)
>>> snapshot.metadata.variables
1
>>> snapshot.metadata.constraints
1
>>> snapshot.metadata.coefficients
1
>>> snapshot.metadata.memory.coefficient_value_bytes
8
>>> solution = model.solve(log_to_console=False)
>>> solution.is_optimal()
True
>>> solution.get_primal(index=0)
1.0
>>> solution.primal_values
[1.0]
```

Prefer `result.value(variable)`, `result.dual(constraint)`, and
`result.reduced_cost(variable)` when you have model objects. Use
`get_primal(index=...)`, `get_constraint_dual(index=...)`,
`get_variable_dual(index=...)`, and raw vectors only when your integration
already tracks Arco variable and constraint order.

When an expert API asks for a variable index, pass the solver-order integer
explicitly. `int(variable)` is available for this purpose, but normal modeling
and result reads should keep using the variable object.

## Edit Raw Coefficients

Use object-facing expressions such as `model.add_constraint(x >= 1.0)` for
ordinary modeling. Use `set_coefficient(...)` and `set_objective(...)` only when
your integration already owns solver-order row and column IDs. Use
`set_variable_name(...)` and `set_constraint_name(...)` for the same raw-index
workflows when imported matrix data needs inspectable names.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
>>> row = model.add_constraint(x >= 1.0, name="demand")
>>> model.set_coefficient(var_idx=int(x), constraint_idx=int(row), coeff=2.0)
>>> model.set_variable_name(index=int(x), name="x")
>>> model.set_constraint_name(index=int(row), name="demand")
>>> model.set_objective(
...     sense=arco.Sense.MINIMIZE,
...     terms=[(int(x), 1.0)],
...     name="min_x",
... )
>>> solution = model.solve(log_to_console=False)
>>> round(solution.value(x), 6)
0.5
```

## Export Sparse Matrices

Use `export_csc()`, `export_crs()`, or `export_coo()` when handing a model to
NumPy/SciPy tooling, a custom analyzer, or a benchmark harness.

```python doctest
>>> import math
>>> import arco
>>> model = arco.Model.from_csc(
...     num_constraints=1,
...     num_variables=1,
...     col_ptrs=[0, 1],
...     row_indices=[0],
...     values=[1.0],
...     var_lower=[0.0],
...     var_upper=[math.inf],
...     con_lower=[1.0],
...     con_upper=[math.inf],
...     is_integer=[False],
... )
>>> csc = model.export_csc()
>>> csc
{'col_ptrs': [0, 1], 'row_indices': [0], 'values': [1.0], 'shape': (1, 1)}
>>> crs = model.export_crs()
>>> crs
{'row_ptrs': [0, 1], 'col_indices': [0], 'values': [1.0], 'shape': (1, 1)}
>>> coo = model.export_coo()
>>> coo
{'rows': [0], 'cols': [0], 'values': [1.0], 'shape': (1, 1)}
```

These exports intentionally expose raw row and column indices. For normal model
debugging, `model.inspect(include_coeffs=True)` is easier to read because it
includes named variable, constraint, objective, and memory metadata.

---

[How-to Guides](./) | [Docs home](../)
