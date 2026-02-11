# Arco Documentation

Arco is an optimization library for linear and mixed-integer programming on
constrained hardware. It ships as a Python package backed by a Rust core, with
the HiGHS solver embedded so there is nothing extra to install. If your problem
fits in memory, arco will solve it; the library is designed to be direct about
resource limits rather than silently degrading.
The name stands for **Assembled Resource-Constrained Optimization**.

## Quickstart

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=float("inf")), name="x")
>>> y = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=float("inf")), name="y")
>>> model.add_constraint(x + y >= 10.0, name="demand")
Constraint('demand', Bounds(10, inf))
>>> model.minimize(3.0 * x + 2.0 * y)
>>> solution = model.solve(log_to_console=False)
>>> solution.is_optimal()
True
>>> round(solution.objective_value, 6)
20.0
```

> [!NOTE]
> Arco embeds the HiGHS solver. No external solver installation is needed.

## [Tutorials](tutorials/)

Step-by-step walkthroughs that build your understanding from a simple LP to
multi-stage block composition.

- [Your First Model](tutorials/your-first-model.md) -- build and solve a two-variable linear program.
- [Integer Programming](tutorials/integer-programming.md) -- add integer and binary decision variables to a model.
- [Indexed Models](tutorials/indexed-models.md) -- use index sets and variable arrays for structured problems.
- [Block Composition](tutorials/block-composition.md) -- compose multi-stage optimization workflows from reusable blocks.

## [How-to Guides](how-to/)

Task-oriented recipes for common operations. Each guide gets straight to the
point with self-contained examples you can copy and run.

- [Building Optimization Models](how-to/building-optimization-models.md) -- variables, constraints, objectives, and solving in recipe form.
- [Define Block Schemas](how-to/block-schemas.md) -- define typed block input/output contracts for block composition.
- [Numpy Integration](how-to/numpy-integration.md) -- array arithmetic, element-wise bounds, indexing, and reduction operators.
- [Configure Solver](how-to/configure-solver.md) -- solver objects, settings, and reusable configurations.
- [Debug Infeasibility](how-to/debug-infeasibility.md) -- use slacks and elastic constraints to diagnose infeasible models.
- [Inspect a Model](how-to/inspect-model.md) -- examine model structure with snapshots before and after solving.
- [Handle Errors](how-to/handle-errors.md) -- catch model-building exceptions and check solver outcomes.
- [Enable Logging](how-to/enable-logging.md) -- configure Rust-side tracing and solver output.

## [Explanation](explanation/)

Background material and design decisions that help you understand why arco works
the way it does.

- [Why Arco](explanation/why-arco.md) -- motivation and design philosophy behind the library.
- [Architecture](explanation/architecture.md) -- how the Rust crates and Python bindings fit together.
- [Core Concepts](explanation/core-concepts.md) -- variables, constraints, expressions, and the model lifecycle.

## [Reference](reference/)

- [API Reference](reference/) -- type stubs, classes, and method signatures for the full Python API.
- [Known Problems](reference/known-problems.md) -- canonical markdown catalog of default optimization problems for benchmarks and regression discussions.
