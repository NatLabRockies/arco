# KDL Tooling Contracts

Arco has one canonical KDL implementation and two editor-facing helpers. Keep
these responsibilities separate so syntax highlighting does not become a second
compiler.

## Ownership

| Tool                                | Responsibility                                                                | Not responsible for                         |
| ----------------------------------- | ----------------------------------------------------------------------------- | ------------------------------------------- |
| `crates/arco-kdl`                   | Canonical parse, semantic validation, and compiler input model                | Editor highlighting                         |
| `integrations/tree-sitter/arco-kdl` | Fast editor parse for structure, highlighting, and injection ranges           | Semantic validation or full algebra parsing |
| `integrations/vscode/arco-kdl`      | VS Code language registration, formatting, and diagnostics from canonical CLI | Reimplementing KDL validation or formatting |

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

The VS Code formatter also shells out to the canonical CLI:

```sh
arco kdl fmt --stdin --stdin-filename path/to/input.kdl
```

That keeps editor formatting behavior aligned with `arco kdl fmt` and avoids a
second formatter implementation.

Local install for VS Code users should stay one command:

```sh
cd integrations/vscode/arco-kdl
npm run install:local
```

## Checks

Use these checks for KDL tooling changes:

```sh
cargo test -p arco-cli kdl_check_json --test example_cli_commands
cd integrations/tree-sitter/arco-kdl && tree-sitter test
scripts/check-kdl-overlay.sh
cd integrations/vscode/arco-kdl && npm run check
cd integrations/vscode/arco-kdl && npm run coverage
```
