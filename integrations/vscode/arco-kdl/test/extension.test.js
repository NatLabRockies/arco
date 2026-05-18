"use strict";

const assert = require("assert/strict");
const { EventEmitter } = require("events");
const fs = require("fs");
const Module = require("module");
const os = require("os");
const path = require("path");
const test = require("node:test");

const EXTENSION_PATH = path.join(__dirname, "..", "extension.js");

function createDocument({
  fileName = "/workspace/model.kdl",
  languageId = "arco-kdl",
  text = "node\tkey=1\nsecond\n",
  version = 1,
  isUntitled = false,
} = {}) {
  const lines = text.split("\n");
  if (lines.at(-1) === "") lines.pop();
  return {
    fileName,
    languageId,
    version,
    isUntitled,
    uri: {
      fsPath: fileName,
      toString: () => `file://${fileName}`,
    },
    lineCount: Math.max(lines.length, 1),
    getText: () => text,
    lineAt(index) {
      const line = lines[Math.min(index, Math.max(lines.length - 1, 0))] || "";
      return {
        text: line,
        range: new this.vscode.Range(
          new this.vscode.Position(index, 0),
          new this.vscode.Position(index, line.length),
        ),
      };
    },
  };
}

function createVscodeMock() {
  const state = {
    activeTextEditor: undefined,
    appliedEdits: [],
    commands: new Map(),
    diagnostics: [],
    formattingProvider: undefined,
    infoMessages: [],
    openDialogSelection: undefined,
    openedTextDocument: undefined,
    shownDocument: undefined,
    warningChoice: undefined,
    warningMessages: [],
    errorMessages: [],
    configurationValues: {
      checkCommand: "",
      validateOnSave: true,
      validateOnChange: false,
    },
    configurationUpdates: [],
    workspaceFolders: new Map(),
    textDocuments: [],
    listeners: {
      changeConfiguration: [],
      changeTextDocument: [],
      openTextDocument: [],
      saveTextDocument: [],
    },
  };

  class Position {
    constructor(line, character) {
      this.line = line;
      this.character = character;
    }

    translate(lineDelta, characterDelta) {
      return new Position(this.line + lineDelta, this.character + characterDelta);
    }
  }

  class Range {
    constructor(start, end) {
      this.start = start;
      this.end = end;
    }
  }

  class Diagnostic {
    constructor(range, message, severity) {
      this.range = range;
      this.message = message;
      this.severity = severity;
    }
  }

  class WorkspaceEdit {
    constructor() {
      this.replacements = [];
    }

    replace(uri, range, newText) {
      this.replacements.push({ uri, range, newText });
    }
  }

  const vscode = {
    Position,
    Range,
    Diagnostic,
    DiagnosticSeverity: {
      Error: 0,
      Warning: 1,
    },
    ConfigurationTarget: {
      Global: "global",
    },
    TextEdit: {
      replace(range, newText) {
        return { range, newText };
      },
    },
    Uri: {
      joinPath(base, ...segments) {
        return {
          fsPath: path.join(base.fsPath, ...segments),
          toString: () => path.join(base.fsPath, ...segments),
        };
      },
    },
    WorkspaceEdit,
    commands: {
      registerCommand(name, callback) {
        state.commands.set(name, callback);
        return { dispose() {} };
      },
    },
    languages: {
      createDiagnosticCollection(source) {
        const collection = {
          source,
          set(uri, diagnostics) {
            state.diagnostics.push({ type: "set", uri, diagnostics });
          },
          delete(uri) {
            state.diagnostics.push({ type: "delete", uri });
          },
          dispose() {},
        };
        return collection;
      },
      registerDocumentFormattingEditProvider(languageId, provider) {
        state.formattingProvider = { languageId, provider };
        return { dispose() {} };
      },
    },
    window: {
      get activeTextEditor() {
        return state.activeTextEditor;
      },
      set activeTextEditor(value) {
        state.activeTextEditor = value;
      },
      showInformationMessage(message) {
        state.infoMessages.push(message);
        return Promise.resolve(undefined);
      },
      showWarningMessage(message, ...choices) {
        state.warningMessages.push({ message, choices });
        return Promise.resolve(state.warningChoice);
      },
      showErrorMessage(message) {
        state.errorMessages.push(message);
        return Promise.resolve(undefined);
      },
      showOpenDialog() {
        return Promise.resolve(state.openDialogSelection);
      },
      showTextDocument(document) {
        state.shownDocument = document;
        return Promise.resolve();
      },
    },
    workspace: {
      get textDocuments() {
        return state.textDocuments;
      },
      getConfiguration() {
        return {
          get(key, fallback) {
            return Object.hasOwn(state.configurationValues, key)
              ? state.configurationValues[key]
              : fallback;
          },
          update(key, value, target) {
            state.configurationValues[key] = value;
            state.configurationUpdates.push({ key, value, target });
            return Promise.resolve();
          },
        };
      },
      getWorkspaceFolder(uri) {
        const root = state.workspaceFolders.get(uri.toString());
        return root ? { uri: { fsPath: root } } : undefined;
      },
      applyEdit(edit) {
        state.appliedEdits.push(edit);
        return Promise.resolve(true);
      },
      openTextDocument(uri) {
        state.openedTextDocument = uri;
        return Promise.resolve({ uri });
      },
      onDidOpenTextDocument(callback) {
        state.listeners.openTextDocument.push(callback);
        return { dispose() {} };
      },
      onDidSaveTextDocument(callback) {
        state.listeners.saveTextDocument.push(callback);
        return { dispose() {} };
      },
      onDidChangeTextDocument(callback) {
        state.listeners.changeTextDocument.push(callback);
        return { dispose() {} };
      },
      onDidChangeConfiguration(callback) {
        state.listeners.changeConfiguration.push(callback);
        return { dispose() {} };
      },
    },
  };

  createDocument.prototype = { vscode };
  return { state, vscode };
}

