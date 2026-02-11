# Numpy Integration

Arco's variable arrays implement the numpy array protocols (`__array_ufunc__`
and `__array_function__`), so you can mix numpy arrays with arco arrays in
natural expressions. Element-wise arithmetic produces `ExprArray` objects that
plug directly into objectives and constraints, while functions like `np.sum`
and `np.dot` reduce to scalar `Expr` objects ready for `minimize` or
`maximize`.

## Weighted sums

Multiplying a numpy array of coefficients by a `VariableArray` produces an
`ExprArray` whose elements are the coefficient-scaled variables. Passing that
result to `np.sum` collapses it into a single `Expr` -- a weighted sum you
can hand to the solver as an objective.

```python doctest
>>> import arco
>>> import numpy as np
>>> model = arco.Model()
>>> units = arco.IndexSet("unit", members=["solar", "wind", "gas"])
>>> costs = np.array([0.0, 0.0, 30.0])
>>> gen = model.add_variables(index_sets=[units], bounds=arco.Bounds(lower=0.0, upper=100.0), name="gen")
>>> weighted = costs * gen
>>> type(weighted).__name__
'ExprArray'
>>> objective = np.sum(costs * gen)
>>> type(objective).__name__
'Expr'
>>> _ = model.add_constraints(expr=gen >= [50.0, 30.0, 0.0])
>>> model.minimize(expr=objective)
>>> solution = model.solve(log_to_console=False)
>>> solution.status
SolutionStatus.OPTIMAL
>>> round(solution.objective_value, 6)
0.0
```

`np.dot` gives the same scalar result for 1D arrays and reads a little closer
to the mathematical notation.

```python doctest
>>> import arco
>>> import numpy as np
>>> model = arco.Model()
>>> units = arco.IndexSet("unit", members=["solar", "wind", "gas"])
>>> costs = np.array([5.0, 3.0, 30.0])
>>> gen = model.add_variables(index_sets=[units], bounds=arco.Bounds(lower=0.0, upper=100.0))
>>> _ = model.add_constraints(expr=gen >= [40.0, 40.0, 0.0])
>>> model.minimize(expr=np.dot(costs, gen))
>>> solution = model.solve(log_to_console=False)
>>> round(solution.objective_value, 6)
320.0
```

## Element-wise bounds

When different variables need different bounds, pass numpy arrays to
`arco.Bounds`. The lower and upper arrays are zipped element-wise with the
variables created by `add_variables`, so each variable gets its own range.

```python doctest
>>> import arco
>>> import numpy as np
>>> model = arco.Model()
>>> units = arco.IndexSet("unit", size=3)
>>> lo = np.array([0.0, 10.0, 20.0])
>>> hi = np.array([100.0, 200.0, 300.0])
>>> p = model.add_variables(index_sets=[units], bounds=arco.Bounds(lo, hi), name="power")
>>> [v.bounds for v in p.variables]
[Bounds(lower=0, upper=100), Bounds(lower=10, upper=200), Bounds(lower=20, upper=300)]
```

This replaces the pattern of looping over indices and calling `add_variable`
one at a time. The resulting `VariableArray` works with all the numpy
operations described in this guide.

## Indexing and slicing

Variable arrays support integer indexing, multi-dimensional tuple indexing,
slicing, and boolean masks. A single-element index returns a `Variable`;
any multi-element selection returns a `VariableArray`.

```python doctest
>>> import arco
>>> import numpy as np
>>> model = arco.Model()
>>> rows = arco.IndexSet("row", size=4)
>>> cols = arco.IndexSet("col", size=3)
>>> x = model.add_variables(index_sets=[rows, cols], bounds=arco.Bounds(lower=0.0, upper=10.0))
>>> x[0, 1]
Variable(1, Bounds(0, 10))
>>> x[1:3, 0:2].shape
(2, 2)
>>> x[0, 1:].shape
(2,)
```

For multi-dimensional indexing, the first component selects rows and the
second selects columns. When both components are slices, the result is a 2D
`VariableArray`. When one component is an integer and the other is a slice,
the result is a 1D `VariableArray` representing a single row or column.

### Boolean masks

Boolean numpy arrays select elements where the mask is `True`. The mask must
have the same total number of elements as the variable array. The result is
always a flat `VariableArray`, regardless of the original dimensionality.

```python doctest
>>> import arco
>>> import numpy as np
>>> model = arco.Model()
>>> rows = arco.IndexSet("row", size=4)
>>> cols = arco.IndexSet("col", size=3)
>>> x = model.add_variables(index_sets=[rows, cols], bounds=arco.Bounds(lower=0.0, upper=10.0))
>>> mask = np.array([[True,False,True],[False,True,False],[True,False,True],[False,True,False]])
>>> x[mask].shape
(6,)
```

