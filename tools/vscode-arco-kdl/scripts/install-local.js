'use strict';

const { spawnSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const root = path.resolve(__dirname, '..');
const manifest = JSON.parse(fs.readFileSync(path.join(root, 'package.json'), 'utf8'));
const vsix = path.join(root, `${manifest.name}-${manifest.version}.vsix`);

if (!fs.existsSync(vsix)) {
  console.error(`Missing VSIX package: ${vsix}`);
  console.error('Run `npm run package` first.');
  process.exit(1);
}

const codeCommand = process.env.VSCODE_CLI || 'code';
const result = spawnSync(codeCommand, ['--install-extension', vsix, '--force'], {
  stdio: 'inherit',
});

if (result.error) {
  console.error(`Failed to run '${codeCommand}': ${result.error.message}`);
  console.error('Set VSCODE_CLI to your VS Code CLI path if `code` is not on PATH.');
  process.exit(1);
}

process.exit(result.status ?? 1);