function attachVscode(document, vscode) {
  return Object.assign(document, { vscode });
}

function createSpawnMock() {
  const calls = [];
  const queue = [];

  function spawn(command, args, options) {
    const behavior = queue.shift() || { code: 0, stdout: "" };
    const child = new EventEmitter();
    child.stdout = new EventEmitter();
    child.stderr = new EventEmitter();
    child.stdin = {
      input: "",
      end(input) {
        this.input = input;
        calls.at(-1).stdin = input;
      },
    };
    child.killCalled = false;
    child.kill = () => {
      child.killCalled = true;
    };
    calls.push({ command, args, options, child, stdin: "" });

    process.nextTick(() => {
      if (behavior.error) {
        child.emit("error", behavior.error);
        return;
      }
      if (behavior.stdout) child.stdout.emit("data", Buffer.from(behavior.stdout));
      if (behavior.stderr) child.stderr.emit("data", Buffer.from(behavior.stderr));
      child.emit("close", behavior.code ?? 0);
    });

    return child;
  }

  return {
    calls,
    queue,
    spawn,
    push(behavior) {
      queue.push(behavior);
    },
  };
}

function loadExtension({ vscodeMock, spawnMock }) {
  delete require.cache[EXTENSION_PATH];
  const originalLoad = Module._load;
  Module._load = function patchedLoad(request, parent, isMain) {
    if (request === "vscode") return vscodeMock;
    if (request === "child_process") return { spawn: spawnMock.spawn };
    return originalLoad.call(this, request, parent, isMain);
  };

  try {
    return require(EXTENSION_PATH);
  } finally {
    Module._load = originalLoad;
  }
}

function loadHarness() {
  const { state, vscode } = createVscodeMock();
  const spawnMock = createSpawnMock();
  const extension = loadExtension({ vscodeMock: vscode, spawnMock });
  return { extension, spawnMock, state, vscode };
}

function waitForTick() {
  return new Promise((resolve) => setImmediate(resolve));
}

