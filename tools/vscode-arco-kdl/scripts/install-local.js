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

const rawCodeCommand = process.env.VSCODE_CLI || 'code';
const codeCommand =
  process.platform === 'win32'
    ? resolveWindowsCommand(rawCodeCommand) ?? rawCodeCommand
    : rawCodeCommand;
const result = spawnSync(codeCommand, ['--install-extension', vsix, '--force'], {
  stdio: 'inherit',
  shell: process.platform === 'win32' && isWindowsCommandShim(codeCommand),
});

if (result.error) {
  console.error(`Failed to run '${codeCommand}': ${result.error.message}`);
  console.error('Set VSCODE_CLI to your VS Code CLI path if `code` is not on PATH.');
  process.exit(1);
}

process.exit(result.status ?? 1);

function resolveWindowsCommand(command) {
  if (isExecutableFile(command)) return command;

  const extensions = (process.env.PATHEXT || '.EXE;.CMD;.BAT').split(';');

  for (const extension of extensions) {
    const lowerCandidate = `${command}${extension.toLowerCase()}`;
    if (isExecutableFile(lowerCandidate)) return lowerCandidate;

    const upperCandidate = `${command}${extension.toUpperCase()}`;
    if (isExecutableFile(upperCandidate)) return upperCandidate;
  }

  const paths = (process.env.PATH || '').split(path.delimiter).filter(Boolean);
  for (const directory of paths) {
    for (const candidateName of candidateExecutableNames(command, extensions)) {
      const candidate = path.join(directory, candidateName);
      if (isExecutableFile(candidate)) return candidate;
    }
  }

  return undefined;
}

function candidateExecutableNames(command, extensions) {
  return [
    ...new Set(
      extensions.flatMap((extension) => [
        `${command}${extension.toLowerCase()}`,
        `${command}${extension.toUpperCase()}`,
      ]),
    ),
  ];
}

function isWindowsCommandShim(command) {
  if (process.platform !== 'win32') return false;
  const extension = path.extname(command).toLowerCase();
  return extension === '.cmd' || extension === '.bat';
}

function isExecutableFile(candidate) {
  try {
    fs.accessSync(candidate, fs.constants.X_OK);
    return true;
  } catch (_error) {
    return false;
  }
}
