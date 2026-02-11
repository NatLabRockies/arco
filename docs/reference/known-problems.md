# Known Problems

This reference page defines Arco's canonical set of known optimization problems.
Each section includes:

- Mathematical formulation.
- A runnable Arco model.
- A known solution check in `python doctest` format.

## Simple LP

```math
\begin{align}
\min_{x,y} \quad & 3x + 2y \\
\text{subject to} \quad & x \geq 1,\; y \geq 2,\; x + y \geq 5
\end{align}
```

Known optimum: `x = 1`, `y = 4`, objective `11`.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=1.0, upper=float("inf")), name="x")
>>> y = model.add_variable(bounds=arco.Bounds(lower=2.0, upper=float("inf")), name="y")
>>> _ = model.add_constraint(expr=x + y >= 5.0, name="demand")
>>> model.minimize(expr=3.0 * x + 2.0 * y)
>>> solution = model.solve(log_to_console=False)
>>> solution.is_optimal()
True
>>> round(solution.objective_value, 6)
11.0
>>> round(solution.get_primal(index=x), 6)
1.0
>>> round(solution.get_primal(index=y), 6)
4.0
```

## Production Mix

```math
\begin{align}
\max_{x,y \ge 0} \quad & 40x + 30y \\
\text{subject to} \quad & x \le 40 \\
& x + y \le 80 \\
& 2x + y \le 100
\end{align}
```

Known optimum: `x = 20`, `y = 60`, objective `2600`.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.NonNegativeFloat, name="x")
>>> y = model.add_variable(bounds=arco.NonNegativeFloat, name="y")
>>> _ = model.add_constraint(expr=x <= 40.0, name="demand")
>>> _ = model.add_constraint(expr=x + y <= 80.0, name="labor_a")
>>> _ = model.add_constraint(expr=2.0 * x + y <= 100.0, name="labor_b")
>>> model.maximize(expr=40.0 * x + 30.0 * y)
>>> solution = model.solve(log_to_console=False)
>>> solution.is_optimal()
True
>>> round(solution.objective_value, 6)
2600.0
>>> round(solution.get_value(x), 6)
20.0
>>> round(solution.get_value(y), 6)
60.0
```

## $A x = b$ Feasibility

```math
\begin{align}
\text{find} \quad & x \ge 0 \\
\text{subject to} \quad & A x = b
\end{align}
```

Using:

```math
A=\begin{bmatrix}1 & 1 \\ 1 & 0\end{bmatrix},\quad b=\begin{bmatrix}3 \\ 1\end{bmatrix}
```

Known solution: `x = [1, 2]`.

```python doctest
>>> import numpy as np
>>> import arco
>>> A = np.array([[1.0, 1.0], [1.0, 0.0]], dtype=float)
>>> b = np.array([3.0, 1.0], dtype=float)
>>> model = arco.Model()
>>> j = arco.IndexSet(name="j", size=2)
>>> x = model.add_variables(index_sets=[j], bounds=arco.NonNegativeFloat, name="x")
>>> _ = model.add_constraints(expr=(A @ x) == b, name="row")
>>> model.minimize(expr=0.0)
>>> solution = model.solve(log_to_console=False)
>>> solution.is_optimal()
True
>>> tuple(float(v) for v in np.round(solution.get_value(x), 6))
(1.0, 2.0)
```

## 0-1 Knapsack

```math
\begin{align}
\max \quad & \sum_i v_i x_i \\
\text{subject to} \quad & \sum_i w_i x_i \le 4 \\
& x_i \in \{0,1\}
\end{align}
```

With values `[6,5,4]` and weights `[3,2,1]`, the known optimum value is `10`
(select items 1 and 3).

```python doctest
>>> import arco
>>> items = ["a", "b", "c"]
>>> values = [6.0, 5.0, 4.0]
>>> weights = [3.0, 2.0, 1.0]
>>> model = arco.Model()
>>> x = {name: model.add_variable(bounds=arco.Binary, name=name) for name in items}
>>> _ = model.add_constraint(
...     expr=sum(w * x[name] for w, name in zip(weights, items, strict=True)) <= 4.0,
...     name="capacity",
... )
>>> model.maximize(expr=sum(v * x[name] for v, name in zip(values, items, strict=True)))
>>> solution = model.solve(log_to_console=False)
>>> solution.is_optimal()
True
>>> round(solution.objective_value, 6)
10.0
>>> tuple(round(solution.get_value(x[name]), 6) for name in items)
(1.0, 0.0, 1.0)
```

## Network Flow (numpy integration)

```math
\begin{align}
\min \quad & \sum_{i,j} c_{ij} x_{ij} \\
\text{subject to} \quad
& \sum_j x_{ij} - \sum_j x_{ji} = b_i \quad \forall i \\
& x_{ij} \in \{0,1\}
\end{align}
```