This is useful for selecting variables that correspond to nonzero entries in a
data matrix or adjacency structure. Build the mask from a numpy condition on
your data, then apply it to the variable array.

```python doctest
>>> import arco
>>> import numpy as np
>>> model = arco.Model()
>>> rows = arco.IndexSet("row", size=3)
>>> cols = arco.IndexSet("col", size=3)
>>> G = np.array([[1, 0, 1], [0, 1, 0], [1, 0, 1]])
>>> x = model.add_variables(index_sets=[rows, cols], bounds=arco.Bounds(lower=0.0, upper=10.0))
>>> active = x[G == 1]
>>> active.shape
(5,)
>>> inactive = x[G == 0]
>>> inactive.shape
(4,)
```

> [!NOTE]
> Fancy indexing with integer lists (e.g. `x[[0, 2]]`) is not supported. Use
> boolean masks instead: `x[np.array([True, False, True])]`.

## Reduction operators

Arco provides three equivalent ways to sum a variable array along a named
dimension: the `.sum(over=...)` method, the `>>` operator, and the `@`
operator. All three take an `IndexSet` on the right-hand side and collapse that
dimension, returning an `ExprArray` with one fewer axis (or a scalar `Expr` if
no dimensions remain).

```python doctest
>>> import arco
>>> model = arco.Model()
>>> i = arco.IndexSet("row", size=3)
>>> j = arco.IndexSet("col", size=4)
>>> x = model.add_variables(index_sets=[i, j], bounds=arco.Bounds(lower=0.0, upper=10.0))
>>> x.sum(over=j).shape
(3,)
>>> (x >> j).shape
(3,)
>>> (x @ j).shape
(3,)
```

Read `x >> j` as "sum x over j". The result has shape `(3,)` because the
column dimension (size 4) was collapsed, leaving only the row dimension
(size 3). Each element of the resulting `ExprArray` is the sum of the four
variables in that row.

Chaining the operator reduces additional dimensions. When all dimensions are
exhausted, the result is a scalar `Expr`.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> i = arco.IndexSet("row", size=3)
>>> j = arco.IndexSet("col", size=4)
>>> x = model.add_variables(index_sets=[i, j], bounds=arco.Bounds(lower=0.0, upper=10.0))
>>> row_sums = x >> j
>>> row_sums.shape
(3,)
>>> total = (x >> j) >> i
>>> type(total).__name__
'Expr'
```

You can also pass multiple index sets to `.sum(over=...)` to reduce several
dimensions at once. Passing no argument sums everything to a scalar.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> i = arco.IndexSet("row", size=3)
>>> j = arco.IndexSet("col", size=4)
>>> x = model.add_variables(index_sets=[i, j], bounds=arco.Bounds(lower=0.0, upper=10.0))
>>> total = x.sum()
>>> type(total).__name__
'Expr'
>>> model.minimize(expr=total)
>>> solution = model.solve(log_to_console=False)
>>> round(solution.objective_value, 6)
0.0
```

## Element-wise addition

Adding two variable arrays of the same shape produces an `ExprArray` where
each element is the sum of the corresponding variables. This works with both
`VariableArray + VariableArray` and `VariableArray + ExprArray`.

```python doctest
>>> import arco
>>> import numpy as np
>>> model = arco.Model()
>>> units = arco.IndexSet("unit", size=3)
>>> supply = model.add_variables(index_sets=[units], bounds=arco.Bounds(lower=0.0, upper=50.0))
>>> backup = model.add_variables(index_sets=[units], bounds=arco.Bounds(lower=0.0, upper=20.0))
>>> combined = supply + backup
>>> type(combined).__name__
'ExprArray'
>>> combined.shape
(3,)
>>> _ = model.add_constraints(expr=combined >= [40.0, 30.0, 25.0])
>>> model.minimize(expr=np.sum(combined))
>>> solution = model.solve(log_to_console=False)
>>> round(solution.objective_value, 6)
95.0
```

## Diagonal extraction and flipping

For 2D variable arrays, `np.diag` extracts the k-th diagonal and `np.fliplr`
reverses the column order. Both return `ExprArray` objects.

```python doctest
>>> import arco
>>> import numpy as np
>>> model = arco.Model()
>>> rows = arco.IndexSet("row", size=3)
>>> cols = arco.IndexSet("col", size=3)
>>> x = model.add_variables(index_sets=[rows, cols], bounds=arco.Bounds(lower=0.0, upper=10.0))
>>> diag = np.diag(x)
>>> diag.shape
(3,)
>>> diag_above = np.diag(x, k=1)
>>> diag_above.shape
(2,)
>>> flipped = np.fliplr(x)
>>> flipped.shape
(3, 3)
```

These operations are useful for assignment and scheduling problems where
diagonal or anti-diagonal constraints appear naturally.

---

[How-to Guides](./) | [Docs home](../)
