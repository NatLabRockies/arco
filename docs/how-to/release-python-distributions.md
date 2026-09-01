# Release Python Distributions

Use this when a release is ready but a Python wheel build or PyPI upload fails.

## Normal release

1. Merge the release-please PR.
2. Let `release-please` dispatch the cargo-dist release.
3. Let each Python wheel job upload its artifact to the GitHub Release.
4. Let PyPI publish after all Python artifacts build.
5. Verify the release files on PyPI.

The wheel matrix is intentionally small: `cp310` and `abi3` across Linux, macOS arm64, and Windows. Linux x86_64 wheels build in the pinned `manylinux_2_28` container, supporting glibc 2.28 or newer. The `abi3` wheels target CPython 3.11 and support Python 3.12, 3.13, and 3.14. VS Code extension upload is separate and does not block PyPI publishing.

## If a wheel job fails

Use the smallest recovery path first:

1. If the failure looks transient, use GitHub's **Re-run failed jobs** on the failed workflow run.
2. If the workflow or packaging code needs a fix, merge the fix and rerun **Manual PyPI Release** from `main` with:
   - `ref`: the release tag or branch to rebuild
   - `skip_existing`: `true`
   - `release_tag`: the GitHub Release tag, when GitHub Release assets should be updated
   - `upload_to_release`: `true`, when GitHub Release assets should be updated
3. Rebuild all Python artifacts rather than stitching together artifacts from multiple runs.

Rebuilding the full reduced matrix is simpler and less error-prone than maintaining a separate artifact staging system.

## If PyPI partially uploaded files

PyPI files are immutable.

1. Keep already uploaded files unchanged.
2. Fix the missing or invalid artifact.
3. Rerun **Manual PyPI Release** with `skip_existing=true`.
4. If a bad file was uploaded, publish a patch release instead of trying to replace it.

The release-please PyPI publish path also uses `skip-existing: true` so reruns do not fail only because a previous attempt uploaded some files.

---

[Back to how-to guides](./)
