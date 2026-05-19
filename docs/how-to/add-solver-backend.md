# Add a Solver Backend

This guide is for contributors adding or maintaining a solver integration.
Adding a backend should be a narrow solver-layer change. It should not require
changes to Python modeling APIs, KDL syntax, block APIs, primitive model
storage, or user examples except documentation that advertises the backend.

## Target ownership

Use these crate boundaries:

| Area                                                  | Owner                                |
| ----------------------------------------------------- | ------------------------------------ |
| Primitive read-only model contract                    | `crates/arco-model`                  |
| Solver capability, config, status, and backend traits | `crates/arco-solver`                 |
| Runtime solve routing                                 | `crates/arco-ops`                    |
| Built-in backend registration                         | `crates/arco-builtin-solvers`        |
| Concrete adapter implementation                       | `crates/arco-<solver>`               |
| Python and CLI selection UX                           | `bindings/python`, `crates/arco-cli` |

The concrete adapter should consume `arco_model::ModelView` and return
`arco_solver::ModelViewSolveResult`. Do not depend on Python objects, KDL parser
nodes, block internals, or private `arco-model` storage.

## Backend skeleton

Create or update a concrete solver crate such as `crates/arco-foo`.

```rust
use arco_model::ModelView;
use arco_solver::{
    ModelViewBackend, ModelViewSolveResult, SolverCapabilityModel, SolverConfig, SolverError,
    SolverFamily, SolverRegistry, SolverStatus, check_empty_model_rejected,
    check_no_objective_rejected, check_small_lp, check_small_milp,
    validate_model_view_solve_result,
};

pub const FAMILY_NAME: &str = "foo";
pub const BACKEND_NAME: &str = "arco-rust-foo";

#[derive(Debug, Default, Clone, Copy)]
pub struct FooModelViewBackend;

impl ModelViewBackend for FooModelViewBackend {
    fn family(&self) -> &'static str {
        FAMILY_NAME
    }

    fn solve_model_view(
        &self,
        model: &dyn ModelView,
        config: &SolverConfig,
    ) -> Result<ModelViewSolveResult, SolverError> {
        solve_model_view(model, config)
    }
}

pub fn register_solver_family(registry: &mut SolverRegistry) {
    registry.add_family(SolverFamily::embedded(
        FAMILY_NAME,
        "Foo",
        SolverCapabilityModel::lp_mip_default(),
    ));
}

pub fn solve_model_view(
    model: &(impl ModelView + ?Sized),
    config: &SolverConfig,
) -> Result<ModelViewSolveResult, SolverError> {
    if model.num_variables() == 0 {
        return Err(SolverError::EmptyModel);
    }
    if model.objective().sense.is_none() && model.objective().terms.is_empty() {
        return Err(SolverError::NoObjective);
    }

    let _facts = model.structural_facts();
    let _log = config.log_to_console.unwrap_or(false);

    // Translate the read-only ModelView into the backend's native model here.
    // Preserve model order: variable, dual, row, and slack vectors must align
    // with Arco variable-id and constraint-id order.

    let result = ModelViewSolveResult {
        fingerprint: model.fingerprint(),
        status: SolverStatus::Optimal,
        objective_value: 0.0,
        primal_values: vec![0.0; model.num_variables()],
        variable_duals: Vec::new(),
        row_values: vec![0.0; model.num_constraints()],
        constraint_duals: Vec::new(),
        metadata: Default::default(),
    };
    validate_model_view_solve_result(model, &result)?;
    Ok(result)
}
```

This skeleton is intentionally small. The actual backend must translate
variables, constraints, bounds, integrality, coefficients, objective sense,
configuration, status, and result vectors.

## Implementation checklist

1. Declare solver identity.
   Add stable `FAMILY_NAME` and `BACKEND_NAME` constants. The family name is
   the public selection token users pass through Python, CLI, or profiles.

2. Register capabilities.
   Use `SolverCapabilityModel::lp_mip_default()` for LP/MIP backends, or
   define explicit capabilities when support differs. Do not overstate support:
   capability metadata drives preflight and user diagnostics.

3. Implement `ModelViewBackend`.
   The backend receives `&dyn ModelView` and `&SolverConfig`. Read model
   structure through trait methods such as `num_variables`, `variable`,
   `constraint`, `objective`, `column`, `variable_name`, and
   `constraint_name`.

