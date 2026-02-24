# Xpress Solver Backend Design

## Goal

Add FICO Xpress as a third solver backend for arco, supporting LP, MIP, and QP. Follow the same patterns established by `arco-highs` and `arco-ipopt`.

## Architecture

New crate `arco-xpress` at `crates/arco-xpress/` with hand-written FFI bindings to the Xpress C API (`libxprs`). Library discovery uses the `XPRESSDIR` environment variable.

### Crate structure

```
crates/arco-xpress/
├── Cargo.toml
├── build.rs            # Locates $XPRESSDIR, emits link directives
├── src/
│   ├── lib.rs          # Re-exports
│   ├── ffi.rs          # Hand-written extern "C" bindings to libxprs
│   ├── solver.rs       # Solver + XpressBackend (SolverBackend impl)
│   ├── solution.rs     # Solution struct + SolutionView impl
│   └── status.rs       # Xpress status → arco-core status mapping
└── tests/
    └── integration.rs  # Integration tests (require Xpress installed)
```

### FFI layer (ffi.rs)

Hand-written `extern "C"` declarations for ~30-40 Xpress C functions:

- Init/cleanup: `XPRSinit`, `XPRSfree`, `XPRScreateprob`, `XPRSdestroyprob`
- Problem loading: `XPRSloadlp`, `XPRSloadglobal` (MIP), `XPRSaddqmatrix64` (QP)
- Optimization: `XPRSlpoptimize`, `XPRSmipoptimize`
- Solution retrieval: `XPRSgetlpsol`, `XPRSgetmipsol`, `XPRSgetduals`
- Controls: `XPRSsetintcontrol`, `XPRSsetdblcontrol`, `XPRSgetintattrib`, `XPRSgetdblattrib`
- Objective: `XPRSchgobjsense`

### Build script (build.rs)

Reads `XPRESSDIR` env var, emits:
- `cargo:rustc-link-search=native={XPRESSDIR}/lib`
- `cargo:rustc-link-lib=dylib=xprs`
- `cargo:rerun-if-env-changed=XPRESSDIR`

Fails with a clear error if `XPRESSDIR` is not set.

### Solver (solver.rs)

- `Solver` struct: holds `Model`, `SolverConfig`, optional `primal_start`
- Configuration: `set_log_to_console`, `set_time_limit`, `set_mip_gap`, `set_verbosity`, `set_presolve`, `set_threads`, `set_tolerance`
- `solve()` / `solve_with_config()` methods
- Implements `arco_solver::Solve` trait
- `XpressBackend`: zero-sized struct implementing `SolverBackend` with `supports_integer() -> true`

### Solution (solution.rs)

- `Solution` struct: `primal_values`, `variable_duals`, `constraint_duals`, `objective_value`, status, `solve_time_seconds`
- Implements `SolutionView` trait
- `into_core_solution()` for converting to `arco_core::solver::Solution`

### Status mapping (status.rs)

| Xpress Status | arco-core Status |
|---|---|
| XPRS_LP_OPTIMAL (1) | Optimal |
| XPRS_LP_INFEAS (2) | Infeasible |
| XPRS_LP_UNBOUNDED (5) | Unbounded |
| XPRS_MIP_OPTIMAL (6) | Optimal |
| XPRS_MIP_INFEAS (5) | Infeasible |
| Time limit reached | TimeLimit |
| Iteration limit reached | IterationLimit |
| Others | Unknown |

## Integration points

1. **Workspace Cargo.toml**: Add `arco-xpress = { path = "crates/arco-xpress" }` to workspace deps
2. **Python bindings Cargo.toml**: Add `arco-xpress = { workspace = true, optional = true }` + feature `xpress = ["arco-xpress"]`
3. **Python `resolve_backend()`**: Replace error stub with `Box::new(arco_xpress::XpressBackend)` behind `#[cfg(feature = "xpress")]`
4. **Python `detect_default_backend()`**: Already handles Xpress (no change)
5. **Python `extract_solver_settings()`**: Already handles PyXpress (no change)
6. **Python `PyXpress` class**: Already exists (no change)

## What does not change

- `arco-solver` traits
- `arco-core`
- `arco-highs` / `arco-ipopt`
- `PyXpress` Python class (already exists)

## Scope

LP + MIP + QP support. No callback support in initial version.