test("activate registers commands, formatter provider, and validates open KDL documents", async () => {
  const { extension, spawnMock, state, vscode } = loadHarness();
  const document = attachVscode(createDocument(), vscode);
  state.textDocuments = [
    document,
    attachVscode(
      createDocument({ languageId: "plaintext", fileName: "/workspace/readme.txt" }),
      vscode,
    ),
  ];
  state.workspaceFolders.set(document.uri.toString(), "/workspace");
  spawnMock.push({ stdout: '{"valid":true,"diagnostics":[]}' });

  const context = { subscriptions: [] };
  extension.activate(context);
  await waitForTick();

  assert.equal(context.subscriptions.length, 10);
  assert.equal(state.formattingProvider.languageId, "arco-kdl");
  assert.deepEqual([...state.commands.keys()].sort(), [
    "arcoKdl.formatCurrentFile",
    "arcoKdl.selectCheckCommand",
    "arcoKdl.showSetup",
    "arcoKdl.validateCurrentFile",
  ]);
  assert.equal(path.basename(spawnMock.calls[0].command), "arco");
  assert.deepEqual(spawnMock.calls[0].args, [
    "kdl",
    "check",
    "/workspace/model.kdl",
    "--format",
    "json",
  ]);
  assert.deepEqual(state.diagnostics.at(-1).diagnostics, []);

  spawnMock.push({ stdout: "node key=1\n" });
  const edits = await state.formattingProvider.provider.provideDocumentFormattingEdits(
    document,
  );
  assert.equal(edits[0].newText, "node key=1\n");
  assert.equal(extension.deactivate(), undefined);
});

test("validateDocument maps CLI diagnostics to VS Code diagnostics", async () => {
  const { extension, spawnMock, state, vscode } = loadHarness();
  const document = attachVscode(createDocument({ text: "first\nsecond line\n" }), vscode);
  const diagnostics = vscode.languages.createDiagnosticCollection("arco-kdl");
  spawnMock.push({
    stdout: JSON.stringify({
      valid: false,
      diagnostics: [
        {
          line: 2,
          column: 4,
          severity: "warning",
          message: "bad field",
          code: "ARCO1",
        },
      ],
    }),
  });

  extension._test.validateDocument(document, diagnostics);
  await waitForTick();

  const item = state.diagnostics.at(-1).diagnostics[0];
  assert.equal(item.message, "bad field");
  assert.equal(item.severity, vscode.DiagnosticSeverity.Warning);
  assert.equal(item.source, "arco-kdl");
  assert.equal(item.code, "ARCO1");
  assert.equal(item.range.start.line, 1);
  assert.equal(item.range.start.character, 3);
});

test("validateDocument handles untitled documents, invalid JSON, stale results, and spawn errors", async () => {
  const { extension, spawnMock, state, vscode } = loadHarness();
  const diagnostics = vscode.languages.createDiagnosticCollection("arco-kdl");
  const untitled = attachVscode(createDocument({ isUntitled: true }), vscode);
  extension._test.validateDocument(untitled, diagnostics);
  assert.equal(state.diagnostics.at(-1).type, "delete");

  const invalid = attachVscode(createDocument(), vscode);
  spawnMock.push({ stdout: "not json", stderr: "broken" });
  extension._test.validateDocument(invalid, diagnostics);
  await waitForTick();
  assert.match(state.diagnostics.at(-1).diagnostics[0].message, /did not return KDL check JSON: broken/);

  const stale = attachVscode(createDocument({ version: 1 }), vscode);
  spawnMock.push({ stdout: '{"valid":true,"diagnostics":[]}' });
  extension._test.validateDocument(stale, diagnostics);
  stale.version = 2;
  await waitForTick();
  assert.notDeepEqual(state.diagnostics.at(-1).diagnostics, []);

  const missing = attachVscode(createDocument(), vscode);
  const enoent = new Error("missing");
  enoent.code = "ENOENT";
  spawnMock.push({ error: enoent });
  extension._test.validateDocument(missing, diagnostics);
  await waitForTick();
  assert.match(state.diagnostics.at(-1).diagnostics[0].message, /Failed to run '.+arco': missing/);
  assert.match(state.warningMessages.at(-1).message, /validator '.+arco' was not found/);
});

test("starting a second validation kills the active validation for the same URI", () => {
  const { extension, spawnMock, vscode } = loadHarness();
  const diagnostics = vscode.languages.createDiagnosticCollection("arco-kdl");
  const document = attachVscode(createDocument(), vscode);

  spawnMock.push({ stdout: '{"valid":true,"diagnostics":[]}' });
  extension._test.validateDocument(document, diagnostics);
  const firstChild = spawnMock.calls[0].child;
  spawnMock.push({ stdout: '{"valid":true,"diagnostics":[]}' });
  extension._test.validateDocument(document, diagnostics);

  assert.equal(firstChild.killCalled, true);
});

