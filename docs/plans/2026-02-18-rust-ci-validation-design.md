# Rust CI Validation Design

## Context

Current GitHub CI (`.github/workflows/ci.yaml`) validates Python wheel build/import
and docs doctests, but it does not enforce Rust formatting, linting, and test
gates on pull requests and pushes to `main`.

The repository already defines the canonical Rust validation commands in
`justfile`:

- `just fmt-check`
- `just clippy`
- `just test`

To keep command behavior consistent across local development and CI, the Rust
CI job should invoke `just` recipes directly instead of duplicating raw cargo
commands in workflow YAML.

## Goals

1. Enforce Rust quality gates in CI for PRs and pushes to `main`.
2. Keep local/CI behavior consistent by using `just` commands.
3. Preserve existing Python validation matrix behavior.
4. Keep workflow changes small and readable.

## Non-Goals

1. Changing release workflow behavior.
2. Reworking Python CI strategy.
3. Refactoring `justfile` recipe semantics.

## Chosen Approach

Add a new `rust-validation` job to `.github/workflows/ci.yaml` that runs on the
same triggers as existing CI and executes:

1. `just fmt-check`
2. `just clippy`
3. `just test`

The job will run on `ubuntu-latest` with:

- `actions/checkout`
- `taiki-e/install-action@just` to install `just`
- existing composite action `./.github/actions/setup-build-env` for Rust toolchain
  and cache setup

`python-version` input to `setup-build-env` can use `3.12` (already used for
doctests and sufficient for the action's `uv` install + license sync step).

## Alternatives Considered

### A) Raw cargo commands in workflow YAML

Pros: one less tool installation step.
Cons: duplicates canonical commands and risks drift from local checks.

### B) Reuse `just ci`

Pros: single command.
Cons: includes Python/docs scope not needed for Rust-only gate; less focused
failure signals.

## Risks and Mitigations

1. **Risk:** `just` not available in runner.
   - **Mitigation:** explicit install step in `rust-validation` job.
2. **Risk:** minor CI runtime increase.
   - **Mitigation:** run Rust and Python jobs in parallel.
3. **Risk:** false confidence if `just` recipes diverge.
   - **Mitigation:** make CI authoritative by directly calling `just` recipes in
     the Rust job.

## Testing and Verification

Local verification before completion:

1. `just fmt-check`
2. `just clippy`
3. `just test`

Workflow syntax check:

1. Ensure `.github/workflows/ci.yaml` remains valid YAML and existing job
   definitions are unchanged except intentional additions.

## Files to Update

1. `.github/workflows/ci.yaml`