Known optimum on the small test graph: objective `2` using the direct arc
`0 \to 1`.

```python doctest
>>> import numpy as np
>>> import arco
>>> costs = np.array([[0.0, 2.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]])
>>> balance = np.array([1.0, -1.0, 0.0])
>>> model = arco.Model()
>>> src = arco.IndexSet(name="src", size=3)
>>> dst = arco.IndexSet(name="dst", size=3)
>>> x = model.add_variables(index_sets=[src, dst], bounds=arco.Binary, name="x")
>>> _ = model.add_constraints(expr=x[costs == 0.0] == 0.0, name="no_arc")
>>> _ = model.add_constraints(expr=(x.sum(over=dst) - x.sum(over=src)) == balance, name="flow")
>>> model.minimize(expr=np.sum(costs * x))
>>> solution = model.solve(log_to_console=False)
>>> solution.is_optimal()
True
>>> round(solution.objective_value, 6)
2.0
>>> round(solution.get_value(x[0, 1]), 6)
1.0
>>> abs(solution.get_value(x[0, 2])) < 1e-6
True
```

## Unit Commitment

```math
\begin{align}
\min_{g,u} \quad & \sum_i \left(F_i u_i + V_i g_i\right) \\
\text{subject to} \quad & \sum_i g_i = D \\
& 0 \le g_i \le \bar{G}_i u_i \\
& u_i \in \{0,1\}
\end{align}
```

Known optimum for this two-unit instance: objective `275`, generation
`[100, 20]`, commitment `[1, 1]`.

```python doctest
>>> import numpy as np
>>> import arco
>>> gen_max = np.array([100.0, 80.0])
>>> fixed_cost = np.array([10.0, 5.0])
>>> variable_cost = np.array([2.0, 3.0])
>>> demand = 120.0
>>> model = arco.Model()
>>> units = arco.IndexSet(name="unit", size=2)
>>> g = model.add_variables(index_sets=[units], bounds=arco.NonNegativeFloat, name="gen")
>>> u = model.add_variables(index_sets=[units], bounds=arco.Binary, name="commit")
>>> _ = model.add_constraint(expr=g.sum() == demand, name="balance")
>>> _ = model.add_constraints(expr=g <= gen_max * u, name="capacity")
>>> model.minimize(expr=np.sum(fixed_cost * u + variable_cost * g))
>>> solution = model.solve(log_to_console=False)
>>> solution.is_optimal()
True
>>> round(solution.objective_value, 6)
275.0
>>> tuple(float(v) for v in np.round(solution.get_value(g), 6))
(100.0, 20.0)
>>> tuple(float(v) for v in np.round(solution.get_value(u), 6))
(1.0, 1.0)
```

## Network Design

```math
\begin{align}
\min_{x,y} \quad & 0.1\sum_{i,j} x_{ij} - \sum_j y_{0j} \\
\text{subject to} \quad & y_{ij} \le G_{ij} x_{ij} \\
& \sum_j y_{ij} = \sum_j y_{ji}\quad \forall i \in \text{internal nodes} \\
& \sum_{i,j} x_{ij} \le 2 \\
& x_{ij} \in \{0,1\},\; y_{ij}\ge 0
\end{align}
```

Known optimum for this toy graph: select edges `0->1` and `1->3`, objective
`-4.8`.

```python doctest
>>> import numpy as np
>>> import arco
>>> G = np.array([
...     [0.0, 5.0, 4.0, 0.0],
...     [0.0, 0.0, 0.0, 5.0],
...     [0.0, 0.0, 0.0, 4.0],
...     [0.0, 0.0, 0.0, 0.0],
... ])
>>> model = arco.Model()
>>> src = arco.IndexSet(name="src", size=4)
>>> dst = arco.IndexSet(name="dst", size=4)
>>> edge = model.add_variables(index_sets=[src, dst], bounds=arco.Binary, name="edge")
>>> flow = model.add_variables(index_sets=[src, dst], bounds=arco.NonNegativeFloat, name="flow")
>>> _ = model.add_constraint(expr=np.sum(edge) <= 2.0, name="edge_budget")
>>> _ = model.add_constraints(expr=flow <= G * edge, name="capacity")
>>> _ = model.add_constraints(
...     expr=flow[1:-1, :].sum(over=dst) == flow[:, 1:-1].sum(over=src),
...     name="flow_conservation",
... )
>>> model.minimize(expr=0.1 * np.sum(edge) - flow[0, :].sum())
>>> solution = model.solve(log_to_console=False)
>>> solution.is_optimal()
True
>>> round(solution.objective_value, 6)
-4.8
>>> solution.get_value(edge[0, 1]) > 0.9
True
>>> solution.get_value(edge[1, 3]) > 0.9
True
>>> abs(solution.get_value(edge[0, 2])) < 1e-6
True
```

