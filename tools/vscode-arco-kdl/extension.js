"use strict";

const { spawn } = require("child_process");
const fs = require("fs");
const path = require("path");
const vscode = require("vscode");

const LANGUAGE_ID = "arco-kdl";
const DIAGNOSTIC_SOURCE = "arco-kdl";
const CONFIGURATION_SECTION = "arco.kdl";
const CHECK_ARGS = ["kdl", "check"];
const JSON_FORMAT_ARGS = ["--format", "json"];
const COMMANDS = {
  validateCurrentFile: "arcoKdl.validateCurrentFile",
  selectCheckCommand: "arcoKdl.selectCheckCommand",
  showSetup: "arcoKdl.showSetup",
};

let extensionContext;
let missingCommandWarningShown = false;
const activeValidations = new Map();
const activeDiagnosticUris = new Map();
const diagnosticReportsByUri = new Map();

function activate(context) {
  extensionContext = context;
  const diagnostics =
    vscode.languages.createDiagnosticCollection(DIAGNOSTIC_SOURCE);
  const validateOpenDocument = (document) =>
    validateIfKdl(document, diagnostics);

  validateOpenDocuments(diagnostics);
  context.subscriptions.push(
    diagnostics,
    vscode.commands.registerCommand(COMMANDS.validateCurrentFile, () =>
      validateActiveDocument(diagnostics),
    ),
    vscode.commands.registerCommand(
      COMMANDS.selectCheckCommand,
      selectCheckCommand,
    ),
    vscode.commands.registerCommand(COMMANDS.showSetup, showSetupDocument),
    vscode.workspace.onDidOpenTextDocument(validateOpenDocument),
    vscode.workspace.onDidCloseTextDocument((document) => {
      if (isArcoKdlDocument(document))
        clearDocumentDiagnostics(document, diagnostics);
    }),
    vscode.workspace.onDidSaveTextDocument((document) => {
      if (configuration().get("validateOnSave", true))
        validateOpenDocument(document);
    }),
    vscode.workspace.onDidChangeTextDocument((event) => {
      if (configuration().get("validateOnChange", false))
        validateOpenDocument(event.document);
    }),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration(CONFIGURATION_SECTION))
        validateOpenDocuments(diagnostics);
    }),
  );
}

function deactivate() {}

function configuration() {
  return vscode.workspace.getConfiguration(CONFIGURATION_SECTION);
}

function validateOpenDocuments(diagnostics) {
  vscode.workspace.textDocuments.forEach((document) =>
    validateIfKdl(document, diagnostics),
  );
}

function validateActiveDocument(diagnostics) {
  const document = vscode.window.activeTextEditor?.document;
  if (document && isArcoKdlDocument(document)) {
    validateDocument(document, diagnostics);
    return;
  }
  vscode.window.showInformationMessage("Open an arco KDL file to validate it.");
}

function validateIfKdl(document, diagnostics) {
  if (isArcoKdlDocument(document)) validateDocument(document, diagnostics);
}

function isArcoKdlDocument(document) {
  return (
    document.languageId === LANGUAGE_ID || document.fileName.endsWith(".kdl")
  );
}

function validateDocument(document, diagnostics) {
  if (document.isUntitled) {
    clearDocumentDiagnostics(document, diagnostics);
    return;
  }

  const uri = document.uri.toString();
  activeValidations.get(uri)?.child.kill();

  const command = resolveCheckCommand(document);
  const version = document.version;
  const child = spawn(
    command,
    [...CHECK_ARGS, document.fileName, ...JSON_FORMAT_ARGS],
    {
      cwd: workspaceDirectory(document),
      shell:
        process.platform === "win32" && isWindowsCommandShim(command),
    },
  );
  activeValidations.set(uri, { child, version });

  let stdout = "";
  let stderr = "";
  let spawnFailed = false;

  child.stdout.on("data", (chunk) => {
    stdout += chunk.toString();
  });
  child.stderr.on("data", (chunk) => {
    stderr += chunk.toString();
  });
  child.on("error", (error) => {
    if (activeValidations.get(uri)?.child !== child) return;
    spawnFailed = true;
    activeValidations.delete(uri);
    setDocumentDiagnostics(document, diagnostics, [
      {
        uri: document.uri,
        diagnostic: commandFailureDiagnostic(document, command, error.message),
      },
    ]);
    if (!missingCommandWarningShown && error.code === "ENOENT") {
      missingCommandWarningShown = true;
      showMissingCommandWarning(command);
    }
  });
  child.on("close", () => {
    if (activeValidations.get(uri)?.child !== child) return;
    activeValidations.delete(uri);
    if (spawnFailed || document.version !== version) return;

    const report = parseReport(stdout);
    if (!isKdlCheckReport(report)) {
      setDocumentDiagnostics(document, diagnostics, [
        {
          uri: document.uri,
          diagnostic: invalidOutputDiagnostic(document, command, stderr),
        },
      ]);
      return;
    }

    setDocumentDiagnostics(
      document,
      diagnostics,
      report.diagnostics.map((item) => ({
        uri: diagnosticUri(document, item),
        diagnostic: toVsCodeDiagnostic(item),
      })),
    );
  });
}

