# Release Python Distributions

Arco builds Python distributions after a Release Please PR is merged, validates
and freezes them in a release candidate, and publishes those exact files through
an immutable GitHub Release.

## Normal release

1. Merge the release PR after normal CI and release-contract checks pass.
2. The workflow builds and installs six wheels: `cp310-cp310` and `cp311-abi3`
   across Linux x86_64, macOS arm64, and Windows. It also builds the sdist.
3. Candidate assembly requires those files, the CLI artifacts, and the VSIX.
4. Publication checks repository immutability, tag identity, inventory, and asset
   digests before publishing the GitHub draft.
5. The PyPI job downloads and verifies the Python assets, then publishes them
   with trusted publishing.

Linux wheels use the pinned `manylinux_2_28_x86_64` image. ABI3 wheels are built
with CPython 3.11 and support later compatible CPython versions.

## Recover a failed build or upload

Use **Re-run failed jobs** on the original workflow run. Before the candidate is
assembled, failed builds can be retried. Once `release-candidate` exists, a full
rerun skips compilation and reuses it. Publication retains matching draft assets,
adds missing files, and rejects digest mismatches. Do not rebuild a frozen
candidate or manually replace release files to bypass a mismatch.

If the candidate has expired or a fix changes package bytes, follow the recovery
policy in [Release Policy](../../RELEASE_POLICY.md); normally a new version is
required.

## Retry PyPI publication

Manually run `release-please.yaml` with the published `vX.Y.Z` tag. The workflow
runs only the PyPI path, requires a published immutable release, downloads the
six wheels and sdist, verifies their SHA-256 digests, and uses `skip-existing: true`.
It does not accept a build ref or rebuild distributions.

If a bad file was already published, issue a new patch release. PyPI files are
not replaced.

[Back to how-to guides](./)
