# Configure Solver

Arco separates model construction from solver configuration. Build a model once,
then swap solver settings without changing model code.

## CLI solver selection and profile config

Arco stores CLI solver selection in a versioned TOML file:

- User scope: `~/.config/arco/solver.toml` (or `ARCO_CONFIG_DIR/solver.toml`)
- Project scope: `./.arco/solver.toml` (or `ARCO_PROJECT_CONFIG_DIR/solver.toml`)

Selections can be a solver family or a profile. Built-in families include
`highs`, `xpress` (when enabled), and `scip` (via `arco-scip`).

```bash
arco solver set highs
arco solver show
```

`arco solver show` displays resolved family/profile/transport and best-effort
availability.

> [!NOTE]
> Legacy `solver.json` is not auto-migrated. Create `solver.toml` explicitly.

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

| Setting          | Type          | Description                                                       |
| ---------------- | ------------- | ----------------------------------------------------------------- |
| `presolve`       | `bool`        | Enable or disable the presolve phase.                             |
| `threads`        | `int`         | Number of threads the solver may use.                             |
| `tolerance`      | `float`       | Feasibility tolerance for primal and dual values.                 |
| `time_limit`     | `float`       | Maximum wall-clock seconds the solver may run.                    |
| `mip_gap`        | `float`       | Relative MIP optimality gap at which the solver stops.            |
| `verbosity`      | `int`         | Solver output verbosity level (backend-specific scale).           |
| `log_to_console` | `bool`        | Whether the solver prints progress to the console during a solve. |
| `lp_algorithm`   | `LpAlgorithm` | Solver-independent LP algorithm preference.                       |

> [!NOTE]
> Not every backend interprets every setting. Arco validates shared settings at
> construction and solve time, then applies each backend's documented mapping
> below.
>
> `time_limit`, `mip_gap`, and `tolerance` must be finite and non-negative.
> `threads` must be at least `1`. Invalid values raise
> `SolverInvalidSettingError` with diagnostic code
> `arco::solver::invalid_setting`.

## Select an LP algorithm

Use `LpAlgorithm` when you want to select an LP algorithm without encoding a
backend's native option names:

```python
import arco

solver = arco.HiGHS(
    lp_algorithm=arco.LpAlgorithm.BARRIER_WITH_CROSSOVER,
    log_to_console=False,
)
solution = model.solve(solver=solver)
```

The same setting is available on `arco.HiGHS`, `arco.Xpress`, `arco.Scip`, the
generic `arco.Solver`, and the one-off `model.solve(lp_algorithm=...)` call.
Each adapter translates the semantic value into its native settings:

| `LpAlgorithm` value       | HiGHS                                  | Xpress            | SCIP              |
| ------------------------- | -------------------------------------- | ----------------- | ----------------- |
| `AUTOMATIC`               | `solver=choose`                        | no optimize flags | `lp/*algorithm=s` |
| `PRIMAL_SIMPLEX`          | `solver=simplex`, `simplex_strategy=4` | `p` flag          | `lp/*algorithm=p` |
| `DUAL_SIMPLEX`            | `solver=simplex`, `simplex_strategy=1` | `d` flag          | `lp/*algorithm=d` |
| `BARRIER`                 | `solver=ipm`, `run_crossover=off`      | `b` flag          | `lp/*algorithm=b` |
| `BARRIER_WITH_CROSSOVER`  | `solver=ipm`, `run_crossover=on`       | `b` flag          | `lp/*algorithm=c` |
| `PRIMAL_DUAL_FIRST_ORDER` | `solver=pdlp`                          | unsupported       | unsupported       |
| `CONCURRENT`              | unsupported                            | `pdb` flags       | unsupported       |

A backend that cannot represent a selected algorithm raises
`SolverInvalidSettingError`; it does not silently substitute another method.
For MIPs, the setting controls LP relaxations according to each solver's native
semantics.

## Xpress (LP / MIP solver)

The Xpress backend is included in prebuilt Arco binaries and default Python
package builds. Rust source builds that opt out of release/default packaging can
still enable it with the `xpress` feature flag. Xpress supports LP, MIP, and QP
problems.

> [!IMPORTANT]
> Xpress requires the FICO Xpress Optimizer SDK installed locally.

> [!NOTE]
> Arco resolves both the Xpress 9 `XPRSloadmip` runtime symbol and the
> Xpress 8 `XPRSloadglobal` compatibility symbol, so Windows installations on
> either major version can load MIP models through the same backend.

### Setup at a glance

