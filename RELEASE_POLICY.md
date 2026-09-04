# Release Policy

Arco ships one product snapshot with one shared semantic version. The version is
kept in sync across:

- `Cargo.toml` workspace metadata
- `bindings/python/pyproject.toml`
- `bindings/python/uv.lock`
- the `vX.Y.Z` Git tag
- the GitHub Release

## Ownership

- Release Please owns Conventional Commit parsing, the changelog, and the
  release PR. It does not create tags or releases.
- Cargo-dist owns CLI target planning, builds, installers, checksums, and
  distribution manifests.
- GitHub Actions owns the candidate packet, protected tag handoff, draft
  release, and publication order.
- PyPI is a consumer of the published GitHub Release. It never rebuilds a
  release.

## Workflow topology

| Workflow | Trigger | Responsibility |
| --- | --- | --- |
| `release-please.yaml` | Push to `main` or `release/*` | Open or update the Release Please PR only |
| `release-candidate.yaml` | Release Please PR opened, updated, or reopened | Build and test the complete release candidate before merge |
| `release-finalize.yaml` | Push to `main` or `release/*` | Verify the exact candidate, create the annotated tag, upload a draft, and publish it |
| `pypi-release.yaml` | Published GitHub Release or manual tag retry | Verify immutable release assets and publish the Python files |

The required `Release candidate` check must be required on Release Please PRs,
and the branch rule must require the PR to be up to date. Ordinary PRs run the
normal CI checks without the release matrix.

## Candidate packet

The Release Candidate workflow checks out GitHub's pull request merge ref. It
runs:

- Cargo-dist's native `plan` and build matrix for CLI artifacts
- `cp310-cp310` and `cp311-abi3` wheels on Linux x86_64, macOS arm64, and
  Windows
- one Linux source distribution
- one VS Code extension package

The Linux wheels use the pinned `manylinux_2_28_x86_64` image. The candidate
jobs upload their outputs as immutable Actions artifacts. The aggregate job
uses GitHub's native `actions/upload-artifact/merge` action to create one
release-bound artifact named:

```text
release-candidate-v<version>-pr<PR>-tree<git-tree>
```

The merged artifact retains its source artifact directories, preventing silent
same-name overwrites. Its `RELEASE_METADATA.json` binds the version, PR, PR head
SHA, tested merge SHA, source tree, workflow run, and artifact name. The native
Actions artifact ID and SHA-256 digest are recorded in the workflow summary;
they are assigned by GitHub after upload and therefore cannot be placed inside
the uploaded file without creating a circular checksum.

GitHub's artifact download action verifies the native artifact digest during
promotion. Cargo-dist remains the authority for the CLI checksums and manifests
inside the candidate packet. The candidate packet and its component artifacts
are retained for 90 days.

## Promotion state machine

1. Release Please opens or updates a PR.
2. The required candidate check builds the merge ref and uploads one immutable
   candidate artifact. A missing matrix cell or required asset fails the check.
3. A human merges the PR.
4. The finalizer finds the candidate by exact PR/tree-bound artifact name and
   verifies its native artifact identity.
5. The finalizer verifies the candidate metadata, required six-wheel matrix,
   Linux sdist, VSIX, Cargo-dist manifests, checksums, release version, and
   merged Git tree.
6. Only after those checks pass, it creates or verifies the annotated `vX.Y.Z`
   tag at the merged commit.
7. It creates or resumes a draft GitHub Release, uploads missing assets without
   replacement, verifies the complete asset set, and publishes the release.
8. GitHub Immutable Releases then lock the tag and release assets.
9. PyPI downloads the exact Python assets from that immutable release and uses
   trusted publishing with `skip-existing: true`.

A squash merge may change the commit SHA. It is accepted only when the final
commit's Git tree equals the candidate's recorded tree. A different tree stops
the finalizer before tag creation.

## Retry behavior

- A failed candidate blocks the Release Please PR. Fix the PR and wait for a
  new candidate run.
- A finalizer failure before or after tag creation is retried by rerunning the
  same workflow. The finalizer verifies the existing tag and draft before
  resuming, and never rebuilds artifacts.
- Existing draft assets must have the same GitHub SHA-256 digest as the
  candidate files. A mismatch stops the retry rather than replacing an asset.
- A failed PyPI publish is retried with `pypi-release.yaml` using only the
  published immutable tag. It does not accept a branch or arbitrary SHA and it
  does not build files.

## GitHub rollout requirements

Before enabling the workflows, an administrator must prove these behaviors in a
sandbox repository:

1. Enable GitHub Immutable Releases.
2. Add a `v*` tag ruleset blocking tag updates and deletion. Permit tag
   creation only for the narrowly scoped finalizer GitHub App integration. Do
   not assume `GITHUB_TOKEN` bypasses a ruleset.
3. Store that App installation token as `RELEASE_TAG_TOKEN` with only the
   repository permissions required to read Actions artifacts, create tags, and
   create and publish releases.
4. Configure `RELEASE_PLEASE_TOKEN` as a bot token for Release Please. This is
   required when the generated PR must trigger its candidate workflow; the
   default Actions token suppresses downstream workflow events.
5. Require the `Release candidate` check and an up-to-date branch on the
   Release Please PR.
6. Prove cross-run artifact download, native digest failure, draft recovery,
   tag-rule enforcement, immutable release locking, and tag-only PyPI retry.

The workflow does not change repository settings, create live tags, publish live
releases, push commits, or publish packages from this change.

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
