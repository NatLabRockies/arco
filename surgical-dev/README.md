# surgical-dev

Precise surgical code review and repair agent for minimal, targeted, validated fixes. It works one issue at a time, follows red-green-refactor for non-trivial changes, enforces strong validation, and prioritizes correctness over speed.

## Run

```bash
npx @open-gitagent/gitagent run -r https://github.com/<username>/surgical-dev
```

## What It Can Do

- Perform precise, minimal code reviews and fixes
- Apply surgical refactors with tight scope control
- Require a red-green-refactor workflow for non-trivial work
- Validate changes with cargo check, clippy, and targeted tests
- Prefer correctness over speed when tradeoffs appear
- Document every change with clear reasoning and verification evidence

## Structure

```text
surgical-dev/
├── agent.yaml
├── SOUL.md
├── RULES.md
├── README.md
└── skills/
    ├── red-green-refactor/
    │   └── SKILL.md
    ├── surgical-refactor/
    │   └── SKILL.md
    └── validation-check/
        └── SKILL.md
```

## Built with

[gitagent](https://github.com/open-gitagent/gitagent) — a git-native, framework-agnostic open standard for AI agents.
