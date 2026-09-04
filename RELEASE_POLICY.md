# Release Policy

Arco ships one product snapshot with one shared semantic version. The version is
kept in sync across:

- `Cargo.toml` workspace metadata
- `bindings/python/pyproject.toml`
- `bindings/python/uv.lock`
- the `vX.Y.Z` Git tag
- the GitHub Release

## Ownership

- Release Please owns Conventional Commit parsing, the changelog, the release
  PR, and the draft GitHub Release and tag created after that PR is merged.
- Cargo-dist owns CLI target planning, builds, installers, checksums, and
  distribution manifests.
- GitHub Actions owns the build matrix and the final release upload.
- PyPI consumes the published GitHub Release. It never rebuilds a release.

## Workflow topology

| Workflow | Trigger | Responsibility |
| --- | --- | --- |
| `release-please.yaml` | Push to `main` or `release/*`, or manual dispatch | Create or update the Release Please PR; build and publish releases after merge; retry PyPI from a published tag |
| `cargo-dist-build.yaml` | Called by `release-please.yaml` | Run Cargo-dist's plan and build stages without publishing |

The full release matrix runs after the Release Please PR is merged. Ordinary PR
CI remains the pre-merge validation path. This avoids trying to promote an
artifact between two independent workflow runs, which GitHub Actions does not
support without API lookup code.

## Release flow

1. Release Please opens or updates the release PR.
2. A human merges the release PR.
3. Release Please creates the `vX.Y.Z` tag and a draft GitHub Release.
4. Cargo-dist plans and builds the CLI artifacts from that tag.
5. The Python matrix builds `cp310-cp310` and `cp311-abi3` wheels on Linux
   x86_64, macOS arm64, and Windows. The Linux ABI3 cell also builds one sdist.
6. The VS Code workflow step builds one VSIX.
7. The aggregate job downloads the immutable Actions artifacts from the same
   workflow run.
8. The GitHub CLI uploads missing artifacts to the existing draft, verifies the
   SHA-256 digest of existing assets, and publishes it. Draft retries never
   replace an existing asset; a digest mismatch fails closed. Published
   immutable releases cannot be changed.
9. The PyPI job in the release workflow downloads the Python files from the
   published immutable release and publishes them with trusted publishing.

The Linux wheels use the pinned `manylinux_2_28_x86_64` image. The release
artifacts include Cargo-dist archives, installers, checksums, manifests, six
Python wheels, one Python sdist, and the VSIX.

## Artifacts and retries

Actions artifacts are an internal handoff within the release workflow and are
retained for 90 days. The GitHub Release is the permanent distribution record.
GitHub's artifact download action verifies the artifact digest during the
same-run handoff, while Cargo-dist remains the authority for CLI checksums and
manifests.

If a build fails, rerun the failed jobs for the merged release commit. The
workflow reuses the existing release tag and draft. If an upload fails after
some assets were uploaded, the next retry retains matching assets and uploads
only missing ones. A different digest stops the retry. Once the release is
published, GitHub Immutable Releases prevent asset or tag mutation.

If PyPI publication fails, run `release-please.yaml` manually with the published
`vX.Y.Z` tag. The workflow then runs only the PyPI retry path. It downloads only
from that immutable GitHub Release, uses `skip-existing: true`, and does not
compile or upload GitHub assets. Keeping this job in the non-reusable release
workflow also preserves PyPI trusted publishing, which does not support
reusable workflows.

## GitHub rollout requirements

Before enabling releases, an administrator must:

1. Enable GitHub Immutable Releases.
2. Add a `v*` tag ruleset blocking tag updates and deletion.
3. Permit the narrowly scoped Release Please token to create the tag and
   release. Release Please creates a lightweight tag through the GitHub API.
4. Configure `RELEASE_PLEASE_TOKEN` if the default Actions token cannot satisfy
   the tag ruleset or release permissions.
5. Require the normal CI checks on the Release Please PR and require branches
   to be up to date before merging.
6. Configure the PyPI trusted publisher for `.github/workflows/release-please.yaml`
   and the `pypi` environment.
7. Prove draft release recovery, tag-rule enforcement, immutable release
   locking, and tag-only PyPI retry in a sandbox repository.

The workflows do not change repository settings, create live releases during
local validation, or publish packages outside the release jobs.

## Branch policy

Before `1.0`, releases come from `main`. After `1.0`, a supported maintenance
line may use a `release/1.0` style branch. Keep release workflows and tooling
sourced from `main`.

## Release notes and versions

Release Please is the source of truth for changelog content. Its sections are
configured in `.github/release-please-config.json`. Use `Release-As` only for a
specific one-off version override:

```text
chore(release): force 0.3.0

Release-As: 0.3.0
```

The version consistency check must remain green before merging a release PR.
