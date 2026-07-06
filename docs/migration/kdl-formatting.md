# KDL Formatting Defaults

The default `arco kdl fmt` output is now Arco surface syntax. This keeps KDL
models in the authoring form used throughout the repository, including readable
algebra blocks instead of strict-KDL `formula "..."` wrappers.

## What changed

Before this change, `arco kdl fmt` emitted the KDL crate's normalized strict KDL
representation. Now it emits the Arco authoring surface by default:

```bash
arco kdl fmt path/to/model.kdl
```

Use `--kdl-compatible` when a downstream tool needs strict normalized KDL:

```bash
arco kdl fmt --kdl-compatible path/to/model.kdl
```

## CI and pre-commit checks

Existing projects that run `arco kdl fmt --check` may see failures the first
time they adopt this formatter version because committed files need one
formatting pass. Choose one path:

- Run `arco kdl fmt <paths>` once and commit the resulting formatting changes.
- Change the check to `arco kdl fmt --check --kdl-compatible <paths>` when the
  strict-KDL representation is required.

After the files are updated, `arco kdl fmt --check` remains stable and
idempotent.

## VS Code setting rename

The VS Code extension uses `arco.kdl.command` for the `arco` CLI path used by
diagnostics and formatting. Existing `arco.kdl.checkCommand` settings still
work as a legacy fallback, but new configurations should use:

```json
{
  "arco.kdl.command": "/Users/me/.local/bin/arco"
}
```

See the [VS Code extension guide](../../tools/vscode-arco-kdl/README.md) for
install, format-on-save, and troubleshooting details.

---

[Back to migration notes](README.md)