4. Translate configuration.
   Map shared settings (`time_limit`, `mip_gap`, `verbosity`, `presolve`,
   `threads`, `tolerance`, `log_to_console`) into native backend options. Use
   `SolverConfig::parameters` for family-specific passthrough settings.

5. Map statuses and diagnostics.
   Convert native statuses into `SolverStatus` and native failures into
   `SolverError`. Keep unavailable runtime/license errors distinct from model
   invalid, unsupported capability, solve failure, and backend internal error.

6. Extract results in model order.
   `primal_values` and `variable_duals` must align with variable-id order.
   `row_values` and `constraint_duals` must align with constraint-id order.
   `primal_values` are required for every variable when a backend returns a
   result. Leave unavailable optional vectors empty rather than fabricating
   unsupported duals or row activities. Call
   `validate_model_view_solve_result(model, &result)?` before returning from
   direct solver entrypoints; the shared model-view registry also rejects
   result vectors whose lengths do not match the input model.

7. Register the backend.
   Add the concrete backend to `register_builtin_model_view_backends` in
   `crates/arco-builtin-solvers` when it should be available as a built-in.
   Register the solver family so selection and preflight can see it. Backend
   registration rejects duplicate family names; use a distinct family string
   unless you are intentionally testing an override path.

8. Keep user surfaces thin.
   Add Python or CLI selection names only after the solver-layer contract is in
   place. Do not introduce Python-only behavior or CLI-only model semantics.

## Memory rules

Solver adapters are on a memory-sensitive path.

- Prefer streaming over variables and columns instead of building duplicate
  full matrices when the backend API allows it.
- If a backend requires a native dense or sparse copy, make that materialization
  explicit in code and tests.
- Preserve Arco variable and constraint order while avoiding name-based lookup
  in hot paths.
- Keep backend metadata compact. Do not store full solver logs in result
  payloads.
- Add tests that would catch accidental dense expansion for active-mask or
  tuple-domain models when the backend supports those paths.

## Tests to add

At minimum, add tests in the concrete solver crate for:

- Empty model behavior.
- A small feasible LP.
- A small infeasible LP.
- A small MILP when the backend supports integer variables.
- Unsupported capability diagnostics when the backend rejects a model class.
- Config mapping for time limit, MIP gap, threads, tolerance, and logging.
- Result vector shape and ordering.
- Status mapping for optimal, infeasible, unbounded, time limit, and failure
  states that the backend can produce deterministically.

Start with the shared primitive backend conformance checks. Use
`small_milp_model()` for integer-capability cases so every backend exercises the
same binary fixture, then add backend-specific cases for native status mapping:

```rust
#[test]
fn foo_backend_passes_shared_conformance() -> Result<(), SolverError> {
    let backend = FooModelViewBackend;
    check_empty_model_rejected(&backend)?;
    check_no_objective_rejected(&backend)?;
    let report = check_small_lp(&backend, &SolverConfig::default())?;
    let milp_report = check_small_milp(&backend, &SolverConfig::default())?;

    assert_eq!(report.family, FAMILY_NAME);
    assert_eq!(report.variables, 1);
    assert_eq!(report.constraints, 1);
    assert_eq!(report.coefficients, 1);
    assert_eq!(milp_report.family, FAMILY_NAME);
    assert_eq!(milp_report.variables, 1);
    assert_eq!(milp_report.constraints, 1);
    assert_eq!(milp_report.coefficients, 1);
    Ok(())
}
```

Run the focused solver checks:

```bash
just test-solver arco-foo
just clippy-solver arco-foo
```

Run broader checks before merging a built-in backend:

```bash
just arch-check
just test-pkg arco-solver
just test-pkg arco-ops
just test-pkg arco-builtin-solvers
```

## What not to change

Adding a backend should not require:

- New Python model-building APIs.
- New KDL syntax.
- New block APIs.
- Changes to primitive model storage.
- Changes to examples that are unrelated to solver selection.
- Special-case solver code in `bindings/python` or `crates/arco-cli` beyond
  thin selection/configuration presentation.

When a backend appears to need one of those changes, first add or adjust a
shared solver/model contract so every surface can use the same capability.

---

[How-to Guides](./) | [Docs home](../)
