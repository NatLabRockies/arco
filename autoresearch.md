# Autoresearch: solver-registry architecture clarity

## Objective

Improve Arco's structural quality for the solver-registry redesign by reducing cross-cutting binding logic, module coupling, and architectural complexity without changing behavior or relaxing rules.

## Metrics

- **Primary**: sentrux Quality (higher is better) — the structural quality gate score from `sentrux check`
- **Secondary**: unresolved imports, import edges — use these as directional signals while keeping the primary metric king

## How to Run

`./autoresearch.sh` — prints `METRIC ...` lines for the benchmark run.

## Files in Scope

- `bindings/python/src/lib.rs` — Python binding orchestration and solver resolution entry points
- `bindings/python/src/solver.rs` — Python solver/profile/selection types and backend-resolution helpers
- `bindings/python/arco/arco.pyi` — Python stub surface for solver selection/profile APIs
- `bindings/python/tests/test_arco_stub_operators.py` — stub and API regression coverage
- `autoresearch.sh` — benchmark runner for sentrux quality
- `autoresearch.checks.sh` — correctness checks for touched Python/Rust binding surfaces

## Off Limits

- `.sentrux/rules.toml` — do not weaken or edit the quality rules to game the metric
- `target/`, `dist/`, `.venv/`, generated build artifacts, or vendored dependencies
- unrelated in-flight edits outside the scoped binding and solver-architecture work

## Constraints

- No benchmark cheating: improve actual code structure, not the rules
- Preserve behavior and public solver APIs unless a change is required by the architecture redesign
- Keep edits minimal and localized
- Run the checks script after successful benchmark runs

## What's Been Tried

- Baseline sentrux check was noisy across the first few runs (Quality 6194 vs 6424), so the benchmark now runs sentrux three times and reports the median.
- The first correctness check attempt used `cargo check -p arco-python --all-features` and hit the Xpress SDK build-script guard; checks now use `--no-default-features`, and the gate also runs `just ci` for broader coverage while staying local and deterministic.
- Current branch already moves toward a generic solver registry API and stricter Python selection semantics.
- Tried thin `bindings/python/src/lib.rs` by moving solver/backend-resolution helpers into `bindings/python/src/solver.rs`; the median sentrux quality slipped to 6193, so that slice was not a win.
- Split the plain-model solve workflow into `bindings/python/src/model_solve.rs`; this was the first structural move that improved the sentrux quality signal, reaching 6427 with checks passing.
- Finished moving the remaining solver helper functions out of `bindings/python/src/lib.rs` into `bindings/python/src/solver.rs`; that produced only a small improvement to 6196, so the gain is likely incremental rather than transformative.
- A broader attempt to move the whole PyModel edit/inspection surface into `bindings/python/src/model_edit.rs` regressed quality and broke the build; the extraction was too wide and mixed too many binding concerns at once.
- A second attempt to split the inspection/export surface into its own `#[pymethods]` block failed because PyO3 treats that as a conflicting implementation for `PyModel`.
- A narrower helper-only extraction for inspection/export logic into `bindings/python/src/model_inspect.rs` succeeded and improved the sentrux quality signal to 6415 while keeping a single PyModel methods block.
- A follow-on helper-only split for naming/metadata/inspect wrappers into `bindings/python/src/model_metadata.rs` and `bindings/python/src/model_inspect.rs` regressed quality to 6181, so the gain from shrinking wrappers appears to have a limit.
- A cohesive naming/metadata-only cluster in `bindings/python/src/model_metadata.rs` also failed to improve the score, landing at 6187 with checks passing.
- Moving the composed-model solve orchestration into the solve helper module also regressed the score, suggesting the split was too incremental and still too wrapper-like.
- Adding `just ci` exposed an unrelated `clippy::float_cmp` failure in `crates/arco-scip/src/lib.rs`; that was fixed with epsilon-based assertions so the broader gate could run.
- Next likely win: stop peeling off accessor/wrapper clusters and look for a boundary that changes model behavior/body complexity in a meaningful way.
