# Configure Solver

Arco separates model construction from solver configuration. You can build a
model once and solve it with different solver settings by swapping the solver
object passed to `model.solve()`. This guide shows how to create, customize,
and reuse solver configurations.

## Create a solver object

Use `arco.HiGHS(...)` to create a solver configuration with explicit settings.
Pass the object to `model.solve(solver=...)` to control how the solver behaves
during optimization.

```python doctest
>>> import arco
>>> solver = arco.HiGHS(time_limit=60.0, log_to_console=False)
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0))
>>> model.minimize(x)
>>> solution = model.solve(solver=solver)
>>> solution.status
SolutionStatus.OPTIMAL
```

When you do not need to customize anything beyond the defaults, `arco.Solver()`
creates a generic solver backed by HiGHS with default settings.

```python doctest
>>> import arco
>>> solver = arco.Solver(log_to_console=False)
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0))
>>> model.minimize(x)
>>> solution = model.solve(solver=solver)
>>> solution.status
SolutionStatus.OPTIMAL
```

## Adjust settings with copy

Use `solver.copy(update={...})` to create a new solver configuration that
inherits all settings from the original and overrides only the keys you
specify. The original solver is left unchanged.

```python doctest
>>> import arco
>>> solver = arco.HiGHS(time_limit=60.0, log_to_console=False)
>>> solver.time_limit
60.0
>>> fast = solver.copy(update={"time_limit": 10.0})
>>> fast.time_limit
10.0
>>> solver.time_limit
60.0
```

This is useful when you have a base configuration for production runs and want
a tighter variant for quick validation without duplicating every setting.

```python doctest
>>> import arco
>>> base = arco.HiGHS(time_limit=120.0, mip_gap=0.01, log_to_console=False)
>>> debug = base.copy(update={"time_limit": 5.0, "mip_gap": 0.05})
>>> debug.time_limit
5.0
>>> debug.mip_gap
0.05
>>> base.mip_gap
0.01
```

## Pass settings directly to solve

When you only need solver settings for a single call and do not plan to reuse
them, pass the settings as keyword arguments directly to `model.solve()`. This
avoids creating a separate solver object.

```python doctest
>>> import arco
>>> model = arco.Model()
>>> x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0))
>>> y = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0))
>>> _ = model.add_constraint(x + y >= 5.0)
>>> model.minimize(x + y)
>>> solution = model.solve(log_to_console=False, time_limit=60.0, mip_gap=0.01)
>>> solution.status
SolutionStatus.OPTIMAL
>>> round(solution.objective_value, 6)
5.0
```

This is equivalent to constructing an `arco.HiGHS(...)` and passing it via the
`solver` keyword, but more concise for one-off solves.

## Solver settings reference

All settings return `None` when not explicitly set, in which case the solver
backend uses its own default.

| Setting          | Type    | Description                                                       |
| ---------------- | ------- | ----------------------------------------------------------------- |
| `presolve`       | `bool`  | Enable or disable the presolve phase.                             |
| `threads`        | `int`   | Number of threads the solver may use.                             |
| `tolerance`      | `float` | Feasibility tolerance for primal and dual values.                 |
| `time_limit`     | `float` | Maximum wall-clock seconds the solver may run.                    |
| `mip_gap`        | `float` | Relative MIP optimality gap at which the solver stops.            |
| `verbosity`      | `int`   | Solver output verbosity level (backend-specific scale).           |
| `log_to_console` | `bool`  | Whether the solver prints progress to the console during a solve. |

> [!NOTE]
> Not every backend interprets every setting. If a setting does not apply to the
> chosen backend it is silently ignored.
>
> `time_limit`, `mip_gap`, and `tolerance` must be finite and non-negative.
> `threads` must be at least `1`.

## Xpress (LP / MIP solver)

The Xpress backend is available when Arco is built with the `xpress` feature flag.
Xpress supports LP, MIP, and QP problems.

> [!IMPORTANT]
> Building with Xpress requires the FICO Xpress Optimizer SDK to be installed
> on the system. Set the `XPRESSDIR` environment variable to your Xpress
> installation directory before building.

### Install Xpress

Download the FICO Xpress Community Edition from
[fico.com](https://www.fico.com/en/products/fico-xpress-optimization). The
community edition is free and supports models up to ~5000 variables/constraints.

| Platform | Typical `XPRESSDIR`                                   |
| -------- | ----------------------------------------------------- |
| macOS    | `~/User Apps/FICO Xpress/xpressmp` or `/Applications/FICO Xpress/xpressmp` |
| Linux    | `/opt/xpressmp`                                       |

The installer generates two license files in `$XPRESSDIR/bin/`:

- `community-xpauth.xpr` — community license (`hostid="any"`, works on any machine)
- `xpauth.xpr` — commercial license (tied to a specific machine)

Arco tries the community license first, then falls back to the commercial one.
If you have a commercial license and want to force it, set `XPAUTH_PATH` to its
full path.

### Build with Xpress support

```bash
export XPRESSDIR="$HOME/User Apps/FICO Xpress/xpressmp"  # adjust to your install

# Rust crate
cargo build --features xpress

# Python wheel (maturin)
maturin develop --features xpress
```

### Usage

```python
import arco

solver = arco.Xpress(threads=4, log_to_console=False)
model = arco.Model()
x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0))
model.minimize(x)
solution = model.solve(solver=solver)
```

### Settings mapping

| Setting          | Xpress control            | Notes                             |
| ---------------- | ------------------------- | --------------------------------- |
| `time_limit`     | `XPRS_MAXTIME`            |                                   |
| `mip_gap`        | `XPRS_MIPRELSTOP`         |                                   |
| `tolerance`      | `XPRS_FEASTOL`            |                                   |
| `presolve`       | `XPRS_PRESOLVE`           | 1 = on, 0 = off                   |
| `threads`        | `XPRS_THREADS`            |                                   |
| `log_to_console` | `XPRS_OUTPUTLOG`          | 1 = on, 0 = off                   |
| `verbosity`      | --                        | Ignored                           |

## IPOPT (nonlinear / continuous solver)

The IPOPT backend is available when Arco is built with the `ipopt` feature flag.
IPOPT is a continuous-only solver -- it does **not** support integer or binary
variables. Passing a model that contains integer variables will raise an error.

> [!IMPORTANT]
> Building with IPOPT requires the IPOPT C library to be installed on the
> system. See the [IPOPT installation guide](https://coin-or.github.io/Ipopt/INSTALL.html)
> for platform-specific instructions.

### Build with IPOPT support

```bash
# Rust crate
cargo build --features ipopt

# Python wheel (maturin)
maturin develop --features ipopt
```

### Usage

```python
import arco

solver = arco.Ipopt(tolerance=1e-8, log_to_console=False)
model = arco.Model()
x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0))
model.minimize(x)
solution = model.solve(solver=solver)
```

### Settings mapping

The following table shows how `SolverSettings` map to IPOPT options. Settings
that do not apply to IPOPT are silently ignored.

| Setting          | IPOPT option                  | Notes                         |
| ---------------- | ----------------------------- | ----------------------------- |
| `time_limit`     | `max_cpu_time`                |                               |
| `tolerance`      | `tol` + `constr_viol_tol`     |                               |
| `verbosity`      | `print_level`                 | Clamped to 0--12              |
| `log_to_console` | `print_level` (0 when false)  |                               |
| `presolve`       | --                            | Ignored                       |
| `threads`        | --                            | Ignored                       |
| `mip_gap`        | --                            | Ignored (continuous only)     |

---

[How-to Guides](./) | [Docs home](../)
