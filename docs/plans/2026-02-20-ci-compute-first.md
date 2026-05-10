# Compute-First CI Workflow Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Reduce duplicate CI computation by introducing a Rust priming stage, splitting Rust checks into clear lanes, and maximizing cache reuse across Rust and Python validation.

**Architecture:** Build a staged CI graph with `changes` -> `ci-plan` + `rust-prime` -> parallel Rust check lanes + Python matrix -> `ci-required` aggregator. Keep `just` recipes authoritative and adjust `just test` to avoid unnecessary Python environment bootstrapping in Rust-only validation.

**Tech Stack:** GitHub Actions workflow YAML, composite GitHub Action config, Rust toolchain + Swatinem/rust-cache, `just`, `uv`.

---

### Task 1: Add path-based change detection outputs

**Files:**

- Modify: `.github/workflows/ci.yaml`
- Test: `.github/workflows/ci.yaml`

**Step 1: Add `changes` job with path filters**

```yaml
changes:
  runs-on: ubuntu-latest
  outputs:
    rust_changed: ${{ steps.filter.outputs.rust }}
    python_changed: ${{ steps.filter.outputs.python }}
    workflow_changed: ${{ steps.filter.outputs.workflow }}
  steps:
    - uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd
      with:
        persist-credentials: false
    - id: filter
      uses: dorny/paths-filter@v3
      with:
        filters: |
          rust:
            - 'crates/**'
            - 'Cargo.toml'
            - 'Cargo.lock'
            - 'justfile'
          python:
            - 'bindings/python/**'
            - 'scripts/**'
          workflow:
            - '.github/**'
```

**Step 2: Validate YAML structure**

Run: `just workflow-quality`
Expected: workflow lint succeeds with no new findings.

**Step 3: Commit**

```bash
git add .github/workflows/ci.yaml
git commit -m "ci: add path-based change detection job"
```

### Task 2: Add CI execution summary job

**Files:**

- Modify: `.github/workflows/ci.yaml`
- Test: `.github/workflows/ci.yaml`

**Step 1: Add `ci-plan` job that depends on `changes` and writes `$GITHUB_STEP_SUMMARY`**

```yaml
ci-plan:
  needs: [changes]
  runs-on: ubuntu-latest
  steps:
    - name: Summarize CI plan
      run: |
        {
          echo "## CI Execution Plan"
          echo "- rust_changed: ${{ needs.changes.outputs.rust_changed }}"
          echo "- python_changed: ${{ needs.changes.outputs.python_changed }}"
          echo "- workflow_changed: ${{ needs.changes.outputs.workflow_changed }}"
          echo ""
          echo "Parallel lanes: rust-fmt, rust-clippy, rust-test, python-validation"
        } >> "$GITHUB_STEP_SUMMARY"
```

**Step 2: Validate workflow quality**

Run: `just workflow-quality`
Expected: success.

**Step 3: Commit**

```bash
git add .github/workflows/ci.yaml
git commit -m "ci: add ci plan summary job"
```

### Task 3: Add Rust priming stage

**Files:**

- Modify: `.github/workflows/ci.yaml`
- Test: `.github/workflows/ci.yaml`

**Step 1: Add `rust-prime` job (`needs: [changes]`) and guard it by relevant change outputs**

```yaml
rust-prime:
  needs: [changes]
  if: needs.changes.outputs.rust_changed == 'true' || needs.changes.outputs.python_changed == 'true' || needs.changes.outputs.workflow_changed == 'true'
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd
      with:
        persist-credentials: false
    - name: Set up build environment
      uses: ./.github/actions/setup-build-env
      with:
        python-version: ${{ env.PYTHON_LATEST }}
        uv-version: ${{ env.UV_VERSION }}
        just-version: ${{ env.JUST_VERSION }}
    - name: Prime Rust artifacts
      run: cargo check --workspace --all-features
```

**Step 2: Validate workflow quality**

Run: `just workflow-quality`
Expected: success.

**Step 3: Commit**

```bash
git add .github/workflows/ci.yaml
git commit -m "ci: add rust prime stage for artifact reuse"
```

### Task 4: Split Rust validation into parallel required lanes

**Files:**

- Modify: `.github/workflows/ci.yaml`
- Test: `.github/workflows/ci.yaml`

**Step 1: Replace monolithic `rust-validation` with jobs `rust-fmt`, `rust-clippy`, `rust-test`**