test("formatter returns a full-document edit from canonical stdin formatter", async () => {
  const { extension, spawnMock, state, vscode } = loadHarness();
  const document = attachVscode(createDocument({ text: "node\tkey=1\n" }), vscode);
  state.workspaceFolders.set(document.uri.toString(), "/workspace");
  spawnMock.push({ stdout: "node key=1\n" });

  const edits = await extension._test.formatDocument(document);

  assert.equal(edits.length, 1);
  assert.equal(edits[0].newText, "node key=1\n");
  assert.deepEqual(spawnMock.calls[0].args, [
    "kdl",
    "fmt",
    "--stdin",
    "--stdin-filename",
    "/workspace/model.kdl",
  ]);
  assert.equal(spawnMock.calls[0].options.cwd, "/workspace");
  assert.equal(spawnMock.calls[0].stdin, "node\tkey=1\n");
});

test("formatter handles unchanged, untitled, nonzero, and missing-CLI cases", async () => {
  const { extension, spawnMock, state, vscode } = loadHarness();

  const unchanged = attachVscode(createDocument({ text: "node key=1\n" }), vscode);
  spawnMock.push({ stdout: "node key=1\n" });
  assert.deepEqual(await extension._test.formatDocument(unchanged), []);

  const untitled = attachVscode(createDocument({ isUntitled: true }), vscode);
  assert.deepEqual(await extension._test.formatDocument(untitled), []);
  assert.match(state.warningMessages.at(-1).message, /Save the arco KDL file/);

  const bad = attachVscode(createDocument(), vscode);
  spawnMock.push({ code: 1, stderr: "parse failed" });
  assert.deepEqual(await extension._test.formatDocument(bad), []);
  assert.match(state.errorMessages.at(-1), /parse failed/);

  const missing = attachVscode(createDocument(), vscode);
  const enoent = new Error("missing");
  enoent.code = "ENOENT";
  spawnMock.push({ error: enoent });
  assert.deepEqual(await extension._test.formatDocument(missing), []);
  assert.match(state.warningMessages.at(-1).message, /validator '.+arco' was not found/);
});

test("formatActiveDocument applies edits and reports non-KDL editors", async () => {
  const { extension, spawnMock, state, vscode } = loadHarness();
  const document = attachVscode(createDocument({ text: "node\tkey=1\n" }), vscode);
  state.activeTextEditor = { document };
  spawnMock.push({ stdout: "node key=1\n" });

  await extension._test.formatActiveDocument();
  assert.equal(state.appliedEdits.length, 1);
  assert.equal(state.appliedEdits[0].replacements[0].newText, "node key=1\n");

  state.activeTextEditor = { document: attachVscode(createDocument({ languageId: "plaintext", fileName: "/tmp/a.txt" }), vscode) };
  await extension._test.formatActiveDocument();
  assert.match(state.infoMessages.at(-1), /Open an arco KDL file to format it/);
});

test("validateActiveDocument validates KDL editors and reports other editors", () => {
  const { extension, spawnMock, state, vscode } = loadHarness();
  const diagnostics = vscode.languages.createDiagnosticCollection("arco-kdl");
  const document = attachVscode(createDocument(), vscode);
  state.activeTextEditor = { document };
  spawnMock.push({ stdout: '{"valid":true,"diagnostics":[]}' });

  extension._test.validateActiveDocument(diagnostics);
  assert.equal(spawnMock.calls.length, 1);

  state.activeTextEditor = undefined;
  extension._test.validateActiveDocument(diagnostics);
  assert.match(state.infoMessages.at(-1), /Open an arco KDL file to validate it/);
});

test("configuration events honor validateOnSave and validateOnChange", () => {
  const { extension, spawnMock, state, vscode } = loadHarness();
  const document = attachVscode(createDocument(), vscode);
  const context = { subscriptions: [] };
  state.textDocuments = [];
  extension.activate(context);

  state.configurationValues.validateOnSave = false;
  state.listeners.saveTextDocument[0](document);
  assert.equal(spawnMock.calls.length, 0);

  state.configurationValues.validateOnSave = true;
  spawnMock.push({ stdout: '{"valid":true,"diagnostics":[]}' });
  state.listeners.saveTextDocument[0](document);
  assert.equal(spawnMock.calls.length, 1);

  state.configurationValues.validateOnChange = false;
  state.listeners.changeTextDocument[0]({ document });
  assert.equal(spawnMock.calls.length, 1);

  state.configurationValues.validateOnChange = true;
  spawnMock.push({ stdout: '{"valid":true,"diagnostics":[]}' });
  state.listeners.changeTextDocument[0]({ document });
  assert.equal(spawnMock.calls.length, 2);

  spawnMock.push({ stdout: '{"valid":true,"diagnostics":[]}' });
  state.textDocuments = [document];
  state.listeners.changeConfiguration[0]({
    affectsConfiguration(section) {
      return section === "arco.kdl";
    },
  });
  assert.equal(spawnMock.calls.length, 3);

  state.listeners.changeConfiguration[0]({
    affectsConfiguration() {
      return false;
    },
  });
  assert.equal(spawnMock.calls.length, 3);
});

