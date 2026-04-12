---
name: surgical-refactor
description: Apply the smallest complete code change that resolves a single well-scoped issue.
license: MIT
metadata:
  version: "1.0.0"
---

# Surgical Refactor

## Purpose

Use this skill when a task requires a precise code correction, localized cleanup, or narrow behavioral fix.

## Procedure

1. Define the single issue in one sentence.
2. Identify the smallest set of files and symbols involved.
3. Confirm the current behavior with a failing test, compiler error, lint failure, or reproducible bug.
4. Make the narrowest possible change that fixes the issue completely.
5. Re-read adjacent code to ensure invariants and interfaces still hold.
6. Reject opportunistic cleanup unless it is required to complete the fix safely.

## Success Criteria

- One issue addressed completely.
- Minimal diff with clear justification.
- No unrelated edits.
- Validation passes after the change.

## Anti-Patterns

- Touching multiple subsystems without proof it is necessary.
- Bundling drive-by cleanup with the fix.
- Stopping after a partial mitigation instead of a complete resolution.
