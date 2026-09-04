# Release Python Distributions

Arco's Python files are built after a Release Please PR is merged and are
published from the resulting immutable GitHub Release.

## Normal release

1. Release Please opens or updates the release PR.
2. Merge the release PR after the normal CI checks pass.
3. The release workflow builds `cp310-cp310` and `cp311-abi3` wheels on Linux
   x86_64, macOS arm64, and Windows, plus one Linux sdist.
4. It also builds the Cargo-dist CLI artifacts and VS Code extension, uploads
   all artifacts to the draft GitHub Release, and publishes that release.
5. The PyPI job downloads the Python files from the published immutable release
   and publishes them with trusted publishing.

Linux wheels use the pinned `manylinux_2_28_x86_64` image. The ABI3 wheel is
built with CPython 3.11 and uses the `cp311-abi3` tag.

## Release failure

Rerun the failed jobs for the merged release commit. Release Please reuses the
existing tag and draft release. Matching draft assets are retained and missing
assets are uploaded; a digest mismatch fails the retry. GitHub Immutable
Releases prevent changes after publication.

## PyPI failure or retry

Run `release-please.yaml` manually with the published `vX.Y.Z` tag. The workflow
then runs only the PyPI retry path. It verifies that the release is immutable,
downloads only its six wheels and one sdist, and uses `skip-existing: true`. It
does not accept a branch or arbitrary SHA and does not rebuild files.

If a bad file was published to PyPI, publish a new patch release. PyPI files are
not replaced.

[Back to how-to guides](./)
