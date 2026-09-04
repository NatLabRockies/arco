# Release Python Distributions

Arco's Python files are built before a Release Please PR is merged and are
published from the immutable GitHub Release afterward.

## Normal release

1. Release Please opens or updates the release PR.
2. Wait for the required `Release candidate` check. It builds
   `cp310-cp310` and `cp311-abi3` wheels on Linux x86_64, macOS arm64, and
   Windows, plus one Linux sdist.
3. Merge the release PR only after the candidate check passes.
4. The finalizer verifies the candidate against the merged Git tree, creates the
   protected tag, uploads the candidate files to a draft GitHub Release, and
   publishes the immutable release.
5. `pypi-release.yaml` downloads the Python files from that published release,
   verifies their GitHub SHA-256 digests, and publishes them with trusted
   publishing.

Linux wheels use the pinned `manylinux_2_28_x86_64` image. The ABI3 wheel is
built with CPython 3.11 and uses the `cp311-abi3` tag.

## Candidate failure

A failed candidate check blocks the Release Please PR. Fix the PR and wait for
its new candidate run. Do not build a replacement release from an arbitrary
branch or upload files directly to a GitHub Release.

## Finalizer failure

Rerun `Finalize immutable release` for the same merged commit. It locates the
candidate by the exact release version, PR number, and Git tree, then verifies
the existing tag and any draft assets before resuming. Existing assets are never
replaced.

The candidate Actions artifact is retained for 90 days. If it has expired or
the final tree differs, stop and create a new release PR instead of rebuilding
under the old release identity.

## PyPI failure or retry

PyPI is independent of GitHub Release publication. Rerun `PyPI release` with
`workflow_dispatch` and the published `vX.Y.Z` tag. The workflow downloads only
from that immutable GitHub Release, verifies the asset digests, and uses
`skip-existing: true`. It does not accept a branch or arbitrary SHA and does
not rebuild files.

If a bad file was published to PyPI, publish a new patch release. PyPI files are
not replaced.

[Back to how-to guides](./)
