# Contributing

Arco is developed as a workspace so Rust crates, the Python-facing API surface,
and tooling can evolve together. Before opening a PR, sync your branch with the
latest `main` so manifests, docs, and active work stay aligned.

## Development workflow

Use workspace `just` targets as the default contributor entry point. This repository requires `just >= 1.43.0` because the root `justfile` uses recipe group attributes unsupported by older releases.

If needed, install a compatible version with:

```bash
cargo install just --locked --version 1.43.0
```

Use workspace `just` targets as the default contributor entry point:

```bash
just fmt
just rust-clippy
just check
```

Treat clippy and compiler warnings as errors and fix them immediately.

For a full local gate before PR creation:

```bash
just ci
```

### Solver-specific validation

Shipped solvers can be smoke-tested locally with isolated config:

```bash
just smoke-solver highs
just smoke-solver scip
just smoke-solver xpress xpress
```

IPOPT is an external/native adapter and is not shipped by default.
`arco solver set ipopt` is available in the default product but returns a
clear unavailable diagnostic. To verify the unavailable diagnostic:

```bash
just build-cli
just smoke-solver-ipopt-unavailable
```

To use native IPOPT (requires system IPOPT libraries):

```bash
just smoke-solver ipopt ipopt
```

For Python commands, use `uv` consistently:

```bash
uv run pytest
```

If you touch Python bindings or Python test harnesses, keep execution under
`uv run` so environments and dependency resolution stay reproducible.

## GitHub automation

The repository ships GitHub Actions for package validation and release:

- `CI` runs source, solver, package, and docs checks. Packaging changes also run
  Python 3.10–3.14 wheel smoke checks.
- KDL overlay validation runs the Tree-sitter grammar corpus and parses tracked
  KDL files. Parser errors fail CI; generated parser files must match the grammar.
- `release-please.yaml` accumulates changes in the release PR without tagging.
- `build-candidate.yml` builds CLI, Python, and VS Code candidate files when a
  maintainer requests a release cutoff.
- `promote-release.yml` validates an approved candidate run, merges the release
  PR, lets Release Please create the tag, and publishes the original files.
- `publish-pypi.yml` downloads and verifies immutable release assets for trusted
  PyPI publication. Check its result separately from release promotion.
- Run `just release-check` after changing Cargo-dist configuration, and
  `just workflow-quality` after changing workflow definitions. Run
  `just script-test` for the benchmark, Python packaging, and KDL helpers; these
  tests also run in GitHub Actions Quality and `just ci`.
- Releases follow one platform version stream (`arco`) that updates workspace
  and Python package versions together.
- `arco` publishes artifacts and releases; Rust crates are internal and
  versioned as part of the same platform release.
- Shared package smoke logic lives in `scripts/python_package_smoke.py`.
  See [Release Python distributions](docs/how-to/release-python-distributions.md)
  for the platform matrix and instructions for adding a wheel target.
- For policy and operator guidance, use [`RELEASE_POLICY.md`](RELEASE_POLICY.md)
  as the source of truth.

## Testing

Use targeted tests first, then broaden based on risk:

- Run tests for the crates and modules you changed.
- Add regression tests for every bug fix.
- Prefer realistic unit and end-to-end coverage over mock-heavy tests.
- When touching optimization plumbing, include cases that exercise memory
  behavior and hot paths.
- For benchmark-sensitive changes, gate regressions against a baseline artifact:
  `just bench-gate <baseline.jsonl> <candidate.jsonl> <duration_pct> <memory_pct>`.

Suggested baseline command:

```bash
just test
```

And when Python tests are present:

```bash
uv run pytest
```

Call out any skipped suites, feature flags, or known test gaps in the PR
description.

## Documentation updates

Documentation ships with behavior changes.

- If docs do not exist for a feature, the feature is not complete.
- Update `README.md` for user-facing behavior and onboarding changes.
- Update `AGENTS.md` when contributor workflow or engineering rules change.
- Keep architecture and design docs in sync if you introduce new docs
  directories (for example `docs/` or `rfd/`).

Include reproduction or validation steps in docs when they help others verify
the change quickly.
