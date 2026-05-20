# KDL Tooling Contracts

Arco has one canonical KDL implementation and two editor-facing helpers. Keep
these responsibilities separate so syntax highlighting does not become a second
compiler.

## Ownership

| Tool                         | Responsibility                                                       | Not responsible for                         |
| ---------------------------- | -------------------------------------------------------------------- | ------------------------------------------- |
| `crates/arco-kdl`            | Canonical parse, semantic validation, and compiler input model       | Editor highlighting                         |
| `tools/tree-sitter-arco-kdl` | Fast editor parse for structure, highlighting, and injection ranges  | Semantic validation or full algebra parsing |
| `tools/vscode-arco-kdl`      | VS Code language registration and diagnostics from the canonical CLI | Reimplementing KDL validation               |

## Canonical validation

Use the CLI for machine-readable validation:

```sh
arco kdl check path/to/input.kdl --format json
```

The command exits `0` when valid and non-zero when invalid. JSON output has this
shape:

```json
{
  "valid": false,
  "diagnostics": [
    {
      "file": "path/to/input.kdl",
      "line": 12,
      "column": 8,
      "severity": "error",
      "message": "unsupported declaration `technology` in path/to/input.kdl",
      "code": "arco::source::unsupported_declaration",
      "help": "remove the declaration or add parser support for it"
    }
  ]
}
```

`line` and `column` are one-based when the canonical parser has a source span.
Some semantic diagnostics currently report file-level errors and omit location.

By default, `arco kdl check` is a structural validation command. It parses KDL,
resolves semantic declarations, and returns editor-friendly diagnostics without
building the full lowered algebraic problem.

Use `--materialize-data` when the check should compile far enough to load
CSV-backed parameter values used by objectives and constraints:

```bash
arco kdl check path/to/input.kdl --format json --materialize-data
```

This mode catches data-contract errors that depend on full CSV parameter
materialization, such as a missing value column used only by an objective,
or an invalid numeric value in a parameter column, without invoking a solver.

`arco inspect --json` reports both declaration counts and expanded instance
counts in `meta.counts`. Use `variable_instances`, `constraint_instances`, and
`coefficient_instances` for pre-solve size checks; `variable` and `constraint`
count KDL declarations. It also reports `meta.memory.sparse_matrix_bytes`, a
conservative sparse-matrix allocation estimate based on value, index, and
column-pointer bytes. Treat it as a no-solve memory signal, not as a guarantee
of the exact allocation behavior of a concrete solver backend.

## Tree-sitter overlay

The tree-sitter grammar is intentionally permissive. It parses KDL structure and
Arco algebra-bearing blocks, then exposes algebra bodies as opaque
`arco_math_text` for editor injections. It must accept supported surface syntax,
including projection-reduce blocks such as:

```kdl
expression investment_by_area_tech {
  reduce "ai" {
    sum "investment"
  }
}
```

It must not duplicate semantic checks from `crates/arco-kdl`.

## VS Code extension

The VS Code helper validates files by running the canonical CLI and converting
JSON diagnostics into editor diagnostics. The extension auto-detects the CLI
from `arco.kdl.checkCommand`, `ARCO_CLI`, workspace `target/{debug,release}`
binaries, then PATH. If no CLI is found, it reports a warning with setup actions
instead of guessing silently.

Local install for VS Code users should stay one command:

```sh
cd tools/vscode-arco-kdl
npm run install:local
```

## Checks

Use these checks for KDL tooling changes:

```sh
cargo test -p arco-cli kdl_check_json --test example_cli_commands
cd tools/tree-sitter-arco-kdl && npx tree-sitter test
scripts/check-kdl-overlay.sh
cd tools/vscode-arco-kdl && npm run check
```
