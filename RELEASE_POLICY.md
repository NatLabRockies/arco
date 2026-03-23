# Release Policy

This repository publishes two products from a single platform version stream:

- **Python package** (`arco` on PyPI)
- **CLI binary** (`arco` via GitHub Releases, powered by `cargo-dist`)

Rust crates and bindings evolve together under one release version.

## Goals

- Keep user-facing releases predictable.
- Keep Python and Rust behavior aligned in every published version.
- Keep release operations simple and hard to misuse.
- Allow each product to ship independently — one failure does not block the other.

## Versioning Model

- We use one shared semantic version across:
  - `Cargo.toml` workspace version
  - `bindings/python/pyproject.toml` project version
  - `bindings/python/uv.lock` package entry for `arco`
- `release-please` manages version bumps and opens a single release PR for the
  repository.

## Release PR Structure

- `release-please` opens one release PR (`arco` component).
- Notes are generated with the default changelog strategy so commit categories
  are preserved.

## Workflow Structure

Three independent workflows handle the release lifecycle:

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `release-please.yaml` | Push to `main` | Version management — creates release PR, tag, GitHub Release |
| `release-python.yaml` | Tag `arco-v*` or `workflow_dispatch` | Builds wheels, publishes to PyPI, uploads to release |
| `release-cli.yaml` | Tag `arco-v*` or `workflow_dispatch` | Builds CLI binaries via `cargo-dist`, uploads to release |

## Publishing Behavior

- `arco` is the published Python package (PyPI + GitHub Release wheel artifacts).
- The `arco` CLI binary is distributed via GitHub Releases with auto-generated
  shell and PowerShell installers powered by `cargo-dist`.
- Rust crates are internal implementation units and are not independently
  published from this repository.

## Independent Release Pipelines

Both product pipelines are triggered by the same `arco-v*` tag and run
independently in parallel:

### Phase 1: Version and Tag (release-please)

- `release-please` creates a release PR with version bumps.
- Merging the PR creates the `arco-v*` tag and a GitHub Release with the
  changelog body.

### Phase 2a: Python Pipeline (release-python.yaml)

1. Extracts version from the tag.
2. Builds Python wheels (3 platforms × 5 Python versions) via `maturin`.
3. Validates wheels with `twine check`.
4. Publishes to PyPI via OIDC Trusted Publishing.
5. Uploads `.whl` files to the GitHub Release.
6. Adds Python install instructions to the release notes.

### Phase 2b: CLI Pipeline (release-cli.yaml)

1. Runs `dist plan` to determine the per-platform build matrix.
2. Builds CLI binaries on native runners (macOS, Linux, Windows, ARM).
3. Builds global artifacts (shell + PowerShell installer scripts).
4. Uploads all CLI artifacts to the GitHub Release.
5. Adds CLI install instructions to the release notes.

### Independence

- Python failure does not block CLI release and vice versa.
- Partial releases are possible — if one pipeline fails, the other's artifacts
  are still published.
- Each pipeline's release note section uses HTML comment markers for
  idempotency (`<!-- python-install -->`, `<!-- cli-install -->`).

## Dry-Run Mode

Both product workflows support safe testing via `workflow_dispatch`:

- **Python dry-run** (`dry_run: true`): Builds and validates wheels without
  publishing to PyPI or uploading to a GitHub Release.
- **CLI dry-run** (tag input `dry-run`): Runs `dist plan` and builds all
  platform artifacts without uploading to a GitHub Release.

## Failure Handling

If a pipeline fails:

- Artifacts from successful build jobs remain available as workflow artifacts.
- The GitHub Release may have partial artifacts from the successful pipeline.
- The failed pipeline can be re-triggered via `workflow_dispatch` with the
  release tag.

## cargo-dist Configuration

CLI binary builds are configured in `[workspace.metadata.dist]` in `Cargo.toml`:

- `dispatch-releases = true` — the workflow uses `workflow_dispatch` and tag
  push instead of cargo-dist's default tag-only trigger.
- `allow-dirty = ["ci"]` — protects hand-edits to the generated workflow from
  being overwritten by `dist generate-ci`.
- Tag format: release-please creates `arco-v*` tags; the workflow strips the
  `arco-` prefix to produce `v*` tags that `dist` can parse.

## How To Read Release PRs

- Rust-only change:
  - It still creates a new `arco` release, because backend behavior affects the
    shipped Python wheel.
- Python-only change:
  - It creates a new `arco` release.
- Mixed change:
  - It creates a new `arco` release.
- Use commit scopes (`rust`, `python`, etc.) to make source of change explicit
  in release notes.

## Future Bindings

- New language bindings should follow the same platform version stream.
- Backend changes are considered cross-binding changes and should advance the
  shared version.

## Commit Conventions

- Use Conventional Commits (`feat:`, `fix:`, `perf:`, `chore:`, etc.).
- Non-conventional commit messages may be skipped or poorly classified in
  release notes.

## Forcing A Release PR

If you need to force a release:

1. Create a conventional commit that touches release-tracked paths.
2. Add a `Release-As` trailer to set the target version explicitly.

Example:

```text
chore(release): force 0.2.1

Release-As: 0.2.1
```

Forced versions apply to the single platform release and propagate to all
tracked version files.

Use `Release-As` only for one-off overrides. If a stale release PR resurfaces
an old forced version, close the stale PR and remove any stale draft release for
that tag before rerunning release automation.
