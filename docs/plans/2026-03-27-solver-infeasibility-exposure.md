# Solver Infeasibility Exposure Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `arco iis <path>` so the configured solver runs native infeasibility analysis on a KDL model and prints a solver-agnostic IIS report containing conflicting constraint rows and variable bounds.

**Architecture:** Introduce a shared infeasibility-analysis capability in the solver abstraction, with typed solver-agnostic result objects in core. Each backend implements native IIS/conflict extraction without Arco building any relaxation. The CLI compiles KDL, builds the solver model, asks the configured backend for infeasibility analysis, and renders a print-model-style ASCII report for the returned rows and bounds.

**Tech Stack:** Rust workspace, `clap`, `miette`, `arco-core`, `arco-solver`, `arco-highs`, `arco-xpress`, `arco-kdl`, CLI integration tests.

---

## Chunk 1: Shared types and capability boundary

### Task 1: Define the shared infeasibility result model

**Files:**

- Modify: `crates/arco-core/src/solver.rs`
- Modify: `crates/arco-core/src/lib.rs`
- Test: `crates/arco-core/src/solver.rs`

- [ ] **Step 1: Write failing unit tests for new shared types**
- Add tests covering:
  - row member construction
  - bound member construction
  - display/status helpers
  - empty-analysis rejection or explicit `has_members()` behavior

- [ ] **Step 2: Add minimal shared types**
- Add a solver-agnostic payload, roughly:
  - `InfeasibilityAnalysis`
  - `InfeasibilityRow`
  - `InfeasibilityBound`
  - `BoundSide`
  - optional `MembershipStrength` if a solver can mark exact vs candidate members

- [ ] **Step 3: Keep the payload solver-neutral**
- Required row fields:
  - row index
  - lowered row name
  - row sense
  - rhs
- Required bound fields:
  - column index
  - lowered variable name
  - side
  - numeric bound
- Optional metadata:
  - backend-specific notes as strings
  - membership strength if needed later

- [ ] **Step 4: Export the new types**
- Re-export from `crates/arco-core/src/lib.rs`

- [ ] **Step 5: Run targeted tests**
- Run: `cargo test -p arco-core solver::tests --all-features`
- Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add crates/arco-core/src/solver.rs crates/arco-core/src/lib.rs
git commit -m "feat: add shared infeasibility analysis types"
```

### Task 2: Add a shared solver capability for IIS analysis

**Files:**

- Modify: `crates/arco-solver/src/backend.rs`
- Modify: `crates/arco-solver/src/lib.rs`
- Test: `crates/arco-solver/src/backend.rs`

- [ ] **Step 1: Write failing trait-level tests or compile checks**
- Cover the new backend method signature and unsupported behavior.

- [ ] **Step 2: Add the capability to `SolverBackend`**
- Add a method along the lines of:
  - `analyze_infeasibility(&self, model: &Model, config: &SolverConfig, primal_start: Option<&[(VariableId, f64)]>) -> Result<InfeasibilityAnalysis, SolverError>`

- [ ] **Step 3: Add a clear unsupported path**
- If a backend has no IIS/conflict support, return a typed solver error such as:
  - `SolverError::UnsupportedFeature("infeasibility_analysis")`
- Do not add any fallback relaxation path.

- [ ] **Step 4: Run targeted tests**
- Run: `cargo test -p arco-solver --all-features`
- Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/arco-solver/src/backend.rs crates/arco-solver/src/lib.rs
git commit -m "feat: add shared solver infeasibility analysis capability"
```

---

## Chunk 2: Backend-native IIS implementations

### Task 3: Implement HiGHS IIS extraction

**Files:**

- Modify: `crates/arco-highs/src/ffi.rs`
- Modify: `crates/arco-highs/src/solver.rs`
- Possibly modify: `crates/arco-highs/src/lib.rs`
- Test: `crates/arco-highs/src/solver.rs`

- [ ] **Step 1: Add a failing HiGHS infeasible-model test**
- Build a tiny infeasible core model directly:
  - variable `x` with bounds `[0, 1]`
  - constraint `demand: x >= 5`
  - objective `min x`
- Assert IIS analysis returns:
  - row `demand`
  - at least one variable bound on `x`

- [ ] **Step 2: Add the required HiGHS FFI bindings**
- Bind native IIS APIs, using documented HiGHS functions like `Highs_getIis`.
- Keep the FFI surface minimal, only what the Rust layer needs.

- [ ] **Step 3: Implement backend translation**
- Build `InfeasibilityAnalysis` from:
  - row indices -> row names via model naming/order
  - column indices -> variable names via model naming/order
  - row/column bound side enums -> shared `BoundSide`
- Let HiGHS do the analysis, do not construct any relaxed model in Arco.

- [ ] **Step 4: Handle solver preconditions inside the backend**
- If HiGHS requires a solve before IIS extraction, keep that logic inside the backend implementation.
- The shared API stays “analyze this model”, not “solve then maybe analyze”.

