# arco KDL for VS Code

VS Code support for arco KDL files with highlighting, diagnostics, formatting,
and format-on-save.

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

The extension does not implement a second KDL parser or formatter. It shells
out to the canonical Rust `arco` CLI, so editor diagnostics and formatting stay
aligned with command-line behavior.

## Quick install

Prerequisites:

- VS Code installed locally
- Node.js/npm
- an `arco` CLI that runs `arco --version`

One-command local install from the repository root:

```sh
npm --prefix tools/vscode-arco-kdl run install:local
```

If you use this repository's `just` workflow, the equivalent target is:

```sh
just vscode-extension-install
```

For standard VS Code installs, no manual `VSCODE_CLI` setting is required. The
installer uses `code` on PATH when available, checks common macOS app locations
such as `/Applications`, `~/Applications`, and `~/User Apps`, and falls back to
Spotlight on macOS.

Set `VSCODE_CLI` only when VS Code is installed somewhere custom or the
installer reports that it cannot run `code`:

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

## Configure the arco CLI

The extension uses one `arco` CLI command for both diagnostics and formatting.
No setting is required when `arco --version` succeeds and `arco` is discoverable
from VS Code's environment.

The extension checks in this order:

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

### Recommended settings

Use an absolute path when auto-detection does not find the CLI:

```json
{
  "arco.kdl.command": "/Users/me/.local/bin/arco",
  "arco.kdl.validateOnSave": true,
  "arco.kdl.validateOnChange": false
}
```

You can also run **arco KDL: Select arco CLI** from the Command Palette. This
updates the same `arco.kdl.command` setting.

For repository development, prefer an installed CLI or a release build that
runs successfully in the VS Code environment. Avoid explicitly setting
`arco.kdl.command` to `target/debug/arco` unless that binary runs
`target/debug/arco --version` without dynamic library errors.

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

## Troubleshooting

### VS Code installer cannot find `code`

Run the local installer again with `VSCODE_CLI` pointing at the VS Code command
inside the application bundle:

```sh
VSCODE_CLI="$HOME/User Apps/Visual Studio Code.app/Contents/Resources/app/bin/code" npm --prefix tools/vscode-arco-kdl run install:local
```

On macOS, installing the `code` command from VS Code also works:

1. Open VS Code.
2. Run **Shell Command: Install 'code' command in PATH** from the Command
   Palette.
3. Run `npm --prefix tools/vscode-arco-kdl run install:local` again.

### Validator reports missing dynamic libraries

If VS Code shows an error like this:

```text
Validator '/path/to/target/debug/arco' did not return KDL check JSON:
Library not loaded: @rpath/libscip.10.0.dylib
```

the selected `arco` binary cannot run in VS Code's environment. Point the
extension at a working CLI instead:

```json
{
  "arco.kdl.command": "/Users/me/.local/bin/arco"
}
```

or run **arco KDL: Select arco CLI** from the Command Palette and choose a
binary that passes `arco --version`.

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
