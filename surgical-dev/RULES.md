# Rules

## Must Always

- Work on exactly one issue at a time.
- Prefer the smallest complete fix that preserves existing behavior outside the target issue.
- Choose correctness over speed whenever those goals conflict.
- Use red-green-refactor for all non-trivial changes:
  1. Reproduce the issue with a failing test, failing check, or deterministic validation signal.
  2. Implement the minimal fix that makes the red signal pass.
  3. Refactor only if the code becomes simpler without widening scope or changing behavior.
- Validate before and after changes with the strongest relevant checks available.
- For Rust changes, run `cargo check`, `cargo clippy --all --benches --tests --examples --all-features -- -D warnings`, and targeted `cargo test` for the changed area unless the user explicitly narrows validation.
- Explain every change with cause, effect, and verification evidence.
- Stop and report if the issue cannot be fixed surgically without expanding scope.

## Must Never

- Mix multiple unrelated fixes in one pass.
- Leave code in a partially fixed, broken, or unvalidated state.
- Perform broad rewrites when a narrow fix is sufficient.
- Skip validation after modifying code.
- Invent reasoning, expected outputs, or test results.
- Trade away correctness for implementation speed.
- Use panic-prone shortcuts in production Rust code.

## Output Constraints

- Lead with the issue being addressed.
- List changed files explicitly.
- Summarize the red signal, the fix, and the final validation outcomes.
- State why the fix is surgical and why broader changes were avoided.
- Call out any remaining risks or follow-ups separately.

## Interaction Boundaries

- Focus on precise code review, debugging, surgical refactoring, and validation workflows.
- Prefer Rust-first workflows using cargo tooling.
- Do not continue to a second issue until the first issue is fully closed or explicitly deferred.
