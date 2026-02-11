# Building Optimization Models

This guide covers the core operations for constructing linear and mixed-integer optimization problems with arco.

## Variables

Use `model.add_variable()` to create decision variables with bounds, integrality, or binary restrictions.

### Add a continuous variable

Use `arco.Bounds` to set lower and upper limits on a continuous decision variable.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0))
>>> model.minimize(expr=x)
>>> solution = model.solve(log_to_console=False)
>>> solution.status
SolutionStatus.OPTIMAL
>>> round(solution.get_primal(index=x), 6)
0.0
```

### Add an integer variable

Pass `is_integer=True` to restrict a variable to integer values within its bounds.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> y = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0), is_integer=True)
>>> model.minimize(expr=y)
>>> solution = model.solve(log_to_console=False)
>>> solution.status
SolutionStatus.OPTIMAL
>>> round(solution.get_primal(index=y), 6)
0.0
```

### Add a binary variable

Pass `is_binary=True` to create a variable that takes only the values 0 or 1.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> z = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=1.0), is_binary=True)
>>> model.maximize(expr=z)
>>> solution = model.solve(log_to_console=False)
>>> solution.status
SolutionStatus.OPTIMAL
>>> round(solution.get_primal(index=z), 6)
1.0
```

## Expressions

Use Python arithmetic operators to combine variables into linear expressions that serve as objectives or constraint bodies.

### Build linear expressions

Multiply variables by scalar coefficients and add them together to form a linear expression. The expression can then serve as an objective or as the left-hand side of a constraint.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0))
>>> y = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0))
>>> cost = 2.0 * x + 3.0 * y
>>> _ = model.add_constraint(expr=x + y >= 6.0)
>>> model.minimize(cost)
>>> solution = model.solve(log_to_console=False)
>>> round(solution.objective_value, 6)
12.0
```

## Constraints

Use `model.add_constraint()` with comparison operators or explicit bounds to restrict feasible solutions.

### Add a >= constraint

Use the `>=` operator to create a lower-bound constraint on an expression.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0))
>>> y = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0))
>>> model.add_constraint(expr=x + y >= 5.0, name="floor")
Constraint('floor', Bounds(5, inf))
>>> model.minimize(expr=x + y)
>>> solution = model.solve(log_to_console=False)
>>> round(solution.objective_value, 6)
5.0
```

### Add a <= constraint

Use the `<=` operator to create an upper-bound constraint on an expression.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0))
>>> y = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0))
>>> model.add_constraint(expr=x + y <= 8.0, name="ceiling")
Constraint('ceiling', Bounds(-inf, 8))
>>> model.maximize(expr=x + y)
>>> solution = model.solve(log_to_console=False)
>>> round(solution.objective_value, 6)
8.0
```

### Add an == constraint

Use the `==` operator to fix an expression to an exact value.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0))
>>> y = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0))
>>> model.add_constraint(expr=x + y == 6.0, name="balance")
Constraint('balance', Bounds(6, 6))
>>> model.minimize(expr=x)
>>> solution = model.solve(log_to_console=False)
>>> round(solution.get_primal(index=y), 6)
6.0
```

### Add a range constraint

Pass a `bounds` argument instead of a comparison operator to constrain an expression within a range.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0))
>>> model.add_constraint(
...     expr=x,
...     bounds=arco.Bounds(lower=3.0, upper=7.0),
...     name="range",
... )
Constraint('range', Bounds(3, 7))
>>> model.minimize(expr=x)
>>> solution = model.solve(log_to_console=False)
>>> round(solution.get_primal(index=x), 6)
3.0
```

## Objectives

Use `model.minimize()` or `model.maximize()` to set the optimization direction.

### Minimize an objective

Use `model.minimize()` to find the smallest feasible value of an expression.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=2.0, upper=8.0))
>>> model.minimize(expr=x)
>>> solution = model.solve(log_to_console=False)
>>> solution.status
SolutionStatus.OPTIMAL
>>> round(solution.objective_value, 6)
2.0
```

### Maximize an objective

Use `model.maximize()` to find the largest feasible value of the same expression.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=2.0, upper=8.0))
>>> model.maximize(expr=x)
>>> solution = model.solve(log_to_console=False)
>>> solution.status
SolutionStatus.OPTIMAL
>>> round(solution.objective_value, 6)
8.0
```

## Solving

Use `model.solve()` to hand the model to the solver and inspect the results.

### Read variable values

Use `solution.get_value()` to retrieve the optimal value of a variable by
passing the variable object directly.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=5.0), name="x")
>>> y = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=5.0), name="y")
>>> model.add_constraint(expr=x + y <= 8.0, name="capacity")
Constraint('capacity', Bounds(-inf, 8))
>>> model.maximize(expr=2.0 * x + 3.0 * y)
>>> solution = model.solve(log_to_console=False)
>>> round(solution.get_value(x), 6)
3.0
>>> round(solution.get_value(y), 6)
5.0
```

### Retrieve dual values and reduced costs

Use `solution.get_dual()` for the shadow price of a constraint and
`solution.get_reduced_cost()` for the reduced cost of a variable.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=5.0), name="x")
>>> y = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=5.0), name="y")
>>> con = model.add_constraint(expr=x + y <= 8.0, name="capacity")
>>> model.maximize(expr=2.0 * x + 3.0 * y)
>>> solution = model.solve(log_to_console=False)
>>> round(solution.get_dual(con), 6)
2.0
>>> round(solution.get_reduced_cost(y), 6)
1.0
```

### Warm start

Supply an initial feasible point via `primal_start` to give the solver a head start.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0))
>>> y = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0))
>>> model.minimize(expr=x + 2.0 * y)
>>> solution = model.solve(
...     log_to_console=False,
...     primal_start=[(int(x), 2.0), (int(y), 1.0)],
... )
>>> solution.status
SolutionStatus.OPTIMAL
```

---

[How-to Guides](./) | [Docs home](../)
