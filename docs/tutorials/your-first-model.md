# Your First Model

This tutorial walks you through building and solving a simple linear program
with arco.

## The problem

A small factory produces two products, X and Y. Each unit of X costs 3 to
produce and each unit of Y costs 2. The factory must produce at least 5 units in
total, and operational rules require at least 1 unit of X and at least 2 units
of Y. You want to find the production plan that minimizes cost.

Written as a linear program, the formulation looks like this:

```math
\min_{x,y} \quad 3x + 2y
```

```math
\text{subject to} \quad x + y \geq 5, \quad x \geq 1, \quad y \geq 2
```

The optimal solution is x = 1, y = 4, with an objective value of 11. Let's
build this model step by step.

## Creating a model

Everything in arco starts with a `Model`. A model is a container that holds
variables, constraints, and an objective. You create one by calling
`arco.Model()` with no arguments.

```python
import arco

model = arco.Model()
```

The model is empty at this point. It has no variables, no constraints, and no
objective. You will add each of these in the steps that follow.

## Adding variables with bounds

Decision variables represent the quantities you are solving for. Each variable
needs bounds that tell the solver what values it is allowed to take. You create
bounds with `arco.Bounds(lower=..., upper=...)` and pass them to
`model.add_variable()`.

In our problem, x must be at least 1 and y must be at least 2. Neither has an
upper limit, so we use `float("inf")` for the upper bound. We also give each
variable a name, which is optional but makes the model easier to debug.

```python
x = model.add_variable(
    bounds=arco.Bounds(lower=1.0, upper=float("inf")),
    name="x",
)
y = model.add_variable(
    bounds=arco.Bounds(lower=2.0, upper=float("inf")),
    name="y",
)
```

The `add_variable` call returns a `Variable` object. You will use these objects
to build expressions and constraints. The lower bounds on x and y already encode
two of our three constraints (x >= 1 and y >= 2), so only the combined demand
constraint remains.

> [!NOTE]
> Arco embeds the HiGHS solver. No external solver installation is needed.

## Adding constraints

The remaining constraint says the total production must be at least 5 units:
x + y >= 5. Arco uses Python's operator overloading to let you write constraints
in a natural mathematical style. The expression `x + y >= 5.0` produces a
`ConstraintExpr` that you pass to `model.add_constraint()`.

```python
_ = model.add_constraint(expr=x + y >= 5.0, name="demand")
```

The name `"demand"` is optional, but naming your constraints makes it much
easier to understand solver output and debug infeasible models later.

## Setting the objective

Every optimization model needs an objective: the expression you want to minimize
or maximize. You set a minimization objective with `model.minimize()`. Like
constraints, the objective expression uses operator overloading, so you can write
`3.0 * x + 2.0 * y` directly.

```python
model.minimize(expr=3.0 * x + 2.0 * y)
```

The model is now complete. It has two variables, one explicit constraint (plus
the variable bounds), and a minimization objective.

## Solving the model

Call `model.solve()` to hand the model to the solver. The method returns a
`SolveResult` that contains the solution status, the optimal objective value, and
the values of each variable. Pass `log_to_console=False` to suppress solver
output.

```python
solution = model.solve(log_to_console=False)
```

The first thing to check after solving is whether the solver found an optimal
solution. The `status` attribute returns a status string, and `is_optimal()`
gives you a convenient boolean check.

## Reading the results

Once you have confirmed the solution is optimal, you can read the objective
value and the value of each decision variable. Use `solution.objective_value`
for the objective and `solution.get_primal(index=var)` for individual variable
values. Because the solver works in floating point, it is good practice to round
results before comparing them.

Here is the complete model and solution check as a self-contained example:

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=1.0, upper=float("inf")), name="x")
>>> y = model.add_variable(bounds=arco.Bounds(lower=2.0, upper=float("inf")), name="y")
>>> model.add_constraint(expr=x + y >= 5.0, name="demand")
Constraint('demand', Bounds(5, inf))
>>> model.minimize(expr=3.0 * x + 2.0 * y)
>>> solution = model.solve(log_to_console=False)
>>> solution.status
SolutionStatus.OPTIMAL
>>> round(solution.objective_value, 6)
11.0
>>> round(solution.get_primal(index=x), 6)
1.0
>>> round(solution.get_primal(index=y), 6)
4.0
```

The solver found that producing 1 unit of X and 4 units of Y gives the minimum
cost of 11. This satisfies all three constraints: the total is 5 (meeting the
demand floor), x is at least 1, and y is at least 2.

## Inspecting the solution object

The `SolveResult` object has several methods for understanding what the solver
found. The `is_optimal()` method returns `True` when the solver proved
optimality. The `is_feasible()` method returns `True` when the solution
satisfies all constraints (which is always the case for an optimal solution).
The `status` attribute returns a `SolutionStatus` enum value describing the
solver outcome.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=1.0, upper=float("inf")), name="x")
>>> y = model.add_variable(bounds=arco.Bounds(lower=2.0, upper=float("inf")), name="y")
>>> model.add_constraint(expr=x + y >= 5.0, name="demand")
Constraint('demand', Bounds(5, inf))
>>> model.minimize(expr=3.0 * x + 2.0 * y)
>>> solution = model.solve(log_to_console=False)
>>> solution.is_optimal()
True
>>> solution.is_feasible()
True
>>> solution.status
SolutionStatus.OPTIMAL
>>> round(solution.objective_value, 6)
11.0
```

You now know how to create a model, define variables with bounds, add
constraints using operator overloading, set an objective, solve the model, and
read the results.

---

[Tutorials](./) | Next: [Integer Programming](integer-programming.md) &rarr;
