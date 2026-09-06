# Release Python Distributions

Merge the Release Please PR after CI passes. Its tag starts Cargo-dist's generated
release workflow, including the Python build hook. Each of the six wheels is
installed and imported before the GitHub draft is published.

Cargo-dist's post-announce hook then dispatches `publish-pypi.yml`. This separate
workflow downloads the six wheels and sdist from the immutable GitHub Release,
verifies their release attestations, and publishes through PyPI trusted publishing.
Check its result separately from the Cargo-dist run.

## Retry

- For a build failure, use **Re-run failed jobs** on the original Cargo-dist run.
- For an interrupted GitHub upload, follow the draft-asset recovery procedure in
  [Release Policy](../../RELEASE_POLICY.md).
- For PyPI failure, manually run `publish-pypi.yml` with the published `vX.Y.Z`
  tag. It downloads and verifies the original files and uses `skip-existing: true`.

The PyPI workflow never rebuilds distributions. If a published package needs a
code fix, issue a new patch release.

## Setup

Register `publish-pypi.yml` and the `pypi` environment with PyPI trusted publishing.
The top-level workflow is intentional: PyPI does not support reusable workflows
as trusted publishers. See [Release Policy](../../RELEASE_POLICY.md) for repository
permissions and immutability requirements.

[Back to how-to guides](./)
