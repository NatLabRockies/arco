# Debug Infeasibility

When a model is infeasible the solver cannot find a solution that satisfies all
constraints simultaneously. This guide shows how to detect infeasibility and use
slacks and elastic constraints to identify which constraints are causing the
problem.

## Detecting infeasibility

The simplest way to discover infeasibility is to solve the model and observe
that the solver raises an exception. Consider a model where a variable is
bounded between 0 and 1, yet a constraint demands it be at least 5. These
requirements are contradictory, and the solver will tell you so.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=1.0))
>>> _ = model.add_constraint(expr=x >= 5.0)
>>> model.minimize(expr=x)
>>> solution = model.solve(log_to_console=False)
>>> solution.is_infeasible()
True
>>> solution.status
SolutionStatus.INFEASIBLE
```

The status confirms the model is infeasible, but it does not tell you which
constraint is responsible. In a model with hundreds of constraints you need a
more targeted approach.

## Using slack variables

A slack variable allows a single constraint to be violated at a cost. You attach
a slack to a suspect constraint and give it a large penalty in the objective. If
the solver activates the slack, that constraint was contributing to the
infeasibility.

Set the objective before adding slacks, because the penalty term is incorporated
into the existing objective expression. Pass the constraint, the bound side you
want to relax, and the penalty cost to `model.add_slack()`.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=2.0))
>>> con = model.add_constraint(expr=x >= 5.0, name="difficult")
>>> model.minimize(expr=x)
>>> slack = model.add_slack(
...     constraint=con,
...     bound="lower",
...     penalty=1000.0,
...     name="slack_difficult",
... )
>>> solution = model.solve(log_to_console=False)
>>> solution.status
SolutionStatus.OPTIMAL
>>> round(solution.get_primal(index=x), 6)
2.0
```

The solver found an optimal solution by violating the `"difficult"` constraint.
The variable `x` sits at its upper bound of 2 rather than the required 5,
confirming that this constraint is the source of the conflict.

> [!NOTE]
> Start with a high penalty so the solver only activates the slack when strictly
> necessary. A penalty that is too low may cause the solver to prefer violation
> over satisfying the constraint even in a feasible model.

## Bulk slacks

When you suspect several constraints, `model.add_slacks()` (plural) relaxes
them all at once with the same penalty. It accepts a list of constraints and
returns a list of `SlackVariable` objects.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0))
>>> c1 = model.add_constraint(expr=x >= 20.0, name="target_a")
>>> c2 = model.add_constraint(expr=x >= 15.0, name="target_b")
>>> model.minimize(expr=x)
>>> slacks = model.add_slacks([c1, c2], bound="lower", penalty=1000.0)
>>> len(slacks)
2
>>> solution = model.solve(log_to_console=False)
>>> solution.status
SolutionStatus.OPTIMAL
>>> round(solution.get_primal(index=0), 6)
10.0
```

Both constraints are violated (the variable cannot exceed its upper bound of
10), confirming they both contribute to the infeasibility.

## Elastic constraints

Sometimes a constraint needs flexibility in both directions. An equality
constraint, for example, may be impossible to satisfy exactly, and you want to
know whether the solution falls above or below the target. The
`model.make_elastic()` method relaxes a constraint on both sides at once, with
separate penalties for upward and downward violation.

As with slacks, set the objective before making a constraint elastic.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0))
>>> con = model.add_constraint(expr=x == 5.0, name="target")
>>> model.minimize(expr=x)
>>> elastic = model.make_elastic(
...     constraint=con,
...     upper_penalty=100.0,
...     lower_penalty=50.0,
...     name="elastic_target",
... )
>>> solution = model.solve(log_to_console=False)
>>> solution.status
SolutionStatus.OPTIMAL
```

The asymmetric penalties let you express a preference: here, violating the
constraint downward costs less than violating it upward. The solver will choose
the direction of violation that minimizes total cost.

> [!WARNING]
> Making every constraint elastic at once can mask the real source of
> infeasibility. Add elasticity to a small group of suspect constraints first,
> solve, and inspect which elastic variables are active before expanding the
> search.

---

[How-to Guides](./) | [Docs home](../)
