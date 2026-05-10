# Rust CI Validation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enforce Rust `fmt`, `clippy`, and `test` checks in GitHub CI using `just` commands for local/CI consistency.

**Architecture:** Add a focused `rust-validation` job to `.github/workflows/ci.yaml` and run canonical `just` recipes (`fmt-check`, `clippy`, `test`). Reuse the existing build environment composite action for toolchain/cache setup and keep Python CI behavior unchanged.

**Tech Stack:** GitHub Actions YAML, `just`, Rust toolchain (`cargo fmt`, `cargo clippy`, `cargo test`).

---

### Task 1: Add Rust Validation Job to CI Workflow

**Files:**

- Modify: `.github/workflows/ci.yaml`

**Step 1: Write the failing check expectation**

Add no code yet; define expected outcome:

- CI workflow should include a new job `rust-validation`.
- Job must run `just fmt-check`, `just clippy`, `just test`.

**Step 2: Verify current workflow lacks Rust validation job (RED)**

Run:

```bash
grep -n "rust-validation\|just fmt-check\|just clippy\|just test" .github/workflows/ci.yaml
```

Expected: no matches for the new job/commands.

**Step 3: Implement minimal workflow change (GREEN)**

Add `rust-validation` job with:

- `actions/checkout`
- `taiki-e/install-action@just`
- `./.github/actions/setup-build-env` (python 3.12 + UV_VERSION)
- steps executing:
  - `just fmt-check`
  - `just clippy`
  - `just test`

**Step 4: Verify workflow now contains expected job/commands**

Run:

```bash
grep -n "rust-validation\|just fmt-check\|just clippy\|just test" .github/workflows/ci.yaml
```

Expected: matches for all entries.

### Task 2: Validate Commands Locally

**Files:**

- Modify: `.github/workflows/ci.yaml` (if any follow-up adjustments)

**Step 1: Run Rust validation commands from justfile**

Run:

```bash
just fmt-check
just clippy
just test
```

Expected: all commands pass.

**Step 2: Refactor only if necessary**

If local command behavior suggests CI mismatch, make minimal YAML adjustments.

**Step 3: Re-run validation commands**

Run same three commands and confirm clean pass.

### Task 3: Final Verification and Chore Update

**Files:**

- Modify: `chores.md`

**Step 1: Verify final workflow diff**

Run:

```bash
git diff -- .github/workflows/ci.yaml
```

Expected: only intended Rust CI job changes.

**Step 2: Mark chore #2 complete**

Remove completed item from `chores.md` and renumber remaining items.

**Step 3: Final validation**

Run:

```bash
just fmt-check
just clippy
just test
```

Expected: all pass.
