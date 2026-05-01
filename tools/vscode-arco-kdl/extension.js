'use strict';

const vscode = require('vscode');
const { spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

const LANGUAGE_ID = 'arco-kdl';
const DIAGNOSTIC_SOURCE = 'arco-kdl';
const CHECK_ARGS = ['kdl', 'check'];
const CHECK_FORMAT_ARGS = ['--format', 'json'];

let missingCommandWarningShown = false;
let extensionContext;

function activate(context) {
  extensionContext = context;
  const diagnostics = vscode.languages.createDiagnosticCollection(DIAGNOSTIC_SOURCE);
  context.subscriptions.push(diagnostics);

  const validateOpenDocument = (document) => {
    if (isArcoKdlDocument(document)) {
      validateDocument(document, diagnostics);
    }
  };

  for (const document of vscode.workspace.textDocuments) {
    validateOpenDocument(document);
  }

  context.subscriptions.push(
    vscode.commands.registerCommand('arcoKdl.validateCurrentFile', () => {
      const document = vscode.window.activeTextEditor?.document;
      if (document && isArcoKdlDocument(document)) {
        validateDocument(document, diagnostics);
      } else {
        vscode.window.showInformationMessage('Open an Arco KDL file to validate it.');
      }
    }),
    vscode.commands.registerCommand('arcoKdl.selectCheckCommand', selectCheckCommand),
    vscode.commands.registerCommand('arcoKdl.showSetup', showSetupDocument),
    vscode.workspace.onDidOpenTextDocument(validateOpenDocument),
    vscode.workspace.onDidSaveTextDocument((document) => {
      if (configuration().get('validateOnSave', true)) {
        validateOpenDocument(document);
      }
    }),
    vscode.workspace.onDidChangeTextDocument((event) => {
      if (configuration().get('validateOnChange', false)) {
        validateOpenDocument(event.document);
      }
    }),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration('arco.kdl')) {
        for (const document of vscode.workspace.textDocuments) {
          validateOpenDocument(document);
        }
      }
    }),
  );
}

function deactivate() {}

function configuration() {
  return vscode.workspace.getConfiguration('arco.kdl');
}

function isArcoKdlDocument(document) {
  return document.languageId === LANGUAGE_ID || document.fileName.endsWith('.kdl');
}

function validateDocument(document, diagnostics) {
  if (document.isUntitled) {
    diagnostics.delete(document.uri);
    return;
  }

  const resolved = resolveCheckCommand(document);
  const child = spawn(resolved.command, [...CHECK_ARGS, document.fileName, ...CHECK_FORMAT_ARGS], {
    cwd: workspaceDirectory(document),
    shell: false,
  });

  let stdout = '';
  let stderr = '';
  let spawnFailed = false;
  child.stdout.on('data', (chunk) => {
    stdout += chunk.toString();
  });
  child.stderr.on('data', (chunk) => {
    stderr += chunk.toString();
  });
  child.on('error', (error) => {
    spawnFailed = true;
    diagnostics.set(document.uri, [commandFailureDiagnostic(document, resolved.command, error.message)]);
    if (!missingCommandWarningShown && error.code === 'ENOENT') {
      missingCommandWarningShown = true;
      showMissingCommandWarning(resolved.command);
    }
  });
  child.on('close', () => {
    if (spawnFailed) {
      return;
    }

    const report = parseReport(stdout);
    if (!report) {
      diagnostics.set(document.uri, [invalidOutputDiagnostic(document, resolved.command, stderr)]);
      return;
    }

    diagnostics.set(
      document.uri,
      report.diagnostics.map((diagnostic) => toVsCodeDiagnostic(document, diagnostic)),
    );
  });
}

function resolveCheckCommand(document) {
  const configured = configuration().get('checkCommand', '').trim();
  if (configured) {
    return { command: configured, source: 'setting' };
  }

  const envCommand = process.env.ARCO_CLI?.trim();
  if (envCommand) {
    return { command: envCommand, source: 'ARCO_CLI' };
  }

  const workspaceRoot = workspaceDirectory(document);
  const workspaceCommand = workspaceRoot ? findWorkspaceArco(workspaceRoot) : undefined;
  if (workspaceCommand) {
    return { command: workspaceCommand, source: 'workspace' };
  }

  const pathCommand = findOnPath('arco');
  if (pathCommand) {
    return { command: pathCommand, source: 'PATH' };
  }

  return { command: 'arco', source: 'fallback' };
}

