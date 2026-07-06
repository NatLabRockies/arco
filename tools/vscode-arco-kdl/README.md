# arco KDL for VS Code

Minimal-setup VS Code support for arco KDL files.

## Features

- registers `.kdl` files as `arco-kdl`
- provides syntax highlighting
- validates open/saved KDL files with the canonical arco CLI
- formats KDL documents with the canonical arco CLI
- shows an `arco KDL` status bar action for validate, format, CLI selection,
  and setup help
- shows diagnostics from:

```sh
arco kdl check <file> --format json
```

- returns format edits from:

```sh
arco kdl fmt --stdin --stdin-filename <file>
```

The extension does not implement a second KDL parser or formatter. The Rust
tooling remains the source of truth.

## Install from this repository

Prerequisites:

- VS Code installed locally. The installer uses `code` on PATH when available
  and also detects standard macOS app locations, including `~/User Apps`.
- Node.js/npm
- arco CLI available by one of these methods:
  - installed on PATH as `arco`
  - `ARCO_CLI=/absolute/path/to/arco`
  - built in this workspace at `target/debug/arco` or `target/release/arco`

One-command local install from the repository root:

This works on macOS, Linux, and Windows.

```sh
npm --prefix tools/vscode-arco-kdl run install:local
```

If you use this repository's `just` workflow, the equivalent target is:

```sh
just vscode-extension-install
```

If VS Code is installed somewhere non-standard, set `VSCODE_CLI`:

```sh
VSCODE_CLI="/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code" npm --prefix tools/vscode-arco-kdl run install:local
```

On Windows PowerShell, for example:

```powershell
$env:VSCODE_CLI = "C:\\Program Files\\Microsoft VS Code\\bin\\code.cmd"
npm --prefix tools/vscode-arco-kdl run install:local
```

Manual install:

```sh
npm --prefix tools/vscode-arco-kdl run package
code --install-extension tools/vscode-arco-kdl/arco-kdl-vscode-0.1.0.vsix --force
```

## arco CLI discovery

No setting is required when `arco` is on PATH. The extension checks in this
order:

1. `arco.kdl.command` setting
2. `ARCO_CLI` environment variable
3. `arco` on PATH
4. common user install paths such as `~/.local/bin/arco` and `~/.cargo/bin/arco`
5. runnable workspace `target/debug/arco` or `target/release/arco`

Auto-detected commands must run `arco --version` successfully. This prevents a
workspace development binary with missing dynamic solver libraries from
shadowing a working installed CLI. If the CLI is missing, the extension shows a
warning with actions to select the CLI path or open this setup guide.

`arco.kdl.checkCommand` remains as a legacy alias for older local settings.

## In-editor actions

Open a `.kdl` file and use the `arco KDL` status bar item for the common
actions:

- validate the current file
- format the current file
- select the arco CLI if auto-detection cannot find it
- open setup help

## Formatting

Use **Format Document**, the `arco KDL` status bar item, or run
**arco KDL: Format Current File** from the Command Palette.

To format KDL files on save:

```json
{
  "[arco-kdl]": {
    "editor.defaultFormatter": "natlabrockies.arco-kdl-vscode",
    "editor.formatOnSave": true
  }
}
```

## VS Code settings

Optional settings:

```json
{
  "arco.kdl.command": "",
  "arco.kdl.validateOnSave": true,
  "arco.kdl.validateOnChange": false
}
```

Use an absolute path if auto-detection does not find the CLI. The same command
is used for diagnostics and formatting:

```json
{
  "arco.kdl.command": "/Users/me/.cargo/bin/arco"
}
```

You can also run **arco KDL: Select arco CLI** from the Command Palette.

## Verify installation

1. Open a `.kdl` file.
2. Click the `arco KDL` status bar item and choose **Validate Current File**.
3. For an invalid file, VS Code should show diagnostics from the canonical CLI.

Direct CLI sanity check:

```sh
arco kdl check path/to/input.kdl --format json
```

Expected valid output:

```json
{ "valid": true, "diagnostics": [] }
```

## Development

Syntax check extension scripts:

```sh
npm run check
```

Package only:

```sh
npm run package
```