```yaml
rust-fmt:
  needs: [rust-prime]
  runs-on: ubuntu-latest
  steps:
    # checkout + setup-build-env
    - run: just fmt-check

rust-clippy:
  needs: [rust-prime]
  runs-on: ubuntu-latest
  steps:
    # checkout + setup-build-env
    - run: just clippy

rust-test:
  needs: [rust-prime]
  runs-on: ubuntu-latest
  steps:
    # checkout + setup-build-env
    - run: just test
```

**Step 2: Ensure skip guards mirror `rust-prime` conditions**

Run: `just workflow-quality`
Expected: no invalid dependency/if combinations.

**Step 3: Commit**

```bash
git add .github/workflows/ci.yaml
git commit -m "ci: split rust validation into fmt clippy and test lanes"
```

### Task 5: Gate Python validation on Rust priming

**Files:**

- Modify: `.github/workflows/ci.yaml`
- Test: `.github/workflows/ci.yaml`

**Step 1: Add `needs: [changes, rust-prime]` to python matrix job and path guards**

```yaml
python-validation:
  needs: [changes, rust-prime]
  if: needs.changes.outputs.python_changed == 'true' || needs.changes.outputs.rust_changed == 'true' || needs.changes.outputs.workflow_changed == 'true'
```

**Step 2: Keep doctest conditional unchanged for 3.12 lane**

Run: `just workflow-quality`
Expected: success.

**Step 3: Commit**

```bash
git add .github/workflows/ci.yaml
git commit -m "ci: gate python matrix on rust prime and change filters"
```

### Task 6: Improve Rust cache reuse settings

**Files:**

- Modify: `.github/actions/setup-build-env/action.yml`
- Test: `.github/actions/setup-build-env/action.yml`

**Step 1: Enable workspace crate caching and preserve target caching**

```yaml
- uses: Swatinem/rust-cache@779680da715d629ac1d338a641029a2f4372abb5
  with:
    shared-key: rust-shared
    cache-targets: true
    cache-workspace-crates: true
```

**Step 2: Verify workflow and action quality**

Run: `just workflow-quality`
Expected: success.

**Step 3: Commit**

```bash
git add .github/actions/setup-build-env/action.yml
git commit -m "ci: enable workspace crate caching for rust jobs"
```

### Task 7: Remove avoidable Python bootstrap from Rust tests

**Files:**

- Modify: `justfile`
- Test: `justfile`

**Step 1: Replace `uv run which python` in `test:` recipe with direct interpreter lookup suitable for CI**

```make
test:
    PYO3_PYTHON=${PYO3_PYTHON:-python3} cargo test --workspace --all-features --exclude arco-python
```

**Step 2: Verify recipe behavior locally**

Run: `just test`
Expected: Rust tests run without creating a fresh uv virtualenv in this step.

**Step 3: Commit**

```bash
git add justfile
git commit -m "ci: avoid uv bootstrap in rust test recipe"
```

### Task 8: Add final required check aggregator

**Files:**

- Modify: `.github/workflows/ci.yaml`
- Test: `.github/workflows/ci.yaml`

**Step 1: Add `ci-required` job depending on required lanes**

```yaml
ci-required:
  needs:
    - workflow-quality
    - ci-plan
    - rust-fmt
    - rust-clippy
    - rust-test
    - python-validation
  if: always()
  runs-on: ubuntu-latest
  steps:
    - name: Fail if any required dependency failed
      run: |
        # check needs.*.result and exit non-zero on failure/cancelled
```

**Step 2: Validate workflow quality and dependency graph**

Run: `just workflow-quality`
Expected: success.

**Step 3: Commit**

```bash
git add .github/workflows/ci.yaml
git commit -m "ci: add required check aggregator job"
```

### Task 9: End-to-end verification and evidence capture

**Files:**

- Modify: `docs/plans/2026-02-20-ci-compute-first.md` (append verification notes)

**Step 1: Run local verification commands**

Run in order:

1. `just workflow-quality`
2. `just fmt-check`
3. `just clippy`
4. `just test`

Expected: all commands pass.

**Step 2: Push and inspect CI evidence**

Run:

1. `gh pr checks <PR_NUMBER> --watch`
2. `gh run view <RUN_ID> --job <JOB_ID> --log`

Expected evidence:

- Rust cache restore lines show full hits in `rust-prime`, Rust lanes, and at
  least one Python lane.
- Rust lanes appear separately in PR checks.
- `ci-plan` summary is visible in run summary.

**Step 3: Commit final doc note**

```bash
git add docs/plans/2026-02-20-ci-compute-first.md
git commit -m "docs: capture ci compute-first verification evidence"
```