function findWorkspaceArco(workspaceRoot) {
  const binaryName = process.platform === 'win32' ? 'arco.exe' : 'arco';
  const candidates = [
    path.join(workspaceRoot, 'target', 'debug', binaryName),
    path.join(workspaceRoot, 'target', 'release', binaryName),
  ];
  return candidates.find(isExecutableFile);
}

function findOnPath(command) {
  const paths = (process.env.PATH || '').split(path.delimiter).filter(Boolean);
  const extensions = process.platform === 'win32'
    ? (process.env.PATHEXT || '.EXE;.CMD;.BAT').split(';')
    : [''];

  for (const directory of paths) {
    for (const extension of extensions) {
      const candidate = path.join(directory, `${command}${extension.toLowerCase()}`);
      if (isExecutableFile(candidate)) {
        return candidate;
      }
      const upperCandidate = path.join(directory, `${command}${extension.toUpperCase()}`);
      if (isExecutableFile(upperCandidate)) {
        return upperCandidate;
      }
    }
  }

  return undefined;
}

function isExecutableFile(candidate) {
  try {
    fs.accessSync(candidate, fs.constants.X_OK);
    return true;
  } catch (_error) {
    return false;
  }
}

function workspaceDirectory(document) {
  const folder = vscode.workspace.getWorkspaceFolder(document.uri);
  return folder ? folder.uri.fsPath : undefined;
}

function parseReport(stdout) {
  try {
    return JSON.parse(stdout);
  } catch (_error) {
    return null;
  }
}

function toVsCodeDiagnostic(document, diagnostic) {
  const line = Math.max((diagnostic.line || 1) - 1, 0);
  const column = Math.max((diagnostic.column || 1) - 1, 0);
  const range = document.lineAt(Math.min(line, document.lineCount - 1)).range;
  const start = range.start.translate(0, Math.min(column, range.end.character));
  const end = start.translate(0, 1);
  const severity = diagnostic.severity === 'warning'
    ? vscode.DiagnosticSeverity.Warning
    : vscode.DiagnosticSeverity.Error;
  const item = new vscode.Diagnostic(new vscode.Range(start, end), diagnostic.message, severity);
  item.source = DIAGNOSTIC_SOURCE;
  item.code = diagnostic.code;
  return item;
}

function commandFailureDiagnostic(document, command, message) {
  return fileDiagnostic(document, `Failed to run '${command}': ${message}`);
}

function invalidOutputDiagnostic(document, command, stderr) {
  const detail = stderr.trim() ? `: ${stderr.trim()}` : '';
  return fileDiagnostic(document, `Validator '${command}' did not return KDL check JSON${detail}`);
}

function fileDiagnostic(document, message) {
  const range = document.lineAt(0).range;
  const diagnostic = new vscode.Diagnostic(range, message, vscode.DiagnosticSeverity.Error);
  diagnostic.source = DIAGNOSTIC_SOURCE;
  return diagnostic;
}

function showMissingCommandWarning(command) {
  vscode.window
    .showWarningMessage(
      `Arco KDL validator '${command}' was not found. Install the Arco CLI or select its path.`,
      'Select CLI',
      'Setup Help',
    )
    .then((choice) => {
      if (choice === 'Select CLI') {
        selectCheckCommand();
      } else if (choice === 'Setup Help') {
        showSetupDocument();
      }
    });
}

async function selectCheckCommand() {
  const selection = await vscode.window.showOpenDialog({
    canSelectFiles: true,
    canSelectFolders: false,
    canSelectMany: false,
    title: 'Select the Arco CLI executable',
  });
  const selected = selection?.[0]?.fsPath;
  if (!selected) {
    return;
  }

  await configuration().update('checkCommand', selected, vscode.ConfigurationTarget.Global);
  vscode.window.showInformationMessage(`Arco KDL validator set to ${selected}`);
}

async function showSetupDocument() {
  if (!extensionContext) {
    return;
  }

  const readme = vscode.Uri.joinPath(extensionContext.extensionUri, 'README.md');
  const document = await vscode.workspace.openTextDocument(readme);
  await vscode.window.showTextDocument(document);
}

module.exports = {
  activate,
  deactivate,
};