test("CLI resolution prefers setting, environment, workspace binary, PATH, then fallback", () => {
  const { extension, state, vscode } = loadHarness();
  const temp = fs.mkdtempSync(path.join(os.tmpdir(), "arco-vscode-test-"));
  const workspace = path.join(temp, "workspace");
  const pathDir = path.join(temp, "bin");
  fs.mkdirSync(path.join(workspace, "target", "debug"), { recursive: true });
  fs.mkdirSync(pathDir);
  const workspaceBinary = path.join(workspace, "target", "debug", "arco");
  const pathBinary = path.join(pathDir, "arco");
  fs.writeFileSync(workspaceBinary, "");
  fs.writeFileSync(pathBinary, "");
  fs.chmodSync(workspaceBinary, 0o755);
  fs.chmodSync(pathBinary, 0o755);

  const originalPath = process.env.PATH;
  const originalArcoCli = process.env.ARCO_CLI;
  const document = attachVscode(createDocument(), vscode);
  state.workspaceFolders.set(document.uri.toString(), workspace);
  process.env.PATH = pathDir;

  state.configurationValues.checkCommand = "/configured/arco";
  assert.equal(extension._test.resolveArcoCommand(document), "/configured/arco");
  state.configurationValues.checkCommand = "";
  process.env.ARCO_CLI = "/env/arco";
  assert.equal(extension._test.resolveArcoCommand(document), "/env/arco");
  delete process.env.ARCO_CLI;
  assert.equal(extension._test.resolveArcoCommand(document), workspaceBinary);
  state.workspaceFolders.clear();
  assert.equal(extension._test.resolveArcoCommand(document), pathBinary);
  process.env.PATH = "";
  assert.equal(extension._test.resolveArcoCommand(document), "arco");
  assert.deepEqual(extension._test.candidateExecutableNames("arco", [".EXE", ".CMD"]), [
    "arco.exe",
    "arco.EXE",
    "arco.cmd",
    "arco.CMD",
  ]);

  process.env.PATH = originalPath;
  if (originalArcoCli === undefined) delete process.env.ARCO_CLI;
  else process.env.ARCO_CLI = originalArcoCli;
  fs.rmSync(temp, { recursive: true, force: true });
});

test("parser and diagnostic helpers cover valid and invalid shapes", () => {
  const { extension, vscode } = loadHarness();
  const document = attachVscode(createDocument({ text: "abc\n" }), vscode);

  assert.deepEqual(extension._test.parseReport('{"valid":true,"diagnostics":[]}'), {
    valid: true,
    diagnostics: [],
  });
  assert.equal(extension._test.parseReport("{"), null);
  assert.equal(extension._test.isKdlCheckReport({ valid: true, diagnostics: [] }), true);
  assert.equal(extension._test.isKdlCheckReport({ valid: true, diagnostics: [{}] }), false);
  assert.equal(extension._test.isKdlDiagnostic({ message: "ok" }), true);
  assert.equal(Boolean(extension._test.isKdlDiagnostic(null)), false);
  assert.equal(extension._test.isArcoKdlDocument(document), true);
  assert.equal(extension._test.isArcoKdlDocument(attachVscode(createDocument({ languageId: "plaintext", fileName: "/tmp/a.kdl" }), vscode)), true);
  assert.equal(extension._test.isArcoKdlDocument(attachVscode(createDocument({ languageId: "plaintext", fileName: "/tmp/a.txt" }), vscode)), false);

  const fallbackRange = extension._test.diagnosticRange(document, {});
  assert.equal(fallbackRange.start.line, 0);
  const clampedRange = extension._test.diagnosticRange(document, { line: 99, column: 99 });
  assert.equal(clampedRange.start.line, 0);
  assert.equal(clampedRange.start.character, 3);
  assert.match(extension._test.commandFailureDiagnostic(document, "arco", "nope").message, /Failed to run/);
  assert.match(extension._test.invalidOutputDiagnostic(document, "arco", "").message, /did not return/);
  assert.equal(extension._test.fileDiagnostic(document, "file-level").source, "arco-kdl");
  assert.equal(extension._test.fullDocumentRange(document).end.character, 3);
});

