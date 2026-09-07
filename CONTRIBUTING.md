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

- `CI` selects source, solver, package, documentation, grammar, and automation
  checks from the changed files. Its CI result check fails if any selected job
  fails or is cancelled. Packaging changes also check Python 3.10–3.14.
- CI calls the shared KDL overlay and workflow-quality workflows when relevant.
  The grammar corpus, tracked KDL files, workflow lint, security scan, and script
  tests must pass. Generated parser files must match the grammar.
- `release-please.yaml` uses `GITHUB_TOKEN` to open and update the release PR
  without tagging. Human pushes do not receive the same candidate approval gate.
- `build-candidate.yml` queues on those automated release PR updates. It produces
  no artifacts until a maintainer selects Approve workflows to run for the exact
  PR revision chosen as the cutoff.
- `promote-release.yml` validates an approved candidate run, merges the release
  PR, lets Release Please create the tag, and publishes the original files. Run it
  manually from the release PR's base branch with the candidate run ID.
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

### Check selection

| Change or event               | Checks                                                                                                                                            |
| ----------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| Ordinary PR                   | Version metadata and changed-file hooks; CI selects affected domains. Rust changes also test the Python bindings and KDL examples.                |
| Build tooling or CI routing   | All affected domains, including architecture and the platform compatibility checks.                                                               |
| Documentation                 | Documentation tests for `docs/**`; prose elsewhere uses the changed-file hooks.                                                                   |
| Push to `main` or `release/*` | Selected checks on the merged source; these runs also maintain shared caches and the benchmark baseline.                                          |
| Approved release candidate    | The complete source suite at the exact candidate commit, then native and package builds, then imports of the original wheels on Python 3.10–3.14. |

Source preflight checks version metadata, Rust formatting, and architecture before
compilation. Workflow quality combines actionlint, a blocking zizmor scan, script
tests, and the Cargo-dist plan in one job. Hooks run on the PR diff and do not set
up solver libraries. Benchmarks retain their job summary and regression guard.

Release Please PRs run their source suite inside the approved candidate workflow.
The standalone CI run skips that duplicate work. Candidate source validation omits
the three preliminary platform compilation jobs, development wheel builds, and
VS Code packaging because the following release jobs build the actual artifacts.
The candidate wheels are reused for compatibility checks; they are not rebuilt for
each interpreter. Linux source tests still exercise the solver and KDL behavior.

Use CI result as the aggregate source-check status when configuring branch rules;
keep the separate `prek` and `lint pr title` checks as well. Successful source CI
does not authorize a release: promotion requires a successful candidate run that
includes artifact checks. A manual CI run selects every source domain and the
ordinary compatibility matrix without publishing anything.

The current `py-type` recipe points at the Rust `bindings/python/src/` directory
and does not check the Python package. Python static typing remains a known gap;
a successful CI run currently covers formatting, lint, tests, and package imports,
not a working Python type check. Correcting the target exposes existing package
and stub diagnostics that need a separate typing cleanup.
