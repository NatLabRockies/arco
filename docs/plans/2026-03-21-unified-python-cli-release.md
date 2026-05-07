# Unified Python + CLI Release Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the custom CLI release workflow with `cargo-dist` while preserving `release-please` as the single version/tag authority and keeping Python package publishing in the same coordinated release pipeline.

**Architecture:** `release-please` continues to produce the shared version, tag, and base release. The workflow then builds Python distributions and CLI artifacts in parallel from the exact release SHA. `cargo-dist` packages CLI assets and generates installer metadata, while a final release step merges Python install text with the existing changelog and the dist-generated CLI install block before marking the release final.

**Tech Stack:** GitHub Actions, `release-please`, `cargo-dist`, Cargo workspace metadata, PyPI/TestPyPI publishing, GitHub Releases

---

## Chunk 1: Dist Configuration and Local Verification

### Task 1: Map release packaging ownership

**Files:**

- Inspect: `Cargo.toml`
- Inspect: `crates/arco-cli/Cargo.toml`
- Inspect: `.release-please-config.json`
- Inspect: `.github/workflows/release.yaml`
- Inspect: `RELEASE_POLICY.md`

- [ ] **Step 1: Confirm shared version source**
  - Verify the workspace version in `Cargo.toml` matches what `release-please` updates.
  - Verify `crates/arco-cli` is the only distributable CLI target.

- [ ] **Step 2: Document dist scope**
  - Record that `cargo-dist` should package only `arco-cli`, not Python bindings or internal crates.

- [ ] **Step 3: Identify target matrix parity**
  - Compare current CLI matrix in `.github/workflows/release-cli-binaries.yaml` with intended dist targets.
  - Expected: Linux, macOS, Windows parity remains intact.

### Task 2: Add and verify `cargo-dist` configuration

**Files:**

- Modify: `Cargo.toml`
- Possibly create: dist configuration file per current `cargo-dist` guidance

- [ ] **Step 1: Write failing expectation checklist**
  - Define expected outputs before changing config:
    - installer snippet generated
    - CLI artifacts generated
    - only `arco` binary included
    - tag format compatible with `arco-vX.Y.Z`

- [ ] **Step 2: Add minimal `cargo-dist` config**
  - Configure dist for GitHub release assets only.
  - Explicitly scope it to `crates/arco-cli`.

- [ ] **Step 3: Run local dist generation command**
  - Run the appropriate non-publishing dist command.
  - Expected: metadata/artifacts/snippet generated without attempting to publish.

- [ ] **Step 4: Verify outputs**
  - Check generated installer script references, artifact names, and release-note snippet.
  - Confirm tag/version assumptions match `release-please`.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml
