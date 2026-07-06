"use strict";

const { spawn, spawnSync } = require("child_process");
const fs = require("fs");
const path = require("path");
const vscode = require("vscode");

const LANGUAGE_ID = "arco-kdl";
const DIAGNOSTIC_SOURCE = "arco-kdl";
const CONFIGURATION_SECTION = "arco.kdl";
const CHECK_ARGS = ["kdl", "check"];
const JSON_FORMAT_ARGS = ["--format", "json"];
const FORMAT_STDIN_ARGS = ["kdl", "fmt", "--stdin"];
const VERSION_ARGS = ["--version"];
const CMD_METACHARACTER_PATTERN = /[&|<>^%]/;
const COMMANDS = {
  formatCurrentFile: "arcoKdl.formatCurrentFile",
  validateCurrentFile: "arcoKdl.validateCurrentFile",
  showActions: "arcoKdl.showActions",
  selectCheckCommand: "arcoKdl.selectCheckCommand",
  showSetup: "arcoKdl.showSetup",
};

let extensionContext;
let statusBarItem;
let missingCommandWarningShown = false;
const activeValidations = new Map();
const activeDiagnosticUris = new Map();
const diagnosticReportsByUri = new Map();
const validationStateByUri = new Map();
let resolvedCommandCache;

function activate(context) {
  extensionContext = context;
  const diagnostics =
    vscode.languages.createDiagnosticCollection(DIAGNOSTIC_SOURCE);
  const validateOpenDocument = (document) =>
    validateIfKdl(document, diagnostics);
  statusBarItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Left,
    100,
  );
  statusBarItem.command = COMMANDS.showActions;
  statusBarItem.tooltip = "arco KDL actions";

  validateOpenDocuments(diagnostics);
  updateStatusBarForActiveEditor();
  context.subscriptions.push(
    diagnostics,
    statusBarItem,
    vscode.commands.registerCommand(COMMANDS.validateCurrentFile, () =>
      validateActiveDocument(diagnostics),
    ),
    vscode.commands.registerCommand(
      COMMANDS.formatCurrentFile,
      formatActiveDocument,
    ),
    vscode.commands.registerCommand(COMMANDS.showActions, () =>
      showActions(diagnostics),
    ),
    vscode.commands.registerCommand(
      COMMANDS.selectCheckCommand,
      selectCheckCommand,
    ),
    vscode.commands.registerCommand(COMMANDS.showSetup, showSetupDocument),
    vscode.window.onDidChangeActiveTextEditor(() =>
      updateStatusBarForActiveEditor(),
    ),
    vscode.languages.registerDocumentFormattingEditProvider(
      { language: LANGUAGE_ID },
      {
        provideDocumentFormattingEdits(document, _options, token) {
          return formatDocument(document, token);
        },
      },
    ),
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
      if (event.affectsConfiguration(CONFIGURATION_SECTION)) {
        invalidateResolvedArcoCommandCache();
        validateOpenDocuments(diagnostics);
      }
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

async function formatActiveDocument() {
  const document = vscode.window.activeTextEditor?.document;
  if (document && isArcoKdlDocument(document)) {
    await vscode.commands.executeCommand("editor.action.formatDocument");
    return;
  }
  vscode.window.showInformationMessage("Open an arco KDL file to format it.");
}

async function showActions(diagnostics) {
  const document = vscode.window.activeTextEditor?.document;
  const hasKdlDocument = document && isArcoKdlDocument(document);
  const activeCommand = hasKdlDocument ? displayArcoCommand(document) : "auto";
  const items = [
    ...(hasKdlDocument
      ? [
          {
            label: "$(play) Validate Current File",
            description: "Run arco kdl check",
            action: "validate",
          },
          {
            label: "$(wand) Format Current File",
            description: "Run arco kdl fmt",
            action: "format",
          },
        ]
      : []),
    {
      label: "$(terminal) Select arco CLI",
      description: activeCommand,
      action: "select",
    },
    {
      label: "$(book) Open Setup Help",
      description: "Install and troubleshooting notes",
      action: "setup",
    },
  ];

  const selection = await vscode.window.showQuickPick(items, {
    placeHolder: hasKdlDocument
      ? "arco KDL actions"
      : "Open a .kdl file to validate or format",
  });
  if (!selection) return;

  if (selection.action === "validate") validateDocument(document, diagnostics);
  else if (selection.action === "format")
    await vscode.commands.executeCommand("editor.action.formatDocument");
  else if (selection.action === "select") await selectCheckCommand();
  else if (selection.action === "setup") await showSetupDocument();
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
  setValidationState(document, { status: "checking" });

  const command = resolveArcoCommand(document);
  const args = [...CHECK_ARGS, document.fileName, ...JSON_FORMAT_ARGS];
  const unsafeArgument = windowsShellUnsafeArgument(command, args);
  if (unsafeArgument) {
    setDocumentDiagnostics(document, diagnostics, [
      {
        uri: document.uri,
        diagnostic: commandFailureDiagnostic(
          document,
          command,
          unsafeWindowsShellArgumentMessage(unsafeArgument),
        ),
      },
    ]);
    setValidationState(document, { status: "setup" });
    return;
  }
  const version = document.version;
  const child = spawn(command, args, commandSpawnOptions(command, workspaceDirectory(document)));
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
    setValidationState(document, { status: "setup" });
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
      setValidationState(document, { status: "setup" });
      return;
    }

    setValidationState(document, validationStateForDiagnostics(report.diagnostics));
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

function formatDocument(document, token) {
  if (!isArcoKdlDocument(document)) return [];

  const input = document.getText();
  const command = resolveArcoCommand(document);
  const args = [...FORMAT_STDIN_ARGS];
  if (!document.isUntitled && document.fileName) {
    args.push("--stdin-filename", document.fileName);
  }
  const unsafeArgument = windowsShellUnsafeArgument(command, args);
  if (unsafeArgument) {
    showFormatterError(command, unsafeWindowsShellArgumentMessage(unsafeArgument));
    setValidationState(document, { status: "setup" });
    return [];
  }

  return new Promise((resolve) => {
    const child = spawn(
      command,
      args,
      commandSpawnOptions(command, workspaceDirectory(document)),
    );

    let stdout = "";
    let stderr = "";
    let settled = false;
    let cancellationListener;

    const finish = (edits) => {
      if (settled) return;
      settled = true;
      cancellationListener?.dispose();
      resolve(edits);
    };

    child.stdout.on("data", (chunk) => {
      stdout += chunk.toString();
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk.toString();
    });
    child.stdin.on("error", () => {});
    child.on("error", (error) => {
      if (token?.isCancellationRequested) {
        finish([]);
        return;
      }

      showFormatterError(command, error.message);
      if (!missingCommandWarningShown && error.code === "ENOENT") {
        missingCommandWarningShown = true;
        setValidationState(document, { status: "setup" });
        showMissingCommandWarning(command);
      }
      finish([]);
    });
    child.on("close", (code) => {
      if (token?.isCancellationRequested) {
        finish([]);
        return;
      }

      if (code !== 0) {
        showFormatterError(command, formatterFailureDetail(code, stderr));
        finish([]);
        return;
      }

      if (stdout === input) {
        finish([]);
        return;
      }

      finish([vscode.TextEdit.replace(fullDocumentRange(document), stdout)]);
    });

    cancellationListener = token?.onCancellationRequested(() => {
      child.kill();
      finish([]);
    });

    child.stdin.end(input);
  });
}

function resolveArcoCommand(document) {
  const workspaceRoot = workspaceDirectory(document);
  const configuredCommand = configuredArcoCommand();
  if (configuredCommand) return configuredCommand;

  const environmentCommand = process.env.ARCO_CLI?.trim();
  if (environmentCommand) return environmentCommand;

  const cacheKey = commandResolutionCacheKey(workspaceRoot);
  if (resolvedCommandCache?.key === cacheKey) return resolvedCommandCache.command;

  const command =
    firstRunnableCommand([
      findOnPath("arco"),
      ...defaultUserArcoCandidates(),
      ...workspaceArcoCandidates(workspaceRoot),
    ]) ||
    "arco";
  resolvedCommandCache = { key: cacheKey, command };
  return command;
}

function displayArcoCommand(document) {
  const configuredCommand = configuredArcoCommand();
  if (configuredCommand) return configuredCommand;

  const environmentCommand = process.env.ARCO_CLI?.trim();
  if (environmentCommand) return environmentCommand;

  const cacheKey = commandResolutionCacheKey(workspaceDirectory(document));
  if (resolvedCommandCache?.key === cacheKey) return resolvedCommandCache.command;
  return "auto-detect";
}

function commandResolutionCacheKey(workspaceRoot) {
  return [
    process.platform,
    process.env.PATH ?? "",
    process.env.PATHEXT ?? "",
    process.env.HOME ?? "",
    process.env.USERPROFILE ?? "",
    workspaceRoot ?? "",
  ].join("\u0000");
}

function invalidateResolvedArcoCommandCache() {
  resolvedCommandCache = undefined;
}

function configuredArcoCommand() {
  return (
    configuration().get("command", "").trim() ||
    configuration().get("checkCommand", "").trim()
  );
}

function workspaceArcoCandidates(workspaceRoot) {
  if (!workspaceRoot) return [];

  const binaryName = process.platform === "win32" ? "arco.exe" : "arco";
  return [
    path.join(workspaceRoot, "target", "debug", binaryName),
    path.join(workspaceRoot, "target", "release", binaryName),
  ];
}

function defaultUserArcoCandidates() {
  const binaryName = process.platform === "win32" ? "arco.exe" : "arco";
  return [
    process.env.HOME && path.join(process.env.HOME, ".local", "bin", binaryName),
    process.env.HOME && path.join(process.env.HOME, ".cargo", "bin", binaryName),
    process.env.USERPROFILE &&
      path.join(process.env.USERPROFILE, ".local", "bin", binaryName),
    process.env.USERPROFILE &&
      path.join(process.env.USERPROFILE, ".cargo", "bin", binaryName),
  ];
}

function firstRunnableCommand(candidates) {
  const seen = new Set();
  for (const candidate of candidates) {
    if (!candidate || seen.has(candidate)) continue;
    seen.add(candidate);
    if (isRunnableArcoCommand(candidate)) return candidate;
  }
  return undefined;
}

function isRunnableArcoCommand(candidate) {
  if (!isExecutableFile(candidate)) return false;
  if (windowsShellUnsafeArgument(candidate, VERSION_ARGS)) return false;

  const result = spawnSync(candidate, VERSION_ARGS, {
    encoding: "utf8",
    timeout: 2_000,
    ...commandSpawnOptions(candidate),
  });
  return result.status === 0 && /^arco\s+\d/.test(result.stdout.trim());
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

function commandSpawnOptions(command, cwd) {
  return {
    cwd,
    shell: process.platform === "win32" && isWindowsCommandShim(command),
  };
}

function windowsShellUnsafeArgument(command, args, platform = process.platform) {
  if (platform !== "win32" || !isWindowsCommandShim(command, platform)) return "";
  return [command, ...args].find(hasCmdMetacharacter) ?? "";
}

function hasCmdMetacharacter(value) {
  return CMD_METACHARACTER_PATTERN.test(value);
}

function unsafeWindowsShellArgumentMessage(argument) {
  return `refusing to run Windows command shim with cmd.exe metacharacters in argument: ${argument}`;
}

function isWindowsCommandShim(command, platform = process.platform) {
  if (platform !== "win32") return false;
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

function setValidationState(document, state) {
  validationStateByUri.set(document.uri.toString(), state);
  updateStatusBarForActiveEditor();
}

function validationStateForDiagnostics(diagnostics) {
  if (!diagnostics.length) return { status: "ready" };

  const errors = diagnostics.filter(
    (diagnostic) => diagnostic.severity !== "warning",
  ).length;
  const warnings = diagnostics.length - errors;
  return { status: "issues", errors, warnings };
}

function updateStatusBarForActiveEditor() {
  if (!statusBarItem) return;

  const document = vscode.window.activeTextEditor?.document;
  if (!document || !isArcoKdlDocument(document)) {
    statusBarItem.hide();
    return;
  }

  const state =
    validationStateByUri.get(document.uri.toString()) ?? { status: "idle" };
  statusBarItem.text = statusBarText(state);
  statusBarItem.tooltip = statusBarTooltip(state);
  statusBarItem.show();
}

function statusBarText(state) {
  if (state.status === "checking") return "$(sync~spin) arco KDL";
  if (state.status === "setup") return "$(debug-disconnect) arco KDL";
  if (state.status === "issues") {
    const count = (state.errors ?? 0) + (state.warnings ?? 0);
    return `$(warning) arco KDL ${count}`;
  }
  if (state.status === "ready") return "$(check) arco KDL";
  return "$(circle-outline) arco KDL";
}

function statusBarTooltip(state) {
  if (state.status === "checking") return "arco KDL: checking";
  if (state.status === "setup")
    return "arco KDL: setup needed. Click for actions.";
  if (state.status === "issues") {
    const errors = state.errors ?? 0;
    const warnings = state.warnings ?? 0;
    return `arco KDL: ${errors} error(s), ${warnings} warning(s). Click for actions.`;
  }
  if (state.status === "ready")
    return "arco KDL: no issues found. Click for actions.";
  return "arco KDL actions";
}

function fullDocumentRange(document) {
  const start = new vscode.Position(0, 0);
  const lastLine = document.lineAt(document.lineCount - 1);
  return new vscode.Range(start, lastLine.rangeIncludingLineBreak.end);
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
  validationStateByUri.delete(sourceUri);
  updateStatusBarForActiveEditor();
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

function formatterFailureDetail(code, stderr) {
  const detail = stderr.trim().split(/\r?\n/).find(Boolean);
  if (detail) return detail;
  return `exited with status ${code}`;
}

function showFormatterError(command, message) {
  vscode.window.showErrorMessage(
    `arco KDL formatter '${command}' failed: ${message}`,
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
      `arco CLI '${command}' was not found. Install the arco CLI or select its path.`,
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
    "command",
    selected,
    vscode.ConfigurationTarget.Global,
  );
  invalidateResolvedArcoCommandCache();
  vscode.window.showInformationMessage(`arco CLI set to ${selected}`);
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

module.exports = {
  activate,
  deactivate,
  _test: {
    FORMAT_STDIN_ARGS,
    configuredArcoCommand,
    displayArcoCommand,
    formatDocument,
    fullDocumentRange,
    invalidateResolvedArcoCommandCache,
    resolveArcoCommand,
    statusBarText,
    statusBarTooltip,
    validationStateForDiagnostics,
    windowsShellUnsafeArgument,
  },
};
