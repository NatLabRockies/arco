# Arco API UX Ladder Plan

## Summary

Design Arco's pre-1.0 API as a strict, correct-by-construction ladder across
Python and KDL/CLI. The Python surface should follow the project's Python API
guidelines: one clear primary positional subject, configuration as
keyword-only arguments, structured objects, explicit names, and narrow errors.
Avoid permissive convenience APIs that trade predictability for shorthand.

The design lens comes from "APIs as ladders": make the underlying API flexible
first, make concepts gradual second, then add convenience without hiding
structure. Reference:
[APIs as ladders](https://blog.sbensu.com/posts/apis-as-ladders/).

## User Audiences

This ladder has to serve more than one user:

- First-time modelers who need a small linear program to work without learning
  the whole system.
- Python users who expect explicit, typed, inspectable objects and predictable
  signatures.
- KDL/CLI users who want declarative, reviewable model files and reproducible
  command-line workflows.
- Power-system and operations researchers building large indexed formulations
  where memory behavior matters.
- Advanced users integrating solver configuration, sparse matrix import/export,
  staged workflows, or Rust APIs.
- Solver backend contributors who add or maintain solver integrations. They
  should primarily work at the solver backend and runtime/facade layers, using
  `ModelView`/DTO contracts rather than changing Python, KDL, primitive model
  storage, or block APIs.

## API Ladder

1. Scalar model
   - User outcome: formulate and solve a small LP/MIP with a handful of scalar
     variables and constraints.
   - Concepts introduced: `Model`, scalar variable/control, scalar expression,
     scalar constraint, objective, solver default, solve result.
   - Python shape: `Model()`, `add_variable(...)`, `add_constraint(expr, *,
name=...)`, `minimize(expr, *, name=...)`, `solve()`, `result.value(x)`.
   - KDL shape: one `model` with scalar `control` declarations, scalar
     constraints, one `minimize`/`maximize`, and an inferred scenario only when
     unambiguous.
   - Keep out of this rung: index sets, arrays, solver profiles, data files,
     raw IDs, and sparse import/export.
   - Success criterion: a new user can read the whole model in one screen and
     understand where each mathematical object lives.

2. Indexed model
   - User outcome: express repeated variables and constraints over named
     dimensions without manual loops over raw integer positions.
   - Concepts introduced: `IndexSet`, named `param`, variable arrays, labeled
     axes, elementwise algebra, reductions, result values for arrays.
   - Python shape: `IndexSet(name=..., members=...)`,
     `param(values, *, axes=(...), name=...)`,
     `add_variables(axes=(...), name=..., bounds=...)`, array expressions, and
     reductions such as `.sum(...)`.
   - KDL shape: top-level or model-local `set`, `param` indexed by sets,
     indexed `control`, indexed `constraint`, and objective reductions.
   - Keep out of this rung: sparse active masks, tuple domains, alias-heavy
     network modeling, block composition, and custom solver configuration.
   - Success criterion: users can move from a scalar example to a small
     production-cost or allocation model by adding axes, not by changing mental
     models.

3. Data-backed model
   - User outcome: load model inputs from tabular data and keep data contracts
     explicit enough for review and reproducibility.
   - Concepts introduced: named data sources, set extraction, parameter
     binding, schema-like shape checks, missing-data diagnostics, and
     file-backed examples.
   - Python shape: structured loaders remain outside the core model API at
     first; users pass validated arrays/dataframes into `param(values, *,
axes=..., name=...)`.
   - KDL shape: `data ... source=...`, `map`, `set`, and `param` declarations
     that bind CSV columns to model symbols.
   - Keep out of this rung: implicit dataframe magic, hidden axis inference
     from column names, and silent filling of missing values.
   - Success criterion: users can tell whether an error is a data-contract
     problem, a model-shape problem, or a solver problem.

4. Sparse and large model
   - User outcome: build large models without accidental Cartesian explosion
     or memory spikes.
   - Concepts introduced: `active` masks, tuple domains, axis aliases, sparse
     variable creation, constraint filtering, and memory-aware inspection.
   - Python shape: `add_variables(..., active=mask)`, boolean `param` masks,
     axis aliases for directed pairs, and inspection that reports active counts
     versus dense shape.
   - KDL shape: explicit subset declarations, tuple domains, `filter`/`if`
     conditions, and diagnostics that name empty or mismatched domains.
   - Keep out of this rung: raw matrix construction and solver-specific
     tuning; those belong after users can inspect model size.
   - Success criterion: users can predict the number of variables,
     constraints, and coefficients before solving.

5. Debug and operations
   - User outcome: diagnose wrong or infeasible models without dropping into
     solver internals first.
   - Concepts introduced: `inspect`, model snapshots, objective terms,
     slacks/elastic constraints, duals where available, solver logs, solver
     profiles, and structured errors.
   - Python shape: `model.inspect(...)`, `result.value(...)`,
     `result.dual(...)`, `result.slack(...)`, explicit solver objects, and
     specific Arco exceptions.
   - KDL/CLI shape: `arco validate`, `arco inspect`, `arco print-model`,
     solver profile commands, compact solve summaries, and machine-readable
     JSON where useful.
   - Keep out of this rung: custom decomposition and staged workflows unless
     the model is already inspectable at each stage.
   - Success criterion: users can answer "what did Arco build?" and "why did
     this fail?" without reading Rust code.

6. Composition and workflows
   - User outcome: compose multiple model-building units or solve stages while
     preserving explicit inputs, outputs, diagnostics, and stable primitive
     contracts.
   - Concepts introduced: blocks, stage inputs/outputs, schema compatibility,
     reusable formulation components, result passing, warm starts where
     supported, and staged solve reports.
   - Python shape: block APIs that accept typed input objects and return
     structured outputs, plus `BlockModel` or equivalent orchestration. Blocks
     build on public model APIs instead of mutating internals.
   - KDL/CLI shape: explicit workflow/stage declarations only after the
     single-model KDL path is stable.
   - Keep out of this rung: hidden global state, implicit result mutation, and
     workflows that cannot be inspected stage by stage. Extension points must
     not require changing primitive model storage, compiler internals, or solver
     backends.
   - Success criterion: users can replace one block or stage without changing
     the contract of adjacent stages or forking Arco internals.

7. Expert escape hatches
   - User outcome: integrate Arco with advanced solver, sparse matrix, Rust, or
     benchmarking workflows without forcing those concepts onto beginners.
   - Concepts introduced: CSC import/export, raw IDs, low-level Rust
     `ModelView`, direct solver configuration, solver-specific parameters, and
     performance benchmarking.
   - Python shape: `from_csc`, raw vectors, ID-indexed accessors, explicit
     advanced namespace or documentation grouping.
   - Rust shape: `arco-model`, `ModelView`, `ModelPatch`, solver backend
     traits, and memory-conscious construction APIs.
   - Solver contributor shape: implement backend capability metadata,
     `ModelView` ingestion, solver-specific configuration translation, status
     mapping, result extraction, and diagnostics without changing higher-level
     modeling APIs.
   - Keep out of this rung: beginner docs and first-run examples.
   - Success criterion: expert APIs remain powerful and stable without
     becoming the accidental default user experience.

## Building Block Architecture

Arco should expose stable building blocks that users can compose into their own
modeling layer without modifying primitives, compiler internals, or solver
backends. This is the architecture constraint behind the ladder: each rung adds
composition power on top of the previous rung instead of punching through it.

- Keep primitives small and stable: variables/controls, sets, parameters,
  expressions, constraints, objectives, model snapshots, and solve results.
- Expose public extension points around those primitives: typed block inputs
  and outputs, validated parameter binding, model inspection, solver selection,
  and result extraction.
- Treat internals as replaceable implementation details: storage layout,
  lowering passes, sparse matrix construction, solver adapters, and compiler
  data structures should not be required for normal extension.
- Make reusable formulation components plain public API users can call, test,
  inspect, and version. A user should be able to build a domain-specific layer
  on top of Arco by composing blocks and functions.
- Preserve a narrow expert path for advanced integration, but do not make raw
  IDs, matrix storage, or internal compiler artifacts the required way to
  extend the system.
- Validate extension boundaries with contract tests: a block should be
  swappable when its typed inputs, outputs, and model effects remain compatible.

## What We Simplify

- Use one vocabulary across surfaces: variables/controls, sets, parameters,
  constraints, objectives, results.
- Keep the first scalar model path short without inventing a second permissive
  API that users later have to unlearn.
- Make names and axes explicit so users can inspect model structure without
  guessing positional meaning.
- Move advanced sparse, solver, and raw-ID operations out of the beginner path
  while preserving them as expert escape hatches.
- Make Python and KDL examples equivalent where possible, so users can move
  between programmatic and declarative workflows.

## Python API Shape

Use strict signatures with a primary positional subject and keyword-only
configuration.

### Sets

- Preferred beginner API: `arco.IndexSet(name="asset", members=[...])`.

### Parameters

- Preferred shape: `arco.param(values, *, axes=(asset, time), name="cost")`.
- Accept `axes=` as the explicit dimension contract.
- Reject duplicate or ambiguous axes with specific exceptions.

### Variables And Controls

- Preferred scalar shape:
  `model.add_variable(name="x", bounds=arco.NonNegativeFloat)`.
- Preferred indexed shape:
  `model.add_variables(axes=(asset, time), name="dispatch", bounds=arco.NonNegativeFloat, active=None)`.
- Keep Python terminology centered on `variable`; keep KDL terminology centered
  on `control` where that remains the domain-facing DSL word.
- If a new Python convenience method is added, prefer `add_control` only when
  it maps directly to KDL `control`; otherwise keep `add_variable` as the
  canonical programmatic operation.

### Constraints

- Preferred scalar shape: `model.add_constraint(expr, *, name="demand")`.
- Preferred indexed shape:
  `model.add_constraints(expr, *, name="capacity", active=None)`.

### Objectives

- Preferred minimization shape: `model.minimize(expr, *, name="total_cost")`.
- Preferred maximization shape: `model.maximize(expr, *, name="profit")`.

### Results

- Preferred value accessor: `result.value(variable)`.
- Preferred dual accessor: `result.dual(constraint)`.
- Preferred slack accessor: `result.slack(constraint)`.
- Keep raw vectors available as advanced properties.

## API Shapes To Avoid

- Avoid `arco.Set("asset", ...)` as the default beginner API unless the project
  explicitly wants a shorter exported name; `set` is already a loaded Python
  concept. If added, require keyword discipline:
  `arco.Set(name="asset", members=[...])`.
- Avoid `arco.param("cost", values, *sets)`: it has too many positional
  meanings and hides the primary subject.
- Avoid `model.variable("x", *sets, lower=..., upper=...)` or
  `model.control("x", *sets, lower=..., upper=...)`: name and axes become
  positional, and `lower`/`upper` competes with the existing structured
  `Bounds`/`BoundType` model.
- Avoid `model.constraint("name", expr, ...)`; the expression is the primary
  subject and should remain positional.
- Avoid `model.minimize("name", expr)` and `model.maximize("name", expr)`; the
  expression is the primary subject.

## KDL/CLI Alignment

- Align vocabulary with Python concepts, not necessarily exact method names:
  `set`, `param`, `variable`/`control`, `constraint`, `minimize`/`maximize`,
  `scenario`.
- Keep KDL `control` unless a later language-design pass proves `variable` is
  clearer across domain examples.
- Allow inferred scenario only when a file has exactly one runnable model and
  no ambiguity.
- Preserve explicit scenarios, reports, tuple domains, data bindings, and
  solver profiles for advanced users.

## Architecture Enablers

The ladder only works if the code architecture keeps user-facing concepts
separate from implementation machinery. Each surface should depend on stable
contracts below it, not on private storage or one-off lowering details.

Target ownership should be:

```text
User surfaces
  bindings/python (arco-python)
  crates/arco-cli
        │
        ├──────────────► crates/arco-blocks
        │                         │
        └──────────────┬──────────┘
                       ▼
Runtime facade
  crates/arco-ops
        │
        ├── Semantic/data construction: crates/arco-kdl
        ├── Validation/reporting:      crates/arco-validate + crates/arco-diagnostics
        ├── Labeled axes/arrays:       crates/arco-arrays
        ├── Primitive model contract:  crates/arco-model
        ├── Portable/export DTOs:      crates/arco-format
        ├── Solver contracts:          crates/arco-solver
        └── Solver registry/adapters:  crates/arco-builtin-solvers
                                      crates/arco-highs
                                      crates/arco-ipopt
                                      crates/arco-xpress
                                      crates/arco-scip
```

- Primitive model ownership: `crates/arco-model` owns variables/controls,
  bounds, expressions, constraints, objectives, model snapshots, sparse
  export/import, `ModelView`, `ModelPatch`, and primitive documents. This
  layer must stay small, allocation-aware, and independent of Python, KDL, CLI,
  blocks, and solver-specific behavior.
- Algebra and array ownership: `crates/arco-arrays` owns binding-agnostic
  labeled axes, parameter alignment, array planning, reductions, masks, tuple
  domains, and shape diagnostics. Python and KDL should share this semantic
  contract instead of maintaining separate array behavior.
- Semantic construction ownership: `crates/arco-kdl` owns KDL parsing, data
  binding, source diagnostics, scenario inference, and symbol resolution. It
  should produce normalized model-building artifacts rather than leaking parser
  structures into Python, CLI, blocks, or solvers.
- Runtime/facade ownership: `crates/arco-ops` owns solve orchestration,
  validation routing, inspection routing, solver selection, result mapping,
  and stable DTO boundaries. Python bindings, CLI, and blocks call this layer
  instead of duplicating solve or inspection behavior.
- Composition ownership: `crates/arco-blocks` owns blocks, workflows, typed
  input/output contracts, stage diagnostics, and swappability checks. It
  composes public model/runtime operations instead of reaching into model
  storage or compiler internals.
- Surface ownership: `bindings/python` and `crates/arco-cli` own ergonomics,
  documentation examples, command/API naming, and error presentation. They must
  stay thin over the shared contracts and must not own distinct modeling
  semantics.
- Solver ownership: `crates/arco-solver` owns solver contracts, selection, and
  preflight. Concrete solver crates own backend-specific capabilities,
  configuration translation, result extraction, status mapping, and
  diagnostics. Solver adapters consume `ModelView`/DTO contracts so adding a
  backend does not change Python, KDL, block, or primitive model APIs.

Architecture acceptance criteria:

- The same scalar and indexed examples can be expressed in Python and KDL and
  lower to equivalent model snapshots.
- A reusable block can be implemented using only public Python/Rust model APIs.
- A new block or domain-specific modeling helper does not require changing
  primitive model storage, KDL parser internals, or solver adapters.
- A new solver backend can be added by implementing backend capability,
  `ModelView`/DTO ingestion, solve configuration, status mapping, result
  extraction, and diagnostics without changing Python, KDL, block, or primitive
  model APIs.
- Inspection and result access use the same DTO shape across Python and CLI.
- Advanced raw-ID and sparse-matrix APIs remain available but are not required
  for normal extension.
- Memory-sensitive paths expose counts and active/sparse structure before
  solve, so convenience layers do not hide accidental dense expansion.

## Stable Public Contracts

Before 1.0, every user-facing API contract should either be made stable or be
removed from the public path. "User-facing" includes Python APIs, CLI/KDL
syntax, documented Rust extension points, serialized DTOs, diagnostics, and
solver integration contracts.

Contracts that should stabilize:

- Primitive model contracts: variables/controls, bounds, expressions,
  constraints, objectives, `ModelView`, `ModelPatch`, snapshots, and sparse
  export/import.
- Labeled-axis contracts: `IndexSet`, parameter axes, axis aliases,
  broadcasting, reductions, active masks, tuple domains, and shape errors.
- Solve contracts: solve request, solver selection, solver capability metadata,
  solver config, solve result, status mapping, primal/dual/slack accessors,
  and timing/resource metadata.
- Diagnostic contracts: stable error categories/codes, source/model
  provenance, shape/domain messages, solver diagnostics, and CLI/Python error
  equivalence.
- Composition contracts: typed block inputs/outputs, block model effects,
  stage diagnostics, swappability checks, and result passing.
- KDL contracts: core declarations, data binding, scenario inference rules,
  report behavior, and inspection/export output shape.

## Pre-1.0 Compatibility Policy

No backwards compatibility promise is required before 1.0. Use the pre-1.0
window to remove unclear names and unstable shortcuts instead of carrying
compatibility layers.

- Prefer direct breaking changes over long-lived aliases when the new API is
  clearer.
- Document old-to-new mappings only when they help current users migrate.
- Remove public APIs that encourage ambiguous positional arguments, hidden dense
  expansion, duplicate semantics across Python/KDL, or internals-dependent
  extension.
- Mark advanced APIs explicitly as advanced rather than preserving them as
  beginner-compatible alternatives.
- Do not preserve behavior solely because examples currently use it; update
  examples to teach the target architecture.

## Solver Backend Contributor Path

Adding a solver should be a narrow backend task, not a cross-cutting API
change. A solver contributor should usually need only the solver contract,
runtime facade, model-view contract, and backend crate.

Simple path:

1. Declare backend identity and capability metadata: family name, supported
   problem classes, integrality support, nonlinear/quadratic support if any,
   dual/slack availability, licensing/runtime requirements, and known limits.
2. Implement `ModelView`/DTO ingestion: read variables, constraints,
   objective, bounds, integrality, coefficients, names, and metadata without
   requiring Python or KDL structures.
3. Translate configuration: map Arco solver settings into backend-specific
   parameters, reject unsupported settings with specific diagnostics, and keep
   secrets/runtime paths out of public result payloads.
4. Solve and map status: convert backend statuses into Arco statuses with
   stable infeasible, unbounded, optimal, time-limit, iteration-limit, and
   internal-error categories.
5. Extract results: objective value, primal values, duals/reduced costs/slacks
   when available, timing, and backend metadata.
6. Register the backend through the solver registry/facade so Python and CLI
   selection work without surface-specific changes.
7. Add conformance tests: empty/no-objective behavior, small LP, small MILP
   when supported, unsupported capability diagnostics, config validation, and
   result-shape consistency.

The solver path should not require changes to Python modeling APIs, KDL
syntax, block APIs, primitive model storage, or existing examples except for
documentation that advertises the new backend.

## Diagnostic UX Contract

Diagnostics are part of the API ladder. Users should not need to know whether
an error came from Python, KDL, lowering, model validation, or a solver backend
to understand the next action.

- Every user-facing failure should have a stable category, actionable message,
  and provenance when available.
- Shape/domain diagnostics should name the affected parameter, variable/control,
  axes, expected shape/domain, actual shape/domain, and whether dense expansion
  was avoided.
- KDL diagnostics should include source locations and semantic object names.
- Python diagnostics should raise specific Arco exception types rather than
  broad `RuntimeError`/`ValueError` where Arco can classify the failure.
- CLI diagnostics should provide human-readable output by default and stable
  machine-readable JSON for automation.
- Solver diagnostics should distinguish model-invalid, unsupported capability,
  backend unavailable, license/runtime unavailable, solve failure, and backend
  internal error.
- The same underlying failure should map to the same diagnostic code across
  Python and CLI where possible.

## Memory And Performance Contract

Memory behavior is a hard product requirement, not an implementation detail.
APIs must make memory-sensitive behavior visible before solve and must avoid
hidden dense expansion.

- Expose dense shape, active count, variable count, constraint count, and
  coefficient count estimates through inspection before solve.
- Preserve sparse active masks and tuple domains through model construction
  instead of expanding them eagerly unless explicitly requested.
- Make array broadcasting and parameter alignment fail fast when they would
  produce ambiguous or unexpectedly dense models.
- Prefer streaming or view-based construction for large models where practical.
- Keep `ModelView` and solver ingestion allocation-aware so backends can read
  model structure without requiring duplicate full-model materialization.
- Include memory-sensitive regression tests for active masks, tuple domains,
  and large indexed examples.
- Document any API that intentionally materializes dense arrays or sparse
  matrices so users can make an informed tradeoff.
- Keep structural validation and full data materialization explicit. For
  example, `arco kdl check` should stay cheap by default and require a flag
  such as `--materialize-data` before loading CSV-backed parameter values used
  by lowered objectives and constraints.

## Examples And Test Matrix

| Ladder rung               | Example/documentation target                              | Contract tests                                                                                          |
| ------------------------- | --------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| Scalar model              | Python and KDL one-screen LP/MIP quickstarts              | scalar build, solve, result value                                                                       |
| Indexed model             | small allocation or production-cost model                 | axis alignment, reductions, array result access                                                         |
| Data-backed model         | CSV-backed KDL example and Python validated-array example | data binding, missing columns, shape mismatch                                                           |
| Sparse and large model    | tuple-domain or active-mask network example               | active count, sparse construction, no dense fallback                                                    |
| Debug and operations      | infeasible/debug walkthrough                              | inspect payload, slack/dual access, diagnostic codes                                                    |
| Composition and workflows | block composition tutorial                                | typed block I/O, swappability, stage diagnostics                                                        |
| Expert/solver backend     | backend fixture solver or small real backend              | capability metadata, config mapping, status/result conformance, CSC import/export and raw-vector access |

## Implementation Tracking

Use the ladder as an implementation map, not only a documentation outline. Each
slice should leave behind a stable user-facing artifact, a contract test, and a
clear note about which architecture boundary it exercised.

| Slice                              | User artifact                                  | Contract evidence                                                                                                                                                                                                                                                                                                                                                                                                                          | Architecture boundary exercised                                                                          | Remaining before 1.0                                                                             |
| ---------------------------------- | ---------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| Scalar Python/KDL                  | README and first-model examples                | scalar Python ladder test; scalar KDL run test; Python/KDL parity test comparing snapshot counts, memory estimates, solve status, and objective value                                                                                                                                                                                                                                                                                      | surfaces call public model/runtime APIs                                                                  | keep examples equivalent as APIs settle                                                          |
| Indexed Python/KDL                 | indexed tutorials and CSV-backed KDL example   | axis/param tests; variable-array, labeled active-mask, and named-reduction duplicate-axis shared-shape tests; indexed CLI run/inspect tests; Python/KDL parity test comparing indexed snapshot counts, memory estimates, solve status, and objective value                                                                                                                                                                                 | labeled axes should converge into shared array semantics                                                 | keep new labeled-axis behavior on the shared array contract instead of per-binding parsing       |
| Data-backed KDL                    | `arco kdl check --materialize-data` docs       | missing-data-file, scalar-param missing-column, tuple-domain missing-column, and invalid-number JSON diagnostic tests using shared diagnostic constants                                                                                                                                                                                                                                                                                    | data binding stays in semantic construction, CLI only presents it                                        | keep new data materialization paths covered by shared diagnostic-code tests                      |
| Sparse and large model             | active-mask docs, sparse result migration note | active-count, large active-mask no-dense-creation, sparse `result.value(array)`, Python array/expression memory estimates with density, dense-vs-active byte budgets, and solver-calibrated sparse-matrix byte budgets, Python/KDL sparse-memory inspect estimates, KDL tuple-domain row-count profiling, and KDL coefficient-instance inspect tests                                                                                       | sparse masks and tuple domains stay visible at surface without forcing dense storage                     | keep future sparse structures covered by memory-budget tests                                     |
| Debug and operations               | inspect/error docs and migration notes         | result value/dual/slack, native diagnostic-code registry tests, shared source/semantic/compile/algebra CLI/KDL registry coverage, shared Python exception code registry coverage, CLI config/driver diagnostic-code registry coverage, shared solver-setting error tests, and unsupported warm-start rejection through `SolverInvalidSettingError`                                                                                         | runtime facade owns inspect/result/error DTOs                                                            | keep new CLI/KDL diagnostics registry-backed as they are added                                   |
| Composition                        | block composition tutorial and schema docs     | typed I/O, swappability, statuses, ordered block report, diagnostics JSON, Python stub coverage for diagnostics accessors, artifact-manifest tests, stage-diagnostics artifacts, persisted artifact-writer tests, `arco.blocks` stub coverage that exposes only the stable decorator surface, and removal of uncompiled PyO3-era block-spec modules from the language-neutral `arco-blocks` crate                                          | blocks compose public model/runtime operations                                                           | keep future KDL workflow diagnostics aligned with the same stage-diagnostics payload             |
| Expert APIs and solver contributor | use-expert-apis and add-solver-backend how-tos | CSC import/export, raw-vector result access, registry-enforced result-shape test; shared backend conformance helper for empty, no-objective, small-LP, and small-MILP behavior proven by HiGHS and SCIP, with Xpress small-LP/small-MILP conformance gated on local runtime/license availability; HiGHS/SCIP/Xpress direct-result conformance paths; stable shared-setting error tests; duplicate-family registration test; boundary tests | concrete solvers consume `ModelView`/DTOs only; raw matrix APIs stay documented as expert escape hatches | keep new solver backends on shared conformance helpers and keep raw-ID APIs out of beginner docs |

Every new API-rung implementation should update this table or replace it with
a generated release checklist before 1.0. The important rule is that a surface
change is not complete until it proves which lower contract it depends on and
which internals it avoids.

## Implementation Changes

- Add API contract tests first:
  - Scalar Python model using `add_variable`, `add_constraint`, `minimize`,
    `solve`, and `result.value`.
  - Indexed Python model using `IndexSet`, `param(values, axes=..., name=...)`,
    variable arrays, reductions, solve, and result access.
  - Sparse Python model proving `active=` avoids dense expansion.
  - KDL equivalents for the scalar and indexed examples.

- Stabilize architecture contracts:
  - Keep primitive model APIs independent from Python, KDL, CLI, and solver
    backends.
  - Route Python and KDL model construction through shared labeled-axis and
    model-building semantics.
  - Route solve, inspect, and result mapping through one runtime/facade layer.
  - Make composition blocks depend on public model APIs and typed contracts,
    not internal storage or compiler artifacts.
  - Keep solver additions confined to solver contracts, runtime registration,
    and concrete backend crates.

- Update Python stubs and bindings:
  - Standardize `add_variable` and `add_variables` as canonical variable APIs.
  - Update `param` signature toward `values, *, axes, name`.
  - Add result accessor aliases if missing.
  - Remove unclear old names before 1.0 or reclassify them explicitly as
    advanced APIs.

- Update docs:
  - Root README shows strict Python and KDL quickstarts side by side.
  - Tutorials follow scalar -> indexed -> sparse -> debug -> workflows.
  - Reference docs document canonical APIs first and advanced APIs separately.
  - Migration notes list renamed or preferred APIs.

## Test Plan

- Run Python API tests through repo-standard `uv` or `just` targets.
- Run Rust parser, semantic, and model tests for KDL and core changes.
- Run solver conformance tests for any backend contract changes.
- Run docs doctests for quickstarts.
- Run example parity tests for one Python/KDL equivalent model.

## Assumptions

- Python strictness is intentional and should shape the beginner API.
- Breaking changes are acceptable before 1.0.
- Convenience must not mean ambiguous positional shorthand.
- Pre-1.0 APIs may break freely to reach the correct 1.0 shape.
- Memory behavior remains a hard public API and architecture contract.