- [ ] **Step 5: Run targeted tests**
- Run: `cargo test -p arco-highs --all-features infeas`
- Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add crates/arco-highs/src/ffi.rs crates/arco-highs/src/solver.rs crates/arco-highs/src/lib.rs
git commit -m "feat: implement highs infeasibility analysis"
```

### Task 4: Implement Xpress IIS extraction

**Files:**

- Modify: `crates/arco-xpress/src/ffi.rs`
- Modify: `crates/arco-xpress/src/solver.rs`
- Possibly modify: `crates/arco-xpress/src/lib.rs`
- Test: `crates/arco-xpress/src/solver.rs` or `crates/arco-xpress/tests/integration.rs`

- [ ] **Step 1: Add a failing Xpress infeasible-model test**
- Reuse the same direct core-model fixture as HiGHS.

- [ ] **Step 2: Add minimal Xpress FFI bindings**
- Bind the IIS calls needed to:
  - generate an IIS
  - read row members
  - read bound members

- [ ] **Step 3: Translate Xpress output into the shared payload**
- Map rows and bounds into the same `InfeasibilityAnalysis` type.
- Preserve only solver-neutral fields in the shared output.

- [ ] **Step 4: Return unsupported cleanly when the feature/build is absent**
- The CLI should receive a structured unsupported error, not a panic and not a fake diagnosis.

- [ ] **Step 5: Run targeted tests**
- Run: `cargo test -p arco-xpress --all-features infeas`
- Expected: PASS when Xpress is available

- [ ] **Step 6: Commit**

```bash
git add crates/arco-xpress/src/ffi.rs crates/arco-xpress/src/solver.rs crates/arco-xpress/src/lib.rs
git commit -m "feat: implement xpress infeasibility analysis"
```

---

## Chunk 3: CLI plumbing and pretty rendering

### Task 5: Add CLI-side execution path for infeasibility analysis

**Files:**

- Modify: `crates/arco-cli/src/main.rs`
- Modify: `crates/arco-cli/src/lib.rs`
- Modify: `crates/arco-cli/src/driver.rs`
- Modify: `crates/arco-cli/src/execution.rs`
- Test: `crates/arco-cli/tests/cli_run.rs`

- [ ] **Step 1: Add a failing CLI test for the new command**
- Add a test that runs:

```bash
arco iis <fixture>
```

- Assert success and output containing:
  - the active backend
  - the infeasible row name
  - the conflicting variable bound

- [ ] **Step 2: Add the clap subcommand**
- Add `Iis { path: PathBuf }` to `crates/arco-cli/src/main.rs`
- Wire it beside `print-model`, `run`, `validate`, and `export`

- [ ] **Step 3: Add driver entrypoints**
- Add a driver function like:
  - `analyze_file_infeasibility(path, backend) -> Result<String, DriverError>`
- It should:
  - compile KDL
  - select configured backend
  - call the shared analysis API
  - render the report

- [ ] **Step 4: Refactor model-building reuse**
- The current `build_model` in `crates/arco-cli/src/execution.rs` is the main seam.
- Extend its return value so CLI analysis has access to:
  - named variables
  - named constraints
  - stable row/column order
- Do not duplicate model translation for `run` vs `iis`.

- [ ] **Step 5: Add CLI-facing analysis errors**
- Distinguish:
  - solver says model is not infeasible / no IIS available
  - backend unsupported
  - backend analysis failed
- Keep errors `miette`-friendly.

- [ ] **Step 6: Run targeted tests**
- Run: `cargo test -p arco-cli --test cli_run --all-features`
- Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add crates/arco-cli/src/main.rs crates/arco-cli/src/lib.rs crates/arco-cli/src/driver.rs crates/arco-cli/src/execution.rs crates/arco-cli/tests/cli_run.rs
git commit -m "feat: add arco iis command"
```

### Task 6: Render IIS output in print-model style

**Files:**

- Modify: `crates/arco-cli/src/execution.rs`
- Possibly modify: `crates/arco-core/src/model/pretty.rs`
- Test: `crates/arco-cli/tests/cli_run.rs`

- [ ] **Step 1: Add a failing rendering test**
- Assert output shape is human-readable and stable, for example:
  - header with backend + scenario + analysis kind
  - `s.t.` block with conflicting rows
  - bounds block with conflicting lower/upper bounds
- The test should verify names, not fragile whitespace everywhere.

- [ ] **Step 2: Choose the least-hacky renderer**
- Recommended design:
  - render an IIS-specific ASCII report that mirrors `print-model` layout
  - reuse shared formatting helpers where possible
  - do not fake a meaningful objective
- Avoid the hack where the IIS is printed as a fake optimization model with a dummy objective unless extraction reuse turns out truly minimal and clean.

- [ ] **Step 3: Include both rows and bounds**
- Rows:
  - print by lowered row name with the original linear row expression
- Bounds:
  - print variable name, side, and numeric bound
- If solver returns only rows or only bounds, render what exists.

