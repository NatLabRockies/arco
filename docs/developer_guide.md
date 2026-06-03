# Developer Guide

This guide is the practical local workflow for Arco contributors.

## Prerequisites

- Rust toolchain (repo targets Rust 1.85.x)
- `just` (task runner)
- `uv` (Python package/env runner)
- Python 3.12+ recommended for local bindings workflows

Install `just` if needed:

```bash
cargo install just --locked --version 1.43.0
```

## First-time setup

From repo root (recommended before running anything else):

```bash
just setup
just py-sync
```

## Build and run locally

### Install CLI into your cargo bin

```bash
cargo install --path ~/dev/arco/crates/arco-cli --force --locked
```

### Run CLI without installing

```bash
cargo run -p arco-cli -- --help
cargo run -p arco-cli -- run examples/dense-lp/input.kdl --compact
```

### Build Python extension in editable mode

```bash
just py-dev
```

## Day-to-day development loops

### Fast Rust loop (single crate)

```bash
just check-pkg arco-ops
just test-pkg arco-ops
just clippy-pkg arco-ops
```

### Fast Python loop

```bash
just py-fmt-check
just py-lint-check
just py-type
just py-test
```

### Docs and examples loop

```bash
just docs-test
just kdl-examples
```

## Architecture policy

Arco uses a repo-local architecture contract:

- `architecture-layers.toml`
- `scripts/check_architecture.py`

Run policy checks with:

```bash
just arch-check
```

Rules are strict:

- every workspace crate must be classified in `architecture-layers.toml`
- unknown/unclassified crates fail the check
- disallowed crate-to-crate layer edges fail the check

When adding a new crate, update `architecture-layers.toml` in the same change.

## GitHub workflow tips

- Open draft PRs early for visibility and CI signal.
- Keep PRs scoped; split unrelated changes before review.
- Reference issues in PR body (`Closes #123`) only when fully resolved.
- Prefer force-push only on your feature branch; avoid rewriting shared history.
- Re-run checks after resolving merge conflicts; don’t trust stale CI.
- If CI fails, post a short root-cause + fix note in the PR for reviewers.

Suggested PR checklist:

- [ ] `just arch-check` passes
- [ ] `just ci` passes
- [ ] docs updated for behavior/API changes
- [ ] migration/dependency impacts called out

## Full local CI-equivalent gate

Before pushing substantial changes:

```bash
just ci
```

This is the canonical pre-push validation path.

## Recommended pre-push checklist

1. Format/lint/tests pass for touched scope.
2. `just arch-check` passes.
3. `just ci` passes for broader changes.
4. Docs updated for any user-visible behavior/API changes.

## Building with optional solver features

Default builds exclude native IPOPT and Xpress runtimes. The shipped solver
selection command (`arco solver set ipopt`) is available in the default product
and returns a clear unavailable diagnostic unless the binary was compiled with
native IPOPT support.

To build the CLI with Xpress SDK support:

```bash
just build-cli-feature xpress
```

To build with both Xpress and IPOPT:

```bash
cargo build -p arco-cli --bin arco --features ipopt,xpress
```

> [!NOTE]
> IPOPT is intentionally outside the normal `--all-features` workspace path.
> The `arco-ipopt` crate compiles without native IPOPT libraries; this
> repository ships the selection surface and unavailable diagnostics, while
> native solve execution is provided by an external adapter build.

## Troubleshooting

### `just ci` fails in optional solver environments

Some solver backends may require external SDK/runtime setup depending on target.
Use package-scoped commands (`just test-pkg`, `just clippy-pkg`) while iterating,
then run full `just ci` in a fully provisioned environment.

### Python binding import/runtime issues

Rebuild editable extension:

```bash
just py-dev
```

Then re-run tests:

```bash
just py-test
```

### Architecture check fails after crate changes

Update `architecture-layers.toml` for:

- new crate classification
- intentional dependency overrides (only when justified)

## PR body template tip

Use concise sections in GitHub PR descriptions:

1. **Summary**: what changed
2. **Why**: problem/risk addressed
3. **Validation**: exact commands run
4. **Follow-ups**: explicit non-goals or deferred work
