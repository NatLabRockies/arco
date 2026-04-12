---
name: validation-check
description: Validate code changes before and after edits using compiler, lint, and test feedback.
license: MIT
metadata:
  version: "1.0.0"
---

# Validation Check

## Purpose

Use this skill whenever code changes are proposed or completed.

## Baseline Validation

Run the relevant pre-change checks when practical to confirm the starting state and capture existing failures.

## Required Rust Validation

- `cargo check`
- `cargo clippy --all --benches --tests --examples --all-features -- -D warnings`
- Targeted `cargo test` for the changed code path

## Workflow

1. Run baseline validation for the relevant scope.
2. Record any failures tied to the issue being fixed.
3. After the edit, re-run the same checks.
4. Treat validation as incomplete if any required command was skipped, failed, or was not relevantly scoped.
5. Report results with command, scope, and outcome.

## Reporting Format

- Command run
- Why it was chosen
- Pass/fail result
- Relevance to the fix

## Guardrails

- Do not claim success without command evidence.
- If full validation is too expensive, explain the narrower scope and residual risk.