function resolveCheckCommand(document) {
  const workspaceRoot = workspaceDirectory(document);
  return (
    configuration().get("checkCommand", "").trim() ||
    process.env.ARCO_CLI?.trim() ||
    (workspaceRoot && findWorkspaceArco(workspaceRoot)) ||
    findOnPath("arco") ||
    "arco"
  );
}

function findWorkspaceArco(workspaceRoot) {
  const binaryName = process.platform === "win32" ? "arco.exe" : "arco";
  return [
    path.join(workspaceRoot, "target", "debug", binaryName),
    path.join(workspaceRoot, "target", "release", binaryName),
  ].find(isExecutableFile);
}

function findOnPath(command) {
  const paths = (process.env.PATH || "").split(path.delimiter).filter(Boolean);
  const extensions =
    process.platform === "win32"
      ? (process.env.PATHEXT || ".EXE;.CMD;.BAT").split(";")
      : [""];

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
  if (process.platform !== "win32") return false;
  const extension = path.extname(command).toLowerCase();
  return extension === ".cmd" || extension === ".bat";
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
  return vscode.workspace.getWorkspaceFolder(document.uri)?.uri.fsPath;
}

function parseReport(stdout) {
  try {
    return JSON.parse(stdout);
  } catch (_error) {
    return null;
  }
}

function isKdlCheckReport(report) {
  return (
    report &&
    typeof report === "object" &&
    typeof report.valid === "boolean" &&
    Array.isArray(report.diagnostics) &&
    report.diagnostics.every(isKdlDiagnostic)
  );
}

function isKdlDiagnostic(diagnostic) {
  return (
    diagnostic &&
    typeof diagnostic === "object" &&
    typeof diagnostic.message === "string"
  );
}

function diagnosticUri(document, diagnostic) {
  if (typeof diagnostic.file === "string" && diagnostic.file.trim()) {
    const baseDirectory =
      workspaceDirectory(document) ?? path.dirname(document.fileName);
    return vscode.Uri.file(path.resolve(baseDirectory, diagnostic.file)).with({
      scheme: document.uri.scheme,
      authority: document.uri.authority,
    });
  }
  return document.uri;
}

function diagnosticRange(diagnostic) {
  if (
    !Number.isInteger(diagnostic.line) ||
    !Number.isInteger(diagnostic.column)
  ) {
    const start = new vscode.Position(0, 0);
    return new vscode.Range(start, start.translate(0, 1));
  }

  const line = Math.max(diagnostic.line - 1, 0);
  const column = Math.max(diagnostic.column - 1, 0);
  const start = new vscode.Position(line, column);
  return new vscode.Range(start, start.translate(0, 1));
}

function toVsCodeDiagnostic(diagnostic) {
  const item = new vscode.Diagnostic(
    diagnosticRange(diagnostic),
    diagnostic.message,
    diagnostic.severity === "warning"
      ? vscode.DiagnosticSeverity.Warning
      : vscode.DiagnosticSeverity.Error,
  );
  item.source = DIAGNOSTIC_SOURCE;
  item.code = diagnostic.code;
  return item;
}