1. Install Xpress Community Edition from [fico.com](https://www.fico.com/en/products/fico-xpress-optimization).
2. If you installed Arco from a prebuilt binary or the default Python package,
   skip build steps and only configure environment variables.
3. If you build the Rust CLI yourself, build with `--features xpress`.
4. Run `arco solver set xpress` and verify with `arco solver show`.

### Platform setup

<details>
<summary>macOS</summary>

Typical `XPRESSDIR` locations:

- `~/User Apps/FICO Xpress/xpressmp`
- `/Applications/FICO Xpress/xpressmp`
- `~/opt/xpressmp`

Build:

```bash
# Optional when not in an auto-detected location
export XPRESSDIR="$HOME/opt/xpressmp"
cargo build --features xpress
uv run --project bindings/python --with maturin maturin develop
```

</details>

<details>
<summary>Linux</summary>

Typical `XPRESSDIR`: `/opt/xpressmp`

Build:

```bash
# Optional when not in an auto-detected location
export XPRESSDIR="/opt/xpressmp"
cargo build --features xpress
uv run --project bindings/python --with maturin maturin develop
```

</details>

<details>
<summary>Windows (PowerShell)</summary>

Typical `XPRESSDIR` locations:

- `C:\xpressmp`
- `%USERPROFILE%\AppData\Local\FICO Xpress\xpressmp`
- `%ProgramFiles%\FICO Xpress\xpressmp`
- `%ProgramFiles(x86)%\FICO Xpress\xpressmp`

Build:

```powershell
$env:XPRESSDIR = "C:\xpressmp"
cargo build --features xpress
uv run --project bindings/python --with maturin maturin develop
```

</details>

> [!TIP]
> Use `arco solver show` after setting the backend. It is the fastest way to
> confirm selection and availability state.

### License files

The installer creates license files under `$XPRESSDIR/bin/`:

- `community-xpauth.xpr` — Community Edition license
- `xpauth.xpr` — commercial license

Arco tries community first, then commercial. To force a specific commercial
license, set `XPAUTH_PATH` to the full license file path.

> [!NOTE]
> Community licenses expire. If Xpress suddenly fails license initialization,
> refresh/reinstall the SDK to regenerate `community-xpauth.xpr`.

### Using a prebuilt Arco binary (no build required)

If your `arco` binary already includes Xpress support, you only need:

1. Xpress SDK installed.
2. `XPRESSDIR` only if auto-detection does not find your install.
3. Optional `XPAUTH_PATH` only for forcing a commercial license file.
4. Solver selection + verification:

```bash
arco solver set xpress
arco solver show
```

<details>
<summary>macOS / Linux: one-session setup</summary>

```bash
export XPRESSDIR="$HOME/opt/xpressmp"
# optional, commercial license only
export XPAUTH_PATH="$XPRESSDIR/bin/xpauth.xpr"

arco solver set xpress
arco solver show
```

</details>

<details>
<summary>macOS / Linux: persistent setup</summary>

```bash
echo 'export XPRESSDIR="$HOME/opt/xpressmp"' >> ~/.zshrc
# optional, commercial license only
echo 'export XPAUTH_PATH="$XPRESSDIR/bin/xpauth.xpr"' >> ~/.zshrc

source ~/.zshrc
arco solver set xpress
arco solver show
```

</details>

<details>
<summary>Windows (PowerShell): one-session setup</summary>

```powershell
$env:XPRESSDIR = "C:\xpressmp"
# optional, commercial license only
$env:XPAUTH_PATH = "$env:XPRESSDIR\bin\xpauth.xpr"

arco solver set xpress
arco solver show
```

</details>

<details>
<summary>Windows (PowerShell): persistent setup</summary>

```powershell
setx XPRESSDIR "C:\xpressmp"
# optional, commercial license only
setx XPAUTH_PATH "C:\xpressmp\bin\xpauth.xpr"

# open a new terminal, then verify
arco solver set xpress
arco solver show
```

</details>

### Build with Xpress support (Rust CLI source builds only)

Use the OS-specific toggle blocks in [Platform setup](#platform-setup) when
building the Rust CLI from source. Python source builds include Xpress by
default; pass `--no-default-features` only when you intentionally need a Python
extension without Xpress support.

<details>
<summary>macOS quick smoke test directly from DMG (no copy needed)</summary>

```bash
hdiutil attach -nobrowse -readonly "$HOME/Downloads/FICO_Xpress_9.8.1_for_ARM_Mac_Installer.dmg"
export XPRESSDIR="/Volumes/FICO Xpress Installer/FICO Xpress/xpressmp"
```

The mounted `xpressmp` tree includes a valid community license, enough for
local CLI/Python verification.

</details>

### CLI usage

```bash
arco solver set xpress
arco solver show
arco run examples/dense-lp/input.kdl --compact
```

If SDK discovery fails, retry with `XPRESSDIR=/path/to/xpressmp`.

### CI notes

CI can run Xpress-backed tests/releases when the SDK archive URL is provided via
`XPRESS_SDK_LINUX_URL` repository secret. The Linux workflows unpack the SDK to
`/opt/xpressmp` and set `XPRESSDIR` automatically.

### Python usage

```python
import arco

solver = arco.Xpress(threads=4, log_to_console=False)
model = arco.Model()
x = model.add_variable(bounds=arco.Bounds(lower=0.0, upper=10.0))
model.minimize(x)
solution = model.solve(solver=solver)
```

You can also select the backend without building a dedicated solver object:

```python
import arco

selection = arco.SolverSelection.family("xpress")
solution = model.solve(solver=selection, log_to_console=False)
```

### Settings mapping

| Setting          | Xpress control                     | Notes                                           |
| ---------------- | ---------------------------------- | ----------------------------------------------- |
| `time_limit`     | `XPRS_MAXTIME`                     |                                                 |
| `mip_gap`        | `XPRS_MIPRELSTOP`                  |                                                 |
| `tolerance`      | `XPRS_FEASTOL`                     |                                                 |
| `presolve`       | `XPRS_PRESOLVE`                    | 1 = on, 0 = off                                 |
| `threads`        | `XPRS_THREADS`                     |                                                 |
| `log_to_console` | `XPRS_OUTPUTLOG`                   | 1 = on, 0 = off                                 |
| `verbosity`      | --                                 | Unsupported; raises `SolverInvalidSettingError` |
| `lp_algorithm`   | optimizer flags + `XPRS_CROSSOVER` | Uses the shared mapping documented above.       |

## SCIP (embedded native LP / MIP solver)

SCIP support is provided by `arco-scip` through
[`russcip`](https://github.com/scipopt/russcip). Arco keeps SCIP embedded in the
default Rust build for development, sends the LP/MIP model through the native
Rust SCIP API, and does not require a separate `scip` executable at runtime.
Official release binaries statically link SCIP so fresh installs remain
self-contained.

> [!IMPORTANT]
> SCIP is distributed under the Apache-2.0 license, but some optional third-party
> SCIP build components may have different licenses. Check SCIP and `russcip`
> redistribution terms before publishing bundled Arco artifacts.
>
> Release builds that use `scip-from-source` require the source build toolchain
> used by `scip-sys` (for example, `cmake` and `libclang`).

### Setup at a glance

1. Build or install Arco normally; SCIP is embedded in the default Rust build.
2. Official release binaries ship SCIP statically, so no extra runtime library
   setup is needed on fresh installs.
3. Run `arco solver set scip` and verify with `arco solver show`.

```bash
arco solver set scip
arco solver show
arco run examples/dense-lp/input.kdl --compact
```

### Python usage

```python
import arco

solver = arco.Scip(time_limit=60.0, log_to_console=False)
solution = model.solve(solver=solver)
```

You can also select a configured SCIP profile or family without constructing a
solver object:

```python
selection = arco.SolverSelection.family("scip")
solution = model.solve(solver=selection, log_to_console=False)
```

### Settings mapping

| Setting          | SCIP handling                              | Notes                                               |
| ---------------- | ------------------------------------------ | --------------------------------------------------- |
| `time_limit`     | `set limits/time`                          | From profile or Python settings                     |
| `mip_gap`        | `set limits/gap`                           | From profile or Python settings                     |
| `log_to_console` | SCIP output toggle                         | `false` keeps logs quiet                            |
| `presolve`       | `presolving/maxrounds`                     | `false` disables presolve; `true` uses SCIP default |
| `threads`        | `parallel/maxnthreads`                     |                                                     |
| `tolerance`      | `numerics/feastol`                         | Feasibility tolerance                               |
| `verbosity`      | `display/verblevel`                        | SCIP verbosity level                                |
| `lp_algorithm`   | `lp/initalgorithm` + `lp/resolvealgorithm` | Shared mapping above                                |

## IPOPT (nonlinear / continuous solver)

The IPOPT backend is available when Arco is built with the `ipopt` feature flag.
IPOPT is a continuous-only solver -- it does not support integer or binary
variables. Passing a model that contains integer variables will raise an error.
IPOPT does not expose the shared `lp_algorithm` setting.

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
without an IPOPT option are validated by Arco's shared solver-settings layer
but are not forwarded to IPOPT.

| Setting          | IPOPT option                 | Notes                                     |
| ---------------- | ---------------------------- | ----------------------------------------- |
| `time_limit`     | `max_cpu_time`               |                                           |
| `tolerance`      | `tol` + `constr_viol_tol`    |                                           |
| `verbosity`      | `print_level`                | Clamped to 0--12                          |
| `log_to_console` | `print_level` (0 when false) |                                           |
| `presolve`       | --                           | Validated, not forwarded                  |
| `threads`        | --                           | Validated, not forwarded                  |
| `mip_gap`        | --                           | Validated, not forwarded; continuous only |

---

[How-to Guides](./) | [Docs home](../)
