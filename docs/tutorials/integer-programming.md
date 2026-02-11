# Integer Programming

This tutorial shows you how to add integer and binary decision variables to an
arco model.

## What is integer programming?

Linear programs allow variables to take any real value within their bounds. In
many real-world problems, however, the decision is discrete: you either build a
factory or you don't, you assign a whole number of trucks to a route, or you
select a subset of items to pack. Integer programming extends linear programming
by restricting some or all variables to take only integer values. When a model
contains at least one integer variable alongside continuous ones, it is called a
mixed-integer program (MIP).

## Integer variables

An integer variable can take any whole-number value within its bounds. You create
one by passing `is_integer=True` to `model.add_variable()`. The bounds still
apply, but the solver will only consider integer-valued solutions for that
variable.

In the example below, we create a single integer variable bounded between 0 and
10, then minimize it. Because the variable is integer-constrained and its lower
bound is 0, the optimal value is exactly 0.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> y = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0), is_integer=True)
>>> model.minimize(expr=y)
>>> solution = model.solve(log_to_console=False)
>>> solution.is_optimal()
True
>>> round(solution.get_primal(index=y), 6)
0.0
```

Without `is_integer=True`, the variable would be continuous and the solver might
return any floating-point value at the lower bound. With it set, the solver
guarantees the result is a whole number. This matters more when constraints push
the optimal continuous solution to a fractional value -- the solver will round to
the nearest feasible integer.

## Binary variables

A binary variable is a special case of an integer variable that can only be 0 or
1. These are the workhorses of combinatorial optimization: they represent yes/no
decisions. You create one by setting `is_binary=True` along with bounds of 0 to
1.

```python
b = model.add_variable(
    bounds=arco.Bounds(lower=0.0, upper=1.0),
    is_binary=True,
    name="select_item",
)
```

Arco marks a binary variable as both integer and binary internally, so you do
not need to pass `is_integer=True` separately.

## The knapsack problem

The classic knapsack problem demonstrates binary variables in action. You have a
set of items, each with a value and a weight, and a knapsack with a fixed weight
capacity. The goal is to choose the subset of items that maximizes total value
without exceeding the capacity.

```math
\max \sum_i v_i x_i \quad \text{s.t.} \quad \sum_i w_i x_i \leq W, \quad x_i \in \{0, 1\}
```

Each decision variable x_i is binary: 1 means the item is packed, 0 means it is
left behind. The constraint ensures the total weight stays within the capacity W.

Consider three items with the following data:

| Item | Value | Weight |
|------|-------|--------|
| a    | 6     | 3      |
| b    | 5     | 2      |
| c    | 4     | 1      |

The knapsack capacity is 4. The best combination is items a and c: their
combined weight is 3 + 1 = 4, exactly filling the knapsack, for a total value
of 6 + 4 = 10.

Let's build this model. We loop over the items, creating a binary variable for
each one. Then we construct the weight constraint and the value objective using
Python's built-in `sum()`, which works with arco expressions thanks to operator
overloading.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> items = ["a", "b", "c"]
>>> values = [6.0, 5.0, 4.0]
>>> weights = [3.0, 2.0, 1.0]
>>> capacity = 4.0
>>> x = {}
>>> for i, item in enumerate(items):
...     x[item] = model.add_variable(
...         bounds=arco.Bounds(lower=0.0, upper=1.0),
...         is_binary=True,
...         name=item,
...     )
>>> weight_expr = sum(weights[i] * x[item] for i, item in enumerate(items))
>>> model.add_constraint(expr=weight_expr <= capacity, name="capacity")
Constraint('capacity', Bounds(-inf, 4))
>>> value_expr = sum(values[i] * x[item] for i, item in enumerate(items))
>>> model.maximize(expr=value_expr)
>>> solution = model.solve(log_to_console=False)
>>> solution.is_optimal()
True
>>> round(solution.objective_value, 6)
10.0
```

The solver picks items a and c (total weight 4, total value 10), leaving item b
behind. Picking all three would exceed the capacity (3 + 2 + 1 = 6 > 4), so
the solver finds the most valuable subset that fits.

> [!NOTE]
> The `get_primal` method returns floating-point values even for binary
> variables. A result like 0.9999999 means the solver selected that item. Always
> round before interpreting binary results.

## BoundType shortcuts

Writing `arco.Bounds(lower=0.0, upper=float("inf"))` for every non-negative
variable gets repetitive. Arco provides `BoundType` shortcuts that bundle the
bounds, integrality, and binary flags into a single token. They are available
directly on the `arco` module and can be passed to the `bounds` parameter.

The most commonly used values are:

| Shortcut                           | Bounds             | Integer | Binary |
|------------------------------------|--------------------|---------|--------|
| `arco.NonNegativeFloat`  | [0, +inf)          | no      | no     |
| `arco.NonNegativeInt`    | [0, +inf)          | yes     | no     |
| `arco.Binary`            | [0, 1]             | yes     | yes    |
| `arco.PositiveFloat`     | (0, +inf)          | no      | no     |
| `arco.PositiveInt`       | [1, +inf)          | yes     | no     |
| `arco.NonPositiveFloat`  | (-inf, 0]          | no      | no     |
| `arco.NonPositiveInt`    | (-inf, 0]          | yes     | no     |
| `arco.NegativeFloat`     | (-inf, 0)          | no      | no     |
| `arco.NegativeInt`       | (-inf, -1]         | yes     | no     |

When you use a `BoundType`, you do not need to pass `is_integer` or `is_binary`
separately -- the shortcut carries that information. This makes the model
definition shorter and less error-prone.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
>>> y = model.add_variable(bounds=arco.Binary, name="y")
>>> model.minimize(expr=x + y)
>>> solution = model.solve(log_to_console=False)
>>> solution.is_optimal()
True
>>> round(solution.objective_value, 6)
0.0
```

Both variables take their minimum value: x is 0.0 (a non-negative float) and y
is 0 (a binary variable). The objective is 0.0 because the solver minimizes the
sum and neither variable is pushed up by any constraint.

Using `arco.Binary` is equivalent to writing
`arco.Bounds(lower=0.0, upper=1.0), is_binary=True`, and
`arco.NonNegativeFloat` is equivalent to
`arco.Bounds(lower=0.0, upper=float("inf"))`. The shortcuts are purely
syntactic sugar -- the resulting model is identical.

## Next steps

You now know how to create integer and binary variables, formulate a
combinatorial problem, and use `BoundType` shortcuts to keep your model
definitions concise.

---

&larr; [Your First Model](your-first-model.md) | [Tutorials](./) | Next: [Indexed Models](indexed-models.md) &rarr;
