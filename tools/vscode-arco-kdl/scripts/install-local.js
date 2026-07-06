'use strict';

const { spawnSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const root = path.resolve(__dirname, '..');
const manifest = JSON.parse(fs.readFileSync(path.join(root, 'package.json'), 'utf8'));
const vsix = path.join(root, `${manifest.name}-${manifest.version}.vsix`);

function main() {
  if (!fs.existsSync(vsix)) {
    console.error(`Missing VSIX package: ${vsix}`);
    console.error('Run `npm run package` first.');
    process.exit(1);
  }

  const codeCommand = resolveCodeCommand(process.env) ?? 'code';
  const result = spawnSync(codeCommand, ['--install-extension', vsix, '--force'], {
    stdio: 'inherit',
    shell: process.platform === 'win32' && isWindowsCommandShim(codeCommand),
  });

  if (result.error) {
    console.error(`Failed to run '${codeCommand}': ${result.error.message}`);
    console.error('Set VSCODE_CLI to your VS Code CLI path if VS Code is installed in a non-standard location.');
    process.exit(1);
  }

  process.exit(result.status ?? 1);
}

function resolveCodeCommand(env) {
  const configuredCommand = env.VSCODE_CLI?.trim();
  if (configuredCommand) {
    return resolveConfiguredCommand(configuredCommand, env) ?? configuredCommand;
  }

  if (process.platform === 'win32') {
    return resolveWindowsCommand('code', env) ?? 'code';
  }

  return findOnPath('code', env) ?? findMacCodeCommand(env) ?? 'code';
}

function resolveConfiguredCommand(command, env) {
  if (isExecutableFile(command)) return command;
  if (path.basename(command) === command) {
    if (process.platform === 'win32') return resolveWindowsCommand(command, env);
    return findOnPath(command, env);
  }
  return undefined;
}

function resolveWindowsCommand(command, env) {
  if (isExecutableFile(command)) return command;

  const extensions = (env.PATHEXT || '.EXE;.CMD;.BAT').split(';');

  for (const extension of extensions) {
    const lowerCandidate = `${command}${extension.toLowerCase()}`;
    if (isExecutableFile(lowerCandidate)) return lowerCandidate;

    const upperCandidate = `${command}${extension.toUpperCase()}`;
    if (isExecutableFile(upperCandidate)) return upperCandidate;
  }

  const paths = (env.PATH || '').split(path.delimiter).filter(Boolean);
  for (const directory of paths) {
    for (const candidateName of candidateExecutableNames(command, extensions)) {
      const candidate = path.join(directory, candidateName);
      if (isExecutableFile(candidate)) return candidate;
    }
  }

  return undefined;
}

function findOnPath(command, env) {
  const paths = (env.PATH || '').split(path.delimiter).filter(Boolean);
  for (const directory of paths) {
    const candidate = path.join(directory, command);
    if (isExecutableFile(candidate)) return candidate;
  }
  return undefined;
}

function findMacCodeCommand(env) {
  if (process.platform !== 'darwin') return undefined;

  for (const candidate of macCodeCandidates(env)) {
    if (isExecutableFile(candidate)) return candidate;
  }

  return findMacCodeCommandWithSpotlight();
}

function macCodeCandidates(env) {
  const homes = [env.HOME, env.USERPROFILE].filter(Boolean);
  const appRoots = [
    '/Applications',
    ...homes.flatMap((home) => [
      path.join(home, 'Applications'),
      path.join(home, 'User Apps'),
    ]),
  ];

  return [
    ...appRoots.map((rootPath) =>
      path.join(rootPath, 'Visual Studio Code.app', 'Contents', 'Resources', 'app', 'bin', 'code'),
    ),
    ...appRoots.map((rootPath) =>
      path.join(
        rootPath,
        'Visual Studio Code - Insiders.app',
        'Contents',
        'Resources',
        'app',
        'bin',
        'code-insiders',
      ),
    ),
  ];
}

function findMacCodeCommandWithSpotlight() {
  const result = spawnSync('mdfind', ['kMDItemCFBundleIdentifier == "com.microsoft.VSCode"'], {
    encoding: 'utf8',
  });
  if (result.status !== 0) return undefined;

  for (const appPath of result.stdout.split(/\r?\n/)) {
    if (!appPath) continue;
    const candidate = path.join(appPath, 'Contents', 'Resources', 'app', 'bin', 'code');
    if (isExecutableFile(candidate)) return candidate;
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

if (require.main === module) main();

module.exports = {
  _test: {
    findMacCodeCommand,
    macCodeCandidates,
    resolveCodeCommand,
  },
};
