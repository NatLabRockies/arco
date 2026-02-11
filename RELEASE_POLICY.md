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

- `arco-python` is the published package (PyPI + GitHub release artifacts).
- Rust crates are internal implementation units and are not independently
  published from this repository.

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
