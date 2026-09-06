# Release Policy

Arco ships one product snapshot with one shared semantic version across the
workspace, Python package, `vX.Y.Z` tag, and GitHub Release. Release Please
updates `Cargo.toml`, `bindings/python/pyproject.toml`, and
`bindings/python/uv.lock` together.

This pipeline accepts stable `vX.Y.Z` versions. Prerelease distribution naming
and promotion are outside its current release contract.

## Ownership

- Release Please owns the changelog, release PR, tag, and draft GitHub Release.
- Cargo-dist owns CLI target planning, builds, installers, checksums, and the
  canonical `dist-manifest.json`.
- `release-please.yaml` owns candidate assembly and publication.
- PyPI consumes the exact Python files from the published immutable release.

`cargo-dist-build.yaml` is a reusable build workflow, with no publication step.
`dist-workspace.toml` uses Cargo-dist's supported `allow-dirty = ["ci"]` option
for handwritten CI while retaining its GitHub matrix and hosting metadata. There
is no generated release workflow or separate manual Python build/publisher.

## Release flow

1. Release Please opens or updates a release PR. Normal CI, release-contract
   checks, and packaging-sensitive Python 3.10–3.14 smoke checks must pass.
2. A human merges the release PR. Release Please creates the tag immediately
   and leaves the GitHub Release in draft mode.
3. The workflow resolves the release from the workspace version and verifies
   that its tag identifies the triggering commit. It does not parse merge titles.
   Repository release immutability must be enabled before building proceeds.
4. Every builder checks out that exact commit SHA. Cargo-dist builds five CLI
   targets, archives, installers, checksums, and the final merged manifest. The
   Python matrix builds and installs six wheels: `cp310-cp310` and `cp311-abi3`
   on Linux x86_64, macOS arm64, and Windows. Linux ABI3 also builds the sdist.
   The VS Code job checks and packages one VSIX.
5. The assembly job verifies the exact inventory against the Cargo-dist manifest
   and required Python/VS Code assets, then stores a single `release-candidate`
   Actions artifact. This candidate is never overwritten.
6. The publisher downloads that candidate, checks immutability settings and tag
   identity again, and reconciles draft assets by SHA-256. Extra assets, absent
   digests, missing candidate files, and different bytes fail closed. Only missing
   assets are uploaded; the complete remote inventory and digests are verified
   before the draft is published. The published release must report immutable.
7. The PyPI job downloads the six wheels and sdist from the immutable GitHub
   Release, verifies their digests, and publishes with trusted publishing.

The candidate is built and validated **after merge**. This provides immutable
published releases; it is not promotion of binaries previously approved on a PR.
GitHub's release immutability locks the published assets and associated tag.

## Artifacts and recovery

Actions artifacts are retained for 90 days; the GitHub Release is the permanent
record. GitHub permits workflow reruns for a shorter period, currently 30 days.
Use **Re-run failed jobs** for the smallest recovery path.

- **Build failure:** rerun failed jobs on the original run. Before candidate
  assembly, a full rerun may replace intermediate build artifacts. No GitHub
  Release assets have been uploaded at this stage.
- **Assembly failure:** fix the failing build or inventory problem before
  publication. Do not manually add files to bypass the candidate checks.
- **Partial GitHub upload or publication failure:** rerun the publication job or
  failed jobs. A full rerun detects the existing `release-candidate`, skips all
  builders and assembly, and publishes those same bytes. Matching remote assets
  are retained; only missing assets are uploaded. A mismatch requires investigation,
  not rebuilding or replacing the existing asset.
- **Expired candidate:** recovery fails rather than silently rebuilding a frozen
  candidate. Recover the original files through an operator-reviewed procedure
  or issue a new release version.
- **PyPI failure:** manually dispatch `release-please.yaml` with the published
  `vX.Y.Z` tag. Only the PyPI path runs. It downloads verified release assets,
  uses `skip-existing: true`, and performs no compilation or GitHub uploads.

If a published package is incorrect, issue a new patch release. Do not replace
published assets. A code or workflow fix that changes release bytes needs a new
version rather than a rebuild under the old tag.

## Repository rollout

Before the first production release, an administrator must:

1. Enable GitHub Immutable Releases. The pipeline checks this setting and refuses
   to publish while it is disabled; this PR does not change repository settings.
2. Add a `v*` tag ruleset blocking tag updates and deletion before publication.
   Permit Release Please to create tags, without granting routine mutation rights.
3. Configure `RELEASE_PLEASE_TOKEN` with repository contents and pull-request
   write access, plus **Administration: read**. GitHub requires administration
   read access for the [immutability-settings check](https://docs.github.com/en/rest/repos/repos#check-if-immutable-releases-are-enabled-for-a-repository);
   the default Actions token cannot supply it. Administration write access is
   not needed by the workflow.
4. Require normal CI and release-contract checks on release PRs, and require the
   branch to be up to date before merging.
5. Configure the PyPI trusted publisher for `.github/workflows/release-please.yaml`
   and the `pypi` environment.
6. In a sandbox repository, prove the entire lifecycle: draft/tag creation,
   platform builds and smoke checks, rejection with immutability disabled,
   interrupted upload recovery from unchanged candidate bytes, published asset/tag
   locking, and tag-only PyPI recovery.

Run `just release-check` with Cargo-dist 0.31.0 installed to validate real planning
and the publication/recovery contracts locally. The tests use a GitHub process
boundary double; they do not create releases or publish packages. Cross-platform
builds and live GitHub locking still require the sandbox run above.

## Branches and release notes

Before `1.0`, releases come from `main`. Supported maintenance lines may later
use `release/1.0` style branches; backport release workflow fixes to each line.
A workflow and its local actions run from that branch's triggering commit.

Release Please remains the source of truth for changelog content. For a one-off
version override, use a `Release-As` footer, for example:

```text
chore(release): force 0.3.0

Release-As: 0.3.0
```

The version consistency check must pass before a release PR is merged.
