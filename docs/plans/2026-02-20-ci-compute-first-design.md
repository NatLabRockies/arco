# Compute-First CI Design

## Context

Current CI runs a single `rust-validation` job and a Python matrix. Caching is
working, but Rust and Python lanes still perform duplicate compilation work,
and the current Rust test recipe triggers extra Python setup overhead.

The team priority is to reduce total compute minutes while preserving useful
parallel feedback and making CI behavior easier to understand.

## Goals

1. Reduce duplicate compilation and total CI compute cost.
2. Keep key checks visible as separate lanes (`fmt`, `clippy`, `test`).
3. Reuse Rust artifacts across Rust and Python validation jobs as much as
   practical.
4. Show contributors a clear CI execution plan and skip reasons.

## Non-Goals

1. Reworking release workflows.
2. Replacing `just`-based command entry points.
3. Fully eliminating all repeated compilation (not feasible with mixed build
   profiles and matrix variance).

## Chosen Approach

Adopt a compute-first staged CI architecture:

1. Add a `changes` job that computes path-based booleans used for conditional
   execution.
2. Add a `ci-plan` job that posts a human-readable summary of which lanes will
   run, what can run in parallel, and what is skipped.
3. Add a single `rust-prime` job that restores/builds/saves reusable Rust
   artifacts.
4. Split Rust checks into separate jobs (`rust-fmt`, `rust-clippy`,
   `rust-test`) and gate them with `needs: rust-prime`.
5. Gate Python validation matrix with `needs: rust-prime` so wheel builds can
   reuse primed Rust cache state.
6. Add a final `ci-required` aggregator job as the single required status check
   for branch protection UX.

## Cache Strategy

1. Use shared Rust cache namespace semantics across Rust and Python jobs to
   maximize cross-lane reuse.
2. Enable workspace crate caching in `Swatinem/rust-cache`:
   - `cache-targets: true`
   - `cache-workspace-crates: true`
3. Keep uv/Python caches separate and intact.
4. Save cache only on successful runs to avoid poisoned cache artifacts.

## Execution and Skip Rules

1. Use path filtering to skip heavy lanes when changes are docs-only or
   workflow-only where safe.
2. Keep required checks deterministic via `ci-required`, even when some child
   jobs are conditionally skipped.
3. Keep logs and summaries explicit so contributors know where time is spent.

## Risks and Mitigations

1. **Risk:** Wall-clock may increase slightly due to `rust-prime` gate.
   - **Mitigation:** Keep `ci-plan` and quality checks parallel; limit priming
     to useful compile work.
2. **Risk:** Cache key changes can reduce hit rates initially.
   - **Mitigation:** Roll out with conservative key design and monitor hit logs.
3. **Risk:** Conditional skip logic can hide required validation.
   - **Mitigation:** Keep skip rules narrow and auditable in one `changes` job.

## Verification Strategy

1. Validate workflow static quality: `just workflow-quality`.
2. Validate YAML structure and conditional expressions via PR CI dry run.
3. Compare CI run timings and cache-hit lines before/after deployment.

## Files Expected to Change

1. `.github/workflows/ci.yaml`
2. `.github/actions/setup-build-env/action.yml`
3. `justfile`
4. `docs/plans/2026-02-20-ci-compute-first-design.md`
