# arco KDL for VS Code

Minimal-setup VS Code support for arco KDL files.

## Features

- registers `.kdl` files as `arco-kdl`
- provides basic syntax highlighting
- validates open/saved KDL files with the canonical arco CLI
- shows diagnostics from:

```sh
arco kdl check <file> --format json
```

The extension does not implement a second KDL parser. The Rust parser in
`crates/arco-kdl` remains the source of truth.

## Install from this repository

Prerequisites:

- VS Code with the `code` command available on PATH
- Node.js/npm
- arco CLI available by one of these methods:
  - installed on PATH as `arco`
  - `ARCO_CLI=/absolute/path/to/arco`
  - built in this workspace at `target/debug/arco` or `target/release/arco`

One-command local install:

```sh
cd tools/vscode-arco-kdl
npm run install:local
```

If VS Code's CLI is not named `code`, set `VSCODE_CLI`:

```sh
VSCODE_CLI="/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code" npm run install:local
```

Manual install:

```sh
cd tools/vscode-arco-kdl
npm run package
code --install-extension arco-kdl-vscode-0.1.0.vsix --force
```

## arco CLI discovery

No setting is required when `arco` is on PATH. The extension checks in this
order:

1. `arco.kdl.checkCommand` setting
2. `ARCO_CLI` environment variable
3. workspace `target/debug/arco`
4. workspace `target/release/arco`
5. `arco` on PATH

If the CLI is missing, the extension shows a warning with actions to select the
CLI path or open this setup guide.

## VS Code settings

Optional settings:

```json
{
  "arco.kdl.checkCommand": "",
  "arco.kdl.validateOnSave": true,
  "arco.kdl.validateOnChange": false
}
```

Use an absolute path if auto-detection does not find the CLI:

```json
{
  "arco.kdl.checkCommand": "/Users/me/.cargo/bin/arco"
}
```

You can also run **arco KDL: Select arco CLI** from the Command Palette.

## Verify installation

1. Open a `.kdl` file.
2. Run **arco KDL: Validate Current File** from the Command Palette.
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
