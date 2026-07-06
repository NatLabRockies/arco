"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const test = require("node:test");

const installer = require("../scripts/install-local.js");

test("resolveCodeCommand finds VS Code under user apps on macOS", (t) => {
  if (process.platform !== "darwin") {
    t.skip("macOS app bundle discovery is only relevant on darwin");
    return;
  }

  const root = fs.mkdtempSync(path.join(os.tmpdir(), "arco-vscode-code-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));

  const home = path.join(root, "home");
  const codeCommand = path.join(
    home,
    "User Apps",
    "Visual Studio Code.app",
    "Contents",
    "Resources",
    "app",
    "bin",
    "code",
  );
  fs.mkdirSync(path.dirname(codeCommand), { recursive: true });
  fs.writeFileSync(codeCommand, "#!/bin/sh\nexit 0\n", { mode: 0o755 });

  assert.equal(
    installer._test.resolveCodeCommand({
      HOME: home,
      PATH: "",
    }),
    codeCommand,
  );
});