## Energy Storage Dispatch

```math
\begin{align}
\min \quad & 10\sum_t p_t \\
\text{subject to} \quad & p_t + r_t + d_t = D_t + c_t \\
& s_t = s_{t-1} + c_t - d_t \\
& 0 \le r_t \le A_t,\; 0 \le p_t \le 10,\; 0 \le s_t \le 2 \\
& 0 \le c_t \le 1,\; 0 \le d_t \le 1
\end{align}
```

Known optimum for `T=2` with initial storage `2`: objective `40`, thermal
generation `[2,2]`.

```python doctest
>>> import numpy as np
>>> import arco
>>> T = 2
>>> demand = np.array([5.0, 5.0])
>>> available = np.array([2.0, 2.0])
>>> initial_storage = 2.0
>>> model = arco.Model()
>>> time = arco.IndexSet(name="t", size=T)
>>> r = model.add_variables(index_sets=[time], bounds=arco.NonNegativeFloat, name="renewable")
>>> p = model.add_variables(index_sets=[time], bounds=arco.Bounds(lower=0.0, upper=10.0), name="thermal")
>>> s = model.add_variables(index_sets=[time], bounds=arco.Bounds(lower=0.0, upper=2.0), name="storage")
>>> c = model.add_variables(index_sets=[time], bounds=arco.Bounds(lower=0.0, upper=1.0), name="charge")
>>> d = model.add_variables(index_sets=[time], bounds=arco.Bounds(lower=0.0, upper=1.0), name="discharge")
>>> _ = model.add_constraints(expr=p + r + d == demand + c, name="balance")
>>> s_prev = np.concatenate([[initial_storage], s[:-1]])
>>> _ = model.add_constraints(expr=s == s_prev + c - d, name="storage_dynamics")
>>> _ = model.add_constraints(expr=r <= available, name="renewable_cap")
>>> model.minimize(expr=10.0 * p.sum())
>>> solution = model.solve(log_to_console=False)
>>> solution.is_optimal()
True
>>> round(solution.objective_value, 6)
40.0
>>> tuple(float(v) for v in np.round(solution.get_value(p), 6))
(2.0, 2.0)
>>> tuple(float(v) for v in np.round(solution.get_value(d), 6))
(1.0, 1.0)
```

## N-Queens

```math
\begin{align}
\text{find}\quad & x_{ij} \in \{0,1\} \\
\text{subject to}\quad
& \sum_j x_{ij} = 1 \;\; \forall i \\
& \sum_i x_{ij} = 1 \;\; \forall j \\
& \sum_{i-j=k} x_{ij} \le 1 \;\; \forall k \\
& \sum_{i+j=k} x_{ij} \le 1 \;\; \forall k
\end{align}
```

For `N=4`, the model is feasible with objective `0`.

```python doctest
>>> import numpy as np
>>> import arco
>>> N = 4
>>> model = arco.Model()
>>> rows = arco.IndexSet(name="row", size=N)
>>> cols = arco.IndexSet(name="col", size=N)
>>> q = model.add_variables(index_sets=[rows, cols], bounds=arco.Binary, name="queen")
>>> _ = model.add_constraints(expr=q.sum(over=cols) == 1.0, name="row")
>>> _ = model.add_constraints(expr=q.sum(over=rows) == 1.0, name="col")
>>> for k in range(-(N - 1), N):
...     _ = model.add_constraint(expr=np.diag(q, k).sum() <= 1.0, name=f"diag_down_{k}")
>>> for k in range(-(N - 1), N):
...     _ = model.add_constraint(expr=np.diag(np.fliplr(q), k).sum() <= 1.0, name=f"diag_up_{k}")
>>> model.minimize(expr=0.0)
>>> solution = model.solve(log_to_console=False)
>>> board = solution.get_value(q).reshape(N, N)
>>> solution.is_optimal()
True
>>> round(solution.objective_value, 6)
0.0
>>> tuple(float(v) for v in np.round(np.sum(board, axis=0), 6))
(1.0, 1.0, 1.0, 1.0)
>>> tuple(float(v) for v in np.round(np.sum(board, axis=1), 6))
(1.0, 1.0, 1.0, 1.0)
>>> bool(max(np.diag(board, k).sum() for k in range(-(N - 1), N)) <= 1.0 + 1e-6)
True
>>> bool(max(np.diag(np.fliplr(board), k).sum() for k in range(-(N - 1), N)) <= 1.0 + 1e-6)
True
```

---

[Back to reference](./) | [Back to docs home](../)
