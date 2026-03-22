# Release Policy

This repository publishes the Python package (`arco`) from a single platform
version stream. Rust crates and bindings evolve together under one release
version.

## Goals

- Keep user-facing releases predictable.
- Keep Python and Rust behavior aligned in every published version.
- Keep release operations simple and hard to misuse.

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

## Publishing Behavior

- `arco` is the published package (PyPI + GitHub release artifacts).
- The `arco` CLI binary is distributed via GitHub Releases with auto-generated
  installers powered by `cargo-dist`.
- Rust crates are internal implementation units and are not independently
  published from this repository.
- Release order is:
  1. `release-please` creates a draft GitHub release and tag.
  2. CI builds Python wheels in parallel across platforms and Python versions.
  3. CI builds CLI binaries via `cargo-dist` in parallel.
  4. CI validates downloaded wheel metadata in one place with `twine check`.
  5. CI publishes Python wheels to PyPI.
  6. CI assembles unified release notes combining Python install instructions,
     CLI install snippets from cargo-dist, and the changelog.
  7. CI uploads all artifacts and marks the GitHub release as final.

## Unified Release Flow

The release workflow coordinates both Python and CLI distributions:

- **Phase 1**: `release-please` produces the version, tag, and initial draft release
- **Phase 2**: Parallel builds for Python wheels and CLI artifacts
  - Python wheels built via `maturin` for Linux, macOS, Windows
  - CLI binaries built via `cargo-dist` with shell and PowerShell installers
- **Phase 3**: Publishing with gating
  - Downloaded wheel artifacts are validated centrally via `twine check`
  - PyPI publish must succeed before final release
  - CLI artifact generation must succeed before final release
  - Either failure blocks the final GitHub release publication
- **Phase 4**: Final assembly
  - Merged release notes with Python and CLI install sections
  - All artifacts uploaded to the GitHub release
  - Release marked as final

## Dry-Run Mode

The workflow supports safe testing via `workflow_dispatch` input:

- **Dry Run Mode** (`dry_run: true`):
  - Builds all artifacts without publishing
  - Runs centralized artifact validation (`twine check`) without publishing
  - Assembles unified release notes for preview/validation
  - Uploads artifacts as workflow artifacts for inspection
  - Skips PyPI and GitHub release publication
  - Useful for validating the build pipeline

## Failure Handling

The final release publication is gated on successful completion of both:

- Python wheel publishing (PyPI)
- CLI artifact generation (cargo-dist)

If either path fails:

- Artifacts from successful builds remain available as workflow artifacts
- The GitHub release remains in draft state (or is not created in dry-run mode)
- The failure must be investigated and the release manually retried or fixed

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
