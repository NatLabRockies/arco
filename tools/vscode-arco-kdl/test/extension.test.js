"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const Module = require("node:module");
const os = require("node:os");
const path = require("node:path");
const test = require("node:test");

let configuredCommand = "";
let legacyConfiguredCommand = "";
let workspaceRoot;
const vscodeStub = createVscodeStub();
const originalLoad = Module._load;
Module._load = function loadWithVscodeStub(request, parent, isMain) {
  if (request === "vscode") return vscodeStub;
  return originalLoad.call(this, request, parent, isMain);
};
const extension = require("../extension.js");
Module._load = originalLoad;

test("formatDocument streams content through arco kdl fmt stdin", async (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "arco-vscode-format-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));

  configuredCommand = createFakeArcoCommand(root);
  const fileName = path.join(root, "input.kdl");
  const document = createDocument("node\tkey=1\n", fileName);

  const edits = await extension._test.formatDocument(document);

  assert.equal(edits.length, 1);
  assert.equal(edits[0].newText, "node key=1\n");
  assert.deepEqual(edits[0].range.start, new vscodeStub.Position(0, 0));
  assert.deepEqual(edits[0].range.end, new vscodeStub.Position(1, 0));
});

test("formatDocument returns no edits when formatter output is unchanged", async (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "arco-vscode-format-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));

  configuredCommand = createFakeArcoCommand(root);
  const fileName = path.join(root, "input.kdl");
  const document = createDocument("node key=1\n", fileName);

  const edits = await extension._test.formatDocument(document);

  assert.deepEqual(edits, []);
});

test("resolveArcoCommand skips unrunnable workspace binary", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "arco-vscode-resolve-"));
  const originalHome = process.env.HOME;
  const originalPath = process.env.PATH;
  const originalUserProfile = process.env.USERPROFILE;
  t.after(() => {
    process.env.HOME = originalHome;
    process.env.PATH = originalPath;
    if (originalUserProfile === undefined) delete process.env.USERPROFILE;
    else process.env.USERPROFILE = originalUserProfile;
    configuredCommand = "";
    legacyConfiguredCommand = "";
    workspaceRoot = undefined;
    fs.rmSync(root, { recursive: true, force: true });
  });

  configuredCommand = "";
  process.env.PATH = "";
  delete process.env.USERPROFILE;

  workspaceRoot = path.join(root, "workspace");
  const workspaceDebugBin = path.join(workspaceRoot, "target", "debug");
  fs.mkdirSync(workspaceDebugBin, { recursive: true });
  createBrokenArcoCommand(workspaceDebugBin);

  const home = path.join(root, "home");
  const userBin = path.join(home, ".local", "bin");
  fs.mkdirSync(userBin, { recursive: true });
  process.env.HOME = home;
  const userArco = createFakeArcoCommand(userBin);

  const document = createDocument("", path.join(workspaceRoot, "input.kdl"));

  assert.equal(extension._test.resolveArcoCommand(document), userArco);
});

test("configuredArcoCommand prefers current setting over legacy setting", () => {
  configuredCommand = "/opt/arco/bin/arco";
  legacyConfiguredCommand = "/legacy/arco";

  assert.equal(extension._test.configuredArcoCommand(), "/opt/arco/bin/arco");

  configuredCommand = "";
  assert.equal(extension._test.configuredArcoCommand(), "/legacy/arco");

  legacyConfiguredCommand = "";
});

test("status bar summarizes validation state", () => {
  assert.equal(
    extension._test.statusBarText({ status: "checking" }),
    "$(sync~spin) arco KDL",
  );
  assert.equal(
    extension._test.statusBarText({ status: "ready" }),
    "$(check) arco KDL",
  );
  assert.equal(
    extension._test.statusBarText({ status: "issues", errors: 1, warnings: 2 }),
    "$(warning) arco KDL 3",
  );
  assert.match(
    extension._test.statusBarTooltip({
      status: "issues",
      errors: 1,
      warnings: 2,
    }),
    /1 error\(s\), 2 warning\(s\)/,
  );
});

