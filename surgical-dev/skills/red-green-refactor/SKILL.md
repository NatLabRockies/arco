---
name: red-green-refactor
description: Enforce red-green-refactor discipline for non-trivial fixes.
license: MIT
metadata:
  version: "1.0.0"
---

# Red-Green-Refactor

## Purpose

Use this skill for any fix that changes behavior, repairs a bug, or alters control flow in a meaningful way.

## Workflow

### Red

- Reproduce the defect with a failing test, failing check, or minimal deterministic case.
- Make the failure specific enough that it proves the issue and will fail again on regression.

### Green

- Implement the smallest code change that makes the red signal pass.
- Avoid refactoring during this step unless required for the fix to compile or function.

### Refactor

- Simplify names, structure, or duplication only if behavior remains unchanged.
- Re-run validation after refactoring.
- Stop once the code is clean enough; do not turn a fix into a redesign.

## Decision Rule

If the bug cannot be demonstrated, first improve observability or isolate the invariant before editing production code.

## Deliverable

Report the red signal, the green fix, the refactor step, and the final validation evidence.