function setDocumentDiagnostics(document, diagnostics, records) {
  const sourceUri = document.uri.toString();
  const previousUris = activeDiagnosticUris.get(sourceUri) ?? new Set();
  const grouped = new Map();

  for (const record of records) {
    const uri = record.uri.toString();
    const current = grouped.get(uri) ?? [];
    current.push(record.diagnostic);
    grouped.set(uri, current);
  }

  const nextUris = new Set(grouped.keys());

  for (const uri of previousUris) {
    if (nextUris.has(uri)) continue;
    const sourceReports = diagnosticReportsByUri.get(uri);
    if (!sourceReports) continue;
    sourceReports.delete(sourceUri);
    if (sourceReports.size === 0) {
      diagnosticReportsByUri.delete(uri);
    }
    publishDiagnostics(diagnostics, uri);
  }

  for (const [uri, items] of grouped) {
    const sourceReports = diagnosticReportsByUri.get(uri) ?? new Map();
    sourceReports.set(sourceUri, items);
    diagnosticReportsByUri.set(uri, sourceReports);
    publishDiagnostics(diagnostics, uri);
  }

  activeDiagnosticUris.set(sourceUri, nextUris);
}

function clearDocumentDiagnostics(document, diagnostics) {
  const sourceUri = document.uri.toString();
  const activeValidation = activeValidations.get(sourceUri);
  if (activeValidation) {
    activeValidation.child.kill();
    activeValidations.delete(sourceUri);
  }

  const previousUris = activeDiagnosticUris.get(sourceUri) ?? new Set();
  for (const uri of previousUris) {
    const sourceReports = diagnosticReportsByUri.get(uri);
    if (!sourceReports) continue;
    sourceReports.delete(sourceUri);
    if (sourceReports.size === 0) {
      diagnosticReportsByUri.delete(uri);
    }
    publishDiagnostics(diagnostics, uri);
  }
  activeDiagnosticUris.delete(sourceUri);
}

function publishDiagnostics(diagnostics, uri) {
  const sourceReports = diagnosticReportsByUri.get(uri);
  if (!sourceReports || sourceReports.size === 0) {
    diagnosticReportsByUri.delete(uri);
    diagnostics.delete(vscode.Uri.parse(uri));
    return;
  }

  const combined = [];
  const seen = new Set();
  for (const items of sourceReports.values()) {
    for (const diagnostic of items) {
      const key = diagnosticKey(diagnostic);
      if (seen.has(key)) continue;
      seen.add(key);
      combined.push(diagnostic);
    }
  }

  diagnostics.set(vscode.Uri.parse(uri), combined);
}

function diagnosticKey(diagnostic) {
  return [
    diagnostic.range.start.line,
    diagnostic.range.start.character,
    diagnostic.range.end.line,
    diagnostic.range.end.character,
    diagnostic.message,
    diagnostic.severity,
    diagnostic.code ?? "",
    diagnostic.source ?? "",
  ].join("|");
}

function commandFailureDiagnostic(document, command, message) {
  return fileDiagnostic(document, `Failed to run '${command}': ${message}`);
}

function invalidOutputDiagnostic(document, command, stderr) {
  const detail = stderr.trim() ? `: ${stderr.trim()}` : "";
  return fileDiagnostic(
    document,
    `Validator '${command}' did not return KDL check JSON${detail}`,
  );
}

function fileDiagnostic(document, message) {
  const diagnostic = new vscode.Diagnostic(
    document.lineAt(0).range,
    message,
    vscode.DiagnosticSeverity.Error,
  );
  diagnostic.source = DIAGNOSTIC_SOURCE;
  return diagnostic;
}

function showMissingCommandWarning(command) {
  vscode.window
    .showWarningMessage(
      `arco KDL validator '${command}' was not found. Install the arco CLI or select its path.`,
      "Select CLI",
      "Setup Help",
    )
    .then((choice) => {
      if (choice === "Select CLI") selectCheckCommand();
      else if (choice === "Setup Help") showSetupDocument();
    });
}

async function selectCheckCommand() {
  const selection = await vscode.window.showOpenDialog({
    canSelectFiles: true,
    canSelectFolders: false,
    canSelectMany: false,
    title: "Select the arco CLI executable",
  });
  const selected = selection?.[0]?.fsPath;
  if (!selected) return;

  await configuration().update(
    "checkCommand",
    selected,
    vscode.ConfigurationTarget.Global,
  );
  vscode.window.showInformationMessage(`arco KDL validator set to ${selected}`);
}

async function showSetupDocument() {
  if (!extensionContext) return;

  const readme = vscode.Uri.joinPath(
    extensionContext.extensionUri,
    "README.md",
  );
  const document = await vscode.workspace.openTextDocument(readme);
  await vscode.window.showTextDocument(document);
}

module.exports = { activate, deactivate };
