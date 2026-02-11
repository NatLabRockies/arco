# Indexed Models

This tutorial teaches you how to build structured, multi-dimensional
optimization models using IndexSets and variable arrays. By the end you will
be able to define named dimensions for your problem, create arrays of variables
over those dimensions, and loop over them to build constraints and objectives.

## IndexSets

Real optimization models rarely consist of a handful of named scalars. They
operate over time periods, locations, technologies, or products -- dimensions
that give the model its shape. In arco, an `IndexSet` represents one of these
named dimensions.

There are two ways to create an IndexSet. If you only care about the size of
the dimension, pass a `size` argument. This is useful when the elements are
anonymous -- for instance, time steps numbered 0, 1, 2.

If the elements have meaningful names, pass a `members` list instead. The size
is inferred from the length of the list, and you can later iterate over the
member names to look up data in dictionaries or dataframes.

```python doctest
>>> import arco
>>> times = arco.IndexSet(name="T", size=3)
>>> times.size
3
>>> techs = arco.IndexSet(name="tech", members=["solar", "wind", "gas"])
>>> techs.size
3
>>> techs.members
['solar', 'wind', 'gas']
```

The `name` argument is a label for the dimension. It does not affect the
mathematics, but it shows up in diagnostic output and makes multi-dimensional
models easier to read.

## Variable arrays

Once you have index sets, you can create a whole grid of variables in a single
call. Pass a list of IndexSets to `model.add_variables()` (note the plural),
and arco returns a `VariableArray` whose shape matches the Cartesian product of
the index sets.

The `shape` attribute tells you the dimensions of the array, and the `sum()`
method returns a linear expression that sums every variable in the array. This
is handy for quick objectives or aggregate constraints.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> T = arco.IndexSet(name="T", size=2)
>>> G = arco.IndexSet(name="G", members=["solar", "wind", "gas"])
>>> gen = model.add_variables(
...     index_sets=[T, G],
...     bounds=arco.Bounds(lower=0.0, upper=100.0),
... )
>>> gen.shape
(2, 3)
>>> model.minimize(expr=gen.sum())
>>> solution = model.solve(log_to_console=False)
>>> solution.status
SolutionStatus.OPTIMAL
```

The solver found the trivial optimum where every variable sits at its lower
bound. The important thing is the structure: two time periods times three
generators gives a 2-by-3 array of decision variables, all created and bounded
in one line.

## Economic dispatch

Let's put everything together in a realistic example. You manage a small power
system with three generators -- solar, wind, and gas -- over two time periods.
Each generator has a maximum capacity and a per-unit cost. Each time period has
a demand that must be met by the combined output of all generators. The goal is
to minimize total generation cost.

The mathematical formulation is:

```math
\min \sum_{t,g} c_g \cdot p_{t,g} \quad \text{s.t.} \quad \sum_g p_{t,g} \geq D_t, \quad 0 \leq p_{t,g} \leq \bar{P}_g
```

Here are the data. Solar can produce up to 50 MW, wind up to 80 MW, and gas up
to 100 MW. Solar and wind have zero marginal cost (the fuel is free), while gas
costs 30 per MW. Demand is 120 MW in the first period and 90 MW in the second.

The first step is to create the model and the index sets, then call
`add_variables` to get the generation array. Each variable represents the
output of one generator in one time period, bounded between 0 and a placeholder
upper bound of 100. We will tighten the upper bounds to the actual capacities
using constraints.

Arco provides array-level operations that replace explicit loops. The
`add_constraints` method (plural) accepts comparisons on whole arrays.
The `sum(over=...)` method on a `VariableArray` reduces along a named
dimension, returning an `ExprArray` you can constrain directly.

The capacity constraints apply to every (time, generator) pair. Because `gen`
is stored flat in row-major order, repeating the per-generator capacities
`T.size` times produces the right-hand side vector. The demand constraints
use `gen.sum(over=G)` to sum across generators for each time period, then
compare the resulting `ExprArray` against the demand list. The objective
zips per-unit costs with the flattened variables and sums the products.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> T = arco.IndexSet(name="T", size=2)
>>> G = arco.IndexSet(name="G", members=["solar", "wind", "gas"])
>>> capacity = {"solar": 50.0, "wind": 80.0, "gas": 100.0}
>>> cost = {"solar": 0.0, "wind": 0.0, "gas": 30.0}
>>> demand = [120.0, 90.0]
>>>
>>> gen = model.add_variables(
...     index_sets=[T, G],
...     bounds=arco.Bounds(lower=0.0, upper=100.0),
... )
>>>
>>> caps = [capacity[g] for g in G.members] * T.size
>>> _ = model.add_constraints(expr=gen <= caps)
>>>
>>> _ = model.add_constraints(expr=gen.sum(over=G) >= demand)
>>>
>>> costs = [cost[g] for g in G.members] * T.size
>>> obj = sum(c * v for c, v in zip(costs, gen.flatten()))
>>> model.minimize(expr=obj)
>>>
>>> solution = model.solve(log_to_console=False)
>>> solution.status
SolutionStatus.OPTIMAL
>>> round(solution.objective_value, 6)
0.0
```

The solver dispatches the free generators (solar and wind) first. Their
combined capacity of 130 MW exceeds both the 120 MW and 90 MW demand periods,
so gas is never needed and the total cost is zero.

> [!NOTE]
> The array operations scale naturally. For larger problems you would typically
> load data from dictionaries or dataframes and build the right-hand side
> vectors programmatically. The index sets grow, the data gets richer, but the
> code shape stays the same.

---

&larr; [Integer Programming](integer-programming.md) | [Tutorials](./) | Next: [Block Composition](block-composition.md) &rarr;
