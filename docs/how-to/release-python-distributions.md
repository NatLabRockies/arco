# Release Python Distributions

Use this guide when a release is ready but one Python wheel or sdist build fails on a specific OS/Python lane. Arco stages Python distributions on a GitHub Release first, validates the complete artifact manifest, and only then publishes to PyPI.

## Release model

Python distributions use one canonical matrix in `.github/python-distribution-matrix.json`:

- Python lanes: `cp310` and `abi3`
- Platforms: `linux`, `macos-x64`, `macos-arm64`, and `windows`
- Expected release payload: eight wheels plus one sdist

The workflows separate three concerns:

1. `python-distributions-stage.yaml` builds selected wheel lanes and optionally uploads them to a GitHub Release.
2. `pypi-publish-staged.yaml` downloads staged release assets, validates the manifest, runs `twine check`, and publishes to PyPI.
3. `pypi-manual-release.yaml` is the operator-facing recovery workflow that can stage all lanes or one failed lane.

## Normal release path

1. Merge the release-please release PR.
2. Wait for `release-please` to dispatch and complete the cargo-dist release.
3. Let `Stage Python distributions` build all matrix lanes and upload them to the GitHub Release.
4. Let `Publish staged distributions to PyPI` validate and publish the staged assets.
5. Verify the PyPI release files for the version.

## Rerun one failed OS/Python lane

If one wheel lane fails, do not publish immediately. Rebuild only the failed lane:

1. Open **Actions → Manual PyPI Release**.
2. Use these inputs:
   - `source_ref`: the release tag, for example `v0.7.0`
   - `tooling_ref`: the current release-tooling commit SHA for reproducible recovery, or `main` when intentionally using the latest tooling
   - `release_tag`: the same release tag, for example `v0.7.0`
   - `platform`: the failed platform, for example `windows`
   - `python`: the failed Python lane, for example `abi3`
   - `upload_to_release`: `true`
   - `publish`: `false`
3. Confirm the workflow uploads the rebuilt file to the GitHub Release.
4. Repeat for any other failed lane.
5. Run **Publish Staged PyPI Distributions** for the release tag after all expected files are present.

## Stage every Python artifact without publishing

Use this when you want a full rehearsal before PyPI publication:

1. Open **Actions → Manual PyPI Release**.
2. Set:
   - `source_ref`: release tag
   - `tooling_ref`: the current release-tooling commit SHA, or `main` when intentionally using the latest tooling
   - `release_tag`: release tag
   - `platform`: `all`
   - `python`: `all`
   - `upload_to_release`: `true`
   - `publish`: `false`
3. Inspect the GitHub Release assets.
4. Run **Publish Staged PyPI Distributions** when the staged manifest is complete.

## Publish staged assets

Run **Actions → Publish Staged PyPI Distributions** with:

- `release_tag`: the release tag containing staged Python assets
- `tooling_ref`: the current release-tooling commit SHA, or `main` when intentionally using the latest tooling
- `skip_existing`: `true` for recovery runs after partial upload attempts

The publish workflow refuses to upload to PyPI unless the staged assets contain every expected wheel and the sdist passes license-file validation.

## If PyPI partially uploaded files

PyPI files are immutable. If some files uploaded before a later file failed:

1. Fix or rebuild the missing/invalid staged artifact.
2. Keep already uploaded files unchanged.
3. Rerun **Publish Staged PyPI Distributions** with `skip_existing=true`.
4. If a bad file was uploaded, publish a patch release instead of trying to replace it.

## Drift guard

`just workflow-quality` runs `scripts/ci/check_python_distribution_workflows.py`. That check fails when manual and release-please workflows stop using the shared staging/publish workflows or when someone reintroduces an inline Python release matrix.

Update `.github/python-distribution-matrix.json` first when the supported wheel matrix changes.

---

[Back to how-to guides](./)