test("select CLI command and setup document update user-facing state", async () => {
  const { extension, state } = loadHarness();
  state.openDialogSelection = [{ fsPath: "/bin/arco" }];
  await extension._test.selectCheckCommand();
  assert.deepEqual(state.configurationUpdates.at(-1), {
    key: "checkCommand",
    value: "/bin/arco",
    target: "global",
  });
  assert.match(state.infoMessages.at(-1), /validator set to \/bin\/arco/);

  state.openDialogSelection = undefined;
  await extension._test.selectCheckCommand();
  assert.equal(state.configurationUpdates.length, 1);

  await extension._test.showSetupDocument();
  assert.equal(state.openedTextDocument, undefined);

  extension.activate({
    extensionUri: { fsPath: "/extension" },
    subscriptions: [],
  });
  await extension._test.showSetupDocument();
  assert.equal(state.openedTextDocument.fsPath, "/extension/README.md");
  assert.equal(state.shownDocument.uri.fsPath, "/extension/README.md");
});

test("runArcoCommand resolves stdout and rejects failures", async () => {
  const { extension, spawnMock } = loadHarness();
  spawnMock.push({ stdout: "ok", stderr: "note" });
  assert.deepEqual(await extension._test.runArcoCommand("arco", ["x"], "input", "/tmp"), {
    stdout: "ok",
    stderr: "note",
  });

  spawnMock.push({ code: 2, stdout: "bad stdout", stderr: "" });
  await assert.rejects(
    () => extension._test.runArcoCommand("arco", ["x"], "", undefined),
    /bad stdout/,
  );

  spawnMock.push({ code: 3 });
  await assert.rejects(
    () => extension._test.runArcoCommand("arco", ["x"], "", undefined),
    /exit code 3/,
  );
});

test("remaining branches cover no-op formatting, stale child errors, error diagnostics, and setup choices", async () => {
  const { extension, spawnMock, state, vscode } = loadHarness();
  const diagnostics = vscode.languages.createDiagnosticCollection("arco-kdl");

  const unchanged = attachVscode(createDocument({ text: "node key=1\n" }), vscode);
  state.activeTextEditor = { document: unchanged };
  spawnMock.push({ stdout: "node key=1\n" });
  await extension._test.formatActiveDocument();
  assert.equal(state.appliedEdits.length, 0);

  const document = attachVscode(createDocument(), vscode);
  spawnMock.push({ stdout: '{"valid":true,"diagnostics":[]}' });
  extension._test.validateDocument(document, diagnostics);
  const firstChild = spawnMock.calls.at(-1).child;
  spawnMock.push({ stdout: '{"valid":true,"diagnostics":[]}' });
  extension._test.validateDocument(document, diagnostics);
  firstChild.emit("error", new Error("late"));
  await waitForTick();
  assert.equal(firstChild.killCalled, true);

  const errorDiagnostic = extension._test.toVsCodeDiagnostic(document, {
    message: "error severity",
  });
  assert.equal(errorDiagnostic.severity, vscode.DiagnosticSeverity.Error);

  state.warningChoice = "Select CLI";
  state.openDialogSelection = [{ fsPath: "/selected/arco" }];
  extension._test.showMissingCommandWarning("arco");
  await waitForTick();
  assert.equal(state.configurationValues.checkCommand, "/selected/arco");

  state.warningChoice = "Setup Help";
  extension.activate({
    extensionUri: { fsPath: "/extension" },
    subscriptions: [],
  });
  extension._test.showMissingCommandWarning("arco");
  await waitForTick();
  assert.equal(state.openedTextDocument.fsPath, "/extension/README.md");

  state.warningChoice = "Ignore";
  extension._test.showMissingCommandWarning("arco");
  await waitForTick();
  assert.equal(state.warningMessages.length >= 3, true);
});
