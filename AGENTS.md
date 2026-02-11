# Arco Agent Guide

Quick operating contract for humans and agents working in this repository. This
is a README-level guide, not full system documentation.

## North Star

- Think a lot before changing code. Understand the system, then act.
- Collaborate in the open. Explain intent, tradeoffs, and risks.
- Prefer correct-by-construction designs over patch-on-patch fixes.
- Prioritize performance when selecting designs, algorithms, and data
  structures.
- This is a memory-performance-focused application: trace stack and heap
  allocations, measure memory behavior, and minimize memory usage so the system
  remains accessible on low-resource computers.
- Leave the codebase simpler than you found it.

## Team Collaboration Rules

- Assume concurrent work by other agents or humans.
- Treat `git status` and `git diff` as read-only context.
- Never revert or overwrite work you did not author.
- If a command hangs for more than 5 minutes, stop, capture context, and report.
- Use `just` targets when available for build, lint, and test workflows.

## Engineering Workflow

Use red-green-refactor for all non-trivial changes:

1. Red: write or update a failing test that proves the behavior gap.
2. Green: implement the smallest correct change to pass.
3. Refactor: simplify while preserving behavior and keeping tests green.

Additional expectations:

- Start from first principles, not bandaids.
- No breadcrumbs. If you delete or move code, do not leave a comment in the old
  place. No "// moved to X", no "relocated". Just remove it.
- Research official docs before introducing new patterns or dependencies.
- Keep modules focused, remove dead code, and avoid unnecessary abstraction.
- Leave each repo better than how you found it. If something is giving a code
  smell, fix it for the next person.

## Testing Standards

- Test a lot. If behavior changes, tests must prove it.
- Prefer rigorous unit tests and end-to-end tests.
- Avoid mock-heavy tests that diverge from production behavior.
- Add regression coverage for every bug fix.
- Unless the user asks otherwise, run only tests you added or modified.
- Run broader suites when risk is high or before major integration points.

## Documentation Contract

- Document everything that affects users, operators, or contributors.
- If the documentation does not exist, the feature does not exist.
- Documentation is a live document and must ship with code changes.
- Update README, architecture notes, examples, and migration notes as needed.
- New APIs and behavior changes require updated docs in the same change.

## Rust Best Practices

- No `unwrap`, `expect`, or panic-prone paths in production Rust code.
- Model invariants with types and enums to prevent invalid states.
- Prefer `crate::` paths over `super::` for clarity.
- Avoid global mutable state, pass explicit context.
- Keep `unsafe` isolated, minimal, and documented.
- Treat warnings as errors and fix them right away.
- Always optimize hot paths and benchmark-sensitive code.
- If tests live in the same Rust module as production code, keep them at the
  bottom in `mod tests {}`.

Rust validation checklist:

1. `cargo fmt`
2. `cargo clippy --all --benches --tests --examples --all-features -- -D warnings`
3. Relevant `cargo test` or `just test` targets

## Python Style

- Use `uv` with `pyproject.toml` for dependency management and command
  execution.
- Run Python commands via `uv run` (for example, `uv run pytest`).
- If a `justfile` defines Python tasks, those targets should call `uv run`.
- Require explicit type hints and structured return types.
- Prefer dataclasses, TypedDict, or Pydantic models over loose dicts.
- Keep Python signatures to one or two positional arguments max, use
  keyword-only parameters for the rest.
- Name functions so the action on the primary object argument is explicit.
- Avoid broad `try/except` blocks and avoid catch-all exception handling.
- Let errors surface with clear types and fail fast at boundaries.
- Use async patterns for I/O-bound code and keep hot paths performant.

## Python Testing

- Use `pytest` with function-based tests and fixtures.
- Avoid class-based test organization.

## Commit Style

- Use Conventional Commits (for example, `feat:`, `fix:`, `docs:`).

## Dependency Policy

- Add dependencies only when necessary.
- Choose actively maintained libraries with strong ecosystem adoption.
- Validate maintenance, API quality, and long-term fit before adding.

## Final Handoff

Before marking work complete:

1. List what changed and why.
2. List commands/tests run and their result.
3. List documentation updates made.
4. Call out follow-ups, risks, or open questions.