git commit -m "build: configure cargo-dist for arco cli"
```

## Chunk 2: Replace CLI Workflow With Dist In CI

### Task 3: Swap reusable CLI workflow for dist packaging

**Files:**

- Modify: `.github/workflows/release.yaml`
- Delete or retire: `.github/workflows/release-cli-binaries.yaml`

- [ ] **Step 1: Write workflow expectations**
  - Expected behavior:
    - build CLI artifacts with dist
    - upload dist outputs
    - no duplicate CLI asset publishing path remains

- [ ] **Step 2: Replace `build-cli-binaries` job wiring**
  - Remove the reusable workflow call in `.github/workflows/release.yaml`.
  - Add a replacement job that runs `cargo-dist` against `release_sha`.

- [ ] **Step 3: Preserve SHA correctness**
  - Ensure checkout still uses `${{ needs.release-please.outputs.release_sha }}`.
  - This guarantees Python and CLI assets come from the same released commit.

- [ ] **Step 4: Upload dry-run outputs first**
  - In early iterations, upload generated dist outputs as workflow artifacts rather than release assets.
  - Expected: inspectable output without public publishing risk.

- [ ] **Step 5: Remove old custom binary packaging workflow**
  - Delete `.github/workflows/release-cli-binaries.yaml` once the dist job is verified.

- [ ] **Step 6: Commit**

```bash
git add .github/workflows/release.yaml .github/workflows/release-cli-binaries.yaml
git commit -m "ci: replace custom cli release workflow with cargo-dist"
```

### Task 4: Add non-publishing dry-run path

**Files:**

- Modify: `.github/workflows/release.yaml`

- [ ] **Step 1: Add workflow-dispatch or staging mode inputs**
  - Add a safe path to run release assembly without real publication.

- [ ] **Step 2: Route Python to non-production endpoint in dry-run mode**
  - Use TestPyPI or skip final publish while still building and smoke-testing.

- [ ] **Step 3: Route CLI to artifact-only mode in dry-run**
  - Keep CLI assets as workflow artifacts in dry-run mode.

- [ ] **Step 4: Emit assembled release body for inspection**
  - Upload the final markdown/text release body as an artifact.
  - Expected: reviewers can inspect the exact eventual notes.

- [ ] **Step 5: Commit**

```bash
git add .github/workflows/release.yaml
git commit -m "ci: add staged dry-run path for unified releases"
```

## Chunk 3: Final Release Assembly and Guardrails

### Task 5: Define a single final release-body writer

**Files:**

- Modify: `.github/workflows/release.yaml`

- [ ] **Step 1: Write target expectation**
  - Expected final release body contains:
    - Python install section
    - dist-generated CLI install section
    - preserved `release-please` changelog

- [ ] **Step 2: Read current release body in workflow**
  - Pull the existing body from the GitHub release created by `release-please`.

- [ ] **Step 3: Merge release sections deterministically**
  - Prepend or insert Python install instructions.
  - Preserve the dist-generated CLI block.
  - Preserve the existing changelog body.

- [ ] **Step 4: Update release once at the end**
  - Ensure only the final publish step edits the release body.
  - Expected: no last-writer-wins race with intermediate jobs.

- [ ] **Step 5: Commit**

```bash
git add .github/workflows/release.yaml
git commit -m "ci: assemble unified github release notes"
```

### Task 6: Gate finalization on both publish paths

**Files:**

- Modify: `.github/workflows/release.yaml`
- Modify: `RELEASE_POLICY.md`

- [ ] **Step 1: Write target expectation**
  - Expected: GitHub release is not finalized if either Python publish or CLI artifact publication fails.

- [ ] **Step 2: Make final release step depend on both tracks**
  - Keep final release publication dependent on:
    - Python publish success
    - dist CLI asset success

- [ ] **Step 3: Verify draft/latest behavior**
  - Ensure the release remains non-final until the final job succeeds.

- [ ] **Step 4: Update policy documentation**
  - Revise `RELEASE_POLICY.md` to match actual coordinated release behavior.

- [ ] **Step 5: Commit**

```bash
git add .github/workflows/release.yaml RELEASE_POLICY.md
git commit -m "docs: align release policy with unified publish flow"
```

## Chunk 4: Documentation and Verification

### Task 7: Document install paths for users

**Files:**

- Modify: `README.md`
- Possibly modify: `CONTRIBUTING.md`

- [ ] **Step 1: Keep Python install guidance explicit**
  - Preserve `uv add arco` and `pip install arco` docs.

- [ ] **Step 2: Add CLI release guidance**
  - Point users to GitHub Releases for standalone CLI installs.
  - Avoid hardcoding installer commands if dist-generated release notes are the source of truth.

- [ ] **Step 3: Update contributor notes if needed**
  - Explain the release split between Python and CLI.

- [ ] **Step 4: Commit**

```bash
git add README.md CONTRIBUTING.md
git commit -m "docs: document unified python and cli release install paths"
```

### Task 8: Verify with dry-run and staging release

**Files:**

- Verify: `.github/workflows/release.yaml`
- Verify: generated dist outputs
- Verify: staging release notes and assets

- [ ] **Step 1: Run local dist verification**
  - Run the exact non-publishing dist command.
  - Expected: installer snippet and CLI assets generated.

- [ ] **Step 2: Run workflow dry-run**
  - Trigger the non-production workflow path.
  - Expected:
    - Python artifacts built
    - CLI dist artifacts built
    - assembled release body artifact uploaded

- [ ] **Step 3: Inspect body and assets**
  - Confirm note ordering and asset naming.

- [ ] **Step 4: Run staging GitHub release**
  - Use draft/prerelease plus TestPyPI.
  - Expected:
    - release page contains merged notes
    - CLI assets attached
    - Python artifacts validated
    - release not finalized early

- [ ] **Step 5: Run failure drill**
  - Intentionally fail one side and verify final release is blocked.

- [ ] **Step 6: Run workflow quality checks**

```bash
just workflow-quality
```

- [ ] **Step 7: Commit**

```bash
git add .
git commit -m "test: verify unified release workflow in staging"
```

## Open Questions

- Exact `cargo-dist` config shape for a workspace with one distributable CLI and one Python binding tree.
- Whether the final release body should place Python install text above or below the dist-generated CLI section.
- Whether staging should use TestPyPI or skip package publication entirely on the first dry-run pass.