test("validationStateForDiagnostics counts errors and warnings", () => {
  assert.deepEqual(extension._test.validationStateForDiagnostics([]), {
    status: "ready",
  });
  assert.deepEqual(
    extension._test.validationStateForDiagnostics([
      { severity: "error" },
      { severity: "warning" },
      { severity: "error" },
    ]),
    { status: "issues", errors: 2, warnings: 1 },
  );
});

function createFakeArcoCommand(root) {
  const fakeCli = path.join(root, "fake-arco.js");
  fs.writeFileSync(
    fakeCli,
    `"use strict";
const args = process.argv.slice(2);
if (args[0] === "--version") {
  console.log("arco 0.8.1");
  process.exit(0);
}
if (args[0] !== "kdl" || args[1] !== "fmt" || args[2] !== "--stdin") {
  console.error(\`unexpected args: \${args.join(" ")}\`);
  process.exit(9);
}
if (args[3] !== "--stdin-filename" || !args[4]) {
  console.error(\`missing stdin filename: \${args.join(" ")}\`);
  process.exit(10);
}
let input = "";
process.stdin.setEncoding("utf8");
process.stdin.on("data", (chunk) => {
  input += chunk;
});
process.stdin.on("end", () => {
  process.stdout.write(input.replace(/\\t/g, " "));
});
`,
  );

  if (process.platform === "win32") {
    const command = path.join(root, "arco.cmd");
    fs.writeFileSync(command, `@"${process.execPath}" "${fakeCli}" %*\r\n`);
    return command;
  }

  const command = path.join(root, "arco");
  fs.writeFileSync(
    command,
    `#!/bin/sh
exec "${process.execPath}" "${fakeCli}" "$@"
`,
    { mode: 0o755 },
  );
  return command;
}

function createBrokenArcoCommand(root) {
  const command = path.join(
    root,
    process.platform === "win32" ? "arco.exe" : "arco",
  );
  if (process.platform === "win32") {
    fs.writeFileSync(command, "@echo dyld failure\r\n@exit /b 134\r\n");
    return command;
  }

  fs.writeFileSync(
    command,
    `#!/bin/sh
echo "dyld failure" >&2
exit 134
`,
    { mode: 0o755 },
  );
  return command;
}

function createDocument(text, fileName) {
  const lines = text.split("\n");
  return {
    fileName,
    isUntitled: false,
    languageId: "arco-kdl",
    lineCount: lines.length,
    uri: {
      fsPath: fileName,
      toString() {
        return `file://${fileName}`;
      },
    },
    getText() {
      return text;
    },
    lineAt(index) {
      const line = lines[index] ?? "";
      const includesLineBreak = index < lines.length - 1;
      const endCharacter = line.length + (includesLineBreak ? 1 : 0);
      return {
        range: new vscodeStub.Range(
          new vscodeStub.Position(index, 0),
          new vscodeStub.Position(index, line.length),
        ),
        rangeIncludingLineBreak: new vscodeStub.Range(
          new vscodeStub.Position(index, 0),
          new vscodeStub.Position(index, endCharacter),
        ),
      };
    },
  };
}

function createVscodeStub() {
  class Position {
    constructor(line, character) {
      this.line = line;
      this.character = character;
    }

    translate(lineDelta, characterDelta) {
      return new Position(
        this.line + lineDelta,
        this.character + characterDelta,
      );
    }
  }

  class Range {
    constructor(start, end) {
      this.start = start;
      this.end = end;
    }
  }

  return {
    Position,
    Range,
    TextEdit: {
      replace(range, newText) {
        return { range, newText };
      },
    },
    workspace: {
      getConfiguration() {
        return {
          get(name, fallback) {
            if (name === "command") return configuredCommand;
            if (name === "checkCommand") return legacyConfiguredCommand;
            return fallback;
          },
          update() {
            return Promise.resolve();
          },
        };
      },
      getWorkspaceFolder() {
        if (!workspaceRoot) return undefined;
        return { uri: { fsPath: workspaceRoot } };
      },
      textDocuments: [],
    },
    window: {
      activeTextEditor: undefined,
      showErrorMessage() {
        return Promise.resolve();
      },
      showInformationMessage() {
        return Promise.resolve();
      },
      showWarningMessage() {
        return Promise.resolve(undefined);
      },
    },
  };
}