- [ ] **Step 4: Keep v1 provenance modest**
- Use lowered row names and variable names first.
- Do not block v1 on source-line spans because current KDL lowering does not preserve them.

- [ ] **Step 5: Run targeted tests**
- Run: `cargo test -p arco-cli --test cli_run --all-features iis`
- Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add crates/arco-cli/src/execution.rs crates/arco-core/src/model/pretty.rs crates/arco-cli/tests/cli_run.rs
git commit -m "feat: pretty print solver infeasibility analysis"
```

---

## Chunk 4: Low-level KDL regression coverage

### Task 7: Add a low-level infeasible KDL fixture

**Files:**

- Create: `examples/infeasible-low-level/input.kdl`
- Optionally create: `examples/infeasible-low-level/input.kdl`
- Test: `crates/arco-cli/tests/cli_run.rs`

- [ ] **Step 1: Create the minimal low-level fixture**
- Use a true low-level `model` + `scenario` pair, not the high-level technology/operation example.
- Recommended fixture:

```kdl
model "InfeasibleBounds" {
  control "x" lower=0 upper=1

  constraint "demand" {
    x >= 5
  }

  minimize "Obj" { x }
}

scenario "InfeasibleBoundsCase" {
  use "InfeasibleBounds"
}
```

- [ ] **Step 2: Add a CLI regression test for `arco iis`**
- Assert:
  - command succeeds on default backend
  - output contains `demand`
  - output contains `x`
  - output indicates the conflicting bound side

- [ ] **Step 3: Add a CLI regression test for `arco run` on the same fixture if useful**
- Assert `run` still reports infeasible cleanly, even though `iis` now provides the richer path.

- [ ] **Step 4: Run targeted tests**
- Run: `cargo test -p arco-cli --test cli_run --all-features infeasible`
- Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add examples/infeasible-low-level/input.kdl crates/arco-cli/tests/cli_run.rs
git commit -m "test: add low-level infeasible kdl fixture for iis"
```

---

## Chunk 5: Documentation and polish

### Task 8: Document the new command and behavior

**Files:**

- Modify: `docs/how-to/debug-infeasibility.md`
- Modify: `docs/how-to/README.md`
- Possibly modify: `README.md`

- [ ] **Step 1: Update the infeasibility guide**
- Add a CLI section showing:

```bash
arco iis path/to/input.kdl
```

- Explain:
  - it uses the configured solver
  - it asks the solver for native infeasibility analysis
  - it prints conflicting rows and bounds
  - Arco does not add its own relaxation in this path

- [ ] **Step 2: Document backend limitations**
- Note that support depends on the selected solver backend.
- If a backend cannot provide IIS analysis, the CLI returns a clear unsupported diagnostic.

- [ ] **Step 3: Update docs index if needed**
- Ensure the how-to index mentions CLI-based infeasibility diagnosis alongside slacks/elastic constraints.

- [ ] **Step 4: Run docs test if relevant**
- Run: `just docs-test`
- Expected: PASS if doctests are affected

- [ ] **Step 5: Commit**

```bash
git add docs/how-to/debug-infeasibility.md docs/how-to/README.md README.md
git commit -m "docs: document solver-native infeasibility analysis"
```

---

## Cross-cutting design rules

- `No Arco-side relaxation`
  - No internal slackification, no elastic wrapper, no repair utility in the `arco iis` path.
  - If a solver internally uses whatever black magic it uses, fine. Arco must not mutate the model into a different diagnostic problem.

- `Shared API first`
  - Backend-specific IIS APIs stay in `arco-highs` and `arco-xpress`.
  - The CLI consumes only the shared abstraction.

- `Solver decides how`
  - The CLI says “analyze infeasibility for this model”.
  - The backend decides whether it must solve first, disable presolve, or call a specific IIS routine.

- `Low-level fixture first`
  - Use a tiny low-level KDL model for the regression test, because it is stable, easy to read, and does not entangle domain normalization.

- `Pretty, not precious`
  - Reuse the visual structure of `print-model`.
  - Do not overreach into source-span mapping in v1. Current lowering does not preserve that metadata cleanly.

## Verification checklist

- Run:
  - `cargo fmt --all`
  - `cargo clippy -p arco-core -p arco-solver -p arco-highs -p arco-cli --benches --tests --examples --all-features -- -D warnings`
  - `cargo test -p arco-core -p arco-solver -p arco-highs -p arco-cli --all-features`
  - `cargo test -p arco-cli --test cli_run --all-features`
  - `just docs-test` if docs changed materially

## Expected v1 output shape

A good target is something like:

```text
IIS Analysis
 backend: highs
 scenario: InfeasibleBoundsCase

s.t.
 demand: x >= 5

Bounds
 0 <= x <= 1
 conflict: upper(x)
```

Not exact wording, but that general shape.

## One deliberate non-goal for v1

- No file/line spans back into KDL source.
- If you want that later, the next clean step is adding row provenance through lowering in `crates/arco-kdl/src/lowering.rs` and declaration spans in `crates/arco-kdl/src/source.rs`.
