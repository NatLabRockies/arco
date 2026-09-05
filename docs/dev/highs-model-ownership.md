# HiGHS model ownership

`arco_highs::PreparedHighsModel` separates model loading from optimization for
callers that need to release a large source model before the native solver
runs. Preparation borrows the `ModelView` only while it gathers dimensions,
objective, bounds, integrality, and compressed-column matrix data:

```rust,ignore
use arco_highs::PreparedHighsModel;

let prepared = PreparedHighsModel::prepare(&model, &config)?;
drop(model);
let result = prepared.solve()?;
```

The prepared value retains only the native HiGHS handle, dimensions, the
optional fingerprint, extraction choice, and timing metadata. It does not
retain the source view, model columns, names, configuration map, or input
arrays. `solve` consumes the prepared value, runs the native LP or MIP, and
returns the shared `ModelViewSolveResult`. Native resources are released when
the solve succeeds, fails, or the unsolved prepared value is dropped.

HiGHS' `Highs_passLp` and `Highs_passMip` APIs copy their pointer arguments
into the native model during the call. The HiGHS 1.15.0 implementation uses
`std::vector::assign` for costs, bounds, integrality, and compressed matrix
arrays in its [official `Highs.cpp` source](https://github.com/ERGO-Code/HiGHS/blob/v1.15.0/highs/lp_data/Highs.cpp#L533-L624).
This makes it safe for the adapter's load buffers to be scoped to preparation;
they are dropped before `solve` starts.

The existing `solve_model_view` function keeps its borrowing API and result
validation. The prepared API captures the fingerprint while the source model
is borrowed. Set `arco.fingerprint=false` to use the zero fingerprint sentinel
and `arco.extract_solution=false` to omit solution vectors; when a nonzero
fingerprint is used, the normal result-shape contract still requires the
primal vector.

Preparation records `highs_matrix_build_s`, `highs_prepare_s`, and
`fingerprint_s`; optimization and extraction remain separately reported as
`highs_run_s` and `solution_extract_s`. Time spent holding a prepared value
between `prepare` and `solve` is outside those phases.

When callers drop the source model after preparation, this boundary can reduce
live source-model memory during optimization. It does not measure or change
HiGHS' own factorization, presolve, or solution memory, and it does not
establish parity with another solver's total process peak.

Focused validation:

```bash
scripts/with_solver_build_env.sh rustup run 1.85.1 cargo test -p arco-highs --lib prepared_model_
scripts/with_solver_build_env.sh rustup run 1.85.1 cargo clippy -p arco-highs --all-targets --all-features -- -D warnings
```
