# KDL Tooling Contracts

Arco has one canonical KDL implementation and two editor-facing helpers. Keep
these responsibilities separate so syntax highlighting does not become a second
compiler.

## Ownership

| Tool                         | Responsibility                                                                    | Not responsible for                         |
| ---------------------------- | --------------------------------------------------------------------------------- | ------------------------------------------- |
| `crates/arco-kdl`            | Canonical parse, semantic validation, and compiler input model                    | Editor highlighting                         |
| `tools/tree-sitter-arco-kdl` | Fast editor parse for structure, highlighting, and injection ranges               | Semantic validation or full algebra parsing |
| `tools/vscode-arco-kdl`      | VS Code language registration, diagnostics, and formatting from the canonical CLI | Reimplementing KDL validation or formatting |

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

## Canonical formatting

Use the CLI for formatting:

```sh
arco kdl fmt path/to/input.kdl
```

The default formatter emits Arco surface syntax for authoring, so algebra is
rendered as readable blocks instead of quoted `formula` strings. Use
`--kdl-compatible` when a tool needs normalized strict KDL output.
Projects with existing `arco kdl fmt --check` gates may need one committed
formatting pass after adopting the Arco surface formatter, or can keep strict
KDL output with `--kdl-compatible`.

Editor integrations should request stdin formatting instead of reading or
writing files directly:

```sh
arco kdl fmt --stdin --stdin-filename path/to/input.kdl
```

## VS Code extension

The VS Code helper validates and formats files by running the canonical CLI. It
converts JSON check output into editor diagnostics and converts formatter stdout
into VS Code document edits. Its primary in-editor UX is the `arco KDL` status
bar item, which exposes validate, format, CLI selection, and setup help without
requiring users to discover commands or settings first.

The extension auto-detects the CLI from `arco.kdl.command`, `ARCO_CLI`, PATH,
common user install paths such as `~/.local/bin/arco` and `~/.cargo/bin/arco`,
then runnable workspace `target/{debug,release}` binaries. Auto-detected
commands must run `arco --version` successfully so a workspace development
binary with missing dynamic solver libraries does not shadow a working installed
CLI. `arco.kdl.checkCommand` remains a legacy fallback. If no CLI is found, it
reports a warning with setup actions instead of guessing silently.

Local install for VS Code users should stay one command:

```sh
npm --prefix tools/vscode-arco-kdl run install:local
```

The local installer should use `code` on PATH when available and auto-detect
standard VS Code app bundle locations on macOS, including `~/User Apps`.
`VSCODE_CLI` is only an escape hatch for non-standard installs.

## Checks

Use these checks for KDL tooling changes:

```sh
cargo test -p arco-cli kdl_check_json --test example_cli_commands
cd tools/tree-sitter-arco-kdl && npx tree-sitter test
scripts/check-kdl-overlay.sh
bash scripts/ci_vscode_extension_check.sh
```
